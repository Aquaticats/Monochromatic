/**
 Settling the starts a filling made: the entry is out of the queue only when its run exists, and an entry whose launcher ended
 without a run goes back to the head of the queue or aside with the launcher's own reason.

 Both are read from the record one start writes, in `start-of-a-queue-entry-the-filling-watches.ts`, and from the answer the
 batch that started it wrote, in `start-answer-of-an-entry.ts`; this module is the round that acts on them.

 What the manager says of the batch's unit is asked first: an entry still queued beside a batch that is going is the state a
 filling killed between its start and its take leaves, and taking the entry out there settles it to that one batch.

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
  takeAQueueEntryOut,
} from './queue-of-runs.ts';
import type { RunCommand, } from './run-command.ts';
import { unitNameOfGiven, } from './running-pi-units.ts';
import { writeSetAsideRecord, } from './set-aside-entry.ts';
import {
  NO_START_ANSWER,
  startAnswerOf,
} from './start-answer-of-an-entry.ts';
import {
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
  readonly outcome: 
    | 'the-run-exists'
      | 'put-back'
      | 'set-aside'
      | 'still-going'
      | 'never-left-the-queue'
      | 'batch-state-unread';

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
   What the manager says of the batch unit now, which is asked before anything is dropped: an entry still queued beside a batch
   that is going is the state a kill leaves, and it settles to that one batch.
   */
  const batchState = await unitStateOf({
    run,
    unitName: `${record.unit}.service`,
  },);
  if (batchState === '') {
    // A manager that cannot be asked says nothing about the batch. Dropping the record could start a second batch for one entry, so
    // the record stays and a reader is told why.
    return {
      name: record.name,
      outcome: 'batch-state-unread',
      line: `${time} ${record.name}: the state of ${record.unit}.service could not be read, so its start is left as it is and its entry stays where it is`,
    };
  }
  if (unitStateIsRunning({ state: batchState, },)) {
    /**
     What taking the entry out of the queue answered, which says whether the filling that was killed had left it there.
     */
    const taken = await takeAQueueEntryOut({
      queueFile,
      name: record.name,
    },);
    return {
      name: record.name,
      outcome: 'still-going',
      ...((taken.taken
        .length
        === 0)
        ? {}
        : {
          line: `${time} ${record.name}: its batch is going, so its entry leaves the queue and the record of its start stays until that batch settles`,
        }),
    };
  }
  /**
   The queue as it stands.
   */
  const queue = await readQueueOfRuns({ queueFile, },);
  // The batch is not going, so an entry still queued is a start that never got handed on: nothing is lost, the record goes and the
  // next pass tries the entry again.
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
   What the launcher said its start ended with, which is a field beside the runs and never the last line its batch printed.
   */
  const answer = await startAnswerOf({
    runsFolder,
    name: record.name,
  },);
  /**
   What a launcher that said nothing leaves behind.
   */
  const why = (answer === NO_START_ANSWER)
    ? 'its launcher ended with no run and wrote no start answer beside the runs'
    : answer.line;
  if ((answer !== NO_START_ANSWER) && answer.gaveUpWaitingForAPlace) {
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
      line: `${time} put ${record.name} back at the head of the queue: it gave up waiting for a place before any run existed (${why})`,
    };
  }
  /**
   Where the entry set-aside record was written.
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
