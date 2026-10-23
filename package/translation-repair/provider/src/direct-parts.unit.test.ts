/**
 Tests for the parts DeepSeek's and Xiaomi's routes are built from that no module of their own owns: the meters and the
 place of both providers in the routing order. The balance parser, prices, usage readers, catalogs, reach and clients
 have a test file each beside their module.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { PROVIDER_ORDER, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  createProviderBudgets,
  DEEPSEEK_DRY_FLOOR_USD,
  deepSeekIsDry,
  deepSeekMeterLevel,
  EveryProviderDryError,
  readEveryMeter,
  reachOf,
  routeProviderFor,
  claudeIsDry,
  claudeMeterLevel,
  xiaomiIsDry,
  xiaomiMeterLevel,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_OPENROUTER_ONLY_CHECKER,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { TEST_MAX_HOLD_MS, } from './budget-hold.test-fixture.ts';

/**
 Every provider false, for a record the cases then set.
 */
const NONE_DRY = Object.fromEntries(PROVIDER_ORDER.map((provider,) => [provider, false,]),) as Record<
  (typeof PROVIDER_ORDER)[number],
  boolean
  >;

/**
 Abort signal every read here carries.
 */
const SIGNAL = new AbortController().signal;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'direct meters',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS DeepSeek dry when the account is unavailable or the balance is at or under the floor, wet '
            + 'above it',
          fn: async () => {
            expect(DEEPSEEK_DRY_FLOOR_USD,).toBe(0,);
            expect(deepSeekIsDry({ balance: { isAvailable: false, totalUsd: 50, }, },),).toBe(true,);
            expect(deepSeekIsDry({ balance: { isAvailable: true, totalUsd: 0, }, },),).toBe(true,);
            expect(deepSeekIsDry({ balance: { isAvailable: true, totalUsd: 0.01, }, },),).toBe(false,);
            expect(deepSeekMeterLevel({ balance: { isAvailable: true, totalUsd: 9.96, }, },),)
              .toEqual(['deepseekUsd=9.96',],);
          },
        },),
        it({
          name: 'READS Xiaomi wet always, and reports the run\'s priced spend to four places',
          fn: async () => {
            expect(xiaomiIsDry({ spend: { spentUsd: 1_000, }, },),).toBe(false,);
            expect(xiaomiMeterLevel({ spend: { spentUsd: 0.0042, }, },),).toEqual(['xiaomiUsdSpent=0.0042',],);
          },
        },),
        it({
          name: 'READS the Claude key wet until a billing refusal, then dry for the run, and reports the run\'s priced '
            + 'spend to four places',
          fn: async () => {
            expect(claudeIsDry({ spend: { spentUsd: 1_000, billingRefused: false, }, },),).toBe(false,);
            expect(claudeIsDry({ spend: { spentUsd: 0, billingRefused: true, }, },),).toBe(true,);
            expect(claudeMeterLevel({ spend: { spentUsd: 0.0042, billingRefused: false, }, },),)
              .toEqual(['anthropicUsdSpent=0.0042',],);
            expect(claudeMeterLevel({ spend: { spentUsd: 0.5, billingRefused: true, }, },),)
              .toEqual(['anthropicUsdSpent=0.5000', 'anthropicBilling=refused',],);
          },
        },),
        it({
          name: 'PUTS the Claude key in the METERS record: wet with its spend, dry once billing refused, and an unset '
            + 'key dry with nothing to report',
          fn: async () => {
            const keyed = await readEveryMeter({
              anthropic: { spend: async () => ({ spentUsd: 0.0042, billingRefused: false, }), },
              signal: SIGNAL,
            },);
            expect(keyed.anthropic,).toEqual({ state: 'wet', fields: ['anthropicUsdSpent=0.0042',], },);
            const refused = await readEveryMeter({
              anthropic: { spend: async () => ({ spentUsd: 0, billingRefused: true, }), },
              signal: SIGNAL,
            },);
            expect(refused.anthropic.state,).toBe('dry',);
            expect((await readEveryMeter({ signal: SIGNAL, },)).anthropic,).toEqual({ state: 'dry', fields: [], },);
          },
        },),
        it({
          name: 'PUTS both in the METERS record: DeepSeek by balance, Xiaomi wet with its spend, and an unset key dry '
            + 'with nothing to report',
          fn: async () => {
            const keyed = await readEveryMeter({
              deepseek: { credits: async () => ({ isAvailable: true, totalUsd: 9.96, }), },
              xiaomi: { spend: async () => ({ spentUsd: 0.0042, }), },
              signal: SIGNAL,
            },);
            expect(keyed.deepseek,).toEqual({ state: 'wet', fields: ['deepseekUsd=9.96',], },);
            expect(keyed.xiaomi,).toEqual({ state: 'wet', fields: ['xiaomiUsdSpent=0.0042',], },);
            const unset = await readEveryMeter({ signal: SIGNAL, },);
            expect(unset.deepseek,).toEqual({ state: 'dry', fields: [], },);
            expect(unset.xiaomi,).toEqual({ state: 'dry', fields: [], },);
            const failing = await readEveryMeter({
              deepseek: {
                credits: async () => {
                  throw new Error('balance offline',);
                },
              },
              signal: SIGNAL,
            },);
            expect(failing.deepseek.state,).toBe('unreadable',);
          },
        },),
        it({
          name: 'HOLDS Xiaomi dry after a payment refusal, which its always-wet meter cannot say, and DeepSeek dry '
            + 'when '
            + 'its balance is empty',
          fn: async () => {
            const budgets = createProviderBudgets({
              maxHoldMs: TEST_MAX_HOLD_MS,
              deepseek: { credits: async () => ({ isAvailable: true, totalUsd: 0, }), },
              xiaomi: { spend: async () => ({ spentUsd: 0.0042, }), },
            },);
            const first = await budgets.read({ signal: SIGNAL, },);
            expect(first.deepseek,).toBe(true,);
            expect(first.xiaomi,).toBe(false,);
            await budgets.markRefused({ provider: 'xiaomi', signal: SIGNAL, paymentRequired: true, },);
            const second = await budgets.read({ signal: SIGNAL, },);
            expect(second.xiaomi,).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: 'direct locks and routing order',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'WALKS the order for deepseek-v4.1-flash: NeuralWatt, then DeepSeek, then Hyper, then OpenRouter',
          fn: async () => {
            const reach = reachOf({ modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, },);
            expect(reach.deepseek,).toBe(true,);
            expect(reach.xiaomi,).toBe(false,);
            /**
             What the router picks given which providers are dry.

             @param dry - providers that read dry

             @returns The choice
             */
            const pick = (dry: readonly (typeof PROVIDER_ORDER)[number][]) => routeProviderFor({
              reach,
              dry: { ...NONE_DRY, ...Object.fromEntries(dry.map((provider,) => [provider, true,]),), },
              saturated: NONE_DRY,
            },);
            expect(pick([],),).toMatchObject({ kind: 'neuralwatt', },);
            expect(pick(['neuralwatt',],),).toMatchObject({ kind: 'deepseek', },);
            expect(pick(['neuralwatt', 'deepseek',],),).toMatchObject({ kind: 'hyper', },);
            expect(pick(['neuralwatt', 'deepseek', 'hyper',],),).toMatchObject({ kind: 'openrouter', },);
          },
        },),
        it({
          name: 'LOSES THE SEAT, NOT THE RUN, when only Xiaomi reads wet and the model is not Xiaomi\'s, still ends the '
            + 'run when Xiaomi reads dry too, and still sends Xiaomi\'s own model to Xiaomi',
          fn: async () => {
            const dryButXiaomi = { ...NONE_DRY, neuralwatt: true, synthetic: true, bedrock: true, deepseek: true,
              hyper: true, openrouter: true, anthropic: true, };
            expect(routeProviderFor({
              reach: reachOf({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, }),
              dry: dryButXiaomi,
              saturated: NONE_DRY,
            },).kind,).toBe('unreachable',);
            expect(function everyProviderDry(): unknown {
              return routeProviderFor({
                reach: reachOf({ modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE, }),
                dry: { ...dryButXiaomi, xiaomi: true, },
                saturated: NONE_DRY,
              },);
            },).toThrow(EveryProviderDryError,);
            expect(routeProviderFor({
              reach: reachOf({ modelId: SEAT_OPENROUTER_ONLY_CHECKER, }),
              dry: dryButXiaomi,
              saturated: NONE_DRY,
            },),).toMatchObject({ kind: 'xiaomi', },);
          },
        },),
        it({
          name: 'WALKS the order for mimo-v2.6-flash: Xiaomi before OpenRouter, OpenRouter once Xiaomi is dry',
          fn: async () => {
            const reach = reachOf({ modelId: SEAT_OPENROUTER_ONLY_CHECKER, },);
            expect(reach.xiaomi,).toBe(true,);
            expect(reach.deepseek,).toBe(false,);
            expect(reach.openrouter,).toBe(true,);
            expect(routeProviderFor({ reach, dry: NONE_DRY, saturated: NONE_DRY, },),)
              .toMatchObject({ kind: 'xiaomi', },);
            expect(routeProviderFor({ reach, dry: { ...NONE_DRY, xiaomi: true, }, saturated: NONE_DRY, },),)
              .toMatchObject({ kind: 'openrouter', },);
            expect(routeProviderFor({ reach, dry: NONE_DRY, saturated: { ...NONE_DRY, xiaomi: true, }, },),)
              .toMatchObject({ kind: 'openrouter', },);
          },
        },),
      ],
    },),
  ],
},);
