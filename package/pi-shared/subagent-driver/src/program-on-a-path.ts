/**
 The program a `PATH` resolves a name to: the first folder of the search path that holds a file this user may run.

 A session's own processes find their tools by this rule, so a launcher that must know which program a session would run has to
 apply the same one. A folder holding a file nobody made executable, or a folder named after the program, hides nothing behind it:
 a shell's search passes both and takes the next folder, and so does this one.

 The folders are searched as a session's own processes search them, and the answer is the path that search would use rather than a
 tidied one: a path is handed to the kernel as written, because string normalization reads `..` as the folder before it in the
 string while the kernel reads it as the folder before the one a link points at. Those differ exactly where a session could pick
 a different program than the launcher did.

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
  sep,
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

//region What one folder answered

/**
 What looking at one folder of the search path found.

 A program stops the search. A folder this search passes by is not decisive. A failure it cannot pass by stops the search where a
 shell's would stop, and is carried whole so the caller can say what went wrong.
 */
type CandidateOutcome =
  | SearchDecision
    | {
      /**
       That folder holds nothing this search may run.
       */
      readonly passedBy: true;
    };

/**
 The answer that stops a search: a program to run, or a failure it cannot pass by.
 */
type SearchDecision =
  | {
    /**
     The program that folder holds, which is what a session would run.
     */
    readonly program: string;
  }
    | {
      /**
       Why the folder could not be looked at.
       */
      readonly failed: unknown;
    };

/**
 Says whether what one folder holds under the program's name is a program this user may run.

 @param candidate - the path to look at, as the search path spells it

 @returns the program, that the folder holds none, or the failure that stops the search

 @example
 ```ts
 const looked = await lookInFolder('/repo/.cache/monochromatic-git-policy-cli-bin/git',);
 ```
 */
async function lookInFolder(candidate: string,): Promise<CandidateOutcome> {
  try {
    /**
     What is at the candidate's name, following a link to what it points at.
     */
    const shown = await stat(candidate,);
    // A folder is executable in the POSIX sense without being a program, so a search that took one would answer with a path that
    // cannot run.
    if (!shown.isFile()) {
      return { passedBy: true, };
    }
    await access(
      candidate,
      constants.X_OK,
    );
    return { program: candidate, };
  }
  catch (error) {
    /**
     Why the candidate could not be looked at.
     */
    const code = errorCodeOf(error,);
    // A candidate that is not there, or that this user may not run, is what the search moves past. Any other failure would
    // silently change which program is found, so it is carried out instead.
    if (PASSED_BY_CODES.has(code,)) {
      return { passedBy: true, };
    }
    return { failed: error, };
  }
}

//endregion What one folder answered

//region The search path

/**
 Reads the folder one entry of a search path names, without tidying how it is written.

 @param entry - one entry of the search path, as written

 @param folder - the folder the search starts in, which an empty entry names

 @returns the folder to look in, which a relative entry is spelled inside

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
  return isAbsolute(entry,) ? entry : `${folder}${sep}${entry}`;
}

/**
 Finds the program a search path resolves a name to.

 @param name - the program's name, without a folder

 @param path - the search path as `PATH` holds it, whose folders are searched left to right

 @param folder - the folder the search starts in, which a session's own processes look from

 @returns the path of the first program found, or {@link NO_SUCH_PROGRAM} when no folder of the search path holds one

 @throws Error when a folder the search would reach cannot be looked at for a reason other than a missing program or one this
 user may not run

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
   Every folder of the search path, in order, each one spelled as the search path spells it.
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
  const outcomes = await Promise.all(folders.map(function lookFor(searchFolder,) {
    return lookInFolder(`${searchFolder}${sep}${name}`,);
  },),);
  /**
   The first answer a session's own search would act on. A search stops at the program it runs, so what a later folder holds
   cannot change what that session would run, and a later folder's failure is one the session never meets.
   */
  const decisions = outcomes.filter(function stopsTheSearch(outcome,): outcome is SearchDecision {
    return ('program' in outcome) || ('failed' in outcome);
  },);
  /**
   The answer that stopped the search, which is none when no folder held a program.
   */
  const decisive = decisions.at(0,);
  if (decisive === undefined) {
    return NO_SUCH_PROGRAM;
  }
  if ('failed' in decisive) {
    throw decisive.failed;
  }
  return decisive.program;
}

//endregion The search path
