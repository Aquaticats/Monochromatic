/**
 Segment-wise prefix and equality predicates over TOML paths, shared by the
 resolver and the set/delete mutators.
 
 @module
 */

import type { TomlPath, } from './types.ts';

/**
 Whether `candidate` is a prefix of `path` (equal length allowed).
 
 @param candidate - Segments tested as a prefix.
 
 @param path - Full path the prefix is compared against.
 
 @returns Whether every `candidate` segment matches `path` and it is no longer.
 
 @example
 ```ts
 isPrefix({ candidate: ['a'], path: ['a', 'b'], },); // true
 ```
 */
export function isPrefix(
  {
    candidate,
    path,
  }: {
    readonly candidate: readonly (string | number)[];
    readonly path: TomlPath;
  },
): boolean {
  return (candidate.length
    <= path.length)
    && candidate.every(function eq(
      seg,
      i,
    ) {
      return seg === path[i];
    },);
}

/**
 Whether `candidate` is a strict prefix of `path` (shorter, and matching).
 
 @param candidate - Segments tested as a strict prefix.
 
 @param path - Full path the prefix is compared against.
 
 @returns Whether `candidate` matches and is strictly shorter than `path`.
 
 @example
 ```ts
 isStrictPrefix({ candidate: ['a'], path: ['a'], },); // false
 ```
 */
export function isStrictPrefix(
  {
    candidate,
    path,
  }: {
    readonly candidate: readonly (string | number)[];
    readonly path: TomlPath;
  },
): boolean {
  return (candidate.length
    < path.length)
    && candidate.every(function eq(
      seg,
      i,
    ) {
      return seg === path[i];
    },);
}

/**
 Whether `left` and `right` are equal segment-wise (same length).
 
 @param left - First segment list.
 
 @param right - Second segment list.
 
 @returns Whether both have identical length and segments.
 
 @example
 ```ts
 segmentsEqual({ left: ['a', 'b'], right: ['a', 'b'], },); // true
 ```
 */
export function segmentsEqual(
  {
    left,
    right,
  }: {
    readonly left: readonly (string | number)[];
    readonly right: TomlPath;
  },
): boolean {
  return (left.length
    === right.length)
    && left.every(function eq(
      seg,
      i,
    ) {
      return seg === right[i];
    },);
}
