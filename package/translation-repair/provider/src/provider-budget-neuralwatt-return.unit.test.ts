/**
 Tests that a NeuralWatt reading that is dry because the day's energy is spent names its cause and the time of day its
 day starts again, and that the budget layer keeps that reading true at the day start, while nothing waits for the
 return (the owner's answer of 2026-10-29, "Never wait, relaunch"; `neuralwatt-never-waits.unit.test.ts` holds the
 cases that show the router and the seat reader refuse at once).

 THE CASE IS XINGZ6019 (2026-10-22): NeuralWatt dry at 19:14:20 UTC because the day's 550 Wh was spent, its day
 restarting at 19:15:08 UTC, 48 s later. Every meter and clock here is a stub and no HTTP is made.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  createProviderBudgets,
  EveryProviderDryError,
  NOBODY_REFUSED,
  readBudgetsPastHolds,
} from '../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { TEST_MAX_HOLD_MS, } from './budget-hold.test-fixture.ts';
import { fakeClockStarted, } from './fake-clock-hold-wait.test-fixture.ts';

/**
 Abort signal every read here carries.
 */
const SIGNAL = new AbortController().signal;

/**
 Milliseconds after 00:00 UTC the account's day starts at: 19:15:08 UTC.
 */
const DAY_START_MS_OF_DAY = 69_308_000;

/**
 Milliseconds a reading is trusted, longer than any wait here, so only the return time can make a reading stale.
 */
const FRESH_MS = 600_000;

/**
 What a NeuralWatt reading carries that these cases move.
 */
type Reading = {
  readonly dayStart: string;
  readonly dailyWh: number;
  readonly spentWh: number;
  readonly reckonedWh: number;
  readonly reservedWh: number;
  readonly remainingWh: number;
  readonly calls: number;
  readonly keyBlockedUntil?: string;
};

/**
 A reading of a day whose energy is spent.
 */
const SPENT_DAY: Reading = {
  dayStart: '2026-10-22T19:15:08.000Z',
  dailyWh: 550,
  spentWh: 550,
  reckonedWh: 0,
  reservedWh: 0,
  remainingWh: 0,
  calls: 40,
};

/**
 A reading of the day that began at the return.
 */
const FRESH_DAY: Reading = {
  dayStart: '2026-10-23T19:15:08.000Z',
  dailyWh: 550,
  spentWh: 0,
  reckonedWh: 0,
  reservedWh: 0,
  remainingWh: 550,
  calls: 0,
};

/**
 Budgets over a NeuralWatt meter that reads the spent day first and the fresh one afterwards.

 @param msBeforeReturn - how long before its day starts again the wall clock reads

 @param clock - monotonic clock the case advances, which the budgets judge every hold and the cache window against; a
 case that waits out another hold hands the fake clock its wait sleeps on, so no real time passes

 @param reads - counts the energy reads

 @param spent - first reading, the spent day unless the case says otherwise

 @param staysSpent - whether every later reading still shows the spent day (default: the new day shows from the second)

 @param hyperCooldownMs - when given, the fleet carries a Hyper meter that reads dry and a
 refusal holds it out this long

 @returns Budgets that trust a reading for {@link FRESH_MS} and add no margin to a return

 @example
 ```ts
 const budgets = budgetsBefore({ msBeforeReturn: 48_000, clock: { now: 1_000, }, reads: { count: 0, }, },);
 ```
 */
function budgetsBefore(
  {
    msBeforeReturn,
    clock,
    reads,
    spent = SPENT_DAY,
    staysSpent = false,
    hyperCooldownMs,
  }: {
    readonly msBeforeReturn: number;
    readonly clock: { readonly now: number; };
    readonly reads: { count: number; };
    readonly spent?: Reading;
    readonly staysSpent?: boolean;
    readonly hyperCooldownMs?: number;
  },
): ReturnType<typeof createProviderBudgets> {
  return createProviderBudgets({
    maxHoldMs: TEST_MAX_HOLD_MS,
    // Conditional spread keeps the fleet without a Hyper meter where the case asks for none.
    ...((hyperCooldownMs === undefined)
      ? {}
      : { hyper: { credits: async () => ({ balance: 0, }), }, cooldownMs: hyperCooldownMs, }),
    neuralwatt: {
      energy: async () => {
        reads.count += 1;
        return ((reads.count === 1) || staysSpent) ? spent : FRESH_DAY;
      },
      dayAnchor: () => ({ known: true, msOfDay: DAY_START_MS_OF_DAY, }),
    },
    freshForMs: FRESH_MS,
    now: () => clock.now,
    wallNow: () => new Date(Date.UTC(
      2_026,
      9,
      23,
      0,
      0,
      0,
      DAY_START_MS_OF_DAY - msBeforeReturn,
    ),),
    returnMarginMs: 0,
  },);
}

/**
 The refusal the router throws once it has waited out the holds it can, read over the budgets of a case.

 @param budgets - budgets the case built, every provider dry

 @param pollMs - how often the wait reads the holds again

 @returns Refusal thrown

 @example
 ```ts
 const thrown = await refusalAfterHolds({ budgets, pollMs: 1, },);
 ```
 */
async function refusalAfterHolds(
  {
    budgets,
    pollMs,
  }: {
    readonly budgets: ReturnType<typeof createProviderBudgets>;
    readonly pollMs: number;
  },
): Promise<unknown> {
  return rejectionOf(() => readBudgetsPastHolds({
    budgets,
    modelId: 'glm-5.3',
    signal: SIGNAL,
    refused: NOBODY_REFUSED,
    pollMs,
  },),);
}

