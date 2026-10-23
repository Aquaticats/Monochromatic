/**
 Tests the run client with NeuralWatt configured: the key that switches the
 provider on, the first place it takes in the order, the day's allowance and
 ledger the environment names, and the start guard's view of it.

 The owner, 2026-10-20: "Use the neuralwatt free energy front and center."

 Every case hands over its own environment and a scripted transport, and its
 own ledger in a throwaway directory, so none reads a key the suite inherits,
 reaches NeuralWatt, or opens the real energy ledger.

 @module
 */

import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import {
  EveryProviderDryError,
  NEURALWATT_DAILY_WH_VAR,
  NEURALWATT_LEDGER_PATH_VAR,
} from '@monochromatic-dev/translation-repair-provider';
import type { TransportExchange, } from '@monochromatic-dev/translation-repair-provider-stream';
import {
  RUN_WRITING_PROVIDERS,
  runClientFrom,
  RunConfigError,
  runEndReports,
} from '../../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Name of the NeuralWatt key variable, named only.
 */
const NEURALWATT_KEY_VAR = 'TRANSLATION_REPAIR_NEURALWATT_API_KEY';

/**
 Stream NeuralWatt answers a chat call with: one content chunk, usage, the
 two documented comment lines, done.
 */
const NEURALWATT_STREAM = [
  `data: ${JSON.stringify({
    id: 'chatcmpl-nap',
    choices: [{ index: 0, delta: { role: 'assistant', content: 'sunbeam', }, finish_reason: null, },],
  },)}\n\n`,
  `data: ${JSON.stringify({ id: 'chatcmpl-nap', choices: [], usage: { prompt_tokens: 90, completion_tokens: 10, total_tokens: 100, }, },)}\n\n`,
  ': energy {"energy_kwh": 0.0000005}\n\n',
  ': cost {"request_cost_usd": 0}\n\n',
  'data: [DONE]\n\n',
].join('',);

/**
 Quota reply of an account with a plan.
 */
const QUOTA_BODY = JSON.stringify({
  snapshot_at: '2026-10-20T20:00:00Z',
  subscription: { plan: 'max', current_period_start: '2026-10-11T05:05:25Z', kwh_used: 13.9, kwh_remaining: 6.1, in_overage: false, },
},);

/**
 By-model usage reply for the window a request asked, with no rows: the account has used no covered energy, and the
 period echoes the window as the route does when it honours it.

 @param exchange - the by-model request

 @returns The reply

 @example
 ```ts
 const reply = usageBodyFor({ exchange, },);
 ```
 */
function usageBodyFor({ exchange, }: { readonly exchange: TransportExchange; },): string {
  /**
   Query the request carried.
   */
  const { searchParams, } = new URL(exchange.url,);
  return JSON.stringify({
    period: {
      start: (searchParams.get('start_date',) ?? '').replace('Z', '+00:00',),
      end: (searchParams.get('end_date',) ?? '').replace('Z', '000+00:00',),
    },
    models: [],
    products: [],
  },);
}

/**
 Whether an exchange is the by-model usage read.

 @param exchange - the exchange

 @returns Whether it is
 */
function isUsageRead(exchange: TransportExchange,): boolean {
  return exchange.url.includes('/usage/by-model',);
}

/**
 The models listing the account answers: the two covered models and no alias, which is all these cases need.
 */
const MODELS_BODY = JSON.stringify({
  object: 'list',
  data: [
    { id: 'deepseek-v4.1-flash', metadata: { huggingface_id: 'deepseek-ai/DeepSeek-V4.1-Flash-1017', }, },
    { id: 'glm-5.3-flash', metadata: { huggingface_id: 'zai-org/GLM-5.3-Flash', }, },
  ],
},);

/**
 Whether an exchange is the models listing.

 @param exchange - the exchange

 @returns Whether it is
 */
function isModelsRead(exchange: TransportExchange,): boolean {
  return exchange.url.endsWith('/models',);
}

/**
 Transport answering NeuralWatt's routes, recording what it was asked.

 @param exchanges - where each exchange is recorded

 @returns Transport for a run client

 @example
 ```ts
 const transport = scriptedNeuralWatt({ exchanges: [], },);
 ```
 */
