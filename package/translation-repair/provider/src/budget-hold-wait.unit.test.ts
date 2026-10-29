/**
 Tests the budget reading that waits out a refusal hold before calling every
 provider dry.

 THE CASE IS THE PIN PASS OF 2026-09-02: two 429 holds, both meters
 wet, and every remaining entry failed inside one second because the holds
 were read as empty meters. Here the reading waits out the shortest hold and
 reads again, and ends the run only when nothing a wait could change is left.

 NO CASE SLEEPS: every reading runs on a fake clock that the case ticks past its holds, and a reading still waiting
 after the tick is aborted, so a wait that does not end fails the case and does not hang it (ledger B996). The fake
 time a reading took is asserted where the case is about a wait, since the reads of the budgets alone cannot tell a
 hold that was waited out from one that was skipped. The timers are process-wide, so the suites that fake them run one
 case at a time.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  type DisposableSandbox,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ProviderRecord, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type BudgetView,
  EveryProviderDryError,
  HOLD_POLL_MS,
  NOBODY_REFUSED,
  type NeuralWattDryness,
  type NeuralWattReturn,
  type ProviderBudgets,
  readBudgetsPastHolds,
  shortestHold,
  waitOutHold,
  type WritingReach,
} from '../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { SEAT_SYNTHETIC_VISION_WITHHELD, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  fakeClockStarted,
  waitedOutOnFakeClock,
} from './fake-clock-hold-wait.test-fixture.ts';

/**
 Model named in the log line.
 */
const MODEL_ID = SEAT_SYNTHETIC_VISION_WITHHELD;

/**
 Milliseconds of the slice every reading here sleeps a hold in, small so that a tick of the fake clock passes a hold in a
 few steps.
 */
const POLL_MS = 1;

/**
 What a case that is not waiting yet reads from {@link settledNow}.
 */
const STILL_WAITING = Symbol('the promise has not settled yet',);

/**
 Options a reading is made with that a case names itself: the rest (the model, the abort and the slice) the helpers give.
 */
type ReadingOptions = Omit<Parameters<typeof readBudgetsPastHolds>[0], 'modelId' | 'pollMs' | 'signal'>;

/**
 Reads a promise as it stands, without waiting for it.

 @param promise - promise read, which may be settled or not

 @returns Its value when it has settled, {@link STILL_WAITING} when it has not

 @example
 ```ts
 expect(await settledNow({ promise: sleeping, },),).toBe(STILL_WAITING,);
 ```
 */
async function settledNow<ValueT,>(
  { promise, }: { readonly promise: Promise<ValueT>; },
): Promise<ValueT | typeof STILL_WAITING> {
  return await Promise.race([promise, Promise.resolve<typeof STILL_WAITING>(STILL_WAITING,),],);
}

/**
 Builds a budget view that reads a scripted sequence of views and reports
 fixed holds.

 @param views - what successive reads return, the last repeating

 @param holds - what the holds report

 @param holdsByAsk - what successive asks for the holds report, the last repeating, in place of `holds`, for holds that
 end while a wait goes on

 @param neuralwatt - the NeuralWatt day the budgets report to a refusal that ends the run

 @param neuralwattAfter - the day the budgets report once this many reads have happened, in place of `neuralwatt`, for a
 day that changes while a hold is waited out

 @param neuralwattHoldIsReturn - whether a NeuralWatt hold stands for a spent day's return, which the budgets then
 report only when it lies inside the bound the ask names (`returnBoundMs`), as the real budgets do

 @param dryness - why NeuralWatt reads dry, which the budgets report to a refusal that ends the run; none when absent

 @returns Budget view plus counts of reads and hold asks

 @example
 ```ts
 const { budgets, reads, } = scriptedBudgets({ views: [ALL_DRY, SYNTHETIC_BACK,], holds: { synthetic: 5, hyper: 20, openrouter: 0, }, },);
 ```
 */
