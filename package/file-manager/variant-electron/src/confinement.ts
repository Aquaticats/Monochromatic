/**
 Confinement of renderer-supplied listing paths to the configured root.
 
 The renderer runs sandboxed with context isolation on and Node integration
 off, so the main process is the only place a listing request can be refused.
 The check is not lexical alone: a symbolic link inside the root still reports
 a path under it, so both the request and the root are canonicalised and the
 canonical pair is compared as well. A symbolic link whose target lies under
 the root stays listable; one that points elsewhere is refused.
 
 @example
 ```ts
 await confinePath({ path: '/home/docs', rootDirectory: '/home' });
 ```
 
 @packageDocumentation
 */

import { realpath, } from 'node:fs/promises';
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
 
 The lexical pass rejects the cheap escapes (`..` segments, absolute paths
 elsewhere) before any filesystem call; the canonical pass then rejects a
 request that is lexically under the root but resolves outside it through a
 symbolic link. The returned path is the lexically resolved request, so the
 renderer keeps seeing the paths it named.
 
 @param path - Renderer-supplied directory path.
 
 @param rootDirectory - Configured root every listing request must stay under.
 
 @returns Resolved absolute path safe to read.
 
 @throws ListingOutsideRootError when the path escapes the root, lexically or canonically.
 
 @throws Error when the path or the root cannot be canonicalised, such as a path that does not exist.
 
 @example
 ```ts
 await confinePath({ path: '/home/docs', rootDirectory: '/home' });
 ```
 */
export async function confinePath(
  {
    path,
    rootDirectory,
  }: {
    readonly path: string;
    readonly rootDirectory: string;
  },
): Promise<string> {
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

  /**
   Root and request with every symbolic link expanded.
   */
  const [canonicalRoot, canonicalPath,] = await Promise.all([
    realpath(root,),
    realpath(resolved,),
  ],);

  if (!isWithinRoot({
    candidate: canonicalPath,
    root: canonicalRoot,
  },))
    throw new ListingOutsideRootError({
      path,
      rootDirectory: root,
    },);

  return resolved;
}
