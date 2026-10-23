/**
 Tests for the DeepSeek client over recorded transports.

 NOTHING HERE REACHES A PROVIDER: every transport is an in-memory script, and every key is invented. The stream shapes
 follow what the owner's grants of 2026-10-22 were probed with. Fixtures are cat-themed invention.

 @module
 */

import { setImmediate as nextTurn, } from 'node:timers/promises';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import {
  exchangeFailureLogText,
  ProviderHttpError,
  type TransportExchange,
  type TransportReply,
} from '@monochromatic-dev/translation-repair-provider-stream';
import {
  ROSTER_MODEL_IDS,
  runSpendUsd,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  createDeepSeekClient,
  createProviderBudgets,
  DEEPSEEK_PER_MODEL_CONCURRENCY,
  DeepSeekBalanceShapeError,
  DeepSeekModelNotServedError,
  isPaymentRefusal,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  askCat,
  DEEPSEEK_USAGE_CHUNK,
  errorChainText,
  firstBodyOf,
  NAP_SCHEMA,
  ONE_QUICK_RETRY,
  scripted,
  SIGNAL,
  streamOf,
} from './direct-client.test-fixture.ts';
import { TEST_MAX_HOLD_MS, } from './budget-hold.test-fixture.ts';

/**
 Balance body with a positive USD total.
 */
const POSITIVE_BALANCE_BODY = '{"is_available":true,"balance_infos":[{"currency":"USD","total_balance":"9.96",'
  + '"granted_balance":"0.00","topped_up_balance":"9.96"}]}';

/**
 Milliseconds a budget reading is trusted in the case that reads one.
 */
const FRESH_MS = 1_000;

/**
 Transport that reads a positive balance and refuses every chat call for payment.

 @param exchange - request the client sent

 @returns The balance on a GET, a 402 on anything else

 @example
 ```ts
 const reply = await positiveBalanceRefusingChat(exchange,);
 ```
 */
async function positiveBalanceRefusingChat(exchange: TransportExchange,): Promise<TransportReply> {
  if (exchange.method === 'GET')
    return { status: 200, bodyText: POSITIVE_BALANCE_BODY, };
  return { status: 402, bodyText: '{"error":{"message":"Insufficient Balance"}}', };
}

