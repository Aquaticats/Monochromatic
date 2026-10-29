/**
 POSIX path operations delegating to `node:path/posix`, selected by
 `package.json` `imports` under the `node` condition of `#posix-path`
 (Node and Bun, and every bundler building for the `node` platform).
 Exports the same names with the same signatures as `posix-path.neutral.ts`,
 so the public entry is identical across both builds and the neutral
 declarations describe both.

 @module
 */

import { posix, } from 'node:path';

/**
 POSIX path separator.
 */
export const sep = '/';

/**
 Whether a path starts with `/`, per `node:path/posix`.

 @param filePath - Path to check

 @returns True when the path is absolute

 @example
 ```ts
 isAbsolute('/foo/bar'); // true
 ```
 */
export function isAbsolute(filePath: string,): boolean {
  return posix.isAbsolute(filePath,);
}

/**
 Normalizes a path per `node:path/posix`.

 @param filePath - Raw path to normalize

 @returns Normalized path

 @example
 ```ts
 normalize('/foo/bar//baz/./qux/../quux'); // '/foo/bar/baz/quux'
 ```
 */
export function normalize(filePath: string,): string {
  return posix.normalize(filePath,);
}

/**
 Returns the directory portion of a POSIX path per `node:path/posix`.

 @param filePath - POSIX path

 @returns Parent directory path

 @example
 ```ts
 dirname('/foo/bar/baz.ts'); // '/foo/bar'
 ```
 */
export function dirname(filePath: string,): string {
  return posix.dirname(filePath,);
}

/**
 Joins path segments per `node:path/posix`.

 @param segments - Path segments to join

 @returns Joined and normalized path

 @example
 ```ts
 join(['/foo', 'bar', 'baz']); // '/foo/bar/baz'
 ```
 */
export function join(segments: readonly string[],): string {
  return posix.join(...segments,);
}

/**
 Resolves a sequence of paths to an absolute path per `node:path/posix`.

 @param segments - Path segments to resolve

 @returns Absolute, normalized path

 @example
 ```ts
 resolve(['/foo', 'bar', 'baz']); // '/foo/bar/baz'
 ```
 */
export function resolve(segments: readonly string[],): string {
  return posix.resolve(...segments,);
}
