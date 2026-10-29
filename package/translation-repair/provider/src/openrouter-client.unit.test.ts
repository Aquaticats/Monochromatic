/**
 Tests for the OpenRouter client over a recorded transport.

 THE STREAM SHAPE IS THE ONE THE PROBE CAPTURED on 2026-09-03 from
 `deepseek/deepseek-v4-flash-0731` via Inceptron: a `: OPENROUTER PROCESSING`
 comment line, reasoning deltas before content, a final chunk carrying
 `usage` with `cost`, and the `[DONE]` sentinel. Fixtures are cat-themed
 invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ProviderHttpError,
  type TransportExchange,
} from '@monochromatic-dev/translation-repair-provider-stream';
import {
  COMPLETION_CAP,
  resetRunSpend,
  runSpendUsd,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  CACHED_UNREPORTED,
  COST_UNREPORTED,
  createOpenRouterClient,
  ENDPOINT_UNREPORTED,
  estimateAbandonedSpend,
  InStreamProviderError,
  OPENROUTER_CHAT_URL,
  OPENROUTER_CREDITS_URL,
  OpenRouterModelNotServedError,
  reportedSpendFieldsOf,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_UNMEASURED_OPENROUTER_ID,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

/**
 One chat completion chunk as the gateway sends it.

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
    id: 'gen-1788450765-akMiAoFBnp64lvAW8sIs',
    object: 'chat.completion.chunk',
    created: 1_788_450_765,
    model: 'deepseek/deepseek-v4.1-flash',
    provider: 'Inceptron',
    choices: [{
      index: 0,
      delta: {
        role: 'assistant',
        ...delta,
      },
      finish_reason: null,
    },],
    ...rest,
  },)}\n\n`;
}

/**
 USD the captured call's usage block reported.
 */
const RECORDED_COST_USD = 0.00015646;

/**
 Whole stream the captured call answered with, answer and cost included.
 */
const RECORDED_STREAM = [
  ': OPENROUTER PROCESSING\n\n',
  chunkOf({ delta: { content: '', reasoning: 'The cat', }, },),
  chunkOf({ delta: { content: '{"spot":', reasoning: null, }, },),
  chunkOf({ delta: { content: ' "windowsill"}', }, },),
  chunkOf({
    delta: { content: '', },
    rest: {
      usage: {
        prompt_tokens: 342,
        completion_tokens: 400,
        total_tokens: 742,
        cost: RECORDED_COST_USD,
        is_byok: false,
      },
    },
  },),
  'data: [DONE]\n\n',
].join('',);

/**
 Abort signal every call here carries.
 */
const SIGNAL = new AbortController().signal;

/**
 Builds a client over a transport that records what it was sent.

 @param reply - what the chat endpoint answers

 @returns Client plus the exchanges the transport saw

 @example
 ```ts
 const { client, exchanges, } = recordedClient({},);
 ```
 */
