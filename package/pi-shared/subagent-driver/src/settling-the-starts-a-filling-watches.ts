/**
 Settling the starts a filling made: the entry is out of the queue only when its run exists, and an entry whose launcher ended
 without a run goes back to the head of the queue or aside with the launcher's own reason.

 Both are read from the record one start writes, in `start-of-a-queue-entry-the-filling-watches.ts`; this module is the round that
 acts on it.

 @module
 */

import { stat, } from 'node:fs/promises';
import { join, } from 'node:path';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  NO_FINISHED_RUN_OF_THAT_NAME,
  SUMMARY_FILE_NAME,
  newestFinishedRunOf,
} from './newest-finished-run-of-a-name.ts';
import {
  putAQueueEntryIn,
  readQueueOfRuns,
} from './queue-of-runs.ts';
import type { RunCommand, } from './run-command.ts';
import { unitNameOfGiven, } from './running-pi-units.ts';
import { writeSetAsideRecord, } from './set-aside-entry.ts';
import {
  launcherReasonIn,
  gaveUpAtItsDeadline,
  removeWatchedStart,
  type WatchedStart,
  watchedStartsIn,
} from './start-of-a-queue-entry-the-filling-watches.ts';
import {
  unitStateIsRunning,
  unitStateOf,
} from './wait-for-a-unit-to-end.ts';

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
 Says whether one start's run exists now: its unit is going, or a run folder of its name wrote a summary after the start.

 @param runsFolder - folder the driver writes its runs into

 @param record - the start

 @param run - how a command is run

 @returns whether the run the filling asked for exists, and the words that show it

 @example
 ```ts
 const exists = await runExistsFor({ runsFolder: '/runs', record, run, });
 ```
 */
export async function runExistsFor({
  runsFolder,
  record,
  run,
}: {
  readonly runsFolder: string;
  readonly record: WatchedStart;
  readonly run: RunCommand;
}): Promise<{
  readonly there: boolean;
  readonly because?: string
}> {
  /**
   The unit the run itself is started as.
   */
  const unitName = unitNameOfGiven({ name: record.runName, },);
  /**
   What the manager says of it, which is the run itself once the launcher has made it.
   */
  const state = await unitStateOf({
    run,
    unitName,
  },);
  if (unitStateIsRunning({ state, },)) {
    return {
      there: true,
      because: `unit ${unitName} is ${state}`,
    };
  }
  /**
   The newest run folder of that name that finished, which is this start's run when it finished after the start.
   */
  const finished = await newestFinishedRunOf({
    runsFolder,
    runName: record.runName,
  },);
  if (finished === NO_FINISHED_RUN_OF_THAT_NAME) {
    return { there: false, };
  }
  /**
   When that run's summary was written.
   */
  const written = await stat(join(
    runsFolder,
    finished,
    SUMMARY_FILE_NAME,
  ),);
  return (written.mtimeMs >= Date.parse(record.startedAt,))
    ? {
      there: true,
      because: `run folder ${finished} holds a summary written after the start`,
    }
    : { there: false, };
}

/**
 Settles every start whose batch has ended: the entry is out of the queue only when its run exists.

 @param runsFolder - folder the driver writes its runs into

 @param queueFile - the queue's path

 @param run - how a command is run

 @param now - where this reads the time

 @param report - told about a record that cannot be read or a start that cannot be settled

 @returns what became of each start, in the order the records were read

 @example
 ```ts
 const settled = await settleWatchedStarts({ runsFolder: '/runs', queueFile: '/runs/queue.json', run, now: Date.now, report: () => {}, });
 ```
 */
export async function settleWatchedStarts({
  runsFolder,
  queueFile,
  run,
  now,
  report,
}: {
  readonly runsFolder: string;
  readonly queueFile: string;
  readonly run: RunCommand;
  readonly now: () => number;
  readonly report: (line: string) => void;
},): Promise<readonly SettledStart[]> {
  /**
   The watched starts as they stand.
   */
  const records = await watchedStartsIn({
    runsFolder,
    report,
  },);
  /**
   What became of each.
   */
  const settled: SettledStart[] = [];
  for (const record of records) {
    try {
      // oxlint-disable-next-line eslint/no-await-in-loop -- one entry after another, and each record is its own file and its own queue write
      settled.push(await settleOneStart({
        runsFolder,
        queueFile,
        record,
        run,
        now,
      },),);
    }
    catch (error) {
      report(`${new Date(now(),).toISOString()} ${record.name}: its start could not be settled and the filling goes on: ${caughtValueText(error,)}`,);
    }
  }
  return settled;
}