function scriptedNeuralWatt(
  { exchanges, }: { readonly exchanges: TransportExchange[]; },
) {
  return async function answer(exchange: TransportExchange,) {
    exchanges.push(exchange,);
    if (isModelsRead(exchange,))
      return { status: 200, bodyText: MODELS_BODY, };
    if (isUsageRead(exchange,))
      return { status: 200, bodyText: usageBodyFor({ exchange, },), };
    return (exchange.method === 'GET')
      ? { status: 200, bodyText: QUOTA_BODY, }
      : { status: 200, bodyText: NEURALWATT_STREAM, };
  };
}

await describe({
  name: 'run client with NeuralWatt',
  concurrency: 1,
  children: [
    it({
      name: 'BUILDS ON THE NEURALWATT KEY ALONE, and sends a DeepSeek V4.1 Flash call to it first',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'run-client-neuralwatt-', },);
        const exchanges: TransportExchange[] = [];
        const client = runClientFrom({
          env: {
            [NEURALWATT_KEY_VAR]: 'tabby-not-a-real-key',
            [NEURALWATT_LEDGER_PATH_VAR]: join(scratch.path, 'neuralwatt-energy.jsonl',),
          },
          transport: scriptedNeuralWatt({ exchanges, },),
        },);
        /**
         Reply the routed call settled on.
         */
        const reply = await client.chatText({
          modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
          messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
          signal: new AbortController().signal,
        },);
        expect(reply.text,).toBe('sunbeam',);
        // THE ROSTER'S WRITING PROVIDERS ARE ON THE CLIENT, so the router and the seat reader count a wet provider by
        // one record.
        expect(client.writingProviders,).toEqual(RUN_WRITING_PROVIDERS,);
        expect(exchanges.map(function urlOf(exchange,): string {
          return `${exchange.method} ${exchange.url.split('?',)[0] ?? ''}`;
        },),).toEqual([
          'GET https://api.neuralwatt.com/v1/quota',
          'GET https://api.neuralwatt.com/v1/models',
          'GET https://api.neuralwatt.com/v1/usage/by-model',
          'POST https://api.neuralwatt.com/v1/chat/completions',
        ],);
        expect(await client.providerDryness({ signal: new AbortController().signal, },),).toEqual({
          neuralwatt: false,
          synthetic: true,
          bedrock: true,
          // THE TWO DIRECT PROVIDERS HAVE NO KEY HERE, so they read dry like the other absent ones.
          deepseek: true,
          xiaomi: true,
          hyper: true,
          openrouter: true,
        },);
      },
    },),

    it({
      name: 'READS NEURALWATT DRY WHEN THE ENVIRONMENT GRANTS NO ENERGY, as on a day with no promo, and ends the call '
        + 'for a model nobody else serves without a request',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'run-client-neuralwatt-', },);
        const exchanges: TransportExchange[] = [];
        const client = runClientFrom({
          env: {
            [NEURALWATT_KEY_VAR]: 'tabby-not-a-real-key',
            [NEURALWATT_LEDGER_PATH_VAR]: join(scratch.path, 'neuralwatt-energy.jsonl',),
            [NEURALWATT_DAILY_WH_VAR]: '0',
          },
          transport: scriptedNeuralWatt({ exchanges, },),
        },);
        expect((await client.providerDryness({ signal: new AbortController().signal, },)).neuralwatt,).toBe(true,);
        /**
         What the call ended with.
         */
        const refusal = await rejectionOf(async function ask(): Promise<void> {
          await client.chatText({
            modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
            messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
            signal: new AbortController().signal,
          },);
        },);
        expect(refusal,).toBeInstanceOf(EveryProviderDryError,);
        // THE METER NEEDS THE DAY'S START, so it reads the account's two routes (no energy is spent reading them),
        // and no chat request goes out.
        expect(exchanges.filter(function isChat(exchange,): boolean {
          return exchange.method === 'POST';
        },).length,).toBe(0,);
      },
    },),

    it({
      name: 'NEVER ROUTES A MODEL NEURALWATT DOES NOT SERVE TO IT: the same key refuses a Synthetic-only seat as '
        + 'unreachable',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'run-client-neuralwatt-', },);
        const exchanges: TransportExchange[] = [];
        const client = runClientFrom({
          env: {
            [NEURALWATT_KEY_VAR]: 'tabby-not-a-real-key',
            [NEURALWATT_LEDGER_PATH_VAR]: join(scratch.path, 'neuralwatt-energy.jsonl',),
          },
          transport: scriptedNeuralWatt({ exchanges, },),
        },);
        expect(await rejectionOf(async function ask(): Promise<void> {
          await client.chatText({
            modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
            messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
            signal: new AbortController().signal,
          },);
        },),).toBeInstanceOf(NoProviderForModelError,);
        expect(exchanges.length,).toBe(0,);
      },
    },),

    it({
      name: 'REGISTERS THE END-OF-RUN QUOTA REPORT when the key is set, and says the run made no NeuralWatt call '
        + 'when none was made, reading no quota',
      fn: async () => {
        await runEndReports();
        await using scratch = await scratchDir({ prefix: 'run-client-neuralwatt-', },);
        const exchanges: TransportExchange[] = [];
        runClientFrom({
          env: {
            [NEURALWATT_KEY_VAR]: 'tabby-not-a-real-key',
            [NEURALWATT_LEDGER_PATH_VAR]: join(scratch.path, 'neuralwatt-energy.jsonl',),
          },
          transport: scriptedNeuralWatt({ exchanges, },),
        },);
        /**
         The lines the run's end prints.
         */
        const lines = await runEndReports();
        expect(lines.length,).toBe(1,);
        expect(lines[0],).toContain('NeuralWatt',);
        expect(lines[0],).toContain('no NeuralWatt call',);
        expect(exchanges.length,).toBe(0,);
      },
    },),

    it({
      name: 'REPORTS THE PLAN ENERGY SPENT BETWEEN THE FIRST NEURALWATT CALL AND THE RUN\'S END: one quota read before '
        + 'the call and one at the end, their difference printed, and nothing read by a run that made no call',
      fn: async () => {
        await runEndReports();
        await using scratch = await scratchDir({ prefix: 'run-client-neuralwatt-', },);
        /**
         Quota replies in the order the account would give them: before the call, then after it.
         */
        const quotas = [
          QUOTA_BODY,
          QUOTA_BODY.replace('"kwh_used":13.9', '"kwh_used":13.9005',),
        ];
        /**
         Quota reads answered so far.
         */
        const reads = { count: 0, };
        const client = runClientFrom({
          env: {
            [NEURALWATT_KEY_VAR]: 'tabby-not-a-real-key',
            [NEURALWATT_LEDGER_PATH_VAR]: join(scratch.path, 'neuralwatt-energy.jsonl',),
          },
          transport: async function answer(exchange: TransportExchange,) {
            if (exchange.method !== 'GET')
              return { status: 200, bodyText: NEURALWATT_STREAM, };
            if (isModelsRead(exchange,))
              return { status: 200, bodyText: MODELS_BODY, };
            if (isUsageRead(exchange,))
              return { status: 200, bodyText: usageBodyFor({ exchange, },), };
            const bodyText = quotas[Math.min(reads.count, quotas.length - 1,)] ?? QUOTA_BODY;
            reads.count += 1;
            return { status: 200, bodyText, };
          },
        },);
        await client.chatText({
          modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
          messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
          signal: new AbortController().signal,
        },);
        expect(reads.count,).toBe(1,);
        /**
         The lines the run's end prints.
         */
        const lines = await runEndReports();
        expect(reads.count,).toBe(2,);
        expect(lines.length,).toBe(1,);
        expect(lines[0],).toContain('PLAN ENERGY SPENT',);
        expect(lines[0],).toContain('0.5 Wh',);
      },
    },),

    it({
      name: 'NAMES THE NEURALWATT KEY AMONG THE VARIABLES A RUN CANNOT START WITHOUT, when no key at all is set',
      fn: async () => {
        /**
         What building with no key raised.
         */
        const refusal = await rejectionOf(async function build(): Promise<void> {
          runClientFrom({
            env: {},
            transport: async function unasked() {
              throw new Error('a case that only builds a client asked a provider',);
            },
          },);
        },);
        expect(refusal,).toBeInstanceOf(RunConfigError,);
        expect(String(refusal,),).toContain(NEURALWATT_KEY_VAR,);
      },
    },),
  ],
},);
