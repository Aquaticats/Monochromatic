/**
 What the built extension bundle asks for at run time, and which of those it cannot answer alone.

 The bundle is loaded by Pi from this package's `dist`, so a specifier it leaves unbundled is resolved from this package's own
 `node_modules` on the machine that loads it. On 2026-11-11 an extension change added a workspace dependency, the extension was
 rebuilt in the main checkout before its dependencies were linked, and rolldown reported `[UNRESOLVED_IMPORT] ... treating it as
 an external dependency` and finished successfully. Every Pi started in that session then ended with `Failed to load extension
 ...: Cannot find module '@monochromatic-dev/async-time-core/ts'`.

 @module
 */

/**
 The prefix node's own modules are named with, which a bundle may always ask for.
 */
const BUILTIN_PREFIX = 'node:';

/**
 The prefixes that mean "a file beside the bundle", which need no package on the machine.
 */
const RELATIVE_PREFIXES: readonly string[] = [
  '.',
  '/',
];

/**
 What one import statement asks for, of a shape a bundle's text can be read for.

 The patterns are the four ways a module asks for another one: a static import, a re-export, a bare import for its side effect,
 and a call to `import()` or `require()`. They are read from the text rather than from a parse tree, so a specifier a string
 literal happens to spell the same way would be read as an import; a bundle is minified into very few lines and this check refuses
 builds, so the price of a false positive is a message a developer reads, not a wrong build.
 */
const IMPORT_PATTERNS: readonly RegExp[] = [
  // oxlint-disable-next-line no-restricted-syntax/no-regex -- the bundle is minified output whose import syntax has a fixed shape, and a scan of it is linear: each pattern anchors on a keyword and reads to the next quote or newline.
  /\bfrom\s*(?<open>["'])(?<specifier>[^"'\n]+)\k<open>/gu,
  // oxlint-disable-next-line no-restricted-syntax/no-regex -- a static import for its side effect is the same shape as the line above.
  /\bimport\s*\(\s*(?<open>["'])(?<specifier>[^"'\n]+)\k<open>/gu,
  // oxlint-disable-next-line no-restricted-syntax/no-regex -- a CommonJS require in a bundle rolldown left alone has one shape.
  /\brequire\s*\(\s*(?<open>["'])(?<specifier>[^"'\n]+)\k<open>/gu,
  // oxlint-disable-next-line no-restricted-syntax/no-regex -- a bare import is the one shape the patterns above do not read, and a minified bundle puts it after a statement boundary.
  /(?:^|[;}])[\s;]*import\s*(?<open>["'])(?<specifier>[^"'\n]+)\k<open>/gu,
];

/**
 Says whether a specifier names something a bundle has to find on the machine that loads it.

 @param specifier - what an import statement asks for

 @returns whether it names a package rather than a file beside the bundle or one of node's own modules

 @example
 ```ts
 const needsAPackage = asksForAPackage('@monochromatic-dev/async-time-core/ts');
 ```
 */
function asksForAPackage(specifier: string,): boolean {
  if (specifier.startsWith(BUILTIN_PREFIX,)) {
    return false;
  }
  return !RELATIVE_PREFIXES.some(function isAPath(prefix,) {
    return specifier.startsWith(prefix,);
  },);
}

/**
 Reads every specifier the bundle asks for at run time that is not a file beside it or one of node's own modules.

 @param bundle - the built bundle's text

 @returns the specifiers, each once, in the order the bundle names them

 @example
 ```ts
 const lodged = lodgedImportsIn({ bundle: await readFile('./dist/final/node/index.mjs', 'utf8') });
 ```
 */
export function lodgedImportsIn({ bundle, }: { readonly bundle: string },): readonly string[] {
  /**
   Every specifier found with where the bundle names it, so the answer reads in the bundle's own order.
   */
  const found: {
    readonly at: number;
    readonly specifier: string
  }[] = [];
  for (const pattern of IMPORT_PATTERNS) {
    for (const match of bundle.matchAll(pattern,)) {
      /**
       What this statement asks for.
       */
      const specifier = match.groups?.specifier ?? '';
      if ((specifier !== '') && asksForAPackage(specifier,)) {
        found.push({
          at: match.index,
          specifier,
        },);
      }
    }
  }
  return found.toSorted(function byWhereItIs(
    left: {
      readonly at: number;
      readonly specifier: string
    },
    right: {
      readonly at: number;
      readonly specifier: string
    },
  ) {
    return left.at - right.at;
  },)
    .flatMap(function once(specifier,) {
      return [specifier.specifier,];
    },)
    .filter(function firstTime(
      specifier: string,
      index: number,
      all: readonly string[],
    ) {
      return all.indexOf(specifier,) === index;
    },);
}
