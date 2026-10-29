/**
 Tests what the budgets do once an entry has given a provider up for refusing right after its refusal cooldown ran out
 (XingZ6020, 2026-10-23: Synthetic read "unreadable, treating as spendable" and refused every call, and each refusal set
 a fresh 300 s cooldown that the seat reader waited out).

 A given-up provider reads dry and its cooldown counts for no wait; a hold that names its own return still counts; the
 provider comes back when the next entry begins; and a provider that served after its cooldown is not given up. The
 budgets are the real ones over stub meters on a clock the case moves, so no case waits.

 Fixtures are cat-themed invention; every meter is a stub and no HTTP is made.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { consoleLinesDuring, } from './neuralwatt-ledger.test-fixture.ts';
import { createProviderBudgets, } from '../dist/final/node/index.mjs';
import { TEST_MAX_HOLD_MS, } from './budget-hold.test-fixture.ts';
import { unreadableQuotas, } from './unreadable-quotas.test-fixture.ts';

/**
 Abort signal every read here carries.
 */
const SIGNAL = new AbortController().signal;

/**
 The refusal cooldown the budgets set by default, in milliseconds.
 */
const COOLDOWN_MS = 300_000;

/**
 Budgets over a Synthetic meter that cannot be read and a Hyper meter that reads wet, on the given clock.

 @param clock - clock the case moves

 @returns Budgets

 @example
 ```ts
 const budgets = budgetsOn({ clock: { now: 1_000, }, },);
 ```
 */
function budgetsOn(
  { clock, }: { readonly clock: { now: number; }; },
): ReturnType<typeof createProviderBudgets> {
  return createProviderBudgets({
    maxHoldMs: TEST_MAX_HOLD_MS,
    synthetic: { quotas: unreadableQuotas, },
    hyper: {
      credits: function credits() {
        return Promise.resolve({ balance: 243, },);
      },
    },
    now: function now(): number {
      return clock.now;
    },
  },);
}

/**
 Refuses Synthetic, lets its cooldown run out, and refuses it again at once.

 @param budgets - budgets the refusals are marked on

 @param clock - clock the case moves

 @example
 ```ts
 await refuseAtOnceAfterTheCooldown({ budgets, clock, },);
 ```
 */
async function refuseAtOnceAfterTheCooldown(
  {
    budgets,
    clock,
  }: {
    readonly budgets: ReturnType<typeof createProviderBudgets>;
    readonly clock: { now: number; };
  },
): Promise<void> {
  await budgets.markRefused({ provider: 'synthetic', signal: SIGNAL, },);
  clock.now += COOLDOWN_MS + 12_000;
  await budgets.markRefused({ provider: 'synthetic', signal: SIGNAL, },);
}

