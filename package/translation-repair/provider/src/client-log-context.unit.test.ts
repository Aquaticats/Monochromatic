/**
 Guards ledger A11 at the client boundary: a SPEND line written by the run
 client, which every entry shares, names the entry and slice whose call it
 priced, even when the call waited in the provider's per-model queue.

 Cat-themed invention throughout.

 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  inEntryLogContext,
  inSliceLogContext,
} from '@monochromatic-dev/translation-repair-typography';
import { createBedrockClient, } from '../dist/final/node/index.mjs';
import { SEAT_HYPER_TEXT_BEDROCK, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Stream the endpoint answers every call with: one content chunk, usage, done.
 */
const NAP_STREAM = [
  `data: ${JSON.stringify({
    id: 'chatcmpl-nap',
    object: 'chat.completion.chunk',
    created: 1_788_811_674,
    choices: [{
      index: 0,
      delta: { role: 'assistant', content: 'sunbeam', },
      finish_reason: null,
    },],
  },)}\n\n`,
  `data: ${JSON.stringify({
    id: 'chatcmpl-nap',
    object: 'chat.completion.chunk',
    created: 1_788_811_674,
    choices: [],
    usage: {
      prompt_tokens: 90,
      completion_tokens: 10,
      total_tokens: 100,
    },
  },)}\n\n`,
  'data: [DONE]\n\n',
].join('',);

/**
 Slices that call at once.
 */
const SLICES: readonly number[] = [
  1,
  2,
  3,
];

await describe({
  name: 'run client log context',
  // SEQUENTIAL: the case diverts the one global `console.info` across awaits.
  concurrency: 1,
  children: [
    it({
      name: 'NAMES the entry and slice on each SPEND line, calls queued behind one slot included',
      fn: async () => {
        /**
         Client whose one slot per model makes the later calls queue.
         */
        const client = createBedrockClient({
          apiKey: 'test-key',
          ledger: {
            path: '/nowhere/bedrock-spend.jsonl',
            note: async function note(): Promise<void> {
              await wait(0,);
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
          },
          baseUrl: 'https://mantle.invalid',
          perModelConcurrency: 1,
          // No stream bound: the card's own would be armed on the real clock
          // around every attempt (ledger B416).
          streamBoundMsOverride: 'unbounded',
          transport: async function transport() {
            // A timer turn, so the queued calls wait behind a live one.
            await wait(0,);
            return {
              status: 200,
              bodyText: NAP_STREAM,
            };
          },
          retryPolicy: {
            limit: 0,
            baseMs: 1,
          },
        },);

        /**
         Lines `console.info` received.
         */
        const lines: string[] = [];
        /**
         `console.info` as it was, put back once the calls return.
         */
        const informed = console.info;
        console.info = (...parts: readonly unknown[]) => {
          lines.push(parts.map(String,)
            .join(' ',),);
        };
        {
          await using restore = {
            [Symbol.asyncDispose]: async () => {
              console.info = informed;
            },
          };
          await inEntryLogContext({
            entry: 'Tabby',
            generation: 'nap-3',
            attempt: 2,
            run: async () => {
              await Promise.all(SLICES.map(async function callFor(sliceIndex,): Promise<void> {
                await inSliceLogContext({
                  lane: 'repair',
                  sliceIndex,
                  run: async () => {
                    await client.chatText({
                      modelId: SEAT_HYPER_TEXT_BEDROCK,
                      messages: [{ role: 'user', content: `Where does cat ${String(sliceIndex,)} nap?`, },],
                      signal: new AbortController().signal,
                    },);
                  },
                },);
              },),);
            },
          },);
        }

        /**
         SPEND lines, one per call.
         */
        const spent = lines.filter(function isSpend(line,): boolean {
          return line.includes(' SPEND ',);
        },);
        expect(spent.length,).toBe(SLICES.length,);
        expect(SLICES.map(function spendLinesFor(sliceIndex,): number {
          return spent.filter(function namesSlice(line,): boolean {
            return line.includes(`[Tabby] [attempt 2] [repair slice ${String(sliceIndex,)}]`,);
          },).length;
        },),).toStrictEqual([
          1,
          1,
          1,
        ],);
      },
    },),
  ],
},);
