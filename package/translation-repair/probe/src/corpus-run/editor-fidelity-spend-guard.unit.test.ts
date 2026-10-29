/**
 Tests for the spend guard of the editor fidelity probe: what one call can cost at most, and the wrapper that refuses a
 call, and ends the arm, once the spend on the run meter plus that call's ceiling would pass `--max-usd`.

 The guard reads what the Claude client reports. Every priced reply and every attempt the client abandoned is a SPEND
 line that moves the run meter, so the cases here make their scripted clients write the meter as the client does. A call
 that throws, and a reply with no usage, wrote nothing a meter could hold, and the guard counts each at the call's
 ceiling by a SPEND line of its own, so the identity the read step checks (the sum of the arm's SPEND lines is the
 guard's total) holds by construction.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  type ChatTextReply,
  createClaudeClient,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  type ModelTransport,
  ProviderHttpError,
  type TransportExchange,
} from '@monochromatic-dev/translation-repair-provider-stream';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import {
  noteRunSpend,
  resetRunSpend,
  runSpendUsd,
  SONNET_5_5_PRICES,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { infoLinesDuring, } from '@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts';
import { anthropicFrameOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/anthropic-frames.test-fixture.ts';
import {
  CLAUDE_CAT_KEY,
  claudeStreamOf,
  PLAIN_USAGE,
} from '@monochromatic-dev/translation-repair-provider/ts/claude-client.test-fixture.ts';
import {
  firstBodyOf,
  ONE_QUICK_RETRY,
  scripted,
} from '@monochromatic-dev/translation-repair-provider/ts/direct-client.test-fixture.ts';
import { CLAUDE_KEY_VARIABLE, } from '@monochromatic-dev/translation-repair-run';
import { withFreshRunSeats, } from '@monochromatic-dev/translation-repair-stage/ts/corpus-run/fresh-run-seats.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  callCeilingUsd,
  cardsClientFrom,
  ConcurrentGuardCallError,
  guardLine,
  guardTotalLine,
  pricesOf,
  RepliedWithoutCallError,
  spendGuardedClient,
} from '../../dist/final/node/index.mjs';
import {
  HAIKU_CARD,
  LOW_CARD,
} from '../editor-fidelity.test-fixture.ts';

/**
 Usage a reply reports.
 */
type CompletionUsage = NonNullable<ChatTextReply['usage']>;

/**
 What a scripted reply reports, whose cost the scripted client puts on the meter itself: a thousand prompt tokens and
 ten thousand completion tokens.
 */
const REPORTED: CompletionUsage = {
  prompt_tokens: 1_000,
  completion_tokens: 10_000,
};

/**
 What one reported reply costs on the meter: (1000 x 2.5 + 10000 x 10) / 1e6 USD at the Sonnet prices.
 */
const REPLY_USD = 0.1025;

/**
 One user message of a thousand characters, which a Sonnet call can cost at most (1000 x 2 + 32000 x 10) / 1e6 =
 0.322 USD.
 */
const THOUSAND_CHARS = [{ role: 'user', content: 'x'.repeat(1_000,), },] as const;

/**
 What a scripted client does on each call, besides answering.

 @example
 ```ts
 const script: ClientScript = { asked: [], pricedUsd: 0.1025, reportsUsage: true, };
 ```
 */
type ClientScript = {
  /**
   Array each exchange's model id is appended to.
   */
  readonly asked: string[];

  /**
   Dollars the client puts on the meter for each reply it gives, as the Claude client's SPEND line does; absent for a
   client that writes no line.
   */
  readonly pricedUsd?: number;

  /**
   Dollars of each attempt the client abandoned before it answered, written to the meter first, in order.
   */
  readonly abandonedUsd?: readonly number[];

  /**
   Whether the reply reports its tokens.
   */
  readonly reportsUsage: boolean;
};

/**
 Puts what the script says on the meter, the way the client's SPEND lines do.

 @param script - what the client does

 @example
 ```ts
 writeMeter({ script, },);
 ```
 */
function writeMeter({ script, }: { readonly script: ClientScript; },): void {
  for (const abandoned of script.abandonedUsd ?? [])
    noteRunSpend({ provider: 'anthropic', costUsd: abandoned, },);
  if (script.pricedUsd !== undefined)
    noteRunSpend({ provider: 'anthropic', costUsd: script.pricedUsd, },);
}

/**
 Client that records the model of every exchange, writes the meter as a Claude client does, and answers.

 @param script - what the client does on each call

 @returns Inner client the guard wraps

 @example
 ```ts
 const inner = scriptedClient({ script: { asked: [], pricedUsd: REPLY_USD, reportsUsage: true, }, },);
 ```
 */
function scriptedClient({ script, }: { readonly script: ClientScript; },): ModelClient {
  return {
    chatText: (request,): Promise<ChatTextReply> => {
      script.asked.push(request.modelId,);
      writeMeter({ script, },);
      return Promise.resolve({
        text: 'meow',
        ...(script.reportsUsage ? { usage: REPORTED, } : {}),
      },);
    },
    chatJson: <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
      script.asked.push(request.modelId,);
      writeMeter({ script, },);
      return Promise.resolve({
        kind: 'ok',
        value: { edits: [], } as ValueT,
        rawText: '{"edits":[]}',
        ...(script.reportsUsage ? { usage: REPORTED, } : {}),
      },);
    },
    quotas: () => Promise.reject(new Error('quotas unused',),),
  };
}

/**
 Asks the guarded client one JSON exchange of a thousand characters.

 @param client - guarded client

 @param modelId - model named

 @param signal - arm's signal

 @returns Outcome the client gives

 @example
 ```ts
 await askJson({ client, modelId: LOW_CARD, signal, },);
 ```
 */
function askJson(
  {
    client,
    modelId,
    signal,
  }: {
    readonly client: ModelClient;
    readonly modelId: ChatJsonRequest<unknown>['modelId'];
    readonly signal: AbortSignal;
  },
): Promise<ChatJsonOutcome<unknown>> {
  return client.chatJson({
    modelId,
    messages: THOUSAND_CHARS,
    signal,
    validate: (value: unknown,): value is unknown => value !== undefined,
  },);
}

/**
 Builds the exchange that asks the guarded client one text exchange of a thousand characters.

 @param client - guarded client

 @param controller - arm's own

 @returns Exchange, which settles with the reply the client gives

 @example
 ```ts
 const second = await raisedBy({ ask: chatTextAsker({ client, controller, }), },);
 ```
 */
function chatTextAsker(
  {
    client,
    controller,
  }: {
    readonly client: ModelClient;
    readonly controller: AbortController;
  },
): () => Promise<unknown> {
  return function askText(): Promise<unknown> {
    return client.chatText({
      modelId: LOW_CARD,
      messages: THOUSAND_CHARS,
      signal: controller.signal,
    },);
  };
}