await describe({
  name: createDeepSeekClient.name,
  // SEQUENTIAL: the run spend meter is process-wide, so cases that read its change must not overlap.
  concurrency: 1,
  children: [
    it({
      name: 'SENDS the wire id deepseek-flash to /chat/completions with the owner\'s bearer key, streaming usage on, '
        + 'the measured cap, JSON mode for a schema, the word json in the system prompt, and NO reasoning member; '
        + 'READS the answer back past comment lines and reasoning_content, and PRICES it at peak with the cache split',
      fn: async () => {
        const { transport, exchanges, } = scripted({
          replies: [{ status: 200, bodyText: streamOf({ usageChunk: DEEPSEEK_USAGE_CHUNK, done: true, }), },],
        },);
        const client = createDeepSeekClient({
          apiKey: 'cat-key-deepseek',
          transport,
          retryPolicy: ONE_QUICK_RETRY,
        },);
        const before = runSpendUsd({ provider: 'deepseek', },);

        const reply = await client.chatText({
          modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
          messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
          signal: SIGNAL,
          responseFormat: NAP_SCHEMA,
        },);

        expect(reply.text,).toBe('{"spot":"sunbeam"}',);
        expect(reply.usage,).toMatchObject({ prompt_tokens: 100, completion_tokens: 10, },);
        const [exchange,] = exchanges;
        expect(exchange?.url,).toBe('https://api.deepseek.com/chat/completions',);
        expect(exchange?.method,).toBe('POST',);
        expect(exchange?.headers.Authorization,).toBe('Bearer cat-key-deepseek',);
        const body = firstBodyOf({ exchanges, },);
        expect(body,).toMatchObject({
          model: 'deepseek-flash',
          stream: true,
          stream_options: { include_usage: true, },
          // THE json_schema TYPE IS REFUSED BY THIS API (HTTP 400, probed 2026-10-22): JSON mode instead.
          response_format: { type: 'json_object', },
        },);
        expect(typeof body.max_tokens,).toBe('number',);
        const wire = JSON.stringify(body,);
        for (const forbidden of ['reasoning', 'thinking', 'effort', '"provider"', '"store"',])
          expect(wire,).not.toContain(forbidden,);
        // The schema rides in the system prompt, whose text carries the word the API's JSON mode requires.
        expect(wire.toLowerCase(),).toContain('json',);
        expect(wire,).toContain('nap_spot',);
        // 40 hits at 0.006, 60 misses at 0.3 and 10 outputs at 1.2 USD per million.
        const priced = ((40 * 0.006) + (60 * 0.3) + (10 * 1.2)) / 1_000_000;
        expect(runSpendUsd({ provider: 'deepseek', },) - before,).toBeCloseTo(priced, 12,);
      },
    },),

    it({
      name: 'FAILS a 402 as a payment refusal after a single request, which is how the router learns this balance is '
        + 'spent',
      fn: async () => {
        const { transport, exchanges, } = scripted({
          replies: [{ status: 402, bodyText: '{"error":{"message":"Insufficient Balance"}}', },],
        },);
        const client = createDeepSeekClient({ apiKey: 'cat-key-deepseek', transport, retryPolicy: ONE_QUICK_RETRY, },);

        const failure: unknown = await rejectionOf(
          () => askCat({ client, modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, },),
        );

        expect(failure,).toBeInstanceOf(ProviderHttpError,);
        expect(isPaymentRefusal({ error: failure, },),).toBe(true,);
        expect(exchanges.length,).toBe(1,);
      },
    },),

    it({
      name: 'HOLDS DeepSeek dry after a 402 while the balance it reads still shows money, since the refusal is the '
        + 'newer evidence, and lets it back in once the balance moves',
      fn: async () => {
        const client = createDeepSeekClient({
          apiKey: 'cat-key-deepseek',
          transport: positiveBalanceRefusingChat,
          retryPolicy: ONE_QUICK_RETRY,
        },);
        /**
         Clock the budgets read.
         */
        const clock = { now: 10_000, };
        const budgets = createProviderBudgets({
          maxHoldMs: TEST_MAX_HOLD_MS,
          deepseek: client,
          freshForMs: FRESH_MS,
          now: () => clock.now,
        },);
        expect((await budgets.read({ signal: SIGNAL, },)).deepseek,).toBe(false,);

        const failure: unknown = await rejectionOf(
          () => askCat({ client, modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, },),
        );
        expect(isPaymentRefusal({ error: failure, },),).toBe(true,);
        await budgets.markRefused({ provider: 'deepseek', signal: SIGNAL, paymentRequired: true, },);

        clock.now += 2 * FRESH_MS;
        expect((await budgets.read({ signal: SIGNAL, },)).deepseek,).toBe(true,);
      },
    },),

    it({
      name: 'THROWS before any request for every roster model but the one the key is granted for (THE LOCK), over the '
        + 'whole roster',
      fn: async () => {
        const { transport, exchanges, } = scripted({ replies: [{ status: 200, bodyText: '', },], },);
        const client = createDeepSeekClient({ apiKey: 'cat-key-deepseek', transport, },);
        /**
         Every roster model but the granted one.
         */
        const others = ROSTER_MODEL_IDS.filter((modelId,) => modelId !== SEAT_HYPER_OPENROUTER_UNMEASURED);

        const failures = await Promise.all(
          others.map((modelId,) => rejectionOf(() => askCat({ client, modelId, },),),),
        );

        expect(ROSTER_MODEL_IDS.length - others.length,).toBe(1,);
        expect(failures.length,).toBe(others.length,);
        for (const failure of failures)
          expect(failure,).toBeInstanceOf(DeepSeekModelNotServedError,);
        expect(exchanges.length,).toBe(0,);
      },
    },),

    it({
      name: 'READS the balance with GET /user/balance and the same key, strict',
      fn: async () => {
        const { transport, exchanges, } = scripted({ replies: [{ status: 200, bodyText: POSITIVE_BALANCE_BODY, },], },);
        const client = createDeepSeekClient({ apiKey: 'cat-key-deepseek', transport, },);

        expect(await client.credits({ signal: SIGNAL, },),).toEqual({ isAvailable: true, totalUsd: 9.96, },);
        expect(exchanges[0]?.url,).toBe('https://api.deepseek.com/user/balance',);
        expect(exchanges[0]?.method,).toBe('GET',);
        expect(exchanges[0]?.headers.Authorization,).toBe('Bearer cat-key-deepseek',);
      },
    },),

    it({
      name: 'REFUSES a balance body of the wrong shape as a stated failure',
      fn: async () => {
        const odd = createDeepSeekClient({
          apiKey: 'cat-key-deepseek',
          transport: scripted({ replies: [{ status: 200, bodyText: '{"is_available":"yes"}', },], },).transport,
        },);
        const failure: unknown = await rejectionOf(async function askForTheFailure(): Promise<unknown> {
          return await odd.credits({ signal: SIGNAL, },);
        },);
        expect(failure,).toBeInstanceOf(DeepSeekBalanceShapeError,);
      },
    },),

    it({
      name: 'FAILS a 401 on the balance read as an HTTP failure after one request, never retrying a refused key',
      fn: async () => {
        const { transport, exchanges, } = scripted({ replies: [{ status: 401, bodyText: 'no', },], },);
        const down = createDeepSeekClient({ apiKey: 'cat-key-deepseek', transport, retryPolicy: ONE_QUICK_RETRY, },);
        const refused: unknown = await rejectionOf(async function askForTheFailure(): Promise<unknown> {
          return await down.credits({ signal: SIGNAL, },);
        },);
        expect(refused,).toBeInstanceOf(ProviderHttpError,);
        expect(exchanges.length,).toBe(1,);
      },
    },),

    it({
      name: 'DROPS the provider\'s words from a 401 on the balance read as well, which may echo a short tail of the '
        + 'key',
      fn: async () => {
        const { transport, } = scripted({
          replies: [{
            status: 401,
            bodyText: '{"error":{"message":"Authentication Fails, Your api key: ****Qx4d is invalid"}}',
          },],
        },);
        const client = createDeepSeekClient({
          apiKey: 'cat-key-whisker-9f3c7a-Qx4d',
          transport,
          retryPolicy: ONE_QUICK_RETRY,
        },);
        const refused: unknown = await rejectionOf(async function askForTheFailure(): Promise<unknown> {
          return await client.credits({ signal: SIGNAL, },);
        },);
        expect(refused,).toBeInstanceOf(ProviderHttpError,);
        expect(exchangeFailureLogText({ error: refused, },),).not.toContain('Qx4d',);
        expect(errorChainText({ error: refused, },),).not.toContain('Qx4d',);
      },
    },),

    it({
      name: 'BOUNDS the calls in flight for the model at the account\'s 2,500, so the 2,501st waits for a place',
      fn: async () => {
        expect(DEEPSEEK_PER_MODEL_CONCURRENCY,).toBe(2_500,);
        /**
         Opens when the case lets the held calls answer.
         */
        const release = Promise.withResolvers<void>();
        /**
         Settles once the transport holds as many calls as the bound allows.
         */
        const full = Promise.withResolvers<void>();
        let inFlight = 0;
        let peak = 0;
        let started = 0;
        const body = streamOf({ usageChunk: DEEPSEEK_USAGE_CHUNK, done: true, },);
        const client = createDeepSeekClient({
          apiKey: 'cat-key-deepseek',
          transport: async function heldTransport(): Promise<TransportReply> {
            started += 1;
            inFlight += 1;
            peak = Math.max(peak, inFlight,);
            if (inFlight === DEEPSEEK_PER_MODEL_CONCURRENCY)
              full.resolve();
            await release.promise;
            inFlight -= 1;
            return { status: 200, bodyText: body, };
          },
        },);

        const calls = Array.from(
          { length: DEEPSEEK_PER_MODEL_CONCURRENCY + 1, },
          () => askCat({ client, modelId: SEAT_HYPER_OPENROUTER_UNMEASURED, },),
        );
        await full.promise;
        // Turns of the event loop, not a clock bound, give the 2,501st call every chance to slip past the bound.
        await nextTurn();
        await nextTurn();
        await nextTurn();
        expect(started,).toBe(DEEPSEEK_PER_MODEL_CONCURRENCY,);
        release.resolve();
        await Promise.all(calls,);
        expect(started,).toBe(DEEPSEEK_PER_MODEL_CONCURRENCY + 1,);
        expect(peak,).toBe(DEEPSEEK_PER_MODEL_CONCURRENCY,);
      },
    },),
  ],
},);
