/**
 Tests explicit provider requirements for validation and performance arms.

 The gate reads its keys and the Bedrock ledger's place from the environment
 it is handed and its meters over the transport it is handed, so every case
 hands over its own of both: no case reads the process's keys, reaches a
 provider, or opens the real spend ledger (ledger M43, M68).

 Fixtures are invented; the keys are placeholders no provider issued.

 @module
 */

import { join, } from 'node:path';

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import {
  fetchTransport,
  type ModelTransport,
  type TransportExchange,
} from '@monochromatic-dev/translation-repair-provider-stream';
import { HYPER_CREDITS_URL, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  BEDROCK_CREDIT_USD_VAR,
  BEDROCK_LEDGER_PATH_VAR,
  NEURALWATT_DAILY_WH_VAR,
  NEURALWATT_LEDGER_PATH_VAR,
  OPENROUTER_CREDITS_URL,
} from '@monochromatic-dev/translation-repair-provider';
import {
  assertRequiredProvidersReady,
  readRequiredProviders,
  RequiredProviderError,
} from '../../dist/final/node/index.mjs';
import { warnLinesDuring, } from '@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts';
import {
  answerEveryFetchWithRefusal,
  ECHOED_KEY,
  statusFailureLogText,
} from '@monochromatic-dev/translation-repair-provider/ts/provider-status-failure.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { lineOf, } from './command-line.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { untilAborted, } from '@monochromatic-dev/translation-repair-provider/ts/stage-trivial-reply.test-fixture.ts';

/**
 Each provider's key environment name.
 */
const KEY_NAMES = {
  neuralwatt: 'TRANSLATION_REPAIR_NEURALWATT_API_KEY',
  synthetic: 'TRANSLATION_REPAIR_SYNTHETIC_API_KEY',
  hyper: 'TRANSLATION_REPAIR_CHARM_HYPER_API_KEY',
  bedrock: 'TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY',
  openrouter: 'TRANSLATION_REPAIR_OPENROUTER_API_KEY',
} as const;

/**
 Wet Synthetic meter response fixture.
 */
const WET_SYNTHETIC_BODY = JSON.stringify({
  weeklyTokenLimit: {
    nextRegenAt: '2026-08-30T00:00:00.000Z',
    percentRemaining: 75,
  },
  rollingFiveHourLimit: {
    nextTickAt: '2026-08-29T15:00:00.000Z',
    tickPercent: 0.05,
    remaining: 500,
    max: 750,
    limited: false,
  },
},);

/**
 Synthetic meter response fixture with no five-hour block, as the reply of 2026-10-19 came, weekly budget left.
 */
const WET_SYNTHETIC_WEEKLY_ONLY_BODY = JSON.stringify({
  subscription: { limit: 640, requests: 3, },
  weeklyTokenLimit: {
    nextRegenAt: '2026-10-19T17:57:00.000Z',
    percentRemaining: 64.5,
  },
},);

/**
 The same reply with the weekly budget spent.
 */
const DRY_SYNTHETIC_WEEKLY_ONLY_BODY = JSON.stringify({
  subscription: { limit: 640, requests: 3, },
  weeklyTokenLimit: {
    nextRegenAt: '2026-10-19T17:57:00.000Z',
    percentRemaining: 0,
  },
},);

/**
 Transport a case must never reach, since the gate refuses first or reads no
 meter over HTTP.

 @returns Never

 @throws Error always
 */
async function unreachedTransport(): ReturnType<ModelTransport> {
  throw new Error('the gate asked a meter this case never answers',);
}

/**
 Transport answering the two read-only routes NeuralWatt's meter reads for the day: the quota, whose
 `current_period_start` places the day, and the by-model usage, whose period echoes the window asked and whose rows
 show no covered energy used.

 @param exchange - request

 @returns Reply

 @example
 ```ts
 const reply = await neuralWattAccountTransport(exchange,);
 ```
 */
async function neuralWattAccountTransport(exchange: TransportExchange,): ReturnType<ModelTransport> {
  if (exchange.url.includes('/usage/by-model',)) {
    /**
     Query the request carried.
     */
    const { searchParams, } = new URL(exchange.url,);
    return {
      status: 200,
      bodyText: JSON.stringify({
        period: {
          start: (searchParams.get('start_date',) ?? '').replace('Z', '+00:00',),
          end: (searchParams.get('end_date',) ?? '').replace('Z', '000+00:00',),
        },
        models: [],
        products: [],
      },),
    };
  }
  return {
    status: 200,
    bodyText: JSON.stringify({
      snapshot_at: '2026-10-20T20:00:00Z',
      subscription: { plan: 'max', current_period_start: '2026-10-20T19:15:08Z', kwh_used: 13.9, },
    },),
  };
}

