import { stat, } from 'node:fs/promises';
import {
  dirname,
  isAbsolute,
  relative,
  resolve,
  sep,
} from 'node:path';
import { caughtErrorHasCode, } from '../io/error.ts';
import { globWatchDirectory, } from '../io/glob.ts';
import { globs, } from '../tracker.ts';

/**
 Returns whether path is currently an existing directory.
 
 @param path - Absolute or relative path to inspect.
 
 @returns Whether path exists and is a directory.
 
 @throws When filesystem metadata cannot be read for a reason other than absence.
 
 @example
 ```ts
 const present = await pathIsDirectory('/tmp/src');
 ```
 */
async function pathIsDirectory(path: string,): Promise<boolean> {
  try {
    return (await stat(path,))
      .isDirectory();
  }
  catch (statError: unknown) {
    if (caughtErrorHasCode({
      error: statError,
      code: 'ENOENT',
    },))
      return false;
    if (caughtErrorHasCode({
      error: statError,
      code: 'ENOTDIR',
    },))
      return false;

    throw statError;
  }
}

/**
 Returns nearest existing directory at or above a target directory, walked
 via {@link nearestExistingDirectoryFromCandidate}.
 
 @param directoryPath - Directory path requested by watch tracking.
 
 @returns Existing directory path suitable for `fs.watch`.
 
 @example
 ```ts
 const dir = await nearestExistingDirectory('/repo/missing/generated');
 ```
 */
export async function nearestExistingDirectory(directoryPath: string,): Promise<string> {
  return await nearestExistingDirectoryFromCandidate(resolve(directoryPath,),);
}

/**
 Walks parent directories, checking each with {@link pathIsDirectory}, until
 one exists.
 
 @param candidate - Directory path currently being inspected.
 
 @returns Existing directory path suitable for watching.
 
 @example
 ```ts
 const dir = await nearestExistingDirectoryFromCandidate('/repo/missing/generated');
 ```
 */
async function nearestExistingDirectoryFromCandidate(candidate: string,): Promise<string> {
  if (await pathIsDirectory(candidate,))
    return candidate;
  /**
   Parent directory for next lookup attempt.
   */
  const parent = dirname(candidate,);
  if (parent === candidate)
    return candidate;
  return await nearestExistingDirectoryFromCandidate(parent,);
}

/**
 Returns whether `ancestorPath` is the same path as, or an ancestor of,
 `descendantPath`.
 
 @param ancestorPath - Candidate ancestor path.
 
 @param descendantPath - Candidate descendant path.
 
 @returns Whether descendant is inside ancestor.
 
 @example
 ```ts
 const covered = pathIsSameOrAncestor({ ancestorPath: '/repo/src', descendantPath: '/repo/src/generated' });
 ```
 */
function pathIsSameOrAncestor(
  {
    ancestorPath,
    descendantPath,
  }: {
    readonly ancestorPath: string;
    readonly descendantPath: string;
  },
): boolean {
  /**
   Relative path from ancestor to descendant.
   */
  const relativePath = relative(
    resolve(ancestorPath,),
    resolve(descendantPath,),
  );
  if (relativePath === '')
    return true;
  if (relativePath === '..')
    return false;
  if (relativePath.startsWith(`..${sep}`,))
    return false;

  return !isAbsolute(relativePath,);
}

/**
 Returns whether event path creates or updates a static directory needed by
 a tracked glob: compares each glob's root from {@link globWatchDirectory}
 against the event path with {@link pathIsSameOrAncestor}.
 
 @param absolutePath - Absolute event path.
 
 @returns Whether event should trigger rerun for a tracked glob root.
 
 @example
 ```ts
 const source = trackedGlobStaticDirectoryAffected('/repo/src');
 ```
 */
export function trackedGlobStaticDirectoryAffected(absolutePath: string,): boolean {
  return [...globs.keys(),].some(function globStaticDirectoryAffected(pattern,): boolean {
    return pathIsSameOrAncestor({
      ancestorPath: absolutePath,
      descendantPath: globWatchDirectory(pattern,),
    },);
  },);
}
