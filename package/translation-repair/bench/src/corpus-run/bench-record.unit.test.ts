/**
 Tests for the client wrapper that prices a bench run.

 Every number the roster-width comparison rests on is read off these rows, and
 a wrapper that mispriced an exchange would print a confident table nobody
 could tell was wrong. The split between what a call SENDS and what it gets
 BACK is the part under test: seating one more producer resends the same
 prompt, so the two halves answer different questions about width.

 Fixtures are invented. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider';
import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import {
  type BenchCall,
  recordingClient,
} from '../../dist/final/node/index.mjs';
import { SEAT_SYNTHETIC_VISION_NO_OPENROUTER, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/rejection-of.test-fixture.ts';

/**
 Model standing in for whichever one a stage seated.
 */
const CAT_MODEL = SEAT_SYNTHETIC_VISION_NO_OPENROUTER;

/**
 Signal no fixture here ever aborts.
 */
const OPEN_SIGNAL = new AbortController().signal;

/**
 Free-text request the wrapper forwards.
 */
const TEXT_REQUEST = {
  modelId: CAT_MODEL,
  messages: [{ role: 'user' as const, content: 'Describe the cat on the windowsill.', },],
  signal: OPEN_SIGNAL,
};

/**
 Schema-validated request, which names its stage by the schema it asked for.
 */
const JSON_REQUEST = {
  ...TEXT_REQUEST,
  responseFormat: {
    type: 'json_schema' as const,
    json_schema: {
      name: 'nap-ballot',
      schema: {},
    },
  },
  validate: function isAnything(value: unknown,): value is unknown {
    return value !== undefined;
  },
};

/**
 Builds a client whose exchanges answer exactly what a case scripts.

 @param reply - what a free-text exchange returns

 @param outcome - what a schema exchange returns, or `throw` to raise instead

 @param refusal - what a schema exchange raises under `throw`, where a case
 reads the row a particular class leaves; a transport drop otherwise

 @returns Client the wrapper can wrap, plus every quota read it served

 @example
 ```ts
 const inner = scriptedClient({ reply: { text: 'mew', }, },);
 ```
 */
function scriptedClient(
  { reply, outcome, refusal, }: {
    readonly reply?: {
      readonly text: string;
      readonly usage?: Record<string, number>;
    };
    readonly outcome?: Record<string, unknown> | 'throw';
    readonly refusal?: Error;
  },
): {
  readonly client: ModelClient;
  readonly quotaCalls: readonly string[];
} {
  /**
   Quota reads this client served, which must stay off the rows.
   */
  const quotaCalls: string[] = [];

  /**
   Scripted client before it is handed over as one.

   Cast once here rather than shaped to the contract: the point of the fixture
   is what comes BACK from an exchange, and scripting a refusal or a bare text
   reply per case is what the rows under test are built from.
   */
  const client = {
    chatText: async function chatText(): Promise<unknown> {
      return reply ?? { text: 'The cat says nothing.', };
    },
    chatJson: async function chatJson(): Promise<unknown> {
      if (outcome === 'throw')
        throw refusal ?? new Error('the provider dropped the connection mid-nap',);

      return outcome ?? { kind: 'ok', value: {}, rawText: '{}', };
    },
    quotas: async function quotas(): Promise<unknown> {
      quotaCalls.push('asked',);
      return {};
    },
  } as unknown as ModelClient;

  return {
    client,
    quotaCalls,
  };
}

