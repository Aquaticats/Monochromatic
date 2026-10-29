/**
 Tests what the budgets say of a provider whose return served nothing: its hold ended, no call it served came after the
 end, and it refused again (the owner's answer of 2026-10-29, "Remember futile returns", measured on XingZ6020, where
 Synthetic refused again each time it came back).

 The budgets are the real ones over stub meters on a clock the case moves, so no case waits. The mark is read through
 `futileReturns`, which the seat reader asks before it waits for a hold. It is kept apart from the cooldown give-up of
 `provider-budget-cooldown-give-up.unit.test.ts`: the give-up reads only a refusal cooldown and takes the provider out
 of routing, while the mark reads every kind of hold, leaves the provider routable and clears when a call is served.
 Both yield to the same two exceptions: the provider is the only writing provider not dry, or its return would leave no
 bench floor unmet where without it one is.

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
 How long after a hold ended the cases refuse again.
 */
const AFTER_MS = 12_000;

/**
 A wait a refusal can state for the provider's return: ten minutes, past the cooldown.
 */
const STATED_MS = 600_000;

/**
 Budgets over a Synthetic meter that cannot be read, on the given clock, with Hyper reading wet beside it or not.

 @param clock - clock the case moves

 @param hyperWet - whether Hyper's meter reads wet, which leaves a writing provider besides Synthetic

 @returns The budgets

 @example
 ```ts
 const budgets = budgetsOn({ clock: { now: 1_000, }, hyperWet: true, },);
 ```
 */
function budgetsOn(
  {
    clock,
    hyperWet,
  }: {
    readonly clock: { now: number; };
    readonly hyperWet: boolean;
  },
): ReturnType<typeof createProviderBudgets> {
  return createProviderBudgets({
    maxHoldMs: TEST_MAX_HOLD_MS,
    synthetic: { quotas: unreadableQuotas, },
    ...(hyperWet
      ? {
        hyper: {
          credits: function credits() {
            return Promise.resolve({ balance: 243, },);
          },
        },
      }
      : {}),
    now: function now(): number {
      return clock.now;
    },
  },);
}

/**
 Refuses Synthetic, lets its hold run out, and refuses it again with nothing served in between.

 @param budgets - budgets the refusals are marked on

 @param clock - clock the case moves

 @param holdMs - how long the first refusal holds Synthetic out, which the clock passes before the second refusal

 @param statedWaitMs - wait both refusals name for Synthetic's return, none when absent

 @example
 ```ts
 await refuseAgainAfterTheHold({ budgets, clock, holdMs: COOLDOWN_MS, },);
 ```
 */
async function refuseAgainAfterTheHold(
  {
    budgets,
    clock,
    holdMs,
    statedWaitMs,
  }: {
    readonly budgets: ReturnType<typeof createProviderBudgets>;
    readonly clock: { now: number; };
    readonly holdMs: number;
    readonly statedWaitMs?: number;
  },
): Promise<void> {
  /**
   What each refusal names for the return, when it names one.
   */
  const stated = (statedWaitMs === undefined) ? {} : { statedWaitMs, };
  await budgets.markRefused({
    provider: 'synthetic',
    signal: SIGNAL,
    ...stated,
  },);
  clock.now += holdMs + AFTER_MS;
  await budgets.markRefused({
    provider: 'synthetic',
    signal: SIGNAL,
    ...stated,
  },);
}

