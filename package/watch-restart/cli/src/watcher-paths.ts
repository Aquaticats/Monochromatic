import {
  relative,
  resolve,
  sep,
} from 'node:path';

/**
 Wraps `path.resolve` for use with `Array.prototype.map`.
 Module-scope so the closure does not capture and the lint stays clean.
 
 @param p - path to resolve
 
 @returns absolute path
 */
function resolveOne(p: string,): string {
  return resolve(p,);
}

/**
 Resolves and sorts watch roots so the deepest match wins when a file
 lives under nested overlapping roots.
 
 @param paths - watch roots from {@link WatcherOptions.paths}
 
 @returns roots resolved to absolute paths, sorted longest-first
 
 @example
 ```ts
 sortRootsByLengthDesc(['src', 'src/server',],); // ['/abs/src/server', '/abs/src']
 ```
 */
export function sortRootsByLengthDesc(
  paths: readonly string[],
): readonly string[] {
  /**
   Mutable absolute-path copy of the input; sorted in-place by descending length below.
   */
  const copy: string[] = paths.map(function mapResolve(p,) {
    return resolveOne(p,);
  },);
  copy.sort(function byLengthDesc(
    a,
    b,
  ): number {
    return b.length
      - a
        .length;
  },);
  return copy;
}

/**
 Tests whether `absPath` lives inside (or equals) `root`.
 
 @param root - absolute directory path (no trailing separator required)
 
 @param absPath - absolute path to test
 
 @returns true when `absPath === root` or `absPath` is a child of `root`
 
 @example
 ```ts
 isPathUnderRoot({ root: '/abs/src', absPath: '/abs/src/index.ts', },); // true
 isPathUnderRoot({ root: '/abs/src', absPath: '/abs/srcZ/index.ts', },); // false
 ```
 */
export function isPathUnderRoot(
  {
    root,
    absPath,
  }: {
    readonly root: string;
    readonly absPath: string;
  },
): boolean {
  if (absPath === root)
    return true;
  /**
   Root with a trailing separator guaranteed, so `startsWith` cannot match sibling roots like `/abs/srcZ` against `/abs/src`.
   */
  const prefix = root.endsWith(sep,) ? root : root + sep;
  return absPath.startsWith(prefix,);
}

/**
 Computes `absPath` relative to the deepest matching root in
 `resolvedRoots`, falling back to `absPath` itself when none match.
 The fallback keeps the watcher defensive (chokidar should not emit for
 paths outside the watched roots) without surfacing an absent root upward.
 
 @param resolvedRoots - absolute roots, sorted deepest-first by {@link sortRootsByLengthDesc}
 
 @param absPath - absolute event path
 
 @returns path relative to its matching root, or `absPath` when none match
 
 @example
 ```ts
 relativePathForRoots({ resolvedRoots: ['/abs/src',], absPath: '/abs/src/a.ts', },); // 'a.ts'
 ```
 */
export function relativePathForRoots(
  {
    resolvedRoots,
    absPath,
  }: {
    readonly resolvedRoots: readonly string[];
    readonly absPath: string;
  },
): string {
  /**
   Deepest matching root; `.find` yields it or none, so absence stays a local concern.
   */
  const root = resolvedRoots.find(function isParent(candidate,) {
    return isPathUnderRoot({
      root: candidate,
      absPath,
    },);
  },);
  if (root === undefined)
    return absPath;
  return relative(
    root,
    absPath,
  );
}
