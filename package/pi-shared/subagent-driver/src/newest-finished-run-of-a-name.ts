/**
 The newest run folder of one run's name that has a summary, which is the run of that name that finished last.

 A run writes `<run name>-<number>`, one folder per attempt, and its summary appears whole and last, so a folder with a summary is
 a run that ended and the highest number among them is the one that ended most recently.

 @module
 */

import {
  access,
  readdir,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import { isDigitsOnly, } from './text-of-an-option.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The character between a run's name and the number of one attempt.
 */
const ATTEMPT_SEPARATOR = '-';

/**
 The summary a finished run writes, and the last thing it writes.
 */
export const SUMMARY_FILE_NAME = 'summary.json';

/**
 No finished run of that name, as a symbol so that no folder can be mistaken for one.
 */
export const NO_FINISHED_RUN_OF_THAT_NAME: unique symbol = Symbol('runs folder holds no finished run of that name',);

/**
 One attempt of one run's name.
 */
export type AttemptOfARun = {
  /**
   The attempt's folder, as the runs folder names it.
   */
  readonly folder: string;

  /**
   The attempt's number.
   */
  readonly number: number;
};

/**
 Reads a folder's entries, or nothing when the folder is not there.

 @param folder - the folder to list

 @returns its entries' names, or nothing at all for a folder that is not there

 @example
 ```ts
 const entries = await entriesOrNothingIn('/runs');
 ```
 */
export async function entriesOrNothingIn(folder: string,): Promise<readonly string[]> {
  try {
    return await readdir(folder,);
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return [];
    }
    throw error;
  }
}

/**
 Says whether a path is there.

 @param path - the path to look for

 @returns whether it is there

 @example
 ```ts
 const there = await isThere('/runs/a-1/summary.json');
 ```
 */
export async function isThere(path: string,): Promise<boolean> {
  try {
    await access(path,);
    return true;
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return false;
    }
    throw error;
  }
}

/**
 Reads every attempt of one run's name, newest first.

 @param runsFolder - folder the driver writes one run folder per run into

 @param runName - the run's name, which every attempt's folder starts with

 @returns the attempts, highest number first, and nothing when no folder of that name is there

 @example
 ```ts
 const attempts = await attemptsOfARun({ runsFolder: '/runs', runName: 'copy-route', },);
 ```
 */
export async function attemptsOfARun({
  runsFolder,
  runName,
}: {
  readonly runsFolder: string;
  readonly runName: string;
},): Promise<readonly AttemptOfARun[]> {
  /**
   The start every attempt's folder name has.
   */
  const prefix = `${runName}${ATTEMPT_SEPARATOR}`;
  return (await entriesOrNothingIn(runsFolder,)).flatMap(function numberOf(entry,): readonly AttemptOfARun[] {
    if (!entry.startsWith(prefix,)) {
      return [];
    }
    /**
     The text after the run's name.
     */
    const written = entry.slice(prefix.length,);
    return isDigitsOnly(written,)
      ? [{
        folder: entry,
        number: Number(written,),
      },]
      : [];
  },)
    .toSorted(function newestFirst(
      left: AttemptOfARun,
      right: AttemptOfARun,
    ) {
      return right.number - left.number;
    },);
}

/**
 Reads the newest attempt of one run's name that has a summary.

 @param runsFolder - folder the driver writes one run folder per run into

 @param runName - the run's name

 @returns the folder of that run that finished last, or {@link NO_FINISHED_RUN_OF_THAT_NAME}

 @throws the error a read gave, other than a folder that is not there

 @example
 ```ts
 const newest = await newestFinishedRunOf({ runsFolder: '/runs', runName: 'copy-route', },);
 ```
 */
export async function newestFinishedRunOf({
  runsFolder,
  runName,
}: {
  readonly runsFolder: string;
  readonly runName: string;
},): Promise<string | typeof NO_FINISHED_RUN_OF_THAT_NAME> {
  /**
   Every attempt of that name, newest first.
   */
  const attempts = await attemptsOfARun({
    runsFolder,
    runName,
  },);
  /**
   Whether each attempt has finished, asked all at once: two attempts are different folders, so neither answer waits on the
   other.
   */
  const finished = await Promise.all(attempts.map(function ofAttempt(attempt,) {
    return isThere(join(
      runsFolder,
      attempt.folder,
      SUMMARY_FILE_NAME,
    ),);
  },));
  /**
   The newest attempt that finished.
   */
  const newest = attempts.find(function finishedAt(
    _attempt,
    index,
  ) {
    return finished[index] === true;
  },);
  return (newest === undefined) ? NO_FINISHED_RUN_OF_THAT_NAME : newest.folder;
}
