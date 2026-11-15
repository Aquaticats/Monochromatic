/**
 The record of one start the filling made, and what becomes of it: an entry is out of the queue only when its run exists.

 The filling hands an entry to a batch's own unit and takes the entry out of the queue at once, so before this the queue lost every
 entry whose launcher did not get as far as a run. On 2026-11-14T11:28:41-05:00 the filling started a queued review, the launcher
 answered `ALREADY RUNNING` twenty seconds later, and nothing put the entry back or said so anywhere a reader looks: the run was
 missing for an hour and three quarters, until the queue was read by hand. The batch's own unit was gone by then, so the filling's
 count of the places asked for no longer saw it either.

 So every start leaves a record beside the runs, named `<the entry's name>-queue-start.json`, written before the batch's unit is asked
 for. A round reads those records and settles each start whose batch has ended: the run exists, so the entry stays out of the queue;
 or it does not, so the entry goes back to the head of the queue when the launcher only gave up waiting for a place, and is set aside
 with the launcher's own reason otherwise. The record goes away once the entry is settled, which is what makes the filling say it
 once.

 @module
 */

import {
  readFile,
  rename,
  rm,
  stat,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { errorCodeOf, } from './error-code-of.ts';
import {
  entriesOrNothingIn,
  NO_FINISHED_RUN_OF_THAT_NAME,
  SUMMARY_FILE_NAME,
  newestFinishedRunOf
} from './newest-finished-run-of-a-name.ts';
import {
  isQueueRecord,
  putAQueueEntryIn,
  type QueueRecord,
  readQueueOfRuns,
} from './queue-of-runs.ts';
import type { RunCommand, } from './run-command.ts';
import { unitNameOfGiven, } from './running-pi-units.ts';
import { writeSetAsideRecord, } from './set-aside-entry.ts';
import {
  unitStateIsRunning,
  unitStateOf,
} from './wait-for-a-unit-to-end.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The end of the file one start's record is written to, which sits beside the runs under the entry's name.
 */
export const WATCHED_START_FILE_SUFFIX = '-queue-start.json';

/**
 The report a caller that wants none passes, which a folder of runs may be read with no one listening.
 */
function noReport(): void {
  // A caller that reads the records without a report has already decided what to do with what it finds.
}

/**
 The end of the file a record is written through, which is renamed over the record.
 */
const BEING_WRITTEN_SUFFIX = '.being-written';

/**
 How much of what a batch printed is read for the reason its launcher ended with. The line a launcher's failure is on is the last
 one the batch printed, and the whole of what a start printed is kept beside the runs when it is long.
 */
const BATCH_OUTPUT_TAIL_CHARACTERS = 65_536;

/**
 The words the retrying start prints when it gave up asking for a place before its deadline, which is no fault of the entry.
 */
const DEADLINE_WORDS = 'no slot freed before the deadline';

/**
 One start the filling made and has not settled.
 */
export type WatchedStart = {
  /**
   The entry's name, which is the name its record and its set-aside record carry.
   */
  readonly name: string;

  /**
   The name the run itself takes, which is what its own unit and its run folders are named by.
   */
  readonly runName: string;

  /**
   The entry as the queue held it, so putting it back needs nothing else.
   */
  readonly entry: QueueRecord;

  /**
   The batch list the entry was started from.
   */
  readonly batchFile: string;

  /**
   Where that list's unit writes both of its streams.
   */
  readonly outputFile: string;

  /**
   The unit the batch was started in, without its `.service` suffix.
   */
  readonly unit: string;

  /**
   When the filling asked for it, as ISO 8601 with its offset.
   */
  readonly startedAt: string;
};

/**
 A watched-start record that cannot be read as one.
 */
export class WatchedStartError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'WatchedStartError';
}

/**
 What became of one watched start.
 */
export type SettledStart = {
  /**
   The entry's name.
   */
  readonly name: string;

  /**
   What became of it.
   */
  readonly outcome: 'the-run-exists' | 'put-back' | 'set-aside' | 'still-going' | 'never-left-the-queue';

  /**
   The line the filling says, or nothing while the start is still going.
   */
  readonly line?: string;
};

/**
 Reads the file one start's record is written to.

 @param runsFolder - folder the driver writes its runs into

 @param name - the entry's name

 @returns the record's path

 @example
 ```ts
 const file = watchedStartFileOf({ runsFolder: '/runs', name: 'a', });
 ```
 */
