/**
 Tests that every provider client names itself on the exchange it hands the
 transport, so the stream guard that cuts a silent stream can say whose stream
 it cut.

 WHY: in `zheermao22` eleven `inception/mercury-2.5` streams sent comment lines
 and no text and were cut by the straggler grace, and the guard that now cuts
 them knows only the model id it was armed with (ledger B804). The model id does
 not say which provider served it: `deepseek-v4.1-flash` is NeuralWatt's, Hyper's
 and OpenRouter's, and the three differ in what a silent stream means.

 WHAT THESE PIN: the exchange of each of the five clients carries its provider's
 name in `provider`.

 Fixtures are cat-themed invention. No call is sent.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type BedrockLedger,
  createBedrockClient,
  createHyperClient,
  createOpenRouterClient,
  createSyntheticClient,
} from '../dist/final/node/index.mjs';
import type { TransportExchange, } from '@monochromatic-dev/translation-repair-provider-stream';
import {
  chatExchangesOf,
  scriptedClient,
  SIGNAL,
} from './neuralwatt-scripted-client.test-fixture.ts';
import {
  keepingTransport,
  settled,
} from './recording-transport.test-fixture.ts';
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_HYPER_ONLY,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_OPENROUTER_ONLY,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Retry policy that makes one attempt.
 */
const ONE_ATTEMPT = {
  limit: 0,
  baseMs: 1,
};

/**
 Ledger that holds nothing, for the Bedrock client, which never reaches it here.
 */
const EMPTY_LEDGER: BedrockLedger = {
  path: '/nowhere/bedrock-spend.jsonl',
  note: async function note(): Promise<void> {
    await Promise.resolve();
  },
  read: async function read() {
    return {
      creditUsd: 200,
      spentUsd: 0,
      reckonedUsd: 0,
      remainingUsd: 200,
      calls: 0,
    };
  },
};

/**
 Messages of the one call each client is asked for.
 */
const MESSAGES = [{ role: 'user', content: 'Where does the cat nap?', },] as const;

await describe({
  name: 'the provider a client names on its exchange',
  concurrency: 1,
  children: [
    it({
      name: 'NAMES synthetic, hyper, openrouter AND bedrock ON THE CHAT EXCHANGE each hands its transport',
      fn: async () => {
        /**
         Exchanges the transport was handed, in call order.
         */
        const seen: TransportExchange[] = [];
        /**
         Transport shared by the four clients.
         */
        const transport = keepingTransport({ seen, },);
        /**
         Each client with the seat it serves and the name it should give.
         */
        const rows = [
          {
            provider: 'synthetic',
            modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            client: createSyntheticClient({ apiKey: 'whiskers', transport, retryPolicy: ONE_ATTEMPT, },),
          },
          {
            provider: 'hyper',
            modelId: SEAT_HYPER_ONLY,
            client: createHyperClient({ apiKey: 'whiskers', transport, retryPolicy: ONE_ATTEMPT, },),
          },
          {
            provider: 'openrouter',
            modelId: SEAT_OPENROUTER_ONLY,
            client: createOpenRouterClient({ apiKey: 'whiskers', transport, retryPolicy: ONE_ATTEMPT, },),
          },
          {
            provider: 'bedrock',
            modelId: SEAT_BEDROCK_ONLY_TEXT,
            client: createBedrockClient({
              apiKey: 'whiskers',
              ledger: EMPTY_LEDGER,
              transport,
              retryPolicy: ONE_ATTEMPT,
              streamBoundMsOverride: 'unbounded',
            },),
          },
        ] as const;
        for (const row of rows) {
          /**
           Exchanges before this row's call.
           */
          const before = seen.length;
          // oxlint-disable-next-line no-await-in-loop -- the rows share one recorder, read in order
          await settled(row.client.chatText({
            modelId: row.modelId as never,
            messages: MESSAGES,
            signal: new AbortController().signal,
          },),);
          expect(seen.slice(before,).map(function providerOf(exchange,): string {
            return exchange.provider ?? 'unnamed';
          },),).toEqual([row.provider,],);
        }
      },
    },),

    it({
      name: 'NAMES neuralwatt ON THE CHAT EXCHANGE its client hands its transport',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-provider-named-', },);
        const { client, exchanges, } = scriptedClient({ dir: scratch.path, },);
        await client.chatText({
          modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
          messages: MESSAGES,
          signal: SIGNAL,
        },);
        const [post,] = chatExchangesOf({ exchanges, },);
        expect(post?.provider,).toBe('neuralwatt',);
      },
    },),
  ],
},);
