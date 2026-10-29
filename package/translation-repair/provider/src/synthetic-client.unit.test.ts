/**
 Tests for the injected-transport Synthetic client:
 request construction, contract enforcement, outcome-as-data JSON handling,
 and per-model concurrency bounds.

 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  MalformedCompletionError,
  stripCodeFence,
  stripThinkBlock,
  ProviderHttpError,
  type TransportExchange,
  type TransportReply,
} from '@monochromatic-dev/translation-repair-provider-stream';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  COMPLETION_CAP,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  createSyntheticClient,
  SYNTHETIC_PER_MODEL_CONCURRENCY,
  SyntheticModelNotServedError,
  SyntheticRequestTooLargeError,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { recordedTransport, } from '@monochromatic-dev/translation-repair-provider-stream/ts/recorded-transport.test-fixture.ts';

/**
 Milliseconds granted for queued microtasks and limiter slots to settle.
 */
const SETTLE_MS = 10;

/**
 How many same-model calls the default-width case holds in flight at once: more than any width Synthetic once granted.
 */
const CONCURRENT_CALLS = 12;

/**
 How long the first of two same-model calls holds the one slot, so the second
 waits in the queue past its own deadline.
 */
const QUEUE_HOLD_MS = 300;

/**
 The queued call's deadline: shorter than its wait in the queue, so the call
 survives only when the deadline leaves queue wait out, and longer than
 nothing that follows the slot opening, since that exchange answers at once.
 */
const QUEUED_DEADLINE_MS = 100;

/**
 Single user message reused across exchanges.
 */
const MESSAGES = [
  {
    role: 'user' as const,
    content: '猫猫的翻译对吗？',
  },
];

/**
 Builds a drained SSE body from content deltas plus optional refusal deltas
 and usage, terminated like the provider terminates streams.

 @param deltas - content deltas in arrival order

 @param refusalDeltas - refusal deltas in arrival order

 @param usage - usage block delivered as the final data event

 @returns Whole `text/event-stream` body as the transport drains it

 @example
 ```ts
 const body = sseBody({ deltas: ['{"a":', '1}',], },);
 ```
 */
function sseBody(
  {
    deltas,
    refusalDeltas = [],
    usage,
  }: {
    readonly deltas: readonly string[];
    readonly refusalDeltas?: readonly string[];
    readonly usage?: {
      readonly prompt_tokens: number;
      readonly completion_tokens: number;
    };
  },
): string {
  /**
   Serialized data events in stream order.
   */
  const events = [
    ...deltas.map(function toContentEvent(delta,) {
      return JSON.stringify({ choices: [{ delta: { content: delta, }, },], },);
    },),
    ...refusalDeltas.map(function toRefusalEvent(delta,) {
      return JSON.stringify({ choices: [{ delta: { refusal: delta, }, },], },);
    },),
    ...(usage === undefined
      ? []
      : [JSON.stringify({ choices: [], usage, },),]),
  ];
  return [
    ...events.map(function toDataLine(event,) {
      return `data: ${event}`;
    },),
    'data: [DONE]',
    '',
  ].join('\n\n',);
}

/**
 Recorded streamed completion carrying JSON content split across deltas
 plus usage.
 */
const COMPLETION_BODY = sseBody({
  deltas: [
    '{"verdict":',
    '"pass"}',
  ],
  usage: {
    prompt_tokens: 12,
    completion_tokens: 3,
  },
},);

/**
 Recorded completion cut off before the `[DONE]` sentinel, which is how a
 stream the provider truncated arrives: HTTP 200 carrying half an answer.
 */
const CUT_COMPLETION_BODY = 'data: {"choices":[{"delta":{"content":"{\\"verdict\\":"}}]}\n\n';

/**
 Verdict shape the chatJson tests validate against.
 */
type CatVerdict = { readonly verdict: string; };

/**
 Guards parsed model JSON as a verdict.

 @param value - parsed candidate

 @returns Whether value carries a string verdict

 @example
 ```ts
 isCatVerdict({ verdict: 'pass', },);
 ```
 */
