/**
 Tests that the `METERS` line and the routed view read a provider the run has held out the same way.

 CuspariaKLSY22 printed `bedrock=wet` on every one of its 27 `METERS` lines while its `JUDGE SEATS` lines read
 `bedrock=dry` from the second on: the key was refused on the first call, the router held the provider out for the
 rest of the run, and the line that is the only record of when a provider was available went on saying the money was
 there. A hold is the three things that keep a provider out of routing beyond its meter: a refused credential, a
 timed hold after a refusal and a refusal for payment.

 Fixtures are cat-themed invention; every meter is a stub and no HTTP is made.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { createProviderBudgets, } from '../dist/final/node/index.mjs';
import { consoleLinesDuring, } from './neuralwatt-ledger.test-fixture.ts';
import { TEST_MAX_HOLD_MS, } from './budget-hold.test-fixture.ts';

/**
 Abort signal every read here carries.
 */
const SIGNAL = new AbortController().signal;

/**
 Milliseconds a reading is trusted in these cases.
 */
const FRESH_MS = 1_000;

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
 Picks the `METERS` lines out of what was logged.

 @param lines - everything the case logged

 @returns The `METERS` lines in order

 @example
 ```ts
 const meters = metersLines({ lines, },);
 ```
 */
function metersLines({ lines, }: { readonly lines: readonly string[]; },): readonly string[] {
  return lines.filter(function isMeters(line,): boolean {
    return line.includes(' METERS ',);
  },);
}

/**
 Budgets over the wet meters, reading on a clock the case moves by hand.

 @param clock - the clock's reading, which the case advances

 @returns Budgets whose readings are trusted for {@link FRESH_MS} and whose rate-limit hold is as long

 @example
 ```ts
 const budgets = budgetsOnClock({ clock: { now: 10_000, }, },);
 ```
 */
function budgetsOnClock(
  { clock, }: { readonly clock: { now: number; }; },
): ReturnType<typeof createProviderBudgets> {
  return createProviderBudgets({
    maxHoldMs: TEST_MAX_HOLD_MS,
    ...WET_METERS,
    freshForMs: FRESH_MS,
    rateLimitBackoffMs: FRESH_MS,
    now: () => clock.now,
  },);
}

await describe({
  name: 'budget meters line for a held provider',
  // One case at a time: each captures the console, which is one stub for the whole process.
  concurrency: 1,
  children: [
    it({
      name: 'PRINTS A PROVIDER WHOSE CREDENTIAL WAS REFUSED AS DRY, though its meter reads wet, and leaves the others '
        + 'as their meters said; the state the refusal logic reads stays what the meter said',
      fn: async ctx => {
        /**
         The states the budgets report, read inside the capture.
         */
        const read: { states?: Awaited<ReturnType<ReturnType<typeof createProviderBudgets>['readStates']>>; } = {};
        const lines = await consoleLinesDuring({
          run: async function readOnce(): Promise<void> {
            const budgets = createProviderBudgets({
              maxHoldMs: TEST_MAX_HOLD_MS,
              ...WET_METERS,
              heldOut: function refused(provider,): boolean {
                return provider === 'openrouter';
              },
            },);
            read.states = await budgets.readStates({ signal: SIGNAL, },);
          },
          sinon: ctx.sinon,
        },);
        const [line,] = metersLines({ lines, },);
        expect(line,).toContain('openrouter=dry',);
        expect(line,).toContain('hyper=wet',);
        expect(line,).toContain('synthetic=wet',);
        expect(read.states?.openrouter,).toBe('wet',);
      },
    },),

    it({
      name: 'PRINTS A PROVIDER IN A TIMED HOLD AS DRY, until the hold lapses, then as its meter says',
      fn: async ctx => {
        /**
         Clock the budgets read.
         */
        const clock = { now: 10_000, };
        const lines = await consoleLinesDuring({
          run: async function holdThenLapse(): Promise<void> {
            const budgets = budgetsOnClock({ clock, },);
            // The refusal names a return ten freshness windows away, so the hold outlasts the meter reading.
            await budgets.markRefused({
              provider: 'hyper',
              signal: SIGNAL,
              statedWaitMs: 10 * FRESH_MS,
            },);
            clock.now += 2 * FRESH_MS;
            await budgets.read({ signal: SIGNAL, },);
            clock.now += 20 * FRESH_MS;
            await budgets.read({ signal: SIGNAL, },);
          },
          sinon: ctx.sinon,
        },);
        const meters = metersLines({ lines, },);
        expect(meters.length,).toBe(3,);
        // The reading that began the refusal was taken before the hold was placed.
        expect(meters[0],).toContain('hyper=wet',);
        expect(meters[1],).toContain('hyper=dry',);
        expect(meters[2],).toContain('hyper=wet',);
      },
    },),

    it({
      name: 'PRINTS A PROVIDER THAT REFUSED FOR PAYMENT AS DRY while its meter has not moved',
      fn: async ctx => {
        /**
         Clock the budgets read.
         */
        const clock = { now: 10_000, };
        const lines = await consoleLinesDuring({
          run: async function refuseForPayment(): Promise<void> {
            const budgets = budgetsOnClock({ clock, },);
            await budgets.markRefused({
              provider: 'openrouter',
              signal: SIGNAL,
              paymentRequired: true,
            },);
            clock.now += 2 * FRESH_MS;
            await budgets.read({ signal: SIGNAL, },);
          },
          sinon: ctx.sinon,
        },);
        const meters = metersLines({ lines, },);
        expect(meters.length,).toBe(2,);
        expect(meters[0],).toContain('openrouter=wet',);
        expect(meters[1],).toContain('openrouter=dry',);
      },
    },),
  ],
},);
