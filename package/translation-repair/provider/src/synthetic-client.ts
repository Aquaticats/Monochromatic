import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  contextRoot,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import type {
  ChatTextReply,
  ChatTextRequest,
  ModelClient,
} from './chat-contract.ts';
import { chatJsonThrough, } from './chat-json-through.ts';
import {
  completionCapFor,
  perModelLimiter,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  formatUsageNote,
  type MalformedCompletionError,
  type ModelTransport,
  parseQuotaSnapshot,
  type QuotaShapeError,
  type QuotaSnapshot,
  ProviderHttpError,
  type TransportReply,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { isSuccessStatus, } from '@monochromatic-dev/translation-repair-markup/ts';
import { syntheticMeterLevel, } from './budget-routing.ts';
import { failureForReply, } from './request-size-refusal.ts';
import { withSchemaInSystemPrompt, } from './schema-prompt.ts';
import { reportSpend, } from './spend-line.ts';
import {
  SYNTHETIC_CHAT_BASE_URL,
  SYNTHETIC_QUOTAS_URL,
  syntheticServes,
} from './synthetic-catalog.ts';
import {
  extractStreamedCompletion,
  requireStreamTerminator,
} from './stream-completion.ts';
import { armCallDeadline, } from './call-deadline.ts';
import {
  DEFAULT_RETRY_POLICY,
  exchangeWithRetry,
  type RetryPolicy,
} from './transient-retry.ts';

//region Synthetic client
// Imperative-shell client over the Synthetic API. Provider protocol failures throw;
// model-content defects (refusal-shaped replies, schema mismatches) flow as data
// because unreliable model output is an ordinary input to this pipeline. Every call
// requires an AbortSignal so user steering can always abort in-flight fan-outs.
//
// NO LOCAL PER-MODEL CAP SINCE 2026-10-19. Synthetic's rate-limit page of that day (`https://synthetic.new/rate-limits`)
// describes only a weekly credit limit and names no concurrency limit, where the account had granted five concurrent requests
// per model (measured live on 2026-08-30, when raising one model to ten drew HTTP 429 responses). A call is therefore not queued
// locally. A refusal at the wire is handled as any 429 is, by the routing layer (`provider-router.ts`), and a caller can still hand
// a finite `perModelConcurrency`.

/**
 Concurrent requests available to each Synthetic model: no ceiling.

 POSITIVE INFINITY MEANS NO LIMIT, as for `HYPER_PER_MODEL_CONCURRENCY` and `OPENROUTER_PER_MODEL_CONCURRENCY`:
 `p-limit` accepts it as unbounded concurrency, and the router's slot ledger reads it as a provider that is never saturated.
 Until 2026-10-19 this was 5, measured live on 2026-08-30 with two zero-retry aggregate arms (5 calls on each of 4 active models
 completed together, while raising gpt-oss alone to 10 produced HTTP 429 responses); Synthetic's rate-limit page of 2026-10-19
 names no concurrency limit.

 @example
 ```ts
 const slots = SYNTHETIC_PER_MODEL_CONCURRENCY; // Infinity
 ```
 */
export const SYNTHETIC_PER_MODEL_CONCURRENCY: number = Number.POSITIVE_INFINITY;

/**
 Logger root for this package's model-facing shell.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Raised when a roster model is addressed to Synthetic and Synthetic has no row for it.

 BEFORE THE WIRE, NOT AFTER. The provider answers such a request with an HTTP
 400 saying the name should start with `hf:`, which the retry ladder does not
 retry and the stage layer records as one lost voice; over a run that is a
 seat failing every call while quorum is met by the rest. Refusing
 here names the actual condition, and a routed client never reaches it
 because reach is decided before the provider is chosen.

 @example
 ```ts
 throw new SyntheticModelNotServedError({ modelId: 'minimax-m3', },);
 ```
 */
export class SyntheticModelNotServedError extends Error {
  /**
   Declares this message safe to forward: a catalog id and our own words.
   */
  readonly messageNamesOnly: true = true;

  /**
   Roster model this provider cannot serve.
   */
  public readonly modelId: RosterModelId;

  /**
   Builds the refusal naming the model.

   @param modelId - roster model addressed to this provider

   @example
   ```ts
   throw new SyntheticModelNotServedError({ modelId: 'minimax-m3', },);
   ```
   */
  public constructor({ modelId, }: { readonly modelId: RosterModelId; },) {
    super(
      `Synthetic does not serve ${modelId}; it is a Charm Hyper endpoint label, so the call `
        + 'must go through the two-provider client createRunClient builds',
    );
    this.name = 'SyntheticModelNotServedError';
    this.modelId = modelId;
  }
}

/**
 Refuses a success reply whose server-sent stream stopped before its terminator.

 MODULE SCOPE BECAUSE IT CAPTURES NOTHING. The reply handed in is its whole
 input, so nesting it at the call site would make a closure over an empty set.

 ONLY A BODY THE STATUS ALREADY ACCEPTED. A non-success reply is reported by
 the status branch at the call site, which names the HTTP code; reading it
 here would replace that with a parse failure about an error page.

 @param attemptReply - one attempt's reply, read before the ladder returns it

 @throws MalformedCompletionError - when a success body stops before
 `[DONE]`, which is what puts a truncated stream on the retry path
 instead of past it

 @example
 ```ts
 const reply = await exchangeWithRetry({ provider: 'synthetic', transport, exchange, policy, verify: wholeMessage, },);
 ```
 */
function wholeMessage(attemptReply: TransportReply,): void {
  if (isSuccessStatus({ status: attemptReply.status, },))
    requireStreamTerminator({ bodyText: attemptReply.bodyText, },);
}

/**
 Builds one client over injected transport.
 Requests to the same model flow through a local limiter that grants every call
 at once by default, since Synthetic names no concurrency limit;
 a caller can hand a finite `perModelConcurrency`.

 @param apiKey - bearer token; never logged

 @param transport - HTTP the client calls over: `fetchTransport` in a run, a
 recorded one in a test. REQUIRED, since a test that left it out would reach
 the provider (ledger M43, X24)

 @param chatBaseUrl - OpenAI-compatible base, overridable for tests

 @param quotasUrl - quota endpoint, overridable for tests

 @param perModelConcurrency - concurrent requests granted to each model;
 unbounded by default, since the provider names no concurrency limit, and finite
 only where a test or caller bounds it

 @param retryPolicy - transient-retry pacing; tests pass tiny backoffs

 @returns Client surface with chatText, chatJson, and quotas

 @example
 ```ts
 const client = createSyntheticClient({ apiKey: process.env['TRANSLATION_REPAIR_SYNTHETIC_API_KEY'] ?? '', transport: fetchTransport, },);
 ```
 */
export function createSyntheticClient(
  {
    apiKey,
    transport,
    chatBaseUrl = SYNTHETIC_CHAT_BASE_URL,
    quotasUrl = SYNTHETIC_QUOTAS_URL,
    perModelConcurrency = SYNTHETIC_PER_MODEL_CONCURRENCY,
    retryPolicy = DEFAULT_RETRY_POLICY,
  }: {
    readonly apiKey: string;
    readonly transport: ModelTransport;
    readonly chatBaseUrl?: string;
    readonly quotasUrl?: string;
    readonly perModelConcurrency?: number;
    readonly retryPolicy?: RetryPolicy;
  },
): ModelClient {
  /**
   Per-model limiter, created lazily for each model (`per-model-limiter.ts`).
   */
  const limiterFor = perModelLimiter({ perModelConcurrency, },);

  /**
   Headers shared by every exchange.
   */
  const headers: Readonly<Record<string, string>> = {
    'Authorization': `Bearer ${apiKey}`,
    'Content-Type': 'application/json',
  };

  /**
   Free-text chat exchange; bounded per model.

   @param request - exchange to perform

   @mutates request - `JSON.stringify` may invoke toJSON methods or getters while serializing messages and response format

   @returns Content text and usage when reported

   @throws {@link ProviderHttpError} on non-success status

   @throws {@link MalformedCompletionError} on contract-violating bodies

   @example
   ```ts
   const reply = await client.chatText({ modelId, messages, signal, },);
   ```
   */
  function chatText(request: ForeignBorrowed<ChatTextRequest>,): Promise<ChatTextReply> {
    /**
     Logger pre-tagged with this function's name.
     */
    const rl = tagged({
      tag: chatText.name,
      l,
    },);

    /**
     Message count for the entry log line.
     */
    const messageCount = request
      .messages
      .length;

    return limiterFor(request.modelId,)(async function performExchange() {
      // BEFORE THE WIRE. A Charm Hyper endpoint label is not a name this
      // provider hosts; asking anyway cost a 400 per call and a seat's whole
      // voice for a run. The two-provider client is where it belongs,
      // and a caller that reached this client with it has been wired wrong.
      if (!syntheticServes(request.modelId,))
        throw new SyntheticModelNotServedError({ modelId: request.modelId, },);

      rl.debug(
        `-> ${request.modelId}: ${String(messageCount,)} ${
          wordForCount({
            count: messageCount,
            one: 'message',
            many: 'messages',
          },)
        }`,
      );

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
          label: request.modelId,
        },);

      /**
       Signal the exchange honors: deadline-joined when armed.
       */
      const exchangeSignal = deadline === undefined
        ? request.signal
        : deadline.callSignal;

      /**
       Messages as they go on the wire, carrying this call's own response
       schema inside the system prompt.

       THIS PROTOCOL HAS NOWHERE ELSE TO PUT IT. The Anthropic path states the
       schema in its own `system` field through `renderToolSystemPrompt`; an
       OpenAI-compatible body carries only `response_format`, which a model
       that does not honour that field never sees.
       */
      const asked = withSchemaInSystemPrompt({
        messages: request.messages,
        // Conditional spread keeps the knob absent instead of undefined.
        ...(request.responseFormat === undefined
          ? {}
          : { responseFormat: request.responseFormat, }),
      },);

      /**
       Exactly what goes on the wire, hoisted so its size can be measured.

       MEASURED, NOT ESTIMATED. The gateway caps this body and reports a body
       over the cap as a parse failure naming our JSON, so the only way to tell
       that refusal from a real malformation is to know how big this was.

       NO THINKING PARAMETER AND NO TOKEN BUDGET, BECAUSE SYNTHETIC IS THIRD
       TIER. The owner's rulings of 2026-10-22: "No, I'm only against
       thinking params on 3rd tier providers." "Anthropic counts as first
       rate, neuralwatt counts as second rate." "Xiaomi and deepseek count as
       first rate." "NeuralWatt does implement what they document correctly,
       so they're clear on thinking params." "But we should never disable
       thinking on anything." They narrowed the standing instruction of
       2026-08-25 ("Please don't set any thinking parameter or budget
       tokens... These providers and models have known issues with
       non-default thinking or budget tokens and we'd rather not step on the
       mines"), which now holds for third tier alone. Synthetic is third
       tier, and its own probes of 2026-10-22 give no reason to move it:
       `low` turned thinking off, and `high` and `max` did not separate
       (GLM-5.3-Flash at about 430 to 740 reasoning tokens for both, on a
       question where NeuralWatt's copy of the model separated `low` at 116
       to 143, `high` at 169 to 222 and `max` at 355 to 579; the figures are
       in `package/translation-repair/provider/README.md`, "Thinking-parameter
       evidence"). That covers `reasoning_effort` too, which is the same lever
       under the name this provider documents it by. A value that disables
       thinking is never sent, here or anywhere. Measurements that led here,
       and the levers left open, are in
       `doc/audit/where-a-round-spends-its-wall-clock.md`.
       */
      const bodyJson = JSON.stringify({
        model: request.modelId,
        messages: asked,
        // The provider is finicky without streaming, and streamed headers
        // arrive before fetch's default headers timeout can fire.
        stream: true,
        stream_options: { include_usage: true, },
        // THE MEASURED CEILING ON EVERY CALL since 2026-09-09
        // (`completion-cap.ts`): this provider meters a weekly token
        // allowance, and a runaway reply spends it.
        max_tokens: completionCapFor({
          modelId: request.modelId,
          // Conditional spread keeps the knob absent instead of undefined.
          ...(request.maxTokens === undefined
            ? {}
            : { requested: request.maxTokens, }),
        },),
        // Conditional spread keeps the optional knob absent instead of undefined.
        ...(request.responseFormat === undefined
          ? {}
          : { response_format: request.responseFormat, }),
      },);

      /**
       Raw reply from the transport seam, retried on transient statuses.
       */
      const reply = await exchangeWithRetry({
        provider: 'synthetic',
        transport,
        exchange: {
          url: `${chatBaseUrl}/chat/completions`,
          label: request.modelId,
          method: 'POST',
          headers,
          bodyJson,
          signal: exchangeSignal,
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
        rl.warn(`<- ${request.modelId}: HTTP ${String(reply.status,)}`,);

        // BYTES RATHER THAN CHARACTERS, which is the whole trap here. This
        // corpus is Chinese, and one character costs three bytes in UTF-8, so a
        // `.length` here would read a third of the wire size and never once fire
        // on the case this exists for.
        throw failureForReply({
          provider: 'synthetic',
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
       Content length for the completion log line.
       */
      const textLength = extracted
        .text
        .length;
      rl.debug(
        `<- ${request.modelId}: ${String(textLength,)} ${
          wordForCount({
            count: textLength,
            one: 'char',
            many: 'chars',
          },)
        }${formatUsageNote({ extracted, },)}`,
      );
      reportSpend({
        provider: 'synthetic',
        label: request.modelId,
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
   Reads the current quota snapshot.

   @param signal - abort signal honored for the read

   @returns Typed budget-relevant quota state

   @throws {@link ProviderHttpError} on non-success status

   @throws {@link QuotaShapeError} on contract-violating bodies

   @example
   ```ts
   const snapshot = await client.quotas({ signal, },);
   ```
   */
  async function quotas(
    { signal, }: { readonly signal: AbortSignal; },
  ): Promise<QuotaSnapshot> {
    /**
     Logger pre-tagged with this function's name.
     */
    const rl = tagged({
      tag: quotas.name,
      l,
    },);

    /**
     Raw reply from the quota endpoint, retried on transient statuses.
     */
    const reply = await exchangeWithRetry({
      provider: 'synthetic',
      transport,
      exchange: {
        url: quotasUrl,
        label: 'quotas',
        method: 'GET',
        headers,
        signal,
      },
      policy: retryPolicy,
    },);

    if (!isSuccessStatus({ status: reply.status, },)) {
      throw new ProviderHttpError({
        provider: 'synthetic',
        status: reply.status,
        bodyText: reply.bodyText,
        summary: `Synthetic /quotas returned HTTP ${String(reply.status,)}:`,
      },);
    }

    /**
     Typed snapshot parsed from the verified body shape.
     */
    const snapshot = parseQuotaSnapshot({ bodyText: reply.bodyText, },);

    /**
     The same fields the METERS record prints, so a reply with no five-hour window logs `syntheticFiveHour=none`.
     */
    const levels = syntheticMeterLevel({ quota: snapshot, },)
      .join(' ',);
    rl.debug(`quota ${levels}`,);
    return snapshot;
  }

  return {
    chatText,
    chatJson,
    quotas,
  };
}

//endregion Synthetic client
