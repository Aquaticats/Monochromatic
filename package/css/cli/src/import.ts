/**
 Monorepo-aware `\@import` inlining over the css-edit CST.
 
 Resolution strategy (mirrors CSS URL resolution):
 1. Absolute paths (`/foo.css`): used as written
 2. Every other specifier, `./foo.css`, `foo.css`, `sub/foo.css`, and
    `\@scope/pkg/foo.css` alike: probed as a URL relative to the importing
    stylesheet first, which is how CSS reads them
 3. Package specifiers (`\@scope/pkg/path.css`, `pkg/path.css`): when no file
    sits beside the importer at that path, resolved via `node_modules`
    traversal, checking `exports` then `style`/`main` fields
 */
import {
  dirname,
  isAbsolute,
  resolve,
} from '@monochromatic-dev/fs-path-core/ts';
import {
  type CSSToken,
  type CssAtRule,
  type CssStylesheet,
  isClosingToken,
  isCssAtRule,
  isTokenString,
  isTokenURL,
  isTriviaToken,
  asCssSource,
  parseCss,
  rawTextOfTokens,
  stringifyNodes,
  tokenData,
  transformStylesheet,
} from '@monochromatic-dev/css-core-edit/ts';
import {
  existsSync,
  readTextFileSync,
} from './fs.ts';
import { resolvePackage, } from './package-resolver.ts';
import { isPackageSpecifier, } from './specifier.ts';

//region Specifier extraction

/**
 What an `\@import` prelude names: the file to inline and the conditions the
 inlined rules must stay under.
 */
type ImportPrelude = {
  /**
   Unescaped, unquoted target text.
   */
  readonly specifier: string;
  /**
   Text after the target: a media query list, `layer(...)`, `supports(...)`,
   or nothing at all.
   */
  readonly conditionText: string;
};

/**
 Index of the prelude token naming the imported file: the first string token
 (covers `'x.css'` and `url("x.css")`) or URL token (covers unquoted
 `url(x.css)`), so trailing conditions never corrupt the specifier.
 
 @param preludeTokens - Token slice between the at-keyword and the `;`.
 
 @returns Index of the target token, or -1 when the prelude names none.
 */
function importTargetIndex(preludeTokens: readonly CSSToken[],): number {
  return preludeTokens.findIndex(function isImportTarget(token,) {
    return isTokenString(token,) || isTokenURL(token,);
  },);
}

/**
 Unquoted text an import target token carries.
 
 @param token - Token at {@link importTargetIndex}.
 
 @returns Unescaped, unquoted specifier text.
 
 @throws When the token is neither a string nor a url() token.
 */
function importTargetText(token: CSSToken,): string {
  if (isTokenString(token,) || isTokenURL(token,))
    return tokenData(token,)
      .value;
  throw new Error(`@import target must be a string or url() token: '${token[1]}'`,);
}

/**
 Tokens a prelude carries from a start index, or none when the index is the
 -1 a failed search reports.
 
 @param tokens - Token slice to cut.
 
 @param startIndex - Index the kept slice begins at.
 
 @returns Tokens from the start index to the end, empty for -1.
 */
function tokensAfterIndex({
  tokens,
  startIndex,
}: {
  readonly tokens: readonly CSSToken[];
  readonly startIndex: number;
},): readonly CSSToken[] {
  if (startIndex === (-1))
    return [];
  return tokens.slice(startIndex,);
}

/**
 Reads the target and the trailing conditions of an `\@import` prelude.
 Conditions are everything after the target and after the parenthesis that
 closes a `url("x.css")` form; `layer(base)`, `supports(...)`, and a media
 query list all sit in that trailing text.
 
 @param node - Import at-rule.
 
 @returns Target text plus the condition text trailing it.
 
 @throws When the prelude carries no string or url() target.
 */