function scriptedBudgets(
  {
    views,
    holds,
    holdsByAsk,
    neuralwatt = { kind: 'unanchored', },
    neuralwattAfter,
    neuralwattHoldIsReturn = false,
    dryness,
  }: {
    readonly views: readonly BudgetView[];
    readonly holds: ProviderRecord<number>;
    readonly holdsByAsk?: readonly ProviderRecord<number>[];
    readonly neuralwatt?: NeuralWattReturn;
    readonly neuralwattAfter?: { readonly reads: number; readonly day: NeuralWattReturn; };
    readonly neuralwattHoldIsReturn?: boolean;
    readonly dryness?: NeuralWattDryness;
  },
): {
  readonly budgets: ProviderBudgets;
  readonly reads: { count: number; };
  readonly holdAsks: { count: number; };
} {
  /**
   How many reads happened.
   */
  const reads = { count: 0, };
  /**
   How many times the holds were asked for.
   */
  const holdAsks = { count: 0, };
  return {
    reads,
    holdAsks,
    budgets: {
      read: async function read(): Promise<BudgetView> {
        /**
         Scripted view for this read, the last one repeating.
         */
        const view = views[Math.min(
          reads.count,
          views.length - 1,
        )];
        reads.count += 1;
        if (view === undefined)
          throw new Error('scripted budgets need at least one view',);
        return view;
      },
      markRefused: async function markRefused(): Promise<void> {
        throw new Error('not asked here',);
      },
      holds: function holdsNow({ returnBoundMs = Number.POSITIVE_INFINITY, }: { readonly returnBoundMs?: number; } = {},
      ): ProviderRecord<number> {
        /**
         The holds this ask reads, the scripted sequence's entry for it or the fixed ones.
         */
        const asked = holdsByAsk?.[Math.min(
          holdAsks.count,
          holdsByAsk.length - 1,
        )] ?? holds;
        holdAsks.count += 1;
        return {
          ...asked,
          neuralwatt: (neuralwattHoldIsReturn && (asked.neuralwatt > returnBoundMs)) ? 0 : asked.neuralwatt,
        };
      },
      // Conditional spread keeps the double without a cause where the case names none.
      ...((dryness === undefined) ? {} : { neuralwattDryness: () => dryness, }),
      neuralwattReturn: function neuralwattReturn(): NeuralWattReturn {
        return ((neuralwattAfter !== undefined) && (reads.count >= neuralwattAfter.reads))
          ? neuralwattAfter.day
          : neuralwatt;
      },
    },
  };
}

/**
 Every provider reading dry.
 */
const ALL_DRY: BudgetView = {
  neuralwatt: true,
  synthetic: true,
  hyper: true,
  bedrock: true,
  deepseek: true,
  xiaomi: true,
  anthropic: true,
  openrouter: true,
};

/**
 The first provider back, the others still dry.
 */
const SYNTHETIC_BACK: BudgetView = {
  neuralwatt: true,
  synthetic: false,
  hyper: true,
  bedrock: true,
  deepseek: true,
  xiaomi: true,
  anthropic: true,
  openrouter: true,
};

/**
 NeuralWatt back, the others still dry.
 */
const NEURALWATT_BACK: BudgetView = { ...ALL_DRY, neuralwatt: false, };

/**
 Every provider but Xiaomi serves a model seated in a writing bench, as the live roster has it: Xiaomi serves one
 checker seat (`xiaomi/mimo-v2.6-flash`) and nothing that writes.
 */
const XIAOMI_CHECKER_ONLY: WritingReach = {
  neuralwatt: true,
  synthetic: true,
  hyper: true,
  bedrock: true,
  deepseek: true,
  xiaomi: false,
  anthropic: true,
  openrouter: true,
};

/**
 Every provider serves a model seated in a writing bench, Xiaomi included.
 */
const XIAOMI_WRITES: WritingReach = { ...XIAOMI_CHECKER_ONLY, xiaomi: true, };

/**
 Every provider dry but Xiaomi, whose meter reads wet always.
 */
const ONLY_XIAOMI_WET: BudgetView = { ...ALL_DRY, xiaomi: false, };

/**
 Nobody held.
 */
const NO_HOLDS: ProviderRecord<number> = {
  neuralwatt: 0,
  synthetic: 0,
  hyper: 0,
  bedrock: 0,
  deepseek: 0,
  xiaomi: 0,
  anthropic: 0,
  openrouter: 0,
};

/**
 What the reading returns, made on a fake clock that passes the holds the case scripts.

 @param sinon - case's own sandbox, which fakes the timers
 @param holdMs - total of the waits the reading takes
 @param options - what the case names of the reading: the budgets, the refusal and the rest

 @returns The view, and the fake milliseconds the reading slept before it returned

 @example
 ```ts
 const { view, waitedMs, } = await viewAfterHolds({ sinon: ctx.sinon, holdMs: 5, options: { budgets, refused: NOBODY_REFUSED, }, },);
 ```
 */