export function watchedStartFileOf({
  runsFolder,
  name,
}: {
  readonly runsFolder: string;
  readonly name: string;
},): string {
  return join(
    runsFolder,
    `${name}${WATCHED_START_FILE_SUFFIX}`,
  );
}

/**
 Says whether a file name is one start's record.

 @param file - the file's name

 @returns whether the file holds a watched start

 @example
 ```ts
 const isOne = isAWatchedStartFile({ file: 'a-queue-start.json', },);
 ```
 */
export function isAWatchedStartFile({ file, }: { readonly file: string },): boolean {
  return file.endsWith(WATCHED_START_FILE_SUFFIX,);
}

/**
 Reads one record's text.

 @param shown - the file's text

 @param file - the file's path, for the message

 @returns the record

 @throws WatchedStartError when the text is not a record this package wrote

 @example
 ```ts
 const record = watchedStartIn({ shown: '{"name":"a"}', file: '/runs/a-queue-start.json', });
 ```
 */
export function watchedStartIn({
  shown,
  file,
}: {
  readonly shown: string;
  readonly file: string;
},): WatchedStart {
  /**
   The text read as anything JSON holds.
   */
  const written = valueInJson({
    shown,
    file
  },);
  if (
    ((typeof written) !== 'object') || (written === null)
      || (!('name' in written))
      || ((typeof written.name) !== 'string')
      || (!('runName' in written))
      || ((typeof written.runName) !== 'string')
      || (!('entry' in written))
      || (!isQueueRecord(written.entry,))
      || (!('batchFile' in written))
      || ((typeof written.batchFile) !== 'string')
      || (!('outputFile' in written))
      || ((typeof written.outputFile) !== 'string')
      || (!('unit' in written))
      || ((typeof written.unit) !== 'string')
      || (!('startedAt' in written))
      || ((typeof written.startedAt) !== 'string')
  ) {
    throw new WatchedStartError(`${file} does not hold the record of a start this package wrote`,);
  }
  return {
    name: written.name,
    runName: written.runName,
    entry: written.entry,
    batchFile: written.batchFile,
    outputFile: written.outputFile,
    unit: written.unit,
    startedAt: written.startedAt,
  };
}

/**
 Reads the value a text holds, which a text that is not JSON is refused with.

 @param shown - the text

 @param file - the file's path, for the message

 @returns the value

 @throws WatchedStartError when the text is not JSON

 @example
 ```ts
 const value = valueInJson({ shown: '{"name":"a"}', file: '/runs/a-queue-start.json', },);
 ```
 */
function valueInJson({
  shown,
  file,
}: {
  readonly shown: string;
  readonly file: string;
},): unknown {
  try {
    return JSON.parse(shown,);
  }
  catch (error) {
    throw new WatchedStartError(`${file} is not JSON, so the start it records cannot be read: ${caughtValueText(error,)}`,);
  }
}

/**
 Writes the record of one start, through a file renamed over it so a reader never sees half a record.

 @param runsFolder - folder the driver writes its runs into

 @param record - the entry, as the queue held it

 @param start - the batch list, its unit, its output and the name the run takes

 @param now - the reading the record's time is written from

 @returns the record's path

 @throws WatchedStartError when the record cannot be written

 @example
 ```ts
 const file = await writeWatchedStart({ runsFolder: '/runs', record: { name: 'a' }, start: { runName: 'a', batchFile: '/runs/batch-a.json', outputFile: '/runs/batch-a.out', unit: 'launch-batch-a' }, now: Date.now, });
 ```
 */
export async function writeWatchedStart({
  runsFolder,
  record,
  start,
  now,
}: {
  readonly runsFolder: string;
  readonly record: QueueRecord;
  readonly start: {
    readonly runName: string;
    readonly batchFile: string;
    readonly outputFile: string;
    readonly unit: string;
  };
  readonly now: () => number;
},): Promise<string> {
  /**
   The record's path.
   */
  const file = watchedStartFileOf({
    runsFolder,
    name: record.name,
  },);
  try {
    await writeFile(
      `${file}${BEING_WRITTEN_SUFFIX}`,
      `${JSON.stringify(
        {
          name: record.name,
          runName: start.runName,
          entry: record,
          batchFile: start.batchFile,
          outputFile: start.outputFile,
          unit: start.unit,
          startedAt: new Date(now(),).toISOString(),
        },
        undefined,
        2,
      )}\n`,
      { encoding: 'utf8', },
    );
    await rename(
      `${file}${BEING_WRITTEN_SUFFIX}`,
      file,
    );
    return file;
  }
  catch (error) {
    throw new WatchedStartError(`the start of ${record.name} could not be written down at ${file}: ${caughtValueText(error,)}`,);
  }
}

