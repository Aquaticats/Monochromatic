/**
 Monorepo-aware `\@import` inlining over the css-edit CST.
 
 Resolution strategy (mirrors CSS conventions):
 1. Relative paths (`./foo.css`, `../bar.css`): resolved against the importer
 2. Bare specifiers (`mixin.css`): tried as relative first (CSS treats these as relative)
 3. Package specifiers (`\@scope/pkg/path.css`, `pkg/path.css`): resolved via
    `node_modules` traversal, checking `exports` then `style`/`main` fields
 */
import {
  dirname,
  isAbsolute,
  resolve,
} from '@monochromatic-dev/fs-path-core/ts';
import {
  type CssAtRule,
  type CssStylesheet,
  isCssAtRule,
  isTokenString,
  isTokenURL,
  asCssSource,
  parseCss,
  rawTextOfTokens,
  tokenData,
  transformStylesheet,
} from '@monochromatic-dev/css-core-edit/ts';
import {
  existsSync,
  readCssFileSync,
} from './fs.ts';
import { resolvePackage, } from './package-resolver.ts';
import { isPackageSpecifier, } from './specifier.ts';

//region Specifier extraction

/**
 Extracts the import target from an `\@import` prelude using parsed token
 data: the first string token (covers `'x.css'` and `url("x.css")`) or URL
 token (covers unquoted `url(x.css)`) wins, so trailing conditions such as
 `layer(base)` or media queries never corrupt the specifier.
 
 @param node - Import at-rule.
 
 @returns Unescaped, unquoted specifier text.
 
 @throws When the prelude carries no string or url() target.
 */
function importSpecifier(node: CssAtRule,): string {
  for (const token of node.preludeTokens) {
    if (isTokenString(token,) || isTokenURL(token,))
      return tokenData(token,)
        .value;
  }
  throw new Error(
    `@import needs a string or url() target: '@import ${
      rawTextOfTokens({ tokens: node.preludeTokens, },)
        .trim()
    }'`,
  );
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

  // Could be bare-local (CSS convention) or a package specifier.
  // Try relative first: CSS treats `@import 'foo.css'` as relative.
  if ((!isPackageSpecifier(specifier,))
    || ((!specifier.includes('/',)) && (!specifier.startsWith('@',))))
    {
    /**
     Attempt to resolve as relative path
     */
    const asRelative = resolve(
      [
        fromDir,
        specifier,
      ],
    );
    if (existsSync(asRelative,))
      return asRelative;
  }

  // Package specifier
  return resolvePackage({
    specifier,
    fromDir,
  },);
}

//endregion Import resolution

//region Inlining

/**
 Recursively inlines `\@import` rules: each specifier resolves via
 {@link resolveSpecifier}, the file is read synchronously (in-memory registry
 first, `node:fs` fallback), parsed, recursively inlined, and spliced in
 place of the `\@import` node. Already-imported files (tracked in `imported`)
 splice to nothing, preventing cycles and duplicates.
 
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
       Absolute path of the imported file.
       */
      const resolvedPath = resolveSpecifier({
        specifier: importSpecifier(node,),
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
          source: asCssSource(readCssFileSync(resolvedPath,),),
        },)
          .root,
        fromFile: resolvedPath,
        imported,
      },);

      return importedRoot.children;
    },
    pruneTriviaBeforeRemoved: true,
  },);
}

//endregion Inlining
