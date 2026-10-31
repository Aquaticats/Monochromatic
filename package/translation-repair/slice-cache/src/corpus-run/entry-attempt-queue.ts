import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  lifecycleStamp,
  wallClockNow,
} from '@monochromatic-dev/translation-repair-markup/ts';
import {
  printRunLine,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { readAttemptOutcome, } from './entry-reattempt.ts';
import type { EntryOutcome, } from './pass-entry-contract.ts';

//region Entry attempt queue
// The re-attempt loop, lifted out of `corpus-pass.ts` so it can be tested
// without a corpus, a provider, or seven hours.
//
// `entry-reattempt.ts` decides whether ONE attempt earned another. This drives
// the sequence those verdicts produce, and the two questions are separable:
// a correct verdict used by a loop that never comes back settles nothing.
//
// EFFECTS ARE INJECTED rather than imported, because every one of them is
// either expensive or unavailable in a test: reading a cache directory,
// spending up to seven hours on an entry, and asking whether the run's wall
// budget is gone. What is left here is the ordering, which is the part with a
// defect worth catching.

/**
 Least a queued entry must carry: something to name it by in a log line.
 */
export type QueueableEntry = {
  /**
   Entry id, used only for reporting.
   */
  readonly id: string;
};

/**
 Run that never ends during an attempt, for a caller with no stop of its own to report.

 @returns False

 @example
 ```ts
 const stopped = neverStopped();
 ```
 */
function neverStopped(): boolean {
  return false;
}

/**
 Runs every pending entry once, then re-runs the ones that earned it.

 RE-ATTEMPTS GO TO THE BACK. Coverage of the corpus is what a first attempt
 buys, so an oversized entry must not spend the run's budget before every
 other entry has been tried at all. A queue gives that ordering for free:
 pushing to the back cannot overtake anything still waiting.

 @param pending - entries to attempt, in the order the caller ranked them

 @param cachedCountFor - slices one entry holds, asked before and after each
 attempt so progress is measured rather than assumed

 @param attempt - runs one attempt, reporting settlement or retry disposition

 @param stopBeforeNext - asked before each attempt; true ends the run. Owned
 by the caller so the reason and its wording stay with the budget that knows
 them, and it may log

 @param runHasStopped - asked after each attempt; true means the run ended
 during it, so the attempt earns no REATTEMPT or STALLED line and no second
 attempt, whatever the cache gained: the caller says why the run stopped, once.
 Never stops the loop on its own; `stopBeforeNext` does that. Absent, the run
 never stops during an attempt

 @param attemptOf - number of the entry's latest attempt, as the caller counts and persists it (the pass's attempts
 file), asked after each attempt and when a NOT RESUMED line is printed; every line this queue prints for an entry
 carries it as `attempt=`, so a REATTEMPT, STALLED, INCOMPLETE or NOT RESUMED line names the attempt it belongs to and
 matches the TALLY line of the same attempt (ledger B960)

 @param wallClock - clock the lines' time is read on, printed as `at=`; the wall clock unless a test fixes it

 @throws `LifecycleAttemptError` when `attemptOf` answers a number that is not a whole number from 1

 @example
 ```ts
 await runAttemptQueue({ pending, cachedCountFor, attempt, stopBeforeNext, attemptOf, },);
 ```
 */
export async function runAttemptQueue<EntryT extends QueueableEntry,>(
  {
    pending,
    cachedCountFor,
    attempt,
    stopBeforeNext,
    attemptOf,
    wallClock = wallClockNow,
    runHasStopped = neverStopped,
  }: {
    readonly pending: readonly EntryT[];
    readonly cachedCountFor: (args: { readonly entry: EntryT; },) => Promise<number>;
    readonly attempt: (args: { readonly entry: EntryT; },) => Promise<EntryOutcome>;
    readonly stopBeforeNext: () => boolean;
    readonly attemptOf: (args: { readonly entry: EntryT; },) => number;
    readonly wallClock?: () => Date;
    readonly runHasStopped?: () => boolean;
  },
): Promise<void> {
  /**
   Attempts still to make, growing at the back as entries earn another.
   */
  const queue: EntryT[] = [...pending,];

  /**
   The clock time and attempt a line about this entry carries, read when the line is printed.

   @param entry - entry the line is about

   @returns `at=` and `attempt=` fields of the line

   @example
   ```ts
   const stamp = stampOf({ entry, },);
   ```
   */
  function stampOf({ entry, }: { readonly entry: EntryT; },): string {
    return lifecycleStamp({
      attempt: attemptOf({ entry, },),
      wallClock,
    },);
  }

  while (queue.length > 0) {
    /**
     Next attempt, taken from the front.
     */
    const entry = nonNullishOrThrow(queue.shift(),);

    if (stopBeforeNext())
      return;

    /* oxlint-disable no-await-in-loop -- one cheap read per attempt, against an attempt that may run seven hours */
    /**
     Slices this entry already holds.
     */
    const cachedBefore = await cachedCountFor({ entry, },);
    /* oxlint-enable no-await-in-loop */

    /* oxlint-disable no-await-in-loop -- entries run sequentially by design; that is the point of a queue */
    /**
     Settlement or retry disposition from this attempt.
     */
    const outcome = await attempt({ entry, },);
    /* oxlint-enable no-await-in-loop */

    /* oxlint-disable no-await-in-loop -- pairs with the read bracketing the other side of this attempt */
    /**
     Slices present now the attempt has stopped.
     */
    const cachedAfter = await cachedCountFor({ entry, },);
    /* oxlint-enable no-await-in-loop */

    /**
     What the attempt earned.
     */
    const verdict = readAttemptOutcome({
      outcome,
      cachedBefore,
      cachedAfter,
    },);

    // A RUN THAT ENDED DURING THE ATTEMPT queues nothing and says nothing about the entry's cache: the caller's stop
    // line names the cause, and a REATTEMPT line before it would read as a contradiction.
    /**
     Whether the run ended during this attempt.
     */
    const runEnded = runHasStopped();

    if ((verdict.kind === 'earned') && (!runEnded)) {
      printRunLine({
        line: `REATTEMPT ${entry.id} queued ${stampOf({ entry, },)}: ${String(verdict.gained,)} more cache ${
          wordForCount({
            count: verdict.gained,
            one: 'record',
            many: 'records',
          },)
        } than it had, so the next attempt starts further along`,
      },);
      queue.push(entry,);
    }
    if ((verdict.kind === 'stalled') && (!runEnded)) {
      printRunLine({
        line: `STALLED ${entry.id} ${stampOf({ entry, },)}: its ${String(verdict.cached,)} cache ${
          wordForCount({
            count: verdict.cached,
            one: 'record is',
            many: 'records are',
          },)
        } what it started with, so a further attempt in this invocation would repeat it`,
      },);
    }
    if (verdict.kind === 'stopped') {
      printRunLine({
        line: `INCOMPLETE ${entry.id} ${stampOf({ entry, },)}: stage-local work remains; whole entry will not restart in `
          + 'this invocation',
      },);
    }
  }
}

//endregion Entry attempt queue
