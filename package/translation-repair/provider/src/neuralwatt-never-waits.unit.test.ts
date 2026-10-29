/**
 Tests that a call whose only way forward is NeuralWatt's day return refuses at once and never waits for it.

 THE OWNER'S ANSWER OF 2026-10-29 is "Never wait, relaunch": "Stop at once and relaunch later." A router call used to
 wait up to 300 s for the return and the seat reader used to wait for a return inside what the pass had left. Both now
 refuse at once, and the refusal still states the time of day the day starts again so the operator knows when to
 relaunch. Holds of other providers (a refusal cooldown, a stated Retry-After) are not part of this and are still
 waited for. Every meter and clock here is a stub, the fake clock stands under every sleep, and no HTTP is made.

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
import { waitedOutOnFakeClock, } from './fake-clock-hold-wait.test-fixture.ts';

/**
 Milliseconds after 00:00 UTC the account's day starts at: 19:15:08 UTC.
 */
const DAY_START_MS_OF_DAY = 69_308_000;

/**
 A reading of a day whose energy is spent.
 */
const SPENT_DAY = {
  dayStart: '2026-10-22T19:15:08.000Z',
  dailyWh: 550,
  spentWh: 550,
  reckonedWh: 0,
  reservedWh: 0,
  remainingWh: 0,
  calls: 40,
};

/**
 A reading of the day that begins at the return.
 */
const FRESH_DAY = {
  dayStart: '2026-10-23T19:15:08.000Z',
  dailyWh: 550,
  spentWh: 0,
  reckonedWh: 0,
  reservedWh: 0,
  remainingWh: 550,
  calls: 0,
};

/**
 Budgets whose NeuralWatt meter reads the spent day first and the new day afterwards, every other provider unconfigured
 and so dry, with the wall clock reading 48 s before the day start.

 @returns Budgets over that fleet, whose monotonic clock does not move

 @example
 ```ts
 const budgets = budgetsOverSpentNeuralWatt();
 ```
 */
function budgetsOverSpentNeuralWatt(): ReturnType<typeof createProviderBudgets> {
  /**
   Energy reads so far, which move the meter from the spent day to the new one.
   */
  const reads = { count: 0, };
  return createProviderBudgets({
    maxHoldMs: TEST_MAX_HOLD_MS,
    neuralwatt: {
      energy: async () => {
        reads.count += 1;
        return (reads.count === 1) ? SPENT_DAY : FRESH_DAY;
      },
      dayAnchor: () => ({ known: true, msOfDay: DAY_START_MS_OF_DAY, }),
    },
    freshForMs: 600_000,
    now: () => 1_000,
    wallNow: () => new Date(Date.UTC(
      2_026,
      9,
      23,
      0,
      0,
      0,
      DAY_START_MS_OF_DAY - 48_000,
    ),),
    returnMarginMs: 0,
  },);
}

await describe({
  name: 'NeuralWatt\'s day return is never waited for',
  concurrency: 1,
  children: [
    it({
      name: 'REFUSES AT ONCE WHEN NEURALWATT\'S DAY RETURN IS THE ONLY WAY FORWARD, with no sleep at all and the time of '
        + 'day the day starts again in the message: every other provider dry and NeuralWatt 48 s from its day start '
        + 'used to be waited out, and now ends the call on the first reading',
      fn: async ctx => {
        const budgets = budgetsOverSpentNeuralWatt();
        const { value: thrown, elapsedMs, } = await waitedOutOnFakeClock({
          sinon: ctx.sinon,
          holdMs: 120_000,
          pollMs: 1_000,
          reading: async ({ signal, }) => await rejectionOf(() =>
            readBudgetsPastHolds({
              budgets,
              modelId: 'glm-5.3',
              signal,
              refused: NOBODY_REFUSED,
              pollMs: 1_000,
            },)
          ),
        },);
        expect(thrown,).toBeInstanceOf(EveryProviderDryError,);
        expect(elapsedMs,).toBe(0,);
        expect((thrown as Error).message,).toContain('at 19:15:08 UTC',);
        expect((thrown as Error).message,).not.toContain('after waiting',);
      },
    },),

    it({
      name: 'HOLDS NO PROVIDER OUT FOR NEURALWATT\'S DAY START: the reading still says dry and still states the cause '
        + 'and the distance, but the holds every wait reads carry no return for it',
      fn: async () => {
        const budgets = budgetsOverSpentNeuralWatt();
        expect((await budgets.read({ signal: new AbortController().signal, },)).neuralwatt,).toBe(true,);
        expect(budgets.holds().neuralwatt,).toBe(0,);
        expect(budgets.neuralwattDryness(),).toEqual({
          kind: 'spent-day',
          dayStart: '2026-10-22T19:15:08.000Z',
          returnsInMs: 48_000,
        },);
      },
    },),
  ],
},);
