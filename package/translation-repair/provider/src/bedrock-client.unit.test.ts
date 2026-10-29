/**
 Tests for the Bedrock client over a recorded transport.

 THE STREAM SHAPES ARE THE ONES THE PROBES CAPTURED on 2026-09-07 from
 bedrock-mantle in us-east-1: the Gemma route answers content chunks, a
 usage chunk and `[DONE]`; the gpt-oss route answers reasoning deltas,
 content chunks and a usage chunk with no sentinel. Fixtures are cat-themed
 invention; no corpus content appears here.

 @module
 */

import { once, } from 'node:events';
import { setTimeout as sleepFor, } from 'node:timers/promises';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  StreamOverrunError,
  ProviderHttpError,
  type TransportExchange,
} from '@monochromatic-dev/translation-repair-provider-stream';
import { COMPLETION_CAP, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  BEDROCK_MODELS,
  type BedrockLedger,
  type BedrockLedgerEntry,
  BedrockModelNotServedError,
  createBedrockClient,
  isStreamBoundCut,
} from '../dist/final/node/index.mjs';
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

/**
 One chat completion chunk as the endpoint sends it.

 @param delta - delta fields for the single choice

 @param rest - top-level fields beyond the choice, usage included

 @returns Event line, newline-terminated

 @example
 ```ts
 const raw = chunkOf({ delta: { content: '{"spot":', }, },);
 ```
 */
function chunkOf(
  {
    delta,
    rest = {},
  }: {
    readonly delta: Readonly<Record<string, unknown>>;
    readonly rest?: Readonly<Record<string, unknown>>;
  },
): string {
  return `data: ${JSON.stringify({
    id: 'chatcmpl-595717d5-4cdb-44eb-bc27-2f6c1f0f1a1b',
    object: 'chat.completion.chunk',
    created: 1_788_811_674,
    choices: [{
      index: 0,
      delta,
      finish_reason: null,
    },],
    ...rest,
  },)}\n\n`;
}

/**
 Usage chunk with no choices, which both routes end on.
 */
const USAGE_CHUNK = `data: ${JSON.stringify({
  id: 'chatcmpl-595717d5-4cdb-44eb-bc27-2f6c1f0f1a1b',
  object: 'chat.completion.chunk',
  created: 1_788_811_674,
  choices: [],
  usage: {
    prompt_tokens: 90,
    completion_tokens: 10,
    total_tokens: 100,
  },
},)}\n\n`;

/**
 Whole stream the Gemma route answered with.
 */
const GEMMA_STREAM = [
  chunkOf({ delta: { role: 'assistant', content: '', }, },),
  chunkOf({ delta: { content: '{"spot":', }, },),
  chunkOf({ delta: { content: '"sunbeam"}', }, },),
  USAGE_CHUNK,
  'data: [DONE]\n\n',
].join('',);

/**
 Whole stream the gpt-oss route answered with: reasoning first, no sentinel.
 */
const GPT_OSS_STREAM = [
  chunkOf({ delta: { role: 'assistant', reasoning: 'The user asks where', }, },),
  chunkOf({ delta: { reasoning: ' the cat naps.', }, },),
  chunkOf({ delta: { content: '{"spot":"windowsill"}', }, },),
  USAGE_CHUNK,
].join('',);

/**
 Abort signal every call here carries.
 */
const SIGNAL = new AbortController().signal;

/**
 Stream bound the bound cases run under, short enough to cross in a test.
 */
const TEST_BOUND_MS = 300;

/**
 Retry backoff base in the case on a bound per attempt. The first backoff
 sleeps at least half of it, longer than the stream bound, so a bound armed
 once around the retry ladder has run out before the retry starts, while each
 attempt itself answers at once and so ends before its own bound on any
 machine.
 */
const BACKOFF_BASE_MS = 1_000;

