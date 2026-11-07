/**
 One spelling per folder: the move compares folders by the path the kernel resolves them to, never by the text a caller
 or Git happened to hold. `/var/home/user/temp` is a link to `/var/mnt/encrypted/temp`, and Git records a worktree by
 its resolved path, so the same folder can be named two ways in the same run.

 @module
 */

import { realpath, } from 'node:fs/promises';
import {
  basename,
  dirname,
  join,
  resolve,
} from 'node:path';
import { MISSING_PATH_CODE, } from './completion-records.ts';
import { errorCodeOf, } from './error-code-of.ts';
import type { MoveOptions, } from './move-options.ts';

/**
 Resolves a path through every link in it. A path that does not exist yet keeps its missing tail: the part that exists is
 resolved and the missing names are appended, so a folder the move is about to make still has one spelling.

 @param path - the path, in any spelling (links, `..`, a trailing slash)

 @returns the canonical path

 @throws Error when a part of the path cannot be inspected for a reason other than absence

 @example
 ```ts
 import { canonicalPath, } from './canonical-path.ts';

 const resolved = await canonicalPath('/var/home/user/temp/agent/',);
 // '/var/mnt/encrypted/temp/agent'
 ```
 */
export async function canonicalPath(path: string,): Promise<string> {
  /**
   The names below the deepest folder that exists, deepest last.
   */
  const missing: string[] = [];
  /**
   The part of the path still being tried.
   */
  let existing = resolve(path,);
  for (;;) {
    try {
      /**
       The resolved part that exists.
       */
      const resolved = await realpath(existing,);
      return join(
        resolved,
        ...missing.toReversed(),
      );
    }
    catch (error) {
      if (errorCodeOf(error,) !== MISSING_PATH_CODE) {
        throw error;
      }
      /**
       The folder that holds the missing name.
       */
      const parent = dirname(existing,);
      if (parent === existing) {
        return existing;
      }
      missing.push(basename(existing,),);
      existing = parent;
    }
  }
}

/**
 The options with the paths the move compares in their canonical spelling: the main checkout and the folder the
 inventory's top-level name maps to.

 @param options - the options as the caller gave them

 @returns the same options with `mainPath` and `inventoryPrefixTo` canonical (the prefix keeps its trailing slash)

 @example
 ```ts
 import { canonicalOptions, } from './canonical-path.ts';

 const canonical = await canonicalOptions(options,);
 ```
 */
export async function canonicalOptions(options: MoveOptions,): Promise<MoveOptions> {
  /**
   Whether the prefix the inventory maps to ends with a slash, which the mapping needs kept.
   */
  const keepsSlash = options.inventoryPrefixTo.endsWith('/',);
  /**
   The mapped-to folder in its canonical spelling.
   */
  const prefixTo = options.inventoryPrefixTo === '' ? '' : await canonicalPath(options.inventoryPrefixTo,);
  return {
    ...options,
    mainPath: await canonicalPath(options.mainPath,),
    inventoryPrefixTo: keepsSlash && (!prefixTo.endsWith('/',)) ? `${prefixTo}/` : prefixTo,
  };
}
