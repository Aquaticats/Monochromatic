/**
 Tests for the order a run attempts its entries in.

 `entry-reattempt.ts` decides whether ONE attempt earned another, and its own
 tests cover that arithmetic. This covers the sequence those verdicts produce,
 which is a separate thing that can be wrong on its own: a correct verdict
 driven by a loop that never comes back settles nothing, and a loop that comes
 back too eagerly spends a three-day budget on one entry.

 THE ORDERING CASE IS THE ONE THAT WOULD COST MOST IF WRONG. A re-attempt that
 jumped the queue would let the largest entry in the corpus take attempt after
 attempt while entries that fit in one were never tried at all, which is worse
 than the cap this replaced: the cap at least moved on.

 Every effect is injected, so no case here reads a corpus, calls a provider,
 or waits on anything.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  divertingConsoleLog,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { runAttemptQueue, } from '../../dist/final/node/index.mjs';

/**
 Scheduler disposition returned by scripted attempt.
 */
type AttemptOutcome =
  | { readonly kind: 'settled'; }
    | { readonly kind: 'resumable-failure'; }
    | { readonly kind: 'stopped'; };

/**
 What one stub entry does on each successive attempt.
 */
type Script = {
  /**
   Entry id.
   */
  readonly id: string;

  /**
   Cached slice count after each attempt, read in order.
   */
  readonly cachedAfter: readonly number[];

  /**
   Attempt number, one-based, that settles this entry; zero for never.
   */
  readonly settlesOn: number;

  /**
   Attempt number, one-based, that stops whole-entry retry; zero or absent for never.
   */
  readonly stopsOn?: number;
};

/**
 Builds a queue harness driving `runAttemptQueue` from a script.

 @param scripts - what each entry does, in the order they are pending

 @param stopAfter - attempts to allow before the budget ends the run; zero
 allows every attempt

 @param runStopsAfter - attempts after which the run reads as stopped during
 an attempt, so the entry earns no second attempt; absent leaves the run never
 stopped

 @returns Order attempts were made in, once the queue has drained

 @example
 ```ts
 const order = await attemptOrder({ scripts, },);
 ```
 */
async function attemptOrder(
  {
    scripts,
    stopAfter = 0,
    runStopsAfter,
  }: {
    readonly scripts: readonly Script[];
    readonly stopAfter?: number;
    readonly runStopsAfter?: number;
  },
): Promise<readonly string[]> {
  /**
   Ids in the order they were attempted.
   */
  const order: string[] = [];

  /**
   How many times each entry has been attempted so far.
   */
  const tries = new Map<string, number>();

  /**
   Script for one id.
   */
  function scriptFor({ id, }: { readonly id: string; },): Script {
    /**
     Matching script, which the cases always provide.
     */
    const found = scripts.find(function byId(script,): boolean {
      return script.id === id;
    },);

    if (found === undefined)
      throw new Error(`no script for ${id}`,);
    return found;
  }

  await runAttemptQueue({
    pending: scripts.map(function toEntry({ id, },): { readonly id: string; } {
      return { id, };
    },),

    cachedCountFor: async function cachedCountFor({ entry, },): Promise<number> {
      /**
       Attempts made against this entry so far.
       */
      const made = tries.get(entry.id,) ?? 0;

      // Before the first attempt nothing is cached; afterwards the script says.
      if (made === 0)
        return 0;
      return scriptFor({ id: entry.id, },)
        .cachedAfter[made - 1] ?? 0;
    },

    stopBeforeNext: function stopBeforeNext(): boolean {
      if (stopAfter === 0)
        return false;
      return order.length >= stopAfter;
    },

    // Conditional spread keeps the hook absent for the cases that never stop the run during an attempt.
    ...((runStopsAfter === undefined)
      ? {}
      : {
        runHasStopped: function runHasStopped(): boolean {
          return order.length >= runStopsAfter;
        },
      }),

    attempt: async function attempt({ entry, },): Promise<AttemptOutcome> {
      order.push(entry.id,);

      /**
       Attempt number this is, one-based.
       */
      const made = (tries.get(entry.id,) ?? 0) + 1;
      tries.set(
        entry.id,
        made,
      );
      const script = scriptFor({ id: entry.id, },);
      if (script.settlesOn === made)
        return { kind: 'settled', };
      if (script.stopsOn === made)
        return { kind: 'stopped', };
      return { kind: 'resumable-failure', };
    },
  },);

  return order;
}