/**
 Reads every watched start this runs folder holds.

 @param runsFolder - folder the driver writes its runs into

 @param report - told about a record that cannot be read, which is no reason to leave the others unsettled

 @returns the records, in the order the folder lists them

 @example
 ```ts
 const records = await watchedStartsIn({ runsFolder: '/runs', report: () => {}, });
 ```
 */
export async function watchedStartsIn({
  runsFolder,
  report = noReport,
}: {
  readonly runsFolder: string;
  readonly report?: (problem: string) => void;
},): Promise<readonly WatchedStart[]> {
  /**
   The folder's entries, and nothing at all when the folder is not there.
   */
  const entries = await entriesOrNothingIn(runsFolder,);
  /**
   Every record, read one after another: each is its own file, and a folder of runs holds thousands of other entries.
   */
  const records: WatchedStart[] = [];
  for (const file of entries.filter(function isARecord(entry,) {
    return isAWatchedStartFile({ file: entry, },);
  },)) {
    /**
     The record's own path.
     */
    const path = join(
      runsFolder,
      file,
    );
    try {
      /**
       What the file holds, read whole.
       */
      // oxlint-disable-next-line eslint/no-await-in-loop -- one record file after another, and each is read whole
      const shown = await readFile(
        path,
        'utf8',
      );
      records.push(watchedStartIn({
        shown,
        file: path,
      },),);
    }
    catch (error) {
      report(`${path} could not be read as the record of a start, so it is left as it is: ${caughtValueText(error,)}`,);
    }
  }
  return records;
}

/**
 Removes one start's record, which is the step that makes the filling say what became of the entry once.

 @param runsFolder - folder the driver writes its runs into

 @param name - the entry's name

 @example
 ```ts
 await removeWatchedStart({ runsFolder: '/runs', name: 'a', });
 ```
 */
export async function removeWatchedStart({
  runsFolder,
  name,
}: {
  readonly runsFolder: string;
  readonly name: string;
},): Promise<void> {
  await rm(
    watchedStartFileOf({
      runsFolder,
      name,
    },),
    { force: true, },
  );
}

/**
 Reads the last of what a batch's unit printed, which ends with the reason its launcher gave.

 @param file - the file both of the unit's streams went to

 @returns the text, and nothing when the file is not there

 @example
 ```ts
 const said = await tailOf({ file: '/runs/batch-a.out', });
 ```
 */
async function tailOf({ file, }: { readonly file: string },): Promise<string> {
  try {
    /**
     What the file system says about the file.
     */
    const { size, } = await stat(file,);
    /**
     The bytes to read, of the end of the file.
     */
    const bytes = Math.min(
      BATCH_OUTPUT_TAIL_CHARACTERS,
      size,
    );
    if (bytes === 0) {
      return '';
    }
    /**
     The file, read again for its end.
     */
    return (await readFile(
      file,
      'utf8',
    )).slice(-bytes,);
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return '';
    }
    throw error;
  }
}

/**
 Reads the reason a launcher ended with, out of what its batch printed.

 @param file - the file both of the batch unit's streams went to

 @returns the last line the batch printed, which names the launcher's own reason, and nothing when it printed none

 @example
 ```ts
 const reason = await launcherReasonIn({ file: '/runs/batch-a.out', });
 ```
 */
export async function launcherReasonIn({ file, }: { readonly file: string },): Promise<string> {
  /**
   The end of what the batch printed.
   */
  const said = await tailOf({ file, },);
  /**
   Its lines, without the empty ones a file ends with.
   */
  const lines = said.split('\n',)
    .filter(function isNotBlank(line,) {
      return line.trim() !== '';
    },);
  /**
   The last line, which the batch prints once every entry it started has ended.
   */
  const last = lines.at(-1,);
  return (last ?? '').trim();
}

/**
 Says whether a launcher only gave up waiting for a place before any run existed.

 @param reason - the last line the batch printed

 @returns whether the retrying start gave up at its deadline

 @example
 ```ts
 const gaveUp = gaveUpAtItsDeadline({ reason: 'no slot freed before the deadline: --name=a', });
 ```
 */
export function gaveUpAtItsDeadline({ reason, }: { readonly reason: string },): boolean {
  return reason.includes(DEADLINE_WORDS,);
}