/**
 What the gate refused with, or that it did not refuse.

 @param act - gate call

 @returns Refusal, or `'passed'`
 */
async function gateOutcome(act: () => Promise<void>,): Promise<unknown> {
  try {
    await act();
    return 'passed';
  }
  catch (error) {
    return error;
  }
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: readRequiredProviders.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS ORDERED DISTINCT PROVIDER REQUIREMENT, the third provider included',
          fn: async () => {
            expect(readRequiredProviders({
              line: lineOf({
                command: 'corpus-pass',
                typed: [
                  '--require-providers',
                  'synthetic,openrouter,hyper,synthetic',
                ],
              },),
            },),).toEqual([
              'synthetic',
              'openrouter',
              'hyper',
            ],);
          },
        },),
        it({
          name: 'REQUIRES NOTHING when the flag is absent, which is every ordinary run (ledger T8)',
          fn: async () => {
            expect(readRequiredProviders({
              line: lineOf({
                command: 'corpus-pass',
                typed: [],
              },),
            },),).toEqual([],);
          },
        },),
        it({
          name: 'REFUSES the flag with no value, an empty value, separators naming nobody or a provider nobody '
            + 'serves, naming what it accepts (ledger T8, B75)',
          fn: async () => {
            for (
              const typed of [
                ['--require-providers',],
                ['--require-providers', '',],
                ['--require-providers', ',',],
                ['--require-providers', 'synthetic,catnip',],
              ]
            ) {
              /**
               What the reader raised for these arguments.
               */
              const refusal = caught(function readFlag(): unknown {
                return readRequiredProviders({
                  line: lineOf({
                    command: 'corpus-pass',
                    typed,
                  },),
                },);
              },);
              expect(refusal,).toBeInstanceOf(StatedRefusalError,);
              expect((refusal as Error).message,).toContain('neuralwatt, synthetic, bedrock, hyper, openrouter',);
            }
          },
        },),
        it({
          name: 'QUOTES the provider it refuses, so a spaced or mistyped name shows as typed (ledger B75)',
          fn: async () => {
            expect((caught(function readSpaced(): unknown {
              return readRequiredProviders({
                line: lineOf({
                  command: 'corpus-pass',
                  typed: ['--require-providers', 'synthetic,cat nip',],
                },),
              },);
            },) as Error).message,).toBe(
              '--require-providers accepts only neuralwatt, synthetic, bedrock, hyper, openrouter, and "cat nip" is none of them',
            );
          },
        },),
      ],
    },),

    describe({
      name: assertRequiredProvidersReady.name,
      concurrency: 1,
      children: [
        it({
          name: 'ASKS NO METER when nothing is required (ledger T8)',
          fn: async () => {
            expect(await gateOutcome(async function gateNothing() {
              await assertRequiredProvidersReady({
                required: [],
                env: {},
                transport: unreachedTransport,
                signal: AbortSignal.timeout(HANG_STOP_MS,),
              },);
            },),).toBe('passed',);
          },
        },),
        it({
          name: 'REFUSES MISSING REQUIRED KEY before transport call',
          fn: async () => {
            /**
             What the gate raised for a requirement whose key is unset.
             */
            const refusal = await gateOutcome(async function gateKeyless() {
              await assertRequiredProvidersReady({
                required: ['synthetic', 'hyper',],
                env: { [KEY_NAMES.hyper]: 'test-hyper', },
                transport: unreachedTransport,
                signal: AbortSignal.timeout(HANG_STOP_MS,),
              },);
            },);
            expect(refusal,).toBeInstanceOf(RequiredProviderError,);
            expect((refusal as Error).message,).toContain('synthetic is not ready: key missing',);
          },
        },),
        it({
          name: 'ACCEPTS EVERY WET METER without model endpoint call, OpenRouter\'s credits included',
          fn: async () => {
            const urls: string[] = [];
            await assertRequiredProvidersReady({
              required: ['synthetic', 'hyper', 'openrouter',],
              env: {
                [KEY_NAMES.synthetic]: 'test-synthetic',
                [KEY_NAMES.hyper]: 'test-hyper',
                [KEY_NAMES.openrouter]: 'test-openrouter',
              },
              transport: async function transport(exchange,) {
                urls.push(exchange.url,);
                if (exchange.url === HYPER_CREDITS_URL)
                  return { status: 200, bodyText: '{"balance":243}', };
                if (exchange.url === OPENROUTER_CREDITS_URL)
                  return { status: 200, bodyText: '{"data":{"total_credits":1913,"total_usage":1855.38}}', };
                return { status: 200, bodyText: WET_SYNTHETIC_BODY, };
              },
              signal: AbortSignal.timeout(HANG_STOP_MS,),
            },);
            expect(urls,).toHaveLength(3,);
            expect(urls,).toContain(HYPER_CREDITS_URL,);
            expect(urls,).toContain(OPENROUTER_CREDITS_URL,);
            expect(urls.some(function modelEndpoint(url,): boolean {
              return url.includes('/chat/',);
            },),).toBe(false,);
          },
        },),
        it({
          name: 'ACCEPTS A SYNTHETIC METER WITH NO FIVE-HOUR BLOCK by its weekly budget, and REFUSES it dry when that is spent',
          fn: async () => {
            /**
             What the gate raised over each reply, `passed` where it raised nothing.
             */
            const outcomes = await Promise.all([WET_SYNTHETIC_WEEKLY_ONLY_BODY, DRY_SYNTHETIC_WEEKLY_ONLY_BODY,].map(
              async function gateReply(bodyText,): Promise<unknown> {
                return await gateOutcome(async function gateWeeklyOnly() {
                  await assertRequiredProvidersReady({
                    required: ['synthetic',],
                    env: { [KEY_NAMES.synthetic]: 'test-synthetic', },
                    transport: async function transport() {
                      return { status: 200, bodyText, };
                    },
                    signal: AbortSignal.timeout(HANG_STOP_MS,),
                  },);
                },);
              },
            ),);
            expect(outcomes[0],).toBe('passed',);
            expect(outcomes[1],).toBeInstanceOf(RequiredProviderError,);
            expect((outcomes[1] as Error).message,).toContain('synthetic is not ready: budget dry',);
          },
        },),
        it({
          name: 'REFUSES A DRY REQUIRED METER, naming the provider, so a measured arm never starts on a '
            + 'provider that cannot serve it',
          fn: async () => {
            /**
             What the gate raised over a spent OpenRouter balance.
             */
            const refusal = await gateOutcome(async function gateDry() {
              await assertRequiredProvidersReady({
                required: ['openrouter',],
                env: { [KEY_NAMES.openrouter]: 'test-openrouter', },
                transport: async function transport() {
                  return { status: 200, bodyText: '{"data":{"total_credits":10,"total_usage":10}}', };
                },
                signal: AbortSignal.timeout(HANG_STOP_MS,),
              },);
            },);
            expect(refusal,).toBeInstanceOf(RequiredProviderError,);
            expect((refusal as Error).message,).toContain('openrouter is not ready: budget dry',);
          },
        },),
        it({
          name: 'REFUSES NAMING THE FIRST REQUIRED PROVIDER when two meters cannot be read and the later one '
            + 'is refused first',
          fn: async () => {
            /**
             Opened when the second provider's meter has been asked, which the
             first provider's answer waits for.
             */
            const laterAsked = Promise.withResolvers<undefined>();
            /**
             What the gate raised over two meters that refuse the key.
             */
            const refusal = await gateOutcome(async function gateBothUnreadable() {
              await assertRequiredProvidersReady({
                required: ['synthetic', 'hyper',],
                env: {
                  [KEY_NAMES.synthetic]: 'test-synthetic',
                  [KEY_NAMES.hyper]: 'test-hyper',
                },
                transport: async function refusesHyperFirst(exchange,) {
                  if (exchange.url === HYPER_CREDITS_URL) {
                    laterAsked.resolve(undefined,);
                    return { status: 401, bodyText: 'no such cat', };
                  }
                  await laterAsked.promise;
                  // A turn of the event loop, after which every continuation the
                  // later provider's refusal needed has run.
                  await wait(0,);
                  return { status: 401, bodyText: 'no such cat', };
                },
                signal: AbortSignal.timeout(HANG_STOP_MS,),
              },);
            },);
            expect(refusal,).toBeInstanceOf(RequiredProviderError,);
            expect((refusal as Error).message,).toContain('synthetic is not ready: meter unavailable',);
          },
        },),
        it({
          name: 'REFUSES A METER THAT CANNOT BE READ as unavailable rather than wet, whatever the transport did '
            + '(ledger T8)',
          fn: async () => {
            /**
             What the gate raised when the Hyper meter refused the key, a status
             the client does not retry.
             */
            const refusal = await gateOutcome(async function gateUnreadable() {
              await assertRequiredProvidersReady({
                required: ['hyper',],
                env: { [KEY_NAMES.hyper]: 'test-hyper', },
                transport: async function transport() {
                  return { status: 401, bodyText: 'no such cat', };
                },
                signal: AbortSignal.timeout(HANG_STOP_MS,),
              },);
            },);
            expect(refusal,).toBeInstanceOf(RequiredProviderError,);
            expect((refusal as Error).message,).toContain('hyper is not ready: meter unavailable',);
          },
        },),
        it({
          name: 'WARNS WHICH METER COULD NOT BE READ, with the provider\'s status and its words and never the key '
            + 'its refusal echoed, when the meter answers over the real transport',
          fn: async ctx => {
            answerEveryFetchWithRefusal({
              sinon: ctx.sinon,
              status: 401,
            },);

            const { result, warned, } = await warnLinesDuring({
              run: async function gateOverRefusingMeter(): Promise<unknown> {
                return gateOutcome(async function gateRefused() {
                  await assertRequiredProvidersReady({
                    required: ['hyper',],
                    env: { [KEY_NAMES.hyper]: ECHOED_KEY, },
                    transport: fetchTransport,
                    signal: AbortSignal.timeout(HANG_STOP_MS,),
                  },);
                },);
              },
            },);

            expect(result,).toBeInstanceOf(RequiredProviderError,);
            expect(warned,).toEqual([
              `[gateProvider] hyper meter could not be read: ${statusFailureLogText({ provider: 'hyper', status: 401, },)}`,
            ],);
          },
        },),
        it({
          name: 'REFUSES WITH THE CALLER\'S OWN REASON AS IT STOPS THE GATE, writing no line for a meter ending in that '
            + 'reason, and the status and the provider\'s words for a meter refused as the stop landed',
          fn: async () => {
            /**
             Why the caller stopped the gate.
             */
            const reason = new Error('the owner called the cats in',);
            /**
             The caller's steering.
             */
            const steering = new AbortController();
            /**
             Meter reads that have begun, by URL, in the order they began.
             */
            const begun: string[] = [];

            const { result, warned, } = await warnLinesDuring({
              run: async function gateAsTheCallerStops(): Promise<unknown> {
                return gateOutcome(async function gateStopped() {
                  await assertRequiredProvidersReady({
                    required: ['synthetic', 'hyper',],
                    env: {
                      [KEY_NAMES.synthetic]: 'test-synthetic',
                      [KEY_NAMES.hyper]: 'test-hyper',
                    },
                    transport: async function transport(exchange,) {
                      begun.push(exchange.url,);
                      // THE STOP LANDS ONCE BOTH METERS ARE ASKED, so it reaches
                      // each read in flight.
                      if (begun.length === 2)
                        steering.abort(reason,);
                      await untilAborted({ signal: exchange.signal, },);
                      if (exchange.url === HYPER_CREDITS_URL)
                        return { status: 401, bodyText: 'this cat has no collar tag', };
                      exchange.signal.throwIfAborted();
                      throw new Error('unreachable: a meter read that waited for its abort was handed a signal that '
                        + 'had not aborted',);
                    },
                    signal: AbortSignal.any([
                      steering.signal,
                      AbortSignal.timeout(HANG_STOP_MS,),
                    ],),
                  },);
                },);
              },
            },);

            expect(warned,).toEqual([
              '[gateProvider] hyper meter could not be read: refused by ProviderHttpError from hyper with HTTP 401 (the '
                + 'provider said: "this cat has no collar tag")',
            ],);
            expect(result,).toBe(reason,);
          },
        },),
        it({
          name: 'READS BEDROCK OFF THE LEDGER THE HANDED ENVIRONMENT NAMES, over no HTTP, wet under its credit and dry '
            + 'at none (ledger T8)',
          fn: async () => {
            /**
             Directory holding this case's empty ledger.
             */
            await using scratch = await scratchDir({ prefix: 'required-providers-ledger-', },);
            const dir = scratch.path;
            /**
             Gate over Bedrock with one credit override.

             @param creditUsd - credit the environment grants

             @returns What the gate did
             */
            async function gateBedrock(creditUsd: string,): Promise<unknown> {
              return await gateOutcome(async function gateLedger() {
                await assertRequiredProvidersReady({
                  required: ['bedrock',],
                  env: {
                    [KEY_NAMES.bedrock]: 'test-bedrock',
                    [BEDROCK_LEDGER_PATH_VAR]: join(dir, 'bedrock-spend.jsonl',),
                    [BEDROCK_CREDIT_USD_VAR]: creditUsd,
                  },
                  transport: unreachedTransport,
                  signal: AbortSignal.timeout(HANG_STOP_MS,),
                  // BEDROCK IS LEFT OFF BY DEFAULT; this case reads its ledger as when it is on.
                  off: [],
                },);
              },);
            }
            expect(await gateBedrock('40',),).toBe('passed',);
            /**
             What the gate raised with no credit to spend.
             */
            const refusal = await gateBedrock('0',);
            expect(refusal,).toBeInstanceOf(RequiredProviderError,);
            expect((refusal as Error).message,).toContain('bedrock is not ready: budget dry',);
          },
        },),
        it({
          name: 'REFUSES BEDROCK WHILE IT IS LEFT OFF, whatever its key and credit say, before any meter is read: the '
            + 'launch would otherwise print status=wet for a provider no run asks',
          fn: async () => {
            /**
             Directory holding this case's empty ledger.
             */
            await using scratch = await scratchDir({ prefix: 'required-providers-bedrock-off-', },);
            /**
             What the gate raised for Bedrock with a key and credit to spare.
             */
            const refusal = await gateOutcome(async function gateOffBedrock() {
              await assertRequiredProvidersReady({
                required: ['bedrock',],
                env: {
                  [KEY_NAMES.bedrock]: 'test-bedrock',
                  [BEDROCK_LEDGER_PATH_VAR]: join(scratch.path, 'bedrock-spend.jsonl',),
                  [BEDROCK_CREDIT_USD_VAR]: '40',
                },
                transport: unreachedTransport,
                signal: AbortSignal.timeout(HANG_STOP_MS,),
              },);
            },);
            expect(refusal,).toBeInstanceOf(RequiredProviderError,);
            expect((refusal as Error).message,).toContain('bedrock is not ready: left off',);
          },
        },),
        it({
          name: 'READS NEURALWATT OFF THE ENERGY LEDGER THE HANDED ENVIRONMENT NAMES, floored by the account\'s two '
            + 'read-only routes, wet under its day\'s allowance and dry at none, and REFUSES A MISSING KEY before '
            + 'reading anything',
          fn: async () => {
            /**
             Directory holding this case's empty ledger.
             */
            await using scratch = await scratchDir({ prefix: 'required-providers-neuralwatt-', },);
            const dir = scratch.path;
            /**
             Gate over NeuralWatt with one allowance and one key.

             @param dailyWh - watt-hours the environment grants a day

             @param key - key the environment holds

             @returns What the gate did
             */
            async function gateNeuralWatt({ dailyWh, key, }: { readonly dailyWh: string; readonly key: string; },): Promise<unknown> {
              return await gateOutcome(async function gateEnergy() {
                await assertRequiredProvidersReady({
                  required: ['neuralwatt',],
                  env: {
                    [KEY_NAMES.neuralwatt]: key,
                    [NEURALWATT_LEDGER_PATH_VAR]: join(dir, 'neuralwatt-energy.jsonl',),
                    [NEURALWATT_DAILY_WH_VAR]: dailyWh,
                  },
                  transport: (key === '') ? unreachedTransport : neuralWattAccountTransport,
                  signal: AbortSignal.timeout(HANG_STOP_MS,),
                },);
              },);
            }
            expect(await gateNeuralWatt({ dailyWh: '550', key: 'test-neuralwatt', },),).toBe('passed',);
            /**
             What the gate raised with no energy to spend.
             */
            const dry = await gateNeuralWatt({ dailyWh: '0', key: 'test-neuralwatt', },);
            expect(dry,).toBeInstanceOf(RequiredProviderError,);
            expect((dry as Error).message,).toContain('neuralwatt is not ready: budget dry',);
            /**
             What the gate raised with no key.
             */
            const keyless = await gateNeuralWatt({ dailyWh: '550', key: '', },);
            expect(keyless,).toBeInstanceOf(RequiredProviderError,);
            expect((keyless as Error).message,).toContain('neuralwatt is not ready: key missing',);
          },
        },),
      ],
    },),
  ],
},);
