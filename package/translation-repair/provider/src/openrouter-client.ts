import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  contextRoot,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { armCallDeadline, } from './call-deadline.ts';
import type {
  ChatTextReply,
  ChatTextRequest,
  ModelCaller,
} from './chat-contract.ts';
import { chatJsonThrough, } from './chat-json-through.ts';
import {
  completionCapFor,
  type OpenRouterServedId,
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
  OPENROUTER_AUTH_HEADER,
  OPENROUTER_CHAT_URL,
  OPENROUTER_CREDITS_URL,
  openRouterFirstContentWindowFor,
  openRouterProviderPreferencesFor,
} from './openrouter-catalog.ts';
import { reportAbandonedSpend, } from './openrouter-abandoned-spend.ts';
import {
  CACHED_UNREPORTED,
  openRouterCachedTokensOf,
} from './openrouter-cached-tokens.ts';
import {
  type EndpointReading,
  openRouterEndpointOf,
} from './openrouter-endpoint.ts';
import {
  COST_UNREPORTED,
  openRouterCostOf,
} from './openrouter-cost.ts';
import {
  type OpenRouterCredits,
  parseOpenRouterCredits,
} from './openrouter-credits.ts';
import { requireNoStreamError, } from './openrouter-stream-error.ts';
import { failureForReply, } from './request-size-refusal.ts';
import { openRouterIdFor, } from './roster-reach.ts';
import { withSchemaInSystemPrompt, } from './schema-prompt.ts';
import { reportSpend, } from './spend-line.ts';
import {
  extractStreamedCompletion,
  requireStreamTerminator,
} from './stream-completion.ts';
import {
  DEFAULT_RETRY_POLICY,
  exchangeWithRetry,
  type RetryPolicy,
} from './transient-retry.ts';

//region OpenRouter client
// Third provider's client, over the same transport seam as the first two.
//
// IT SPEAKS THE OPENAI-COMPATIBLE PROTOCOL THE SYNTHETIC CLIENT SPEAKS, by
// measurement rather than by analogy: the probe of 2026-09-03 drove every
// roster model twenty times through chat completions, the Anthropic Messages
// endpoint and the Responses endpoint, and chat completions conformed on
// every attempt and answered fastest on every model, where Messages answered
// Kimi-K3 eight times slower and conformed on 9 of 20 DeepSeek Flash attempts
// (`doc/planning/translation-repair-openrouter-2026-09-03.md`). So the body
// is the Synthetic body plus a `provider` field, the stream is read by the
// same reader, and the only things this file adds are the ones that ARE
// different: the spelling of the model name, the routing preferences, a
// credits meter in USD, and the cost the wire reports per call.
//
// IT THROWS THE SAME FAILURE CLASS AS THE FIRST PROVIDER, for the reason
// `hyper-client.ts` records: `benchmark.ts` branches on it to read a status
// off a failed call, and a fresh class here would make that site blind to
// exactly the provider added to survive the others' exhaustion.
//
// NO REQUEST PACER AND NO PER-MODEL CEILING BY DEFAULT. The provider states
// no request-rate limit for paid models beyond DDoS protection, and routes
// each model's calls across many upstreams; the probe ran twelve calls per
// model in flight without a refusal. A width past that is unmeasured, which
// is why the limiter seam stays injectable.

/**
 Local representation of this provider's absence of a per-model ceiling.

 @example
 ```ts
 const width = OPENROUTER_PER_MODEL_CONCURRENCY;
 ```
 */
export const OPENROUTER_PER_MODEL_CONCURRENCY: number = Number.POSITIVE_INFINITY;

/**
 Logger root for this package's model-facing shell.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Refusal raised when a roster model has no spelling on this provider.

 A THROW RATHER THAN A DATA OUTCOME, because it is a routing mistake in our
 own code and not a thing a model did.

 @example
 ```ts
 throw new OpenRouterModelNotServedError({ modelId, },);
 ```
 */
export class OpenRouterModelNotServedError extends Error {
  /**
   Declares this message safe to forward: it names a model.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds failure naming the model this provider has no spelling for.

   @param modelId - roster model that was addressed here

   @example
   ```ts
   new OpenRouterModelNotServedError({ modelId: 'hf:Qwen/Qwen3.8-27B', },);
   ```
   */
  public constructor({ modelId, }: { readonly modelId: string; },) {
    super(`OpenRouter does not serve ${modelId}; route it to another provider or pick another model`,);
    this.name = 'OpenRouterModelNotServedError';
  }
}

