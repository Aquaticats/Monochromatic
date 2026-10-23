import { realpath, } from 'node:fs/promises';
import {
  basename,
  dirname,
  isAbsolute,
  join,
  relative,
  resolve,
  sep,
} from 'node:path';

import { rethrowUnlessMissingPath, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  RUNS_SEARCH_DEPTH,
  searchEnters,
} from './cache-account-slices.ts';

//region Search reach
// WHETHER A SEARCH FOR RUNS DIRECTORIES WOULD FIND THE RUNS OF A DIRECTORY, for the audit's warning that a launch which
// follows the runbook writes where the roots searched do not look. Two spellings of one directory (the home directory
// and a link to it) are one directory, and a root reaches only as far down as the walk goes.

/**
 The real path of a directory, or of the nearest ancestor that exists with the missing names kept after it.

 A BOUNDED STRUCTURAL WALK up the path, one level per call, so the number of names in the path is also the stack bound.

 @param path - directory to resolve, which need not exist

 @returns Absolute path with every link on the way to the existing part resolved

 @throws What `realpath` raises for any failure but an absent path, such as a loop of links or a denied permission

 @example
 ```ts
 const real = await realPathOf({ path: '/cats/home/temp/agent', },); // '/cats/home/temp/agent' where none of it exists
 ```
 */
async function realPathOf({ path, }: { readonly path: string; },): Promise<string> {
  /**
   The path as an absolute one.
   */
  const absolute = resolve(path,);
  try {
    return await realpath(absolute,);
  }
  catch (error) {
    rethrowUnlessMissingPath({ error, },);
    /**
     The directory above, which has the same answer to give for the part that exists.
     */
    const parent = dirname(absolute,);
    if (parent === absolute)
      return absolute;
    return join(
      await realPathOf({ path: parent, },),
      basename(absolute,),
    );
  }
}

/**
 Whether a search that starts at a root would find the runs directories that sit up to a number of levels below another
 directory.

 THE ROOT AND THE DIRECTORY ARE COMPARED BY REAL PATH, since the home directory of the process and its real place are
 both spellings of one directory. THE WALK STOPS `RUNS_SEARCH_DEPTH` LEVELS BELOW A ROOT, so a root holding the
 directory farther up than the runs' own depth allows leaves the deepest runs unread, and it enters no hidden directory
 and no dependency tree, so a directory behind one is not reached at all.

 @param root - directory the search starts at

 @param directory - directory whose runs are asked about

 @param runsLevels - how many levels below `directory` its runs directories sit, at the most

 @returns Whether `directory` is the root or lies below it, and the deepest runs of it are within the walk

 @throws What `realpath` raises for any failure but an absent path

 @example
 ```ts
 const reached = await searchReaches({ root: '/cats/home', directory: '/cats/home/temp/agent', runsLevels: 2, },); // true
 ```
 */
export async function searchReaches(
  {
    root,
    directory,
    runsLevels,
  }: {
    readonly root: string;
    readonly directory: string;
    readonly runsLevels: number;
  },
): Promise<boolean> {
  /**
   The way down from the root to the directory.
   */
  const way = relative(
    await realPathOf({ path: root, },),
    await realPathOf({ path: directory, },),
  );
  if (way === '')
    return true;
  if (way === '..')
    return false;
  if (way.startsWith(`..${sep}`,))
    return false;
  if (isAbsolute(way,))
    return false;
  /**
   The names the search would have to enter on the way down.
   */
  const names = way.split(sep,);
  if (!names.every(function isEntered(name,): boolean {
    return searchEnters({ name, },);
  },))
    return false;
  return (names.length + runsLevels) <= RUNS_SEARCH_DEPTH;
}

//endregion Search reach
