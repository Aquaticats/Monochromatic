/**
 Glob resolution for task-depends item resolution.
 
 Splits glob patterns into base directories and relative suffixes,
 then expands them to absolute file paths using `tiny-readdir-glob`.
 
 @module
 */

import { resolve, } from 'node:path';

import readdirGlob from 'tiny-readdir-glob';

//region Glob resolution

/**
 Glob metacharacters that mark where a static prefix ends.
 */
const GLOB_META_CHARS = '*?{[';

/**
 Returns the index of the first glob metacharacter in `s`, or `-1`
 when none are present.
 
 Exported for direct equivalence testing of the linear scan.
 
 @param s - candidate glob pattern
 
 @returns first metacharacter index
 
 @example
 ```ts
 firstGlobMetaIndex('src/*.ts'); // 4
 firstGlobMetaIndex('src/index.ts'); // -1
 ```
 
 @internal
 */
export function firstGlobMetaIndex(s: string,): number {
  // Single linear pass; returns at the first metacharacter, or -1 once the cursor runs past the end.
  for (let cursorIndex = 0; cursorIndex < s
    .length; cursorIndex += 1) {
    if (GLOB_META_CHARS.includes(s.charAt(cursorIndex,),))
      return cursorIndex;
  }
  return -1;
}

/**
 Splits a glob pattern into a base directory and a relative glob suffix,
 using {@link firstGlobMetaIndex} to locate the first wildcard.
 
 Everything before the first wildcard segment becomes the `cwd`;
 the remainder becomes the pattern passed to the matcher.
 
 @param pattern - Glob pattern, absolute or relative
 
 @returns Tuple of `[resolvedCwd, relativeGlob]`
 
 @example
 ```ts
 splitGlob('/tmp/foo/*.ts') // ['/tmp/foo', '*.ts']
 splitGlob('src/**') // ['/abs/path/src', '**']
 ```
 */
function splitGlob(pattern: string,): readonly [
  cwd: string,
  relativeGlob: string,
] {
  /**
   Position of the first wildcard character; `-1` means the pattern is a literal path.
   */
  const metaIndex = firstGlobMetaIndex(pattern,);

  if (metaIndex === (-1)) {
    return [
      resolve(pattern,),
      '',
    ];
  }

  /**
   Literal portion of the pattern preceding the first wildcard; the matcher's `cwd` is derived from it.
   */
  const staticPrefix = pattern.slice(
    0,
    metaIndex,
  );
  /**
   Last `/` inside the static prefix; splits the directory `cwd` from the remaining glob suffix.
   */
  const lastSep = staticPrefix.lastIndexOf('/',);

  if (lastSep === (-1)) {
    return [
      resolve('.',),
      pattern,
    ];
  }

  return [
    resolve(staticPrefix.slice(
      0,
      lastSep,
    ),),
    pattern.slice(lastSep + 1,),
  ];
}

/**
 Resolves a glob pattern into file paths using `tiny-readdir-glob`.
 
 @param pattern - Glob pattern to expand
 
 @returns Array of matched absolute file paths
 
 @example
 ```ts
 const files = await resolveGlobFiles('src/*.ts');
 ```
 */
export async function resolveGlobFiles(pattern: string,): Promise<string[]> {
  /**
   Base directory and relative glob suffix produced by {@link splitGlob}.
   */
  const [cwd, relativeGlob,] = splitGlob(pattern,);

  if (relativeGlob === '')
    return [cwd,];

  /**
   Files matched by `tiny-readdir-glob`; rebound here so destructuring carries TSDoc above the const block.
   */
  const { files, } = await readdirGlob(
    relativeGlob,
    { cwd, },
  );
  return files;
}

//endregion Glob resolution
