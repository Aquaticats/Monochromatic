/**
 Tests for the decisions client (2026-09-18): one POST to OpenRouter's
 decisions endpoint, the reply read into the typed contract, spend reported
 on the OpenRouter meter, a non-success status refused, a body off the
 documented shape refused. Recorded transport; cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  resetRunSpend,
  runSpendUsd,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type ModelTransport,
  ProviderHttpError,
} from '@monochromatic-dev/translation-repair-provider-stream';
import {
  type DecisionReply,
  DecisionReplyShapeError,
} from '@monochromatic-dev/translation-repair-provider';
import { createDecisionsClient, } from '../dist/final/node/index.mjs';
import { SEAT_OPENROUTER_DECISIONS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

/**
 Retry pacing that never waits, so a refused status returns at once.
 */
const NO_RETRY = {
  limit: 0,
  baseMs: 0,
};

/**
 What the endpoint answered the cat noticeboard question with.
 */
const REPLY_BODY = JSON.stringify({
  model: 'typesafe/jev-1.13-20260917',
  answers: {
    best: {
      type: 'choice',
      choice: '2',
      probabilities: {
        '0': 0.02,
        '1': 0.08,
        '2': 0.9,
      },
      confidence: 0.86,
    },
    urgent: {
      type: 'noul',
      noul: 0.12,
    },
  },
  usage: {
    input_tokens: 512,
    output_tokens: 7,
    cost: 0.0000215,
  },
  id: 'gen-dec-cat',
  provider: 'TypeSafe',
},);

/**
 Recorded transport answering every exchange the same way and keeping what
 it was sent.

 @param status - HTTP status to answer with

 @param bodyText - body to answer with

 @returns Transport plus the exchanges it saw

 @example
 ```ts
 const rig = recordedTransport({ status: 200, bodyText: REPLY_BODY, },);
 ```
 */
function recordedTransport(
  {
    status,
    bodyText,
  }: {
    readonly status: number;
    readonly bodyText: string;
  },
): {
  readonly transport: ModelTransport;
  readonly sent: { readonly bodies: string[]; readonly urls: string[]; };
} {
  /**
   Exchanges as they arrived.
   */
  const sent = {
    bodies: [] as string[],
    urls: [] as string[],
  };
  return {
    sent,
    transport: async (exchange,) => {
      sent.bodies.push(exchange.bodyJson ?? '',);
      sent.urls.push(exchange.url,);
      return {
        status,
        bodyText,
      };
    },
  };
}