/**
 Asks the Gemma route once and reads the outcome as data.

 @param client - client under test

 @returns Answer text, empty on a failure, and whether the failure was a
 stream bound cut

 @example
 ```ts
 const outcome = await boundedAsk({ client, },);
 ```
 */
async function boundedAsk(
  { client, }: { readonly client: ReturnType<typeof createBedrockClient>; },
): Promise<{ readonly text: string; readonly cut: boolean; }> {
  try {
    /**
     Reply when the call completes.
     */
    const reply = await client.chatText({
      modelId: SEAT_HYPER_TEXT_BEDROCK,
      messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
      signal: SIGNAL,
    },);
    return {
      text: reply.text,
      cut: false,
    };
  }
  catch (error) {
    return {
      text: '',
      cut: isStreamBoundCut({ error, },),
    };
  }
}

/**
 In-memory ledger recording what the client notes and answering a fixed
 reading.

 @param remainingUsd - what the reading says is left

 @returns Ledger plus the entries noted into it

 @example
 ```ts
 const { ledger, noted, } = memoryLedger({ remainingUsd: 150, },);
 ```
 */
function memoryLedger({ remainingUsd = 150, }: { readonly remainingUsd?: number; },) {
  /**
   Entries the client noted, in order.
   */
  const noted: BedrockLedgerEntry[] = [];

  /**
   Ledger surface over the list.
   */
  const ledger: BedrockLedger = {
    path: '/nowhere/bedrock-spend.jsonl',
    note: async function note(entry,): Promise<void> {
      noted.push(entry,);
    },
    read: async function read() {
      return {
        creditUsd: 200,
        spentUsd: 200 - remainingUsd,
        reckonedUsd: 0,
        remainingUsd,
        calls: noted.length,
      };
    },
  };
  return {
    ledger,
    noted,
  };
}

/**
 What an abandoned attempt could have been billed, off the body it sent:
 no more prompt tokens than the body has bytes, and no more completion tokens
 than its `max_tokens` (ledger P1).

 @param bodyJson - body the attempt sent

 @returns Entry fields the ledger should hold for it

 @example
 ```ts
 const bound = boundOf({ bodyJson: exchanges[0]?.bodyJson ?? '', },);
 ```
 */
function boundOf(
  { bodyJson, }: { readonly bodyJson: string; },
): Pick<BedrockLedgerEntry, 'completionTokens' | 'model' | 'promptTokens' | 'usd'> & { readonly estimated: string; } {
  /**
   Body as sent.
   */
  const body = JSON.parse(bodyJson,) as { readonly model: keyof typeof BEDROCK_MODELS; readonly max_tokens: number; };
  /**
   Prices of the model it asked.
   */
  const prices = BEDROCK_MODELS[body.model];
  /**
   Bytes the body carried.
   */
  const promptTokens = Buffer.byteLength(bodyJson,);
  return {
    model: body.model,
    promptTokens,
    completionTokens: body.max_tokens,
    usd: ((promptTokens * prices.promptUsdPerMillion) + (body.max_tokens * prices.completionUsdPerMillion))
      / 1_000_000,
    estimated: 'abandoned-bound',
  };
}

/**
 Builds a client over a transport that records what it was sent.

 @param reply - what the chat endpoint answers

 @returns Client plus the exchanges the transport saw and the ledger's notes

 @example
 ```ts
 const { client, exchanges, } = recordedClient({},);
 ```
 */
