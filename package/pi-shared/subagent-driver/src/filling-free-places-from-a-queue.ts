/**
 One pass of filling the free places from a queue: the agents going are counted against the level the pass is given, the entries
 are started in the queue's order while places are free, and an entry whose start fails never holds the entries behind it.

 An entry whose own start fails is set aside where that failure happens, with its reason and the time, and the pass goes on with the
 next entry. A failure of the machine is not the entry's: the entry keeps its place and the pass stops, because starting a later
 entry would take the place the earlier one is waiting for and the queue's order is the whole point of the queue.

 The places are counted once, at the start of the pass, and a run this pass starts is counted against the next entry it asks about:
 a started run has no unit for some seconds, so a second count inside the same pass would read its place as free and start two runs
 into one place.

 @module
 */

import type { GoingRun, } from './going-runs.ts';
import {
  poolOfAQueueEntry,
  resumesAQueueEntry,
  shapeOfAQueueEntry,
} from './entry-of-a-queue.ts';
import {
  type FreeSpace,
  freeSpaceFloorTextOf,
  freeSpaceIsUnderTheFloor,
  freeSpaceTextOf,
} from './free-space-floor.ts';
import { poolChoiceFor, } from './pool-gate.ts';
import type { PoolMembership, } from './pool-limits.ts';
import {
  type PoolTallies,
  talliesOf,
} from './pool-tallies.ts';
import {
  readQueueOfRuns,
  type QueueRecord,
  takeAQueueEntryOut,
} from './queue-of-runs.ts';
import {
  removeSetAsideRecord,
  setAsideRecordIsThere,
} from './set-aside-entry.ts';
import type {
  FailureOfAStart,
  StartOfAnEntry,
} from './starting-an-entry-of-a-queue.ts';

/**
 How many entries one pass starts at most, which is the coordinator's own number since 2026-11-13T15:05-05:00: each review starts by
 reading a whole diff and its files, and the reviewer's provider ended nine reviews that day on its limit of tokens a minute, three
 and five of them within seconds of each other. Fewer starts in one round is the part of that the filling controls.
 */
export const FILLING_STARTS_A_ROUND = 2;

/**
 Milliseconds one pass waits between starts, so a run it started has its unit before the next entry is asked about.
 */
export const FILLING_STAGGER_MILLISECONDS = 20_000;

/**
 The seams one pass asks, so a test answers them without a unit, a worktree or a process.
 */
export type FillingSeams = {
  /**
   The runs going now, which is what the level is counted against.
   */
  readonly runsGoing: () => Promise<readonly GoingRun[]>;

  /**
   The room the volume that holds the repository has.
   */
  readonly freeSpace: () => FreeSpace;

  /**
   Starts one entry.
   */
  readonly start: (request: { readonly record: QueueRecord }) => Promise<StartOfAnEntry>;

  /**
   Sets one entry aside: writes its record and takes it out of the queue.
   */
  readonly setAside: (request: {
    readonly record: QueueRecord;
    readonly failure: FailureOfAStart;
  }) => Promise<{
    readonly file: string;
    readonly left: number;
  }>;

  /**
   Passes time between one start and the next.
   */
  readonly wait: (milliseconds: number) => Promise<void>;

  /**
   Says one line as it happens.
   */
  readonly report: (line: string) => void;
};

/**
 What one pass answered.
 */
export type FillingOutcome = {
  /**
   Every line the pass said, in the order it said them.
   */
  readonly lines: readonly string[];

  /**
   The entries it started, in the order it started them.
   */
  readonly started: readonly string[];

  /**
   The entries it set aside, in the order it set them aside.
   */
  readonly setAsideNames: readonly string[];

  /**
   How many agents were going when the pass counted them.
   */
  readonly runsGoingAtTheStart: number;

  /**
   How many places the level left free then.
   */
  readonly placesFreeAtTheStart: number;

  /**
   How many entries the queue holds now.
   */
  readonly left: number;

  /**
   Why the pass stopped starting entries, when it stopped with places free.
   */
  readonly stoppedBecause?: string;
};

/**
 Reads the pool an entry asks for, which is what tells whether the machine has a place for it now.

 @param record - the entry

 @returns the pool it asks for, every pool it could hold, or none

 @example
 ```ts
 const pool = requestedPoolOf({ record: { name: 'a', branch: 'feat/a' } });
 ```
 */
function requestedPoolOf({ record, }: { readonly record: QueueRecord },): PoolMembership {
  return poolOfAQueueEntry({ record, },);
}

/**
 Says whether an entry may start now, given the runs going and the pools they hold.

 The pool a run may take carries the limit it is counted against, so the five runs on the reviewing model that the recorded 429s set
 are read from the reviewing pool's own row and never from a second number beside it.

 @param record - the entry

 @param tallies - how many runs each pool holds, with the limit every pool was counted against

 @returns whether a place is free for this entry

 @example
 ```ts
 const mayStart = mayStartNow({ record: { name: 'a', branch: 'feat/a' }, tallies });
 ```
 */
