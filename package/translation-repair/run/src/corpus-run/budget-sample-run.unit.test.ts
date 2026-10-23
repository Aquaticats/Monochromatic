/**
 Tests for the one reading of every provider's meter a budget sample takes.

 THE SUMMARY LINE SAYS WHAT ROUTING WOULD DO with each provider, in provider
 order, and the reading itself is left in the log by the budget layer. The
 transport is scripted by URL and the Bedrock ledger lives in a throwaway
 directory, so nothing here reaches a provider, and the keys are invented
 stand-ins.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import type {
  ModelTransport,
  TransportExchange,
  TransportReply,
} from '@monochromatic-dev/translation-repair-provider-stream';
import { compareCodePoints, } from '@monochromatic-dev/translation-repair-markup';
import { sampleBudgets, } from '../../dist/final/node/index.mjs';
import { capturingLoggerPair, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Environment variable the Bedrock ledger's place is read from.
 */
const LEDGER_VARIABLE = 'TRANSLATION_REPAIR_BEDROCK_LEDGER';

/**
 Environment variable that moves the NeuralWatt energy ledger.
 */
const NEURALWATT_LEDGER_VARIABLE = 'TRANSLATION_REPAIR_NEURALWATT_LEDGER';

/**
 Environment variable the Bedrock credit is read from.
 */
const CREDIT_VARIABLE = 'TRANSLATION_REPAIR_BEDROCK_CREDIT_USD';

/**
 Sentence the summary closes with.
 */
const SUMMARY_CLOSING = '. The reading this command logged is the record; read a collection of them with '
  + '`mise run //package/translation-repair/run:meter-report`';

/**
 Transport whose every provider refuses the key, so no meter can be read.

 @param seen - exchanges asked, in order

 @returns Transport answering 401 to every exchange

 @example
 ```ts
 const transport = unreachableTransport({ seen: [], },);
 ```
 */
function unreachableTransport(
  { seen, }: { readonly seen: TransportExchange[]; },
): ModelTransport {
  return function refuseEverything(exchange,): Promise<TransportReply> {
    seen.push(exchange,);
    return Promise.resolve({
      status: 401,
      bodyText: 'the cat flap is shut',
    },);
  };
}

/**
 Environment with the four invented keys and a ledger in the given directory.

 @param ledgerDir - throwaway directory the Bedrock ledger would live in

 @param extra - further variables

 @returns Environment the sample reads

 @example
 ```ts
 const env = envWith({ ledgerDir, extra: {}, },);
 ```
 */
function envWith(
  {
    ledgerDir,
    extra,
  }: {
    readonly ledgerDir: string;
    readonly extra: Readonly<Record<string, string>>;
  },
): Readonly<Record<string, string>> {
  return {
    TRANSLATION_REPAIR_NEURALWATT_API_KEY: 'whisker-key-0005',
    // The day's start, which the unreachable quota cannot give: 15:15 EDT in America/New_York is 19:15 UTC.
    TRANSLATION_REPAIR_NEURALWATT_DAY_START: '19:15',
    [NEURALWATT_LEDGER_VARIABLE]: join(
      ledgerDir,
      'neuralwatt-energy.jsonl',
    ),
    TRANSLATION_REPAIR_SYNTHETIC_API_KEY: 'whisker-key-0001',
    TRANSLATION_REPAIR_CHARM_HYPER_API_KEY: 'whisker-key-0002',
    TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY: 'whisker-key-0003',
    TRANSLATION_REPAIR_OPENROUTER_API_KEY: 'whisker-key-0004',
    [LEDGER_VARIABLE]: join(
      ledgerDir,
      'bedrock-ledger.jsonl',
    ),
    ...extra,
  };
}

