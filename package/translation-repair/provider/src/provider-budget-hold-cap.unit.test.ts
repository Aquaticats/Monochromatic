/**
 Tests that a hold a refusal states for itself is capped at the longest hold the run can use (F9 of the re-check of the
 second fix round).

 A provider's `Retry-After` header can state any number of seconds: `99999999999` is 99,999,999,999,000 ms, and the hold
 that wait sets keeps the provider out of routing for about 3,170 years while the wait for the hold polls once a second.
 No timer overflows, since a hold is a timestamp and `waitOutHold` (`budget-hold-wait.ts`) sleeps it out in slices of at
 most `HOLD_POLL_MS`, so no delay past the 2,147,483,647 ms that `setTimeout` takes ever reaches it, which is why the
 largest whole number is a valid bound. The run's hard cap is the longest a
 hold can matter, so a hold past it is cut to it and the stated value is logged once, so the cut shows in the log with
 the number the provider gave.

 Fixtures are cat-themed invention; every meter is a stub and no HTTP is made.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import {
  createProviderBudgets,
  HoldBoundError,
} from '../dist/final/node/index.mjs';
import { consoleLinesDuring, } from './neuralwatt-ledger.test-fixture.ts';

/**
 Abort signal every read here carries.
 */
const SIGNAL = new AbortController().signal;

/**
 Longest hold these cases allow, in milliseconds.
 */
const CAP_MS = 5_000;

/**
 A wait a header can state that no run outlasts: 99,999,999,999 seconds.
 */
const STATED_MS = 99_999_999_999_000;

/**
 Meters that all read wet, whatever is asked.
 */
const WET_METERS = {
  synthetic: {
    quotas: async () => ({
      weekly: { percentRemaining: 62, nextRegenAt: '2026-10-21T13:24:00.000Z', },
    }),
  },
  hyper: { credits: async () => ({ balance: 243, }), },
  openrouter: {
    credits: async () => ({ purchasedUsd: 100, usedUsd: 40, remainingUsd: 60, }),
  },
};

/**
 Budgets over the wet meters, on a clock that stands still, with the cap these cases use.

 @returns Budgets whose holds are cut to {@link CAP_MS}

 @example
 ```ts
 const budgets = cappedBudgets();
 ```
 */
function cappedBudgets(): ReturnType<typeof createProviderBudgets> {
  return createProviderBudgets({
    ...WET_METERS,
    maxHoldMs: CAP_MS,
    // Shorter than the cap, so the meter's own hold never decides what the cases read.
    rateLimitBackoffMs: 1_000,
    now: () => 10_000,
  },);
}

await describe({
  name: 'the cap on a hold a refusal states',
  // One case at a time: each captures the console, which is one stub for the whole process.
  concurrency: 1,
  children: [
    it({
      name: 'CUTS A STATED WAIT PAST THE CAP TO THE CAP, and logs the stated value once however often the provider '
        + 'refuses',
      fn: async ctx => {
        /**
         Hold left on the provider after two refusals that each state the huge wait.
         */
        const held: { left?: number; } = {};
        const lines = await consoleLinesDuring({
          run: async function refuseTwice(): Promise<void> {
            const budgets = cappedBudgets();
            await budgets.markRefused({ provider: 'hyper', signal: SIGNAL, statedWaitMs: STATED_MS, },);
            await budgets.markRefused({ provider: 'hyper', signal: SIGNAL, statedWaitMs: STATED_MS, },);
            held.left = budgets.holds().hyper;
          },
          sinon: ctx.sinon,
        },);

        expect(held.left,).toBe(CAP_MS,);
        expect(lines.filter(function namesTheStatedValue(line,): boolean {
          return line.includes(String(STATED_MS,),) && line.includes('capped',);
        },).length,).toBe(1,);
      },
    },),

    it({
      name: 'CUTS A STATED WAIT OF A REFUSAL FOR PAYMENT THE SAME WAY',
      fn: async ctx => {
        /**
         Hold left on the provider after the payment refusal.
         */
        const held: { left?: number; } = {};
        await consoleLinesDuring({
          run: async function refuseForPayment(): Promise<void> {
            const budgets = cappedBudgets();
            await budgets.markRefused({
              provider: 'openrouter',
              signal: SIGNAL,
              paymentRequired: true,
              statedWaitMs: STATED_MS,
            },);
            held.left = budgets.holds().openrouter;
          },
          sinon: ctx.sinon,
        },);

        expect(held.left,).toBe(CAP_MS,);
      },
    },),

    it({
      name: 'LEAVES A STATED WAIT INSIDE THE CAP AS THE PROVIDER STATED IT, and logs no cut (the control arm)',
      fn: async ctx => {
        /**
         Hold left on the provider after a refusal that states a short wait.
         */
        const held: { left?: number; } = {};
        const lines = await consoleLinesDuring({
          run: async function refuseShort(): Promise<void> {
            const budgets = cappedBudgets();
            await budgets.markRefused({ provider: 'hyper', signal: SIGNAL, statedWaitMs: 3_000, },);
            held.left = budgets.holds().hyper;
          },
          sinon: ctx.sinon,
        },);

        expect(held.left,).toBe(3_000,);
        expect(lines.some(function namesACut(line,): boolean {
          return line.includes('capped',);
        },),).toBe(false,);
      },
    },),

    // ONE CASE PER BOUND: the type says number and nothing at run time did, so
    // Infinity reintroduced the unbounded hold the bound exists to end and NaN made the cut return NaN. A case that
    // looped over the six bad values stopped at the first failure and hid the rest.
    ...([
      [Number.POSITIVE_INFINITY, 'Infinity',],
      [Number.NEGATIVE_INFINITY, '-Infinity',],
      [Number.NaN, 'NaN',],
      [0, '0',],
      [-1, '-1',],
      [1.5, '1.5',],
    ] as const).map(function refusedBoundCase([maxHoldMs, shown,],) {
      return it({
        name: `REFUSES A BOUND OF ${shown} when the budgets are built, with the class that names it and `
          + 'the value it was given',
        fn: async () => {
          /**
           What the build threw for this bound, or nothing when it built.
           */
          let thrown: unknown;
          try {
            createProviderBudgets({ ...WET_METERS, maxHoldMs, },);
          }
          catch (error) {
            thrown = error;
          }

          expect(thrown instanceof HoldBoundError,).toBe(true,);
          if (!(thrown instanceof HoldBoundError))
            throw new Error(`refused ${shown} by construction`,);
          expect(thrown.name,).toBe('HoldBoundError',);
          expect(thrown.message,).toContain(shown,);
          expect(thrown.message,).toContain('maxHoldMs',);
        },
      },);
    },),

    it({
      name: 'BUILDS WITH THE SMALLEST AND THE LARGEST WHOLE BOUNDS (control arm): one millisecond is a bound, and '
        + 'so is the largest integer a number holds exactly, which the claude client cases use',
      fn: async () => {
        expect(() => {
          createProviderBudgets({ ...WET_METERS, maxHoldMs: 1, },);
          createProviderBudgets({ ...WET_METERS, maxHoldMs: Number.MAX_SAFE_INTEGER, },);
        },).not.toThrow();
      },
    },),
  ],
},);