export function mayStartNow({
  record,
  tallies,
}: {
  readonly record: QueueRecord;
  readonly tallies: PoolTallies;
},): boolean {
  /**
   What the entry is.
   */
  const shape = shapeOfAQueueEntry({ record, },);
  // An entry the filling cannot read is one it must take out of the way rather than pass over, so it is always asked to start.
  if (shape.kind === 'unreadable') {
    return true;
  }
  /**
   Which pool the entry may take.
   */
  const choice = poolChoiceFor({
    requested: requestedPoolOf({ record, },),
    tallies,
    resumes: resumesAQueueEntry({ record, },),
  },);
  return choice.destination !== 'no-room';
}

/**
 Reads the line an entry's start ended with.

 @param record - the entry

 @param started - what starting it answered

 @param time - when the line is said, as ISO 8601

 @returns the line

 @example
 ```ts
 const line = startedLineOfAnEntry({ record: { name: 'a' }, started, time: '2026-11-14T00:00:00.000Z' });
 ```
 */
function startedLineOfAnEntry({
  record,
  started,
  time,
}: {
  readonly record: QueueRecord;
  readonly started: Extract<StartOfAnEntry, { readonly started: true }>;
  readonly time: string;
},): string {
  return `${time} started ${record.name} in ${started.unit}.service; output in ${started.outputFile}`;
}

/**
 Reads the line one entry set aside ends with.

 @param record - the entry

 @param failure - why its start failed

 @param file - where its record is

 @param time - when the line is said, as ISO 8601

 @returns the line

 @example
 ```ts
 const line = setAsideLineOf({ record: { name: 'a' }, failure, file: '/runs/a-set-aside.json', time: '2026-11-14T00:00:00.000Z' });
 ```
 */
function setAsideLineOf({
  record,
  failure,
  file,
  time,
}: {
  readonly record: QueueRecord;
  readonly failure: FailureOfAStart;
  readonly file: string;
  readonly time: string;
},): string {
  return `${time} set aside ${record.name} (${failure.kind}); ${file}; the filling goes on with the next entry: ${failure.reason}`;
}

/**
 Drops every set-aside record whose entry is still in the queue, which is what a kill between writing a record and taking its entry
 out leaves.

 @param entries - the queue's entries

 @param runsFolder - folder the driver writes its runs into

 @param report - where the pass says what it found

 @example
 ```ts
 await dropUnfinishedSetAsides({ entries: [], runsFolder: '/runs', report: () => {} });
 ```
 */
async function dropUnfinishedSetAsides({
  entries,
  runsFolder,
  report,
}: {
  readonly entries: readonly QueueRecord[];
  readonly runsFolder: string;
  readonly report: (line: string) => void;
},): Promise<void> {
  for (const record of entries) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- one entry after another, and each record is its own file
    if (await setAsideRecordIsThere({
      runsFolder,
      name: record.name,
    },)) {
      // oxlint-disable-next-line eslint/no-await-in-loop -- the record of the entry just read
      await removeSetAsideRecord({
        runsFolder,
        name: record.name,
      },);
      report(`${new Date().toISOString()} ${record.name}: a set-aside record was written and its entry never left the queue, so the record is dropped and the entry is tried again\n`,);
    }
  }
}

/**
 Fills the free places from a queue, once.

 @param request - the queue, the level, the folders, and how many entries this pass may start

 @param seams - how the machine is asked and how an entry is started

 @returns what the pass started, what it set aside, and why it stopped

 @example
 ```ts
 const outcome = await fillFreePlacesFromAQueue({ request, seams });
 ```
 */
