import { resolve, } from 'node:path';

/**
 Glob metacharacters that mark where a static prefix ends.
 */
const GLOB_META_CHARS = '*?{[';

/**
 Result of splitting a glob into matcher root, matcher suffix, and original prefix.
 */
export type GlobSplit = readonly [
  cwd: string,
  relativeGlob: string,
  originalPrefix: string,
];

/**
 Returns the index of the first glob metacharacter in `s`, or `-1`
 when no metacharacters are present. Exported for direct unit testing.
 
 Single left-to-right pass over UTF-16 code units. The returned index
 feeds `slice` in {@link splitGlob}, so positions are measured in code
 units (via `charAt`), matching `slice`, rather than code points; this
 keeps astral characters before a metacharacter from shifting the split.
 O(n) time, O(1) stack, no recursion (stack-safe under engines without
 tail-call elimination).
 
 @param s - candidate glob pattern
 
 @returns first metacharacter index
 
 @example
 ```ts
 firstGlobMetaIndex('src/*.ts');
 firstGlobMetaIndex('src/index.ts');
 ```
 */
export function firstGlobMetaIndex(s: string,): number {
  for (let cursorIndex = 0; cursorIndex < s
    .length; cursorIndex += 1) {
    if (GLOB_META_CHARS.includes(s.charAt(cursorIndex,),))
      return cursorIndex;
  }
  return -1;
}

/**
 Splits a glob pattern into a static base directory, a relative glob suffix,
 and the original static prefix string for reconstructing output paths.
 Everything before the first wildcard segment, located via {@link firstGlobMetaIndex},
 becomes `cwd`; the remainder becomes the pattern passed to the matcher.
 
 @param pattern - Glob pattern, absolute or relative.
 
 @returns Tuple of `[resolvedCwd, relativeGlob, originalPrefix]`.
 
 @example
 ```ts
 splitGlob('/tmp/foo/*.ts');
 splitGlob('./src/a/**​/*.md');
 ```
 */
export function splitGlob(pattern: string,): GlobSplit {
  /**
   Position of the first metacharacter.
   */
  const metaIndex = firstGlobMetaIndex(pattern,);

  if (metaIndex === (-1)) {
    return [
      resolve(pattern,),
      '',
      pattern,
    ];
  }

  /**
   Static prefix up to last `/` before first metacharacter.
   */
  const staticPrefix = pattern.slice(
    0,
    metaIndex,
  );
  /**
   Index of last separator in static prefix.
   */
  const lastSep = staticPrefix.lastIndexOf('/',);

  if (lastSep === (-1)) {
    return [
      resolve('.',),
      pattern,
      '.',
    ];
  }

  /**
   Original prefix as written in pattern.
   */
  const originalPrefix = staticPrefix.slice(
    0,
    lastSep,
  );
  return [
    resolve(originalPrefix,),
    pattern.slice(lastSep + 1,),
    originalPrefix,
  ];
}
