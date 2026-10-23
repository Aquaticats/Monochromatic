/**
 Tests the budget reading that waits out a refusal hold before calling every
 provider dry.

 THE CASE IS THE PIN PASS OF 2026-09-02: two 429 holds, both meters
 wet, and every remaining entry failed inside one second because the holds
 were read as empty meters. Here the reading waits out the shortest hold and
 reads again, and ends the run only when nothing a wait could change is left.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

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

/**
 Abort signal that never fires.
 */
const SIGNAL = new AbortController().signal;

/**
 Model named in the log line.
 */
const MODEL_ID = SEAT_SYNTHETIC_VISION_WITHHELD;

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
 What the run-ending read throws.

 @param budgets - scripted budgets the read goes over

 @returns The thrown value, or undefined when the read returned

 @example
 ```ts
 const thrown = await refusalOfReading({ budgets, },);
 ```
 */
async function refusalOfReading(
  { budgets, }: { readonly budgets: ProviderBudgets; },
): Promise<unknown> {
  return await rejectionOf(() => readBudgetsPastHolds({
    budgets,
    modelId: MODEL_ID,
    signal: SIGNAL,
    refused: NOBODY_REFUSED,
    pollMs: 1,
  },),);
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
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SLEEPS the hold out in poll-sized steps and ENDS WITH THE ABORT REASON when the call is '
            + 'aborted, before a step and after the last one alike',
          fn: async () => {
            /**
             Monotonic clock around a short wait.
             */
            const before = performance.now();
            await waitOutHold({
              ms: 6,
              signal: SIGNAL,
              pollMs: 2,
            },);
            expect(performance.now() - before,).toBeGreaterThanOrEqual(5,);

            /**
             Abort already fired.
             */
            const aborted = new AbortController();
            aborted.abort(new Error('caller gave up',),);
            /**
             What the wait threw.
             */
            let thrown: unknown;
            try {
              await waitOutHold({
                ms: 50,
                signal: aborted.signal,
                pollMs: 1,
              },);
            } catch (error) {
              thrown = error;
            }
            expect((thrown as Error).message,).toBe('caller gave up',);
            expect(HOLD_POLL_MS,).toBeGreaterThan(0,);
          },
        },),
      ],
    },),

    describe({
      name: readBudgetsPastHolds.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RETURNS THE READING for a model Xiaomi does not serve when Xiaomi, wet, is the one provider not dry, '
            + 'since a model that lost its providers is a lost seat and only every provider dry ends the run',
          fn: async () => {
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
            expect(await readBudgetsPastHolds({
              budgets,
              modelId: MODEL_ID,
              signal: SIGNAL,
              refused: NOBODY_REFUSED,
              pollMs: 1,
            },),).toEqual(view,);
            expect(await readBudgetsPastHolds({
              budgets,
              modelId: MODEL_ID,
              signal: SIGNAL,
              refused: NOBODY_REFUSED,
              pollMs: 1,
            },),).toEqual(view,);
          },
        },),
        it({
          name: 'ENDS THE RUN WITH A STOP LINE when Xiaomi, wet, serves only a checker seat and every other provider '
            + 'reads dry: a wet provider counts only if a writing bench could still reach its floor from it',
          fn: async () => {
            const { budgets, } = scriptedBudgets({ views: [ONLY_XIAOMI_WET,], holds: NO_HOLDS, },);
            const thrown = await rejectionOf(() => readBudgetsPastHolds({
              budgets,
              modelId: MODEL_ID,
              signal: SIGNAL,
              refused: NOBODY_REFUSED,
              pollMs: 1,
              writing: XIAOMI_CHECKER_ONLY,
            },),);
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
          fn: async () => {
            const { budgets, holdAsks, } = scriptedBudgets({ views: [ONLY_XIAOMI_WET,], holds: NO_HOLDS, },);
            expect(await readBudgetsPastHolds({
              budgets,
              modelId: MODEL_ID,
              signal: SIGNAL,
              refused: NOBODY_REFUSED,
              pollMs: 1,
              writing: XIAOMI_WRITES,
            },),).toEqual(ONLY_XIAOMI_WET,);
            expect(holdAsks.count,).toBe(0,);
          },
        },),

        it({
          name: 'WAITS OUT THE HOLDS with Xiaomi wet and checker-only, as it does with every provider dry: the all-dry '
            + 'wait engages, and the return it waited for is read',
          fn: async () => {
            const { budgets, reads, } = scriptedBudgets({
              views: [ONLY_XIAOMI_WET, { ...ONLY_XIAOMI_WET, neuralwatt: false, },],
              holds: NO_HOLDS,
              holdsByAsk: [{ ...NO_HOLDS, neuralwatt: 6, }, NO_HOLDS,],
            },);
            expect(await readBudgetsPastHolds({
              budgets,
              modelId: MODEL_ID,
              signal: SIGNAL,
              refused: NOBODY_REFUSED,
              pollMs: 1,
              writing: XIAOMI_CHECKER_ONLY,
            },),).toEqual({ ...ONLY_XIAOMI_WET, neuralwatt: false, },);
            expect(reads.count,).toBe(2,);
          },
        },),

        it({
          name: 'STATES THE ACCOUNT\'S NEURALWATT DAY START in the run-ending refusal, from the budgets\' own reading '
            + 'of the decided anchor, whether the refusal comes at once or after a hold is waited out',
          fn: async () => {
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
            const atOnce = await refusalOfReading({
              budgets: scriptedBudgets({ views: [ALL_DRY,], holds: NO_HOLDS, neuralwatt, },).budgets,
            },);
            expect((atOnce as EveryProviderDryError).message,).toContain('at 19:15:08 UTC',);
            expect((atOnce as EveryProviderDryError).message,).not.toContain('00:00 UTC',);
            /**
             The refusal raised after the shortest hold ended and every provider still read dry.
             */
            const afterWait = await refusalOfReading({
              budgets: scriptedBudgets({
                views: [ALL_DRY,],
                holds: NO_HOLDS,
                holdsByAsk: [{ ...NO_HOLDS, synthetic: 5, }, NO_HOLDS,],
                neuralwatt,
              },).budgets,
            },);
            expect((afterWait as EveryProviderDryError).message,).toContain('after waiting 5ms',);
            expect((afterWait as EveryProviderDryError).message,).toContain('at 19:15:08 UTC',);
          },
        },),

        it({
          name: 'READS THE NEURALWATT DAY AGAIN FOR THE REFUSAL AFTER THE WAIT, so a day start the run learned while the '
            + 'hold was waited out is stated and the refusal does not say that no time of day can be stated',
          fn: async () => {
            /**
             The refusal raised after the shortest hold ended, with the day start learned on the second budget read.
             */
            const afterWait = await refusalOfReading({
              budgets: scriptedBudgets({
                views: [ALL_DRY,],
                holds: NO_HOLDS,
                holdsByAsk: [{ ...NO_HOLDS, synthetic: 5, }, NO_HOLDS,],
                neuralwatt: { kind: 'unanchored', },
                neuralwattAfter: { reads: 2, day: { kind: 'at', msOfDay: 69_308_000, }, },
              },).budgets,
            },);
            const { message, } = afterWait as EveryProviderDryError;
            expect(message,).toContain('after waiting 5ms',);
            expect(message,).toContain('at 19:15:08 UTC',);
            expect(message,).not.toContain('no time of day can be stated',);
          },
        },),

        it({
          name: 'SAYS A PAYMENT REFUSAL when Xiaomi reads dry itself, even though it serves nothing here',
          fn: async () => {
            const { budgets, } = scriptedBudgets({ views: [ALL_DRY,], holds: NO_HOLDS, },);
            const thrown = await refusalOfReading({ budgets, },);
            const { message, } = thrown as EveryProviderDryError;
            expect(message,).toContain('Xiaomi has refused a call for payment',);
            expect(message,).toContain('xiaomi dry',);
          },
        },),

        it({
          name: 'WAITS ON NO HOLD when Xiaomi, wet but serving nothing here, is the only reading that is not dry: a '
            + 'wet provider is something to buy from, so nothing is read twice and no hold is asked for',
          fn: async () => {
            const view: BudgetView = { ...ALL_DRY, xiaomi: false, };
            const { budgets, reads, holdAsks, } = scriptedBudgets({
              views: [view,],
              holds: { ...NO_HOLDS, synthetic: 5, },
            },);
            expect(await readBudgetsPastHolds({
              budgets,
              modelId: MODEL_ID,
              signal: SIGNAL,
              refused: NOBODY_REFUSED,
              pollMs: 1,
            },),).toEqual(view,);
            expect(reads.count,).toBe(1,);
            expect(holdAsks.count,).toBe(0,);
          },
        },),

        it({
          name: 'RETURNS THE READING with the routing refusal folded in when at least one provider is '
            + 'spendable, asking for no holds',
          fn: async () => {
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
            expect(await readBudgetsPastHolds({
              budgets,
              modelId: MODEL_ID,
              signal: SIGNAL,
              refused: 'synthetic',
              pollMs: 1,
            },),).toEqual({
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
          fn: async () => {
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
            expect(await readBudgetsPastHolds({
              budgets,
              modelId: MODEL_ID,
              signal: SIGNAL,
              refused: NOBODY_REFUSED,
              pollMs: 1,
            },),).toEqual(SYNTHETIC_BACK,);
            expect(reads.count,).toBe(2,);
            expect(holdAsks.count,).toBe(1,);
          },
        },),

        it({
          name: 'DOES NOT FOLD THE ROUTING REFUSAL INTO THE SECOND READING: the refuser held out is '
            + 'the one whose hold was waited out, so with the other providers dry by meter the call '
            + 'goes back to the refuser rather than ending the run after the wait',
          fn: async () => {
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
            expect(await readBudgetsPastHolds({
              budgets,
              modelId: MODEL_ID,
              signal: SIGNAL,
              refused: 'synthetic',
              pollMs: 1,
            },),).toEqual({
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
          },
        },),

        it({
          name: 'GOES BACK TO A REFUSER THE BUDGETS GAVE NO HOLD, at once, when only the routing refusal reads it '
            + 'dry: a hold of nothing has already ended, so the refuser is back (ledger B124; Uekawakuyuurei, '
            + '2026-09-04: 166 calls ended "every provider dry" while Synthetic\'s meter read wet)',
          fn: async () => {
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
            expect(await readBudgetsPastHolds({
              budgets,
              modelId: MODEL_ID,
              signal: SIGNAL,
              refused: 'synthetic',
              pollMs: 1,
            },),).toEqual(refuserBack,);
            expect(reads.count,).toBe(1,);
            // Asked once: the refuser's own hold is what says it is back.
            expect(holdAsks.count,).toBe(1,);
          },
        },),

        it({
          name: 'ENDS THE RUN when every provider reads dry with no hold to wait out, and when they still '
            + 'read dry after the hold that was running ended and no other hold is left',
          fn: async () => {
            /**
             Both cases, run side by side since neither touches the other.
             */
            const outcomes = await Promise.all(([
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
                  signal: SIGNAL,
                  refused: NOBODY_REFUSED,
                  pollMs: 1,
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
            },),);
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
          },
        },),

        it({
          name: 'KEEPS WAITING WHILE A HOLD STILL RUNS: a shorter hold on a provider that still reads dry ends first, '
            + 'and the wait goes on for the NeuralWatt return behind it instead of ending the run short of it',
          fn: async () => {
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
            expect(await readBudgetsPastHolds({
              budgets,
              modelId: MODEL_ID,
              signal: SIGNAL,
              refused: NOBODY_REFUSED,
              pollMs: 1,
            },),).toEqual(NEURALWATT_BACK,);
            expect(reads.count,).toBe(3,);
            expect(holdAsks.count,).toBe(2,);
          },
        },),

        it({
          name: 'BOUNDS THE SUM OF THE NEURALWATT WAITS, not each: another hold waited out for as long as the bound '
            + 'leaves nothing for a return that lies inside it alone, so the call refuses at once with the distance '
            + 'instead of waiting on (a 300 s hold then a return 250 s away, scaled to 25 ms and 15 ms under a 20 ms '
            + 'bound)',
          fn: async () => {
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
            const thrown = await rejectionOf(() => readBudgetsPastHolds({
              budgets,
              modelId: MODEL_ID,
              signal: SIGNAL,
              refused: NOBODY_REFUSED,
              pollMs: 1,
              returnBoundMs: 20,
            },),);
            expect(thrown,).toBeInstanceOf(EveryProviderDryError,);
            expect((thrown as EveryProviderDryError).neuralwattReturnsInMs,).toBe(15,);
            expect(holdAsks.count,).toBe(2,);
          },
        },),

        it({
          name: 'BOUNDS THE WAITS ONLY BY THE CALL\'S ABORT: holds that never end are waited on until the call is '
            + 'aborted, which ends the wait with its reason',
          fn: async () => {
            const aborted = new AbortController();
            const { budgets, } = scriptedBudgets({
              views: [ALL_DRY,],
              holds: {
                ...NO_HOLDS,
                hyper: 1_000_000,
              },
            },);
            setTimeout(function giveUp(): void {
              aborted.abort(new Error('caller gave up',),);
            }, 30,);
            const thrown = await rejectionOf(() => readBudgetsPastHolds({
              budgets,
              modelId: MODEL_ID,
              signal: aborted.signal,
              refused: NOBODY_REFUSED,
              pollMs: 1,
            },),);
            expect((thrown as Error).message,).toBe('caller gave up',);
          },
        },),
      ],
    },),
  ],
},);