/**
 What asking raised, caught so a case can read it.

 @param ask - exchange

 @returns Raised value, or undefined when the exchange completed

 @example
 ```ts
 const raised = await raisedBy({ ask: () => askJson({ client, modelId, signal, },), },);
 ```
 */
async function raisedBy({ ask, }: { readonly ask: () => Promise<unknown>; },): Promise<unknown> {
  try {
    await ask();
  }
  catch (error) {
    return error;
  }
  return undefined;
}

/**
 Client over which every call fails with the error given.

 @param asked - array each exchange's model id is appended to

 @param failure - what each exchange throws

 @returns Client that answers nothing

 @example
 ```ts
 const inner = failingClient({ asked: [], failure: new Error('cut',), },);
 ```
 */
function failingClient({ asked, failure, }: { readonly asked: string[]; readonly failure: Error; },): ModelClient {
  return {
    ...scriptedClient({ script: { asked, reportsUsage: true, }, },),
    chatJson: () => Promise.reject(failure,),
  };
}

/**
 The USD each `cost=` field of the given SPEND lines holds, summed.

 @param lines - lines the logger wrote

 @returns Total of the costs on the lines that are SPEND records

 @example
 ```ts
 const total = costOfSpendLines({ lines: logged, },);
 ```
 */
function costOfSpendLines({ lines, }: { readonly lines: readonly string[]; },): number {
  /**
   Dollars of the SPEND lines read so far.
   */
  let total = 0;
  for (const line of lines) {
    if (line.includes('SPEND provider=',))
      total += Number((line.split(' cost=',)[1] ?? '0').split(' ',)[0],);
  }
  return total;
}

/**
 A whole stream with its terminator frame cut away, which is what a dropped connection leaves.
 */
const CUT = claudeStreamOf({ text: '{"spot":"sun', stopReason: 'end_turn', usage: PLAIN_USAGE, })
  .replace(anthropicFrameOf({ body: { type: 'message_stop', }, },), '',);

/**
 A 200 stream with text and then an error event of the given type in place of its terminator.

 @param errorType - type the error event names

 @returns Stream's body

 @example
 ```ts
 const body = streamWithErrorEvent({ errorType: 'overloaded_error', },);
 ```
 */
function streamWithErrorEvent({ errorType, }: { readonly errorType: string; },): string {
  /**
   The frame that ends a whole stream.
   */
  const terminator = anthropicFrameOf({ body: { type: 'message_stop', }, },);
  /**
   The error event that takes its place.
   */
  const errorEvent = anthropicFrameOf({ body: { type: 'error', error: { type: errorType, message: 'Napping', }, }, },);
  return claudeStreamOf({ text: '{"spot":"sun', stopReason: 'end_turn', usage: PLAIN_USAGE, })
    .replace(terminator, errorEvent,);
}

/**
 A stream cut by an overloaded event, which the client retries.
 */
const OVERLOADED = streamWithErrorEvent({ errorType: 'overloaded_error', },);

/**
 A stream cut by an event naming a refused request, which the client does not retry.
 */
const REFUSED_EVENT = streamWithErrorEvent({ errorType: 'invalid_request_error', },);

/**
 Retry pacing that gives a call its five attempts at a millisecond apart.
 */
const FOUR_RETRIES = {
  limit: 4,
  baseMs: 1,
} as const;

/**
 Asks the guarded client one JSON exchange and keeps both what it raised and the info lines the call logged.

 @param client - guarded client
 @param controller - arm's own

 @returns What the call raised, or undefined when it completed, with the lines it logged

 @example
 ```ts
 const { result, logged, } = await askLogged({ client, controller, },);
 ```
 */
async function askLogged(
  {
    client,
    controller,
  }: {
    readonly client: ModelClient;
    readonly controller: AbortController;
  },
): Promise<{ readonly result: unknown; readonly logged: readonly string[]; }> {
  return await infoLinesDuring({
    run: async () =>
      await raisedBy({
        ask: function askOnce(): Promise<unknown> {
          return askJson({ client, modelId: LOW_CARD, signal: controller.signal, },);
        },
      },),
  },);
}

/**
 The Claude client built over a transport with no admit given, which tells nobody of its attempts.

 @param transport - scripted transport
 @param retries - retry pacing

 @returns Claude client's surface as a guard takes it

 @example
 ```ts
 const inner = claudeUntoldOf({ transport, },);
 ```
 */
function claudeUntoldOf(
  {
    transport,
    retries = ONE_QUICK_RETRY,
  }: {
    readonly transport: ModelTransport;
    readonly retries?: { readonly limit: number; readonly baseMs: number; };
  },
): ModelClient {
  const claude = createClaudeClient({ apiKey: CLAUDE_CAT_KEY, transport, retryPolicy: retries, },);
  return {
    chatText: claude.chatText,
    chatJson: claude.chatJson,
    quotas: () => Promise.reject(new Error('quotas unused',),),
  };
}

/**
 The Claude client built over a transport once the guard hands it the admit of each attempt.

 @param transport - scripted transport
 @param retries - retry pacing

 @returns Builder a guard takes in place of a client

 @example
 ```ts
 const client = spendGuardedClient({ inner: claudeInnerOf({ transport, retries, }), maxUsd: 1, controller, },);
 ```
 */
function claudeInnerOf(
  {
    transport,
    retries,
  }: {
    readonly transport: ModelTransport;
    readonly retries: { readonly limit: number; readonly baseMs: number; };
  },
): (admit: Parameters<typeof createClaudeClient>[0]['admitAttempt'],) => ModelClient {
  return function build(admit,): ModelClient {
    const claude = createClaudeClient({
      apiKey: CLAUDE_CAT_KEY,
      transport,
      retryPolicy: retries,
      ...((admit === undefined) ? {} : { admitAttempt: admit, }),
    },);
    return {
      chatText: claude.chatText,
      chatJson: claude.chatJson,
      quotas: () => Promise.reject(new Error('quotas unused',),),
    };
  };
}

/**
 The most one attempt of the scripted call could be billed, off the body the first request carried.

 @param exchanges - what the transport saw

 @returns Dollars at the Sonnet prices, the bytes priced at the cache-write rate and the whole completion cap at the
 output rate

 @example
 ```ts
 const oneAttempt = attemptBoundOf({ exchanges, },);
 ```
 */
function attemptBoundOf({ exchanges, }: { readonly exchanges: readonly TransportExchange[]; },): number {
  /**
   Bytes of the first request's body.
   */
  const bytes = Buffer.byteLength(exchanges[0]?.bodyJson ?? '', 'utf8',);
  return ((bytes * SONNET_5_5_PRICES.cacheWriteUsdPerMillion)
    + (Number(firstBodyOf({ exchanges, },).max_tokens,) * SONNET_5_5_PRICES.outputUsdPerMillion)) / 1_000_000;
}

