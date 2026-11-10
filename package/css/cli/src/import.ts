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
  readTextFileSync,
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
          source: asCssSource(readTextFileSync(resolvedPath,),),
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