function recordedClient(
  { reply = { status: 200, bodyText: RECORDED_STREAM, }, }: {
    readonly reply?: { readonly status: number; readonly bodyText: string; };
  },
) {
  /**
   Every exchange the transport was handed.
   */
  const exchanges: TransportExchange[] = [];
  return {
    exchanges,
    client: createOpenRouterClient({
      apiKey: 'test-key',
      transport: async function transport(exchange,) {
        exchanges.push(exchange,);
        if (exchange.url === OPENROUTER_CREDITS_URL)
          return { status: 200, bodyText: '{"data":{"total_credits":1913,"total_usage":1855.38}}', };
        return reply;
      },
      retryPolicy: {
        limit: 0,
        baseMs: 1,
      },
    },),
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: createOpenRouterClient.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SENDS the chat completions body with the OpenRouter slug, the routing preferences, the '
            + 'schema in both places, and streaming usage on, and READS the answer and its cost back',
          fn: async () => {
            const { client, exchanges, } = recordedClient({},);
            /**
             One schema'd call as a stage would make it.
             */
            const reply = await client.chatText({
              modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
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

            expect(reply.text,).toBe('{"spot": "windowsill"}',);
            expect(reply.usage,).toMatchObject({
              prompt_tokens: 342,
              completion_tokens: 400,
            },);

            /**
             What went on the wire.
             */
            const [exchange,] = exchanges;
            if (exchange === undefined)
              throw new Error('nothing was sent',);
            expect(exchange.url,).toBe(OPENROUTER_CHAT_URL,);
            expect(exchange.headers.Authorization,).toBe('Bearer test-key',);
            /**
             Body as the gateway would parse it.
             */
            const body: unknown = JSON.parse(exchange.bodyJson ?? '{}',);
            expect(body,).toMatchObject({
              model: SEAT_HYPER_OPENROUTER_UNMEASURED_OPENROUTER_ID,
              stream: true,
              stream_options: { include_usage: true, },
              provider: {
                zdr: true,
                require_parameters: true,
                sort: 'price',
                // DeepInfra and Wafer ignored for this seat since 2026-09-18 (XingZ607),
                // OpenInference since 2026-09-23 (XingZ624 and XingZ625, class ninety-one),
                // DekaLLM and Sail Research the same day (XingZ626, class ninety-three),
                // which also named Morph ahead of the price sort.
                ignore: [
                  'deepinfra',
                  'wafer',
                  'open-inference',
                  'dekallm',
                  'sail-research',
                ],
                order: ['morph',],
              },
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
            // The schema is restated in the system prompt as on the Synthetic
            // path, so a model that ignores `response_format` still reads it.
            expect(JSON.stringify(body,),).toContain('nap_spot',);
            // THE MEASURED CEILING RIDES ON EVERY CALL since 2026-09-09: the
            // per-token provider bills an abandoned stream to its end on endpoints
            // that do not honour a cancel, and this is the bound that holds there.
            expect(body,).toMatchObject({ max_tokens: COMPLETION_CAP[SEAT_HYPER_OPENROUTER_UNMEASURED], },);
          },
        },),

        it({
          name: 'LOWERS a caller\'s max_tokens to the measured ceiling and keeps a smaller one, so no call '
            + 'asks the per-token provider for more output than a finished call has needed',
          fn: async () => {
            const { client, exchanges, } = recordedClient({},);
            await client.chatText({
              modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
              messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
              signal: SIGNAL,
              maxTokens: 1_000_000,
            },);
            await client.chatText({
              modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
              messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
              signal: SIGNAL,
              maxTokens: 50,
            },);
            /**
             Both bodies as the gateway would parse them.
             */
            const bodies = exchanges.map(function parse(exchange,): unknown {
              return JSON.parse(exchange.bodyJson ?? '{}',);
            },);
            expect(bodies[0],).toMatchObject({ max_tokens: COMPLETION_CAP[SEAT_HYPER_OPENROUTER_UNMEASURED], },);
            expect(bodies[1],).toMatchObject({ max_tokens: 50, },);
          },
        },),

        it({
          name: 'KEEPS MiniMax M3 off Parasail and ModelRun and asks Together then CoreWeave on the wire: the '
            + 'body\'s provider.ignore carries the catalog row\'s slugs, so the endpoint that answers into the '
            + 'reasoning channel and the one that times out in-stream are never routed to, and provider.order '
            + 'names the measured endpoints ahead of the price sort (ledger H12)',
          fn: async () => {
            const { client, exchanges, } = recordedClient({},);
            await client.chatText({
              modelId: SEAT_HYPER_VISION,
              messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
              signal: SIGNAL,
            },);
            /**
             What went on the wire.
             */
            const [exchange,] = exchanges;
            if (exchange === undefined)
              throw new Error('nothing was sent',);
            /**
             Body as the gateway would parse it.
             */
            const body: unknown = JSON.parse(exchange.bodyJson ?? '{}',);
            expect(body,).toMatchObject({
              model: 'minimax/minimax-m3',
              provider: {
                zdr: true,
                require_parameters: true,
                sort: 'price',
                ignore: [
                  'parasail',
                  'modelrun',
                ],
                order: ['together', 'coreweave',],
              },
            },);
          },
        },),

        it({
          name: 'READS credits purchased, used and remaining off the credits endpoint',
          fn: async () => {
            const { client, } = recordedClient({},);
            expect(await client.credits({ signal: SIGNAL, },),).toEqual({
              purchasedUsd: 1_913,
              usedUsd: 1_855.38,
              remainingUsd: 1_913 - 1_855.38,
            },);
          },
        },),

        it({
          name: 'THROWS the shared HTTP failure class on a non-success /credits reply too, naming the '
            + 'endpoint, the status and the body, so the budget layer reads one failure class off both '
            + 'endpoints',
          fn: async () => {
            /**
             Client whose credits endpoint answers a server error.
             */
            const client = createOpenRouterClient({
              apiKey: 'test-key',
              transport: async function creditsBroken(exchange,) {
                return (exchange.url === OPENROUTER_CREDITS_URL)
                  ? { status: 500, bodyText: 'oops', }
                  : { status: 200, bodyText: RECORDED_STREAM, };
              },
              retryPolicy: {
                limit: 0,
                baseMs: 1,
              },
            },);
            /**
             What a broken credits endpoint produces.
             */
            let thrown: unknown;
            try {
              await client.credits({ signal: SIGNAL, },);
            } catch (error) {
              thrown = error;
            }
            expect(thrown,).toBeInstanceOf(ProviderHttpError,);
            expect(String(thrown,),).toBe('ProviderHttpError: OpenRouter /credits returned HTTP 500: oops',);
            expect((thrown as ProviderHttpError).status,).toBe(500,);
            expect((thrown as ProviderHttpError).provider,).toBe('openrouter',);
          },
        },),

        it({
          name: 'REFUSES a roster model it has no slug for before touching the wire, since that is a '
            + 'routing mistake in our own code',
          fn: async () => {
            const { client, exchanges, } = recordedClient({},);
            /**
             What a call for a name outside the catalog produces.
             */
            let thrown: unknown;
            try {
              await client.chatText({
                // A departed identity no catalog serves, cast past the roster type
                // the way a stale artifact could carry it.
                modelId: 'hf:zai-org/GLM-4.7-Flash' as never,
                messages: [{ role: 'user', content: 'meow', },],
                signal: SIGNAL,
              },);
            } catch (error) {
              thrown = error;
            }
            expect(thrown instanceof OpenRouterModelNotServedError,).toBe(true,);
            expect(exchanges,).toHaveLength(0,);
          },
        },),

        it({
          name: 'NAMES AN IN-STREAM PROVIDER FAILURE by its code and endpoint when a 200 stream carries '
            + 'the gateway\'s error chunk and no terminator, rather than calling the reply cut off: on '
            + '2026-09-04 that misnaming hid one endpoint failing two calls in five',
          fn: async () => {
            const { client, } = recordedClient({
              reply: {
                status: 200,
                bodyText: `: OPENROUTER PROCESSING\n\n${
              chunkOf({
                delta: {},
                rest: {
                  provider: 'ModelRun',
                  error: {
                    code: 504,
                    message: 'error code: 504',
                    metadata: { error_type: 'timeout', },
                  },
                },
              },)
            }`,
              },
            },);
            /**
             What the failed stream produces once the ladder (limit 0) gives up.
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
            expect(thrown instanceof InStreamProviderError,).toBe(true,);
            expect((thrown as InStreamProviderError).code,).toBe(504,);
            expect((thrown as InStreamProviderError).endpoint,).toBe('ModelRun',);
            expect((thrown as InStreamProviderError).message,).toContain('served by ModelRun',);
          },
        },),

        it({
          name: 'SPENDS ONE EXCHANGE on a 200 stream whose error chunk carries a 4xx code, since the same '
            + 'refusal over plain HTTP is returned unretried, and a 429 chunk rides the whole ladder',
          fn: async () => {
            /**
             What each code drew and ended with, over a ladder of two retries.
             */
            const outcomes = await Promise.all([400, 429,].map(async function driveCode(code,): Promise<{
              readonly code: string;
              readonly exchanges: number;
              readonly ended: string;
            }> {
              /**
               Exchanges this code drew.
               */
              const exchanges: TransportExchange[] = [];
              /**
               Client whose every stream carries the error chunk.
               */
              const client = createOpenRouterClient({
                apiKey: 'test-key',
                transport: async function alwaysFails(exchange,) {
                  exchanges.push(exchange,);
                  return {
                    status: 200,
                    bodyText: chunkOf({
                      delta: {},
                      rest: {
                        provider: 'ModelRun',
                        error: {
                          code,
                          message: 'the sill refused',
                          metadata: { error_type: 'invalid_request', },
                        },
                      },
                    },),
                  };
                },
                retryPolicy: {
                  limit: 2,
                  baseMs: 1,
                },
              },);
              /**
               What the call ended with.
               */
              const thrown = await client.chatText({
                modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
                messages: [{ role: 'user', content: 'meow', },],
                signal: SIGNAL,
              },).then(
                function noRefusal(): string {
                  return 'returned';
                },
                function refused(error: unknown,): string {
                  return Error.isError(error,) ? `${error.name}: ${error.message}` : String(error,);
                },
              );
              return {
                code: String(code,),
                exchanges: exchanges.length,
                ended: thrown,
              };
            },),);
            /**
             Exchanges each code drew.
             */
            const drawn = Object.fromEntries(outcomes.map(function toDrawn(outcome,): readonly [string, number,] {
              return [outcome.code, outcome.exchanges,];
            },),);
            /**
             What each code ended the call with.
             */
            const ended = Object.fromEntries(outcomes.map(function toEnded(outcome,): readonly [string, string,] {
              return [outcome.code, outcome.ended,];
            },),);
            expect({ drawn, ended, },).toStrictEqual({
              drawn: { '400': 1, '429': 3, },
              ended: {
                '400': 'InStreamRefusalError: stream carried a refusal of the request itself instead of a completion: '
                  + 'code 400, type invalid_request, served by ModelRun; the gateway had already sent a success '
                  + 'status, and repeating the request meets the same refusal',
                '429': 'InStreamProviderError: stream carried a provider failure instead of a completion: code 429, '
                  + 'type invalid_request, served by ModelRun; the gateway had already sent a success status',
              },
            },);
          },
        },),

        it({
          name: 'THROWS the shared HTTP failure class on a non-success status, which the budget layer '
            + 'reads for 402 and 429',
          fn: async () => {
            const { client, } = recordedClient({
              reply: {
                status: 402,
                bodyText: '{"error":{"message":"insufficient credits"}}',
              },
            },);
            /**
             What a payment refusal produces.
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
            expect(thrown instanceof ProviderHttpError,).toBe(true,);
            expect((thrown as ProviderHttpError).status,).toBe(402,);
            expect((thrown as ProviderHttpError).provider,).toBe('openrouter',);
          },
        },),

        it({
          name: 'HANDS the transport a deadline signal of its own and the caller\'s maxAnswerChars where the '
            + 'caller sets both, and the caller\'s own signal with no answer bound where the caller sets '
            + 'neither, the answer read back either way',
          fn: async () => {
            const { client, exchanges, } = recordedClient({},);
            /**
             Answer to a call that sets neither knob, asked first so its
             exchange is the first the transport records.
             */
            const bare = await client.chatText({
              modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
              messages: [
                {
                  role: 'user',
                  content: 'Where does the cat sleep?',
                },
              ],
              signal: SIGNAL,
            },);
            /**
             Answer to a call that sets a deadline and an answer bound.
             */
            const knobbed = await client.chatText({
              modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
              messages: [
                {
                  role: 'user',
                  content: 'Where does the cat sleep?',
                },
              ],
              signal: SIGNAL,
              exchangeTimeoutMs: HANG_STOP_MS,
              maxAnswerChars: 1_000,
            },);

            /**
             What each call put on the wire, in the order the calls were made.
             */
            const [
              bareExchange,
              knobbedExchange,
            ] = exchanges;
            if ((bareExchange === undefined) || (knobbedExchange === undefined))
              throw new Error('two calls were made and the transport recorded fewer than two exchanges',);
            expect({
              answer: bare.text,
              carriesAnswerBound: 'maxAnswerChars' in bareExchange,
              signalIsTheCallers: bareExchange.signal === SIGNAL,
            },).toEqual({
              answer: '{"spot": "windowsill"}',
              carriesAnswerBound: false,
              signalIsTheCallers: true,
            },);
            expect({
              answer: knobbed.text,
              maxAnswerChars: knobbedExchange.maxAnswerChars,
              signalIsTheCallers: knobbedExchange.signal === SIGNAL,
            },).toEqual({
              answer: '{"spot": "windowsill"}',
              maxAnswerChars: 1_000,
              signalIsTheCallers: false,
            },);
          },
        },),

        it({
          name: 'CARRIES costUsd, endpoint and cachedTokens onto the spend fields only where the wire '
            + 'sent them, dropping an unreported one',
          fn: async () => {
            expect(reportedSpendFieldsOf({
              cost: 0.5,
              endpoint: {
                reported: true,
                name: 'Inceptron',
              },
              cachedTokens: 12,
            },),).toEqual({
              costUsd: 0.5,
              endpoint: 'Inceptron',
              cachedTokens: 12,
            },);
            expect(reportedSpendFieldsOf({
              cost: COST_UNREPORTED,
              endpoint: ENDPOINT_UNREPORTED,
              cachedTokens: CACHED_UNREPORTED,
            },),).toEqual({},);
            expect(reportedSpendFieldsOf({
              cost: 0.5,
              endpoint: ENDPOINT_UNREPORTED,
              cachedTokens: CACHED_UNREPORTED,
            },),).toEqual({ costUsd: 0.5, },);
            expect(reportedSpendFieldsOf({
              cost: COST_UNREPORTED,
              endpoint: {
                reported: true,
                name: 'Inceptron',
              },
              cachedTokens: 12,
            },),).toEqual({
              endpoint: 'Inceptron',
              cachedTokens: 12,
            },);
          },
        },),
      ],
    },),

    describe({
      name: 'every billed OpenRouter attempt is reckoned (ledger P1)',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RECKONS A TRUNCATED ATTEMPT THE LADDER RETRIED beside the whole one it reported: the reckoning '
            + 'wrapped the whole ladder, so an attempt refused and retried inside it left no line',
          fn: async () => {
            resetRunSpend();
            /**
             Stream that stopped before its terminator.
             */
            const truncated = chunkOf({ delta: { content: '{"spot":', }, },);
            /**
             Exchanges the transport saw.
             */
            const exchanges: TransportExchange[] = [];
            /**
             Client whose first stream stops early and whose second is whole.
             */
            const client = createOpenRouterClient({
              apiKey: 'test-key',
              transport: async function truncatedThenWhole(exchange,) {
                exchanges.push(exchange,);
                return (exchanges.length === 1)
                  ? { status: 200, bodyText: truncated, }
                  : { status: 200, bodyText: RECORDED_STREAM, };
              },
              retryPolicy: {
                limit: 1,
                baseMs: 1,
              },
            },);
            await client.chatText({
              modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
              messages: [{ role: 'user', content: 'meow', },],
              signal: SIGNAL,
            },);
            /**
             Body the refused attempt sent.
             */
            const bodyJson = exchanges[0]?.bodyJson ?? '';
            /**
             Body as sent.
             */
            const body = JSON.parse(bodyJson,) as {
              readonly model: Parameters<typeof estimateAbandonedSpend>[0]['servedId'];
              readonly max_tokens: number;
            };
            /**
             What the refused attempt is reckoned to have cost.
             */
            const reckoned = estimateAbandonedSpend({
              servedId: body.model,
              deliveredChars: truncated.length,
              requestBodyBytes: Buffer.byteLength(bodyJson,),
              maxTokens: body.max_tokens,
            },);
            expect(runSpendUsd({ provider: 'openrouter', },),).toBeCloseTo(RECORDED_COST_USD + reckoned.usd, 12,);
          },
        },),
      ],
    },),

    describe({
      name: 'the run meter moves only where the wire reported a cost (ledger P1)',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'MOVES the run meter by the cost the wire reported and never by a stream that reported '
            + 'none, so a total reads only what the gateway charged',
          fn: async () => {
            resetRunSpend();
            /**
             Stream whose usage block reports no cost.
             */
            const bare = [
              chunkOf({
                delta: {
                  content: '{"spot": "windowsill"}',
                },
              },),
              chunkOf({
                delta: {
                  content: '',
                },
                rest: {
                  usage: {
                    prompt_tokens: 342,
                    completion_tokens: 400,
                    total_tokens: 742,
                  },
                },
              },),
              'data: [DONE]\n\n',
            ].join('',);
            await recordedClient({
              reply: {
                status: 200,
                bodyText: bare,
              },
            },).client.chatText({
              modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
              messages: [
                {
                  role: 'user',
                  content: 'Where does the cat sleep?',
                },
              ],
              signal: SIGNAL,
            },);
            // Meter the uncosted stream left flat. The costed stream's read
            // is this case's positive control, proving the meter can move
            // at all.
            expect(runSpendUsd({ provider: 'openrouter', },),).toBe(0,);
            /**
             Stream whose usage block reports a cost.
             */
            const reporting = [
              chunkOf({
                delta: {
                  content: '{"spot": "windowsill"}',
                },
              },),
              chunkOf({
                delta: {
                  content: '',
                },
                rest: {
                  usage: {
                    prompt_tokens: 342,
                    completion_tokens: 400,
                    total_tokens: 742,
                    cost: 0.5,
                    is_byok: false,
                  },
                },
              },),
              'data: [DONE]\n\n',
            ].join('',);
            await recordedClient({
              reply: {
                status: 200,
                bodyText: reporting,
              },
            },).client.chatText({
              modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
              messages: [
                {
                  role: 'user',
                  content: 'Where does the cat sleep?',
                },
              ],
              signal: SIGNAL,
            },);
            expect(runSpendUsd({ provider: 'openrouter', },),).toBeCloseTo(0.5, 12,);
          },
        },),
      ],
    },),

    describe({
      name: 'a completed call writes what the wire reported onto its spend line',
      // ONE CASE AT A TIME: the case diverts `console.info`, a process global.
      concurrency: 1,
      children: [
        it({
          name: 'WRITES the cost, the endpoint and the cached prompt tokens the stream reported onto the '
            + 'spend line of a completed call',
          fn: async () => {
            /**
             Stream whose last chunk reports a cost and cached prompt tokens,
             every chunk naming the endpoint that served it.
             */
            const reporting = [
              chunkOf({
                delta: {
                  content: '{"spot": "windowsill"}',
                },
              },),
              chunkOf({
                delta: {
                  content: '',
                },
                rest: {
                  usage: {
                    prompt_tokens: 342,
                    completion_tokens: 400,
                    total_tokens: 742,
                    cost: 0.5,
                    is_byok: false,
                    prompt_tokens_details: {
                      cached_tokens: 128,
                    },
                  },
                },
              },),
              'data: [DONE]\n\n',
            ].join('',);
            const { client, } = recordedClient({
              reply: {
                status: 200,
                bodyText: reporting,
              },
            },);

            /**
             Lines `console.info` received.
             */
            const lines: string[] = [];
            /**
             `console.info` as it was, put back once the call returns.
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
              await client.chatText({
                modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
                messages: [
                  {
                    role: 'user',
                    content: 'Where does the cat sleep?',
                  },
                ],
                signal: SIGNAL,
              },);
            }

            /**
             Each spend line from its marker on, the logger's own prefix
             (level, clock time, tags) left off.
             */
            const spent = lines
              .filter(function isSpend(line,): boolean {
                return line.includes(' SPEND ',);
              },)
              .map(function fromMarker(line,): string {
                return line.slice(line.indexOf('SPEND ',),);
              },);
            expect(spent,).toEqual([
              `SPEND provider=openrouter model=${SEAT_HYPER_OPENROUTER_UNMEASURED_OPENROUTER_ID} prompt=342 completion=400 cost=0.5 `
                + 'endpoint=Inceptron cached=128',
            ],);
          },
        },),
      ],
    },),
  ],
},);