await describe({
  name: `${createProviderBudgets.name} cooldown give-up`,
  // One case at a time: each stubs the console, which is one for the whole process.
  concurrency: 1,
  children: [
    it({
      name: 'WAITS FOR A COOLDOWN BEFORE THE PROVIDER REFUSES AGAIN, and not for the one that follows its repeat: the '
        + 'hold counts at first and the provider reads dry, then the hold counts for nothing and the provider stays dry',
      fn: async ctx => {
        const clock = { now: 1_000, };
        const budgets = budgetsOn({ clock, },);
        await consoleLinesDuring({
          run: async function refuse(): Promise<void> {
            await budgets.markRefused({ provider: 'synthetic', signal: SIGNAL, },);
            expect(budgets.holds().synthetic,).toBe(COOLDOWN_MS,);
            expect((await budgets.read({ signal: SIGNAL, },)).synthetic,).toBe(true,);

            clock.now += COOLDOWN_MS + 12_000;
            expect(budgets.holds().synthetic,).toBe(0,);
            expect((await budgets.read({ signal: SIGNAL, },)).synthetic,).toBe(false,);

            await budgets.markRefused({ provider: 'synthetic', signal: SIGNAL, },);
            expect(budgets.holds().synthetic,).toBe(0,);
            expect((await budgets.read({ signal: SIGNAL, },)).synthetic,).toBe(true,);
          },
          sinon: ctx.sinon,
        },);
      },
    },),

    it({
      name: 'KEEPS WHAT THE METER SAID: a provider given up reads dry, and its state is still the unreadable one the '
        + 'meter gave, so the seat reader still words it as a meter that said nothing',
      fn: async ctx => {
        const clock = { now: 1_000, };
        const budgets = budgetsOn({ clock, },);
        await consoleLinesDuring({
          run: async function refuse(): Promise<void> {
            await refuseAtOnceAfterTheCooldown({ budgets, clock, },);
            expect((await budgets.readStates({ signal: SIGNAL, },)).synthetic,).toBe('unreadable',);
          },
          sinon: ctx.sinon,
        },);
      },
    },),

    it({
      name: 'STILL COUNTS A HOLD THAT NAMES ITS OWN RETURN on a provider given up: Synthetic stating 600 s after the '
        + 'give-up is waited for those 600 s',
      fn: async ctx => {
        const clock = { now: 1_000, };
        const budgets = budgetsOn({ clock, },);
        await consoleLinesDuring({
          run: async function refuse(): Promise<void> {
            await refuseAtOnceAfterTheCooldown({ budgets, clock, },);
            expect(budgets.holds().synthetic,).toBe(0,);
            await budgets.markRefused({ provider: 'synthetic', signal: SIGNAL, statedWaitMs: 600_000, },);
            expect(budgets.holds().synthetic,).toBe(600_000,);
          },
          sinon: ctx.sinon,
        },);
      },
    },),

    it({
      name: 'ASKS THE PROVIDER AGAIN WHEN THE NEXT ENTRY BEGINS, and waits for a cooldown still running as for any other',
      fn: async ctx => {
        const clock = { now: 1_000, };
        const budgets = budgetsOn({ clock, },);
        await consoleLinesDuring({
          run: async function refuse(): Promise<void> {
            budgets.beginEntry({ entryId: 'CatEntry1', },);
            await refuseAtOnceAfterTheCooldown({ budgets, clock, },);
            expect((await budgets.read({ signal: SIGNAL, },)).synthetic,).toBe(true,);

            budgets.beginEntry({ entryId: 'CatEntry2', },);
            expect(budgets.holds().synthetic,).toBe(COOLDOWN_MS,);

            clock.now += COOLDOWN_MS + 12_000;
            expect((await budgets.read({ signal: SIGNAL, },)).synthetic,).toBe(false,);
          },
          sinon: ctx.sinon,
        },);
      },
    },),

    it({
      name: 'DOES NOT GIVE UP A PROVIDER THAT SERVED AFTER ITS COOLDOWN, however often it refuses afterwards',
      fn: async ctx => {
        const clock = { now: 1_000, };
        const budgets = budgetsOn({ clock, },);
        await consoleLinesDuring({
          run: async function refuse(): Promise<void> {
            await budgets.markRefused({ provider: 'synthetic', signal: SIGNAL, },);
            clock.now += COOLDOWN_MS + 12_000;
            budgets.markServed({ provider: 'synthetic', },);
            clock.now += 30_000;
            await budgets.markRefused({ provider: 'synthetic', signal: SIGNAL, },);
            expect(budgets.holds().synthetic,).toBe(COOLDOWN_MS,);
          },
          sinon: ctx.sinon,
        },);
      },
    },),

    it({
      name: 'LOGS THE ENTRY, THE PROVIDER AND THE COOLDOWN WAITED OUT when a provider is given up, and the providers '
        + 'asked again when the next entry begins',
      fn: async ctx => {
        const clock = { now: 1_000, };
        const budgets = budgetsOn({ clock, },);
        const lines = await consoleLinesDuring({
          run: async function refuse(): Promise<void> {
            budgets.beginEntry({ entryId: 'XingZ6020', },);
            await refuseAtOnceAfterTheCooldown({ budgets, clock, },);
            budgets.beginEntry({ entryId: 'XingZ6021', },);
          },
          sinon: ctx.sinon,
        },);

        expect(lines.some(function givesUp(line,): boolean {
          return line.includes('synthetic: refused again right after its 300000ms refusal cooldown ran out',)
            && line.includes('entry XingZ6020 stops waiting for synthetic\'s cooldowns',);
        },),).toBe(true,);
        expect(lines.some(function asksAgain(line,): boolean {
          return line.includes('entry XingZ6021 begins: synthetic no longer given up on, so asked again',);
        },),).toBe(true,);
      },
    },),
  ],
},);
