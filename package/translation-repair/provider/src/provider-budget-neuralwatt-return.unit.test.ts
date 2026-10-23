/**
 Tests that a NeuralWatt reading that is dry because the day's energy is spent names its return as a hold, so the
 waits that already wait out holds (the router's and the seat reader's) wait for it when it is near.

 THE CASE IS XINGZ6019 (2026-10-22): NeuralWatt dry at 19:14:20 UTC because the day's 550 Wh was spent, its day
 restarting at 19:15:08 UTC, 48 s later. The bench stood short of its floor and the entry stopped 48 s before the
 energy came back. Every meter and clock here is a stub and no HTTP is made.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import {
  createProviderBudgets,
  EveryProviderDryError,
  NOBODY_REFUSED,
  readBudgetsPastHolds,
} from '../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';

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

 @param clock - monotonic clock the case advances, absent for the real one

 @param reads - counts the energy reads

 @param spent - first reading, the spent day unless the case says otherwise

 @param withXiaomi - whether the fleet carries a Xiaomi client, whose meter reads wet always, as the live roster's does

 @param marginMs - milliseconds the budgets add to a return (none unless the case moves it)

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
    withXiaomi = false,
    marginMs = 0,
    staysSpent = false,
    hyperCooldownMs,
  }: {
    readonly msBeforeReturn: number;
    readonly clock?: { now: number; };
    readonly reads: { count: number; };
    readonly spent?: Reading;
    readonly withXiaomi?: boolean;
    readonly marginMs?: number;
    readonly staysSpent?: boolean;
    readonly hyperCooldownMs?: number;
  },
): ReturnType<typeof createProviderBudgets> {
  return createProviderBudgets({
    // Conditional spread keeps the fleet without a Xiaomi client where the case asks for none.
    ...(withXiaomi ? { xiaomi: { spend: async () => ({ spentUsd: 0.0042, }), }, } : {}),
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
    // Conditional spread keeps the real clock where the case advances none.
    ...((clock === undefined) ? {} : { now: () => clock.now, }),
    wallNow: () => new Date(Date.UTC(
      2_026,
      9,
      23,
      0,
      0,
      0,
      DAY_START_MS_OF_DAY - msBeforeReturn,
    ),),
    returnMarginMs: marginMs,
  },);
}

/**
 The hold NeuralWatt carries after one reading of its spent day, for a caller that waits at most a minute.

 @param msBeforeReturn - how long before its day starts again the wall clock reads

 @returns Milliseconds NeuralWatt is held out for

 @example
 ```ts
 const held = await heldFor({ msBeforeReturn: 60_000, },);
 ```
 */
async function heldFor({ msBeforeReturn, }: { readonly msBeforeReturn: number; },): Promise<number> {
  const budgets = budgetsBefore({
    msBeforeReturn,
    clock: { now: 1_000, },
    reads: { count: 0, },
  },);
  await budgets.read({ signal: SIGNAL, },);
  return budgets.holds({ returnBoundMs: 60_000, },).neuralwatt;
}

/**
 The hold a router call would wait out at a distance, with the production margin of a second.

 @param msBeforeReturn - how long before its day starts again the wall clock reads

 @returns Milliseconds NeuralWatt is held out for under the router's bound

 @example
 ```ts
 const held = await heldWithin({ msBeforeReturn: 300_000, },);
 ```
 */