/**
 Settles one start: keeps its entry out of the queue while its run exists, and puts the entry back or sets it aside otherwise.

 @param runsFolder - folder the driver writes its runs into

 @param queueFile - the queue's path

 @param record - the start

 @param run - how a command is run

 @param now - where this reads the time

 @returns what became of the entry

 @example
 ```ts
 const settled = await settleOneStart({ runsFolder: '/runs', queueFile: '/runs/queue.json', record, run, now: Date.now, });
 ```
 */
async function settleOneStart({
  runsFolder,
  queueFile,
  record,
  run,
  now,
}: {
  readonly runsFolder: string;
  readonly queueFile: string;
  readonly record: WatchedStart;
  readonly run: RunCommand;
  readonly now: () => number;
}): Promise<SettledStart> {
  /**
   When the filling says what it found.
   */
  const time = new Date(now(),).toISOString();
  /**
   The queue as it stands.
   */
  const queue = await readQueueOfRuns({ queueFile, },);
  // A kill between writing the record and taking the entry out leaves the entry where it was: nothing is lost, so the record goes
  // away and the next pass tries the entry again.
  if (queue.some(function isStillQueued(entry,) {
    return entry.name === record.name;
  },)) {
    await removeWatchedStart({
      runsFolder,
      name: record.name,
    },);
    return {
      name: record.name,
      outcome: 'never-left-the-queue',
      line: `${time} ${record.name}: its entry never left the queue, so the record of its start is dropped and the entry is tried again`,
    };
  }
  /**
   Whether the batch that was asked for this entry is still going, which is the start being under way.
   */
  const batchState = await unitStateOf({
    run,
    unitName: `${record.unit}.service`,
  },);
  if (unitStateIsRunning({ state: batchState, },)) {
    return {
      name: record.name,
      outcome: 'still-going',
    };
  }
  /**
   Whether the run the filling asked for exists now.
   */
  const exists = await runExistsFor({
    runsFolder,
    record,
    run,
  },);
  if (exists.there) {
    await removeWatchedStart({
      runsFolder,
      name: record.name,
    },);
    return {
      name: record.name,
      outcome: 'the-run-exists',
      line: `${time} ${record.name}: its run exists (${exists.because ?? 'a run folder holds its summary'}), so the entry stays out of the queue`,
    };
  }
  /**
   The reason the launcher ended with, read from what its batch printed.
   */
  const reason = await launcherReasonIn({ file: record.outputFile, },);
  /**
   What to say about the launcher when it printed nothing at all.
   */
  const why = (reason === '')
    ? `its launcher ended with no run and printed nothing into ${record.outputFile}`
    : reason;
  if (gaveUpAtItsDeadline({ reason: why, },)) {
    await putAQueueEntryIn({
      queueFile,
      entry: record.entry,
      position: 0,
    },);
    await removeWatchedStart({
      runsFolder,
      name: record.name,
    },);
    return {
      name: record.name,
      outcome: 'put-back',
      line: `${time} put ${record.name} back at the head of the queue: its launcher gave up waiting for a place before any run existed (${why})`,
    };
  }
  /**
   Where the entry's set-aside record was written.
   */
  const file = await writeSetAsideRecord({
    runsFolder,
    record: {
      name: record.name,
      entry: record.entry,
      reason: why,
      setAsideAt: time,
    },
  },);
  await removeWatchedStart({
    runsFolder,
    name: record.name,
  },);
  return {
    name: record.name,
    outcome: 'set-aside',
    line: `${time} set aside ${record.name} (the-launcher); ${file}; its launcher ended with no run: ${why}`,
  };
}
