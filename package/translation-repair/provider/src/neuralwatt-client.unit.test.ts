/**
 Tests for the NeuralWatt client over a recorded transport and a real energy
 ledger in a throwaway directory.

 THE STREAM SHAPE IS THE ONE THE DOCUMENTATION PRINTS
 (`https://docs.neuralwatt.com/energy-methodology.md`, fetched 2026-10-20):
 OpenAI-compatible chunks, a usage chunk, then the `: energy` and `: cost`
 comments, then `data: [DONE]`. No case calls NeuralWatt or reads a key; the
 transport is a script. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  MalformedCompletionError,
  ProviderHttpError,
  type TransportExchange,
  type TransportReply,
} from '@monochromatic-dev/translation-repair-provider-stream';
import {
  COMPLETION_CAP,
  runSpendUsd,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  createNeuralWattClient,
  createNeuralWattLedger,
  estimateNeuralWattWh,
  isBudgetRefusal,
  isPaymentRefusal,
  NEURALWATT_ENERGY_MARGIN,
  NeuralWattAllowanceError,
  NeuralWattModelNotServedError,
  promptTokensOfBody,
  SCHEMA_BLOCK_HEADING,
} from '../dist/final/node/index.mjs';
import {
  consoleLinesDuring,
  handClock,
  jsonRecordOf,
  MIDNIGHT_DAY,
} from './neuralwatt-ledger.test-fixture.ts';
import {
  ask,
  chatExchangesOf,
  streamOf,
} from './neuralwatt-scripted-client.test-fixture.ts';
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

/**
 Host every case points the client at; no case reaches it.
 */
const BASE_URL = 'https://neuralwatt.invalid/v1';

/**
 Abort signal every call here carries.
 */
const SIGNAL = new AbortController().signal;

/**
 The energy comment the page prints: 0.000015 kWh, which is 0.015 Wh.
 */
const ENERGY_COMMENT = ': energy {"energy_kwh": 0.000015, "energy_joules": 54.0, "attribution_ratio": 0.25}\n\n';

/**
 An energy comment small enough to leave room for a second call: 0.0000005 kWh, which is 0.0005 Wh.
 */
const SMALL_ENERGY_COMMENT = ': energy {"energy_kwh": 0.0000005}\n\n';

/**
 The cost comment of a free request.
 */
const FREE_COST_COMMENT = ': cost {"request_cost_usd": 0, "cache_savings_usd": 0.0}\n\n';

/**
 Quota reply with a given charged energy.

 @param kwhUsed - charged kWh this period

 @returns Success reply

 @example
 ```ts
 const reply = quotaReply({ kwhUsed: 13.9, },);
 ```
 */
function quotaReply({ kwhUsed, }: { readonly kwhUsed: number; },): TransportReply {
  return {
    status: 200,
    bodyText: JSON.stringify({
      snapshot_at: '2026-10-20T20:00:00Z',
      subscription: {
        plan: 'max',
        current_period_start: '2026-10-11T05:05:25Z',
        kwh_included: 20,
        kwh_used: kwhUsed,
        kwh_remaining: 20 - kwhUsed,
        in_overage: false,
      },
    },),
  };
}

/**
 Builds a client over a scripted transport and a ledger file in a fresh
 directory.

 @param dir - directory the ledger file lives in

 @param dailyWh - day's allowance

 @param chat - replies to chat completions, in order, the last repeating, or a function answering each

 @param quota - replies to quota reads, in order, the last repeating

 @returns Client, ledger, and what the transport saw

 @example
 ```ts
 const { client, ledger, } = scripted({ dir, },);
 ```
 */
