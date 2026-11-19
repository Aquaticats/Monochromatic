/**
 The folder one start of a run writes into: claimed by making it, so that the making succeeds for one start and fails for every
 other start of that name.

 A number read from a listing of the runs folder is no reservation: two starts of one run name listed the folder before either made
 its own, both read the same next number, and the second start wrote over the first run's summary. Making the folder is the one
 step the operating system decides for one start only, so the claim is the making, and a start that loses tries the number after it.
 A number a caller named is claimed the same way and refused when it is taken, because a run folder is one run's.

 @module
 */

import {
  mkdir,
  readdir,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import {
  RunNumbersExhaustedError,
  nextRunNumber,
} from './next-run-number.ts';

/**
 The code the operating system gives a folder that is already there.
 */
const PATH_EXISTS_CODE = 'EEXIST';

/**
 A run folder a start cannot have, because another start of the same run name holds it.
 */
export class RunFolderTakenError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'RunFolderTakenError';
}

/**
 A run folder this start holds, and the number it carries.
 */
export type ClaimedRunFolder = {
  /**
   The number the folder's name ends with.
   */
  readonly runNumber: number;

  /**
   The folder, which this start made and no other start made.
   */
  readonly runFolder: string;
};

/**
 Names a run folder without taking it, so the pinned-number preflight and the claim check the same path.

 @param runsFolder - parent directory that holds every run folder

 @param runName - name used as the folder prefix

 @param runNumber - number used as the folder suffix

 @returns path the launcher checks or claims

 @example
 ```ts
 const folder = pathForRunNumber({ runsFolder: '/runs', runName: 'copy-route', runNumber: 4, },);
 ```
 */
export function pathForRunNumber({
  runsFolder,
  runName,
  runNumber,
}: {
  readonly runsFolder: string;
  readonly runName: string;
  readonly runNumber: number;
},): string {
  return join(
    runsFolder,
    `${runName}-${String(runNumber,)}`,
  );
}

/**
 Makes one run folder and answers it, or says the operating system gave that name to somebody else.

 @param runsFolder - folder the runs live in

 @param runName - the run's name, which the folder's name starts with

 @param runNumber - the number the folder's name ends with

 @returns the claimed folder, or nothing when the folder is already there

 @example
 ```ts
 const claimed = await folderForNumber({ runsFolder: '/runs', runName: 'copy-route', runNumber: 4 },);
 ```
 */
async function folderForNumber({
  runsFolder,
  runName,
  runNumber,
}: {
  readonly runsFolder: string;
  readonly runName: string;
  readonly runNumber: number;
},): Promise<{ readonly claimed?: ClaimedRunFolder }> {
  /**
   The folder this number names.
   */
  const runFolder = pathForRunNumber({
    runsFolder,
    runName,
    runNumber,
  },);
  try {
    // No `recursive`: a folder that is already there is the answer, not something to write into.
    await mkdir(runFolder,);
    return {
      claimed: {
        runNumber,
        runFolder,
      },
    };
  }
  catch (error) {
    if (errorCodeOf(error,) === PATH_EXISTS_CODE) {
      return {};
    }
    throw error;
  }
}

/**
 Claims the folder one start of a run writes into.

 The runs folder is made first, which is what lets each claim below be the one non-recursive `mkdir` the operating system decides
 for a single start. A number the caller named is claimed once and refused when it is taken; no number named, the claim starts at
 the number after every folder of that name and takes the next free one, so a restart never writes over the run before it even when
 that run's own folder is made in the same instant.

 @param runsFolder - folder the runs live in

 @param runName - the run's name, which every folder of it starts with

 @param wantedNumber - the number the caller named, when it named one

 @returns the folder this start holds

 @throws RunFolderTakenError when the caller named a number whose folder is already there

 @example
 ```ts
 const { runFolder } = await claimRunFolder({ runsFolder: '/runs', runName: 'copy-route' },);
 ```
 */
export async function claimRunFolder({
  runsFolder,
  runName,
  wantedNumber,
}: {
  readonly runsFolder: string;
  readonly runName: string;
  readonly wantedNumber?: number;
},): Promise<ClaimedRunFolder> {
  await mkdir(
    runsFolder,
    { recursive: true, },
  );
  if (wantedNumber !== undefined) {
    /**
     The folder this start claimed with the number the caller named.
     */
    const named = await folderForNumber({
      runsFolder,
      runName,
      runNumber: wantedNumber,
    },);
    if (named.claimed !== undefined) {
      return named.claimed;
    }
    throw new RunFolderTakenError(
      `${pathForRunNumber({
        runsFolder,
        runName,
        runNumber: wantedNumber,
      },)} is already there, so a run named ${runName} cannot write there: one run folder holds one run, and a start writes over nothing another start wrote. Start this run without --run=${String(wantedNumber,)} and it takes the next free number.`,
    );
  }
  /**
   The number being tried, which one claim per turn of the loop moves past, held in a field because a turn of the loop changes it.
   */
  const trying: { number: number } = {
    number: nextRunNumber({
      runName,
      entries: await readdir(runsFolder,),
    },),
  };
  /* oxlint-disable eslint/no-await-in-loop -- one claim, then the number after it, then the next claim: which number is free is
     what the claim that failed says, and two claims must not be taken at once. See doc/lint/sequential-awaits.md. */
  for (;;) {
    /**
     The folder this turn of the loop claimed, which is none when its number is taken.
     */
    const claimed = await folderForNumber({
      runsFolder,
      runName,
      runNumber: trying.number,
    },);
    if (claimed.claimed !== undefined) {
      return claimed.claimed;
    }
    if (trying.number >= Number.MAX_SAFE_INTEGER) {
      // Nothing follows this number: a run folder number cannot rise past the largest number a run can be counted to, so this
      // loop ends here rather than asking for one taken folder forever.
      throw new RunNumbersExhaustedError(
        `every number this claim tried, the last of them ${runName}-${String(trying.number,)}, is taken, and ${String(trying.number,)} is the largest number a run can be counted to, so no number can follow it: no run of this name can be given a folder. A person removes the folders of this name that are not wanted, or starts the run under another name.`,
      );
    }
    trying.number += 1;
  }
  /* oxlint-enable eslint/no-await-in-loop */
}