function importPrelude(node: CssAtRule,): ImportPrelude {
  /**
   Index of the token naming the imported file.
   */
  const targetIndex = importTargetIndex(node.preludeTokens,);
  if (targetIndex === (-1)) {
    throw new Error(
      `@import needs a string or url() target: '@import ${
        rawTextOfTokens({ tokens: node.preludeTokens, },)
          .trim()
      }'`,
    );
  }

  /**
   Target token itself; the index above guarantees its presence.
   */
  const targetToken = node.preludeTokens[targetIndex];
  if (targetToken === undefined)
    throw new Error(`@import target token missing at index ${targetIndex}`,);

  /**
   First token after the target that carries meaning: trivia and the closing
   parenthesis of the target's own `url()` form are skipped.
   */
  const conditionStart = node.preludeTokens
    .findIndex(function startsCondition(
      token,
      index,
    ) {
      return (index > targetIndex)
        && (!isTriviaToken(token,))
        && (!isClosingToken(token,));
    },);

  /**
   Tokens carrying the trailing conditions, empty when the import has none.
   */
  const conditionTokens = tokensAfterIndex({
    tokens: node.preludeTokens,
    startIndex: conditionStart,
  },);

  return {
    specifier: importTargetText(targetToken,),
    conditionText: rawTextOfTokens({ tokens: conditionTokens, },)
      .trim(),
  };
}

//endregion Specifier extraction

//region Import resolution

/**
 Resolves a CSS \@import specifier to an absolute file path, falling back to
 {@link resolvePackage} for package specifiers.
 
 @param specifier - Bare import specifier (quotes/url() already stripped)
 
 @param fromFile - Absolute path of the importing file
 
 @returns Absolute path to the resolved CSS file
 
 @throws When the specifier cannot be resolved
 
 @example
 ```ts
 resolveSpecifier({
   specifier: './tokens.css',
   fromFile: '/project/src/main.css',
 }); // → '/project/src/tokens.css'
 ```
 */
function resolveSpecifier({
  specifier,
  fromFile,
}: {
  readonly specifier: string;
  readonly fromFile: string;
},): string {
  /**
   Directory containing the importing file
   */
  const fromDir = dirname(fromFile,);

  // Absolute path (rare but possible)
  if (isAbsolute(specifier,)) {
    if (existsSync(specifier,))
      return specifier;
    throw new Error(`CSS @import absolute path not found: '${specifier}'`,);
  }

  // Explicit relative path
  if (specifier.startsWith('.',)) {
    /**
     Resolved absolute path from relative specifier
     */
    const resolved = resolve(
      [
        fromDir,
        specifier,
      ],
    );
    if (existsSync(resolved,))
      return resolved;
    throw new Error(
      `CSS @import relative path not found: '${specifier}' from '${fromFile}'`,
    );
  }

  // Every remaining specifier could be a URL relative to the importing
  // stylesheet or a package reference. CSS resolves `@import 'sub/theme.css'`
  // against the importer, so the relative candidate is probed for all of
  // them, not only for specifiers without a slash.
  /**
   The specifier read as a path relative to the importing file.
   */
  const asRelative = resolve(
    [
      fromDir,
      specifier,
    ],
  );
  if (existsSync(asRelative,))
    return asRelative;

  // No such file beside the importer: a package specifier names one inside
  // a node_modules directory.
  if (isPackageSpecifier(specifier,)) {
    return resolvePackage({
      specifier,
      fromDir,
    },);
  }

  // Neither a relative file nor a package reference: the specifier is
  // outside every form this resolver knows, so refuse it rather than guess.
  throw new Error(`CSS @import specifier not found: '${specifier}' from '${fromFile}'`,);
}

//endregion Import resolution

//region Import conditions

/**
 One at-rule wrapping the rules an `\@import` pulls in, so a conditioned
 import inlines under the condition the author wrote.
 */
type ConditionWrapper = {
  /**
   Wrapper at-rule name: `layer`, `supports`, or `media`.
   */
  readonly name: string;
  /**
   Wrapper prelude; empty for the anonymous `layer` form.
   */
  readonly params: string;
};

/**
 Sentinel returned by {@link readParenthesizedGroup} when the text does not
 start with a balanced group. A unique Symbol encodes genuine absence
 without a banned nullish union; identity comparison narrows the result back
 to the group.
 */
const NO_PARENTHESIZED_GROUP = Symbol('import condition text opens no balanced parenthesized group',);

/**
 Sentinel returned by {@link readLayerCondition} when no layer condition
 leads the text.
 */
const NO_LAYER_CONDITION = Symbol('layer keyword absent from the import prelude trailing text',);

/**
 Sentinel returned by {@link readSupportsCondition} when no supports
 condition leads the text.
 */
const NO_SUPPORTS_CONDITION = Symbol('supports keyword absent from the import prelude trailing text',);

/**
 Reads a balanced parenthesized group from the start of a text.
 
 @param text - Text starting with `(`.
 
 @returns Group contents without its parentheses plus the text after the
 group, or {@link NO_PARENTHESIZED_GROUP} when no balanced group starts the
 text.
 */
