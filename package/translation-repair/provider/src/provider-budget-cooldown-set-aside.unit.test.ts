/**
 Tests when the budgets set an entry's give-up of a provider aside (XingZ6020, 2026-10-23, and the ruling that giving a
 provider's cooldowns up must never be the cause of a run ending).

 An entry gives a provider up after it refuses right after a cooldown the entry waited out. The give-up narrows where a
 call may go, so it holds only while another provider that keeps the run going reads wet. When the given-up provider is
 the only writing provider not dry, every other one being dry or held, the give-up is set aside: the provider's
 cooldown is waited out as before and the provider is asked again, because reading it dry would leave every provider
 dry and end the run for a provider that serves again after its cooldown. The give-up comes back in force when another
 writing provider reads wet again.

 The budgets are the real ones over stub meters on a clock the case moves, so no case waits. Fixtures are cat-themed
 invention; every meter is a stub and no HTTP is made.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type { ProviderRecord, } from '@monochromatic-dev/translation-repair-shared-contract';
import { SEAT_SYNTHETIC_TEXT_EVERYWHERE, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  createProviderBudgets,
  HOLD_POLL_MS,
  readBudgetsPastHolds,
} from '../dist/final/node/index.mjs';
import { TEST_MAX_HOLD_MS, } from './budget-hold.test-fixture.ts';
import { consoleLinesDuring, } from './neuralwatt-ledger.test-fixture.ts';
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
 A clock step that passes the cooldown and the freshness window of a meter reading.
 */
const PAST_COOLDOWN_MS = COOLDOWN_MS + 12_000;

/**
 A clock step that passes the freshness window of a meter reading and not the cooldown.
 */
const PAST_FRESHNESS_MS = 61_000;

/**
 The providers that keep a run going while they read wet, all but Xiaomi whose one seat is a checker.
 */