await describe({
  name: createDecisionsClient.name,
  // ONE CASE AT A TIME: the spend case diverts `console.info` and reads the
  // process-wide spend total, which a case running beside it would write to.
  concurrency: 1,
  children: [
    it({
      name: 'POSTS the seat\'s served id, the state and the questions, and reads the answers, model and '
        + 'usage back',
      fn: async () => {
        const rig = recordedTransport({
          status: 200,
          bodyText: REPLY_BODY,
        },);
        const client = createDecisionsClient({
          apiKey: 'test-key',
          transport: rig.transport,
          retryPolicy: NO_RETRY,
        },);
        const reply = await client.decide({
          modelId: SEAT_OPENROUTER_DECISIONS,
          state: { notice: '走失猫咪 Mittens', },
          questions: {
            best: {
              type: 'choice',
              instructions: 'Which candidate is best?',
              criteria: {
                '0': 'none',
                '1': 'candidate 1',
                '2': 'candidate 2',
              },
            },
            urgent: {
              type: 'noul',
              instructions: 'Is the notice urgent?',
            },
          },
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          exchangeTimeoutMs: HANG_STOP_MS,
        },);
        expect(rig.sent.urls,).toEqual(['https://openrouter.ai/api/alpha/decisions',],);
        /**
         Body as the endpoint would have parsed it.
         */
        const body: unknown = JSON.parse(rig.sent.bodies[0] ?? '{}',);
        expect(body,).toMatchObject({
          model: 'typesafe/jev-1.13',
          state: { notice: '走失猫咪 Mittens', },
        },);
        expect(reply.model,).toBe('typesafe/jev-1.13-20260917',);
        expect(reply.answers.best,).toMatchObject({
          type: 'choice',
          choice: '2',
        },);
        expect(reply.answers.urgent,).toEqual({
          type: 'noul',
          noul: 0.12,
        },);
        expect(reply.usage,).toEqual({
          prompt_tokens: 512,
          completion_tokens: 7,
          total_tokens: 519,
        },);
        expect(reply.costUsd,).toBe(0.0000215,);
      },
    },),

    it({
      name: 'REPORTS A SPEND LINE of unreported counts and no cost, and hands back no usage and no cost, when the '
        + 'endpoint sent neither',
      fn: async () => {
        resetRunSpend();
        const client = createDecisionsClient({
          apiKey: 'test-key',
          transport: recordedTransport({
            status: 200,
            bodyText: JSON.stringify({
              model: 'typesafe/jev-1.13-20260917',
              answers: { urgent: { type: 'noul', noul: 0.12, }, },
            },),
          },).transport,
          retryPolicy: NO_RETRY,
        },);
        /**
         Lines `console.info` received while the call ran.
         */
        const informed: string[] = [];
        /**
         `console.info` as it was, put back once the call returns.
         */
        const original = console.info;
        console.info = (...parts: readonly unknown[]) => {
          informed.push(parts.map(String,)
            .join(' ',),);
        };
        /**
         Decision asked while the console is diverted, put back as the call ends.

         @returns Reply the client read
         */
        async function decideInformed(): Promise<DecisionReply> {
          await using restore = {
            [Symbol.asyncDispose]: async () => {
              console.info = original;
            },
          };
          return await client.decide({
            modelId: SEAT_OPENROUTER_DECISIONS,
            state: 'a cat',
            questions: { urgent: { type: 'noul', instructions: 'Is the notice urgent?', }, },
            signal: AbortSignal.timeout(HANG_STOP_MS,),
          },);
        }
        /**
         Reply the call returned.
         */
        const reply = await decideInformed();
        expect(reply.usage,).toBe(undefined,);
        expect(reply.costUsd,).toBe(undefined,);
        expect(runSpendUsd({ provider: 'openrouter', },),).toBe(0,);
        expect(informed.filter(function isSpend(line,): boolean {
          return line.includes(' SPEND ',);
        },).map(function fromMarker(line,): string {
          return line.slice(line.indexOf('SPEND ',),);
        },),).toEqual([
          'SPEND provider=openrouter model=typesafe/jev-1.13 prompt=unreported completion=unreported',
        ],);
      },
    },),

    it({
      name: 'NAMES OPENROUTER AS THE PROVIDER of the HTTP error a refused status raises, since the decisions '
        + 'endpoint is OpenRouter\'s',
      fn: async () => {
        /**
         Client whose endpoint answers a payment refusal.
         */
        const refused = createDecisionsClient({
          apiKey: 'test-key',
          transport: recordedTransport({
            status: 402,
            bodyText: '{"error":{"code":402,"message":"the bookshop is closed"}}',
          },).transport,
          retryPolicy: NO_RETRY,
        },);
        /**
         What the refused decision raised.
         */
        let caught: unknown;
        try {
          await refused.decide({
            modelId: SEAT_OPENROUTER_DECISIONS,
            state: 'a cat',
            questions: { asleep: { type: 'noul', instructions: 'Asleep?', }, },
            signal: AbortSignal.timeout(HANG_STOP_MS,),
          },);
        } catch (error) {
          caught = error;
        }
        expect(caught instanceof ProviderHttpError,).toBe(true,);
        expect(
          caught instanceof ProviderHttpError
            ? caught.provider
            : undefined,
        ).toBe('openrouter',);
      },
    },),

    it({
      name: 'REFUSES a non-success status as an HTTP error and a body off the documented shape as a '
        + 'shape error, never as an answer',
      fn: async () => {
        const refused = createDecisionsClient({
          apiKey: 'test-key',
          transport: recordedTransport({
            status: 402,
            bodyText: '{"error":{"code":402,"message":"the bookshop is closed"}}',
          },).transport,
          retryPolicy: NO_RETRY,
        },);
        await expect(refused.decide({
          modelId: SEAT_OPENROUTER_DECISIONS,
          state: 'a cat',
          questions: { asleep: { type: 'noul', instructions: 'Asleep?', }, },
          signal: AbortSignal.timeout(HANG_STOP_MS,),
        },),).rejects.toBeInstanceOf(ProviderHttpError,);

        const malformed = createDecisionsClient({
          apiKey: 'test-key',
          transport: recordedTransport({
            status: 200,
            bodyText: '{"model":"typesafe/jev-1.13","answers":{"asleep":{"type":"noul"}}}',
          },).transport,
          retryPolicy: NO_RETRY,
        },);
        await expect(malformed.decide({
          modelId: SEAT_OPENROUTER_DECISIONS,
          state: 'a cat',
          questions: { asleep: { type: 'noul', instructions: 'Asleep?', }, },
          signal: AbortSignal.timeout(HANG_STOP_MS,),
        },),).rejects.toBeInstanceOf(DecisionReplyShapeError,);
      },
    },),

    it({
      name: 'REFUSES an answers field that is an array rather than reading it as spurious '
        + 'index-keyed answers named "0", "1" and so on, which an array element shaped like a '
        + 'choice answer is read as today with no refusal at all (ledger B92)',
      fn: async () => {
        const arrayed = createDecisionsClient({
          apiKey: 'test-key',
          transport: recordedTransport({
            status: 200,
            bodyText: JSON.stringify({
              model: 'typesafe/jev-1.13',
              answers: [{ type: 'choice', choice: '2', },],
            },),
          },).transport,
          retryPolicy: NO_RETRY,
        },);
        await expect(arrayed.decide({
          modelId: SEAT_OPENROUTER_DECISIONS,
          state: 'a cat',
          questions: { best: { type: 'choice', instructions: 'Which cat?', criteria: { '2': 'Mittens', }, }, },
          signal: AbortSignal.timeout(HANG_STOP_MS,),
        },),).rejects.toBeInstanceOf(DecisionReplyShapeError,);
      },
    },),
  ],
},);
