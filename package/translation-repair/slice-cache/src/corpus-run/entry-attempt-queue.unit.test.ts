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
} from '@monochromatic-dev/test-harness-core/ts';

import {
  divertingConsoleLog,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { runAttemptQueue, } from '../../dist/final/node/index.mjs';

/**
 Instant the injected clock reads, so no case waits on the real one.
 */
const PRINTED_AT = new Date('2026-10-22T08:10:11.123Z',);

/**
 The clock time every line a case prints carries.
 */
const AT = 'at=2026-10-22T08:10:11.123Z';

/**
 Clock that reads the same instant every time it is asked.

 @returns The instant the line is printed at

 @example
 ```ts
 await runAttemptQueue({ pending, cachedCountFor, attempt, stopBeforeNext, attemptOf, wallClock: fixedWallClock, },);
 ```
 */
function fixedWallClock(): Date {
  return PRINTED_AT;
}

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

    attemptOf: function attemptOf({ entry, },): number {
      return tries.get(entry.id,) ?? 0;
    },

    wallClock: fixedWallClock,

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
      name: 'PRINTS NOTHING WHEN THE BUDGET ENDS THE PASS BEFORE AN ENTRY STARTS, since no pause cuts an attempt short '
        + 'and nothing is named as not resumed',
      fn: async (ctx,) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await runAttemptQueue({
          pending: [{ id: 'tabby', },],
          cachedCountFor: async () => 0,
          stopBeforeNext: () => true,
          attemptOf: () => 0,
          wallClock: fixedWallClock,
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

    it({
      name: 'STAMPS REATTEMPT, STALLED AND INCOMPLETE with the clock time and the attempt each closed, the attempt being '
        + 'the number the caller counts (what attempts.json holds, here four earlier ones in, not a count this queue '
        + 'keeps), so the line of one attempt never reads as another\'s (2026-10-22, XingZ60: REATTEMPT, then attempt 2 '
        + 'under a fresh cap)',
      fn: async (ctx,) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        /**
         Attempts made, by id.
         */
        const made: string[] = [];
        /**
         Cached slices after each attempt, by the order attempts are made in.
         */
        const cachedAfterAttempt = [
          310,
          310,
          0,
        ];
        /**
         Attempts the caller counted for the entries in earlier invocations.
         */
        const EARLIER = 4;
        await runAttemptQueue({
          pending: [{ id: 'siamese', }, { id: 'sphynx', },],
          cachedCountFor: async function cachedCountFor(): Promise<number> {
            // Before each attempt nothing beyond what the last one left; after it, the script.
            return made.length === 0 ? 0 : (cachedAfterAttempt[made.length - 1] ?? 0);
          },
          stopBeforeNext: () => false,
          attemptOf: function attemptOf({ entry, },): number {
            return EARLIER + made.filter(function isThis(id,): boolean {
              return id === entry.id;
            },).length;
          },
          wallClock: fixedWallClock,
          attempt: async function attempt({ entry, },): Promise<AttemptOutcome> {
            made.push(entry.id,);
            // The third attempt (siamese's second) stops on stage-local work; the first two fail with a resumable failure.
            return (made.length === 3) ? { kind: 'stopped', } : { kind: 'resumable-failure', };
          },
        },);
        expect(printed.lines,).toEqual([
          `REATTEMPT siamese queued ${AT} attempt=5: 310 more cache records than it had, so the next attempt starts `
            + 'further along',
          `STALLED sphynx ${AT} attempt=5: its 310 cache records are what it started with, so a further attempt in `
            + 'this invocation would repeat it',
          `INCOMPLETE siamese ${AT} attempt=6: stage-local work remains; whole entry will not restart in this invocation`,
        ],);
      },
    },),
  ],
},);