function scripted(
  {
    dir,
    dailyWh = 550,
    chat = [{ status: 200, bodyText: streamOf({ comments: ENERGY_COMMENT + FREE_COST_COMMENT, }), },],
    quota = [quotaReply({ kwhUsed: 13.9023, }),],
    dayStartSetting,
  }: {
    readonly dir: string;
    readonly dailyWh?: number;
    readonly chat?: readonly TransportReply[] | ((exchange: TransportExchange, index: number,) => Promise<TransportReply>);
    readonly quota?: readonly TransportReply[];
    readonly dayStartSetting?: { readonly msOfDay: number; };
  },
) {
  /**
   Every exchange the transport was handed.
   */
  const exchanges: TransportExchange[] = [];

  /**
   Chat exchanges seen so far, for scripting by position.
   */
  const counts = { chat: 0, quota: 0, };

  /**
   Clock the ledger reads.
   */
  const clock = handClock({ start: '2026-10-20T12:00:00.000Z', },);

  /**
   Ledger the client spends against.
   */
  const ledger = createNeuralWattLedger({
    path: join(dir, 'neuralwatt-energy.jsonl',),
    dailyWh,
    now: clock.now,
  },);
  return {
    exchanges,
    ledger,
    clock,
    client: createNeuralWattClient({
      apiKey: 'test-key',
      ledger,
      baseUrl: BASE_URL,
      now: clock.now,
      ...((dayStartSetting === undefined) ? {} : { dayStartSetting, }),
      transport: async function transport(exchange,): Promise<TransportReply> {
        exchanges.push(exchange,);
        // THE BY-MODEL ROUTE IS SCRIPTED IN neuralwatt-day.unit.test.ts: here it is down, so the ledger counts alone.
        if ((exchange.method === 'GET') && exchange.url.includes('/usage/by-model',))
          return { status: 503, bodyText: 'no script', };
        if (exchange.method === 'GET') {
          counts.quota += 1;
          return quota[Math.min(counts.quota - 1, quota.length - 1,)] ?? { status: 500, bodyText: 'no script', };
        }
        counts.chat += 1;
        if ((typeof chat) === 'function')
          return await (chat as (exchange: TransportExchange, index: number,) => Promise<TransportReply>)(
            exchange,
            counts.chat - 1,
          );
        return (chat as readonly TransportReply[])[Math.min(counts.chat - 1, (chat as readonly TransportReply[]).length - 1,)]
          ?? { status: 500, bodyText: 'no script', };
      },
      retryPolicy: { limit: 0, baseMs: 1, },
    },),
  };
}

