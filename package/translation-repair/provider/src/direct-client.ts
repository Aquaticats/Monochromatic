import { tagged, } from '@monochromatic-dev/module-logger/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  contextRoot,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import {
  completionCapFor,
  perModelLimiter,
  type ProviderName,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  isSuccessStatus,
  wholeOpening,
} from '@monochromatic-dev/translation-repair-markup/ts';
import {
  type ExtractedCompletion,
  formatUsageNote,
  type ModelTransport,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { armCallDeadline, } from './call-deadline.ts';
import type {
  ChatTextReply,
  ChatTextRequest,
  JsonSchemaResponseFormat,
  ModelCaller,
} from './chat-contract.ts';
import { chatJsonThrough, } from './chat-json-through.ts';
import {
  NOT_REPORTED,
  systemFingerprintOf,
} from './direct-usage.ts';
import { requireWholeStream, } from './require-whole-stream.ts';
import { withoutRefusedKeyBody, } from './refused-key-body.ts';
import { failureForReply, } from './request-size-refusal.ts';
import { withSchemaInSystemPrompt, } from './schema-prompt.ts';
import { reportSpend, } from './spend-line.ts';
import { extractStreamedCompletion, } from './stream-completion.ts';
import {
  DEFAULT_RETRY_POLICY,
  exchangeWithRetry,
  type RetryPolicy,
} from './transient-retry.ts';

//region Direct client
// ONE OPENAI-COMPATIBLE CLIENT FOR THE TWO PROVIDERS THE OWNER'S OWN KEYS
// REACH DIRECTLY, DeepSeek and Xiaomi (2026-10-22), instead of two copies of
// `hyper-client.ts`. What the two share is everything a call does: a streamed
// chat completion with `stream_options.include_usage`, `max_tokens` at the
// measured ceiling, the response schema restated in the system prompt, the
// retry ladder with a whole-stream check, the per-call deadline, the spend
// line. What differs is a route: the endpoint, the model lock, how a response
// format goes on the wire, and how a call is priced. `deepseek-client.ts` and
// `xiaomi-client.ts` are the thin wrappers that hold one route each.
//
// MODELLED ON `neuralwatt-client.ts`, minus the energy ledger: neither provider
// meters by energy, and neither is reserved against before a call.
//
// NO REASONING OR THINKING PARAMETER YET, on either route. The owner's
// standing instruction of 2026-08-25 was lifted for NeuralWatt on 2026-10-21
// (`reasoning_effort`), and on 2026-10-22 for DeepSeek direct, Xiaomi direct and
// Anthropic as well: both of these routes are cleared for their documented
// thinking parameters. None is sent until an effort bench has measured one, and
// a value that disables thinking is never sent, here or anywhere. Both models
// reason by default and put the reasoning in `delta.reasoning_content`, which
// the stream reader drops (content is the answer channel), so reasoning tokens
// are billed and never shown.
//
// NO PER-MODEL CEILING BY DEFAULT in this factory; the wrappers set what each
// provider's documentation states (DeepSeek: 2,500 concurrent requests for
// `deepseek-flash`; Xiaomi: 100 requests a minute, paced).

/**
 Logger root for this package's model-facing shell.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 What distinguishes one direct provider from the other.

 @example
 ```ts
 const route: DirectRoute = deepSeekRoute({ baseUrl, },);
 ```
 */
export type DirectRoute = {
  /**
   Provider this route belongs to, named on every failure and spend line.
   */
  readonly provider: ProviderName;

  /**
   Absolute URL the chat body is posted to.
   */
  readonly chatUrl: string;

  /**
   Spells one roster model the way this provider names it, and is THE LOCK:
   it throws for every model the owner's key there is not granted for, so no
   request is built for one.

   @throws A not-served error of this provider's own, for any other model
   */
  readonly servedIdFor: (args: { readonly modelId: RosterModelId; }) => string;

  /**
   The `response_format` member this provider accepts for a call's schema.
   */
  readonly responseFormatOnWire: (
    args: { readonly responseFormat: ForeignBorrowed<JsonSchemaResponseFormat>; },
  ) => Readonly<Record<string, unknown>>;

  /**
   The cost and cache fields the spend line carries for one finished call,
   priced from the card.
   */
  readonly spendFieldsOf: (
    args: {
      readonly extracted: ExtractedCompletion;
      readonly bodyText: string;
    },
  ) => Pick<Parameters<typeof reportSpend>[0], 'costUsd' | 'cachedTokens'>;
};

/**
 Most UTF-16 units of a fingerprint a log line carries.
 */
const FINGERPRINT_LOG_UNITS = 80;

/**
 Wire member naming the model build, appended to the completion log line.

 @param bodyText - whole drained stream

 @returns The note, empty where the stream named no build

 @example
 ```ts
 const note = fingerprintNoteOf({ bodyText, },);
 ```
 */
function fingerprintNoteOf({ bodyText, }: { readonly bodyText: string; },): string {
  /**
   Build the stream named, when it did.
   */
  const fingerprint = systemFingerprintOf({ bodyText, },);
  // ENCODED AND BOUNDED, because the string is the provider's choice and
  // rides on a line whose grammar splits on spaces.
  return (fingerprint === NOT_REPORTED)
    ? ''
    : ` system_fingerprint=${encodeURIComponent(wholeOpening({
      text: fingerprint,
      units: FINGERPRINT_LOG_UNITS,
    },),)}`;
}

/**
 Client surface of a direct provider: the shared one every route has.

 @example
 ```ts
 const client: DirectClient = createDirectClient({ route, apiKey, transport, perModelConcurrency, },);
 ```
 */
export type DirectClient = ModelCaller;

/**
 Builds one client over injected transport, speaking chat completions to the
 route it is given.

 @param route - the provider's endpoint, model lock, wire format and price

 @param apiKey - bearer token; never logged

 @param transport - HTTP the client calls over: `fetchTransport` in a run, a
 recorded one in a test. REQUIRED, since a test that left it out would reach
 the provider (ledger M43, X24)

 @param perModelConcurrency - bound on concurrent calls to one model; the
 provider's stated ceiling, or unbounded where it states none

 @param retryPolicy - transient-retry pacing; tests pass tiny backoffs

 @returns Client surface with chatText and chatJson

 @example
 ```ts
 const client = createDirectClient({ route, apiKey: process.env['DEEPSEEK_API_KEY'] ?? '', transport: fetchTransport, perModelConcurrency: 2_500, },);
 ```
 */
export function createDirectClient(
  {
    route,
    apiKey,
    transport,
    perModelConcurrency,
    retryPolicy = DEFAULT_RETRY_POLICY,
  }: {
    readonly route: DirectRoute;
    readonly apiKey: string;
    readonly transport: ModelTransport;
    readonly perModelConcurrency: number;
    readonly retryPolicy?: RetryPolicy;
  },
): DirectClient {
  /**
   Transport that gives a refused key's 401 back without the provider's words.
   */
  const guardedTransport = withoutRefusedKeyBody({ transport, },);

  /**
   Per-model limiter, created lazily for each model (`per-model-limiter.ts`).
   */
  const limiterFor = perModelLimiter({ perModelConcurrency, },);

  /**
   Headers shared by every exchange, auth included.
   */
  const headers: Readonly<Record<string, string>> = {
    Authorization: `Bearer ${apiKey}`,
    'content-type': 'application/json',
  };

  /**
   Free-text chat exchange.

   @param request - exchange to perform

   @mutates request - `JSON.stringify` may invoke toJSON methods or getters while serializing messages and response
   format

   @returns Content text and usage when reported

   @throws A not-served error when the owner's key there is not granted for the model

   @throws {@link ProviderHttpError} on non-success status

   @throws {@link MalformedCompletionError} on a stream that never ended

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
     fails at once, with no request built, instead of queueing behind live ones.
     */
    const servedId = route.servedIdFor({ modelId: request.modelId, },);

    return await limiterFor(request.modelId,)(async function performExchange() {
      rl.debug(`-> ${servedId}: ${String(request.messages
        .length,)} ${
        wordForCount({
          count: request.messages
            .length,
          one: 'message',
          many: 'messages',
        },)
      }`,);

      /**
       Messages as they go on the wire, carrying this call's own response
       schema inside the system prompt, as on the Synthetic path.
       */
      const asked = withSchemaInSystemPrompt({
        messages: request.messages,
        // Conditional spread keeps the knob absent instead of undefined.
        ...(request.responseFormat === undefined ? {} : { responseFormat: request.responseFormat, }),
      },);

      /**
       Exactly what goes on the wire.

       NO THINKING PARAMETER, NO REASONING EFFORT AND NO TOKEN BUDGET YET: the
       owner's instruction of 2026-08-25, recorded in full at the Synthetic
       body, was lifted for this route on 2026-10-22, so a measured effort may
       be sent later; a value that disables thinking is never sent.
       */
      const bodyJson = JSON.stringify({
        model: servedId,
        messages: asked,
        stream: true,
        stream_options: { include_usage: true, },
        // THE MEASURED CEILING ON EVERY CALL since 2026-09-09
        // (`completion-cap.ts`): a runaway reply spends the owner's balance.
        // XIAOMI HONOURS `max_tokens` in the one probe made, though its chat reference lists only
        // `max_completion_tokens`: the parent session measured on 2026-10-22 on mimo-v2.6-flash
        // with a cap of 64 (one non-streamed call pair), both members returned finish_reason "length" with
        // completion_tokens 64, reasoning_tokens 22 included. The result lines are in
        // `doc/decision/translation-repair-direct-providers.md`, section "The Xiaomi token cap".
        max_tokens: completionCapFor({
          modelId: request.modelId,
          // Conditional spread keeps the knob absent instead of undefined.
          ...(request.maxTokens === undefined ? {} : { requested: request.maxTokens, }),
        },),
        // Conditional spread keeps the optional knob absent instead of undefined.
        ...(request.responseFormat === undefined
          ? {}
          : { response_format: route.responseFormatOnWire({ responseFormat: request.responseFormat, },), }),
      },);

      /**
       Per-exchange deadline armed inside the slot so local queue wait behind
       concurrent same-model calls never counts against it; absent when the
       caller set no deadline.
       */
      using deadline = request.exchangeTimeoutMs === undefined
        ? undefined
        : armCallDeadline({
          signal: request.signal,
          timeoutMs: request.exchangeTimeoutMs,
          label: servedId,
        },);

      /**
       Raw reply from the transport seam, retried on transient statuses.
       */
      const reply = await exchangeWithRetry({
        provider: route.provider,
        transport: guardedTransport,
        exchange: {
          url: route.chatUrl,
          label: servedId,
          method: 'POST',
          headers,
          bodyJson,
          signal: (deadline === undefined) ? request.signal : deadline.callSignal,
          // Conditional spread keeps the knob absent instead of undefined.
          ...(request.maxAnswerChars === undefined ? {} : { maxAnswerChars: request.maxAnswerChars, }),
        },
        policy: retryPolicy,
        // A TRUNCATED BODY IS A TRANSPORT FAILURE WEARING A SUCCESS STATUS,
        // so the ladder reads it inside its own try and retries the attempt.
        verify: requireWholeStream,
      },);
      if (!isSuccessStatus({ status: reply.status, },)) {
        rl.warn(`<- ${servedId}: HTTP ${String(reply.status,)}`,);

        // BYTES RATHER THAN CHARACTERS: this corpus is Chinese.
        throw failureForReply({
          provider: route.provider,
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
        }${formatUsageNote({ extracted, },)}${fingerprintNoteOf({ bodyText: reply.bodyText, },)}`,
      );
      reportSpend({
        provider: route.provider,
        label: servedId,
        extracted,
        ...route.spendFieldsOf({
          extracted,
          bodyText: reply.bodyText,
        },),
      },);
      return extracted;
    },);
  }

  return {
    chatText,
    chatJson: chatJsonThrough({ chatText, },),
  };
}

//endregion Direct client