function isCatVerdict(value: unknown,): value is CatVerdict {
  return isJsonRecord(value,) && ((typeof value.verdict) === 'string');
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: stripCodeFence.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'unwraps fenced JSON with a language tag',
          fn: async () => {
            expect(stripCodeFence({ text: '```json\n{"a":1}\n```', },),).toBe('{"a":1}',);
          },
        },),

        it({
          name: 'returns unfenced and unterminated content trimmed',
          fn: async () => {
            expect(stripCodeFence({ text: '  {"a":1}\n', },),).toBe('{"a":1}',);
            expect(stripCodeFence({ text: '```json\n{"a":1}', },),).toBe('```json\n{"a":1}',);
          },
        },),
      ],
    },),

    describe({
      name: stripThinkBlock.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'splits the answer off an embedded thinking block',
          fn: async () => {
            expect(stripThinkBlock({ text: '<think>猫喜欢晒太阳吗？是的。</think>\n{"a":1}', },),)
              .toEqual({ answer: '\n{"a":1}', truncatedThinking: false, },);
          },
        },),

        it({
          name: 'passes content without a thinking block through untouched',
          fn: async () => {
            expect(stripThinkBlock({ text: '{"a":1}', },),)
              .toEqual({ answer: '{"a":1}', truncatedThinking: false, },);
          },
        },),

        it({
          name: 'reports truncation when thinking never closes',
          fn: async () => {
            expect(stripThinkBlock({ text: '<think>还在想猫的事情', },),)
              .toEqual({ answer: '', truncatedThinking: true, },);
          },
        },),
      ],
    },),

    describe({
      name: createSyntheticClient.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'sends auth, model, and knobs, returning content and usage',
          fn: async () => {
            /** Transport replaying one recorded completion. */
            const { transport, exchanges, } = recordedTransport({
              replies: [{ status: 200, bodyText: COMPLETION_BODY, },],
            },);
            /** Client under test with injected transport. */
            const client = createSyntheticClient({
              apiKey: 'test-key',
              transport,
            },);
            /** A caller ceiling under the seat's cap, so the body must carry the caller's. */
            const requested = Math.floor(COMPLETION_CAP[SEAT_SYNTHETIC_TEXT_EVERYWHERE] / 2,);
            /** Reply of one exchange with every knob set. */
            const reply = await client.chatText({
              modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
              messages: MESSAGES,
              signal: new AbortController().signal,
              maxTokens: requested,
              responseFormat: {
                type: 'json_schema',
                json_schema: { name: 'cat_verdict', schema: { type: 'object', }, },
              },
            },);

            expect(reply.text,).toBe('{"verdict":"pass"}',);
            expect(reply.usage,).toEqual({ prompt_tokens: 12, completion_tokens: 3, },);
            expect(exchanges,).toHaveLength(1,);
            expect(exchanges[0]?.url,)
              .toBe('https://api.synthetic.new/openai/v1/chat/completions',);
            expect(exchanges[0]?.headers.Authorization,).toBe('Bearer test-key',);

            /** Request body decoded for knob assertions. */
            const body: unknown = JSON.parse(exchanges[0]?.bodyJson ?? '{}',);
            expect(isJsonRecord(body,) ? body.model : '',).toBe(SEAT_SYNTHETIC_TEXT_EVERYWHERE,);
            expect(isJsonRecord(body,) ? body.stream : false,).toBe(true,);
            expect(isJsonRecord(body,) ? body.max_tokens : 0,).toBe(requested,);
            // The serving stack does not honor sampling knobs reliably;
            // no call may carry one.
            expect(isJsonRecord(body,) && ('temperature' in body),).toBe(false,);

            /** Structured-output block decoded from the request body. */
            const responseFormat = isJsonRecord(body,)
              ? body.response_format
              : '';
            expect(
              isJsonRecord(responseFormat,)
                ? responseFormat.type
                : '',
            ).toBe('json_schema',);
          },
        },),

        it({
          name: 'omits usage when the server omits or mistypes it',
          fn: async () => {
            /** Streamed body without a usage event. */
            const bare = sseBody({ deltas: ['喵',], },);
            /** Streamed body whose usage counts are strings. */
            const mistyped = 'data: {"choices":[{"delta":{"content":"喵"}}]}\n\n'
              + 'data: {"choices":[],"usage":{"prompt_tokens":"12","completion_tokens":"3"}}\n\n'
              + 'data: [DONE]\n';
            /** Transport replaying both defective-usage bodies. */
            const { transport, } = recordedTransport({
              replies: [
                { status: 200, bodyText: bare, },
                { status: 200, bodyText: mistyped, },
              ],
            },);
            /** Client under test. */
            const client = createSyntheticClient({ apiKey: 'test-key', transport, },);

            expect(
              (await client.chatText({
                modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
                messages: MESSAGES,
                signal: new AbortController().signal,
              },)).usage,
            ).toBe(undefined,);
            expect(
              (await client.chatText({
                modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
                messages: MESSAGES,
                signal: new AbortController().signal,
              },)).usage,
            ).toBe(undefined,);
          },
        },),

        it({
          name: 'REFUSES a stream that ended without its terminator, after spending every attempt',
          fn: async () => {
            /** Transport replaying a body cut off before `[DONE]`. */
            const { transport, exchanges, } = recordedTransport({
              replies: [{ status: 200, bodyText: CUT_COMPLETION_BODY, },],
            },);
            /** Client under test, on a tiny test backoff. */
            const client = createSyntheticClient({
              apiKey: 'test-key',
              transport,
              retryPolicy: { limit: 2, baseMs: 1, },
            },);
            /** Failure the truncated stream produced. */
            let caught: unknown;

            try {
              await client.chatText({
                modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
                messages: MESSAGES,
                signal: new AbortController().signal,
              },);
            }
            catch (error) {
              caught = error;
            }

            expect(caught instanceof MalformedCompletionError,).toBe(true,);
            // THE FIRST ATTEMPT PLUS BOTH RETRIES. A provider that cuts a stream
            // still answers HTTP 200, so this count is what the verify wiring changed: the
            // ladder used to return after one call, and the voice was simply lost.
            expect(exchanges.length,).toBe(3,);
          },
        },),

        it({
          name: 'ACCEPTS the retry when only the first attempt was cut short',
          fn: async () => {
            /** Transport cutting the first stream and completing the second. */
            const { transport, exchanges, } = recordedTransport({
              replies: [
                { status: 200, bodyText: CUT_COMPLETION_BODY, },
                { status: 200, bodyText: COMPLETION_BODY, },
              ],
            },);
            /** Client under test, on a tiny test backoff. */
            const client = createSyntheticClient({
              apiKey: 'test-key',
              transport,
              retryPolicy: { limit: 2, baseMs: 1, },
            },);

            /** Answer the second attempt carried. */
            const reply = await client.chatText({
              modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
              messages: MESSAGES,
              signal: new AbortController().signal,
            },);

            expect(reply.text,).toBe('{"verdict":"pass"}',);
            // EXACTLY TWO: the cut one and the whole one. A third would say a whole
            // body is being retried too, which would double the cost of every call.
            expect(exchanges.length,).toBe(2,);
          },
        },),

        it({
          name: 'throws ProviderHttpError once transient retries exhaust',
          fn: async () => {
            /** Transport replaying an endless throttle; retries then throws. */
            const { transport, exchanges, } = recordedTransport({
              replies: [{ status: 429, bodyText: '{"error":"slow down"}', },],
            },);
            /** Client under test, on a tiny test backoff. */
            const client = createSyntheticClient({
              apiKey: 'test-key',
              transport,
              retryPolicy: { limit: 2, baseMs: 20, },
            },);
            /** Value caught from the throttled exchange. */
            let caught: unknown;
            try {
              await client.chatText({
                modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                messages: MESSAGES,
                signal: new AbortController().signal,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught instanceof ProviderHttpError,).toBe(true,);
            expect(
              caught instanceof ProviderHttpError
                ? caught.status
                : 0,
            ).toBe(429,);
            expect(
              caught instanceof ProviderHttpError
                ? caught.provider
                : undefined,
            ).toBe('synthetic',);
            // First attempt plus two transient retries.
            expect(exchanges,).toHaveLength(3,);
          },
        },),

        it({
          name: 'MEASURES THE BODY IN BYTES, NOT CHARACTERS, when re-raising the gateway\'s size '
            + 'refusal. The content here is Chinese, so its character count sits comfortably under '
            + 'the passing size while its UTF-8 weight is a third again over it. A client counting '
            + '`.length` would call this small, hand back the gateway\'s parse error, and never once '
            + 'fire on the only corpus this pipeline actually translates',
          fn: async () => {
            /**
             Content whose characters number far fewer than its bytes.

             3600000 characters at three UTF-8 bytes each is 10800000 bytes, over the
             10485760 measured to pass, while the character count is barely a third
             of it. The two readings disagree by design.
             */
            const wide = '猫'.repeat(3_600_000,);

            /** Transport replaying the gateway\'s answer to an oversize body. */
            const { transport, } = recordedTransport({
              replies: [{
                status: 400,
                bodyText: 'Could not parse request as valid JSON. '
                  + 'Unterminated string in JSON at position 10444203',
              },],
            },);

            /** Client under test. */
            const client = createSyntheticClient({ apiKey: 'test-key', transport, },);

            /** Value caught from the oversize exchange. */
            let caught: unknown;
            try {
              await client.chatText({
                modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                messages: [{ role: 'user' as const, content: wide, },],
                signal: new AbortController().signal,
              },);
            }
            catch (error) {
              caught = error;
            }

            expect(caught instanceof SyntheticRequestTooLargeError,).toBe(true,);
            if (!(caught instanceof SyntheticRequestTooLargeError))
              throw new Error('size failure by construction',);
            expect(caught.bodyBytes > caught.passingBodyBytes,).toBe(true,);
            expect(caught.provider,).toBe('synthetic',);
            // The character count is what a `.length` reading would have seen, and it
            // is under the cap: this is the gap the case exists to hold open.
            expect(wide.length < caught.passingBodyBytes,).toBe(true,);
          },
        },),

        it({
          name: 'LEAVES A SMALL MALFORMED BODY AS AN ORDINARY ERROR, so a request we genuinely '
            + 'broke still reports as broken. Without this, every parse error would be re-raised as '
            + 'a size problem and whoever chased one would go hunting for a limit they never hit',
          fn: async () => {
            /** Transport replaying a parse failure on a small body. */
            const { transport, } = recordedTransport({
              replies: [{
                status: 400,
                bodyText: 'Could not parse request as valid JSON. '
                  + 'Unterminated string in JSON at position 41',
              },],
            },);

            /** Client under test. */
            const client = createSyntheticClient({ apiKey: 'test-key', transport, },);

            /** Value caught from the refused exchange. */
            let caught: unknown;
            try {
              await client.chatText({
                modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                messages: MESSAGES,
                signal: new AbortController().signal,
              },);
            }
            catch (error) {
              caught = error;
            }

            expect(caught instanceof ProviderHttpError,).toBe(true,);
            expect(caught instanceof SyntheticRequestTooLargeError,).toBe(false,);
          },
        },),

        it({
          name: 'retries transient 502s and succeeds on a later attempt',
          fn: async () => {
            /** Transport failing once with 502, then succeeding. */
            const { transport, exchanges, } = recordedTransport({
              replies: [
                { status: 502, bodyText: 'bad gateway', },
                { status: 200, bodyText: COMPLETION_BODY, },
              ],
            },);
            /** Client under test, on a tiny test backoff. */
            const client = createSyntheticClient({
              apiKey: 'test-key',
              transport,
              retryPolicy: { limit: 2, baseMs: 20, },
            },);
            /** Reply that survived one transient failure. */
            const reply = await client.chatText({
              modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              messages: MESSAGES,
              signal: new AbortController().signal,
            },);
            expect(reply.text,).toBe('{"verdict":"pass"}',);
            expect(exchanges,).toHaveLength(2,);
          },
        },),

        it({
          name: 'does not retry non-transient failures',
          fn: async () => {
            /** Transport replaying a permanent client error. */
            const { transport, exchanges, } = recordedTransport({
              replies: [{ status: 400, bodyText: '{"error":"bad request"}', },],
            },);
            /** Client under test. */
            const client = createSyntheticClient({ apiKey: 'test-key', transport, },);
            /** Value caught from the rejected exchange. */
            let caught: unknown;
            try {
              await client.chatText({
                modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                messages: MESSAGES,
                signal: new AbortController().signal,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(
              caught instanceof ProviderHttpError
                ? caught.status
                : 0,
            ).toBe(400,);
            expect(exchanges,).toHaveLength(1,);
          },
        },),

        it({
          name: 'arms the exchange deadline inside the slot, not at dispatch',
          fn: async () => {
            /**
             Exchanges the transport has served.
             */
            const served = { count: 0, };

            /**
             Transport holding the first exchange, answering every later one
             at once, and refusing an exchange whose signal has aborted.

             @param exchange - request under attempt

             @returns Success reply, after the hold for the first exchange

             @throws The exchange signal's abort reason, when it has aborted

             @example
             ```ts
             await holdFirstReply(exchange,);
             ```
             */
            async function holdFirstReply(
              exchange: ForeignBorrowed<TransportExchange>,
            ): Promise<TransportReply> {
              served.count += 1;
              // THE FIRST CALL HOLDS THE SLOT past the second call's deadline,
              // and the second answers at once: once its slot opens, no timer
              // can end before its reply on any machine. Each call slept 150 ms
              // against a deadline of 220 ms here, with a stall of 70 ms between
              // the two enough to reverse them.
              if (served.count === 1)
                await wait(QUEUE_HOLD_MS,);
              // REFUSES AN ABORTED EXCHANGE, as a fetch on its signal would, so a
              // deadline that counted the queue, and fired while the second call
              // waited there, fails the call. The sleeping transport here ignored
              // its signal, and the case passed whichever way the deadline was
              // armed.
              if (exchange.signal.aborted)
                throw exchange.signal.reason;
              return { status: 200, bodyText: COMPLETION_BODY, };
            }

            /** Client with one slot so the second call queues locally. */
            const client = createSyntheticClient({
              apiKey: 'test-key',
              transport: holdFirstReply,
              perModelConcurrency: 1,
            },);
            /**
             Two same-model calls race for one slot; the second waits in the
             queue longer than its own deadline, then answers at once. It only
             survives when the deadline excludes queue wait.
             */
            const replies = await Promise.all([
              client.chatText({
                modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                messages: MESSAGES,
                signal: new AbortController().signal,
              },),
              client.chatText({
                modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                messages: MESSAGES,
                signal: new AbortController().signal,
                exchangeTimeoutMs: QUEUED_DEADLINE_MS,
              },),
            ],);
            expect(replies[0]?.text,).toBe('{"verdict":"pass"}',);
            expect(replies[1]?.text,).toBe('{"verdict":"pass"}',);
          },
        },),

        it({
          name: 'forfeits a hung exchange to its deadline',
          fn: async () => {
            /**
             Transport that never answers, rejecting only on abort.

             @param exchange - request left hanging

             @returns Never resolves; rejects with the abort reason

             @example
             ```ts
             await hangForever(exchange,);
             ```
             */
            async function hangForever(
              exchange: ForeignBorrowed<TransportExchange>,
            ): Promise<TransportReply> {
              /** Gate rejected by the exchange signal's abort reason. */
              const gate = Promise.withResolvers<TransportReply>();
              exchange.signal.addEventListener(
                'abort',
                function onAbort() {
                  gate.reject(exchange.signal.reason,);
                },
              );
              return gate.promise;
            }

            /** Client under test. */
            const client = createSyntheticClient({
              apiKey: 'test-key',
              transport: hangForever,
            },);
            /** Value caught from the forfeited exchange. */
            let caught: unknown;
            try {
              await client.chatText({
                modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                messages: MESSAGES,
                signal: new AbortController().signal,
                exchangeTimeoutMs: 50,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(String(caught,),).toContain('Timeout',);
            expect(String(caught,),).toContain(SEAT_HYPER_OPENROUTER_VISION_EDITOR,);
          },
        },),

        it({
          name: 'retries thrown transport failures and succeeds later',
          fn: async () => {
            /** Every exchange the client attempted, in order. */
            const exchanges: TransportExchange[] = [];

            /**
             Transport dropping its first exchange like a mid-stream reset.

             @param exchange - request under attempt

             @returns Success reply from the second attempt on

             @example
             ```ts
             await dropOnce(exchange,);
             ```
             */
            async function dropOnce(exchange: TransportExchange,): Promise<TransportReply> {
              exchanges.push(exchange,);
              if (exchanges.length === 1)
                throw new TypeError('fetch failed: connection reset',);
              return { status: 200, bodyText: COMPLETION_BODY, };
            }

            /** Client under test, on a tiny test backoff. */
            const client = createSyntheticClient({
              apiKey: 'test-key',
              transport: dropOnce,
              retryPolicy: { limit: 2, baseMs: 20, },
            },);
            /** Reply that survived one dropped connection. */
            const reply = await client.chatText({
              modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              messages: MESSAGES,
              signal: new AbortController().signal,
            },);
            expect(reply.text,).toBe('{"verdict":"pass"}',);
            expect(exchanges,).toHaveLength(2,);
          },
        },),

        it({
          name: 'rethrows thrown transport failures once retries exhaust',
          fn: async () => {
            /** Every exchange the client attempted, in order. */
            const exchanges: TransportExchange[] = [];
            /** Failure thrown on every attempt; identity must survive. */
            const failure = new TypeError('fetch failed: connection reset',);

            /**
             Transport dropping every exchange.

             @param exchange - request under attempt

             @returns Never; every attempt throws

             @example
             ```ts
             await dropAlways(exchange,);
             ```
             */
            async function dropAlways(exchange: TransportExchange,): Promise<TransportReply> {
              exchanges.push(exchange,);
              throw failure;
            }

            /** Client under test, on a tiny test backoff. */
            const client = createSyntheticClient({
              apiKey: 'test-key',
              transport: dropAlways,
              retryPolicy: { limit: 2, baseMs: 20, },
            },);
            /** Value caught once retries exhausted. */
            let caught: unknown;
            try {
              await client.chatText({
                modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                messages: MESSAGES,
                signal: new AbortController().signal,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBe(failure,);
            // First attempt plus two transient retries.
            expect(exchanges,).toHaveLength(3,);
          },
        },),

        it({
          name: 'propagates thrown failures untouched after caller abort',
          fn: async () => {
            /** Every exchange the client attempted, in order. */
            const exchanges: TransportExchange[] = [];
            /** Failure the aborted stream throws; identity must survive. */
            const failure = new Error('stream torn down by abort',);

            /**
             Transport whose exchange dies under an aborted signal.

             @param exchange - request under attempt

             @returns Never; the aborted stream always throws

             @example
             ```ts
             await tornDown(exchange,);
             ```
             */
            async function tornDown(exchange: TransportExchange,): Promise<TransportReply> {
              exchanges.push(exchange,);
              throw failure;
            }

            /** Client under test. */
            const client = createSyntheticClient({
              apiKey: 'test-key',
              transport: tornDown,
              retryPolicy: { limit: 2, baseMs: 20, },
            },);
            /** Controller aborted before the exchange settles. */
            const aborted = new AbortController();
            aborted.abort(new Error('user stop',),);
            /** Value caught from the aborted exchange. */
            let caught: unknown;
            try {
              await client.chatText({
                modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                messages: MESSAGES,
                signal: aborted.signal,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBe(failure,);
            // Abort means steering: no retry may follow.
            expect(exchanges,).toHaveLength(1,);
          },
        },),

        it({
          name: 'throws MalformedCompletionError on contract-violating success bodies',
          fn: async () => {
            /** Transport replaying a success status without choices. */
            const { transport, } = recordedTransport({
              replies: [{ status: 200, bodyText: '{"unexpected":true}', },],
            },);
            /**
             Client under test, on a tiny test backoff: a malformed body is
             retried, and the production backoff slept 12.2 s here (ledger T5).
             */
            const client = createSyntheticClient({
              apiKey: 'test-key',
              transport,
              retryPolicy: { limit: 2, baseMs: 1, },
            },);
            /** Value caught from the malformed exchange. */
            let caught: unknown;
            try {
              await client.chatText({
                modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                messages: MESSAGES,
                signal: new AbortController().signal,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught instanceof MalformedCompletionError,).toBe(true,);
          },
        },),

        it({
          name: 'passes the caller signal through to the transport untouched',
          fn: async () => {
            /** Transport recording the exchange for signal identity. */
            const { transport, exchanges, } = recordedTransport({
              replies: [{ status: 200, bodyText: COMPLETION_BODY, },],
            },);
            /** Client under test. */
            const client = createSyntheticClient({ apiKey: 'test-key', transport, },);
            /** Controller whose signal identity must survive the plumbing. */
            const controller = new AbortController();
            await client.chatText({
              modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              messages: MESSAGES,
              signal: controller.signal,
            },);
            expect(exchanges[0]?.signal,).toBe(controller.signal,);
          },
        },),

        it({
          name: 'GRANTS NO DEFAULT SAME-MODEL CEILING, so every concurrent call to one model reaches the transport at once, beside the '
            + 'cross-model call',
          fn: async () => {
            /** Gate holding every transport call open until released. */
            const gate = Promise.withResolvers<void>();
            /** Models observed entering the transport, in order. */
            const entered: string[] = [];

            /**
             Transport that records entry then waits for the gate.

             @param exchange - request whose model gets recorded

             @returns Recorded completion once the gate opens

             @example
             ```ts
             const client = createSyntheticClient({ apiKey: 'test-key', transport: gatedTransport, },);
             ```
             */
            async function gatedTransport(
              exchange: TransportExchange,
            ): Promise<TransportReply> {
              /** Request body decoded to name the entering model. */
              const body: unknown = JSON.parse(exchange.bodyJson ?? '{}',);
              /** Seat this call entered as, which names its cap. */
              const enteredModel = String(isJsonRecord(body,) ? body.model : 'unknown',);
              entered.push(enteredModel,);
              // THE MEASURED CEILING RIDES ON EVERY CALL since 2026-09-09, with
              // no caller asking for one: the weekly token allowance lasts longer
              // when a runaway reply cannot spend it.
              expect(isJsonRecord(body,) ? body.max_tokens : 0,).toBe(COMPLETION_CAP[enteredModel as RosterModelId],);
              await gate.promise;
              return { status: 200, bodyText: COMPLETION_BODY, };
            }
            expect(SYNTHETIC_PER_MODEL_CONCURRENCY,).toBe(Number.POSITIVE_INFINITY,);

            /** Client under test. */
            const client = createSyntheticClient({
              apiKey: 'test-key',
              transport: gatedTransport,
            },);

            /** Many same-model calls, where Synthetic once admitted five. */
            const sameModelCalls = Array.from(
              { length: CONCURRENT_CALLS, },
              function callModel() {
                return client.chatText({
                  modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
                  messages: MESSAGES,
                  signal: new AbortController().signal,
                },);
              },
            );
            /** Cross-model call proving slots are keyed by model identity. */
            const crossModelCall = client.chatText({
              modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              messages: MESSAGES,
              signal: new AbortController().signal,
            },);
            await wait(SETTLE_MS,);

            expect(entered.filter(function gpt(id,) {
              return id === SEAT_SYNTHETIC_TEXT_EVERYWHERE;
            },),).toHaveLength(CONCURRENT_CALLS,);
            expect(entered.filter(function glm(id,) {
              return id === SEAT_HYPER_OPENROUTER_VISION_EDITOR;
            },),).toHaveLength(1,);

            gate.resolve();
            await Promise.all([...sameModelCalls, crossModelCall,],);
            expect(entered,).toHaveLength(CONCURRENT_CALLS + 1,);
          },
        },),

        it({
          name: 'widens same-model concurrency to perModelConcurrency slots',
          fn: async () => {
            /** Gate holding every transport call open until released. */
            const gate = Promise.withResolvers<void>();
            /** Count of calls observed entering the transport. */
            const entered: string[] = [];

            /**
             Transport that records entry then waits for the gate.

             @param exchange - request whose model gets recorded

             @returns Recorded completion once the gate opens

             @example
             ```ts
             const client = createSyntheticClient({ apiKey: 'test-key', transport: gatedTransport, },);
             ```
             */
            async function gatedTransport(
              exchange: TransportExchange,
            ): Promise<TransportReply> {
              /** Request body decoded to name the entering model. */
              const body: unknown = JSON.parse(exchange.bodyJson ?? '{}',);
              entered.push(String(isJsonRecord(body,) ? body.model : 'unknown',),);
              await gate.promise;
              return { status: 200, bodyText: COMPLETION_BODY, };
            }
            /** Client granted two slots per model, like a two-pack account. */
            const client = createSyntheticClient({
              apiKey: 'test-key',
              transport: gatedTransport,
              perModelConcurrency: 2,
            },);

            /** Three same-model calls in flight against two slots. */
            const inFlight = [
              client.chatText({
                modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
                messages: MESSAGES,
                signal: new AbortController().signal,
              },),
              client.chatText({
                modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
                messages: MESSAGES,
                signal: new AbortController().signal,
              },),
              client.chatText({
                modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
                messages: MESSAGES,
                signal: new AbortController().signal,
              },),
            ];
            await wait(SETTLE_MS,);

            // Two slots admit two calls at once; the third queues until release.
            expect(entered,).toHaveLength(2,);

            gate.resolve();
            await Promise.all(inFlight,);
            expect(entered,).toHaveLength(3,);
          },
        },),

        it({
          name: 'returns ok with typed value for fenced schema-valid content',
          fn: async () => {
            /** Streamed body whose content wraps valid JSON in a code fence. */
            const fenced = sseBody({ deltas: ['```json\n{"verdict":"pass"}\n```',], },);
            /** Transport replaying the fenced completion. */
            const { transport, } = recordedTransport({
              replies: [{ status: 200, bodyText: fenced, },],
            },);
            /** Client under test. */
            const client = createSyntheticClient({ apiKey: 'test-key', transport, },);
            /** Outcome of the schema-validated exchange. */
            const outcome = await client.chatJson({
              modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
              messages: MESSAGES,
              signal: new AbortController().signal,
              validate: isCatVerdict,
            },);
            expect(outcome.kind,).toBe('ok',);
            expect(
              outcome.kind === 'ok'
                ? outcome.value.verdict
                : '',
            ).toBe('pass',);
          },
        },),

        it({
          name: 'admits think-wrapped JSON and ignores refusal phrasing inside thinking',
          fn: async () => {
            /** Streamed body deliberating about refusing, then answering validly. */
            const deliberated = sseBody({
              deltas: [
                '<think>Should I say i cannot help? No, the request is benign.</think>\n',
                '```json\n{"verdict":"pass"}\n```',
              ],
              usage: {
                prompt_tokens: 40,
                completion_tokens: 900,
              },
            },);
            /** Transport replaying the deliberated completion. */
            const { transport, } = recordedTransport({
              replies: [{ status: 200, bodyText: deliberated, },],
            },);
            /** Client under test. */
            const client = createSyntheticClient({ apiKey: 'test-key', transport, },);
            /** Outcome of the deliberated exchange. */
            const outcome = await client.chatJson({
              modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              messages: MESSAGES,
              signal: new AbortController().signal,
              validate: isCatVerdict,
            },);
            expect(outcome.kind,).toBe('ok',);
            expect(
              outcome.kind === 'ok'
                ? outcome.value.verdict
                : '',
            ).toBe('pass',);
            expect(outcome.usage,).toEqual({ prompt_tokens: 40, completion_tokens: 900, },);
          },
        },),

        it({
          name: 'reports truncated thinking as schema-mismatch naming the cause',
          fn: async () => {
            /** Streamed body whose content died inside its thinking block. */
            const truncated = sseBody({ deltas: ['<think>猫的翻译问题很复杂，首先',], },);
            /** Transport replaying the truncated completion. */
            const { transport, } = recordedTransport({
              replies: [{ status: 200, bodyText: truncated, },],
            },);
            /** Client under test. */
            const client = createSyntheticClient({ apiKey: 'test-key', transport, },);
            /** Outcome of the truncated exchange. */
            const outcome = await client.chatJson({
              modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              messages: MESSAGES,
              signal: new AbortController().signal,
              validate: isCatVerdict,
            },);
            expect(outcome.kind,).toBe('schema-mismatch',);
            expect(
              outcome.kind === 'schema-mismatch'
                ? outcome.detail
                : '',
            ).toContain('thinking',);
          },
        },),

        it({
          name: 'treats the API refusal field as first-class refusal',
          fn: async () => {
            /** Streamed body refusing through refusal deltas with no content. */
            const refusedByApi = sseBody({
              deltas: [],
              refusalDeltas: ['Request declined ', 'by policy.',],
            },);
            /** Transport replaying the API-refused completion. */
            const { transport, } = recordedTransport({
              replies: [{ status: 200, bodyText: refusedByApi, },],
            },);
            /** Client under test. */
            const client = createSyntheticClient({ apiKey: 'test-key', transport, },);
            /** Outcome of the API-refused exchange. */
            const outcome = await client.chatJson({
              modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
              messages: MESSAGES,
              signal: new AbortController().signal,
              validate: isCatVerdict,
            },);
            expect(outcome.kind,).toBe('refusal-shaped',);
            expect(
              outcome.kind === 'refusal-shaped'
                ? outcome.marker
                : '',
            ).toBe('api-refusal-field',);
            expect(
              outcome.kind === 'refusal-shaped'
                ? outcome.rawText
                : '',
            ).toBe('Request declined by policy.',);
          },
        },),

        it({
          name: 'returns refusal-shaped for unparseable apologetic content',
          fn: async () => {
            /** Streamed body whose content is a prose refusal. */
            const refusal = sseBody({
              deltas: ["I'm sorry, but I can't assist with that request.",],
            },);
            /** Transport replaying the refusal completion. */
            const { transport, } = recordedTransport({
              replies: [{ status: 200, bodyText: refusal, },],
            },);
            /** Client under test. */
            const client = createSyntheticClient({ apiKey: 'test-key', transport, },);
            /** Outcome of the refused exchange. */
            const outcome = await client.chatJson({
              modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
              messages: MESSAGES,
              signal: new AbortController().signal,
              validate: isCatVerdict,
            },);
            expect(outcome.kind,).toBe('refusal-shaped',);
            // "i can't assist" precedes "i'm sorry, but" in the marker list,
            // and first marker in list order wins.
            expect(
              outcome.kind === 'refusal-shaped'
                ? outcome.marker
                : '',
            ).toBe("i can't assist",);
          },
        },),

        it({
          name: 'returns schema-mismatch for valid JSON the guard rejects',
          fn: async () => {
            /** Streamed body whose content is valid JSON of the wrong shape. */
            const wrongShape = sseBody({ deltas: ['{"verdicts":["pass"]}',], },);
            /** Transport replaying the wrong-shape completion. */
            const { transport, } = recordedTransport({
              replies: [{ status: 200, bodyText: wrongShape, },],
            },);
            /** Client under test. */
            const client = createSyntheticClient({ apiKey: 'test-key', transport, },);
            /** Outcome of the guard-rejected exchange. */
            const outcome = await client.chatJson({
              modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              messages: MESSAGES,
              signal: new AbortController().signal,
              validate: isCatVerdict,
            },);
            expect(outcome.kind,).toBe('schema-mismatch',);
          },
        },),

        it({
          name: 'returns schema-mismatch for unparseable non-refusal content',
          fn: async () => {
            /** Streamed body whose content is confident prose, not JSON and not refusal. */
            const prose = sseBody({ deltas: ['The translation is excellent overall.',], },);
            /** Transport replaying the prose completion. */
            const { transport, } = recordedTransport({
              replies: [{ status: 200, bodyText: prose, },],
            },);
            /** Client under test. */
            const client = createSyntheticClient({ apiKey: 'test-key', transport, },);
            /** Outcome of the prose exchange. */
            const outcome = await client.chatJson({
              modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              messages: MESSAGES,
              signal: new AbortController().signal,
              validate: isCatVerdict,
            },);
            expect(outcome.kind,).toBe('schema-mismatch',);
            expect(
              outcome.kind === 'schema-mismatch'
                ? outcome.detail
                : '',
            ).toContain('not valid JSON',);
          },
        },),

        it({
          name: 'reads quota snapshots through the transport',
          fn: async () => {
            /** Recorded quotas with invented numbers. */
            const quotas = {
              weeklyTokenLimit: {
                nextRegenAt: '2026-07-17T00:10:00.000Z',
                percentRemaining: 87.5,
              },
              rollingFiveHourLimit: {
                nextTickAt: '2026-07-16T22:55:00.000Z',
                tickPercent: 0.05,
                remaining: 613.4,
                max: 640,
                limited: false,
              },
            } as const;
            /** The same quotas as the wire carries them. */
            const quotasBody = JSON.stringify(quotas,);
            /** Transport replaying the quotas body. */
            const { transport, exchanges, } = recordedTransport({
              replies: [{ status: 200, bodyText: quotasBody, },],
            },);
            /** Client under test. */
            const client = createSyntheticClient({ apiKey: 'test-key', transport, },);
            /** Snapshot of the recorded quota state. */
            const snapshot = await client.quotas({ signal: new AbortController().signal, },);
            expect(snapshot.fiveHour?.remaining,).toBe(quotas.rollingFiveHourLimit.remaining,);
            expect(snapshot.weekly.percentRemaining,).toBe(quotas.weeklyTokenLimit.percentRemaining,);
            expect(exchanges[0]?.url,).toBe('https://api.synthetic.new/v2/quotas',);
            expect(exchanges[0]?.method,).toBe('GET',);
          },
        },),

        it({
          name: 'NAMES SYNTHETIC AS THE PROVIDER of the HTTP error a refused quotas read raises',
          fn: async () => {
            /** Transport replaying an unauthorized quotas read. */
            const { transport, } = recordedTransport({
              replies: [{ status: 401, bodyText: 'missing authorization', },],
            },);
            /** Client under test, with no retry wait. */
            const client = createSyntheticClient({
              apiKey: 'test-key',
              transport,
              retryPolicy: { limit: 0, baseMs: 1, },
            },);
            /** Value caught from the refused read. */
            let caught: unknown;
            try {
              await client.quotas({ signal: new AbortController().signal, },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught instanceof ProviderHttpError,).toBe(true,);
            expect(
              caught instanceof ProviderHttpError
                ? [caught.provider, caught.status,]
                : undefined,
            ).toStrictEqual(['synthetic', 401,],);
          },
        },),

        it({
          name: 'READS A QUOTAS BODY WITH NO FIVE-HOUR BLOCK into a snapshot with no five-hour window, as the live reply of '
            + '2026-10-19 came',
          fn: async () => {
            /** Recorded quotas without the rolling block, with invented numbers. */
            const quotasBody = JSON.stringify({
              subscription: { limit: 640, requests: 3, },
              search: { hourly: { limit: 200, requests: 0, }, },
              freeToolCalls: { limit: 0, requests: 0, },
              weeklyTokenLimit: {
                nextRegenAt: '2026-10-19T17:57:00.000Z',
                percentRemaining: 64.5,
              },
            },);
            /** Transport replaying the quotas body. */
            const { transport, } = recordedTransport({
              replies: [{ status: 200, bodyText: quotasBody, },],
            },);
            /** Client under test. */
            const client = createSyntheticClient({ apiKey: 'test-key', transport, },);
            /** Snapshot of the recorded quota state. */
            const snapshot = await client.quotas({ signal: new AbortController().signal, },);
            expect(snapshot,).toEqual({
              weekly: {
                percentRemaining: 64.5,
                nextRegenAt: '2026-10-19T17:57:00.000Z',
              },
            },);
            expect(Object.hasOwn(
              snapshot,
              'fiveHour',
            ),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: SyntheticModelNotServedError.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES a Charm Hyper endpoint label before the wire, so the transport never sees a request',
          fn: async () => {
            /** Transport that would answer cleanly, proving the refusal happens before any exchange. */
            const { transport, exchanges, } = recordedTransport({
              replies: [{ status: 200, bodyText: COMPLETION_BODY, },],
            },);
            /** Client under test. */
            const client = createSyntheticClient({ apiKey: 'test-key', transport, },);
            /** Value caught from the refused call. */
            let caught: unknown;
            try {
              await client.chatText({
                modelId: SEAT_HYPER_TEXT_BEDROCK,
                messages: MESSAGES,
                signal: new AbortController().signal,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(SyntheticModelNotServedError,);
            expect(
              caught instanceof SyntheticModelNotServedError
                ? caught.modelId
                : '',
            ).toBe(SEAT_HYPER_TEXT_BEDROCK,);
            expect(exchanges,).toHaveLength(0,);
          },
        },),

        it({
          name: 'REFUSES the same label on the JSON surface, which rides the text one and must not grow its own path',
          fn: async () => {
            /** Transport that would answer cleanly, proving the refusal happens before any exchange. */
            const { transport, exchanges, } = recordedTransport({
              replies: [{ status: 200, bodyText: COMPLETION_BODY, },],
            },);
            /** Client under test. */
            const client = createSyntheticClient({ apiKey: 'test-key', transport, },);
            /** Value caught from the refused call. */
            let caught: unknown;
            try {
              await client.chatJson({
                modelId: SEAT_HYPER_VISION,
                messages: MESSAGES,
                signal: new AbortController().signal,
                validate: isCatVerdict,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(SyntheticModelNotServedError,);
            expect(exchanges,).toHaveLength(0,);
          },
        },),
      ],
    },),
  ],
},);
