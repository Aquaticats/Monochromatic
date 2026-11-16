/**
 The one owner of one restart's start: a marker created with an exclusive create just before a launcher is asked to start the run,
 so two checks that both planned the same restart cannot both start it, and a start whose process was killed before it made a run
 folder is taken over once.

 The marker names its process. A marker whose process is alive is a start somebody else is making, whether it is still waiting for a
 pool slot or already running; a marker whose process is gone and that left no run folder is a start that died, and the next check
 takes it over. Nothing waits and nothing is locked: one create decides.

 @module
 */

import {
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import { readRegularFileAt, } from './reading-a-regular-file.ts';
import {
  fieldsOf,
  isRecord,
} from './read-pi-fields.ts';

/**
 The code the operating system gives a path that already exists.
 */
const ALREADY_THERE_CODE = 'EEXIST';

/**
 The code the operating system gives a process that is not there.
 */
const NO_SUCH_PROCESS_CODE = 'ESRCH';

/**
 What taking a restart's start answered.
 */
export type StartOwnership =
  | {
    readonly kind: 'mine';

    /**
     The marker's path, which stays as the record that this process started the run.
     */
    readonly file: string;
  }
    | {
      readonly kind: 'somebody else';

    /**
     The words that say who took it, for the line a caller prints.
     */
      readonly line: string;
    };

/**
 Says whether a process named by its number is still there.

 A process this account may not signal is still there: the operating system refuses with `EPERM`, which is not the answer a gone
 process gives.

 @param pid - the process number a marker names

 @returns whether the process is alive

 @example
 ```ts
 const alive = processIsAlive(4212);
 ```
 */
export function processIsAlive(pid: number,): boolean {
  try {
    process.kill(
      pid,
      0,
    );
    return true;
  }
  catch (error) {
    // Only ESRCH says the process is gone; any other refusal means it is there and this account may not signal it.
    return errorCodeOf(error,) !== NO_SUCH_PROCESS_CODE;
  }
}

/**
 Reads the process number a start marker names.

 @param file - the marker's path

 @returns the number, or nothing when the marker names none

 @example
 ```ts
 const pid = markerPidIn('/reviews/a/restart-2-started.json');
 ```
 */
async function markerPidIn(file: string,): Promise<{ readonly pid?: number }> {
  /**
   What the marker holds.
   */
  const read = await readRegularFileAt({ path: file, },);
  if (read.kind !== 'text') {
    return {};
  }
  try {
    /**
     What the marker's text holds, of any shape.
     */
    const kept: unknown = JSON.parse(read.text,);
    if (!isRecord(kept,)) {
      return {};
    }
    /**
     The process number the record names.
     */
    const { pid, } = fieldsOf(kept,);
    return ((typeof pid) === 'number') ? { pid, } : {};
  }
  catch (error) {
    // A marker a kill cut short names no process; the caller treats it as a start that died.
    return (String(error,)
      .length
      > 0) ? {} : {};
  }
}

/**
 The path of one restart's start marker.

 @param reviewsFolder - folder that holds one folder per review

 @param review - the review's own name

 @param number - the run number the start is for

 @returns the marker's path

 @example
 ```ts
 const file = startMarkerOf({ reviewsFolder: '/reviews', review: 'a', number: 2 });
 ```
 */
export function startMarkerOf({
  reviewsFolder,
  review,
  number,
}: {
  readonly reviewsFolder: string;
  readonly review: string;
  readonly number: number;
},): string {
  return join(
    reviewsFolder,
    review,
    `restart-${String(number,)}-started.json`,
  );
}

/**
 Takes one restart's start, or says who already has it.

 @param reviewsFolder - folder that holds one folder per review

 @param review - the review's own name

 @param number - the run number the start is for

 @returns the ownership, or the line that says who took it

 @example
 ```ts
 const start = await takeTheStart({ reviewsFolder: '/reviews', review: 'a', number: 2 });
 ```
 */
export async function takeTheStart({
  reviewsFolder,
  review,
  number,
}: {
  readonly reviewsFolder: string;
  readonly review: string;
  readonly number: number;
},): Promise<StartOwnership> {
  /**
   The marker's path.
   */
  const file = startMarkerOf({
    reviewsFolder,
    review,
    number,
  },);
  /**
   The text this marker writes: its own process number, so a later check can tell it is gone.
   */
  const mine = `${JSON.stringify({
    pid: process.pid,
    startedAt: new Date().toISOString(),
    review,
    number,
  },)}\n`;
  // A marker whose process is gone is taken over once: that is the state a kill before the run folder leaves, and nothing else
  // ever takes it. A marker whose process is alive is somebody's start, and is not touched.
  /* oxlint-disable eslint/no-await-in-loop -- the two attempts are ordered by design, the second only after the first found an
     owner whose process is gone; the marker's create is what decides. See doc/lint/sequential-awaits.md. */
  for (const attempt of [
    0,
    1,
  ]) {
    try {
      await writeFile(
        file,
        mine,
        {
          encoding: 'utf8',
          flag: 'wx',
        },
      );
      return {
        kind: 'mine',
        file,
      };
    }
    catch (error) {
      if (errorCodeOf(error,) !== ALREADY_THERE_CODE) {
        throw error;
      }
      /**
       The process number the marker names.
       */
      const { pid, } = await markerPidIn(file,);
      if ((pid !== undefined) && (String(pid,) !== '')
        && processIsAlive(pid,)) {
        return {
          kind: 'somebody else',
          line: `${file} names process ${String(pid,)}, which is still there`,
        };
      }
      if (attempt === 1) {
        return {
          kind: 'somebody else',
          line: `${file} is held by a start that keeps taking over`,
        };
      }
      // The marker's process is gone; the start died before a run folder was made, so this check takes it over.
      await rm(
        file,
        { force: true, },
      );
    }
  }
  /* oxlint-enable eslint/no-await-in-loop */
  return {
    kind: 'somebody else',
    line: `${file} is held`,
  };
}