function recordedClient(
  { reply = { status: 200, bodyText: GEMMA_STREAM, }, }: {
    readonly reply?: { readonly status: number; readonly bodyText: string; };
  },
) {
  /**
   Every exchange the transport was handed.
   */
  const exchanges: TransportExchange[] = [];

  /**
   Ledger the client writes to.
   */
  const { ledger, noted, } = memoryLedger({},);
  return {
    exchanges,
    noted,
    client: createBedrockClient({
      apiKey: 'test-key',
      ledger,
      baseUrl: 'https://mantle.invalid',
      transport: async function transport(exchange,) {
        exchanges.push(exchange,);
        return reply;
      },
      retryPolicy: {
        limit: 0,
        baseMs: 1,
      },
      // NO STREAM BOUND: a card's own is armed on the real clock around every
      // attempt (ledger B416), and these cases read what was sent.
      streamBoundMsOverride: 'unbounded',
    },),
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: createBedrockClient.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SENDS the chat completions body on the Gemma route with the Bedrock spelling, the schema '
            + 'in both places, streaming usage on, no provider field and no store, and READS the answer '
            + 'back, PRICING it into the ledger',
          fn: async () => {
            const { client, exchanges, noted, } = recordedClient({},);

            /**
             One schema'd call as a stage would make it, on the shared seat.
             */
            const reply = await client.chatText({
              modelId: SEAT_HYPER_TEXT_BEDROCK,
              messages: [
                { role: 'system', content: 'You are a careful cat.', },
                { role: 'user', content: 'Where does the cat sleep?', },
              ],
              signal: SIGNAL,
              responseFormat: {
                type: 'json_schema',
                json_schema: {
                  name: 'nap_spot',
                  schema: {
                    type: 'object',
                    required: ['spot',],
                    properties: { spot: { type: 'string', }, },
                  },
                },
              },
            },);

            expect(reply.text,).toBe('{"spot":"sunbeam"}',);
            expect(reply.usage,).toMatchObject({
              prompt_tokens: 90,
              completion_tokens: 10,
            },);

            /**
             What went on the wire.
             */
            const [exchange,] = exchanges;
            if (exchange === undefined)
              throw new Error('nothing was sent',);
            expect(exchange.url,).toBe('https://mantle.invalid/openai/v1/chat/completions',);
            expect(exchange.headers.Authorization,).toBe('Bearer test-key',);

            /**
             Body as the endpoint would parse it.
             */
            const body: unknown = JSON.parse(exchange.bodyJson ?? '{}',);
            expect(body,).toMatchObject({
              model: 'google.gemma-4-26b-a4b',
              stream: true,
              stream_options: { include_usage: true, },
              response_format: {
                type: 'json_schema',
                json_schema: {
                  name: 'nap_spot',
                  schema: {
                    type: 'object',
                    required: ['spot',],
                    properties: { spot: { type: 'string', }, },
                  },
                },
              },
            },);
            expect(JSON.stringify(body,),).toContain('nap_spot',);
            expect(JSON.stringify(body,),).not.toContain('"provider"',);
            expect(JSON.stringify(body,),).not.toContain('"store"',);
            expect(JSON.stringify(body,),).not.toContain('reasoning_effort',);
            // THE MEASURED CEILING RIDES ON EVERY CALL since 2026-09-09: the
            // credit behind this provider is never topped up.
            expect(body,).toMatchObject({ max_tokens: COMPLETION_CAP[SEAT_HYPER_TEXT_BEDROCK], },);

            /**
             What the ledger was told.
             */
            const [entry,] = noted;
            if (entry === undefined)
              throw new Error('nothing was noted',);
            expect(entry.model,).toBe('google.gemma-4-26b-a4b',);
            expect(entry.promptTokens,).toBe(90,);
            expect(entry.completionTokens,).toBe(10,);
            expect(entry.usd,).toBeGreaterThan(0,);
          },
        },),

        it({
          name: 'READS the gpt-oss route\'s stream, which ends on its usage chunk with no sentinel and '
            + 'carries reasoning deltas first, under /v1',
          fn: async () => {
            const { client, exchanges, } = recordedClient({
              reply: {
                status: 200,
                bodyText: GPT_OSS_STREAM,
              },
            },);

            /**
             One plain call on the seat every provider serves.
             */
            const reply = await client.chatText({
              modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
              messages: [{ role: 'user', content: 'Where does the cat nap?', },],
              signal: SIGNAL,
            },);
            expect(reply.text,).toBe('{"spot":"windowsill"}',);
            expect(reply.usage?.completion_tokens,).toBe(10,);
            expect(exchanges[0]?.url,).toBe('https://mantle.invalid/v1/chat/completions',);
            expect(JSON.parse(exchanges[0]?.bodyJson ?? '{}',),).toMatchObject({ model: 'openai.gpt-oss-120b', },);
          },
        },),

        it({
          name: 'REFUSES a Gemma-route stream cut off before its sentinel as malformed rather than '
            + 'returning the fragment, since a stream that stopped early comes back as 200, and LEDGERS it at '
            + 'what it could have been billed, since the endpoint accepted it (ledger P1)',
          fn: async () => {
            const { client, noted, exchanges, } = recordedClient({
              reply: {
                status: 200,
                bodyText: chunkOf({ delta: { content: '{"spot":', }, },),
              },
            },);

            /**
             What the cut stream produced.
             */
            let thrown: unknown;
            try {
              await client.chatText({
                modelId: SEAT_BEDROCK_ONLY_TEXT,
                messages: [{ role: 'user', content: 'meow', },],
                signal: SIGNAL,
              },);
            } catch (error) {
              thrown = error;
            }
            expect(thrown,).toBeInstanceOf(Error,);
            expect((thrown as Error).name,).toBe('MalformedCompletionError',);
            expect(noted,).toHaveLength(1,);
            expect(noted[0],).toMatchObject(boundOf({ bodyJson: exchanges[0]?.bodyJson ?? '', },),);
          },
        },),

        it({
          name: 'READS the ledger for its credits, which is this provider\'s whole budget signal',
          fn: async () => {
            const { client, } = recordedClient({},);
            expect(await client.credits({ signal: SIGNAL, },),).toEqual({
              creditUsd: 200,
              spentUsd: 50,
              reckonedUsd: 0,
              remainingUsd: 150,
              calls: 0,
            },);
          },
        },),

        it({
          name: 'REFUSES a roster model it has no spelling for before touching the wire, since that is a '
            + 'routing mistake in our own code',
          fn: async () => {
            const { client, exchanges, } = recordedClient({},);

            /**
             What a call for a seat this provider does not serve produces.
             */
            let thrown: unknown;
            try {
              await client.chatText({
                modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
                messages: [{ role: 'user', content: 'meow', },],
                signal: SIGNAL,
              },);
            } catch (error) {
              thrown = error;
            }
            expect(thrown instanceof BedrockModelNotServedError,).toBe(true,);
            expect(exchanges,).toHaveLength(0,);
          },
        },),

        it({
          name: 'THROWS the shared HTTP failure class on a non-success status, which the budget layer '
            + 'reads for a refusal, and NOTES nothing',
          fn: async () => {
            const { client, noted, } = recordedClient({
              reply: {
                status: 429,
                bodyText: '{"error":{"message":"Too many requests"}}',
              },
            },);

            /**
             What the refused call produced.
             */
            let thrown: unknown;
            try {
              await client.chatText({
                modelId: SEAT_BEDROCK_ONLY_VISION_UNSEATED,
                messages: [{ role: 'user', content: 'meow', },],
                signal: SIGNAL,
              },);
            } catch (error) {
              thrown = error;
            }
            expect(thrown instanceof ProviderHttpError,).toBe(true,);
            expect((thrown as ProviderHttpError).status,).toBe(429,);
            expect((thrown as ProviderHttpError).provider,).toBe('bedrock',);
            expect(noted,).toHaveLength(0,);
          },
        },),

        it({
          name: 'VALIDATES a schema\'d answer through the shared outcome reader on chatJson',
          fn: async () => {
            const { client, } = recordedClient({},);

            /**
             Outcome of a guarded call.
             */
            const outcome = await client.chatJson({
              modelId: SEAT_BEDROCK_ONLY_TEXT,
              messages: [{ role: 'user', content: 'Where does the cat nap?', },],
              signal: SIGNAL,
              validate: function isNapSpot(value: unknown,): value is { readonly spot: string; } {
                return ((typeof value) === 'object') && (value !== null) && ('spot' in value);
              },
            },);
            expect(outcome.kind,).toBe('ok',);
            if (outcome.kind === 'ok')
              expect(outcome.value.spot,).toBe('sunbeam',);
          },
        },),

        it({
          name: 'ARMS NO STREAM BOUND when the override is unbounded, though the Gemma card carries one: the transport '
            + 'is handed the caller\'s own signal',
          fn: async () => {
            const { client, exchanges, } = recordedClient({},);
            await client.chatText({
              modelId: SEAT_HYPER_TEXT_BEDROCK,
              messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
              signal: SIGNAL,
            },);
            expect(BEDROCK_MODELS['google.gemma-4-26b-a4b'].streamBoundMs,).not.toBe('unbounded',);
            expect(exchanges,).toHaveLength(1,);
            // By identity: a bound's own signal is another AbortSignal, which a
            // deep comparison could not tell from the caller's.
            expect(exchanges[0]?.signal,).toBe(SIGNAL,);
          },
        },),

        it({
          name: 'READS THE CARD\'S OWN STREAM BOUND where no override is given: the gpt-oss card carries none, so the '
            + 'transport is handed the caller\'s own signal',
          fn: async () => {
            /**
             Every exchange the transport was handed.
             */
            const exchanges: TransportExchange[] = [];
            const client = createBedrockClient({
              apiKey: 'test-key',
              ledger: memoryLedger({},).ledger,
              baseUrl: 'https://mantle.invalid',
              transport: async function transport(exchange,) {
                exchanges.push(exchange,);
                return {
                  status: 200,
                  bodyText: GPT_OSS_STREAM,
                };
              },
              retryPolicy: {
                limit: 0,
                baseMs: 1,
              },
            },);
            await client.chatText({
              modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
              messages: [{ role: 'user', content: 'Where does the cat nap?', },],
              signal: SIGNAL,
            },);
            expect(BEDROCK_MODELS['openai.gpt-oss-120b'].streamBoundMs,).toBe('unbounded',);
            expect(exchanges,).toHaveLength(1,);
            expect(exchanges[0]?.signal,).toBe(SIGNAL,);
          },
        },),

        it({
          name: 'SENDS A CALLER\'S OWN CEILING under the measured cap and HANDS ITS ANSWER BOUND to the transport, '
            + 'under a deadline signal of its own rather than the caller\'s',
          fn: async () => {
            const { client, exchanges, } = recordedClient({},);
            await client.chatText({
              modelId: SEAT_HYPER_TEXT_BEDROCK,
              messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
              signal: SIGNAL,
              maxTokens: 64,
              maxAnswerChars: 2_000,
              exchangeTimeoutMs: HANG_STOP_MS,
            },);

            /**
             What went on the wire.
             */
            const [exchange,] = exchanges;
            if (exchange === undefined)
              throw new Error('nothing was sent',);
            expect(64,).toBeLessThan(COMPLETION_CAP[SEAT_HYPER_TEXT_BEDROCK],);
            expect(JSON.parse(exchange.bodyJson ?? '{}',),).toMatchObject({ max_tokens: 64, },);
            expect(exchange.maxAnswerChars,).toBe(2_000,);
            expect(exchange.signal,).not.toBe(SIGNAL,);
          },
        },),

        it({
          name: 'FORFEITS A CALL THAT OUTLASTS ITS DEADLINE, naming the served model and the deadline',
          fn: async () => {
            const client = createBedrockClient({
              apiKey: 'test-key',
              ledger: memoryLedger({},).ledger,
              baseUrl: 'https://mantle.invalid',
              transport: async function waitsForAbort(exchange,) {
                // Answers only once the exchange is aborted, with the abort's
                // own reason, as a fetch on that signal would.
                await once(
                  exchange.signal,
                  'abort',
                );
                throw exchange.signal.reason;
              },
              retryPolicy: {
                limit: 0,
                baseMs: 1,
              },
              // No stream bound, so the deadline is the one timer this call
              // races (ledger B416).
              streamBoundMsOverride: 'unbounded',
            },);

            /**
             What the call threw.
             */
            const thrown = await client
              .chatText({
                modelId: SEAT_HYPER_TEXT_BEDROCK,
                messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
                signal: SIGNAL,
                exchangeTimeoutMs: 50,
              },)
              .then(
              function unexpected(): unknown {
                return undefined;
              },
              function caught(error: unknown,): unknown {
                return error;
              },
              );
            expect((thrown as Error).message,).toBe('Timeout: google.gemma-4-26b-a4b exceeded its 50ms deadline',);
          },
        },),

        it({
          name: 'GIVES EACH ATTEMPT THE WHOLE STREAM BOUND, so a retry after a transient failure is not cut by '
            + 'the time spent before it, in the failed attempt or the backoff (ledger P13: the bound was armed once '
            + 'around the retry ladder)',
          fn: async () => {
            /**
             Attempts the transport saw.
             */
            const calls = { count: 0, };
            const client = createBedrockClient({
              apiKey: 'test-key',
              ledger: memoryLedger({},).ledger,
              baseUrl: 'https://mantle.invalid',
              // BOTH ATTEMPTS ANSWER AT ONCE and the time passes in the backoff
              // between them, so no attempt races its own bound: each attempt
              // slept 200 ms against a bound of 300 ms here, which a stall of
              // 100 ms between arming the bound and the sleep would have cut.
              transport: async function failureThenWhole() {
                calls.count += 1;
                return (calls.count === 1)
                  ? {
                    status: 503,
                    bodyText: 'busy',
                  }
                  : {
                    status: 200,
                    bodyText: GEMMA_STREAM,
                  };
              },
              retryPolicy: {
                limit: 1,
                baseMs: BACKOFF_BASE_MS,
              },
              streamBoundMsOverride: TEST_BOUND_MS,
            },);
            expect({
              ...(await boundedAsk({ client, },)),
              calls: calls.count,
            },).toEqual({
              text: '{"spot":"sunbeam"}',
              cut: false,
              calls: 2,
            },);
          },
        },),

        it({
          name: 'NEVER RETRIES A STREAM THE BOUND CUT, since the cut is this system\'s own end and the router holds '
            + 'Bedrock out for the model on it',
          fn: async () => {
            /**
             Attempts the transport saw.
             */
            const calls = { count: 0, };
            const client = createBedrockClient({
              apiKey: 'test-key',
              ledger: memoryLedger({},).ledger,
              baseUrl: 'https://mantle.invalid',
              transport: async function queuedPastTheBound(exchange,) {
                calls.count += 1;
                await sleepFor(
                  TEST_BOUND_MS * 2,
                  undefined,
                  { signal: exchange.signal, },
                );
                return {
                  status: 200,
                  bodyText: GEMMA_STREAM,
                };
              },
              retryPolicy: {
                limit: 2,
                baseMs: 1,
              },
              streamBoundMsOverride: TEST_BOUND_MS,
            },);
            expect({
              ...(await boundedAsk({ client, },)),
              calls: calls.count,
            },).toEqual({
              text: '',
              cut: true,
              calls: 1,
            },);
          },
        },),
      ],
    },),

    describe({
      name: 'every billed Bedrock attempt reaches the ledger (ledger P1)',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LEDGERS A STREAM THIS PIPELINE ENDED, at what it could have been billed: 873 Bedrock streams ended '
            + 'cut or overrun across the logs and none reached the ledger, the only guard on the owner\'s card',
          fn: async () => {
            /**
             Exchanges the transport saw.
             */
            const exchanges: TransportExchange[] = [];
            /**
             Ledger the client writes to.
             */
            const { ledger, noted, } = memoryLedger({},);
            /**
             Client whose one stream overruns.
             */
            const client = createBedrockClient({
              apiKey: 'test-key',
              ledger,
              baseUrl: 'https://mantle.invalid',
              transport: async function overrunning(exchange,) {
                exchanges.push(exchange,);
                throw new StreamOverrunError({
                  label: 'google.gemma-4-e2b',
                  channel: 'content',
                  charsSeen: 900,
                  cap: 800,
                  rawChars: 81_000,
                },);
              },
              retryPolicy: {
                limit: 2,
                baseMs: 1,
              },
              // No stream bound on the real clock (ledger B416).
              streamBoundMsOverride: 'unbounded',
            },);
            await expect(client.chatText({
              modelId: SEAT_BEDROCK_ONLY_TEXT,
              messages: [{ role: 'user', content: 'meow', },],
              signal: SIGNAL,
            },),).rejects.toBeInstanceOf(StreamOverrunError,);
            expect(noted,).toHaveLength(1,);
            expect(noted[0],).toMatchObject(boundOf({ bodyJson: exchanges[0]?.bodyJson ?? '', },),);
          },
        },),
        it({
          name: 'LEDGERS EVERY BILLED ATTEMPT OF A RETRIED CALL, the refused one at its bound and the whole one '
            + 'at its reported usage',
          fn: async () => {
            /**
             Exchanges the transport saw.
             */
            const exchanges: TransportExchange[] = [];
            /**
             Ledger the client writes to.
             */
            const { ledger, noted, } = memoryLedger({},);
            /**
             Client whose first stream stops early and whose second is whole.
             */
            const client = createBedrockClient({
              apiKey: 'test-key',
              ledger,
              baseUrl: 'https://mantle.invalid',
              transport: async function truncatedThenWhole(exchange,) {
                exchanges.push(exchange,);
                return (exchanges.length === 1)
                  ? { status: 200, bodyText: chunkOf({ delta: { content: '{"spot":', }, },), }
                  : { status: 200, bodyText: GEMMA_STREAM, };
              },
              retryPolicy: {
                limit: 1,
                baseMs: 1,
              },
              // No stream bound on the real clock (ledger B416).
              streamBoundMsOverride: 'unbounded',
            },);
            await client.chatText({
              modelId: SEAT_BEDROCK_ONLY_TEXT,
              messages: [{ role: 'user', content: 'meow', },],
              signal: SIGNAL,
            },);
            expect(noted,).toHaveLength(2,);
            expect(noted[0],).toMatchObject(boundOf({ bodyJson: exchanges[0]?.bodyJson ?? '', },),);
            expect(noted[1],).toMatchObject({ promptTokens: 90, completionTokens: 10, },);
            expect(noted[1]?.estimated,).toBeUndefined();
          },
        },),
        it({
          name: 'LEDGERS A WHOLE CALL WHOSE STREAM REPORTED NO USAGE at its bound, marked so, since the call was '
            + 'billed whatever the stream said; none of 190,009 logged Bedrock calls lacked usage, and the guard '
            + 'on the owner\'s card must not rest on that',
          fn: async () => {
            const { client, noted, exchanges, } = recordedClient({
              reply: {
                status: 200,
                bodyText: [
                  chunkOf({ delta: { role: 'assistant', content: '', }, },),
                  chunkOf({ delta: { content: '{"spot":"sunbeam"}', }, },),
                  'data: [DONE]\n\n',
                ].join('',),
              },
            },);
            await client.chatText({
              modelId: SEAT_BEDROCK_ONLY_TEXT,
              messages: [{ role: 'user', content: 'meow', },],
              signal: SIGNAL,
            },);
            expect(noted,).toHaveLength(1,);
            expect(noted[0],).toMatchObject({
              ...boundOf({ bodyJson: exchanges[0]?.bodyJson ?? '', },),
              estimated: 'unreported-bound',
            },);
          },
        },),
      ],
    },),
  ],
},);
