import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  contextRoot,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { extractAnthropicCompletion, } from './anthropic-completion.ts';
import { requireWholeAnthropicMessage, } from './anthropic-whole-message.ts';
import { buildAnthropicBody, } from './anthropic-request.ts';
import { armCallDeadline, } from './call-deadline.ts';
import type {
  ChatTextReply,
  ChatTextRequest,
  ModelCaller,
} from './chat-contract.ts';
import { chatJsonThrough, } from './chat-json-through.ts';
import {
  completionCapFor,
  type CreditsShapeError,
  HYPER_API_VERSION,
  HYPER_AUTH_HEADER,
  HYPER_CREDITS_URL,
  HYPER_MESSAGES_URL,
  type HyperCredits,
  type HyperServedId,
  parseHyperCredits,
  perModelLimiter,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  formatUsageNote,
  type MalformedCompletionError,
  type ModelTransport,
  ProviderHttpError,
  type TransportReply,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { isSuccessStatus, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  createRequestPace,
  HYPER_PACE_WINDOW_MS,
  HYPER_REQUESTS_PER_HOUR,
  type RequestPace,
} from './request-pace.ts';
import { failureForReply, } from './request-size-refusal.ts';
import { reportSpend, } from './spend-line.ts';
import { hyperIdFor, } from './roster-reach.ts';
import {
  DEFAULT_RETRY_POLICY,
  exchangeWithRetry,
  type RetryPolicy,
} from './transient-retry.ts';

//region Hyper client
// Second provider's client, over the same transport seam as the first.
//
// WHY A SECOND FILE RATHER THAN A BRANCH. Everything above the wire is shared
// and already extracted: `chat-json-outcome.ts` reads a reply, `transient-retry`
// paces the attempts, and the transport drains the stream under the guards. The
// only things this file adds are the ones that ARE different: a body in a
// different protocol, a different reader for the stream, a different spelling
// of the model name, and a balance instead of a quota.
//
// IT THROWS THE SAME FAILURE CLASS AS THE OTHER PROVIDERS, deliberately.
// `benchmark.ts` branches on `error instanceof ProviderHttpError` to read a
// status off a failed call, and a fresh class here would make that site
// silently blind to exactly the provider added to survive the other one's
// exhaustion. The class was named `SyntheticHttpError` until 2026-10-19, after
// the first provider, and was renamed once a Bedrock refusal was logged under
// that name; each throw site passes `provider: 'hyper'` so the failure says
// whose it was.
//
// STREAMS ARE READ BY THE ANTHROPIC SCANNER, named on the exchange. A body
// drained with the wrong reader yields an empty answer channel, which every
// guard reads as a well-behaved call that produced nothing, so getting this
// wrong is silent rather than loud.

/**
 Local representation of Hyper's absence of per-model concurrency ceiling.

 Live probes on 2026-08-24 completed widths through 32 on `minimax-m3`.
 A second structured probe on 2026-08-30 completed 64 of 64 calls to
 `deepseek-v4-flash-0731` in 2,147 ms without retry or non-200 status.
 The owner confirmed Hyper has no concurrency ceiling and separately limits
 this account to 1,000 requests per hour.

 `p-limit` explicitly accepts positive infinity as unbounded concurrency.
 Keeping that value inside injected limiter seam preserves tests that set
 finite widths without inventing provider serialization in normal operation.

 @example
 ```ts
 const width = HYPER_PER_MODEL_CONCURRENCY;
 ```
 */
export const HYPER_PER_MODEL_CONCURRENCY: number = Number.POSITIVE_INFINITY;

/**
 Logger root for this package's model-facing shell.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Refusal raised when a roster model has no spelling on this provider.

 A THROW RATHER THAN A DATA OUTCOME, because it is a routing mistake in our
 own code and not a thing a model did. Every outcome this client returns as
 data describes something a model wrote; a call addressed to a provider that
 does not serve the model never reaches one.

 @example
 ```ts
 throw new ModelNotServedError({ modelId, },);
 ```
 */
export class ModelNotServedError extends Error {
  /**
   Declares this message safe to forward: it names a model.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds failure naming the model this provider has no spelling for.

   @param modelId - roster model that was addressed here

   @example
   ```ts
   new ModelNotServedError({ modelId: 'hf:Qwen/Qwen3.8-27B', },);
   ```
   */
  public constructor({ modelId, }: { readonly modelId: string; },) {
    super(`Charm Hyper does not serve ${modelId}; route it to the other provider or pick another model`,);
    this.name = 'ModelNotServedError';
  }
}

/**
 Client surface for the credit-metered provider.

 @example
 ```ts
 const client: HyperClient = createHyperClient({ apiKey, transport, },);
 ```
 */
export type HyperClient = ModelCaller & {
  /**
   Remaining balance, which is this provider's whole budget signal.
   */
  readonly credits: (args: { readonly signal: AbortSignal; },) => Promise<HyperCredits>;

  /**
   How long this client's request window would make a call wait now, which
   the router reads to send a model another provider serves elsewhere while
   the window is full (class one hundred forty-nine).
   */
  readonly pace: Pick<RequestPace, 'waitMs'>;
};

/**
 Refuses a success reply whose event stream stopped before its terminator
 or reported an error.

 MODULE SCOPE BECAUSE IT CAPTURES NOTHING. The reply handed in is its whole
 input, so nesting it at the call site would make a closure over an empty set.

 ONLY A BODY THE STATUS ALREADY ACCEPTED. A non-success reply is reported by
 the status branch at the call site, which names the HTTP code; reading it
 here would replace that with a parse failure about an error page.

 @param attemptReply - one attempt's reply, read before the ladder returns it

 @throws MalformedCompletionError - when a success body stops before
 `message_stop` or carries an error event, which is what puts a truncated
 or failed stream on the retry path instead of past it

 @example
 ```ts
 const reply = await exchangeWithRetry({ provider: 'hyper', transport, exchange, policy, verify: wholeMessage, },);
 ```
 */
function wholeMessage(attemptReply: TransportReply,): void {
  if (isSuccessStatus({ status: attemptReply.status, },))
    requireWholeAnthropicMessage({
      provider: 'hyper',
      bodyText: attemptReply.bodyText,
    },);
}

/**
 Spells one roster model the way this provider names it.

 @param modelId - roster model the caller addressed

 @returns Wire identifier for the request body

 @throws {@link ModelNotServedError} when this provider serves no such model

 @example
 ```ts
 const served = servedIdFor({ modelId, },);
 ```
 */
function servedIdFor(
  { modelId, }: { readonly modelId: RosterModelId; },
): HyperServedId {
  /**
   Spelling this provider uses, or that it serves no such model.
   */
  const spelling = hyperIdFor({ modelId, },);

  if (!spelling.served)
    throw new ModelNotServedError({ modelId, },);
  return spelling.id;
}

/**
 Builds one client over injected transport, speaking the Messages protocol.

 @param apiKey - bearer token; never logged

 @param transport - HTTP the client calls over: `fetchTransport` in a run, a
 recorded one in a test. REQUIRED, since a test that left it out would reach
 the provider (ledger M43, X24)

 @param messagesUrl - completion endpoint, overridable for tests

 @param creditsUrl - balance endpoint, overridable for tests

 @param perModelConcurrency - optional local test or caller bound;
 normal operation remains unbounded because provider has no concurrency ceiling

 @param retryPolicy - transient-retry pacing; tests pass tiny backoffs

 @param requestsPerHour - request starts allowed in any rolling hour,
 retries and credit reads included; the provider limits this account to
 1,000 and refuses the rest with HTTP 429, so calls queue here instead
 (`request-pace.ts` has the measurement); a whole number of one or more, which `createRequestPace` refuses otherwise

 @returns Client surface with chatText, chatJson, and credits

 @example
 ```ts
 const client = createHyperClient({ apiKey: process.env['TRANSLATION_REPAIR_CHARM_HYPER_API_KEY'] ?? '', transport: fetchTransport, },);
 ```
 */
export function createHyperClient(
  {
    apiKey,
    transport,
    messagesUrl = HYPER_MESSAGES_URL,
    creditsUrl = HYPER_CREDITS_URL,
    perModelConcurrency = HYPER_PER_MODEL_CONCURRENCY,
    retryPolicy = DEFAULT_RETRY_POLICY,
    requestsPerHour = HYPER_REQUESTS_PER_HOUR,
  }: {
    readonly apiKey: string;
    readonly transport: ModelTransport;
    readonly messagesUrl?: string;
    readonly creditsUrl?: string;
    readonly perModelConcurrency?: number;
    readonly retryPolicy?: RetryPolicy;
    readonly requestsPerHour?: number;
  },
): HyperClient {
  /**
   Per-model limiter, created lazily for each model (`per-model-limiter.ts`).
   */
  const limiterFor = perModelLimiter({ perModelConcurrency, },);

  /**
   Request-rate pacer every attempt on this client takes a turn from.
   */
  const pace = createRequestPace({
    perWindow: requestsPerHour,
    windowMs: HYPER_PACE_WINDOW_MS,
  },);

  /**
   The transport behind the pacer: every attempt, retries included, waits
   for a place in the window before it goes out.

   @param exchange - request the retry ladder is sending

   @returns Transport's reply

   @example
   ```ts
   const reply = await pacedTransport(exchange,);
   ```
   */
  async function pacedTransport(
    exchange: Parameters<ModelTransport>[0],
  ): Promise<Awaited<ReturnType<ModelTransport>>> {
    await pace.take({ signal: exchange.signal, },);
    return await transport(exchange,);
  }

  /**
   Headers shared by every exchange, auth included.
   */
  const headers: Readonly<Record<string, string>> = {
    [HYPER_AUTH_HEADER]: `Bearer ${apiKey}`,
    'content-type': 'application/json',
    'anthropic-version': HYPER_API_VERSION,
  };

  /**
   Free-text chat exchange; bounded per model.

   @param request - exchange to perform

   @mutates request - `JSON.stringify` may invoke toJSON methods or getters while serializing messages and response format

   @returns Content text and usage when reported

   @throws {@link ModelNotServedError} when this provider serves no such model

   @throws {@link ProviderHttpError} on non-success status

   @throws {@link MalformedCompletionError} on a stream that never terminated or reported an error

   @example
   ```ts
   const reply = await client.chatText({ modelId, messages, signal, },);
   ```
   */
  async function chatText(request: ForeignBorrowed<ChatTextRequest>,): Promise<ChatTextReply> {
    /**
     Logger pre-tagged with this function's name.
     */
    const rl = tagged({
      tag: chatText.name,
      l,
    },);

    /**
     Wire spelling, resolved BEFORE the slot is taken so a misrouted call
     fails at once instead of queueing behind live ones.

     REACHED SYNCHRONOUSLY DESPITE THE `async`, because an async body runs up
     to its first `await`, so this still refuses before asking for a slot.
     The `async` is what turns its throw into a rejection rather than a
     synchronous exception out of a function that returns a promise.
     */
    const servedId = servedIdFor({ modelId: request.modelId, },);

    return await limiterFor(request.modelId,)(async function performExchange() {
      /**
       Message count for the entry log line.
       */
      const messageCount = request
        .messages
        .length;

      rl.debug(`-> ${servedId}: ${String(messageCount,)} ${
        wordForCount({
          count: messageCount,
          one: 'message',
          many: 'messages',
        },)
      }`,);

      /**
       Per-exchange deadline armed inside the slot so local queue wait
       behind concurrent same-model calls never counts against it;
       absent when the caller set no deadline.
       */
      using deadline = request.exchangeTimeoutMs === undefined
        ? undefined
        : armCallDeadline({
          signal: request.signal,
          timeoutMs: request.exchangeTimeoutMs,
          label: servedId,
        },);

      /**
       Signal the exchange honors: deadline-joined when armed.
       */
      const exchangeSignal = deadline === undefined
        ? request.signal
        : deadline.callSignal;

      /**
       Exactly what goes on the wire, hoisted so its size can be measured.

       THE SCHEMA IS STATED DOWNSTREAM, not here. `buildAnthropicBody` routes
       every schema-bearing call through `renderToolSystemPrompt`, which
       prints the whole schema into this protocol's `system` field along with
       its format rules, which was checked before adding a second copy.
       */
      const bodyJson = JSON.stringify(buildAnthropicBody({
        modelId: servedId,
        messages: request.messages,
        // Conditional spreads keep optional knobs absent instead of undefined.
        ...(request.responseFormat === undefined
          ? {}
          : { responseFormat: request.responseFormat, }),
        // THE MEASURED CEILING ON EVERY CALL since 2026-09-09
        // (`completion-cap.ts`), under this provider's own per-model
        // ceiling: a daily limit lasts longer when a runaway reply cannot
        // spend it.
        maxTokens: completionCapFor({
          modelId: request.modelId,
          // Conditional spread keeps the knob absent instead of undefined.
          ...(request.maxTokens === undefined
            ? {}
            : { requested: request.maxTokens, }),
        },),
      },),);

      /**
       Raw reply from the transport seam, retried on transient statuses.
       */
      const reply = await exchangeWithRetry({
        provider: 'hyper',
        transport: pacedTransport,
        exchange: {
          url: messagesUrl,
          label: servedId,
          method: 'POST',
          headers,
          bodyJson,
          signal: exchangeSignal,
          wireFormat: 'anthropic',
          // Conditional spread keeps the knob absent instead of undefined.
          ...(request.maxAnswerChars === undefined
            ? {}
            : { maxAnswerChars: request.maxAnswerChars, }),
        },
        policy: retryPolicy,
        // A TRUNCATED BODY IS A TRANSPORT FAILURE WEARING A SUCCESS STATUS,
        // so the ladder reads it inside its own try and retries the attempt.
        verify: wholeMessage,
      },);

      if (!isSuccessStatus({ status: reply.status, },)) {
        rl.warn(`<- ${servedId}: HTTP ${String(reply.status,)}`,);

        // BYTES RATHER THAN CHARACTERS: this corpus is Chinese, and one
        // character costs three bytes in UTF-8.
        throw failureForReply({
          provider: 'hyper',
          status: reply.status,
          bodyText: reply.bodyText,
          requestBodyBytes: Buffer.byteLength(bodyJson,),
        },);
      }

      /**
       Answer and usage reassembled from the drained event stream, whose
       answer is usually the tool's arguments rather than any text.
       */
      const extracted = extractAnthropicCompletion({
        provider: 'hyper',
        bodyText: reply.bodyText,
      },);

      /**
       Answer length for the completion log line.
       */
      const textLength = extracted
        .text
        .length;

      rl.debug(
        `<- ${servedId}: ${String(textLength,)} ${
          wordForCount({
            count: textLength,
            one: 'char',
            many: 'chars',
          },)
        }${formatUsageNote({ extracted, },)}`,
      );
      reportSpend({
        provider: 'hyper',
        label: servedId,
        extracted,
      },);
      return extracted;
    },);
  }

  /**
   JSON exchange over this client's `chatText` (`chat-json-through.ts`).
   */
  const chatJson = chatJsonThrough({ chatText, },);

  /**
   Reads the remaining balance, which is this provider's whole budget signal.

   @param signal - abort signal honored for the read

   @returns Typed balance

   @throws {@link ProviderHttpError} on non-success status

   @throws {@link CreditsShapeError} on contract-violating bodies

   @example
   ```ts
   const { balance, } = await client.credits({ signal, },);
   ```
   */
  async function credits(
    { signal, }: { readonly signal: AbortSignal; },
  ): Promise<HyperCredits> {
    /**
     Logger pre-tagged with this function's name.
     */
    const rl = tagged({
      tag: credits.name,
      l,
    },);

    /**
     Raw reply from the balance endpoint, retried on transient statuses.
     */
    const reply = await exchangeWithRetry({
      provider: 'hyper',
      transport: pacedTransport,
      exchange: {
        url: creditsUrl,
        label: 'credits',
        method: 'GET',
        headers,
        signal,
      },
      policy: retryPolicy,
    },);

    if (!isSuccessStatus({ status: reply.status, },))
      throw new ProviderHttpError({
        provider: 'hyper',
        status: reply.status,
        bodyText: reply.bodyText,
        summary: `Charm Hyper /credits returned HTTP ${String(reply.status,)}:`,
      },);

    /**
     Typed balance parsed from the verified body shape.
     */
    const parsed = parseHyperCredits({ bodyText: reply.bodyText, },);

    rl.debug(`balance ${String(parsed.balance,)}`,);
    return parsed;
  }

  return {
    chatText,
    chatJson,
    credits,
    pace: { waitMs: pace.waitMs, },
  };
}

//endregion Hyper client
