/**
 The attempts of a review run as the runs folder holds them: one folder per attempt, named `<review>-<attempt>`, each with a summary
 when the attempt stopped.

 An attempt can be killed before it writes a summary, so the newest attempt is not always the one that ended: the attempt a review goes
 on from is the newest one that wrote a summary, and every reader here asks for attempts newest first so that a killed one neither hides
 an older finish nor is read as work still going.

 @module
 */

import {
  readFile,
  stat,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import {
  entriesOrNothingIn,
  SUMMARY_FILE_NAME,
} from './newest-finished-run-of-a-name.ts';
import { isDigitsOnly, } from './text-of-an-option.ts';
import { REVIEW_RUN_PREFIX, } from './reviewer-of-a-change.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 A folder with no closing record yet, as a symbol so no record can be mistaken for none.
 */
const NOT_FINISHED_YET: unique symbol = Symbol('the folder checked holds no closing record, so that work has not stopped',);

/**
 Reads one run's summary and when it was written.

 @param file - the summary's path

 @returns what it says and when it was last written, or nothing when it is not there

 @throws the error a read gave, other than a summary that is not there

 @example
 ```ts
 const summary = await summaryOfARunIn({ file: '/runs/a-1/summary.json' });
 ```
 */
async function summaryOfARunIn({ file, }: { readonly file: string },): Promise<{
  readonly shown: string;
  readonly endedAt: number;
} | typeof NOT_FINISHED_YET> {
  try {
    /**
     What the summary says, and when it was written.
     */
    const [shown, written,] = await Promise.all([
      readFile(
        file,
        'utf8',
      ),
      stat(file,),
    ],);
    return {
      shown,
      endedAt: written.mtimeMs,
    };
  }
  catch (error) {
    // A run still going has no summary, which is the answer that puts nothing back for it.
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return NOT_FINISHED_YET;
    }
    throw error;
  }
}

/**
 One review's newest attempt: its name, and the folder that attempt wrote.
 */
type NewestAttemptOfAReview = {
  /**
   The review run's name.
   */
  readonly name: string;

  /**
   The folder the attempt wrote.
   */
  readonly runFolder: string;
};

/**
 Reads the folder one attempt of one review run wrote.

 @param name - the review run's name

 @param attempt - the attempt's number

 @returns the folder's name, below the runs folder

 @example
 ```ts
 const folder = runFolderOf({ name: 'review-a', attempt: 2 });
 ```
 */
export function runFolderOf({
  name,
  attempt,
}: {
  readonly name: string;
  readonly attempt: number;
},): string {
  return `${name}-${String(attempt,)}`;
}

/**
 Reads every attempt of every review run in the runs folder, newest first, one list per review name.

 The attempts matter and not only the newest, because an attempt can be killed before it writes a summary: the newest attempt then
 holds no record of stopping, and the attempt a review goes on from is the newest one that does.

 @param runsFolder - folder the driver writes one run folder per run into

 @returns one list per review name, its attempts newest first

 @example
 ```ts
 const attempts = await attemptsOfEachReview({ runsFolder: '/runs' });
 ```
 */
export async function attemptsOfEachReview({ runsFolder, }: { readonly runsFolder: string },): Promise<ReadonlyMap<string, readonly number[]>> {
  /**
   The attempts of each review name found so far, in the order the folder listed them.
   */
  const attempts = new Map<string, number[]>();
  for (const entry of await entriesOrNothingIn(runsFolder,)) {
    if (!entry.startsWith(REVIEW_RUN_PREFIX,)) {
      continue;
    }
    /**
     Where the attempt's number begins.
     */
    const separator = entry.lastIndexOf('-',);
    if (separator <= 0) {
      continue;
    }
    /**
     The attempt's number, as the folder writes it.
     */
    const written = entry.slice(separator + 1,);
    if (!isDigitsOnly(written,)) {
      continue;
    }
    /**
     The review's name.
     */
    const name = entry.slice(
      0,
      separator,
    );
    /**
     The attempt's number.
     */
    const attempt = Number(written,);
    /**
     The attempts of that name read so far.
     */
    const known = attempts.get(name,);
    if (known === undefined) {
      attempts.set(
        name,
        [attempt,],
      );
    }
    else {
      known.push(attempt,);
    }
  }
  /**
   The attempts of each name, newest first.
   */
  const newestFirst = new Map<string, readonly number[]>();
  for (const [name, numbers,] of attempts) {
    newestFirst.set(
      name,
      numbers.toSorted(function newestOf(
        a,
        b,
      ) {
        return b - a;
      },),
    );
  }
  return newestFirst;
}

