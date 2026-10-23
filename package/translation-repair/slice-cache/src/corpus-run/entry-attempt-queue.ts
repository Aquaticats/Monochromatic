import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';

import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
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
 Takes no wait before an attempt, for a caller whose run never pauses.

 @example
 ```ts
 await noWait();
 ```
 */
async function noWait(): Promise<void> {
  await Promise.resolve();
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

 @param beforeNext - awaited before each attempt and before the stop check, so a pause the caller takes (a pass parked
 until NeuralWatt's energy returns) ends before the budget is asked whether the run goes on. Absent, no pause

 @param interrupted - asked after each attempt; true means a pause cut the attempt short, so the entry goes back to the
 front of the queue with no verdict, no REATTEMPT, STALLED or INCOMPLETE line, and the attempt that follows the pause
 starts further along; when the run then ends before that attempt, a NOT RESUMED line names the entry. Absent, no
 attempt is cut by a pause

 @example
 ```ts
 await runAttemptQueue({ pending, cachedCountFor, attempt, stopBeforeNext, },);
 ```
 */
export async function runAttemptQueue<EntryT extends QueueableEntry,>(
  {
    pending,
    cachedCountFor,
    attempt,
    stopBeforeNext,
    runHasStopped = neverStopped,
    beforeNext = noWait,
    interrupted = neverStopped,
  }: {
    readonly pending: readonly EntryT[];
    readonly cachedCountFor: (args: { readonly entry: EntryT; },) => Promise<number>;
    readonly attempt: (args: { readonly entry: EntryT; },) => Promise<EntryOutcome>;
    readonly stopBeforeNext: () => boolean;
    readonly runHasStopped?: () => boolean;
    readonly beforeNext?: () => Promise<void>;
    readonly interrupted?: () => boolean;
  },
): Promise<void> {
  /**
   Attempts still to make, growing at the back as entries earn another.
   */
  const queue: EntryT[] = [...pending,];

  /**
   Entries a pause cut short and that have not been attempted since, which a stop must say were not resumed.
   */
  const cutShort = new Set<string>();

  while (queue.length > 0) {
    /**
     Next attempt, taken from the front.
     */
    const entry = nonNullishOrThrow(queue.shift(),);

    /* oxlint-disable no-await-in-loop -- a pause the caller takes between attempts, before the budget is asked */
    await beforeNext();
    /* oxlint-enable no-await-in-loop */
    if (stopBeforeNext()) {
      if (cutShort.has(entry.id,)) {
        console.log(
          `NOT RESUMED ${entry.id}: a pause cut its attempt short and the pass ended after the pause, so it was not `
            + 'attempted again in this invocation and stays resumable',
        );
      }
      return;
    }

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

    // AN ATTEMPT A PAUSE CUT SHORT EARNS NO VERDICT: the entry goes back to the front, to start again after the pause.
    if (interrupted()) {
      cutShort.add(entry.id,);
      queue.unshift(entry,);
      continue;
    }
    cutShort.delete(entry.id,);

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
      console.log(
        `REATTEMPT ${entry.id} queued: ${String(verdict.gained,)} more cache ${
          wordForCount({
            count: verdict.gained,
            one: 'record',
            many: 'records',
          },)
        } than it had, so the next attempt starts further along`,
      );
      queue.push(entry,);
    }
    if ((verdict.kind === 'stalled') && (!runEnded)) {
      console.log(
        `STALLED ${entry.id}: its ${String(verdict.cached,)} cache ${
          wordForCount({
            count: verdict.cached,
            one: 'record is',
            many: 'records are',
          },)
        } what it started with, so a further attempt in this invocation would repeat it`,
      );
    }
    if (verdict.kind === 'stopped') {
      console.log(
        `INCOMPLETE ${entry.id}: stage-local work remains; whole entry will not restart in this invocation`,
      );
    }
  }
}

//endregion Entry attempt queue