await describe({
  name: 'NeuralWatt dryness names its cause and its day start',
  concurrency: 1,
  children: [
    it({
      name: 'NAMES NO RETURN FOR ENERGY HELD BY CALLS IN FLIGHT: a day whose settled spend leaves more than the '
        + 'smallest call and whose reservations alone leave less releases them in seconds, not at the day start',
      fn: async () => {
        const budgets = budgetsBefore({
          msBeforeReturn: 48_000,
          clock: { now: 1_000, },
          reads: { count: 0, },
          spent: { ...SPENT_DAY, spentWh: 100, reservedWh: 449.9995, remainingWh: 0.0005, },
        },);
        expect((await budgets.read({ signal: SIGNAL, },)).neuralwatt,).toBe(true,);
        expect(budgets.holds().neuralwatt,).toBe(0,);
        expect(budgets.neuralwattDryness().kind,).toBe('reserved',);
      },
    },),

    it({
      name: 'READS THE METER AGAIN WHEN A TIMER FIRES A MILLISECOND EARLY: the return counts as passed two '
        + 'milliseconds before it, the slack Node\'s timers were measured to need',
      fn: async () => {
        const reads = { count: 0, };
        const clock = { now: 1_000, };
        const budgets = budgetsBefore({ msBeforeReturn: 48_000, clock, reads, },);
        await budgets.read({ signal: SIGNAL, },);
        clock.now += 47_998;
        expect((await budgets.read({ signal: SIGNAL, },)).neuralwatt,).toBe(false,);
        expect(reads.count,).toBe(2,);
      },
    },),

    it({
      name: 'STATES THE DAY START WITHOUT A DISTANCE when the account still reads the day spent after the return has '
        + 'passed: the account is lagging, so the cause names no return to relaunch for',
      fn: async () => {
        const reads = { count: 0, };
        const clock = { now: 1_000, };
        const budgets = budgetsBefore({ msBeforeReturn: 48_000, clock, reads, staysSpent: true, },);
        await budgets.read({ signal: SIGNAL, },);
        clock.now += 48_000;
        expect((await budgets.read({ signal: SIGNAL, },)).neuralwatt,).toBe(true,);
        expect(reads.count,).toBe(2,);
        expect(budgets.neuralwattDryness(),).toEqual({ kind: 'spent-day', dayStart: SPENT_DAY.dayStart, },);
        expect(budgets.holds().neuralwatt,).toBe(0,);
      },
    },),

    it({
      name: 'READS THE METER AGAIN AT THE RETURN, although the reading is still fresh, and finds the new day wet',
      fn: async () => {
        const reads = { count: 0, };
        const clock = { now: 1_000, };
        const budgets = budgetsBefore({ msBeforeReturn: 48_000, clock, reads, },);
        expect((await budgets.read({ signal: SIGNAL, },)).neuralwatt,).toBe(true,);
        clock.now += 47_997;
        expect((await budgets.read({ signal: SIGNAL, },)).neuralwatt,).toBe(true,);
        expect(reads.count,).toBe(1,);
        clock.now += 3;
        expect((await budgets.read({ signal: SIGNAL, },)).neuralwatt,).toBe(false,);
        expect(reads.count,).toBe(2,);
        expect(budgets.holds().neuralwatt,).toBe(0,);
      },
    },),

    it({
      name: 'NAMES NO RETURN FOR A BLOCKED KEY, whose allowance comes back at its own time and not at the day start',
      fn: async () => {
        const budgets = budgetsBefore({
          msBeforeReturn: 48_000,
          clock: { now: 1_000, },
          reads: { count: 0, },
          spent: { ...SPENT_DAY, remainingWh: 400, keyBlockedUntil: '2026-10-23T05:00:00.000Z', },
        },);
        expect((await budgets.read({ signal: SIGNAL, },)).neuralwatt,).toBe(true,);
        expect(budgets.holds().neuralwatt,).toBe(0,);
      },
    },),

    it({
      name: 'WAITS OUT ANOTHER PROVIDER\'S HOLD BUT NOT NEURALWATT\'S DAY: every provider dry, Hyper held 40 ms and '
        + 'NeuralWatt 600 s from its day start, so the call waits out Hyper and the refusal that follows states the '
        + 'time of day and the distance for the operator to relaunch by',
      fn: async ctx => {
        using fake = fakeClockStarted({ sinon: ctx.sinon, },);
        // THE FAKE CLOCK IS THE MONOTONIC CLOCK, since Hyper's 40 ms hold is judged against it and passes only as the case
        // ticks it.
        const budgets = budgetsBefore({
          msBeforeReturn: 600_000,
          clock: fake.clock,
          reads: { count: 0, },
          staysSpent: true,
          hyperCooldownMs: 40,
        },);
        await budgets.read({ signal: SIGNAL, },);
        await budgets.markRefused({ provider: 'hyper', signal: SIGNAL, },);
        /**
         The router's call, sleeping out Hyper's 40 ms hold in slices of a millisecond on the fake clock.
         */
        const refusal = refusalAfterHolds({ budgets, pollMs: 1, },);
        await fake.clock.tickAsync(40 + 1,);
        const thrown = await refusal;
        expect(thrown,).toBeInstanceOf(EveryProviderDryError,);
        expect((thrown as Error).message,).toContain('after waiting',);
        expect((thrown as Error).message,).toContain('at 19:15:08 UTC (in 10 min)',);
      },
    },),
  ],
},);