await describe({
  name: createNeuralWattClient.name,
  concurrency: 1,
  children: [
    it({
      name: 'SENDS THE CHAT COMPLETIONS BODY with the NeuralWatt spelling of each seat, streaming usage on, the '
        + 'schema in the system prompt and the response format, the measured ceiling as max_tokens, the one '
        + 'documented reasoning_effort on the GLM seat and none on the DeepSeek seat, no store or sampling knob, the '
        + 'bearer key in the header',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-client-', },);
        const { client, exchanges, } = scripted({ dir: scratch.path, },);
        /**
         One schema'd call on the GLM seat.
         */
        const reply = await client.chatText({
          modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
          messages: [
            { role: 'system', content: 'You are a careful cat.', },
            { role: 'user', content: 'Where does the cat sleep?', },
          ],
          signal: SIGNAL,
          responseFormat: {
            type: 'json_schema',
            json_schema: {
              name: 'spot',
              strict: true,
              schema: { type: 'object', properties: { spot: { type: 'string', }, }, required: ['spot',], },
            },
          },
        },);
        expect(reply.text,).toBe('{"spot":"sunbeam"}',);

        const [post,] = chatExchangesOf({ exchanges, },);
        expect(post?.url,).toBe(`${BASE_URL}/chat/completions`,);
        expect(post?.method,).toBe('POST',);
        expect(post?.headers.Authorization ?? post?.headers.authorization,).toBe('Bearer test-key',);
        /**
         Body as sent.
         */
        const body = jsonRecordOf({ text: post?.bodyJson ?? '{}', },);
        expect(body.model,).toBe('glm-5.3-flash',);
        expect(body.stream,).toBe(true,);
        expect(body.stream_options,).toEqual({ include_usage: true, },);
        expect(body.max_tokens,).toBe(COMPLETION_CAP[SEAT_HYPER_OPENROUTER_VISION_EDITOR],);
        expect(body.response_format,).toMatchObject({ type: 'json_schema', },);
        // The one parameter this provider may be sent, since the owner's lift of 2026-10-21 (ledger B807).
        expect(body.reasoning_effort,).toBe('low',);
        for (const absent of ['temperature', 'reasoning', 'thinking', 'store', 'provider',])
          expect(Object.hasOwn(body, absent,),).toBe(false,);
        expect(JSON.stringify(body.messages,),).toContain(SCHEMA_BLOCK_HEADING,);
        expect(JSON.stringify(body.messages,),).toContain('spot',);

        const [ask2,] = [await client.chatText({
          modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
          messages: [{ role: 'user', content: 'Again?', },],
          signal: SIGNAL,
        },),];
        expect(ask2.text,).toBe('{"spot":"sunbeam"}',);
        /**
         Body of the second ask, on the DeepSeek seat, whose card names no effort.
         */
        const secondBody = jsonRecordOf({ text: chatExchangesOf({ exchanges, },)[1]?.bodyJson ?? '{}', },);
        expect(secondBody.model,).toBe('deepseek-v4.1-flash',);
        expect(Object.hasOwn(secondBody, 'reasoning_effort',),).toBe(false,);
      },
    },),

    it({
      name: 'REFUSES A MODEL THIS PROVIDER HAS NO SPELLING FOR before any slot or request, as a routing mistake',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-client-', },);
        const { client, exchanges, } = scripted({ dir: scratch.path, },);
        expect(await rejectionOf(async function wrong(): Promise<void> {
          await client.chatText({
            modelId: SEAT_BEDROCK_ONLY_TEXT,
            messages: [{ role: 'user', content: 'Hi', },],
            signal: SIGNAL,
          },);
        },),).toBeInstanceOf(NeuralWattModelNotServedError,);
        expect(exchanges.length,).toBe(0,);
      },
    },),

    it({
      name: 'SETTLES AT THE MEASURED ENERGY and the SPEND line: 0.000015 kWh is written as 0.015 Wh, unmarked, the '
        + 'hold is released, and a free request adds nothing to the run\'s USD meter',
      fn: async ctx => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-client-', },);
        const { client, ledger, } = scripted({ dir: scratch.path, },);
        /**
         USD the run meter held before the call.
         */
        const before = runSpendUsd({ provider: 'neuralwatt', },);
        /**
         Lines the call logged.
         */
        const lines = await consoleLinesDuring({
          run: async function call(): Promise<void> {
            await ask({ client, },);
          },
          sinon: ctx.sinon,
        },);
        expect(lines.some(function isSpend(line,): boolean {
          return line.includes('SPEND provider=neuralwatt model=deepseek-v4.1-flash prompt=90 completion=10 cost=0',);
        },),).toBe(true,);
        expect(runSpendUsd({ provider: 'neuralwatt', },),).toBe(before,);
        expect(await ledger.read({ day: MIDNIGHT_DAY, },),).toMatchObject({ reservedWh: 0, reckonedWh: 0, calls: 1, },);
        expect((await ledger.read({ day: MIDNIGHT_DAY, },)).spentWh,).toBeCloseTo(0.015, 12,);
        expect(jsonRecordOf({ text: (await readFile(ledger.path, 'utf8',)).trim(), },).estimated,).toBeUndefined();
      },
    },),

    it({
      name: 'KEEPS THE ESTIMATE WHEN THE STREAM CARRIED NO ENERGY COMMENT, the bucket mean for the body\'s bytes '
        + 'times the margin of one, and marks the line estimated',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-client-', },);
        const { client, ledger, exchanges, } = scripted({
          dir: scratch.path,
          chat: [{ status: 200, bodyText: streamOf({ comments: FREE_COST_COMMENT, }), },],
        },);
        await ask({ client, },);
        /**
         Prompt tokens the body's bytes stand for.
         */
        const promptTokens = promptTokensOfBody({
          bodyBytes: Buffer.byteLength(chatExchangesOf({ exchanges, },)[0]?.bodyJson ?? '',),
        },);
        /**
         Estimate that prompt carries.
         */
        const { estimateWh, } = estimateNeuralWattWh({ servedId: 'deepseek-v4.1-flash', promptTokens, },);
        /**
         Line the call wrote.
         */
        const line = jsonRecordOf({ text: (await readFile(ledger.path, 'utf8',)).trim(), },);
        expect(line.wh,).toBeCloseTo(estimateWh, 12,);
        expect(line.estimated,).toBe('unreported-estimate',);
        expect(line.promptTokens,).toBe(promptTokens,);
        expect(estimateWh,).toBeCloseTo(
          estimateNeuralWattWh({ servedId: 'deepseek-v4.1-flash', promptTokens, },).bucketMeanWh,
          12,
        );
        expect(NEURALWATT_ENERGY_MARGIN,).toBe(1,);
      },
    },),

    it({
      name: 'A REPORTED COST ABOVE ZERO DOES NOT END THE DAY: the SPEND line still carries the cost, the ledger line '
        + 'is a plain measured one, nothing is logged as billed, and the next call is still sent and settled',
      fn: async ctx => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-client-', },);
        const { client, ledger, exchanges, } = scripted({
          dir: scratch.path,
          chat: [
            {
              status: 200,
              bodyText: streamOf({ comments: `${ENERGY_COMMENT}: cost {"request_cost_usd": 0.000107}\n\n`, }),
            },
            { status: 200, bodyText: streamOf({ comments: ENERGY_COMMENT + FREE_COST_COMMENT, }), },
          ],
        },);
        /**
         Lines the first call logged.
         */
        const lines = await consoleLinesDuring({
          run: async function call(): Promise<void> {
            await ask({ client, },);
          },
          sinon: ctx.sinon,
        },);
        expect(lines.some(function isSpend(line,): boolean {
          return line.includes('SPEND provider=neuralwatt model=deepseek-v4.1-flash',) && line.includes('cost=0.000107',);
        },),).toBe(true,);
        expect(lines.some(function says(line,): boolean {
          return line.toLowerCase()
            .includes('billed',);
        },),).toBe(false,);
        expect('billedUsd' in jsonRecordOf({ text: (await readFile(ledger.path, 'utf8',)).trim(), },),).toBe(false,);
        /**
         Reading of the day after the costed call.
         */
        const energy = await client.energy({ signal: SIGNAL, },);
        expect('billedToday' in energy,).toBe(false,);
        expect(energy.calls,).toBe(1,);

        // The call after the costed one is granted: a second request leaves the transport.
        await ask({ client, },);
        expect(chatExchangesOf({ exchanges, },).length,).toBe(2,);
        expect((await client.energy({ signal: SIGNAL, },)).calls,).toBe(2,);
      },
    },),

    it({
      name: 'REFUSES A CALL WHOSE ESTIMATE DOES NOT FIT, BEFORE ANY REQUEST, with a budget refusal the router moves on '
        + 'from, which is not a payment refusal, naming what was left and what the call needed',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-client-', },);
        const { client, exchanges, } = scripted({ dir: scratch.path, dailyWh: 0.001, },);
        /**
         What the call throws.
         */
        const refusal = await rejectionOf(async function tooBig(): Promise<void> {
          await ask({ client, },);
        },);
        expect(refusal,).toBeInstanceOf(NeuralWattAllowanceError,);
        expect(isBudgetRefusal({ error: refusal, },),).toBe(true,);
        expect(isPaymentRefusal({ error: refusal, },),).toBe(false,);
        expect(String(refusal,),).toContain('Wh',);
        expect(String(refusal,),).toContain('UTC',);
        expect(chatExchangesOf({ exchanges, },).length,).toBe(0,);
      },
    },),


    it({
      name: 'HOLDS A CALL\'S ESTIMATE WHILE IT IS IN FLIGHT: with room for one call and a half, a second sent at once is '
        + 'refused where it would have been sent, and a later call goes once the first has settled',
      fn: async () => {
        // THE ESTIMATE ONE CALL OF THIS PROMPT HOLDS, read off a probe whose stream carries no energy comment.
        await using probeDir = await scratchDir({ prefix: 'whiskers-neuralwatt-probe-', },);
        const probe = scripted({
          dir: probeDir.path,
          chat: [{ status: 200, bodyText: streamOf({ comments: FREE_COST_COMMENT, }), },],
        },);
        await ask({ client: probe.client, },);
        /**
         Watt-hours the probe's one line holds, which is the estimate of that prompt.
         */
        const estimateWh = Number(jsonRecordOf({ text: (await readFile(probe.ledger.path, 'utf8',)).trim(), },).wh,);

        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-client-', },);
        /**
         Opens the first call's stream when the case says so.
         */
        const gate: { open: () => void; } = { open: () => undefined, };
        /**
         Resolves when the first chat request has reached the transport.
         */
        const arrived: { resolve: () => void; } = { resolve: () => undefined, };
        const arrival = new Promise<void>(function wait(resolve,): void {
          arrived.resolve = resolve;
        },);
        const { client, exchanges, } = scripted({
          dir: scratch.path,
          dailyWh: estimateWh * 1.5,
          chat: async function hold(_exchange, index,): Promise<TransportReply> {
            // ONLY THE FIRST CALL WAITS: the third, sent after the gate opened, must not.
            if (index === 0) {
              arrived.resolve();
              await new Promise<void>(function waitFor(resolve,): void {
                gate.open = resolve;
              },);
            }
            return { status: 200, bodyText: streamOf({ comments: SMALL_ENERGY_COMMENT, }), };
          },
        },);
        const first = ask({ client, },);
        await arrival;
        expect((await client.energy({ signal: SIGNAL, },)).reservedWh,).toBeCloseTo(estimateWh, 12,);
        expect(await rejectionOf(async function second(): Promise<void> {
          await ask({ client, },);
        },),).toBeInstanceOf(NeuralWattAllowanceError,);
        expect(chatExchangesOf({ exchanges, },).length,).toBe(1,);

        gate.open();
        await first;
        expect((await client.energy({ signal: SIGNAL, },)).reservedWh,).toBe(0,);
        // The first call settled at 0.0005 Wh, so what was left fits another.
        await ask({ client, },);
        expect(chatExchangesOf({ exchanges, },).length,).toBe(2,);
      },
    },),

    it({
      name: 'NOTES A STREAM THAT NEVER FINISHED AT ITS ESTIMATE and releases the hold when the call fails: the reply '
        + 'stopped before its sentinel is a transport failure that was billed',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-client-', },);
        const { client, ledger, } = scripted({
          dir: scratch.path,
          chat: [{ status: 200, bodyText: streamOf({ done: false, }), },],
        },);
        expect(await rejectionOf(async function cut(): Promise<void> {
          await ask({ client, },);
        },),).toBeInstanceOf(MalformedCompletionError,);
        /**
         Line the abandoned attempt left.
         */
        const line = jsonRecordOf({ text: (await readFile(ledger.path, 'utf8',)).trim(), },);
        expect(line.estimated,).toBe('abandoned-estimate',);
        expect(line.wh,).toBeGreaterThan(0,);
        expect(await ledger.read({ day: MIDNIGHT_DAY, },),).toMatchObject({ reservedWh: 0, calls: 1, },);
      },
    },),

    it({
      name: 'WRITES NOTHING FOR A CALL THAT DELIVERED NOTHING, a dropped connection or a refusal of the request, and '
        + 'releases its hold, with the failure the other providers\' clients raise',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-client-', },);
        const dropped = scripted({
          dir: scratch.path,
          chat: async function drop(): Promise<TransportReply> {
            throw new Error('connection dropped',);
          },
        },);
        await rejectionOf(async function call(): Promise<void> {
          await ask({ client: dropped.client, },);
        },);
        expect(await dropped.ledger.read({ day: MIDNIGHT_DAY, },),).toMatchObject({ reservedWh: 0, calls: 0, spentWh: 0, },);

        await using other = await scratchDir({ prefix: 'whiskers-neuralwatt-client-', },);
        const refused = scripted({
          dir: other.path,
          chat: [{ status: 429, bodyText: '{"error":{"message":"slow down"}}', },],
        },);
        /**
         What the refused call throws.
         */
        const failure = await rejectionOf(async function call(): Promise<void> {
          await ask({ client: refused.client, },);
        },);
        expect(failure,).toBeInstanceOf(ProviderHttpError,);
        expect(failure,).toMatchObject({ status: 429, provider: 'neuralwatt', },);
        expect(await refused.ledger.read({ day: MIDNIGHT_DAY, },),).toMatchObject({ reservedWh: 0, calls: 0, },);
      },
    },),

    it({
      name: 'READS THE QUOTA ONCE BEFORE THE FIRST CALL AND NEVER AGAIN BEFORE THE END: one GET with the bearer key '
        + 'ahead of the first chat request, then the account usage once, and none of either ahead of the second',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-client-', },);
        const { client, exchanges, } = scripted({ dir: scratch.path, },);
        await ask({ client, },);
        await ask({ client, },);
        expect(exchanges.map(function shape(exchange,): string {
          return `${exchange.method} ${exchange.url.split('?',)[0] ?? ''}`;
        },),).toEqual([
          `GET ${BASE_URL}/quota`,
          `GET ${BASE_URL}/models`,
          `GET ${BASE_URL}/usage/by-model`,
          `POST ${BASE_URL}/chat/completions`,
          `POST ${BASE_URL}/chat/completions`,
        ],);
        expect(exchanges[0]?.headers.Authorization ?? exchanges[0]?.headers.authorization,).toBe('Bearer test-key',);
        expect(exchanges[2]?.url,).toBe(
          `${BASE_URL}/usage/by-model?start_date=2026-10-20T05:05:25.000Z&end_date=2026-10-20T12:00:00.000Z`,
        );
      },
    },),

    it({
      name: 'REPORTS THE DIFFERENCE OF THE TWO QUOTA READINGS AT THE END OF A RUN, a second GET after the calls, and '
        + 'reports plan energy spent when the charged figure moved',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-client-', },);
        const { client, exchanges, } = scripted({
          dir: scratch.path,
          quota: [quotaReply({ kwhUsed: 13.9023, }), quotaReply({ kwhUsed: 13.9523, }),],
        },);
        await ask({ client, },);
        /**
         Difference of the two readings.
         */
        const spend = await client.planSpend({ signal: SIGNAL, },);
        expect(spend.kind,).toBe('measured',);
        if (spend.kind === 'measured')
          expect(spend.kwh,).toBeCloseTo(0.05, 9,);
        expect(exchanges.filter(function isQuota(exchange,): boolean {
          return (exchange.method === 'GET') && exchange.url.endsWith('/quota',);
        },).length,).toBe(2,);
      },
    },),

    it({
      name: 'TAKES NO QUOTA READING FOR A RUN THAT MADE NO NEURALWATT CALL, and says there was no starting reading',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-client-', },);
        const { client, exchanges, } = scripted({ dir: scratch.path, },);
        /**
         Report of a run with no call.
         */
        const spend = await client.planSpend({ signal: SIGNAL, },);
        expect(spend,).toMatchObject({ kind: 'not-comparable', },);
        expect(exchanges.length,).toBe(0,);
      },
    },),

    it({
      name: 'GOES ON WITHOUT A STARTING READING WHEN THE QUOTA CANNOT BE READ BUT THE SETTING GIVES THE DAY: the call is '
        + 'sent, the failure is logged, and the end-of-run report says it has no starting reading',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-client-', },);
        const { client, exchanges, } = scripted({
          dir: scratch.path,
          quota: [{ status: 429, bodyText: 'Too Many Requests', },],
          dayStartSetting: { msOfDay: 0, },
        },);
        expect((await ask({ client, },)).text,).toBe('{"spot":"sunbeam"}',);
        expect(chatExchangesOf({ exchanges, },).length,).toBe(1,);
        /**
         Report after the unreadable start.
         */
        const spend = await client.planSpend({ signal: SIGNAL, },);
        expect(spend,).toMatchObject({ kind: 'not-comparable', },);
      },
    },),

    it({
      name: 'READS THE DAY\'S ENERGY OFF THE LEDGER, holds in flight included, as its meter',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-client-', },);
        const { client, } = scripted({ dir: scratch.path, },);
        await ask({ client, },);
        expect(await client.energy({ signal: SIGNAL, },),).toMatchObject({
          dayStart: '2026-10-20T05:05:25.000Z',
          dailyWh: 550,
          reservedWh: 0,
        },);
        expect((await client.energy({ signal: SIGNAL, },)).spentWh,).toBeCloseTo(0.015, 12,);
        expect(await rejectionOf(async function aborted(): Promise<void> {
          await client.energy({ signal: AbortSignal.abort(new Error('stopped',),), },);
        },),).toBeInstanceOf(Error,);
      },
    },),

    it({
      name: 'REPORTS WHAT THE RUN ITSELF DREW as the sum of the lines its own calls wrote, none before a call, and '
        + 'not the energy already in the ledger file from an earlier run of the day',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-client-', },);
        const { client, ledger, clock, } = scripted({ dir: scratch.path, },);
        // AN EARLIER RUN OF THE SAME DAY LEFT 0.4 WH IN THE FILE: ANOTHER LEDGER OBJECT OVER IT.
        const earlier = createNeuralWattLedger({ path: ledger.path, dailyWh: 550, now: clock.now, },);
        const outcome = await earlier.reserve({
          model: 'deepseek-v4.1-flash',
          promptTokens: 10,
          estimateWh: 0.5,
          day: { kind: 'anchored', start: new Date('2026-10-20T05:05:25.000Z',), },
        },);
        if (!outcome.granted)
          throw new Error('the call fits and should have been granted',);
        await outcome.reservation.settle({ measuredWh: 0.4, },);
        expect(client.drawnWh(),).toBe(0,);
        await ask({ client, },);
        await ask({ client, },);
        // TWO CALLS OF 0.015 WH EACH (ENERGY_COMMENT); THE EARLIER RUN'S 0.4 WH IS IN THE DAY'S SPEND, NOT IN THE DRAW.
        expect(client.drawnWh(),).toBeCloseTo(0.03, 12,);
        expect((await client.energy({ signal: SIGNAL, },)).spentWh,).toBeCloseTo(0.43, 12,);
      },
    },),
  ],
},);
