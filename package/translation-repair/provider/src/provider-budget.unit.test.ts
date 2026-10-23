/**
 Tests for the cached budget view: what it reads, what it caches, what a
 refused call corrects, and what it does when a meter cannot be read at all.

 @module
 */

import { wait, } from '@monochromatic-dev/module-async-time/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { createProviderBudgets, } from '../dist/final/node/index.mjs';

import {
  HOUR_MS,
  stubWallClock,
  WALL_START_MS,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/wall-clock-stub.test-fixture.ts';
import { TEST_MAX_HOLD_MS, } from './budget-hold.test-fixture.ts';

/**
 Quota snapshot of a provider with budget left.
 */
const WET_QUOTA = {
  fiveHour: {
    limited: false,
    remaining: 400,
    max: 500,
    nextTickAt: '2026-08-24T10:00:00.000Z',
  },
  weekly: {
    percentRemaining: 62,
    nextRegenAt: '2026-08-24T13:24:00.000Z',
  },
};

/**
 Quota snapshot of a provider whose weekly credit ran out, which is the
 state multi-provider routing was built for.
 */
const DRY_QUOTA = {
  ...WET_QUOTA,
  weekly: {
    percentRemaining: 0,
    nextRegenAt: '2026-08-24T13:24:00.000Z',
  },
};

/**
 Quota snapshot of a reply with no five-hour window, the shape Synthetic's reply has had since 2026-10-19, weekly credit left.
 */
const WEEKLY_ONLY_QUOTA = { weekly: WET_QUOTA.weekly, };

/**
 OpenRouter credits with money left, the live reading of 2026-09-03.
 */
const WET_CREDITS = {
  purchasedUsd: 1_913,
  usedUsd: 1_855.38,
  remainingUsd: 57.62,
};

/**
 Builds stub meters that answer as told.

 @param quota - what the first provider's quota endpoint returns

 @param balance - what the second provider's balance endpoint returns

 @param remainingUsd - what the third provider's credits endpoint leaves

 @param quotaThrows - whether the first provider's meter is unreachable

 @returns Every meter plus the count of reads each took

 @example
 ```ts
 const { synthetic, hyper, openrouter, reads, } = stubProviders({ quota: WET_QUOTA, balance: 243, },);
 ```
 */
function stubProviders(
  {
    quota = WET_QUOTA,
    balance = 243,
    remainingUsd = WET_CREDITS.remainingUsd,
    quotaThrows = false,
  }: {
    readonly quota?: {
      readonly fiveHour?: typeof WET_QUOTA.fiveHour;
      readonly weekly: typeof WET_QUOTA.weekly;
    };
    readonly balance?: number;
    readonly remainingUsd?: number;
    readonly quotaThrows?: boolean;
  },
) {
  /**
   How many times each meter was read.
   */
  const reads = {
    quota: 0,
    credits: 0,
    openrouter: 0,
  };

  return {
    reads,
    synthetic: {
      quotas: async function quotas() {
        reads.quota += 1;
        if (quotaThrows)
          throw new Error('meter unreachable',);
        return quota;
      },
    },
    hyper: {
      credits: async function credits() {
        reads.credits += 1;
        return { balance, };
      },
    },
    openrouter: {
      credits: async function credits() {
        reads.openrouter += 1;
        return {
          ...WET_CREDITS,
          remainingUsd,
        };
      },
    },
  };
}

/**
 Abort signal every read in these tests carries.
 */
const SIGNAL = new AbortController().signal;

/**
 Nobody dry.
 */
const ALL_WET = {
  neuralwatt: true,
  synthetic: false,
  hyper: false,
  bedrock: true,
  deepseek: true,
  xiaomi: true,
  anthropic: true,
  openrouter: false,
} as const;

await describe({
  name: createProviderBudgets.name,
  children: [
    it({
      name: 'reads every meter and reports a spendable roster',
      fn: async () => {
        /** Stub providers with budget everywhere. */
        const { synthetic, hyper, openrouter, reads, } = stubProviders({},);
        /** Budget view under test. */
        const budgets = createProviderBudgets({
          maxHoldMs: TEST_MAX_HOLD_MS,
          synthetic,
          hyper,
          openrouter,
        },);

        expect(await budgets.read({ signal: SIGNAL, },),).toEqual(ALL_WET,);
        expect(reads,).toEqual({ quota: 1, credits: 1, openrouter: 1, },);
        // The fourth provider has no client here, so it reads as dry and is
        // never asked; `bedrock: true` in every view this file expects says so.
      },
    },),

    it({
      name: 'TREATS EACH UNCONFIGURED PROVIDER AS DRY while the others remain wet',
      fn: async () => {
        /** Stub providers with budget everywhere. */
        const { synthetic, hyper, openrouter, } = stubProviders({},);
        const syntheticOnly = createProviderBudgets({ maxHoldMs: TEST_MAX_HOLD_MS, synthetic, },);
        const hyperOnly = createProviderBudgets({ maxHoldMs: TEST_MAX_HOLD_MS, hyper, },);
        const openRouterOnly = createProviderBudgets({ maxHoldMs: TEST_MAX_HOLD_MS, openrouter, },);

        expect(await syntheticOnly.read({ signal: SIGNAL, },),).toEqual({
          neuralwatt: true,
          synthetic: false,
          hyper: true,
          bedrock: true,
          deepseek: true,
          xiaomi: true,
          anthropic: true,
          openrouter: true,
        },);
        expect(await hyperOnly.read({ signal: SIGNAL, },),).toEqual({
          neuralwatt: true,
          synthetic: true,
          hyper: false,
          bedrock: true,
          deepseek: true,
          xiaomi: true,
          anthropic: true,
          openrouter: true,
        },);
        expect(await openRouterOnly.read({ signal: SIGNAL, },),).toEqual({
          neuralwatt: true,
          synthetic: true,
          hyper: true,
          bedrock: true,
          deepseek: true,
          xiaomi: true,
          anthropic: true,
          openrouter: false,
        },);
      },
    },),

    it({
      name: 'reads an exhausted weekly credit as dry',
      fn: async () => {
        /** Stub providers with the first one's weekly credit spent. */
        const { synthetic, hyper, openrouter, } = stubProviders({ quota: DRY_QUOTA, },);
        /** Budget view under test. */
        const budgets = createProviderBudgets({
          maxHoldMs: TEST_MAX_HOLD_MS,
          synthetic,
          hyper,
          openrouter,
        },);

        expect(await budgets.read({ signal: SIGNAL, },),).toEqual({
          ...ALL_WET,
          synthetic: true,
        },);
      },
    },),

    it({
      name: 'READS A REPLY WITH NO FIVE-HOUR WINDOW BY ITS WEEKLY CREDIT: wet with credit left, dry once the week is spent',
      fn: async () => {
        /** Stub providers whose first one answers with a weekly block alone, credit left. */
        const wet = stubProviders({ quota: WEEKLY_ONLY_QUOTA, },);
        /** Stub providers whose first one answers with a weekly block alone, credit spent. */
        const dry = stubProviders({
          quota: {
            weekly: {
              ...WET_QUOTA.weekly,
              percentRemaining: 0,
            },
          },
        },);

        expect(await createProviderBudgets({
          maxHoldMs: TEST_MAX_HOLD_MS,
          synthetic: wet.synthetic,
          hyper: wet.hyper,
          openrouter: wet.openrouter,
        },).read({ signal: SIGNAL, },),).toEqual(ALL_WET,);
        expect(await createProviderBudgets({
          maxHoldMs: TEST_MAX_HOLD_MS,
          synthetic: dry.synthetic,
          hyper: dry.hyper,
          openrouter: dry.openrouter,
        },).read({ signal: SIGNAL, },),).toEqual({
          ...ALL_WET,
          synthetic: true,
        },);
      },
    },),

    it({
      name: 'reads a spent balance and spent credits as dry',
      fn: async () => {
        /** Stub providers with the second and third out of money. */
        const { synthetic, hyper, openrouter, } = stubProviders({
          balance: 0,
          remainingUsd: 0,
        },);
        /** Budget view under test. */
        const budgets = createProviderBudgets({
          maxHoldMs: TEST_MAX_HOLD_MS,
          synthetic,
          hyper,
          openrouter,
        },);

        expect(await budgets.read({ signal: SIGNAL, },),).toEqual({
          neuralwatt: true,
          synthetic: false,
          hyper: true,
          bedrock: true,
          deepseek: true,
          xiaomi: true,
          anthropic: true,
          openrouter: true,
        },);
      },
    },),

    it({
      name: 'ACCEPTS an unreadable meter as spendable rather than exhausted',
      fn: async () => {
        /** Stub providers with the first one's meter unreachable. */
        const { synthetic, hyper, openrouter, } = stubProviders({ quotaThrows: true, },);
        /** Budget view under test. */
        const budgets = createProviderBudgets({
          maxHoldMs: TEST_MAX_HOLD_MS,
          synthetic,
          hyper,
          openrouter,
        },);

        // THE DECISION THIS FILE TURNS ON. A meter that times out is a
        // monitoring failure; reading it as exhaustion converts it into an
        // outage, stopping calls that would have succeeded. Being wrong this
        // way costs one refused call, which the router recovers from.
        expect(await budgets.read({ signal: SIGNAL, },),).toEqual(ALL_WET,);
      },
    },),

    it({
      name: 'READS WHAT EACH METER SAID beside the routed view, so a meter that could not be read is told apart '
        + 'from one that answered wet although both route as spendable',
      fn: async () => {
        /** Stub providers with the first one's meter unreachable and the second one's spent. */
        const { synthetic, hyper, openrouter, } = stubProviders({ quotaThrows: true, balance: 0, },);
        /** Budget view under test. */
        const budgets = createProviderBudgets({
          maxHoldMs: TEST_MAX_HOLD_MS,
          synthetic,
          hyper,
          openrouter,
        },);

        expect(await budgets.readStates({ signal: SIGNAL, },),).toEqual({
          neuralwatt: 'dry',
          synthetic: 'unreadable',
          hyper: 'dry',
          bedrock: 'dry',
          deepseek: 'dry',
          xiaomi: 'dry',
          anthropic: 'dry',
          openrouter: 'wet',
        },);
        expect(await budgets.read({ signal: SIGNAL, },),).toEqual({
          neuralwatt: true,
          synthetic: false,
          hyper: true,
          bedrock: true,
          deepseek: true,
          xiaomi: true,
          anthropic: true,
          openrouter: false,
        },);
      },
    },),

    it({
      name: 'SHARES ONE METER READ between the states and the view inside a window',
      fn: async () => {
        /** Stub providers with budget everywhere. */
        const { synthetic, hyper, openrouter, reads, } = stubProviders({},);
        /** Budget view under test. */
        const budgets = createProviderBudgets({
          maxHoldMs: TEST_MAX_HOLD_MS,
          synthetic,
          hyper,
          openrouter,
        },);

        await budgets.read({ signal: SIGNAL, },);
        await budgets.readStates({ signal: SIGNAL, },);
        expect(reads.quota,).toBe(1,);
      },
    },),

    it({
      name: 'spends one meter read per window rather than one per call',
      fn: async () => {
        /** Stub providers with budget everywhere. */
        const { synthetic, hyper, openrouter, reads, } = stubProviders({},);
        /** Clock the cache is judged against. */
        let clock = 1_000;
        /** Budget view under test, on an injected clock. */
        const budgets = createProviderBudgets({
          maxHoldMs: TEST_MAX_HOLD_MS,
          synthetic,
          hyper,
          openrouter,
          freshForMs: 500,
          now: () => clock,
        },);

        await budgets.read({ signal: SIGNAL, },);
        await budgets.read({ signal: SIGNAL, },);
        expect(reads.quota,).toBe(1,);
        expect(reads.openrouter,).toBe(1,);

        clock += 500;
        await budgets.read({ signal: SIGNAL, },);
        expect(reads.quota,).toBe(2,);
        expect(reads.openrouter,).toBe(2,);
      },
    },),

    it({
      name: 'HOLDS A PROVIDER THAT REFUSED US WHILE ITS METER READS WET for the rate-limit backoff, '
        + 'not the exhaustion cooldown, re-reading the meter at once: a 429 from a wet provider is '
        + 'its concurrency limit, and the five-minute hold on it is what ended the pin pass of '
        + '2026-09-02',
      fn: async () => {
        /** Stub providers whose meters all report budget left. */
        const { synthetic, hyper, openrouter, reads, } = stubProviders({},);
        /** Clock the holds are judged against. */
        let clock = 1_000;
        /** Budget view under test, on an injected clock. */
        const budgets = createProviderBudgets({
          maxHoldMs: TEST_MAX_HOLD_MS,
          synthetic,
          hyper,
          openrouter,
          cooldownMs: 10_000,
          rateLimitBackoffMs: 300,
          now: () => clock,
        },);

        await budgets.read({ signal: SIGNAL, },);
        await budgets.markRefused({
          provider: 'hyper',
          signal: SIGNAL,
        },);
        // The refusal forced a second read inside the freshness window.
        expect(reads.credits,).toBe(2,);
        expect(budgets.holds(),).toEqual({
          neuralwatt: 0,
          synthetic: 0,
          hyper: 300,
          bedrock: 0,
          deepseek: 0,
          xiaomi: 0,
          anthropic: 0,
          openrouter: 0,
        },);

        // Held out for the backoff even though the meter says spendable.
        expect(await budgets.read({ signal: SIGNAL, },),).toEqual({
          ...ALL_WET,
          hyper: true,
        },);

        clock += 300;
        expect(budgets.holds(),).toEqual({
          neuralwatt: 0,
          synthetic: 0,
          hyper: 0,
          bedrock: 0,
          deepseek: 0,
          xiaomi: 0,
          anthropic: 0,
          openrouter: 0,
        },);
        expect(await budgets.read({ signal: SIGNAL, },),).toEqual(ALL_WET,);
      },
    },),

    it({
      name: 'READS A PROVIDER DRY UNTIL ITS METER MOVES after a payment refusal on a wet meter, placing '
        + 'no timed hold for it, the fourteenth class: OpenRouter at 0.01 USD answered every call 402 '
        + 'and came back to the same wall every minute (hakureico, 2026-09-07)',
      fn: async () => {
        /** Stub providers whose meters all report budget left. */
        const { synthetic, hyper, } = stubProviders({},);
        /** OpenRouter's balance, which a top-up moves. */
        const balance = { usd: 0.01, };
        /** Clock the reads are judged against. */
        let clock = 1_000;
        /** Budget view under test, on an injected clock and a movable balance. */
        const budgets = createProviderBudgets({
          maxHoldMs: TEST_MAX_HOLD_MS,
          synthetic,
          hyper,
          openrouter: {
            credits: async () => ({
              ...WET_CREDITS,
              remainingUsd: balance.usd,
            }),
          },
          freshForMs: 100,
          cooldownMs: 10_000,
          rateLimitBackoffMs: 300,
          now: () => clock,
        },);

        expect((await budgets.read({ signal: SIGNAL, },)).openrouter,).toBe(false,);
        await budgets.markRefused({
          provider: 'openrouter',
          signal: SIGNAL,
          paymentRequired: true,
        },);
        // Dry at once, with no hold to wait out: the balance is the reason.
        expect((await budgets.read({ signal: SIGNAL, },)).openrouter,).toBe(true,);
        expect(budgets.holds().openrouter,).toBe(0,);

        // Still dry after every backoff and cooldown has passed, since the
        // meter still reads the balance that refused.
        clock += 20_000;
        expect((await budgets.read({ signal: SIGNAL, },)).openrouter,).toBe(true,);

        // A top-up moves the meter, and the next reading brings it back.
        balance.usd = 5;
        clock += 200;
        expect((await budgets.read({ signal: SIGNAL, },)).openrouter,).toBe(false,);
      },
    },),

    it({
      name: 'HOLDS A PROVIDER FOR THE WAIT ITS REFUSAL NAMES past the rate-limit backoff, whatever the '
        + 'meter reads: Hyper\'s daily limit said "try again in 2h25m18s" on a meter reading wet '
        + '(Huasheng, 2026-09-07), and a 60 s hold walked back into it 831 times',
      fn: async () => {
        /** Stub providers whose meters all report budget left. */
        const { synthetic, hyper, openrouter, } = stubProviders({},);
        /** Clock the holds are judged against. */
        let clock = 1_000;
        /** Budget view under test, on an injected clock. */
        const budgets = createProviderBudgets({
          maxHoldMs: TEST_MAX_HOLD_MS,
          synthetic,
          hyper,
          openrouter,
          cooldownMs: 10_000,
          rateLimitBackoffMs: 300,
          now: () => clock,
        },);

        await budgets.read({ signal: SIGNAL, },);
        await budgets.markRefused({
          provider: 'hyper',
          signal: SIGNAL,
          statedWaitMs: 5_000,
        },);
        expect(budgets.holds().hyper,).toBe(5_000,);

        // Still out once the backoff alone would have ended.
        clock += 300;
        expect(budgets.holds().hyper,).toBe(4_700,);
        expect((await budgets.read({ signal: SIGNAL, },)).hyper,).toBe(true,);

        clock += 4_700;
        expect(budgets.holds().hyper,).toBe(0,);
        expect((await budgets.read({ signal: SIGNAL, },)).hyper,).toBe(false,);
      },
    },),

    it({
      name: 'HOLDS A PROVIDER FOR THE WAIT ITS REFUSAL NAMES even when every other provider is dry and '
        + 'the meter alone would hold nothing, since a provider that named its return is not coming back '
        + 'sooner for being asked',
      fn: async () => {
        /** Stub providers with the second and third out of money. */
        const { synthetic, hyper, openrouter, } = stubProviders({
          balance: 0,
          remainingUsd: 0,
        },);
        /** Clock the holds are judged against. */
        const clock = 1_000;
        /** Budget view under test, on an injected clock. */
        const budgets = createProviderBudgets({
          maxHoldMs: TEST_MAX_HOLD_MS,
          synthetic,
          hyper,
          openrouter,
          cooldownMs: 10_000,
          rateLimitBackoffMs: 300,
          now: () => clock,
        },);

        await budgets.read({ signal: SIGNAL, },);
        await budgets.markRefused({
          provider: 'synthetic',
          signal: SIGNAL,
          statedWaitMs: 5_000,
        },);
        expect(budgets.holds().synthetic,).toBe(5_000,);
      },
    },),

    it({
      name: 'HOLDS NOTHING when a wet provider refuses us and every other provider is dry, since a hold '
        + 'with nowhere to move traffic only herds every call into the same limit when it ends '
        + '(XIEPT2 on Hyper alone, 2026-09-03: 429 bursts every 80 seconds and no slice settled in five '
        + 'minutes); the transport ladder paces each call instead',
      fn: async () => {
        /** Stub providers with the second and third out of money. */
        const { synthetic, hyper, openrouter, reads, } = stubProviders({
          balance: 0,
          remainingUsd: 0,
        },);
        /** Clock the holds are judged against. */
        const clock = 1_000;
        /** Budget view under test. */
        const budgets = createProviderBudgets({
          maxHoldMs: TEST_MAX_HOLD_MS,
          synthetic,
          hyper,
          openrouter,
          cooldownMs: 10_000,
          rateLimitBackoffMs: 300,
          now: () => clock,
        },);

        await budgets.read({ signal: SIGNAL, },);
        await budgets.markRefused({
          provider: 'synthetic',
          signal: SIGNAL,
        },);
        expect(reads.quota,).toBe(2,);
        expect(budgets.holds(),).toEqual({
          neuralwatt: 0,
          synthetic: 0,
          hyper: 0,
          bedrock: 0,
          deepseek: 0,
          xiaomi: 0,
          anthropic: 0,
          openrouter: 0,
        },);
        // The refuser is still spendable at once; only the dry providers are out.
        expect(await budgets.read({ signal: SIGNAL, },),).toEqual({
          neuralwatt: true,
          synthetic: false,
          hyper: true,
          bedrock: true,
          deepseek: true,
          xiaomi: true,
          anthropic: true,
          openrouter: true,
        },);
      },
    },),

    it({
      name: 'HOLDS a wet refuser for the backoff when ANY other provider is wet, so with Hyper dry '
        + 'the traffic moves to OpenRouter rather than herding',
      fn: async () => {
        /** Stub providers with only the second out of money. */
        const { synthetic, hyper, openrouter, } = stubProviders({ balance: 0, },);
        /** Budget view under test. */
        const budgets = createProviderBudgets({
          maxHoldMs: TEST_MAX_HOLD_MS,
          synthetic,
          hyper,
          openrouter,
          cooldownMs: 10_000,
          rateLimitBackoffMs: 300,
          now: () => 1_000,
        },);

        await budgets.markRefused({
          provider: 'synthetic',
          signal: SIGNAL,
        },);
        expect(budgets.holds(),).toEqual({
          neuralwatt: 0,
          synthetic: 300,
          hyper: 0,
          bedrock: 0,
          deepseek: 0,
          xiaomi: 0,
          anthropic: 0,
          openrouter: 0,
        },);
      },
    },),

    it({
      name: 'HOLDS A PROVIDER OUT FOR THE WHOLE COOLDOWN when its meter cannot be read on the refusal, '
        + 'since a refusal is stickier than a reading that never came',
      fn: async () => {
        /** Stub providers whose first meter is unreachable. */
        const { synthetic, hyper, openrouter, } = stubProviders({ quotaThrows: true, },);
        /** Clock the holds are judged against. */
        let clock = 1_000;
        /** Budget view under test, on an injected clock. */
        const budgets = createProviderBudgets({
          maxHoldMs: TEST_MAX_HOLD_MS,
          synthetic,
          hyper,
          openrouter,
          cooldownMs: 300,
          rateLimitBackoffMs: 50,
          now: () => clock,
        },);

        await budgets.markRefused({
          provider: 'synthetic',
          signal: SIGNAL,
        },);
        expect(budgets.holds().synthetic,).toBe(300,);

        clock += 50;
        // Past the backoff, still inside the cooldown.
        expect((await budgets.read({ signal: SIGNAL, },)).synthetic,).toBe(true,);

        clock += 250;
        // The cooldown ended and an unreadable meter counts as spendable.
        expect((await budgets.read({ signal: SIGNAL, },)).synthetic,).toBe(false,);
      },
    },),

    it({
      name: 'never lets a cooldown expiring bring a spent provider back',
      fn: async () => {
        /** Stub providers with the second one's balance at nothing. */
        const { synthetic, hyper, openrouter, } = stubProviders({ balance: 0, },);
        /** Clock the cooldown is judged against. */
        let clock = 1_000;
        /** Budget view under test, on an injected clock. */
        const budgets = createProviderBudgets({
          maxHoldMs: TEST_MAX_HOLD_MS,
          synthetic,
          hyper,
          openrouter,
          cooldownMs: 300,
          now: () => clock,
        },);

        await budgets.markRefused({
          provider: 'hyper',
          signal: SIGNAL,
        },);
        // A dry meter agrees with the refusal, so the hold is the cooldown.
        expect(budgets.holds().hyper,).toBe(300,);
        clock += 1_000;

        // The cooldown is one-directional: it can only hold a provider out.
        expect(await budgets.read({ signal: SIGNAL, },),).toEqual({
          ...ALL_WET,
          hyper: true,
        },);
      },
    },),

    it({
      name: 'COALESCES concurrent reads into ONE reading of each meter. The '
        + 'first shape checked staleness before the await and stamped after '
        + 'it, so every call arriving mid-read started its own: a live '
        + 'calibration spent 158 quota reads in 46.5 minutes against a '
        + '60-second window that allows about 46',
      fn: async () => {
        /** How many times each meter was read. */
        const reads = {
          quota: 0,
          credits: 0,
          openrouter: 0,
        };

        /** Meters slow enough that concurrent callers genuinely overlap. */
        const slow = {
          synthetic: {
            quotas: async function quotas() {
              reads.quota += 1;
              await wait(20,);
              return WET_QUOTA;
            },
          },
          hyper: {
            credits: async function credits() {
              reads.credits += 1;
              await wait(20,);
              return { balance: 243, };
            },
          },
          openrouter: {
            credits: async function credits() {
              reads.openrouter += 1;
              await wait(20,);
              return WET_CREDITS;
            },
          },
        };

        /** Budget layer over the slow meters. */
        const budgets = createProviderBudgets({ ...slow, maxHoldMs: TEST_MAX_HOLD_MS, },);

        /** Five calls launched before any of them can finish. */
        const views = await Promise.all([
          budgets.read({ signal: SIGNAL, },),
          budgets.read({ signal: SIGNAL, },),
          budgets.read({ signal: SIGNAL, },),
          budgets.read({ signal: SIGNAL, },),
          budgets.read({ signal: SIGNAL, },),
        ],);

        expect(reads,).toEqual({
          quota: 1,
          credits: 1,
          openrouter: 1,
        },);

        // Every sharer must get the reading, not just the caller that started it.
        for (const view of views)
          expect(view,).toEqual(ALL_WET,);
      },
    },),

    it({
      name: 'SHARES ONE FORCED READING among refusals arriving while it is in flight: the pin pass '
        + 'of 2026-09-02 saw eight 429s inside one second, and each forcing its own meter call '
        + 'would spend the reading budget the freshness window exists to protect',
      fn: async () => {
        /** How many times each meter was read. */
        const reads = {
          quota: 0,
          credits: 0,
          openrouter: 0,
        };

        /** Meters slow enough that concurrent refusals genuinely overlap. */
        const slow = {
          synthetic: {
            quotas: async function quotas() {
              reads.quota += 1;
              await wait(20,);
              return WET_QUOTA;
            },
          },
          hyper: {
            credits: async function credits() {
              reads.credits += 1;
              await wait(20,);
              return { balance: 243, };
            },
          },
          openrouter: {
            credits: async function credits() {
              reads.openrouter += 1;
              await wait(20,);
              return WET_CREDITS;
            },
          },
        };

        /** Budget layer over the slow meters, on a clock that does not move. */
        const budgets = createProviderBudgets({
          maxHoldMs: TEST_MAX_HOLD_MS,
          ...slow,
          rateLimitBackoffMs: 50,
          now: () => 1_000,
        },);

        await budgets.read({ signal: SIGNAL, },);
        // Three refusals landing before the reading the first one forced can finish.
        await Promise.all([
          budgets.markRefused({
            provider: 'hyper',
            signal: SIGNAL,
          },),
          budgets.markRefused({
            provider: 'hyper',
            signal: SIGNAL,
          },),
          budgets.markRefused({
            provider: 'synthetic',
            signal: SIGNAL,
          },),
        ],);

        // The first read plus ONE forced by the burst, not one per refusal.
        expect(reads,).toEqual({
          quota: 2,
          credits: 2,
          openrouter: 2,
        },);
        expect(budgets.holds(),).toEqual({
          neuralwatt: 0,
          synthetic: 50,
          hyper: 50,
          bedrock: 0,
          deepseek: 0,
          xiaomi: 0,
          anthropic: 0,
          openrouter: 0,
        },);
      },
    },),

    it({
      name: 'HOLDS A REFUSER NO LONGER THAN THE BACKOFF when the system clock is set back an hour after the '
        + 'refusal (ledger B78), where the default view, reading the wall clock, held it an hour and the backoff',
      fn: async ctx => {
        const wall = stubWallClock({ sinon: ctx.sinon, atMs: WALL_START_MS, },);
        /** Stub providers whose meters all report budget left. */
        const { synthetic, hyper, openrouter, } = stubProviders({},);
        /** Budget view under test, on its default clock. */
        const budgets = createProviderBudgets({
          maxHoldMs: TEST_MAX_HOLD_MS,
          synthetic,
          hyper,
          openrouter,
          cooldownMs: 10_000,
          rateLimitBackoffMs: 300,
        },);

        await budgets.read({ signal: SIGNAL, },);
        await budgets.markRefused({
          provider: 'hyper',
          signal: SIGNAL,
        },);
        wall.step({ byMs: -HOUR_MS, },);
        expect(budgets.holds().hyper,).toBeLessThanOrEqual(300,);
      },
    },),

    it({
      name: 'READS THE METERS ONCE for two reads inside the window when the clock read zero at the first '
        + '(ledger B78): a reading stamped 0 was taken for no reading at all and the meters were read again, '
        + 'which a clock counting from the process\'s start reads in its first millisecond',
      fn: async () => {
        /** Stub providers with budget everywhere. */
        const { synthetic, hyper, openrouter, reads, } = stubProviders({},);
        /** Budget view under test, on a clock still at its origin. */
        const budgets = createProviderBudgets({
          maxHoldMs: TEST_MAX_HOLD_MS,
          synthetic,
          hyper,
          openrouter,
          freshForMs: 500,
          now: () => 0,
        },);

        await budgets.read({ signal: SIGNAL, },);
        await budgets.read({ signal: SIGNAL, },);
        expect(reads,).toEqual({ quota: 1, credits: 1, openrouter: 1, },);
      },
    },),
    it({
      name: 'REPORTS THE NEURALWATT DAY TO A REFUSAL THAT ENDS THE RUN: off with no NeuralWatt client, unanchored '
        + 'while its client has decided no day start, and the decided start once it has',
      fn: async () => {
        /** Stub providers with budget everywhere. */
        const { synthetic, hyper, openrouter, } = stubProviders({},);
        /** The anchor the stub client has decided, none at first. */
        const decided: { anchor: { readonly known: false; } | { readonly known: true; readonly msOfDay: number; }; } = {
          anchor: { known: false, },
        };
        /** Budget view over no NeuralWatt client. */
        const without = createProviderBudgets({ maxHoldMs: TEST_MAX_HOLD_MS, synthetic, hyper, openrouter, },);
        /** Budget view over a stub NeuralWatt client. */
        const withClient = createProviderBudgets({
          maxHoldMs: TEST_MAX_HOLD_MS,
          neuralwatt: {
            // Asking for the NeuralWatt day must not read the energy.
            energy: async () => {
              throw new Error('the energy ledger must not be read to state the day',);
            },
            dayAnchor: () => decided.anchor,
          },
          synthetic,
          hyper,
          openrouter,
        },);

        expect(without.neuralwattReturn(),).toEqual({ kind: 'off', },);
        expect(withClient.neuralwattReturn(),).toEqual({ kind: 'unanchored', },);
        decided.anchor = {
          known: true,
          msOfDay: 69_308_000,
        };
        expect(withClient.neuralwattReturn(),).toEqual({
          kind: 'at',
          msOfDay: 69_308_000,
        },);
      },
    },),
  ],
},);