await describe({
  name: `${createProviderBudgets.name} futile returns`,
  // One case at a time: each stubs the console, which is one for the whole process.
  concurrency: 1,
  children: [
    it({
      name: 'READS NO FUTILE RETURN BEFORE ANY HOLD HAS ENDED: the first refusal of a provider is no return, and a '
        + 'second refusal that arrives inside the hold came from a call in flight when it was set',
      fn: async ctx => {
        const clock = { now: 1_000, };
        const budgets = budgetsOn({
          clock,
          hyperWet: true,
        },);
        await consoleLinesDuring({
          run: async function refuse(): Promise<void> {
            await budgets.markRefused({ provider: 'synthetic', signal: SIGNAL, },);
            expect(budgets.futileReturns().get('synthetic',),).toBeUndefined();

            clock.now += COOLDOWN_MS / 3;
            await budgets.markRefused({ provider: 'synthetic', signal: SIGNAL, },);
            expect(budgets.futileReturns().get('synthetic',),).toBeUndefined();
          },
          sinon: ctx.sinon,
        },);
      },
    },),

    it({
      name: 'MARKS A COOLDOWN RETURN THAT SERVED NOTHING: the hold ended, nothing was served after it, and Synthetic '
        + 'refused again 12 s later, so the mark names the 300 s hold and the 12 s',
      fn: async ctx => {
        const clock = { now: 1_000, };
        const budgets = budgetsOn({
          clock,
          hyperWet: true,
        },);
        await consoleLinesDuring({
          run: async function refuse(): Promise<void> {
            await refuseAgainAfterTheHold({
              budgets,
              clock,
              holdMs: COOLDOWN_MS,
            },);
            expect(budgets.futileReturns().get('synthetic',),).toEqual({
              holdMs: COOLDOWN_MS,
              refusedAfterMs: AFTER_MS,
            },);
            expect(budgets.futileReturns().get('hyper',),).toBeUndefined();
          },
          sinon: ctx.sinon,
        },);
      },
    },),

    it({
      name: 'MARKS A STATED RETURN THAT SERVED NOTHING TOO, which the cooldown give-up never reads: a refusal that '
        + 'names its return in ten minutes, followed by the same refusal after the ten minutes, is a futile return',
      fn: async ctx => {
        const clock = { now: 1_000, };
        const budgets = budgetsOn({
          clock,
          hyperWet: true,
        },);
        await consoleLinesDuring({
          run: async function refuse(): Promise<void> {
            await refuseAgainAfterTheHold({
              budgets,
              clock,
              holdMs: STATED_MS,
              statedWaitMs: STATED_MS,
            },);
            expect(budgets.futileReturns().get('synthetic',),).toEqual({
              holdMs: STATED_MS,
              refusedAfterMs: AFTER_MS,
            },);
            expect(budgets.holds().synthetic,).toBe(STATED_MS,);
          },
          sinon: ctx.sinon,
        },);
      },
    },),

    it({
      name: 'MARKS NOTHING FOR A PROVIDER THAT SERVED AFTER ITS HOLD ENDED, however soon it refuses afterwards: a call '
        + 'served once the hold ended is the provider coming back',
      fn: async ctx => {
        const clock = { now: 1_000, };
        const budgets = budgetsOn({
          clock,
          hyperWet: true,
        },);
        await consoleLinesDuring({
          run: async function refuse(): Promise<void> {
            await budgets.markRefused({ provider: 'synthetic', signal: SIGNAL, },);
            clock.now += COOLDOWN_MS + AFTER_MS;
            budgets.markServed({ provider: 'synthetic', },);
            clock.now += AFTER_MS;
            await budgets.markRefused({ provider: 'synthetic', signal: SIGNAL, },);
            expect(budgets.futileReturns().get('synthetic',),).toBeUndefined();
          },
          sinon: ctx.sinon,
        },);
      },
    },),

    it({
      name: 'CLEARS THE MARK WHEN THE PROVIDER LATER SERVES A CALL, and marks it again when its next return serves '
        + 'nothing',
      fn: async ctx => {
        const clock = { now: 1_000, };
        const budgets = budgetsOn({
          clock,
          hyperWet: true,
        },);
        await consoleLinesDuring({
          run: async function refuse(): Promise<void> {
            await refuseAgainAfterTheHold({
              budgets,
              clock,
              holdMs: COOLDOWN_MS,
            },);
            expect(budgets.futileReturns().get('synthetic',),).toBeDefined();

            budgets.markServed({ provider: 'synthetic', },);
            expect(budgets.futileReturns().get('synthetic',),).toBeUndefined();

            // Its hold runs out again, and the next refusal comes with nothing served after the end.
            clock.now += COOLDOWN_MS + AFTER_MS;
            await budgets.markRefused({ provider: 'synthetic', signal: SIGNAL, },);
            expect(budgets.futileReturns().get('synthetic',),).toEqual({
              holdMs: COOLDOWN_MS,
              refusedAfterMs: AFTER_MS,
            },);
          },
          sinon: ctx.sinon,
        },);
      },
    },),

    it({
      name: 'FORGETS THE MARK WHEN THE NEXT ENTRY BEGINS, and counts only a return that ended after the entry began: '
        + 'the new entry meets the provider fresh, as it does for the give-up',
      fn: async ctx => {
        const clock = { now: 1_000, };
        const budgets = budgetsOn({
          clock,
          hyperWet: true,
        },);
        await consoleLinesDuring({
          run: async function refuse(): Promise<void> {
            budgets.beginEntry({ entryId: 'CatEntry1', },);
            await refuseAgainAfterTheHold({
              budgets,
              clock,
              holdMs: COOLDOWN_MS,
            },);
            expect(budgets.futileReturns().get('synthetic',),).toBeDefined();

            budgets.beginEntry({ entryId: 'CatEntry2', },);
            expect(budgets.futileReturns().get('synthetic',),).toBeUndefined();

            // The hold that was running when the second entry began ends inside it, and the provider refuses again.
            clock.now += COOLDOWN_MS + AFTER_MS;
            await budgets.markRefused({ provider: 'synthetic', signal: SIGNAL, },);
            expect(budgets.futileReturns().get('synthetic',),).toEqual({
              holdMs: COOLDOWN_MS,
              refusedAfterMs: AFTER_MS,
            },);
          },
          sinon: ctx.sinon,
        },);
      },
    },),

    it({
      name: 'COUNTS NO RETURN THAT ENDED BEFORE THE ENTRY BEGAN: the entry judges the provider on its own wait, so a '
        + 'refusal in a new entry after a hold that ran out before it marks nothing',
      fn: async ctx => {
        const clock = { now: 1_000, };
        const budgets = budgetsOn({
          clock,
          hyperWet: true,
        },);
        await consoleLinesDuring({
          run: async function refuse(): Promise<void> {
            budgets.beginEntry({ entryId: 'CatEntry1', },);
            await budgets.markRefused({ provider: 'synthetic', signal: SIGNAL, },);

            clock.now += COOLDOWN_MS + AFTER_MS;
            budgets.beginEntry({ entryId: 'CatEntry2', },);
            await budgets.markRefused({ provider: 'synthetic', signal: SIGNAL, },);
            expect(budgets.futileReturns().get('synthetic',),).toBeUndefined();
          },
          sinon: ctx.sinon,
        },);
      },
    },),

    it({
      name: 'LEAVES THE GIVE-UP AND THE MARK EACH THEIR OWN WORK ON ONE PROVIDER: after a cooldown refused at once the '
        + 'entry has given Synthetic up (its cooldown counts for no wait) and the return is marked, and a stated '
        + 'return that follows still counts as a hold, as the give-up keeps it, while the mark stays for the reader',
      fn: async ctx => {
        const clock = { now: 1_000, };
        const budgets = budgetsOn({
          clock,
          hyperWet: true,
        },);
        await consoleLinesDuring({
          run: async function refuse(): Promise<void> {
            await refuseAgainAfterTheHold({
              budgets,
              clock,
              holdMs: COOLDOWN_MS,
            },);
            expect(budgets.holds().synthetic,).toBe(0,);
            expect((await budgets.read({ signal: SIGNAL, },)).synthetic,).toBe(true,);
            expect(budgets.futileReturns().get('synthetic',),).toBeDefined();

            await budgets.markRefused({ provider: 'synthetic', signal: SIGNAL, statedWaitMs: STATED_MS, },);
            expect(budgets.holds().synthetic,).toBe(STATED_MS,);
            expect(budgets.futileReturns().get('synthetic',),).toBeDefined();
          },
          sinon: ctx.sinon,
        },);
      },
    },),

    it({
      name: 'YIELDS TO THE ONE EXCEPTION THE GIVE-UP YIELDS TO: when Synthetic is the only writing provider not dry, '
        + 'the give-up is set aside, its cooldown is waited for, and the return is not marked as one to skip',
      fn: async ctx => {
        const clock = { now: 1_000, };
        const budgets = budgetsOn({
          clock,
          hyperWet: false,
        },);
        await consoleLinesDuring({
          run: async function refuse(): Promise<void> {
            await refuseAgainAfterTheHold({
              budgets,
              clock,
              holdMs: COOLDOWN_MS,
            },);
            expect(budgets.holds().synthetic,).toBe(COOLDOWN_MS,);
            expect(budgets.futileReturns().get('synthetic',),).toBeUndefined();
          },
          sinon: ctx.sinon,
        },);
      },
    },),

    it({
      name: 'YIELDS TO THE OTHER EXCEPTION TOO: when with Synthetic back no bench floor the entry needs would be left '
        + 'unmet while without it one is, the return is not marked as one to skip',
      fn: async ctx => {
        const clock = { now: 1_000, };
        /**
         The floor stands unmet while Synthetic is dry and met once it is wet.
         */
        const budgets = createProviderBudgets({
          maxHoldMs: TEST_MAX_HOLD_MS,
          synthetic: { quotas: unreadableQuotas, },
          hyper: {
            credits: function credits() {
              return Promise.resolve({ balance: 243, },);
            },
          },
          unmetFloors: function translatorsNeedSynthetic({ dry, },): readonly string[] {
            return dry.synthetic ? ['translators 1 of 8 reachable, floor 2',] : [];
          },
          now: function now(): number {
            return clock.now;
          },
        },);
        await consoleLinesDuring({
          run: async function refuse(): Promise<void> {
            await refuseAgainAfterTheHold({
              budgets,
              clock,
              holdMs: COOLDOWN_MS,
            },);
            expect(budgets.holds().synthetic,).toBe(COOLDOWN_MS,);
            expect(budgets.futileReturns().get('synthetic',),).toBeUndefined();
          },
          sinon: ctx.sinon,
        },);
      },
    },),

    it({
      name: 'LOGS THE PROVIDER, THE HOLD AND HOW SOON IT REFUSED AGAIN when a return is marked, once for each return',
      fn: async ctx => {
        const clock = { now: 1_000, };
        const budgets = budgetsOn({
          clock,
          hyperWet: true,
        },);
        const lines = await consoleLinesDuring({
          run: async function refuse(): Promise<void> {
            budgets.beginEntry({ entryId: 'XingZ6020', },);
            await refuseAgainAfterTheHold({
              budgets,
              clock,
              holdMs: STATED_MS,
              statedWaitMs: STATED_MS,
            },);
          },
          sinon: ctx.sinon,
        },);

        /**
         The lines that mark the return.
         */
        const marks = lines.filter(function marksReturn(line,): boolean {
          return line.includes('its return served nothing',);
        },);
        expect(marks.length,).toBe(1,);
        expect(marks[0],).toContain('synthetic: refused again 12000ms after its 600000ms hold ended',);
        expect(marks[0],).toContain('entry XingZ6020',);
      },
    },),
  ],
},);