function readParenthesizedGroup(text: string,): {
  readonly content: string;
  readonly rest: string;
} | typeof NO_PARENTHESIZED_GROUP {
  for (let depth = 0, index = 0; index < text.length; index += 1) {
    /**
     Character the depth scan reads.
     */
    const character = text.charAt(index,);
    if (character === '(')
      depth += 1;
    else if (character === ')') {
      depth -= 1;
      // Depth returns to zero: the group that opened at the start closes here.
      if (depth === 0) {
        return {
          content: text.slice(
            1,
            index,
          ),
          rest: text.slice(index + 1,),
        };
      }
    }
  }
  return NO_PARENTHESIZED_GROUP;
}

/**
 Whether text starts with a whole keyword rather than a longer identifier.
 
 @param text - Text to inspect.
 
 @param keyword - Keyword to look for.
 
 @returns Whether the keyword stands alone at the start of the text.
 */
function startsWithWholeKeyword({
  text,
  keyword,
}: {
  readonly text: string;
  readonly keyword: string;
},): boolean {
  if (!text.startsWith(keyword,))
    return false;

  /**
   Character directly after the keyword, empty at the end of the text.
   */
  const nextCharacter = text.charAt(keyword.length,);
  return (nextCharacter === '')
    || (nextCharacter === '(')
    || (nextCharacter.trim() === '');
}

/**
 Reads a leading `layer` or `layer(<name>)` import condition.
 
 @param text - Trailing import condition text.
 
 @returns Layer name (empty for the anonymous form) plus the condition text
 after it, or {@link NO_LAYER_CONDITION} when no layer condition leads the
 text.
 */
function readLayerCondition(text: string,): {
  readonly name: string;
  readonly rest: string;
} | typeof NO_LAYER_CONDITION {
  if (!startsWithWholeKeyword({
    text,
    keyword: 'layer',
  },))
    return NO_LAYER_CONDITION;

  /**
   Text after the keyword, before any space between them.
   */
  const afterKeyword = text.slice('layer'.length,);
  if (!afterKeyword.startsWith('(',)) {
    return {
      name: '',
      rest: afterKeyword.trim(),
    };
  }

  /**
   Layer name and the condition text after its group.
   */
  const group = readParenthesizedGroup(afterKeyword,);
  if (group === NO_PARENTHESIZED_GROUP)
    return NO_LAYER_CONDITION;

  /**
   Layer name inside the parentheses, trimmed of the space around it.
   */
  const name = group.content
    .trim();
  /**
   Condition text after the layer group.
   */
  const rest = group.rest
    .trim();
  return {
    name,
    rest,
  };
}

/**
 Reads a leading `supports(<condition>)` import condition.
 
 @param text - Trailing import condition text.
 
 @returns `@supports` prelude plus the condition text after it, or
 {@link NO_SUPPORTS_CONDITION} when no supports condition leads the text.
 */
function readSupportsCondition(text: string,): {
  readonly params: string;
  readonly rest: string;
} | typeof NO_SUPPORTS_CONDITION {
  if (!startsWithWholeKeyword({
    text,
    keyword: 'supports',
  },))
    return NO_SUPPORTS_CONDITION;

  /**
   Parenthesized supports condition and the text after it.
   */
  const group = readParenthesizedGroup(text.slice('supports'.length,),);
  if (group === NO_PARENTHESIZED_GROUP)
    return NO_SUPPORTS_CONDITION;

  /**
   Supports condition text inside the parentheses.
   */
  const condition = group.content
    .trim();
  /**
   Supports condition wrapped back in the parentheses the wrapper emits.
   */
  const params = `(${condition})`;
  /**
   Condition text after the supports group.
   */
  const rest = group.rest
    .trim();
  return {
    params,
    rest,
  };
}

/**
 Splits trailing import conditions into the at-rules that must wrap the
 inlined rules, in the order CSS reads them: `layer`, then `supports`, then a
 media query list.
 
 @param conditionText - Trailing import condition text.
 
 @returns Wrapper at-rules outermost first; empty for an unconditional import.
 */
function conditionWrappers(conditionText: string,): ConditionWrapper[] {
  if (conditionText === '')
    return [];

  /**
   Layer condition leading the text, when one does.
   */
  const layer = readLayerCondition(conditionText,);
  if (layer !== NO_LAYER_CONDITION) {
    return [
      {
        name: 'layer',
        params: layer.name,
      },
      ...conditionWrappers(layer.rest,),
    ];
  }

  /**
   Supports condition leading the text, when one does.
   */
  const supports = readSupportsCondition(conditionText,);
  if (supports !== NO_SUPPORTS_CONDITION) {
    return [
      {
        name: 'supports',
        params: supports.params,
      },
      ...conditionWrappers(supports.rest,),
    ];
  }

  return [
    {
      name: 'media',
      params: conditionText,
    },
  ];
}