async function viewAfterHolds(
  {
    sinon,
    holdMs,
    options,
  }: {
    readonly sinon: DisposableSandbox;
    readonly holdMs: number;
    readonly options: ReadingOptions;
  },
): Promise<{ readonly view: BudgetView; readonly waitedMs: number; }> {
  /**
   The reading's result and the time it took.
   */
  const { value, elapsedMs, } = await waitedOutOnFakeClock({
    sinon,
    holdMs,
    pollMs: POLL_MS,
    reading: async ({ signal, },) =>
      await readBudgetsPastHolds({
        ...options,
        modelId: MODEL_ID,
        signal,
        pollMs: POLL_MS,
      },),
  },);
  return {
    view: value,
    waitedMs: elapsedMs,
  };
}

/**
 What the run-ending read throws, made on a fake clock that passes the holds the case scripts.

 @param sinon - case's own sandbox, which fakes the timers
 @param holdMs - total of the waits the reading takes before it refuses
 @param options - what the case names of the reading: the budgets, the refusal and the rest

 @returns The thrown value, and the fake milliseconds the reading slept before it threw

 @throws Error when the read returned instead of throwing

 @example
 ```ts
 const { thrown, } = await refusalAfterHolds({ sinon: ctx.sinon, holdMs: 0, options: { budgets, refused: NOBODY_REFUSED, }, },);
 ```
 */