await describe({
  name: sampleBudgets.name,
  children: [
    it({
      name: 'ASKS each meter with its own provider\'s key and says routing would use every provider when no meter answers',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'budget-run-', },);
        const seen: TransportExchange[] = [];
        const {
          logger,
          lines,
        } = capturingLoggerPair();

        await sampleBudgets({
          env: envWith({
            ledgerDir: scratch.path,
            extra: {},
          },),
          transport: unreachableTransport({ seen, },),
          l: logger,
        },);

        expect(lines.at(-1,),).toBe(
          `[sampleBudgets] SAMPLED: routing would use neuralwatt, use synthetic, use bedrock, avoid deepseek, avoid xiaomi, avoid anthropic, use hyper, use openrouter${SUMMARY_CLOSING}`,
        );
        // The meters are read together, so the order they are asked in is not the contract.
        expect(seen.map(function seenOf(exchange,): string {
          return `${exchange.method} ${exchange.url.split('?',)[0] ?? ''} ${exchange.headers.Authorization ?? ''}`;
        },).toSorted(function byText(
          left,
          right,
        ): number {
          return compareCodePoints({
            left,
            right,
          },);
        },),).toEqual([
          'GET https://api.neuralwatt.com/v1/models Bearer whisker-key-0005',
          'GET https://api.neuralwatt.com/v1/quota Bearer whisker-key-0005',
          'GET https://api.neuralwatt.com/v1/usage/by-model Bearer whisker-key-0005',
          'GET https://api.synthetic.new/v2/quotas Bearer whisker-key-0001',
          'GET https://hyper.charm.land/v1/credits Bearer whisker-key-0002',
          'GET https://openrouter.ai/api/v1/credits Bearer whisker-key-0004',
        ],);
      },
    },),
    it({
      name: 'SAYS routing would avoid Bedrock when its credit is spent, and use the providers whose meters are unreadable',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'budget-run-', },);
        const {
          logger,
          lines,
        } = capturingLoggerPair();

        await sampleBudgets({
          env: envWith({
            ledgerDir: scratch.path,
            extra: { [CREDIT_VARIABLE]: '0', },
          },),
          transport: unreachableTransport({ seen: [], },),
          l: logger,
        },);

        expect(lines.at(-1,),).toBe(
          `[sampleBudgets] SAMPLED: routing would use neuralwatt, use synthetic, avoid bedrock, avoid deepseek, avoid xiaomi, avoid anthropic, use hyper, use openrouter${SUMMARY_CLOSING}`,
        );
      },
    },),
    it({
      name: 'SAYS routing would avoid NeuralWatt when the environment grants it no energy, and sends it no chat request',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'budget-run-', },);
        const seen: TransportExchange[] = [];
        const {
          logger,
          lines,
        } = capturingLoggerPair();

        await sampleBudgets({
          env: envWith({
            ledgerDir: scratch.path,
            extra: { TRANSLATION_REPAIR_NEURALWATT_DAILY_WH: '0', },
          },),
          transport: unreachableTransport({ seen, },),
          l: logger,
        },);

        expect(lines.at(-1,),).toBe(
          `[sampleBudgets] SAMPLED: routing would avoid neuralwatt, use synthetic, use bedrock, avoid deepseek, avoid xiaomi, avoid anthropic, use hyper, use openrouter${SUMMARY_CLOSING}`,
        );
        // THE METER READS THE ACCOUNT'S TWO ROUTES FOR THE DAY'S START; no energy is spent reading them.
        expect(seen.some(function sentChat(exchange,): boolean {
          return exchange.url.includes('neuralwatt',) && (exchange.method === 'POST');
        },),).toBe(false,);
      },
    },),
    it({
      name: 'SAYS routing would avoid NeuralWatt when its day start can be learned from neither the quota nor the '
        + 'environment, and asks only the quota of it',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'budget-run-', },);
        const seen: TransportExchange[] = [];
        const {
          logger,
          lines,
        } = capturingLoggerPair();

        await sampleBudgets({
          env: Object.fromEntries(Object.entries(envWith({
            ledgerDir: scratch.path,
            extra: {},
          },),).filter(function keeps([name,],): boolean {
            return name !== 'TRANSLATION_REPAIR_NEURALWATT_DAY_START';
          },),),
          transport: unreachableTransport({ seen, },),
          l: logger,
        },);

        expect(lines.at(-1,),).toBe(
          `[sampleBudgets] SAMPLED: routing would avoid neuralwatt, use synthetic, use bedrock, avoid deepseek, avoid xiaomi, avoid anthropic, use hyper, use openrouter${SUMMARY_CLOSING}`,
        );
        expect(seen.filter(function reachedNeuralWatt(exchange,): boolean {
          return exchange.url.includes('neuralwatt',);
        },).map(function urlOf(exchange,): string {
          return exchange.url;
        },),).toEqual(['https://api.neuralwatt.com/v1/quota',],);
      },
    },),
    it({
      name: 'SAMPLES THE TWO DIRECT PROVIDERS when their keys are set: DeepSeek is asked its balance with its own key, '
        + 'and Xiaomi, which documents no balance, is read from the run\'s own spend and sent no request',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'budget-run-', },);
        const seen: TransportExchange[] = [];
        const {
          logger,
          lines,
        } = capturingLoggerPair();

        await sampleBudgets({
          env: envWith({
            ledgerDir: scratch.path,
            extra: {
              DEEPSEEK_API_KEY: 'whisker-key-0006',
              TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY: 'whisker-key-0007',
            },
          },),
          transport: unreachableTransport({ seen, },),
          l: logger,
        },);

        // A meter that cannot be read is spendable, as for every other provider; Xiaomi answers wet.
        expect(lines.at(-1,),).toBe(
          `[sampleBudgets] SAMPLED: routing would use neuralwatt, use synthetic, use bedrock, use deepseek, use xiaomi, avoid anthropic, use hyper, use openrouter${SUMMARY_CLOSING}`,
        );
        expect(seen.filter(function reachedDirect(exchange,): boolean {
          return exchange.url.includes('deepseek',) || exchange.url.includes('xiaomi',);
        },).map(function lineOf(exchange,): string {
          return `${exchange.method} ${exchange.url} ${exchange.headers.Authorization ?? ''}`;
        },),).toEqual(['GET https://api.deepseek.com/user/balance Bearer whisker-key-0006',],);
      },
    },),
    it({
      name: 'REFUSES as stated, before any meter is asked, when a key is not set',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'budget-run-', },);
        const seen: TransportExchange[] = [];
        const { logger, } = capturingLoggerPair();

        const refusal = await rejectionOf(async function sampleWithoutKeys(): Promise<void> {
          await sampleBudgets({
            env: { [LEDGER_VARIABLE]: join(scratch.path, 'ledger.jsonl',), },
            transport: unreachableTransport({ seen, },),
            l: logger,
          },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(seen,).toEqual([],);
      },
    },),
    it({
      name: 'REFUSES as stated, naming the variable and the value, a Bedrock credit override that is no amount',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'budget-run-', },);
        const { logger, } = capturingLoggerPair();

        const refusal = await rejectionOf(async function sampleWithBadCredit(): Promise<void> {
          await sampleBudgets({
            env: envWith({
              ledgerDir: scratch.path,
              extra: { [CREDIT_VARIABLE]: 'plenty', },
            },),
            transport: unreachableTransport({ seen: [], },),
            l: logger,
          },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          'BedrockCreditOverrideError: TRANSLATION_REPAIR_BEDROCK_CREDIT_USD must be a non-negative number of USD; '
            + 'it holds "plenty"',
        );
      },
    },),
  ],
},);
