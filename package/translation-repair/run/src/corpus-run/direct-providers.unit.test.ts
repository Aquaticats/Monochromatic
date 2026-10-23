/**
 Tests for how the run takes in DeepSeek's and Xiaomi's own routes: the two
 keys, the refusals, the optional status, the spend bucket and the meter log.

 NO KEY HERE IS REAL and no request leaves the process: every transport is an
 in-memory script.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import {
  configureProviders,
  priceTally,
  readMeterLine,
  readRequiredProviders,
  REQUIRABLE_PROVIDERS,
  RunConfigError,
  tallySpend,
} from '../../dist/final/node/index.mjs';
import { lineOf, } from './command-line.test-fixture.ts';
import { TEST_MAX_HOLD_MS, } from '@monochromatic-dev/translation-repair-provider/ts/budget-hold.test-fixture.ts';

/**
 Transport that answers DeepSeek's balance and refuses everything else.

 @param request - exchange the client sent

 @returns A reply by address

 @example
 ```ts
 const reply = await balanceOnly({ url: 'https://api.deepseek.com/user/balance', },);
 ```
 */
async function balanceOnly(
  request: { readonly url: string; },
): Promise<{ readonly status: number; readonly bodyText: string; }> {
  if (request.url.endsWith('/user/balance',))
    return {
      status: 200,
      bodyText: JSON.stringify({
        is_available: true,
        balance_infos: [{ currency: 'USD', total_balance: '9.96', },],
      },),
    };
  return { status: 400, bodyText: '{}', };
}

/**
 Abort signal every read here carries.
 */
const SIGNAL = new AbortController().signal;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: configureProviders.name,
      concurrency: 1,
      children: [
        it({
          name: 'BUILDS DeepSeek on DEEPSEEK_API_KEY alone, reads its balance wet, and leaves every other provider '
            + 'and Xiaomi unbuilt',
          fn: async () => {
            const providers = configureProviders({
              maxHoldMs: TEST_MAX_HOLD_MS,
              env: { DEEPSEEK_API_KEY: 'cat-key-deepseek', },
              transport: balanceOnly,
              off: [],
            },);
            expect(providers.deepseek,).not.toBeUndefined();
            expect(providers.xiaomi,).toBeUndefined();
            expect(providers.openrouter,).toBeUndefined();
            const view = await providers.budgets.read({ signal: SIGNAL, },);
            expect(view.deepseek,).toBe(false,);
            expect(view.xiaomi,).toBe(true,);
          },
        },),
        it({
          name: 'BUILDS Xiaomi on TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY alone, reads it wet with no request, and '
            + 'exposes its pace',
          fn: async () => {
            const providers = configureProviders({
              maxHoldMs: TEST_MAX_HOLD_MS,
              env: { TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY: 'cat-key-xiaomi', },
              transport: async function noRequestExpected() {
                throw new Error('Xiaomi has no balance endpoint, so its meter must send nothing',);
              },
              off: [],
            },);
            expect(providers.xiaomi?.pace.waitMs(),).toBe(0,);
            expect(providers.deepseek,).toBeUndefined();
            const view = await providers.budgets.read({ signal: SIGNAL, },);
            expect(view.xiaomi,).toBe(false,);
            expect(view.deepseek,).toBe(true,);
          },
        },),
        it({
          name: 'NEVER BORROWS another provider\'s key: with only OpenRouter keyed, neither direct provider is built',
          fn: async () => {
            const providers = configureProviders({
              maxHoldMs: TEST_MAX_HOLD_MS,
              env: { TRANSLATION_REPAIR_OPENROUTER_API_KEY: 'cat-key-openrouter', },
              transport: balanceOnly,
              off: [],
            },);
            expect(providers.deepseek,).toBeUndefined();
            expect(providers.xiaomi,).toBeUndefined();
          },
        },),
        it({
          name: 'COUNTS EITHER DIRECT KEY as a key, and names both variables when there is none at all',
          fn: async () => {
            expect(function noKey(): unknown {
              return configureProviders({ maxHoldMs: TEST_MAX_HOLD_MS, env: {}, transport: balanceOnly, off: [], },);
            },).toThrow(RunConfigError,);
            try {
              configureProviders({ maxHoldMs: TEST_MAX_HOLD_MS, env: {}, transport: balanceOnly, off: [], },);
            }
            catch (error) {
              expect(String(error,),).toContain('DEEPSEEK_API_KEY, TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY',);
            }
          },
        },),
      ],
    },),

    describe({
      name: readRequiredProviders.name,
      concurrency: 1,
      children: [
        it({
          name: 'KEEPS BOTH DIRECT PROVIDERS OUT of what an arm may require, saying they are optional',
          fn: async () => {
            expect(REQUIRABLE_PROVIDERS.includes('deepseek' as never,),).toBe(false,);
            expect(REQUIRABLE_PROVIDERS.includes('xiaomi' as never,),).toBe(false,);
            for (const name of ['deepseek', 'xiaomi',]) {
              try {
                readRequiredProviders({
                  line: lineOf({ command: 'corpus-pass', typed: ['--require-providers', name,], },),
                },);
                throw new Error(`${name} must be refused`,);
              }
              catch (error) {
                expect(error,).toBeInstanceOf(StatedRefusalError,);
                expect((error as Error).message,).toBe(
                  `--require-providers accepts only neuralwatt, synthetic, bedrock, hyper, openrouter, and "${name}" `
                    + 'is an optional provider that no arm requires wet',
                );
              }
            }
          },
        },),
      ],
    },),

    describe({
      name: priceTally.name,
      concurrency: 1,
      children: [
        it({
          name: 'COUNTS DeepSeek and Xiaomi seats among the seats billed in USD, with the cost their lines reported',
          fn: async () => {
            const cost = priceTally({
              tally: tallySpend({
                lines: [
                  'SPEND provider=deepseek model=deepseek-flash prompt=100 completion=10 cost=0.00004',
                  'SPEND provider=xiaomi model=mimo-v2.6-flash prompt=100 completion=10 cost=0.00001',
                ],
              },),
            },);
            expect(cost.openRouter.map((seat,) => seat.provider,).toSorted(),).toEqual(['deepseek', 'xiaomi',],);
            expect(cost.totalUsd,).toBeCloseTo(0.00005, 12,);
            expect(cost.subscription.length,).toBe(0,);
            expect(cost.freeEnergy.length,).toBe(0,);
          },
        },),
      ],
    },),

    describe({
      name: readMeterLine.name,
      concurrency: 1,
      children: [
        it({
          name: 'READS the new providers\' states off a METERS line, and an older line as absent for both',
          fn: async () => {
            const stamp = '[info] [2026-10-22T17:00:00.000Z] [translation-repair] [takeReading] METERS';
            expect(readMeterLine({
              line: `${stamp} neuralwatt=dry synthetic=dry bedrock=dry deepseek=wet xiaomi=wet anthropic=wet hyper=dry openrouter=dry `
                + 'deepseekUsd=9.96 xiaomiUsdSpent=0.0042 anthropicUsdSpent=0.0100',
            },),).toMatchObject({
              deepseek: 'wet',
              xiaomi: 'wet',
              anthropic: 'wet',
              levels: ['deepseekUsd=9.96', 'xiaomiUsdSpent=0.0042', 'anthropicUsdSpent=0.0100',],
            },);
            expect(readMeterLine({ line: `${stamp} synthetic=wet hyper=dry`, },),).toMatchObject({
              deepseek: 'absent',
              xiaomi: 'absent',
              anthropic: 'absent',
            },);
          },
        },),
      ],
    },),
  ],
},);