/**
 Builds the client over the guard's admit, which on its one call tells the guard of an attempt of the bound given after
 0.5 USD of an earlier attempt is on the meter.

 @param boundUsd - most the attempt can be billed

 @returns Builder a guard takes in place of a client

 @example
 ```ts
 const inner = innerWithBound({ boundUsd: 0.5, },);
 ```
 */
function innerWithBound(
  { boundUsd, }: { readonly boundUsd: number; },
): (admit: Parameters<typeof createClaudeClient>[0]['admitAttempt'],) => ModelClient {
  return function build(admit,): ModelClient {
    return {
      ...scriptedClient({ script: { asked: [], reportsUsage: true, }, },),
      chatJson: <ValueT,>(): Promise<ChatJsonOutcome<ValueT>> => {
        noteRunSpend({ provider: 'anthropic', costUsd: 0.5, },);
        admit?.({ label: LOW_CARD, bound: { promptTokens: 1_000, completionTokens: 32_000, usd: boundUsd, }, },);
        noteRunSpend({ provider: 'anthropic', costUsd: 0.1, },);
        return Promise.resolve({
          kind: 'ok',
          value: { edits: [], } as ValueT,
          rawText: '{"edits":[]}',
          usage: REPORTED,
        },);
      },
    };
  };
}

