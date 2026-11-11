/**
 The program a `PATH` resolves a name to: the first folder of the search path that holds a file this user may run.

 A session's own processes find their tools by this rule, so a launcher that must know which program a session would run has to
 apply the same one. A folder holding a file nobody made executable, or a folder named after the program, hides nothing behind it:
 a shell's search passes both and takes the next folder, and so does this one.

 @module
 */

import {
  access,
  constants,
  stat,
} from 'node:fs/promises';
import {
  delimiter,
  isAbsolute,
  join,
  resolve,
} from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';

//region Constants

/**
 The answer of a search that found nothing: a value of its own, so that a search which found no program cannot be read as a path,
 and no absence travels as a nullish slot.
 */
export const NO_SUCH_PROGRAM: unique symbol = Symbol('no executable program in any folder of PATH');

/**
 The failures a search passes by instead of stopping at: nothing at that name, a path holding something that is not a folder, a file
 this user may not run, and a link that points at itself. A shell's search takes the next folder for each of them, so this one
 does the same.
 */
const PASSED_BY_CODES: ReadonlySet<string> = new Set<string>([
  'EACCES',
  'ELOOP',
  'ENOENT',
  'ENOTDIR',
]);

//endregion Constants

//region One candidate

/**
 Says whether what one folder holds under the program's name is a program this user may run.

 @param candidate - the path to look at

 @returns the path when it is a program, or {@link NO_SUCH_PROGRAM} when it is not there, is not an ordinary file, or may not be
 run

 @example
 ```ts
 const program = await programAt('/repo/.cache/monochromatic-git-policy-cli-bin/git',);
 ```
 */
async function programAt(candidate: string,): Promise<string | typeof NO_SUCH_PROGRAM> {
  try {
    /**
     What is at the candidate's name, following a link to what it points at.
     */
    const shown = await stat(candidate,);
    // A folder is executable in the POSIX sense without being a program, so a search that took one would answer with a path that
    // cannot run.
    if (!shown.isFile()) {
      return NO_SUCH_PROGRAM;
    }
    await access(
      candidate,
      constants.X_OK,
    );
    return candidate;
  }
  catch (error) {
    /**
     Why the candidate could not be looked at.
     */
    const code = errorCodeOf(error,);
    // A candidate that is not there, or that this user may not run, is what the search moves past. Any other failure would
    // silently change which program is found, so it is thrown instead.
    if (PASSED_BY_CODES.has(code,)) {
      return NO_SUCH_PROGRAM;
    }
    throw error;
  }
}

//endregion One candidate

//region The search path

/**
 Reads the folder one entry of a search path names.

 @param entry - one entry of the search path, as written

 @param folder - the folder the search starts in, which an empty entry names

 @returns the folder to look in

 @example
 ```ts
 const folder = searchFolderOf({ entry: '.cache/monochromatic-git-policy-cli-bin', folder: '/repo', },);
 ```
 */
function searchFolderOf({
  entry,
  folder,
}: {
  readonly entry: string;
  readonly folder: string;
},): string {
  if (entry === '') {
    return folder;
  }
  return isAbsolute(entry,) ? entry : resolve(
    folder,
    entry,
  );
}

/**
 Finds the program a search path resolves a name to.

 @param name - the program's name, without a folder

 @param path - the search path as `PATH` holds it, whose folders are searched left to right

 @param folder - the folder the search starts in, which a session's own processes look from

 @returns the path of the first program found, or {@link NO_SUCH_PROGRAM} when no folder of the search path holds one

 @throws Error when a folder of the search path cannot be looked at for a reason other than a missing program or one this user
 may not run

 @example
 ```ts
 const git = await programOnAPath({ name: 'git', path: sessionPath, folder: worktree, },);
 ```
 */
export async function programOnAPath({
  name,
  path,
  folder,
}: {
  readonly name: string;
  readonly path: string;
  readonly folder: string;
},): Promise<string | typeof NO_SUCH_PROGRAM> {
  /**
   Every folder of the search path, in order, each one a whole path already.
   */
  const folders = path.split(delimiter,)
    .map(function folderOfEntry(entry,) {
      return searchFolderOf({
        entry,
        folder,
      },);
    },);
  /**
   What each folder holds under the program's name, in search-path order: no folder's answer depends on another's, so they are
   all looked at at once and the order is kept by the list rather than by the awaits.
   */
  const programs = await Promise.all(folders.map(function lookIn(searchFolder,) {
    return programAt(join(
      searchFolder,
      name,
    ),);
  },),);
  return programs.find(function isAProgram(program,) {
    return (typeof program) === 'string';
  },) ?? NO_SUCH_PROGRAM;
}

//endregion The search path
