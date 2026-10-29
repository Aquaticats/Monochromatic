/**
 Tests for the Xiaomi client over recorded transports.

 NOTHING HERE REACHES A PROVIDER: every transport is an in-memory script, and every key is invented. The stream shapes
 follow what the owner's grants of 2026-10-22 were probed with. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { ProviderHttpError, } from '@monochromatic-dev/translation-repair-provider-stream';
import {
  noteRunSpend,
  resetRunSpend,
  ROSTER_MODEL_IDS,
  runSpendUsd,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  createXiaomiClient,
  isPaymentRefusal,
  XiaomiModelNotServedError,
} from '../dist/final/node/index.mjs';
import {
  SEAT_OPENROUTER_ONLY_CHECKER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  askCat,
  firstBodyOf,
  NAP_SCHEMA,
  ONE_QUICK_RETRY,
  scripted,
  SIGNAL,
  streamOf,
  XIAOMI_USAGE_CHUNK,
} from './direct-client.test-fixture.ts';

await describe({
  name: createXiaomiClient.name,
  // SEQUENTIAL: the run spend meter is process-wide, so cases that read its change must not overlap.
  concurrency: 1,
  children: [
    it({
      name: 'SENDS the wire id mimo-v2.6-flash to /v1/chat/completions with the schema unchanged and NO reasoning '
        + 'member, READS the answer, and PRICES it with the cached count',
      fn: async () => {
        const { transport, exchanges, } = scripted({
          replies: [{ status: 200, bodyText: streamOf({ usageChunk: XIAOMI_USAGE_CHUNK, done: true, }), },],
        },);
        const client = createXiaomiClient({ apiKey: 'cat-key-xiaomi', transport, retryPolicy: ONE_QUICK_RETRY, },);
        const before = runSpendUsd({ provider: 'xiaomi', },);

        const reply = await client.chatText({
          modelId: SEAT_OPENROUTER_ONLY_CHECKER,
          messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
          signal: SIGNAL,
          responseFormat: NAP_SCHEMA,
        },);

        expect(reply.text,).toBe('{"spot":"sunbeam"}',);
        expect(exchanges[0]?.url,).toBe('https://api.xiaomimimo.com/v1/chat/completions',);
        expect(exchanges[0]?.headers.Authorization,).toBe('Bearer cat-key-xiaomi',);
        const body = firstBodyOf({ exchanges, },);
        expect(body,).toMatchObject({
          model: 'mimo-v2.6-flash',
          stream: true,
          stream_options: { include_usage: true, },
          response_format: NAP_SCHEMA,
        },);
        const wire = JSON.stringify(body,);
        for (const forbidden of ['reasoning', 'thinking', 'effort', '"provider"', '"store"',])
          expect(wire,).not.toContain(forbidden,);
        // 50 cached at 0.0028, 50 uncached at 0.14 and 10 outputs at 0.28 USD per million.
        const priced = ((50 * 0.0028) + (50 * 0.14) + (10 * 0.28)) / 1_000_000;
        expect(runSpendUsd({ provider: 'xiaomi', },) - before,).toBeCloseTo(priced, 12,);
      },
    },),

    it({
      name: 'FAILS a 402 as a payment refusal after a single request, which is how the router learns the account is '
        + 'out '
        + 'of money',
      fn: async () => {
        const { transport, exchanges, } = scripted({
          replies: [{ status: 402, bodyText: '{"error":"Insufficient account balance"}', },],
        },);
        const client = createXiaomiClient({ apiKey: 'cat-key-xiaomi', transport, retryPolicy: ONE_QUICK_RETRY, },);

        const refusal: unknown = await rejectionOf(() => askCat({ client, modelId: SEAT_OPENROUTER_ONLY_CHECKER, },),);

        expect(refusal,).toBeInstanceOf(ProviderHttpError,);
        expect(isPaymentRefusal({ error: refusal, },),).toBe(true,);
        expect(exchanges.length,).toBe(1,);
      },
    },),

    it({
      name: 'THROWS before any request for every roster model but the one the key is granted for (THE LOCK), over the '
        + 'whole roster',
      fn: async () => {
        const { transport, exchanges, } = scripted({ replies: [{ status: 200, bodyText: '', },], },);
        const client = createXiaomiClient({ apiKey: 'cat-key-xiaomi', transport, },);
        /**
         Every roster model but the granted one.
         */
        const others = ROSTER_MODEL_IDS.filter((modelId,) => modelId !== SEAT_OPENROUTER_ONLY_CHECKER);

        const failures = await Promise.all(
          others.map((modelId,) => rejectionOf(() => askCat({ client, modelId, },),),),
        );

        expect(ROSTER_MODEL_IDS.length - others.length,).toBe(1,);
        expect(failures.length,).toBe(others.length,);
        for (const failure of failures)
          expect(failure,).toBeInstanceOf(XiaomiModelNotServedError,);
        expect(exchanges.length,).toBe(0,);
      },
    },),

    it({
      name: 'PACES every attempt to its request window: a full window makes the next call wait, and the wait is '
        + 'readable',
      fn: async () => {
        const { transport, } = scripted({
          replies: [{ status: 200, bodyText: streamOf({ usageChunk: XIAOMI_USAGE_CHUNK, done: true, }), },],
        },);
        const client = createXiaomiClient({ apiKey: 'cat-key-xiaomi', transport, requestsPerMinute: 2, },);
        expect(client.pace.waitMs(),).toBe(0,);
        await Promise.all([
          askCat({ client, modelId: SEAT_OPENROUTER_ONLY_CHECKER, },),
          askCat({ client, modelId: SEAT_OPENROUTER_ONLY_CHECKER, },),
        ],);
        expect(client.pace.waitMs(),).toBeGreaterThan(0,);
      },
    },),

    it({
      name: 'TAKES A SECOND TURN FROM THE WINDOW for a retried attempt: one call retried once fills a window of two, '
        + 'while the same call over a window of three leaves a place free',
      fn: async () => {
        /**
         Replies for one call that is retried: a stream cut before its terminator, then a whole one.
         */
        const retried = [
          { status: 200, bodyText: streamOf({ usageChunk: XIAOMI_USAGE_CHUNK, done: false, }), },
          { status: 200, bodyText: streamOf({ usageChunk: XIAOMI_USAGE_CHUNK, done: true, }), },
        ];
        const tight = scripted({ replies: retried, },);
        const tightClient = createXiaomiClient({
          apiKey: 'cat-key-xiaomi',
          transport: tight.transport,
          retryPolicy: ONE_QUICK_RETRY,
          requestsPerMinute: 2,
        },);
        await askCat({ client: tightClient, modelId: SEAT_OPENROUTER_ONLY_CHECKER, },);
        expect(tight.exchanges.length,).toBe(2,);
        expect(tightClient.pace.waitMs(),).toBeGreaterThan(0,);

        const roomy = scripted({ replies: retried, },);
        const roomyClient = createXiaomiClient({
          apiKey: 'cat-key-xiaomi',
          transport: roomy.transport,
          retryPolicy: ONE_QUICK_RETRY,
          requestsPerMinute: 3,
        },);
        await askCat({ client: roomyClient, modelId: SEAT_OPENROUTER_ONLY_CHECKER, },);
        expect(roomy.exchanges.length,).toBe(2,);
        expect(roomyClient.pace.waitMs(),).toBe(0,);
      },
    },),

    it({
      name: 'READS its spend off the run meter, with no request to a billing endpoint Xiaomi does not document',
      fn: async () => {
        const { transport, exchanges, } = scripted({ replies: [{ status: 200, bodyText: '', },], },);
        const client = createXiaomiClient({ apiKey: 'cat-key-xiaomi', transport, },);
        resetRunSpend();
        expect((await client.spend({ signal: SIGNAL, },)).spentUsd,).toBe(0,);
        noteRunSpend({ provider: 'xiaomi', costUsd: 0.0042, },);
        expect((await client.spend({ signal: SIGNAL, },)).spentUsd,).toBeCloseTo(0.0042, 12,);
        resetRunSpend();
        expect(exchanges.length,).toBe(0,);
      },
    },),
  ],
},);
