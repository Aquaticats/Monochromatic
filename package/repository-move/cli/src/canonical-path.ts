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
 Lists a path and every folder above it, deepest first, ending at the root.

 @param path - an absolute path

 @returns the path and its ancestors

 @example
 ```ts
 import { ancestorChain, } from './canonical-path.ts';

 ancestorChain('/a/b',);
 // ['/a/b', '/a', '/']
 ```
 */
export function ancestorChain(path: string,): readonly string[] {
  /**
   The path and its ancestors, filled from the deepest up.
   */
  const chain: string[] = [];
  for (let cursor = resolve(path,); !chain.includes(cursor,); cursor = dirname(cursor,)) {
    chain.push(cursor,);
  }
  return chain;
}

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
   The path and its ancestors, deepest first.
   */
  const chain = ancestorChain(path,);
  /**
   What resolving each of them found: the resolved path, or nothing for a name that does not exist.
   */
  const resolved = await Promise.all(chain.map(async function resolveOne(candidate,) {
    try {
      return await realpath(candidate,);
    }
    catch (error) {
      if (errorCodeOf(error,) !== MISSING_PATH_CODE) {
        throw error;
      }
      return undefined;
    }
  },),);
  /**
   The place of the deepest one that exists.
   */
  const deepest = resolved.findIndex(function exists(found,) {
    return found !== undefined;
  },);
  /**
   The resolved path of the deepest folder that exists.
   */
  const base = resolved[deepest];
  if (base === undefined) {
    return resolve(path,);
  }
  return join(
    base,
    ...chain.slice(
      0,
      deepest,
    )
      .map(function nameOf(missing,) {
        return basename(missing,);
      },)
      .toReversed(),
  );
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
    inventoryPrefixTo: (keepsSlash && (!prefixTo.endsWith('/',))) ? `${prefixTo}/` : prefixTo,
  };
}