async function heldWithin({ msBeforeReturn, }: { readonly msBeforeReturn: number; },): Promise<number> {
  const budgets = budgetsBefore({
    msBeforeReturn,
    clock: { now: 1_000, },
    reads: { count: 0, },
    marginMs: 1_000,
  },);
  await budgets.read({ signal: SIGNAL, },);
  return budgets.holds({ returnBoundMs: 300_000, },).neuralwatt;
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
  name: 'NeuralWatt dryness names its return',
  concurrency: 1,
  children: [
    it({
      name: 'HOLDS NEURALWATT OUT FOR THE 48 SECONDS UNTIL ITS DAY STARTS AGAIN when the day\'s energy is spent, so '
        + 'the holds every wait reads carry the return, and the reading still says dry',
      fn: async () => {
        const reads = { count: 0, };
        const budgets = budgetsBefore({ msBeforeReturn: 48_000, clock: { now: 1_000, }, reads, },);
        expect((await budgets.read({ signal: SIGNAL, },)).neuralwatt,).toBe(true,);
        expect(budgets.holds().neuralwatt,).toBe(48_000,);
        expect(budgets.holds().synthetic,).toBe(0,);
      },
    },),

    it({
      name: 'NAMES NO HOLD FOR A RETURN PAST THE BOUND, so the waits do not hold an entry for hours: the refusal states '
        + 'the day start instead',
      fn: async () => {
        const reads = { count: 0, };
        const budgets = budgetsBefore({
          msBeforeReturn: 82_800_000,
          clock: { now: 1_000, },
          reads,
        },);
        expect((await budgets.read({ signal: SIGNAL, },)).neuralwatt,).toBe(true,);
        expect(budgets.holds({ returnBoundMs: 300_000, },).neuralwatt,).toBe(0,);
      },
    },),

    it({
      name: 'NAMES A HOLD AT THE BOUND AND NONE ONE MILLISECOND PAST IT',
      fn: async () => {
        expect([
          await heldFor({ msBeforeReturn: 60_000, },),
          await heldFor({ msBeforeReturn: 60_001, },),
        ],).toEqual([60_000, 0,],);
      },
    },),

    it({
      name: 'HOLDS THE RETURN AT ANY DISTANCE for a caller that names no bound, and none past the bound a caller '
        + 'names: the router names 300 s, the seat reader names none, and the production margin counts inside the '
        + 'bound, the way the refusal\'s clause and the pass\'s park count it',
      fn: async () => {
        const far = budgetsBefore({ msBeforeReturn: 82_800_000, clock: { now: 1_000, }, reads: { count: 0, }, },);
        await far.read({ signal: SIGNAL, },);
        expect(far.holds().neuralwatt,).toBe(82_800_000,);
        expect(far.holds({ returnBoundMs: 300_000, },).neuralwatt,).toBe(0,);
        expect([
          await heldWithin({ msBeforeReturn: 299_000, }),
          await heldWithin({ msBeforeReturn: 299_001, }),
        ],).toEqual([300_000, 0,],);
      },
    },),

    it({
      name: 'ENDS THE ROUTER\'S CALL AT THE BOUND AND STATES THE DISTANCE: every provider dry and NeuralWatt 301 s '
        + 'from its day start throws the refusal with the milliseconds to the return on it, which the pass reads to '
        + 'park instead of stopping, and the message states the time of day',
      fn: async () => {
        const budgets = budgetsBefore({ msBeforeReturn: 301_000, clock: { now: 1_000, }, reads: { count: 0, }, },);
        const thrown = await refusalAfterHolds({ budgets, pollMs: 1, },);
        expect(thrown,).toBeInstanceOf(EveryProviderDryError,);
        expect((thrown as EveryProviderDryError).neuralwattReturnsInMs,).toBe(301_000,);
        expect((thrown as Error).message,).toContain('at 19:15:08 UTC',);
      },
    },),

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
      name: 'TELLS A REFUSAL HOLD FROM THE DAY START: a refusal that holds NeuralWatt longer than its return is the '
        + 'hold that won, and the day start is not named for it',
      fn: async () => {
        const reads = { count: 0, };
        const clock = { now: 1_000, };
        const budgets = budgetsBefore({ msBeforeReturn: 48_000, clock, reads, },);
        await budgets.read({ signal: SIGNAL, },);
        expect(budgets.neuralwattHeldByReturn(),).toBe(true,);
        await budgets.markRefused({ provider: 'neuralwatt', signal: SIGNAL, statedWaitMs: 120_000, },);
        expect(budgets.holds().neuralwatt,).toBeGreaterThan(48_000,);
        expect(budgets.neuralwattHeldByReturn(),).toBe(false,);
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
      name: 'ENDS THE RUN WITH THE DAY START STATED when the account still reads the day spent after the return was '
        + 'waited for, and offers no return to park for: the router waited for it already',
      fn: async () => {
        /**
         The monotonic instant the case began at, from which the wall clock stub moves with real time.
         */
        const began = performance.now();
        const budgets = createProviderBudgets({
          neuralwatt: {
            energy: async () => SPENT_DAY,
            dayAnchor: () => ({ known: true, msOfDay: DAY_START_MS_OF_DAY, }),
          },
          freshForMs: FRESH_MS,
          // The wall clock reads 50 ms before the day start when the case begins and moves with real time, so the day
          // start passes during the wait and the next reading names the day after.
          wallNow: () => new Date(Date.UTC(
            2_026,
            9,
            23,
            0,
            0,
            0,
            (DAY_START_MS_OF_DAY - 50) + Math.round(performance.now() - began,),
          ),),
          returnMarginMs: 20,
        },);
        const thrown = await refusalAfterHolds({ budgets, pollMs: 10, },);
        expect(thrown,).toBeInstanceOf(EveryProviderDryError,);
        expect((thrown as Error).message,).toContain('after waiting',);
        expect((thrown as Error).message,).toContain('at 19:15:08 UTC',);
        expect((thrown as EveryProviderDryError).neuralwattReturnsInMs,).toBeUndefined();
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
      name: 'LETS THE ROUTER\'S WAIT SEE THE RETURN: every provider dry but NeuralWatt coming back in 50 ms waits it '
        + 'out and reads NeuralWatt wet, instead of ending the run',
      fn: async () => {
        const reads = { count: 0, };
        // THE REAL MONOTONIC CLOCK, since real time passes during the wait and the return is read against it.
        const budgets = budgetsBefore({ msBeforeReturn: 50, reads, withXiaomi: true, marginMs: 20, },);
        /**
         What the router's read comes to once the wait has ended, over the live roster's shape: a funded Xiaomi that
         serves a checker seat and nothing that writes, and every other provider unconfigured and so dry.
         */
        const view = await readBudgetsPastHolds({
          budgets,
          modelId: 'glm-5.3',
          signal: SIGNAL,
          refused: NOBODY_REFUSED,
          pollMs: 10,
          writing: {
            neuralwatt: true,
            synthetic: true,
            hyper: true,
            bedrock: true,
            deepseek: true,
            xiaomi: false,
            openrouter: true,
          },
        },);
        expect(view.neuralwatt,).toBe(false,);
        expect(view.xiaomi,).toBe(false,);
        expect(reads.count,).toBe(2,);
      },
    },),

    it({
      name: 'KEEPS THE DISTANCE ON THE REFUSAL AFTER WAITING OUT ANOTHER HOLD: every provider dry, Hyper held 40 ms '
        + 'and NeuralWatt 600 s from its day start (past the router\'s 300 s), so the call waits out Hyper and the '
        + 'refusal still carries the distance and the time of day, which the pass parks on',
      fn: async () => {
        const budgets = budgetsBefore({
          msBeforeReturn: 600_000,
          reads: { count: 0, },
          staysSpent: true,
          hyperCooldownMs: 40,
        },);
        await budgets.read({ signal: SIGNAL, },);
        await budgets.markRefused({ provider: 'hyper', signal: SIGNAL, },);
        const thrown = await refusalAfterHolds({ budgets, pollMs: 1, },);
        expect(thrown,).toBeInstanceOf(EveryProviderDryError,);
        expect((thrown as Error).message,).toContain('after waiting',);
        expect((thrown as EveryProviderDryError).neuralwattReturnsAt,).toBe('19:15:08 UTC',);
        expect((thrown as EveryProviderDryError).neuralwattReturnsInMs,).toBeGreaterThan(590_000,);
        expect((thrown as EveryProviderDryError).neuralwattReturnsInMs,).toBeLessThanOrEqual(600_000,);
      },
    },),

    it({
      name: 'CARRIES THE INSTANT OF THE READ THAT NAMED THE RETURN when the caller gives it, and none when it does '
        + 'not, so a park counts the wait from the read and the router\'s own refusal counts it from its arrival',
      fn: async () => {
        /**
         The refusal a seat reader makes, stamped.
         */
        const stamped = new EveryProviderDryError({ neuralwattReturnsInMs: 301_000, neuralwattReadAtMs: 1_000, },);
        expect(stamped.neuralwattReadAtMs,).toBe(1_000,);
        expect(new EveryProviderDryError({ neuralwattReturnsInMs: 301_000, },).neuralwattReadAtMs,).toBeUndefined();
      },
    },),
  ],
},);