/**
 Client surface for the per-token USD provider.

 @example
 ```ts
 const client: OpenRouterClient = createOpenRouterClient({ apiKey, transport, },);
 ```
 */
export type OpenRouterClient = ModelCaller & {
  /**
   Credits purchased, used and remaining, which is this provider's whole
   budget signal.
   */
  readonly credits: (args: { readonly signal: AbortSignal; },) => Promise<OpenRouterCredits>;
};

/**
 Refuses a success reply whose server-sent stream carried a provider failure
 or stopped before its terminator.

 THE FAILURE IS ASKED FIRST. A stream the upstream failed mid-way carries an
 `error` chunk and no terminator, so the terminator check alone would name
 the framing ("cut off") where the wire named the cause (a code and an
 endpoint); `openrouter-stream-error.ts` records the day that misnaming cost.

 @param attemptReply - one attempt's reply, read before the ladder returns it

 @throws InStreamRefusalError - when a success body carries the gateway's
 error chunk with a 4xx code other than 408 and 429, which the ladder
 returns unretried as it returns the same refusal over plain HTTP

 @throws InStreamProviderError - when a success body carries any other error
 chunk, which puts the failed call on the retry path under its own name

 @throws MalformedCompletionError - when a success body stops before
 `[DONE]`, which is what puts a truncated stream on the retry path
 instead of past it

 @example
 ```ts
 const reply = await exchangeWithRetry({ provider: 'openrouter', transport, exchange, policy, verify: wholeMessage, },);
 ```
 */
function wholeMessage(attemptReply: TransportReply,): void {
  if (!isSuccessStatus({ status: attemptReply.status, },))
    return;
  requireNoStreamError({
    provider: 'openrouter',
    bodyText: attemptReply.bodyText,
  },);
  requireStreamTerminator({ bodyText: attemptReply.bodyText, },);
}

/**
 Spells one roster model the way this provider names it.

 @param modelId - roster model the caller addressed

 @returns Wire identifier for the request body

 @throws {@link OpenRouterModelNotServedError} when this provider serves no such model

 @example
 ```ts
 const served = servedIdFor({ modelId, },);
 ```
 */
function servedIdFor(
  { modelId, }: { readonly modelId: RosterModelId; },
): OpenRouterServedId {
  /**
   Spelling this provider uses, or that it serves no such model.
   */
  const spelling = openRouterIdFor({ modelId, },);

  if (!spelling.served)
    throw new OpenRouterModelNotServedError({ modelId, },);
  return spelling.id;
}

/**
 Fields of a spend report, each present only where the wire sent its value.

 THE CONDITIONAL SPREADS KEEP A FIELD ABSENT rather than zero or blank where
 no chunk reported one, so the line and the meter read unreported and no
 reader sums a zero the wire never sent.

 @param cost - USD the stream's cost module read, its unreported mark where
 no chunk carried a cost

 @param endpoint - upstream the gateway named, unreported where no chunk
 named one

 @param cachedTokens - cache count the stream's cached-tokens module read,
 its unreported mark where no chunk carried one

 @returns Fields to spread over a `reportSpend` call

 @example
 ```ts
 reportSpend({ provider: 'openrouter', label, extracted, ...reportedSpendFieldsOf({ cost, endpoint, cachedTokens, }), },);
 ```
 */
export function reportedSpendFieldsOf(
  {
    cost,
    endpoint,
    cachedTokens,
  }: {
    readonly cost: number | typeof COST_UNREPORTED;
    readonly endpoint: EndpointReading;
    readonly cachedTokens: number | typeof CACHED_UNREPORTED;
  },
): Pick<Parameters<typeof reportSpend>[0], 'costUsd' | 'endpoint' | 'cachedTokens'> {
  return {
    ...((cost === COST_UNREPORTED) ? {} : { costUsd: cost, }),
    ...(endpoint.reported ? { endpoint: endpoint.name, } : {}),
    ...((cachedTokens === CACHED_UNREPORTED) ? {} : { cachedTokens, }),
  };
}