async function refusalAfterHolds(
  {
    sinon,
    holdMs,
    options,
  }: {
    readonly sinon: DisposableSandbox;
    readonly holdMs: number;
    readonly options: ReadingOptions;
  },
): Promise<{ readonly thrown: unknown; readonly waitedMs: number; }> {
  /**
   What the reading threw and the time it took.
   */
  const { value, elapsedMs, } = await waitedOutOnFakeClock({
    sinon,
    holdMs,
    pollMs: POLL_MS,
    reading: async ({ signal, },) =>
      await rejectionOf(() => readBudgetsPastHolds({
        ...options,
        modelId: MODEL_ID,
        signal,
        pollMs: POLL_MS,
      },),),
  },);
  return {
    thrown: value,
    waitedMs: elapsedMs,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: shortestHold.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES the first hold to end and reports zero when nobody is held',
          fn: async () => {
            expect(shortestHold({
              holds: {
                neuralwatt: 0,
                synthetic: 0,
                hyper: 4_000,
                bedrock: 0,
                deepseek: 0,
                xiaomi: 0,
                anthropic: 0,
                openrouter: 0,
              },
            },),).toBe(4_000,);
            expect(shortestHold({
              holds: {
                neuralwatt: 0,
                synthetic: 300,
                hyper: 4_000,
                bedrock: 0,
                deepseek: 0,
                xiaomi: 0,
                anthropic: 0,
                openrouter: 900,
              },
            },),).toBe(300,);
            expect(shortestHold({ holds: NO_HOLDS, },),).toBe(0,);
          },
        },),
      ],
    },),

    describe({
      name: waitOutHold.name,
      // The cases fake the process-wide timers, so they run one at a time.
      concurrency: 1,
      children: [
        it({
          name: 'SLEEPS the hold out in poll-sized steps and ENDS WITH THE ABORT REASON when the call is '
            + 'aborted, before a step and after the last one alike',
          fn: async ctx => {
            /**
             A wait of 6 ms in steps of 2, which sleeps exactly the hold.
             */
            const slept = await waitedOutOnFakeClock({
              sinon: ctx.sinon,
              holdMs: 6,
              pollMs: 2,
              reading: async ({ signal, },) =>
                await waitOutHold({
                  ms: 6,
                  signal,
                  pollMs: 2,
                },),
            },);
            expect(slept.elapsedMs,).toBe(6,);

            /**
             Abort already fired.
             */
            const aborted = new AbortController();
            aborted.abort(new Error('caller gave up',),);
            /**
             What the wait threw, and the time it took before it threw.
             */
            const early = await waitedOutOnFakeClock({
              sinon: ctx.sinon,
              holdMs: 50,
              pollMs: 1,
              reading: async () =>
                await rejectionOf(() => waitOutHold({
                  ms: 50,
                  signal: aborted.signal,
                  pollMs: 1,
                },),),
            },);
            expect((early.value as Error).message,).toBe('caller gave up',);
            expect(early.elapsedMs,).toBe(0,);
            expect(HOLD_POLL_MS,).toBeGreaterThan(0,);

            using fake = fakeClockStarted({ sinon: ctx.sinon, },);
            /**
             Abort fired inside the second step of 4 ms, which the wait notices where that step ends and not before.
             */
            const midStep = new AbortController();
            const noticedAtStepEnd = rejectionOf(() => waitOutHold({
              ms: 10,
              signal: midStep.signal,
              pollMs: 4,
            },),);
            void Promise.allSettled([noticedAtStepEnd,],);
            await fake.clock.tickAsync(5,);
            midStep.abort(new Error('caller gave up',),);
            await fake.clock.tickAsync(2,);
            expect(await settledNow({ promise: noticedAtStepEnd, },),).toBe(STILL_WAITING,);
            await fake.clock.tickAsync(1,);
            expect(((await settledNow({ promise: noticedAtStepEnd, },)) as Error).message,).toBe('caller gave up',);

            /**
             Abort fired inside the last step, which the wait notices where it ends, after the loop.
             */
            const lastStep = new AbortController();
            const noticedAfterLastStep = rejectionOf(() => waitOutHold({
              ms: 6,
              signal: lastStep.signal,
              pollMs: 2,
            },),);
            void Promise.allSettled([noticedAfterLastStep,],);
            await fake.clock.tickAsync(5,);
            lastStep.abort(new Error('caller gave up',),);
            await fake.clock.tickAsync(1,);
            expect(((await settledNow({ promise: noticedAfterLastStep, },)) as Error).message,).toBe('caller gave up',);
          },
        },),
      ],
    },),

    describe({
      name: readBudgetsPastHolds.name,
      // The cases fake the process-wide timers, so they run one at a time.
      concurrency: 1,
      children: [
        it({
          name: 'RETURNS THE READING for a model Xiaomi does not serve when Xiaomi, wet, is the one provider not dry, '
            + 'since a model that lost its providers is a lost seat and only every provider dry ends the run',
          fn: async ctx => {
            const view: BudgetView = {
              neuralwatt: true,
              synthetic: true,
              hyper: true,
              bedrock: true,
              deepseek: true,
              xiaomi: false,
              anthropic: true,
              openrouter: true,
            };
            const { budgets, } = scriptedBudgets({ views: [view,], holds: NO_HOLDS, },);
            expect((await viewAfterHolds({
              sinon: ctx.sinon,
              holdMs: 0,
              options: {
                budgets,
                refused: NOBODY_REFUSED,
              },
            },)).view,).toEqual(view,);
            expect((await viewAfterHolds({
              sinon: ctx.sinon,
              holdMs: 0,
              options: {
                budgets,
                refused: NOBODY_REFUSED,
              },
            },)).view,).toEqual(view,);
          },
        },),
        it({
          name: 'ENDS THE RUN WITH A STOP LINE when Xiaomi, wet, serves only a checker seat and every other provider '
            + 'reads dry: a wet provider counts only if a writing bench could still reach its floor from it',
          fn: async ctx => {
            const { budgets, } = scriptedBudgets({ views: [ONLY_XIAOMI_WET,], holds: NO_HOLDS, },);
            const { thrown, } = await refusalAfterHolds({
              sinon: ctx.sinon,
              holdMs: 0,
              options: {
                budgets,
                refused: NOBODY_REFUSED,
                writing: XIAOMI_CHECKER_ONLY,
              },
            },);
            expect(thrown,).toBeInstanceOf(EveryProviderDryError,);
            const { message, } = thrown as EveryProviderDryError;
            expect(message,).toContain('Xiaomi reads wet but serves no seat that writes',);
            expect(message,).not.toContain('Xiaomi has refused a call for payment',);
            expect(message,).toContain('xiaomi wet (serves no seat that writes)',);
          },
        },),

        it({
          name: 'DOES NOT END THE RUN when Xiaomi, wet, serves a writer seat: the same reading returns, and no hold is '
            + 'asked for',
          fn: async ctx => {
            const { budgets, holdAsks, } = scriptedBudgets({ views: [ONLY_XIAOMI_WET,], holds: NO_HOLDS, },);
            expect((await viewAfterHolds({
              sinon: ctx.sinon,
              holdMs: 0,
              options: {
                budgets,
                refused: NOBODY_REFUSED,
                writing: XIAOMI_WRITES,
              },
            },)).view,).toEqual(ONLY_XIAOMI_WET,);
            expect(holdAsks.count,).toBe(0,);
          },
        },),

        it({
          name: 'WAITS OUT THE HOLDS with Xiaomi wet and checker-only, as it does with every provider dry: the all-dry '
            + 'wait engages, and the return it waited for is read',
          fn: async ctx => {
            const { budgets, reads, } = scriptedBudgets({
              views: [ONLY_XIAOMI_WET, { ...ONLY_XIAOMI_WET, neuralwatt: false, },],
              holds: NO_HOLDS,
              holdsByAsk: [{ ...NO_HOLDS, neuralwatt: 6, }, NO_HOLDS,],
            },);
            const { view, waitedMs, } = await viewAfterHolds({
              sinon: ctx.sinon,
              holdMs: 6,
              options: {
                budgets,
                refused: NOBODY_REFUSED,
                writing: XIAOMI_CHECKER_ONLY,
              },
            },);
            expect(view,).toEqual({ ...ONLY_XIAOMI_WET, neuralwatt: false, },);
            expect(reads.count,).toBe(2,);
            // The reads alone cannot tell a hold waited out from one skipped: the fake time slept can.
            expect(waitedMs,).toBe(6,);
          },
        },),

        it({
          name: 'STATES THE ACCOUNT\'S NEURALWATT DAY START in the run-ending refusal, from the budgets\' own reading '
            + 'of the decided anchor, whether the refusal comes at once or after a hold is waited out',
          fn: async ctx => {
            /**
             The XingZ6018 account: its day starts at 19:15:08 UTC.
             */
            const neuralwatt: NeuralWattReturn = {
              kind: 'at',
              msOfDay: 69_308_000,
            };
            /**
             The refusal raised at once, with no hold to wait out.
             */
            const { thrown: atOnce, } = await refusalAfterHolds({
              sinon: ctx.sinon,
              holdMs: 0,
              options: {
                budgets: scriptedBudgets({ views: [ALL_DRY,], holds: NO_HOLDS, neuralwatt, },).budgets,
                refused: NOBODY_REFUSED,
              },
            },);
            expect((atOnce as EveryProviderDryError).message,).toContain('at 19:15:08 UTC',);
            expect((atOnce as EveryProviderDryError).message,).not.toContain('00:00 UTC',);
            /**
             The refusal raised after the shortest hold ended and every provider still read dry.
             */
            const { thrown: afterWait, waitedMs, } = await refusalAfterHolds({
              sinon: ctx.sinon,
              holdMs: 5,
              options: {
                budgets: scriptedBudgets({
                  views: [ALL_DRY,],
                  holds: NO_HOLDS,
                  holdsByAsk: [{ ...NO_HOLDS, synthetic: 5, }, NO_HOLDS,],
                  neuralwatt,
                },).budgets,
                refused: NOBODY_REFUSED,
              },
            },);
            expect((afterWait as EveryProviderDryError).message,).toContain('after waiting 5ms',);
            expect((afterWait as EveryProviderDryError).message,).toContain('at 19:15:08 UTC',);
            // The message says what the reading counted; the fake time says what it slept.
            expect(waitedMs,).toBe(5,);
          },
        },),

        it({
          name: 'READS THE NEURALWATT DAY AGAIN FOR THE REFUSAL AFTER THE WAIT, so a day start the run learned while the '
            + 'hold was waited out is stated and the refusal does not say that no time of day can be stated',
          fn: async ctx => {
            /**
             The refusal raised after the shortest hold ended, with the day start learned on the second budget read.
             */
            const { thrown: afterWait, waitedMs, } = await refusalAfterHolds({
              sinon: ctx.sinon,
              holdMs: 5,
              options: {
                budgets: scriptedBudgets({
                  views: [ALL_DRY,],
                  holds: NO_HOLDS,
                  holdsByAsk: [{ ...NO_HOLDS, synthetic: 5, }, NO_HOLDS,],
                  neuralwatt: { kind: 'unanchored', },
                  neuralwattAfter: { reads: 2, day: { kind: 'at', msOfDay: 69_308_000, }, },
                },).budgets,
                refused: NOBODY_REFUSED,
              },
            },);
            const { message, } = afterWait as EveryProviderDryError;
            expect(message,).toContain('after waiting 5ms',);
            expect(message,).toContain('at 19:15:08 UTC',);
            expect(message,).not.toContain('no time of day can be stated',);
            expect(waitedMs,).toBe(5,);
          },
        },),

        it({
          name: 'SAYS A PAYMENT REFUSAL when Xiaomi reads dry itself, even though it serves nothing here',
          fn: async ctx => {
            const { budgets, } = scriptedBudgets({ views: [ALL_DRY,], holds: NO_HOLDS, },);
            const { thrown, } = await refusalAfterHolds({
              sinon: ctx.sinon,
              holdMs: 0,
              options: {
                budgets,
                refused: NOBODY_REFUSED,
              },
            },);
            const { message, } = thrown as EveryProviderDryError;
            expect(message,).toContain('Xiaomi has refused a call for payment',);
            expect(message,).toContain('xiaomi dry',);
          },
        },),

        it({
          name: 'WAITS ON NO HOLD when Xiaomi, wet but serving nothing here, is the only reading that is not dry: a '
            + 'wet provider is something to buy from, so nothing is read twice and no hold is asked for',
          fn: async ctx => {
            const view: BudgetView = { ...ALL_DRY, xiaomi: false, };
            const { budgets, reads, holdAsks, } = scriptedBudgets({
              views: [view,],
              holds: { ...NO_HOLDS, synthetic: 5, },
            },);
            const { view: read, waitedMs, } = await viewAfterHolds({
              sinon: ctx.sinon,
              holdMs: 0,
              options: {
                budgets,
                refused: NOBODY_REFUSED,
              },
            },);
            expect(read,).toEqual(view,);
            expect(reads.count,).toBe(1,);
            expect(holdAsks.count,).toBe(0,);
            expect(waitedMs,).toBe(0,);
          },
        },),

        it({
          name: 'RETURNS THE READING with the routing refusal folded in when at least one provider is '
            + 'spendable, asking for no holds',
          fn: async ctx => {
            const { budgets, reads, holdAsks, } = scriptedBudgets({
              views: [{
                neuralwatt: true,
                synthetic: false,
                hyper: false,
                bedrock: false,
                deepseek: false,
                xiaomi: false,
                anthropic: false,
                openrouter: false,
              },],
              holds: NO_HOLDS,
            },);
            expect((await viewAfterHolds({
              sinon: ctx.sinon,
              holdMs: 0,
              options: {
                budgets,
                refused: 'synthetic',
              },
            },)).view,).toEqual({
              neuralwatt: true,
              synthetic: true,
              hyper: false,
              bedrock: false,
              deepseek: false,
              xiaomi: false,
              anthropic: false,
              openrouter: false,
            },);
            expect(reads.count,).toBe(1,);
            expect(holdAsks.count,).toBe(0,);
          },
        },),

        it({
          name: 'WAITS OUT THE SHORTEST HOLD when every provider reads dry and a refusal holds one, then '
            + 'returns the second reading',
          fn: async ctx => {
            const { budgets, reads, holdAsks, } = scriptedBudgets({
              views: [ALL_DRY, SYNTHETIC_BACK,],
              holds: {
                neuralwatt: 0,
                synthetic: 5,
                hyper: 20,
                bedrock: 0,
                deepseek: 0,
                xiaomi: 0,
                anthropic: 0,
                openrouter: 0,
              },
            },);
            const { view, waitedMs, } = await viewAfterHolds({
              sinon: ctx.sinon,
              holdMs: 5,
              options: {
                budgets,
                refused: NOBODY_REFUSED,
              },
            },);
            expect(view,).toEqual(SYNTHETIC_BACK,);
            expect(reads.count,).toBe(2,);
            expect(holdAsks.count,).toBe(1,);
            // The shortest hold is 5 ms, not the 20 ms beside it: the reads alone pass with no wait at all.
            expect(waitedMs,).toBe(5,);
          },
        },),

        it({
          name: 'DOES NOT FOLD THE ROUTING REFUSAL INTO THE SECOND READING: the refuser held out is '
            + 'the one whose hold was waited out, so with the other providers dry by meter the call '
            + 'goes back to the refuser rather than ending the run after the wait',
          fn: async ctx => {
            /**
             The refuser wet by meter, the others dry by meter, on both reads;
             the first read folds the refusal in and sees every provider dry.
             */
            const { budgets, reads, holdAsks, } = scriptedBudgets({
              views: [{
                neuralwatt: true,
                synthetic: false,
                hyper: true,
                bedrock: true,
                deepseek: true,
                xiaomi: true,
                anthropic: true,
                openrouter: true,
              },],
              holds: {
                neuralwatt: 0,
                synthetic: 5,
                hyper: 0,
                bedrock: 0,
                deepseek: 0,
                xiaomi: 0,
                anthropic: 0,
                openrouter: 0,
              },
            },);
            const { view, waitedMs, } = await viewAfterHolds({
              sinon: ctx.sinon,
              holdMs: 5,
              options: {
                budgets,
                refused: 'synthetic',
              },
            },);
            expect(view,).toEqual({
              neuralwatt: true,
              synthetic: false,
              hyper: true,
              bedrock: true,
              deepseek: true,
              xiaomi: true,
              anthropic: true,
              openrouter: true,
            },);
            expect(reads.count,).toBe(2,);
            expect(holdAsks.count,).toBe(1,);
            // The refuser's hold is waited out before the second read: the reads alone pass with no wait at all.
            expect(waitedMs,).toBe(5,);
          },
        },),

        it({
          name: 'GOES BACK TO A REFUSER THE BUDGETS GAVE NO HOLD, at once, when only the routing refusal reads it '
            + 'dry: a hold of nothing has already ended, so the refuser is back (ledger B124; Uekawakuyuurei, '
            + '2026-09-04: 166 calls ended "every provider dry" while Synthetic\'s meter read wet)',
          fn: async ctx => {
            /**
             The refuser wet by meter and unheld, the others dry by meter.
             */
            const refuserBack: BudgetView = {
              neuralwatt: true,
              synthetic: false,
              hyper: true,
              bedrock: true,
              deepseek: true,
              xiaomi: true,
              anthropic: true,
              openrouter: true,
            };
            const { budgets, reads, holdAsks, } = scriptedBudgets({
              views: [refuserBack,],
              holds: NO_HOLDS,
            },);
            const { view, waitedMs, } = await viewAfterHolds({
              sinon: ctx.sinon,
              holdMs: 0,
              options: {
                budgets,
                refused: 'synthetic',
              },
            },);
            expect(view,).toEqual(refuserBack,);
            expect(reads.count,).toBe(1,);
            // Asked once: the refuser's own hold is what says it is back.
            expect(holdAsks.count,).toBe(1,);
            expect(waitedMs,).toBe(0,);
          },
        },),

        it({
          name: 'ENDS THE RUN when every provider reads dry with no hold to wait out, and when they still '
            + 'read dry after the hold that was running ended and no other hold is left',
          fn: async ctx => {
            /**
             Both cases, run side by side since neither touches the other, on one fake clock that passes the hold of the
             second.
             */
            const { value: outcomes, elapsedMs, } = await waitedOutOnFakeClock({
              sinon: ctx.sinon,
              holdMs: 5,
              pollMs: POLL_MS,
              reading: async ({ signal, },) =>
                await Promise.all(([
                  [[NO_HOLDS,], 1,],
                  [[{
                    neuralwatt: 0,
                    synthetic: 5,
                    hyper: 20,
                    bedrock: 0,
                    deepseek: 0,
                    xiaomi: 0,
                    anthropic: 0,
                    openrouter: 0,
                  }, NO_HOLDS,], 2,],
                ] as const).map(async function endsTheRun([holdsByAsk, expectedReads,],): Promise<{
                  readonly thrown: unknown;
                  readonly reads: number;
                  readonly expectedReads: number;
                }> {
                  const { budgets, reads, } = scriptedBudgets({
                    views: [ALL_DRY,],
                    holds: NO_HOLDS,
                    holdsByAsk,
                  },);
                  try {
                    await readBudgetsPastHolds({
                      budgets,
                      modelId: MODEL_ID,
                      signal,
                      refused: NOBODY_REFUSED,
                      pollMs: POLL_MS,
                    },);
                  } catch (error) {
                    return {
                      thrown: error,
                      reads: reads.count,
                      expectedReads,
                    };
                  }
                  return {
                    thrown: undefined,
                    reads: reads.count,
                    expectedReads,
                  };
                },),),
            },);
            for (const outcome of outcomes) {
              expect(outcome.thrown instanceof EveryProviderDryError,).toBe(true,);
              expect(outcome.reads,).toBe(outcome.expectedReads,);
              // The message states what was measured, so a reader can tell
              // exhaustion from refusal holds.
              expect((outcome.thrown as Error).message,)
                .toContain(
                'meters read neuralwatt dry, synthetic dry, bedrock dry, deepseek dry, xiaomi dry, anthropic dry, hyper dry, '
                  + 'openrouter dry; holds neuralwatt',
                );
            }
            /**
             The waited case names the wait it made.
             */
            const [, waited,] = outcomes;
            if (waited === undefined)
              throw new Error('the waited case did not run',);
            expect((waited.thrown as Error).message,).toContain('after waiting 5ms',);
            // The last of the two to end is the waited one, so the fake time is the hold it slept.
            expect(elapsedMs,).toBe(5,);
          },
        },),

        it({
          name: 'KEEPS WAITING WHILE A HOLD STILL RUNS: a shorter hold on a provider that still reads dry ends first, '
            + 'and the wait goes on for the NeuralWatt return behind it instead of ending the run short of it',
          fn: async ctx => {
            const { budgets, reads, holdAsks, } = scriptedBudgets({
              views: [ALL_DRY, ALL_DRY, NEURALWATT_BACK,],
              holds: NO_HOLDS,
              holdsByAsk: [
                {
                  ...NO_HOLDS,
                  hyper: 5,
                  neuralwatt: 12,
                },
                {
                  ...NO_HOLDS,
                  neuralwatt: 7,
                },
                NO_HOLDS,
              ],
            },);
            const { view, waitedMs, } = await viewAfterHolds({
              sinon: ctx.sinon,
              holdMs: 12,
              options: {
                budgets,
                refused: NOBODY_REFUSED,
              },
            },);
            expect(view,).toEqual(NEURALWATT_BACK,);
            expect(reads.count,).toBe(3,);
            expect(holdAsks.count,).toBe(2,);
            // Both holds are slept, the 5 ms of Hyper's and then the 7 ms of NeuralWatt's.
            expect(waitedMs,).toBe(12,);
          },
        },),

        it({
          name: 'BOUNDS THE SUM OF THE NEURALWATT WAITS, not each: another hold waited out for as long as the bound '
            + 'leaves nothing for a return that lies inside it alone, so the call refuses at once with the distance '
            + 'instead of waiting on (a 300 s hold then a return 250 s away, scaled to 25 ms and 15 ms under a 20 ms '
            + 'bound)',
          fn: async ctx => {
            const { budgets, holdAsks, } = scriptedBudgets({
              views: [ALL_DRY,],
              holds: NO_HOLDS,
              neuralwattHoldIsReturn: true,
              dryness: { kind: 'spent-day', dayStart: '2026-10-22T19:15:08.000Z', returnsInMs: 15, },
              // The first ask sees a return past the bound and a hold on Hyper; the second sees the return inside it.
              holdsByAsk: [
                { ...NO_HOLDS, hyper: 25, neuralwatt: 45, },
                { ...NO_HOLDS, neuralwatt: 15, },
                NO_HOLDS,
              ],
            },);
            const { thrown, waitedMs, } = await refusalAfterHolds({
              sinon: ctx.sinon,
              holdMs: 25,
              options: {
                budgets,
                refused: NOBODY_REFUSED,
                returnBoundMs: 20,
              },
            },);
            expect(thrown,).toBeInstanceOf(EveryProviderDryError,);
            expect((thrown as EveryProviderDryError).neuralwattReturnsInMs,).toBe(15,);
            expect(holdAsks.count,).toBe(2,);
            // Only Hyper's 25 ms is slept: the 15 ms return does not fit in what the bound leaves.
            expect(waitedMs,).toBe(25,);
          },
        },),

        it({
          name: 'BOUNDS THE WAITS ONLY BY THE CALL\'S ABORT: holds that never end are waited on until the call is '
            + 'aborted, which ends the wait with its reason',
          fn: async ctx => {
            using fake = fakeClockStarted({ sinon: ctx.sinon, },);
            const aborted = new AbortController();
            const { budgets, } = scriptedBudgets({
              views: [ALL_DRY,],
              holds: {
                ...NO_HOLDS,
                hyper: 1_000_000,
              },
            },);
            const reading = rejectionOf(() => readBudgetsPastHolds({
              budgets,
              modelId: MODEL_ID,
              signal: aborted.signal,
              refused: NOBODY_REFUSED,
              pollMs: POLL_MS,
            },),);
            void Promise.allSettled([reading,],);
            // The abort is the caller's own, after 30 fake milliseconds in which nothing else ended the wait.
            await fake.clock.tickAsync(30,);
            expect(await settledNow({ promise: reading, },),).toBe(STILL_WAITING,);
            aborted.abort(new Error('caller gave up',),);
            await fake.clock.tickAsync(POLL_MS,);
            /**
             What the wait threw once the abort was noticed, or the mark that it was still waiting.
             */
            const thrown = await settledNow({ promise: reading, },);
            expect(thrown,).not.toBe(STILL_WAITING,);
            expect((thrown as Error).message,).toBe('caller gave up',);
          },
        },),
      ],
    },),
  ],
},);