const WRITING_BUT_XIAOMI: ProviderRecord<boolean> = {
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
 What one case moves: the clock, and whether DeepSeek's balance is available.
 */
type World = {
  /**
   The clock the budgets count on.
   */
  now: number;

  /**
   Whether DeepSeek says its account can make calls.
   */
  deepseekAvailable: boolean;
};

/**
 Budgets over a Synthetic meter that cannot be read, and, where the case asks, a DeepSeek meter that follows the
 world and a Xiaomi meter that always reads wet.

 @param world - what the case moves

 @param deepseek - whether the budgets have a DeepSeek meter

 @param xiaomi - whether the budgets have a Xiaomi meter

 @param writing - the providers that keep the run going while they read wet; every provider where absent

 @returns The budgets

 @example
 ```ts
 const budgets = budgetsOn({ world, deepseek: true, xiaomi: false, },);
 ```
 */
function budgetsOn(
  {
    world,
    deepseek,
    xiaomi,
    writing,
  }: {
    readonly world: World;
    readonly deepseek: boolean;
    readonly xiaomi: boolean;
    readonly writing?: ProviderRecord<boolean>;
  },
): ReturnType<typeof createProviderBudgets> {
  return createProviderBudgets({
    maxHoldMs: TEST_MAX_HOLD_MS,
    synthetic: { quotas: unreadableQuotas, },
    ...(deepseek
      ? {
        deepseek: {
          credits: function credits() {
            return Promise.resolve({
              isAvailable: world.deepseekAvailable,
              totalUsd: 9.96,
            },);
          },
        },
      }
      : {}),
    ...(xiaomi
      ? {
        xiaomi: {
          spend: function spend() {
            return { spentUsd: 0, };
          },
        },
      }
      : {}),
    ...((writing === undefined) ? {} : { writing, }),
    now: function now(): number {
      return world.now;
    },
  },);
}

/**
 Refuses Synthetic, lets its cooldown run out, and refuses it again at once.

 @param budgets - budgets the refusals are marked on

 @param world - what the case moves

 @example
 ```ts
 await refuseAtOnceAfterTheCooldown({ budgets, world, },);
 ```
 */
async function refuseAtOnceAfterTheCooldown(
  {
    budgets,
    world,
  }: {
    readonly budgets: ReturnType<typeof createProviderBudgets>;
    readonly world: World;
  },
): Promise<void> {
  await budgets.markRefused({
    provider: 'synthetic',
    signal: SIGNAL,
  },);
  world.now += PAST_COOLDOWN_MS;
  await budgets.markRefused({
    provider: 'synthetic',
    signal: SIGNAL,
  },);
}

await describe({
  name: `${createProviderBudgets.name} give-up set aside`,
  // One case at a time: each stubs the console, which is one for the whole process.
  concurrency: 1,
  children: [
    it({
      name: 'SETS THE GIVE-UP ASIDE WHEN THE ALTERNATIVE GOES DRY: DeepSeek reads wet at the give-up and dry later, so '
        + 'the cooldown Synthetic is held for counts again and Synthetic is asked once it has ended',
      fn: async ctx => {
        const world: World = {
          now: 1_000,
          deepseekAvailable: true,
        };
        const budgets = budgetsOn({
          world,
          deepseek: true,
          xiaomi: false,
        },);
        await consoleLinesDuring({
          run: async function goDry(): Promise<void> {
            budgets.beginEntry({ entryId: 'CatEntry1', },);
            await refuseAtOnceAfterTheCooldown({
              budgets,
              world,
            },);
            expect(budgets.holds().synthetic,).toBe(0,);
            expect((await budgets.read({ signal: SIGNAL, },)).synthetic,).toBe(true,);

            world.deepseekAvailable = false;
            world.now += PAST_FRESHNESS_MS;
            expect((await budgets.read({ signal: SIGNAL, },)).synthetic,).toBe(true,);
            expect(budgets.holds().synthetic,).toBe(COOLDOWN_MS - PAST_FRESHNESS_MS,);

            world.now += COOLDOWN_MS;
            expect((await budgets.read({ signal: SIGNAL, },)).synthetic,).toBe(false,);
          },
          sinon: ctx.sinon,
        },);
      },
    },),

    it({
      name: 'WAITS FOR THE COOLDOWN OF A PROVIDER THAT IS THE ONLY ONE NOT DRY, however often it refuses right after '
        + 'one: nobody else is wet, so the give-up is set aside',
      fn: async ctx => {
        const world: World = {
          now: 1_000,
          deepseekAvailable: false,
        };
        const budgets = budgetsOn({
          world,
          deepseek: true,
          xiaomi: false,
        },);
        await consoleLinesDuring({
          run: async function refuseAlone(): Promise<void> {
            budgets.beginEntry({ entryId: 'CatEntry1', },);
            await refuseAtOnceAfterTheCooldown({
              budgets,
              world,
            },);
            expect(budgets.holds().synthetic,).toBe(COOLDOWN_MS,);
            expect((await budgets.read({ signal: SIGNAL, },)).synthetic,).toBe(true,);

            world.now += PAST_COOLDOWN_MS;
            expect(budgets.holds().synthetic,).toBe(0,);
            expect((await budgets.read({ signal: SIGNAL, },)).synthetic,).toBe(false,);

            await budgets.markRefused({
              provider: 'synthetic',
              signal: SIGNAL,
            },);
            expect(budgets.holds().synthetic,).toBe(COOLDOWN_MS,);
          },
          sinon: ctx.sinon,
        },);
      },
    },),

    it({
      name: 'DOES NOT COUNT A WET PROVIDER THAT KEEPS NO RUN GOING AS AN ALTERNATIVE: Xiaomi reads wet and writes '
        + 'nothing, so Synthetic is still the only writing provider not dry and its cooldown is waited for',
      fn: async ctx => {
        const world: World = {
          now: 1_000,
          deepseekAvailable: false,
        };
        const budgets = budgetsOn({
          world,
          deepseek: false,
          xiaomi: true,
          writing: WRITING_BUT_XIAOMI,
        },);
        await consoleLinesDuring({
          run: async function refuseBesideACheckerAlone(): Promise<void> {
            await refuseAtOnceAfterTheCooldown({
              budgets,
              world,
            },);
            expect(budgets.holds().synthetic,).toBe(COOLDOWN_MS,);
          },
          sinon: ctx.sinon,
        },);
      },
    },),

    it({
      name: 'LOGS THE SET-ASIDE ONCE with the provider, the entry and how the other writing providers read, and says '
        + 'nothing when another provider is wet',
      fn: async ctx => {
        const dry: World = {
          now: 1_000,
          deepseekAvailable: false,
        };
        const dryBudgets = budgetsOn({
          world: dry,
          deepseek: true,
          xiaomi: false,
        },);
        const lines = await consoleLinesDuring({
          run: async function refuseAlone(): Promise<void> {
            dryBudgets.beginEntry({ entryId: 'XingZ6020', },);
            await refuseAtOnceAfterTheCooldown({
              budgets: dryBudgets,
              world: dry,
            },);
            dry.now += PAST_COOLDOWN_MS;
            await dryBudgets.markRefused({
              provider: 'synthetic',
              signal: SIGNAL,
            },);
          },
          sinon: ctx.sinon,
        },);
        /**
         The lines that set the give-up aside.
         */
        const setAside = lines.filter(function isSetAside(line,): boolean {
          return line.includes('sets its give-up aside',);
        },);
        expect(setAside.length,).toBe(1,);
        expect(setAside[0],).toContain('synthetic',);
        expect(setAside[0],).toContain('entry XingZ6020',);
        expect(setAside[0],).toContain('no other writing provider reads wet (',);
        expect(setAside[0],).toContain('deepseek dry',);

        const wet: World = {
          now: 1_000,
          deepseekAvailable: true,
        };
        const wetBudgets = budgetsOn({
          world: wet,
          deepseek: true,
          xiaomi: false,
        },);
        const quiet = await consoleLinesDuring({
          run: async function refuseBesideALiveProvider(): Promise<void> {
            await refuseAtOnceAfterTheCooldown({
              budgets: wetBudgets,
              world: wet,
            },);
          },
          sinon: ctx.sinon,
        },);
        expect(quiet.some(function isSetAside(line,): boolean {
          return line.includes('sets its give-up aside',);
        },),).toBe(false,);
      },
    },),

    it({
      name: 'LETS THE ROUTER WAIT OUT THE COOLDOWN of a given-up provider that is the only one not dry, and ask it again, '
        + 'where the give-up alone raised EveryProviderDryError and ended the run',
      fn: async ctx => {
        const clock = ctx.sinon.useFakeTimers({
          toFake: [
            'setTimeout',
            'clearTimeout',
          ],
          shouldClearNativeTimers: true,
        },);
        using _installed = {
          [Symbol.dispose]: function removeClock(): void {
            clock.restore();
          },
        };
        const world: World = {
          get now(): number {
            return clock.now;
          },
          deepseekAvailable: false,
        };
        const budgets = budgetsOn({
          world,
          deepseek: true,
          xiaomi: false,
        },);
        await consoleLinesDuring({
          run: async function routeAlone(): Promise<void> {
            budgets.beginEntry({ entryId: 'CatEntry1', },);
            await budgets.markRefused({
              provider: 'synthetic',
              signal: SIGNAL,
            },);
            await clock.tickAsync(PAST_COOLDOWN_MS,);
            await budgets.markRefused({
              provider: 'synthetic',
              signal: SIGNAL,
            },);

            /**
             The call that routed here, refused by Synthetic a moment ago.
             */
            const reading = readBudgetsPastHolds({
              budgets,
              modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
              signal: SIGNAL,
              refused: 'synthetic',
              pollMs: HOLD_POLL_MS,
            },);
            await clock.tickAsync(PAST_COOLDOWN_MS,);
            expect((await reading).synthetic,).toBe(false,);
          },
          sinon: ctx.sinon,
        },);
      },
    },),
  ],
},);