export async function fillFreePlacesFromAQueue({
  request,
  seams,
}: {
  readonly request: {
    readonly queueFile: string;
    readonly runsFolder: string;
    readonly level: number;
    readonly startsAtMost?: number;
    readonly staggerMilliseconds?: number;
  };
  readonly seams: FillingSeams;
},): Promise<FillingOutcome> {
  /**
   Every line the pass says.
   */
  const lines: string[] = [];
  /**
   Says one line as it happens, and keeps it.
   
   @param line - the line, with its own line ending
   */
  function say(line: string,): void {
    lines.push(line,);
    seams.report(line,);
  }
  /**
   The queue as it stands, which is read again for every entry so that an entry another session queued meanwhile is seen.
   */
  const queueAsItStands = await readQueueOfRuns({ queueFile: request.queueFile, },);
  await dropUnfinishedSetAsides({
    entries: queueAsItStands,
    runsFolder: request.runsFolder,
    report: say,
  },);
  /**
   The runs going now, and the pools they hold.
   */
  const counted = await seams.runsGoing();
  /**
   The runs the pass counts against the pools: those going now, and one more for every entry it starts.
   */
  const runs: GoingRun[] = [...counted,];
  /**
   The room the volume has.
   */
  const freeSpace = seams.freeSpace();
  /**
   How many places the level leaves.
   */
  const placesFree = Math.max(
    0,
    request.level - counted.length,
  );
  /**
   How many entries this pass may start.
   */
  const startsAtMost = request.startsAtMost ?? FILLING_STARTS_A_ROUND;
  say(`${new Date().toISOString()} ${String(counted.length,)} agents going, level ${String(request.level,)}: ${String(placesFree,)} places free, ${String(queueAsItStands.length,)} entries in the queue; ${freeSpaceTextOf(freeSpace,)}\n`,);
  /**
   The entries this pass started, in the order it started them.
   */
  const started: string[] = [];
  /**
   The entries this pass set aside, in the order it set them aside.
   */
  const setAsideNames: string[] = [];
  /**
   Reads what the pass answers, so every way out of the loop says the same thing.

   @param left - how many entries the queue holds after the pass

   @param stoppedBecause - why the pass stopped with places free, when it did

   @returns what the pass answers
   */
  function outcomeOf({
    left,
    stoppedBecause,
  }: {
    readonly left: number;
    readonly stoppedBecause?: string;
  },): FillingOutcome {
    return {
      lines,
      started,
      setAsideNames,
      runsGoingAtTheStart: counted.length,
      placesFreeAtTheStart: placesFree,
      left,
      ...((stoppedBecause === undefined) ? {} : { stoppedBecause, }),
    };
  }
  if (freeSpaceIsUnderTheFloor(freeSpace,)) {
    /**
     Why the pass started nothing.
     */
    const why = `${freeSpaceTextOf(freeSpace,)}, under the floor of ${freeSpaceFloorTextOf()}`;
    say(`${new Date().toISOString()} ${why}; nothing is started while the drive is that full\n`,);
    return outcomeOf({
      left: queueAsItStands.length,
      stoppedBecause: why,
    },);
  }
  /* oxlint-disable eslint/no-await-in-loop -- one entry after another, because the stagger is what keeps the places in the queue's
     order and each start's place is counted before the next entry is asked about. See doc/lint/sequential-awaits.md. */
  for (;;) {
    if ((started.length >= startsAtMost) || (started.length >= placesFree)) {
      return outcomeOf({ left: (await readQueueOfRuns({ queueFile: request.queueFile, },)).length, },);
    }
    /**
     The queue as it stands at this entry.
     */
    const entries = await readQueueOfRuns({ queueFile: request.queueFile, },);
    if (entries.length === 0) {
      return outcomeOf({ left: 0, },);
    }
    /**
     Every pool's count, with this pass's own starts counted in.
     */
    const tallies = talliesOf({ runs, },);
    /**
     The first entry of the queue that may start now, which is what keeps the queue's order.
     */
    const record = entries.find(function mayStartNowHere(entry,) {
      return mayStartNow({
        record: entry,
        tallies,
      },);
    },);
    if (record === undefined) {
      /**
       Why the pass starts nothing more.
       */
      const why = `no entry of the ${String(entries.length,)} left may start now: every pool one of them asks for is full`;
      say(`${new Date().toISOString()} ${why}\n`,);
      return outcomeOf({
        left: entries.length,
        stoppedBecause: why,
      },);
    }
    /**
     What starting the chosen entry answered.
     */
    const outcome = await seams.start({ record, },);
    if (!outcome.started) {
      /**
       Why the entry's start failed.
       */
      const { failure, } = outcome;
      if (failure.kind === 'the-machine') {
        /**
         Why the pass stops.
         */
        const why = `the machine said no: ${failure.reason}`;
        say(`${new Date().toISOString()} left ${record.name} in place at position ${String(entries.indexOf(record,),)} of ${String(entries.length,)}; the filling stops this pass and starts nothing behind it: ${failure.reason}\n`,);
        return outcomeOf({
          left: entries.length,
          stoppedBecause: why,
        },);
      }
      /**
       Where the entry's record was written.
       */
      const kept = await seams.setAside({
        record,
        failure,
      },);
      setAsideNames.push(record.name,);
      say(`${setAsideLineOf({
        record,
        failure,
        file: kept.file,
        time: new Date().toISOString(),
      },)}\n`,);
      continue;
    }
    /**
     The queue as it stands once the entry left it, which is read again at the moment of writing so an entry another session
     queued meanwhile is not written away with a list read before it.
     */
    const taken = await takeAQueueEntryOut({
      queueFile: request.queueFile,
      name: record.name,
    },);
    /**
     How many entries the queue holds once the entry left it.
     */
    const { entries: left, } = taken;
    started.push(record.name,);
    runs.push({
      name: record.name,
      membership: requestedPoolOf({ record, },),
    },);
    say(`${startedLineOfAnEntry({
      record,
      started: outcome,
      time: new Date().toISOString(),
    },)}; ${String(left.length,)} entries left\n`,);
    for (const line of outcome.lines) {
      say(`${line}\n`,);
    }
    if (started.length < startsAtMost) {
      await seams.wait(request.staggerMilliseconds ?? FILLING_STAGGER_MILLISECONDS,);
    }
  }
  /* oxlint-enable eslint/no-await-in-loop */
}