await describe({
  name: '',
  // SEQUENTIAL: the run spend meter is process-wide and the guard reads it, so cases must not overlap.
  concurrency: 1,
  children: [
    describe({
      name: callCeilingUsd.name,
      children: [
        it({
          name: 'PRICES THE MOST ONE CALL CAN COST: the prompt at one input token a character and the whole completion cap '
            + 'at the output rate',
          fn: async () => {
            expect(callCeilingUsd({
              prices: pricesOf({ cardId: LOW_CARD, },),
              promptTokens: 1_000,
            },),).toBeCloseTo(0.322, 10,);
          },
        },),
      ],
    },),

    describe({
      name: guardLine.name,
      children: [
        it({
          name: 'SAYS WHAT HOLDS THE ARM TO --max-usd and what it counts, so a log shows the guard was on, at what amount, '
            + 'what the earlier arms of the bench spent, and that a call that throws is counted at its ceiling',
          fn: async () => {
            expect(guardLine({ maxUsd: 2, benchSpentUsd: 7.25, },),).toBe(
              'spend guard: each call, and each attempt the client makes inside it, is refused, and the arm ends, '
                + 'when the spend so far plus its ceiling would pass --max-usd 2.0000; the arms run before this one '
                + 'spent --bench-spent-usd 7.2500; the spend so far is every SPEND line the client wrote, the '
                + 'attempts it abandoned included, and a call whose last attempt wrote no line and threw, or a reply '
                + 'that reports no usage, is counted at its ceiling',
            );
            expect(guardLine({ maxUsd: 3, benchSpentUsd: 0, },),).toContain('--bench-spent-usd 0.0000;',);
          },
        },),
      ],
    },),

    describe({
      name: guardTotalLine.name,
      children: [
        it({
          name: 'SAYS WHAT THE GUARD COUNTED in the one grammar the read step totals: dollars, calls, and the calls it '
            + 'counted at their ceiling itself',
          fn: async () => {
            expect(guardTotalLine({ total: { usd: 0.322, calls: 3, atCeiling: 1, }, },),).toBe(
              'GUARD total=0.322000 calls=3 atCeiling=1',
            );
          },
        },),
      ],
    },),

    describe({
      name: spendGuardedClient.name,
      children: [
        it({
          name: 'PASSES A CALL whose ceiling fits what is left and hands back the inner reply untouched, JSON and text '
            + 'alike',
          fn: async () => {
            resetRunSpend();
            const asked: string[] = [];
            const controller = new AbortController();
            const client = spendGuardedClient({
              inner: scriptedClient({ script: { asked, pricedUsd: REPLY_USD, reportsUsage: true, }, },),
              maxUsd: 1,
              controller,
            },);
            expect(await askJson({ client, modelId: LOW_CARD, signal: controller.signal, },),).toStrictEqual({
              kind: 'ok',
              value: { edits: [], },
              rawText: '{"edits":[]}',
              usage: REPORTED,
            },);
            expect(await client.chatText({
              modelId: LOW_CARD,
              messages: THOUSAND_CHARS,
              signal: controller.signal,
            },),).toStrictEqual({ text: 'meow', usage: REPORTED, },);
            expect(asked,).toStrictEqual([LOW_CARD, LOW_CARD,],);
            expect(controller.signal.aborted,).toBe(false,);
            expect(client.guardTotal().usd,).toBeCloseTo(2 * REPLY_USD, 10,);
            expect(client.guardTotal().calls,).toBe(2,);
            expect(client.guardTotal().atCeiling,).toBe(0,);
          },
        },),

        it({
          name: 'STOPS THE ARM BEFORE THE CALL THAT WOULD PASS --max-usd: replies that cost 0.1025 USD each on the meter '
            + 'reach the cap, the seven calls that fit are made, the eighth is refused before the inner client is '
            + 'reached, and the refusal names the spend so far, that call\'s ceiling and --max-usd',
          fn: async () => {
            resetRunSpend();
            const asked: string[] = [];
            const controller = new AbortController();
            const client = spendGuardedClient({
              inner: scriptedClient({ script: { asked, pricedUsd: REPLY_USD, reportsUsage: true, }, },),
              maxUsd: 1,
              controller,
            },);
            for (let call = 0; call < 7; call += 1) {
              // oxlint-disable-next-line no-await-in-loop -- each call settles, and is measured, before the next is asked
              await askJson({ client, modelId: LOW_CARD, signal: controller.signal, },);
            }
            expect(asked.length,).toBe(7,);
            const raised = await raisedBy({
              ask: function eighth(): Promise<unknown> {
                return askJson({ client, modelId: LOW_CARD, signal: controller.signal, },);
              },
            },);
            expect(raised,).toBeInstanceOf(StatedRefusalError,);
            expect((raised as StatedRefusalError).message,).toBe(
              `the editor fidelity probe will not ask ${LOW_CARD}: the spend so far is 0.7175 USD, this call can cost `
                + '0.3220 USD at most, and together they pass --max-usd 1.0000; the arm ends here, and the rows it '
                + 'finished stay in their files',
            );
            expect(asked.length,).toBe(7,);
          },
        },),

        it({
          name: 'ENDS THE ARM, not one voice of it: the refusal aborts the arm\'s signal with itself as the reason, so '
            + 'every layer that rethrows an abort carries it out, and every later call is refused the same way without '
            + 'reaching the inner client',
          fn: async () => {
            resetRunSpend();
            const asked: string[] = [];
            const controller = new AbortController();
            const client = spendGuardedClient({
              inner: scriptedClient({ script: { asked, pricedUsd: REPLY_USD, reportsUsage: true, }, },),
              maxUsd: 0.3,
              controller,
            },);
            const first = await raisedBy({
              ask: function firstCall(): Promise<unknown> {
                return askJson({ client, modelId: LOW_CARD, signal: controller.signal, },);
              },
            },);
            expect(first,).toBeInstanceOf(StatedRefusalError,);
            expect(controller.signal.aborted,).toBe(true,);
            expect(controller.signal.reason,).toBe(first,);
            const second = await raisedBy({ ask: chatTextAsker({ client, controller, }), },);
            expect(second,).toBe(first,);
            expect(asked,).toStrictEqual([],);
          },
        },),

        it({
          name: 'REFUSES THE FIRST CALL when its ceiling alone passes --max-usd, naming a spend of nothing so far',
          fn: async () => {
            resetRunSpend();
            const controller = new AbortController();
            const client = spendGuardedClient({
              inner: scriptedClient({ script: { asked: [], pricedUsd: REPLY_USD, reportsUsage: true, }, },),
              maxUsd: 0.3,
              controller,
            },);
            const raised = await raisedBy({
              ask: function firstCall(): Promise<unknown> {
                return askJson({ client, modelId: LOW_CARD, signal: controller.signal, },);
              },
            },);
            expect((raised as StatedRefusalError).message,).toBe(
              `the editor fidelity probe will not ask ${LOW_CARD}: the spend so far is 0.0000 USD, this call can cost `
                + '0.3220 USD at most, and together they pass --max-usd 0.3000; the arm ends here, and the rows it '
                + 'finished stay in their files',
            );
          },
        },),

        it({
          name: 'LETS A CALL THROUGH that brings the total exactly to --max-usd, since only a total past it is refused',
          fn: async () => {
            resetRunSpend();
            const asked: string[] = [];
            const controller = new AbortController();
            const client = spendGuardedClient({
              inner: scriptedClient({ script: { asked, pricedUsd: REPLY_USD, reportsUsage: true, }, },),
              maxUsd: 0.322,
              controller,
            },);
            await askJson({ client, modelId: LOW_CARD, signal: controller.signal, },);
            expect(asked,).toStrictEqual([LOW_CARD,],);
          },
        },),

        it({
          name: 'COUNTS A REPLY THAT REPORTS NO USAGE AT ITS CEILING, since its cost is unknown and an unknown cost may '
            + 'not be assumed small: the guard writes the SPEND line itself, marked unreported-bound, so two such calls '
            + 'of 0.322 USD stop an arm held to 0.6',
          fn: async () => {
            resetRunSpend();
            const asked: string[] = [];
            const controller = new AbortController();
            const client = spendGuardedClient({
              inner: scriptedClient({ script: { asked, reportsUsage: false, }, },),
              maxUsd: 0.6,
              controller,
            },);
            const { logged, } = await infoLinesDuring({
              run: async () => await askJson({ client, modelId: LOW_CARD, signal: controller.signal, },),
            },);
            const raised = await raisedBy({
              ask: function secondCall(): Promise<unknown> {
                return askJson({ client, modelId: LOW_CARD, signal: controller.signal, },);
              },
            },);
            expect((raised as StatedRefusalError).message,).toContain('the spend so far is 0.3220 USD',);
            expect(asked,).toStrictEqual([LOW_CARD,],);
            expect(logged.filter(function isBound(line,): boolean {
              return line.includes(`SPEND provider=anthropic model=${LOW_CARD} prompt=1000 completion=32000 cost=0.322 `
                + 'estimated=unreported-bound',);
            },).length,).toBe(1,);
            expect(runSpendUsd({ provider: 'anthropic', },),).toBeCloseTo(0.322, 10,);
            expect(client.guardTotal().atCeiling,).toBe(1,);
          },
        },),

        it({
          name: 'COUNTS A CALL THAT THROWS AT ITS CEILING, since the client may have been billed for a stream it never '
            + 'delivered: the guard writes the SPEND line itself, marked abandoned-bound, and two such calls leave no '
            + 'room for a third under 0.7 USD',
          fn: async () => {
            resetRunSpend();
            const asked: string[] = [];
            const controller = new AbortController();
            const client = spendGuardedClient({
              inner: failingClient({ asked, failure: new Error('the stream was cut',), },),
              maxUsd: 0.7,
              controller,
            },);
            const { logged, } = await infoLinesDuring({
              run: async () => {
                for (let call = 0; call < 2; call += 1) {
                  // oxlint-disable-next-line no-await-in-loop -- each call fails before the next is asked
                  const raised = await raisedBy({
                    ask: function failingCall(): Promise<unknown> {
                      return askJson({ client, modelId: LOW_CARD, signal: controller.signal, },);
                    },
                  },);
                  expect((raised as Error).message,).toBe('the stream was cut',);
                }
              },
            },);
            expect(controller.signal.aborted,).toBe(false,);
            expect(logged.filter(function isBound(line,): boolean {
              return line.includes(`SPEND provider=anthropic model=${LOW_CARD} prompt=1000 completion=32000 cost=0.322 `
                + 'estimated=abandoned-bound',);
            },).length,).toBe(2,);
            const third = await raisedBy({
              ask: function thirdCall(): Promise<unknown> {
                return askJson({ client, modelId: LOW_CARD, signal: controller.signal, },);
              },
            },);
            expect(third,).toBeInstanceOf(StatedRefusalError,);
            expect((third as StatedRefusalError).message,).toContain('the spend so far is 0.6440 USD',);
            expect(client.guardTotal(),).toStrictEqual({ usd: 0.644, calls: 2, atCeiling: 2, },);
          },
        },),

        it({
          name: 'COUNTS NOTHING FOR AN HTTP ERROR THAT CARRIED NO BODY, which proves no billable body was delivered, and '
            + 'keeps asking; the same status with a body counts at the ceiling',
          fn: async () => {
            resetRunSpend();
            const asked: string[] = [];
            const controller = new AbortController();
            const bare = spendGuardedClient({
              inner: failingClient({
                asked,
                failure: new ProviderHttpError({ provider: 'anthropic', status: 529, bodyText: '', },),
              },),
              maxUsd: 0.4,
              controller,
            },);
            for (let call = 0; call < 3; call += 1) {
              // oxlint-disable-next-line no-await-in-loop -- each call fails before the next is asked
              const raised = await raisedBy({
                ask: function bareCall(): Promise<unknown> {
                  return askJson({ client: bare, modelId: LOW_CARD, signal: controller.signal, },);
                },
              },);
              expect(raised,).toBeInstanceOf(ProviderHttpError,);
            }
            expect(controller.signal.aborted,).toBe(false,);
            expect(bare.guardTotal(),).toStrictEqual({ usd: 0, calls: 3, atCeiling: 0, },);
            expect(runSpendUsd({ provider: 'anthropic', },),).toBe(0,);

            const bodied = spendGuardedClient({
              inner: failingClient({
                asked,
                failure: new ProviderHttpError({
                  provider: 'anthropic',
                  status: 529,
                  bodyText: '{"type":"error","error":{"type":"overloaded_error"}}',
                },),
              },),
              maxUsd: 0.4,
              controller,
            },);
            await raisedBy({
              ask: function bodiedCall(): Promise<unknown> {
                return askJson({ client: bodied, modelId: LOW_CARD, signal: controller.signal, },);
              },
            },);
            expect(bodied.guardTotal().usd,).toBeCloseTo(0.322, 10,);
          },
        },),

        it({
          name: 'COUNTS EACH ATTEMPT THE CLIENT ABANDONED BEFORE IT ANSWERED, because the client writes it on the meter: '
            + 'two abandoned attempts of 0.4 USD and a reply of 0.1 leave 0.9 USD spent by the first call, so the '
            + 'second is refused under 1.0',
          fn: async () => {
            resetRunSpend();
            const asked: string[] = [];
            const controller = new AbortController();
            const client = spendGuardedClient({
              inner: scriptedClient({
                script: { asked, pricedUsd: 0.1, abandonedUsd: [0.4, 0.4,], reportsUsage: true, },
              },),
              maxUsd: 1,
              controller,
            },);
            await askJson({ client, modelId: LOW_CARD, signal: controller.signal, },);
            const raised = await raisedBy({
              ask: function secondCall(): Promise<unknown> {
                return askJson({ client, modelId: LOW_CARD, signal: controller.signal, },);
              },
            },);
            expect((raised as StatedRefusalError).message,).toContain('the spend so far is 0.9000 USD',);
            expect(asked.length,).toBe(1,);
          },
        },),

        it({
          name: 'PRICES EACH CALL BY ITS OWN MODEL, so a Haiku call is not held to the Sonnet ceiling, and keeps the rest '
            + 'of the client it wraps',
          fn: async () => {
            resetRunSpend();
            const asked: string[] = [];
            const controller = new AbortController();
            const inner = {
              ...scriptedClient({ script: { asked, pricedUsd: 0.01, reportsUsage: true, }, },),
              providerHolds: 'held',
            };
            const client = spendGuardedClient({
              inner,
              maxUsd: 0.1,
              controller,
            },);
            await askJson({ client, modelId: HAIKU_CARD, signal: controller.signal, },);
            expect(asked,).toStrictEqual([HAIKU_CARD,],);
            expect((client as unknown as { readonly providerHolds: string; }).providerHolds,).toBe('held',);
          },
        },),

        it({
          name: 'TOTALS WHAT THE REAL CLAUDE CLIENT WRITES: over a scripted transport, a call whose two streams were cut '
            + 'before their end (two abandoned-bound lines of the client, then a throw the guard counts at its '
            + 'ceiling) and a call that answers hold a guard total equal to the sum of every SPEND line cost the '
            + 'arm logged, and the sum is not under what the client abandoned',
          fn: async () => {
            resetRunSpend();
            /**
             A whole stream with its terminator frame cut away.
             */
            const cut = claudeStreamOf({ text: '{"spot":"sun', stopReason: 'end_turn', usage: PLAIN_USAGE, })
              .replace(anthropicFrameOf({ body: { type: 'message_stop', }, },), '',);
            const { transport, } = scripted({
              replies: [
                { status: 200, bodyText: cut, },
                { status: 200, bodyText: cut, },
                {
                  status: 200,
                  bodyText: claudeStreamOf({
                    text: '{"spot":"sunbeam"}',
                    stopReason: 'end_turn',
                    usage: PLAIN_USAGE,
                  }),
                },
              ],
            },);
            const controller = new AbortController();
            const claude = createClaudeClient({ apiKey: CLAUDE_CAT_KEY, transport, retryPolicy: ONE_QUICK_RETRY, },);
            const client = spendGuardedClient({
              inner: {
                chatText: claude.chatText,
                chatJson: claude.chatJson,
                quotas: () => Promise.reject(new Error('quotas unused',),),
              },
              maxUsd: 5,
              controller,
            },);
            const { logged, } = await infoLinesDuring({
              run: async () => {
                await raisedBy({
                  ask: function cutCall(): Promise<unknown> {
                    return client.chatJson({
                      modelId: 'claude-sonnet-5-5-effort-low',
                      messages: THOUSAND_CHARS,
                      signal: controller.signal,
                      validate: (value: unknown,): value is unknown => value !== undefined,
                    },);
                  },
                },);
                await client.chatJson({
                  modelId: 'claude-sonnet-5-5-effort-low',
                  messages: THOUSAND_CHARS,
                  signal: controller.signal,
                  validate: (value: unknown,): value is unknown => value !== undefined,
                },);
              },
            },);
            /**
             Cost of every SPEND line the arm logged, the client's and the guard's together.
             */
            const logTotal = costOfSpendLines({ lines: logged, },);
            expect(client.guardTotal().usd,).toBeCloseTo(logTotal, 9,);
            expect(runSpendUsd({ provider: 'anthropic', },),).toBeCloseTo(logTotal, 9,);
            expect(client.guardTotal().calls,).toBe(2,);
            expect(client.guardTotal().atCeiling,).toBe(1,);
            expect(logged.filter(function isBound(line,): boolean {
              return line.includes('estimated=abandoned-bound',);
            },).length,).toBe(3,);
            expect(logTotal,).toBeGreaterThan(0.9,);
          },
        },),
      ],
    },),

    describe({
      name: `${spendGuardedClient.name} one call at a time`,
      children: [
        it({
          name: 'ADMITS ONE CALL AT A TIME: a call asked while another is in flight throws before the inner client is '
            + 'reached, does not end the arm, and the next call after the first settles is admitted',
          fn: async () => {
            resetRunSpend();
            const asked: string[] = [];
            const controller = new AbortController();
            const release = Promise.withResolvers<ChatTextReply>();
            const client = spendGuardedClient({
              inner: {
                ...scriptedClient({ script: { asked, pricedUsd: REPLY_USD, reportsUsage: true, }, },),
                chatText: (request,): Promise<ChatTextReply> => {
                  asked.push(request.modelId,);
                  // Only the first call stays in flight; any other would answer, so a missing guard fails the case
                  // by an assertion and not by a hang.
                  return (asked.length === 1) ? release.promise : Promise.resolve({ text: 'meow', usage: REPORTED, },);
                },
              },
              maxUsd: 5,
              controller,
            },);
            const first = client.chatText({
              modelId: LOW_CARD,
              messages: THOUSAND_CHARS,
              signal: controller.signal,
            },);
            const second = await raisedBy({ ask: chatTextAsker({ client, controller, }), },);
            expect(second,).toBeInstanceOf(ConcurrentGuardCallError,);
            expect(asked,).toStrictEqual([LOW_CARD,],);
            expect(controller.signal.aborted,).toBe(false,);
            release.resolve({ text: 'meow', usage: REPORTED, },);
            await first;
            expect(client.guardTotal().calls,).toBe(1,);
            const third = await raisedBy({
              ask: function afterFirstSettled(): Promise<unknown> {
                return askJson({ client, modelId: LOW_CARD, signal: controller.signal, },);
              },
            },);
            expect(third,).toBeUndefined();
            expect(client.guardTotal().calls,).toBe(2,);
          },
        },),
      ],
    },),

    describe({
      name: 'a guard over a client that tells it of each attempt',
      children: [
        it({
          name: 'ADMITS AN ATTEMPT WHOSE BOUND BRINGS THE SPEND EXACTLY TO --max-usd AND REFUSES ONE A CENT OVER, the '
            + 'attempt-only boundary: 0.5 USD already on the meter and an attempt bound of 0.5 under 1.0 is sent, and '
            + 'a bound of 0.51 is not',
          fn: async () => {
            resetRunSpend();
            const atBoundary = new AbortController();
            const sent = spendGuardedClient({
              inner: innerWithBound({ boundUsd: 0.5, },),
              maxUsd: 1,
              controller: atBoundary,
            },);
            expect(await raisedBy({
              ask: function atTheLine(): Promise<unknown> {
                return askJson({ client: sent, modelId: LOW_CARD, signal: atBoundary.signal, },);
              },
            },),).toBeUndefined();
            expect(atBoundary.signal.aborted,).toBe(false,);

            resetRunSpend();
            const pastBoundary = new AbortController();
            const refused = spendGuardedClient({
              inner: innerWithBound({ boundUsd: 0.51, },),
              maxUsd: 1,
              controller: pastBoundary,
            },);
            const raised = await raisedBy({
              ask: function aCentOver(): Promise<unknown> {
                return askJson({ client: refused, modelId: LOW_CARD, signal: pastBoundary.signal, },);
              },
            },);
            expect(raised,).toBeInstanceOf(StatedRefusalError,);
            expect((raised as StatedRefusalError).message,).toContain('this attempt can cost 0.5100 USD at most',);
            expect(pastBoundary.signal.aborted,).toBe(true,);
          },
        },),

        it({
          name: 'REFUSES THE ATTEMPT THAT WOULD PASS --max-usd INSIDE A CALL IT ADMITTED: attempts each cut by an '
            + 'overloaded event are about 0.323 USD apiece on the meter, the call fits under 1.5 at its ceiling of '
            + '0.322 and is admitted, and the fifth attempt is refused before it is sent, so four requests go out, '
            + 'the spend stays under 1.5 and the arm ends',
          fn: async () => {
            resetRunSpend();
            const { transport, exchanges, } = scripted({ replies: [{ status: 200, bodyText: OVERLOADED, },], },);
            const controller = new AbortController();
            const client = spendGuardedClient({
              inner: claudeInnerOf({ transport, retries: FOUR_RETRIES, }),
              maxUsd: 1.5,
              controller,
            },);
            const raised = await askLogged({ client, controller, },);
            expect(raised.result,).toBeInstanceOf(StatedRefusalError,);
            expect((raised.result as StatedRefusalError).message,).toContain('this attempt can cost',);
            expect((raised.result as StatedRefusalError).message,).toContain('together they pass --max-usd 1.5000',);
            expect(exchanges.length,).toBe(4,);
            expect(controller.signal.aborted,).toBe(true,);
            expect(runSpendUsd({ provider: 'anthropic', },),).toBeLessThan(1.5,);
            expect(client.guardTotal().usd,).toBeLessThan(1.5,);
            expect(client.guardTotal().atCeiling,).toBe(0,);
          },
        },),

        it({
          name: 'REFUSES A RETRY THROUGH THE WHOLE ROUTE THE PROBE BUILDS, the run client over the Claude key with the '
            + 'seat tally and the card guard: the first attempt is cut and written at its bound, the retry would pass '
            + '0.5, so it is never sent, and the arm\'s signal carries the refusal',
          fn: async () => {
            using _fresh = withFreshRunSeats();
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-guard-', },);
            resetRunSpend();
            const { transport, exchanges, } = scripted({ replies: [{ status: 200, bodyText: CUT, },], },);
            const controller = new AbortController();
            const client = spendGuardedClient({
              inner: function routeOf(admit,): ModelClient {
                return cardsClientFrom({
                  env: { [CLAUDE_KEY_VARIABLE]: CLAUDE_CAT_KEY, },
                  transport,
                  promptPayloadDir: scratch.path,
                  cardIds: [LOW_CARD,],
                  admitClaudeAttempt: admit,
                },);
              },
              maxUsd: 0.5,
              controller,
            },);
            const raised = await askLogged({ client, controller, },);
            expect(raised.result,).toBeDefined();
            expect(exchanges.length,).toBe(1,);
            expect(controller.signal.reason,).toBeInstanceOf(StatedRefusalError,);
            expect((controller.signal.reason as StatedRefusalError).message,).toContain('this attempt can cost',);
            expect(client.guardTotal().usd,).toBeLessThan(0.5,);
          },
        },),

        it({
          name: 'LETS EVERY ATTEMPT OF A CALL THROUGH THAT FITS, and counts each attempt once: five cut attempts under '
            + '2 USD are five bound lines of the client and nothing of the guard\'s own, although the call throws, so '
            + 'the guard\'s total is the meter\'s',
          fn: async () => {
            resetRunSpend();
            const { transport, exchanges, } = scripted({ replies: [{ status: 200, bodyText: OVERLOADED, },], },);
            const controller = new AbortController();
            const client = spendGuardedClient({
              inner: claudeInnerOf({ transport, retries: FOUR_RETRIES, }),
              maxUsd: 2,
              controller,
            },);
            const raised = await askLogged({ client, controller, },);
            expect(raised.result,).toBeInstanceOf(Error,);
            expect(raised.result,).not.toBeInstanceOf(StatedRefusalError,);
            expect(exchanges.length,).toBe(5,);
            expect(client.guardTotal().atCeiling,).toBe(0,);
            expect(client.guardTotal().calls,).toBe(1,);
            expect(client.guardTotal().usd,).toBeCloseTo(runSpendUsd({ provider: 'anthropic', },), 9,);
            expect(client.guardTotal().usd,).toBeCloseTo(5 * attemptBoundOf({ exchanges, }), 6,);
            expect(costOfSpendLines({ lines: raised.logged, },),).toBeCloseTo(client.guardTotal().usd, 9,);
          },
        },),

        it({
          name: 'COUNTS NOTHING FOR A CALL THAT NEVER LEFT, a rejection by a layer above the client such as a retained '
            + 'prompt claim or a withheld seat: no attempt was told to the guard, so the throw is free',
          fn: async () => {
            resetRunSpend();
            const controller = new AbortController();
            const client = spendGuardedClient({
              inner: function neverSends(): ModelClient {
                return failingClient({ asked: [], failure: new Error('the retained claim was rejected again',), });
              },
              maxUsd: 1,
              controller,
            },);
            const raised = await raisedBy({
              ask: function askOnce(): Promise<unknown> {
                return askJson({ client, modelId: LOW_CARD, signal: controller.signal, },);
              },
            },);
            expect((raised as Error).message,).toBe('the retained claim was rejected again',);
            expect(client.guardTotal(),).toStrictEqual({ usd: 0, calls: 1, atCeiling: 0, },);
            expect(runSpendUsd({ provider: 'anthropic', },),).toBe(0,);
          },
        },),

        it({
          name: 'ENDS THE ARM WHEN A REPLY COMES WITH NO REQUEST BEHIND IT, and books nothing for it: a client that tells '
            + 'the guard of no attempt and still answers with usage is a prompt answered from reuse, which cost nothing, '
            + 'so no SPEND line is written at the ceiling, the total stays at zero, the arm\'s signal carries the ending, '
            + 'and a later call is refused with it before the client is reached',
          fn: async () => {
            resetRunSpend();
            /**
             Model ids the scripted client was asked.
             */
            const asked: string[] = [];
            const controller = new AbortController();
            const client = spendGuardedClient({
              inner: function replays(): ModelClient {
                return scriptedClient({ script: { asked, reportsUsage: true, }, },);
              },
              maxUsd: 5,
              controller,
            },);
            const first = await askLogged({ client, controller, },);
            expect(first.logged.filter(function isSpend(line,): boolean {
              return line.includes('SPEND provider=',);
            },),).toStrictEqual([],);
            expect(client.guardTotal(),).toStrictEqual({ usd: 0, calls: 1, atCeiling: 0, },);
            expect(runSpendUsd({ provider: 'anthropic', },),).toBe(0,);
            expect(first.result,).toBeInstanceOf(RepliedWithoutCallError,);
            expect(controller.signal.aborted,).toBe(true,);
            expect(controller.signal.reason,).toBe(first.result,);

            const later = await raisedBy({
              ask: function askAgain(): Promise<unknown> {
                return askJson({ client, modelId: LOW_CARD, signal: controller.signal, },);
              },
            },);
            expect(later,).toBe(first.result,);
            expect(asked.length,).toBe(1,);
          },
        },),

        it({
          name: 'ENDS THE ARM ON A TEXT REPLY WITH NO REQUEST BEHIND IT as it does on a JSON one, and the ending names '
            + 'the reuse and the log line that holds its digest',
          fn: async () => {
            resetRunSpend();
            const controller = new AbortController();
            const client = spendGuardedClient({
              inner: function replays(): ModelClient {
                return scriptedClient({ script: { asked: [], reportsUsage: true, }, },);
              },
              maxUsd: 5,
              controller,
            },);
            const raised = await raisedBy({ ask: chatTextAsker({ client, controller, },), },);
            expect(client.guardTotal(),).toStrictEqual({ usd: 0, calls: 1, atCeiling: 0, },);
            expect(raised,).toBeInstanceOf(RepliedWithoutCallError,);
            expect((raised as Error).message,).toContain('answered from reuse',);
            expect((raised as Error).message,).toContain('PROMPT-REUSE',);
          },
        },),

        it({
          name: 'COUNTS ONE ATTEMPT ONCE WHEN THE CALL THROWS AFTER RETRIES: two cut attempts are two bound lines of the '
            + 'client, and the third, which fails with nothing delivered and no line, is counted at its ceiling by '
            + 'the guard, so three counts for three attempts and none for the call itself',
          fn: async () => {
            resetRunSpend();
            const { transport: cutTransport, exchanges, } = scripted({
              replies: [{ status: 200, bodyText: CUT, },],
            },);
            /**
             Requests the transport was handed, to tell the third from the first two.
             */
            const sent = { count: 0, };
            const controller = new AbortController();
            const client = spendGuardedClient({
              inner: claudeInnerOf({
                transport: async function dropsTheThird(exchange,) {
                  sent.count += 1;
                  if (sent.count === 3)
                    throw new Error('socket hang up',);
                  return await cutTransport(exchange,);
                },
                retries: { limit: 2, baseMs: 1, },
              }),
              maxUsd: 5,
              controller,
            },);
            const raised = await askLogged({ client, controller, },);
            expect((raised.result as Error).message,).toContain('socket hang up',);
            expect(sent.count,).toBe(3,);
            expect(exchanges.length,).toBe(2,);
            expect(raised.logged.filter(function isBound(line,): boolean {
              return line.includes('estimated=abandoned-bound',);
            },).length,).toBe(3,);
            expect(client.guardTotal().atCeiling,).toBe(1,);
            expect(client.guardTotal().usd,).toBeCloseTo((2 * attemptBoundOf({ exchanges, })) + 0.322, 6,);
            expect(client.guardTotal().usd,).toBeCloseTo(costOfSpendLines({ lines: raised.logged, }), 9,);
          },
        },),

        it({
          name: 'COUNTS A REPLY WHOSE OWN LINE IS MISSING EVEN WHEN AN EARLIER ATTEMPT WROTE ONE: the meter is compared '
            + 'with its reading at the start of the last attempt, so the abandoned line of the first attempt does not '
            + 'hide that the second, which reports usage, wrote nothing',
          fn: async () => {
            resetRunSpend();
            const controller = new AbortController();
            const client = spendGuardedClient({
              inner: function twoAttempts(admit,): ModelClient {
                return {
                  ...scriptedClient({ script: { asked: [], reportsUsage: true, }, },),
                  chatJson: async <ValueT,>(): Promise<ChatJsonOutcome<ValueT>> => {
                    admit({ label: LOW_CARD, bound: { promptTokens: 1_000, completionTokens: 32_000, usd: 0.322, }, },);
                    noteRunSpend({ provider: 'anthropic', costUsd: 0.4, },);
                    admit({ label: LOW_CARD, bound: { promptTokens: 1_000, completionTokens: 32_000, usd: 0.322, }, },);
                    return {
                      kind: 'ok',
                      value: { edits: [], } as ValueT,
                      rawText: '{"edits":[]}',
                      usage: REPORTED,
                    };
                  },
                };
              },
              maxUsd: 5,
              controller,
            },);
            await askJson({ client, modelId: LOW_CARD, signal: controller.signal, },);
            expect(client.guardTotal().atCeiling,).toBe(1,);
            expect(client.guardTotal().usd,).toBeCloseTo(0.4 + 0.322, 9,);
          },
        },),

        it({
          name: 'COUNTS A REPLY WITH USAGE BUT NO LINE ON THE METER AT ITS CEILING, since a client that reports tokens '
            + 'and writes nothing leaves a cost nobody wrote down',
          fn: async () => {
            resetRunSpend();
            const controller = new AbortController();
            const client = spendGuardedClient({
              inner: scriptedClient({ script: { asked: [], reportsUsage: true, }, },),
              maxUsd: 5,
              controller,
            },);
            await askJson({ client, modelId: LOW_CARD, signal: controller.signal, },);
            expect(client.guardTotal(),).toStrictEqual({ usd: 0.322, calls: 1, atCeiling: 1, },);
            expect(runSpendUsd({ provider: 'anthropic', },),).toBeCloseTo(0.322, 10,);
          },
        },),

        it({
          name: 'READS THE SPEND OF THE ARM ALONE, not the meter\'s whole: lines the meter held before the guard was '
            + 'built are not the arm\'s, so a reply of 0.1025 USD after 0.5 USD already on the meter is a total of '
            + '0.1025 USD',
          fn: async () => {
            resetRunSpend();
            noteRunSpend({ provider: 'anthropic', costUsd: 0.5, },);
            const controller = new AbortController();
            const client = spendGuardedClient({
              inner: scriptedClient({ script: { asked: [], pricedUsd: REPLY_USD, reportsUsage: true, }, },),
              maxUsd: 1,
              controller,
            },);
            await askJson({ client, modelId: LOW_CARD, signal: controller.signal, },);
            expect(client.guardTotal().usd,).toBeCloseTo(REPLY_USD, 10,);
            expect(runSpendUsd({ provider: 'anthropic', },),).toBeCloseTo(0.5 + REPLY_USD, 10,);
          },
        },),

        it({
          name: 'KEEPS A STREAM ERROR EVENT SAFE TO CALL FREE: a 200 stream with text and then an error event naming a '
            + 'refused request is thrown with an empty body text, which the guard without attempts counts as a call '
            + 'that delivered nothing, and the client has written the stream\'s bound line, so the guard\'s total '
            + 'is the meter\'s, with the attempts told and without them',
          fn: async () => {
            resetRunSpend();
            const { transport, exchanges, } = scripted({ replies: [{ status: 200, bodyText: REFUSED_EVENT, },], },);
            const controller = new AbortController();
            const withoutAttempts = spendGuardedClient({
              inner: claudeUntoldOf({ transport, }),
              maxUsd: 5,
              controller,
            },);
            const first = await askLogged({ client: withoutAttempts, controller, },);
            expect(first.result,).toBeInstanceOf(ProviderHttpError,);
            expect((first.result as ProviderHttpError).bodyExcerpt,).toBe('',);
            expect(exchanges.length,).toBe(1,);
            expect(first.logged.filter(function isBound(line,): boolean {
              return line.includes('estimated=abandoned-bound',);
            },).length,).toBe(1,);
            expect(withoutAttempts.guardTotal().atCeiling,).toBe(0,);
            expect(withoutAttempts.guardTotal().usd,).toBeCloseTo(runSpendUsd({ provider: 'anthropic', },), 9,);

            resetRunSpend();
            const withAttempts = spendGuardedClient({
              inner: claudeInnerOf({ transport, retries: ONE_QUICK_RETRY, }),
              maxUsd: 5,
              controller,
            },);
            const second = await askLogged({ client: withAttempts, controller, },);
            expect(second.result,).toBeInstanceOf(ProviderHttpError,);
            expect(withAttempts.guardTotal().atCeiling,).toBe(0,);
            expect(withAttempts.guardTotal().usd,).toBeCloseTo(runSpendUsd({ provider: 'anthropic', },), 9,);
            expect(withAttempts.guardTotal().usd,).toBeCloseTo(attemptBoundOf({ exchanges, }), 6,);
          },
        },),
      ],
    },),

    describe({
      name: `${cardsClientFrom.name} decide and quotas`,
      children: [
        it({
          name: 'SENDS NOTHING FOR decide OR quotas, so neither can bill the Claude key: over the Claude key alone the '
            + 'decisions exchange is refused for want of a decisions client and the quotas read is the exhausted '
            + 'compatibility snapshot, each with no exchange on the transport, while a chat exchange on the same '
            + 'client does reach it',
          fn: async () => {
            using _fresh = withFreshRunSeats();
            await using scratch = await scratchDir({ prefix: 'editor-fidelity-decide-', },);
            resetRunSpend();
            const { transport, exchanges, } = scripted({
              replies: [
                {
                  status: 200,
                  bodyText: claudeStreamOf({ text: '{"edits":[]}', stopReason: 'end_turn', usage: PLAIN_USAGE, }),
                },
              ],
            },);
            const controller = new AbortController();
            const client = cardsClientFrom({
              env: { [CLAUDE_KEY_VARIABLE]: CLAUDE_CAT_KEY, },
              transport,
              promptPayloadDir: scratch.path,
              cardIds: [LOW_CARD,],
            },);
            expect(client.decide,).toBeDefined();
            const decided = await raisedBy({
              ask: function askDecide(): Promise<unknown> {
                return (client.decide as NonNullable<ModelClient['decide']>)({
                  modelId: LOW_CARD,
                  state: 'The cat is asleep.',
                  questions: { asleep: { type: 'noul', instructions: 'Is the cat asleep?', }, },
                  signal: controller.signal,
                },);
              },
            },);
            expect(decided,).toBeInstanceOf(NoProviderForModelError,);
            expect((decided as NoProviderForModelError).message,).toContain('no decisions client is configured',);
            const quota = await client.quotas({ signal: controller.signal, },);
            expect(quota.fiveHour?.limited,).toBe(true,);
            expect(quota.fiveHour?.remaining,).toBe(0,);
            expect(exchanges.length,).toBe(0,);

            // The positive control: the same transport records the exchange a chat call makes.
            await infoLinesDuring({
              run: async () => await askJson({ client, modelId: LOW_CARD, signal: controller.signal, },),
            },);
            expect(exchanges.length,).toBe(1,);
          },
        },),
      ],
    },),
  ],
},);