/**
 Builds one client over injected transport, speaking chat completions.

 @param apiKey - bearer token; never logged

 @param transport - HTTP the client calls over: `fetchTransport` in a run, a
 recorded one in a test. REQUIRED, since a test that left it out would reach
 the provider (ledger M43, X24)

 @param chatUrl - completion endpoint, overridable for tests

 @param creditsUrl - credits endpoint, overridable for tests

 @param perModelConcurrency - optional local test or caller bound; normal
 operation is unbounded because the provider states no ceiling

 @param retryPolicy - transient-retry pacing; tests pass tiny backoffs

 @returns Client surface with chatText, chatJson, and credits

 @example
 ```ts
 const client = createOpenRouterClient({ apiKey: process.env['TRANSLATION_REPAIR_OPENROUTER_API_KEY'] ?? '', transport: fetchTransport, },);
 ```
 */
export function createOpenRouterClient(
  {
    apiKey,
    transport,
    chatUrl = OPENROUTER_CHAT_URL,
    creditsUrl = OPENROUTER_CREDITS_URL,
    perModelConcurrency = OPENROUTER_PER_MODEL_CONCURRENCY,
    retryPolicy = DEFAULT_RETRY_POLICY,
  }: {
    readonly apiKey: string;
    readonly transport: ModelTransport;
    readonly chatUrl?: string;
    readonly creditsUrl?: string;
    readonly perModelConcurrency?: number;
    readonly retryPolicy?: RetryPolicy;
  },
): OpenRouterClient {
  /**
   Per-model limiter, created lazily for each model (`per-model-limiter.ts`).
   */
  const limiterFor = perModelLimiter({ perModelConcurrency, },);

  /**
   Headers shared by every exchange, auth included.
   */
  const headers: Readonly<Record<string, string>> = {
    [OPENROUTER_AUTH_HEADER]: `Bearer ${apiKey}`,
    'content-type': 'application/json',
  };

  /**
   Free-text chat exchange; bounded per model where a bound was given.

   @param request - exchange to perform

   @mutates request - `JSON.stringify` may invoke toJSON methods or getters while serializing messages and response format

   @returns Content text and usage when reported

   @throws {@link OpenRouterModelNotServedError} when this provider serves no such model

   @throws {@link ProviderHttpError} on non-success status

   @throws {@link MalformedCompletionError} on a stream that never terminated

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
       Messages as they go on the wire, carrying this call's own response
       schema inside the system prompt, as on the Synthetic path.
       */
      const asked = withSchemaInSystemPrompt({
        messages: request.messages,
        // Conditional spread keeps the knob absent instead of undefined.
        ...(request.responseFormat === undefined
          ? {}
          : { responseFormat: request.responseFormat, }),
      },);

      /**
       Ceiling this call sends, which also bounds what an abandoned stream
       is reckoned to have cost (ledger P7).
       */
      const maxTokens = completionCapFor({
        modelId: request.modelId,
        // Conditional spread keeps the knob absent instead of undefined.
        ...(request.maxTokens === undefined
          ? {}
          : { requested: request.maxTokens, }),
      },);

      /**
       Exactly what goes on the wire, hoisted so its size can be measured.

       NO THINKING PARAMETER AND NO REASONING BUDGET: the owner's rulings of
       2026-10-22, recorded in full at the Synthetic body, clear thinking
       parameters for first and second rate providers and keep them off third
       tier, and name no tier for OpenRouter, so it is treated as third tier
       and not cleared. A value that disables thinking is never sent, here or
       anywhere. `max_tokens` IS ALWAYS SENT since 2026-09-09, at the measured
       ceiling in `completion-cap.ts` or a caller's lower one:
       this is the per-token provider, and a stream a round abandoned kept
       billing to its own end on the endpoints that do not honour a cancel.
       */
      const bodyJson = JSON.stringify({
        model: servedId,
        messages: asked,
        stream: true,
        stream_options: { include_usage: true, },
        provider: openRouterProviderPreferencesFor({ servedId, },),
        max_tokens: maxTokens,
        // Conditional spread keeps the optional knob absent instead of undefined.
        ...(request.responseFormat === undefined
          ? {}
          : { response_format: request.responseFormat, }),
      },);

      /**
       Size of the body every attempt sends, for the reckoning.
       */
      const requestBodyBytes = Buffer.byteLength(bodyJson,);

      /**
       Raw reply from the transport seam, retried on transient statuses;
       every attempt that delivered something and failed writes its reckoned
       spend line, the ones the ladder retried included (since 2026-09-09 for
       the last attempt, ledger P1 for the rest).
       */
      const reply = await exchangeWithRetry({
        provider: 'openrouter',
        transport,
        exchange: {
          url: chatUrl,
          label: servedId,
          method: 'POST',
          headers,
          bodyJson,
          signal: exchangeSignal,
          // Conditional spread keeps the knob absent instead of undefined.
          ...(request.maxAnswerChars === undefined
            ? {}
            : { maxAnswerChars: request.maxAnswerChars, }),
          // The card's own first-content window, where it names one (ledger B809).
          ...openRouterFirstContentWindowFor({ servedId, }),
        },
        policy: retryPolicy,
        // A TRUNCATED BODY IS A TRANSPORT FAILURE WEARING A SUCCESS STATUS,
        // so the ladder reads it inside its own try and retries the attempt.
        verify: wholeMessage,
        onAbandonedAttempt: function reckonAbandoned({ deliveredChars, },): Promise<void> {
          reportAbandonedSpend({
            servedId,
            deliveredChars,
            requestBodyBytes,
            maxTokens,
          },);
          // The ladder's contract is asynchronous for the Bedrock ledger's
          // write; this reckoning is a log line and a meter, both synchronous.
          return Promise.resolve();
        },
      },);

      if (!isSuccessStatus({ status: reply.status, },)) {
        rl.warn(`<- ${servedId}: HTTP ${String(reply.status,)}`,);

        // BYTES RATHER THAN CHARACTERS: this corpus is Chinese, and one
        // character costs three bytes in UTF-8.
        throw failureForReply({
          provider: 'openrouter',
          status: reply.status,
          bodyText: reply.bodyText,
          requestBodyBytes: Buffer.byteLength(bodyJson,),
        },);
      }

      /**
       Content and usage reassembled from the drained event stream.
       */
      const extracted = extractStreamedCompletion({ bodyText: reply.bodyText, },);

      /**
       USD this call was charged, as the final chunk reported it.
       */
      const cost = openRouterCostOf({ bodyText: reply.bodyText, },);

      /**
       Upstream the gateway named as serving this call.
       */
      const endpoint = openRouterEndpointOf({ bodyText: reply.bodyText, },);

      /**
       Prompt tokens the upstream served from its cache, where it said.
       */
      const cachedTokens = openRouterCachedTokensOf({ bodyText: reply.bodyText, },);

      /**
       Content length for the completion log line.
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
        provider: 'openrouter',
        label: servedId,
        extracted,
        ...reportedSpendFieldsOf({
          cost,
          endpoint,
          cachedTokens,
        },),
      },);
      return extracted;
    },);
  }

  /**
   JSON exchange over this client's `chatText` (`chat-json-through.ts`).
   */
  const chatJson = chatJsonThrough({ chatText, },);

  /**
   Reads credits purchased and used, which is this provider's whole budget
   signal.

   @param signal - abort signal honored for the read

   @returns Typed credits

   @throws {@link ProviderHttpError} on non-success status

   @throws {@link import('./openrouter-credits.ts').OpenRouterCreditsShapeError} on contract-violating bodies

   @example
   ```ts
   const { remainingUsd, } = await client.credits({ signal, },);
   ```
   */
  async function credits(
    { signal, }: { readonly signal: AbortSignal; },
  ): Promise<OpenRouterCredits> {
    /**
     Logger pre-tagged with this function's name.
     */
    const rl = tagged({
      tag: credits.name,
      l,
    },);

    /**
     Raw reply from the credits endpoint, retried on transient statuses.
     */
    const reply = await exchangeWithRetry({
      provider: 'openrouter',
      transport,
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
        provider: 'openrouter',
        status: reply.status,
        bodyText: reply.bodyText,
        summary: `OpenRouter /credits returned HTTP ${String(reply.status,)}:`,
      },);

    /**
     Typed credits parsed from the verified body shape.
     */
    const parsed = parseOpenRouterCredits({ bodyText: reply.bodyText, },);

    /**
     What is left, for the log line.
     */
    const { remainingUsd, } = parsed;

    rl.debug(`remaining ${remainingUsd.toFixed(2,)} USD`,);
    return parsed;
  }

  return {
    chatText,
    chatJson,
    credits,
  };
}

//endregion OpenRouter client
