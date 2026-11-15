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
  type ClaimedAnEntry,
  claimAnEntryOfAQueue,
  completeTheClaim,
  giveBackTheClaim,
  settleTheQueueAfterAKill,
} from './claims-on-the-entries-of-a-queue.ts';
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
  type QueueClaim,
  type QueueRecord,
  readQueueOfRuns,
  withoutTheClaim,
} from './queue-of-runs.ts';
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
   Starts one entry, which this filling has just claimed.
   */
  readonly start: (request: {
    readonly record: QueueRecord;
    readonly claim: QueueClaim;
  }) => Promise<StartOfAnEntry>;

  /**
   Sets one entry aside: writes its record and completes the claim that took it out of the queue.
   */
  readonly setAside: (request: {
    readonly record: QueueRecord;
    readonly failure: FailureOfAStart;
    readonly claim: {
      readonly queueId: string;
      readonly claim: QueueClaim;
    };
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
 Fills the free places from a queue, once.

 A pass first settles what a kill left behind (see `settleTheQueueAfterAKill`), then counts the agents going, then starts entries in
 the queue's order while places are free. Each entry is claimed before anything is started for it, and the claim is completed, given
 back or set aside before the pass asks about the next entry.

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
    readonly lockWaitMilliseconds?: number;
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
   The entries the queue holds once what a kill left behind is settled.
   */
  const stoodAtTheStart = await settleTheQueueAfterAKill({
    queueFile: request.queueFile,
    runsFolder: request.runsFolder,
    report: say,
    ...(request.lockWaitMilliseconds === undefined ? {} : { waitMilliseconds: request.lockWaitMilliseconds, }),
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
  say(`${new Date().toISOString()} ${String(counted.length,)} agents going, level ${String(request.level,)}: ${String(placesFree,)} places free, ${String(stoodAtTheStart,)} entries in the queue; ${freeSpaceTextOf(freeSpace,)}\n`,);
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
      left: stoodAtTheStart,
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
     Every entry of the queue that may start now, in the queue's order.
     */
    const candidates = entries.filter(function mayStartNowHere(entry,) {
      return mayStartNow({
        record: entry,
        tallies,
      },);
    },);
    if (candidates.length === 0) {
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
     The entry this pass claimed, or nothing when every entry that may start is being started by another filling.
     */
    let mine: {
      readonly record: QueueRecord;
      readonly queueId: string;
      readonly claim: QueueClaim;
    } | undefined;
    for (const candidate of candidates) {
      /**
       What taking a claim on this candidate answered.
       */
      // oxlint-disable-next-line eslint/no-await-in-loop -- each candidate is its own write under the queue's lock
      const taken: ClaimedAnEntry = await claimAnEntryOfAQueue({
        queueFile: request.queueFile,
        record: candidate,
        now: Date.now,
        ...(request.lockWaitMilliseconds === undefined ? {} : { waitMilliseconds: request.lockWaitMilliseconds, }),
      },);
      if (taken.kind === 'claimed') {
        mine = {
          record: candidate,
          queueId: taken.queueId,
          claim: taken.claim,
        };
        break;
      }
      // Another filling claimed it, or it left the queue, between this pass's read and its claim: the entry behind it is asked
      // about rather than the pass stopping, because an entry another filling is starting is being started.
      say(`${new Date().toISOString()} ${candidate.name}: ${taken.kind === 'claimed-by-another' ? 'another filling claimed it first' : 'it left the queue before this pass claimed it'}, so this pass asks about the entry behind it\n`,);
    }
    if (mine === undefined) {
      /**
       Why the pass starts nothing more.
       */
      const why = `every entry of the ${String(entries.length,)} left that may start now is being started by another filling`;
      return outcomeOf({
        left: entries.length,
        stoppedBecause: why,
      },);
    }
    /**
     The entry this pass starts, as the queue holds it without the claim.
     */
    const record = withoutTheClaim({ record: mine.record, },);
    /**
     What starting the claimed entry answered.
     */
    let outcome: StartOfAnEntry;
    try {
      outcome = await seams.start({
        record,
        claim: mine.claim,
      },);
    }
    catch (error) {
      // A start that threw before it answered leaves the claim to be given back here: a claim whose holder is alive is not recovered
      // by another filling, so leaving it would keep the entry where no pass starts it.
      await giveBackTheClaim({
        queueFile: request.queueFile,
        queueId: mine.queueId,
        token: mine.claim.token,
      },);
      throw error;
    }
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
        /**
         What giving the claim back left in the queue.
         */
        const givenBack = await giveBackTheClaim({
          queueFile: request.queueFile,
          queueId: mine.queueId,
          token: mine.claim.token,
        },);
        say(`${new Date().toISOString()} left ${record.name} in place at position ${String(entries.indexOf(mine.record,),)} of ${String(entries.length,)} and gave its claim back; the filling stops this pass and starts nothing behind it: ${failure.reason}\n`,);
        return outcomeOf({
          left: givenBack.left,
          stoppedBecause: why,
        },);
      }
      /**
       Where the entry's record was written.
       */
      const kept = await seams.setAside({
        record,
        failure,
        claim: {
          queueId: mine.queueId,
          claim: mine.claim,
        },
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
     What completing the claim left, which is the queue without this attempt and with every other entry where it stood.
     */
    const completed = await completeTheClaim({
      queueFile: request.queueFile,
      queueId: mine.queueId,
      token: mine.claim.token,
    },);
    if (completed.taken !== 1) {
      say(`${new Date().toISOString()} ${record.name}: the start took effect but the queue no longer held this filling's attempt, so nothing was removed for it; ${String(completed.left,)} entries left\n`,);
    }
    started.push(record.name,);
    runs.push({
      name: record.name,
      membership: requestedPoolOf({ record, },),
    },);
    say(`${startedLineOfAnEntry({
      record,
      started: outcome,
      time: new Date().toISOString(),
    },)}; ${String(completed.left,)} entries left\n`,);
    for (const line of outcome.lines) {
      say(`${line}\n`,);
    }
    if (started.length < startsAtMost) {
      await seams.wait(request.staggerMilliseconds ?? FILLING_STAGGER_MILLISECONDS,);
    }
  }
  /* oxlint-enable eslint/no-await-in-loop */
}