await describe({
  name: runAttemptQueue.name,
  children: [
    it({
      name: 'ATTEMPTS EVERY ENTRY ONCE BEFORE ANY RE-ATTEMPT, which is the '
        + 'property that keeps one oversized entry from spending the whole '
        + 'budget while entries that fit in a single attempt are never tried',
      fn: async () => {
        expect(
          await attemptOrder({
            scripts: [
              {
                id: 'mainecoon',
                cachedAfter: [
                  5,
                  5,
                ],
                settlesOn: 0,
              },
              {
                id: 'tabby',
                cachedAfter: [0,],
                settlesOn: 0,
              },
              {
                id: 'whiskers',
                cachedAfter: [3,],
                settlesOn: 1,
              },
            ],
          },),
        ).toEqual([
          'mainecoon',
          'tabby',
          'whiskers',
          'mainecoon',
        ],);
      },
    },),

    it({
      name: 'KEEPS RE-ATTEMPTING while an entry buys slices, so an entry too '
        + 'large for one attempt settles across a sequence of them rather than '
        + 'needing a relaunch that would move the build under its cache',
      fn: async () => {
        expect(
          await attemptOrder({
            scripts: [
              {
                id: 'siamese',
                cachedAfter: [
                  45,
                  64,
                  90,
                ],
                settlesOn: 3,
              },
            ],
          },),
        ).toEqual([
          'siamese',
          'siamese',
          'siamese',
        ],);
      },
    },),

    it({
      name: 'DOES NOT RESTART WHOLE ENTRY after stage-local incomplete result even when cache grew',
      fn: async () => {
        expect(
          await attemptOrder({
            scripts: [{
              id: 'naturalness-rejected',
              cachedAfter: [13,],
              settlesOn: 0,
              stopsOn: 1,
            },],
          },),
        ).toEqual(['naturalness-rejected',],);
      },
    },),

    it({
      name: 'DROPS an entry that bought nothing, which is the stop condition: '
        + 'no progress guarantee holds, so an entry whose attempt cached '
        + 'nothing would otherwise repeat until the budget was gone',
      fn: async () => {
        expect(
          await attemptOrder({
            scripts: [
              {
                id: 'stuck',
                cachedAfter: [0,],
                settlesOn: 0,
              },
            ],
          },),
        ).toEqual(['stuck',],);
      },
    },),

    it({
      name: 'STOPS on the budget without attempting what is left, so a run '
        + 'that has spent its time starts nothing new',
      fn: async () => {
        expect(
          await attemptOrder({
            scripts: [
              {
                id: 'first',
                cachedAfter: [5,],
                settlesOn: 1,
              },
              {
                id: 'second',
                cachedAfter: [5,],
                settlesOn: 1,
              },
              {
                id: 'third',
                cachedAfter: [5,],
                settlesOn: 1,
              },
            ],
            stopAfter: 2,
          },),
        ).toEqual([
          'first',
          'second',
        ],);
      },
    },),
    it({
      name: 'RESUMES AN ENTRY A WAIT INTERRUPTED at the front of the queue after the wait, reading no verdict for the '
        + 'interrupted attempt: no REATTEMPT, no STALLED, no INCOMPLETE line, and the entries behind it '
        + 'keep their order',
      fn: async (ctx,) => {
        /**
         What happened, in order: attempts by id, and the waits taken before an attempt.
         */
        const events: string[] = [];
        /**
         Whether a wait is pending after the first attempt.
         */
        const pending = { wait: false, };
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await runAttemptQueue({
          pending: [{ id: 'tabby', }, { id: 'mainecoon', },],
          cachedCountFor: async () => 0,
          stopBeforeNext: () => false,
          beforeNext: async function beforeNext(): Promise<void> {
            if (pending.wait) {
              events.push('wait',);
              pending.wait = false;
            }
          },
          interrupted: function interrupted(): boolean {
            return pending.wait;
          },
          attempt: async function attempt({ entry, },): Promise<AttemptOutcome> {
            events.push(entry.id,);
            // The first attempt is cut by a wait the caller takes before the next one.
            if (events.length === 1)
              pending.wait = true;
            return { kind: 'settled', };
          },
        },);
        expect(events,).toEqual(['tabby', 'wait', 'tabby', 'mainecoon',],);
        expect(printed.lines,).toEqual([],);
      },
    },),

    it({
      name: 'ENDS WITHOUT RESUMING when the budget is gone after the wait, since the wait came before the stop check, '
        + 'and names the entry the pause cut short as not attempted again',
      fn: async (ctx,) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        /**
         Attempts made.
         */
        const attempts: string[] = [];
        /**
         Whether the wait has been taken.
         */
        const waited = { done: false, };
        await runAttemptQueue({
          pending: [{ id: 'tabby', },],
          cachedCountFor: async () => 0,
          stopBeforeNext: () => waited.done,
          beforeNext: async function beforeNext(): Promise<void> {
            if (attempts.length > 0)
              waited.done = true;
          },
          interrupted: () => attempts.length === 1,
          attempt: async function attempt({ entry, },): Promise<AttemptOutcome> {
            attempts.push(entry.id,);
            return { kind: 'settled', };
          },
        },);
        expect(attempts,).toEqual(['tabby',],);
        expect(printed.lines,).toEqual([
          'NOT RESUMED tabby: a pause cut its attempt short and the pass ended after the pause, so it was not '
            + 'attempted again in this invocation and stays resumable',
        ],);
      },
    },),

    it({
      name: 'NAMES NO ENTRY THE PASS ENDS ON AFTER IT WAS CUT SHORT AND THEN ATTEMPTED AGAIN, even one that earned a '
        + 'second attempt and sits at the front when the budget ends the pass: the REATTEMPT line stands alone',
      fn: async (ctx,) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        /**
         Attempts made.
         */
        const attempts: string[] = [];
        await runAttemptQueue({
          pending: [{ id: 'tabby', },],
          // The second attempt caches one record, so the entry is queued again behind nothing.
          cachedCountFor: async () => (attempts.length >= 2) ? 1 : 0,
          // The budget ends the pass on the third ask, with the entry at the front.
          stopBeforeNext: () => attempts.length === 2,
          // The first attempt is cut short by a pause; the second is not.
          interrupted: () => attempts.length === 1,
          attempt: async function attempt({ entry, },): Promise<AttemptOutcome> {
            attempts.push(entry.id,);
            return { kind: 'resumable-failure', };
          },
        },);
        expect(attempts,).toEqual(['tabby', 'tabby',],);
        expect(printed.lines,).toEqual([
          'REATTEMPT tabby queued: 1 more cache record than it had, so the next attempt starts further along',
        ],);
      },
    },),

    it({
      name: 'NAMES NO ENTRY WHEN THE BUDGET ENDS THE PASS WITH NO PAUSE IN PLAY, since nothing was cut short',
      fn: async (ctx,) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await runAttemptQueue({
          pending: [{ id: 'tabby', },],
          cachedCountFor: async () => 0,
          stopBeforeNext: () => true,
          attempt: async (): Promise<AttemptOutcome> => ({ kind: 'settled', }),
        },);
        expect(printed.lines,).toEqual([],);
      },
    },),

    it({
      name: 'QUEUES NO SECOND ATTEMPT FOR AN ENTRY THE RUN STOPPED DURING, even one that cached more than it had, '
        + 'since the caller names the stop and a further attempt would meet the same refusal',
      fn: async () => {
        expect(
          await attemptOrder({
            scripts: [
              {
                id: 'mainecoon',
                cachedAfter: [
                  5,
                  5,
                ],
                settlesOn: 0,
              },
            ],
            runStopsAfter: 1,
          },),
        ).toEqual(['mainecoon',],);
      },
    },),
  ],
},);