/**
 Reads the newest attempt of every review run in the runs folder.

 @param runsFolder - folder the driver writes one run folder per run into

 @returns one entry per review name, the folder of its newest attempt

 @example
 ```ts
 const newest = await newestAttemptOfEachReview({ runsFolder: '/runs' });
 ```
 */
export async function newestAttemptOfEachReview({ runsFolder, }: { readonly runsFolder: string },): Promise<readonly {
  readonly name: string;
  readonly runFolder: string;
}[]> {
  /**
   The newest attempt of each review name found so far.
   */
  const newest = new Map<string, {
    readonly attempt: number;
    readonly runFolder: string;
  }>();
  for (const entry of await entriesOrNothingIn(runsFolder,)) {
    if (!entry.startsWith(REVIEW_RUN_PREFIX,)) {
      continue;
    }
    /**
     Where the attempt's number begins.
     */
    const separator = entry.lastIndexOf('-',);
    if (separator <= 0) {
      continue;
    }
    /**
     The attempt's number, as the folder writes it.
     */
    const written = entry.slice(separator + 1,);
    if (!isDigitsOnly(written,)) {
      continue;
    }
    /**
     The review's name.
     */
    const name = entry.slice(
      0,
      separator,
    );
    /**
     The attempt's number.
     */
    const attempt = Number(written,);
    /**
     The newest attempt of that name read so far.
     */
    const known = newest.get(name,);
    if ((known === undefined) || (attempt > known.attempt)) {
      newest.set(
        name,
        {
          attempt,
          runFolder: entry,
        },
      );
    }
  }
  /**
   The newest attempt of each review name, as a list.
   */
  const newestAttempts: NewestAttemptOfAReview[] = [];
  for (const [name, attempt,] of newest) {
    newestAttempts.push({
      name,
      runFolder: attempt.runFolder,
    },);
  }
  return newestAttempts;
}

/**
 No attempt of that name wrote a summary, as a symbol so no summary can be mistaken for none.
 */
export const NO_ATTEMPT_WROTE_A_SUMMARY: unique symbol = Symbol('this review has no attempt holding a summary',);

/**
 Reads the newest attempt of one review run that wrote a summary, and what that summary says.

 An attempt with no summary did not stop on its own terms. While its run is going the review's name is skipped before this is asked; when
 nothing of that name is going the attempt was killed, and the attempt a review goes on from is an older one that did write a summary.
 Reading that silence as work still going left a review unresumed for as long as its folders stood.

 @param runsFolder - folder the driver writes one run folder per run into

 @param name - the review run's name

 @param attempts - the attempts of that name, newest first

 @returns the newest attempt that wrote a summary, the summary's text, and when it was written, or that none wrote one

 @throws the error a read gave, other than a summary that is not there

 @example
 ```ts
 const found = await newestAttemptThatWroteASummary({ runsFolder: '/runs', name: 'review-a', attempts: [2, 1] });
 ```
 */
export async function newestAttemptThatWroteASummary({
  runsFolder,
  name,
  attempts,
}: {
  readonly runsFolder: string;
  readonly name: string;
  readonly attempts: readonly number[];
},): Promise<{
  readonly attempt: number;
  readonly shown: string;
  readonly endedAt: number;
} | typeof NO_ATTEMPT_WROTE_A_SUMMARY> {
  /* oxlint-disable eslint/no-await-in-loop -- one attempt of the name after another, newest first */
  for (const attempt of attempts) {
    /**
     What this attempt's summary says, or that it wrote none.
     */
    const summary = await summaryOfARunIn({
      file: join(
        runsFolder,
        runFolderOf({
          name,
          attempt,
        },),
        SUMMARY_FILE_NAME,
      ),
    },);
    if (summary !== NOT_FINISHED_YET) {
      return {
        attempt,
        shown: summary.shown,
        endedAt: summary.endedAt,
      };
    }
  }
  /* oxlint-enable eslint/no-await-in-loop */
  return NO_ATTEMPT_WROTE_A_SUMMARY;
}
