/**
 Confinement of renderer-supplied listing paths to the configured root.
 
 The renderer runs sandboxed with context isolation on and Node integration
 off, so the main process is the only place a listing request can be refused.
 
 @example
 ```ts
 await confinePath({ path: '/home/docs', rootDirectory: '/home' });
 ```
 
 @packageDocumentation
 */

import {
  resolve,
  sep,
} from 'node:path';

/**
 Whether `candidate` is `root` itself or lies below it.
 
 @param candidate - Absolute path to test.
 
 @param root - Absolute root path to test against.
 
 @returns Whether the candidate is the root or a descendant of it.
 
 @example
 ```ts
 isWithinRoot({ candidate: '/home/docs', root: '/home' });
 ```
 */
function isWithinRoot(
  {
    candidate,
    root,
  }: {
    readonly candidate: string;
    readonly root: string;
  },
): boolean {
  return (candidate === root) || candidate.startsWith(`${root}${sep}`,);
}

/**
 Error thrown when a listing request escapes the configured root directory.
 
 @example
 ```ts
 new ListingOutsideRootError({ path: '/etc', rootDirectory: '/home' });
 ```
 */
export class ListingOutsideRootError extends Error {
  /**
   Builds a descriptive confinement error.
   
   @param path - Rejected request path.
   
   @param rootDirectory - Configured root the request had to stay under.
   
   @example
   ```ts
   new ListingOutsideRootError({ path: '/etc', rootDirectory: '/home' });
   ```
   */
  public constructor(
    {
      path,
      rootDirectory,
    }: {
      readonly path: string;
      readonly rootDirectory: string;
    },
  ) {
    super(`Refusing to list ${path}: outside the configured root ${rootDirectory}`,);
    this.name = 'ListingOutsideRootError';
  }
}

/**
 Confines a renderer-supplied path under the root directory.
 
 The request is folded lexically and compared against the root, so `..`
 segments and absolute paths elsewhere are refused before any filesystem call.
 
 @param path - Renderer-supplied directory path.
 
 @param rootDirectory - Configured root every listing request must stay under.
 
 @returns Resolved absolute path safe to read.
 
 @throws ListingOutsideRootError when the path escapes the root.
 
 @example
 ```ts
 await confinePath({ path: '/home/docs', rootDirectory: '/home' });
 ```
 */
export function confinePath(
  {
    path,
    rootDirectory,
  }: {
    readonly path: string;
    readonly rootDirectory: string;
  },
): string {
  /**
   Configured root with relative segments folded away.
   */
  const root = resolve(rootDirectory,);

  /**
   Request path with `..` and `.` segments folded lexically.
   */
  const resolved = resolve(path,);

  if (!isWithinRoot({
    candidate: resolved,
    root,
  },))
    throw new ListingOutsideRootError({
      path,
      rootDirectory: root,
    },);

  return resolved;
}
