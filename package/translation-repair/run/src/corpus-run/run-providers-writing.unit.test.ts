/**
 Tests that the roster of writing providers and the bench floors `configureProviders` is handed reach the budgets it
 builds, where they decide whether an entry's give-up of a provider holds or is set aside.

 Xiaomi's meter always reads wet and its one seat is a checker, so with the roster that leaves Xiaomi out, Synthetic
 given up beside a wet Xiaomi is the only writing provider not dry and its cooldown is waited out; with no roster every
 provider counts, Xiaomi is an alternative, and the give-up holds. A floor source decides the rest: the give-up is set
 aside when Synthetic's return would leave no floor unmet where without it one is, and holds when a floor stays unmet
 with Synthetic back as well. The floor source is asked about the providers left alone and with Synthetic back. The
 meters of every other provider are refused by the transport, and the budgets run on a fake clock, so no case waits and
 no provider is asked.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  type DisposableSandbox,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { consoleLinesDuring, } from '@monochromatic-dev/translation-repair-provider/ts/neuralwatt-ledger.test-fixture.ts';
import { TEST_MAX_HOLD_MS, } from '@monochromatic-dev/translation-repair-provider/ts/budget-hold.test-fixture.ts';
import {
  type ProviderRecord,
  providerRecord,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { configureProviders, } from '../../dist/final/node/index.mjs';

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
 Refuses Synthetic, lets the cooldown run out, refuses it again at once, and asks the holds.

 @param writing - roster handed to `configureProviders`, none where the case hands none

 @param sinon - case's own sandbox, which fakes the clock the budgets count on

 @returns How long the budgets say Synthetic's hold is worth waiting for

 @example
 ```ts
 const waitedMs = await waitedAfterTwoRefusals({ sinon: ctx.sinon, },);
 ```
 */
async function waitedAfterTwoRefusals(
  {
    writing,
    unmetFloors,
    sinon,
  }: {
    readonly writing?: ProviderRecord<boolean>;
    readonly unmetFloors?: (args: { readonly dry: ProviderRecord<boolean>; },) => readonly string[];
    readonly sinon: DisposableSandbox;
  },
): Promise<number> {
  /**
   The fake clock the budgets read through `performance`.
   */
  const clock = sinon.useFakeTimers({
    toFake: [
      'setTimeout',
      'clearTimeout',
      'performance',
    ],
    shouldClearNativeTimers: true,
  },);
  using _installed = {
    [Symbol.dispose]: function removeClock(): void {
      clock.restore();
    },
  };
  /**
   The providers with Synthetic and Xiaomi on and every meter refused.
   */
  const providers = configureProviders({
    maxHoldMs: TEST_MAX_HOLD_MS,
    env: {
      TRANSLATION_REPAIR_SYNTHETIC_API_KEY: 'calico-not-a-real-key',
      TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY: 'tabby-not-a-real-key',
    },
    transport: async function refuseEveryMeter() {
      return {
        status: 400,
        bodyText: '{}',
      };
    },
    off: [],
    ...((writing === undefined) ? {} : { writing, }),
    ...((unmetFloors === undefined) ? {} : { unmetFloors, }),
  },);
  await providers.budgets.markRefused({
    provider: 'synthetic',
    signal: SIGNAL,
  },);
  await clock.tickAsync(PAST_COOLDOWN_MS,);
  await providers.budgets.markRefused({
    provider: 'synthetic',
    signal: SIGNAL,
  },);
  return providers.budgets.holds().synthetic;
}

