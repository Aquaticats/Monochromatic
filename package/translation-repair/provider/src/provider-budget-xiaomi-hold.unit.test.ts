/**
 Tests that a Xiaomi payment refusal holds the provider dry the way
 `doc/decision/translation-repair-direct-providers.md`
 says it does, in both directions.

 XIAOMI'S METER IS ITS RUN SPEND, so the budget layer clears a payment mark only when that spend moves. A refusal
 with no call settling afterwards therefore holds Xiaomi dry for the rest of the process, and that is intended: a 402
 means the account is out of money, and nothing in the process can know it was topped up. A call that settles after the
 refusal moves the spend and clears the mark early.

 Fixtures are cat-themed invention; every meter is a stub and no HTTP is made.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { createProviderBudgets, } from '../dist/final/node/index.mjs';
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
 Budgets whose Xiaomi spend is whatever the case's spend holder says now.

 @param clock - the clock's reading, which the case advances

 @param spend - the spend the Xiaomi meter reports, which the case moves

 @returns Budgets whose readings are trusted for {@link FRESH_MS}

 @example
 ```ts
 const budgets = xiaomiBudgets({ clock: { now: 10_000, }, spend: { usd: 0.0042, }, },);
 ```
 */
function xiaomiBudgets(
  {
    clock,
    spend,
  }: {
    readonly clock: { now: number; };
    readonly spend: { usd: number; };
  },
): ReturnType<typeof createProviderBudgets> {
  return createProviderBudgets({
    maxHoldMs: TEST_MAX_HOLD_MS,
    xiaomi: { spend: async () => ({ spentUsd: spend.usd, }), },
    freshForMs: FRESH_MS,
    now: () => clock.now,
  },);
}

await describe({
  name: 'Xiaomi payment hold',
  concurrency: 1,
  children: [
    it({
      name: 'HOLDS Xiaomi dry across every later read while no call settles after the refusal and its spend stays where '
        + 'it was, however long the process runs (the current behaviour; the owner has not ruled on recovery)',
      fn: async () => {
        /**
         Clock the budgets read.
         */
        const clock = { now: 10_000, };
        /**
         Spend the meter reports, never moved.
         */
        const spend = { usd: 0.0042, };
        const budgets = xiaomiBudgets({ clock, spend, },);
        expect((await budgets.read({ signal: SIGNAL, },)).xiaomi,).toBe(false,);
        await budgets.markRefused({ provider: 'xiaomi', signal: SIGNAL, paymentRequired: true, },);
        clock.now += 2 * FRESH_MS;
        expect((await budgets.read({ signal: SIGNAL, },)).xiaomi,).toBe(true,);
        clock.now += 1_000 * FRESH_MS;
        expect((await budgets.read({ signal: SIGNAL, },)).xiaomi,).toBe(true,);
        clock.now += 1_000_000 * FRESH_MS;
        expect((await budgets.read({ signal: SIGNAL, },)).xiaomi,).toBe(true,);
      },
    },),
    it({
      name: 'CLEARS the hold early when a call settles after the refusal and moves the spend, so one more call meets '
        + 'the refusal again and re-marks it',
      fn: async () => {
        /**
         Clock the budgets read.
         */
        const clock = { now: 10_000, };
        /**
         Spend the meter reports, moved by the case.
         */
        const spend = { usd: 0.0042, };
        const budgets = xiaomiBudgets({ clock, spend, },);
        await budgets.read({ signal: SIGNAL, },);
        await budgets.markRefused({ provider: 'xiaomi', signal: SIGNAL, paymentRequired: true, },);
        clock.now += 2 * FRESH_MS;
        expect((await budgets.read({ signal: SIGNAL, },)).xiaomi,).toBe(true,);
        spend.usd = 0.0051;
        clock.now += 2 * FRESH_MS;
        expect((await budgets.read({ signal: SIGNAL, },)).xiaomi,).toBe(false,);
        await budgets.markRefused({ provider: 'xiaomi', signal: SIGNAL, paymentRequired: true, },);
        clock.now += 2 * FRESH_MS;
        expect((await budgets.read({ signal: SIGNAL, },)).xiaomi,).toBe(true,);
      },
    },),
  ],
},);