await describe({
  name: recordingClient.name,
  children: [
    it({
      name:
        'records both halves of a free-text exchange and keeps the server`s own total rather than recomputing it, '
          + 'so a server stating a total its two halves do not add up to is priced as it billed '
          + 'rather than as the bench would have derived',
      fn: async () => {
        /**
         Wrapper over a server reporting a total larger than both halves.
         */
        const recorder = recordingClient({ inner: scriptedClient({ reply: {
          text: 'The cat naps.',
          usage: { prompt_tokens: 40, completion_tokens: 12, total_tokens: 100, },
        }, },).client, },);
        await recorder.client
          .chatText(TEXT_REQUEST,);

        /**
         Row that exchange left.
         */
        const row = recorder.calls[0] as BenchCall;
        expect(row.promptTokens,).toBe(40,);
        expect(row.completionTokens,).toBe(12,);
        expect(row.tokens,).toBe(100,);
        expect(row.outcome,).toBe('text',);

        // A free-text call asks for no schema, so the stage it names is `text`
        // rather than an empty string that would group with a real stage.
        expect(row.schema,).toBe('text',);
        expect(row.modelId,).toBe(CAT_MODEL,);
      },
    },),
    it({
      name:
        'falls back to the sum of both halves when the server states no total, '
          + 'and records zeros rather than a guess when it reports no usage at all',
      fn: async () => {
        /**
         Wrapper over a server that omits the total.
         */
        const summing = recordingClient({ inner: scriptedClient({ reply: {
          text: 'The cat stretches.',
          usage: { prompt_tokens: 7, completion_tokens: 5, },
        }, },).client, },);
        await summing.client
          .chatText(TEXT_REQUEST,);
        expect((summing.calls[0] as BenchCall).tokens,).toBe(12,);

        /**
         Wrapper over a server that reports no usage block.
         */
        const silent = recordingClient({
          inner: scriptedClient({ reply: { text: 'The cat blinks.', }, },).client,
        },);
        await silent.client
          .chatText(TEXT_REQUEST,);

        /**
         Row from the silent server.
         */
        const row = silent.calls[0] as BenchCall;
        expect(row.promptTokens,).toBe(0,);
        expect(row.completionTokens,).toBe(0,);
        expect(row.tokens,).toBe(0,);
      },
    },),
    it({
      name:
        'names a schema exchange by the schema it asked for, and carries the outcome kind through, '
          + 'so a refusal costs its tokens on the row that says it refused',
      fn: async () => {
        /**
         Wrapper over a server refusing the ballot.
         */
        const recorder = recordingClient({ inner: scriptedClient({ outcome: {
          kind: 'refusal-shaped',
          rawText: 'I would rather not judge cats.',
          usage: { prompt_tokens: 90, completion_tokens: 8, },
        }, },).client, },);
        await recorder.client
          .chatJson(JSON_REQUEST,);

        /**
         Row that refusal left.
         */
        const row = recorder.calls[0] as BenchCall;
        expect(row.schema,).toBe('nap-ballot',);
        expect(row.outcome,).toBe('refusal-shaped',);
        expect(row.promptTokens,).toBe(90,);
        expect(row.completionTokens,).toBe(8,);
      },
    },),
    it({
      name:
        'records a throw as a row of its own and rethrows it, because a transport failure after retries '
          + 'is the commonest cost under provider load and a wrapper that swallowed it would report a cheaper run than happened',
      fn: async () => {
        /**
         Wrapper over a server that drops the connection.
         */
        const recorder = recordingClient({ inner: scriptedClient({ outcome: 'throw', },).client, },);

        /**
         Failure the wrapper rethrew.
         */
        let caught: unknown;
        try {
          await recorder.client
            .chatJson(JSON_REQUEST,);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(Error,);

        /**
         Row the failure left.
         */
        const row = recorder.calls[0] as BenchCall;
        // An unmarked class is named and never quoted: its message may carry
        // a provider's words.
        expect(row.outcome,).toBe('threw refused by Error',);

        // Zero on every half: no usage came back, and crediting the prompt
        // would price a call the provider never billed.
        expect(row.promptTokens,).toBe(0,);
        expect(row.completionTokens,).toBe(0,);
        expect(row.tokens,).toBe(0,);
      },
    },),
    it({
      name: 'RECORDS A MARKED REFUSAL WHOLE on the row of a throw, the router\'s reason and how long the seat is held '
        + 'out included, since the row is all a reader of the bench learns of why the call failed',
      fn: async () => {
        /**
         Refusal a router raises for a seat no provider can take for now.
         */
        const refusal = new NoProviderForModelError({
          modelId: CAT_MODEL,
          reason: 'every provider serving this model ran past its stream bound; held out for another 90000ms',
        },);
        /**
         Wrapper over a client the router refused.
         */
        const recorder = recordingClient({ inner: scriptedClient({
          outcome: 'throw',
          refusal,
        },).client, },);

        expect(await rejectionOf({ promise: recorder.client.chatJson(JSON_REQUEST,), },),).toBe(refusal,);
        expect(recorder.calls.map(function outcomeOf({ outcome, },): string {
          return outcome;
        },),).toEqual([
          `threw no provider can take ${CAT_MODEL}: every provider serving this model ran past its stream bound; `
            + 'held out for another 90000ms',
        ],);
      },
    },),
    it({
      name: 'forwards a quota read without recording it, since reading quota costs no generation',
      fn: async () => {
        /**
         Inner client counting quota reads.
         */
        const inner = scriptedClient({},);

        /**
         Wrapper over it.
         */
        const recorder = recordingClient({ inner: inner.client, },);
        await recorder.client
          .quotas({ signal: OPEN_SIGNAL, },);
        expect(inner.quotaCalls,).toHaveLength(1,);
        expect(recorder.calls,).toHaveLength(0,);
      },
    },),
  ],
},);