await describe({
  name: `${configureProviders.name} writing roster and bench floors`,
  // One case at a time: each captures the console and fakes the clock, which are one for the whole process.
  concurrency: 1,
  children: [
    it({
      name: 'HANDS THE ROSTER TO THE BUDGETS: with Xiaomi outside it, a wet Xiaomi is no alternative to a given-up '
        + 'Synthetic, whose cooldown is waited out',
      fn: async ctx => {
        /**
         What the budgets said of Synthetic's hold.
         */
        const said = { waitedMs: -1, };
        await consoleLinesDuring({
          run: async function refuseBesideACheckerAlone(): Promise<void> {
            said.waitedMs = await waitedAfterTwoRefusals({
              sinon: ctx.sinon,
              writing: providerRecord({
                of: function writesExceptXiaomi(provider,): boolean {
                  return provider !== 'xiaomi';
                },
              },),
            },);
          },
          sinon: ctx.sinon,
        },);
        expect(said.waitedMs,).toBe(COOLDOWN_MS,);
      },
    },),

    it({
      name: 'COUNTS EVERY PROVIDER WHEN NO ROSTER IS HANDED OVER: a wet Xiaomi is an alternative, so the give-up of '
        + 'Synthetic holds and its cooldown counts for no wait',
      fn: async ctx => {
        /**
         What the budgets said of Synthetic's hold.
         */
        const said = { waitedMs: -1, };
        await consoleLinesDuring({
          run: async function refuseBesideAChecker(): Promise<void> {
            said.waitedMs = await waitedAfterTwoRefusals({ sinon: ctx.sinon, },);
          },
          sinon: ctx.sinon,
        },);
        expect(said.waitedMs,).toBe(0,);
      },
    },),

    it({
      name: 'HANDS THE FLOORS TO THE BUDGETS: with a bench floor that only Synthetic\'s return lifts, the give-up of '
        + 'Synthetic beside a wet Xiaomi is set aside and its cooldown waited out, and the floor source is asked about '
        + 'Xiaomi alone and then about Xiaomi with Synthetic back',
      fn: async ctx => {
        /**
         What the budgets said of Synthetic's hold, and the views the floor source was asked about.
         */
        const said: { waitedMs: number; views: ProviderRecord<boolean>[]; } = {
          waitedMs: -1,
          views: [],
        };
        await consoleLinesDuring({
          run: async function refuseBesideACheckerWithAFloor(): Promise<void> {
            said.waitedMs = await waitedAfterTwoRefusals({
              sinon: ctx.sinon,
              unmetFloors: function aFloorOnlySyntheticLifts({ dry, },): readonly string[] {
                said.views.push(dry,);
                return dry.synthetic ? ['translators 0 of 8 reachable, floor 2',] : [];
              },
            },);
          },
          sinon: ctx.sinon,
        },);
        expect(said.waitedMs,).toBe(COOLDOWN_MS,);
        expect(said.views.slice(-2,),).toEqual([
          providerRecord({
            of: function dryExceptXiaomi(provider,): boolean {
              return provider !== 'xiaomi';
            },
          },),
          providerRecord({
            of: function dryExceptXiaomiAndSynthetic(provider,): boolean {
              return (provider !== 'xiaomi') && (provider !== 'synthetic');
            },
          },),
        ],);
      },
    },),

    it({
      name: 'KEEPS THE GIVE-UP WHEN SYNTHETIC\'S RETURN LIFTS NOTHING: a floor that stays unmet with Synthetic back as '
        + 'well leaves its cooldown counting for no wait, though Xiaomi is no writing provider\'s alternative to wait for',
      fn: async ctx => {
        /**
         What the budgets said of Synthetic's hold.
         */
        const said = { waitedMs: -1, };
        await consoleLinesDuring({
          run: async function refuseBesideACheckerWithAHopelessFloor(): Promise<void> {
            said.waitedMs = await waitedAfterTwoRefusals({
              sinon: ctx.sinon,
              unmetFloors: function aFloorStaysUnmet(): readonly string[] {
                return ['translators 0 of 8 reachable, floor 2',];
              },
            },);
          },
          sinon: ctx.sinon,
        },);
        expect(said.waitedMs,).toBe(0,);
      },
    },),

    it({
      name: 'LETS THE ROSTER DECIDE FIRST: with Xiaomi outside it, Synthetic is the only writing provider not dry and its '
        + 'cooldown is waited out, though the floor source finds nothing unmet',
      fn: async ctx => {
        /**
         What the budgets said of Synthetic's hold.
         */
        const said = { waitedMs: -1, };
        await consoleLinesDuring({
          run: async function refuseBesideACheckerWithNoFloor(): Promise<void> {
            said.waitedMs = await waitedAfterTwoRefusals({
              sinon: ctx.sinon,
              writing: providerRecord({
                of: function writesExceptXiaomi(provider,): boolean {
                  return provider !== 'xiaomi';
                },
              },),
              unmetFloors: function everyFloorMet(): readonly string[] {
                return [];
              },
            },);
          },
          sinon: ctx.sinon,
        },);
        expect(said.waitedMs,).toBe(COOLDOWN_MS,);
      },
    },),
  ],
},);