/**
 Renders one wrapper at-rule around a body of CSS text.
 
 @param wrapper - Wrapper at-rule to emit.
 
 @param body - CSS text the wrapper encloses.
 
 @returns At-rule text with the body inside its block.
 */
function wrapperText({
  wrapper,
  body,
}: {
  readonly wrapper: ConditionWrapper;
  readonly body: string;
},): string {
  /**
   Space-separated prelude, empty for the anonymous layer form.
   */
  const prelude = wrapper.params === '' ? '' : ` ${wrapper.params}`;
  /**
   Imported rules without the blank space their file ended with, so the
   wrapper closes on the line after its last rule.
   */
  const enclosedBody = body.trim();
  if (enclosedBody === '')
    return `@${wrapper.name}${prelude} {}`;
  return `@${wrapper.name}${prelude} {\n${enclosedBody}\n}`;
}

/**
 Wraps inlined CSS text in the at-rules the import conditions name.
 
 @param css - Inlined CSS text.
 
 @param wrappers - Wrapper at-rules outermost first.
 
 @returns CSS text enclosed by every wrapper, innermost condition deepest.
 */
function wrapInConditions({
  css,
  wrappers,
}: {
  readonly css: string;
  readonly wrappers: readonly ConditionWrapper[];
},): string {
  return wrappers.reduceRight(
    function wrapOne(
      innerCss,
      wrapper,
    ) {
      return wrapperText({
        wrapper,
        body: innerCss,
      },);
    },
    css,
  );
}

//endregion Import conditions

//region Inlining

/**
 Recursively inlines `\@import` rules: each specifier resolves via
 {@link resolveSpecifier}, the file is read synchronously (in-memory registry
 first, `node:fs` fallback), parsed, recursively inlined, and spliced in
 place of the `\@import` node. Conditions the at-rule carried, such as a
 media query, `layer(base)`, or `supports(...)`, wrap the spliced rules in the
 matching at-rule instead of disappearing. Already-imported files (tracked in
 `imported`) splice to nothing, preventing cycles and duplicates.
 
 @param root - Parsed stylesheet to process.
 
 @param fromFile - Absolute path of the file the stylesheet came from.
 
 @param imported - Absolute paths already inlined; mutated as files inline.
 
 @returns Stylesheet with every import replaced by its file's contents.
 
 @example
 ```ts
 inlineCssImports({
   root: parsed.root,
   fromFile: '/project/src/main.css',
   imported: new Set(['/project/src/main.css']),
 });
 ```
 */
export function inlineCssImports({
  root,
  fromFile,
  imported,
}: {
  readonly root: CssStylesheet;
  readonly fromFile: string;
  readonly imported: Set<string>;
},): CssStylesheet {
  return transformStylesheet({
    root,
    visit: function inlineImportNode(node,) {
      if ((!isCssAtRule(node,)) || (node.name !== 'import'))
        return node;

      /**
       Target the at-rule names plus the conditions trailing it.
       */
      const prelude = importPrelude(node,);
      /**
       Absolute path of the imported file.
       */
      const resolvedPath = resolveSpecifier({
        specifier: prelude.specifier,
        fromFile,
      },);

      // Already-inlined files splice to nothing: prevents circular imports
      // and duplicate content.
      if (imported.has(resolvedPath,))
        return [];
      imported.add(resolvedPath,);

      /**
       Imported file parsed and recursively inlined.
       */
      const importedRoot = inlineCssImports({
        root: parseCss({
          source: asCssSource(readTextFileSync(resolvedPath,),),
        },)
          .root,
        fromFile: resolvedPath,
        imported,
      },);

      // An unconditional import splices its rules in place; a conditioned one
      // keeps the condition by wrapping them in the at-rules it named.
      if (prelude.conditionText === '')
        return importedRoot.children;

      return parseCss({
        source: asCssSource(wrapInConditions({
          css: stringifyNodes({ nodes: importedRoot.children, },),
          wrappers: conditionWrappers(prelude.conditionText,),
        },),),
      },)
        .root
        .children;
    },
    pruneTriviaBeforeRemoved: true,
  },);
}

//endregion Inlining
