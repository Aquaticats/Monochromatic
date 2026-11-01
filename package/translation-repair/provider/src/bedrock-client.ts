import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  contextRoot,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type BedrockServedId,
  completionCapFor,
  perModelLimiter,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  BEDROCK_AUTH_HEADER,
  BEDROCK_MANTLE_BASE_URL,
  BEDROCK_MODELS,
  bedrockChatUrlFor,
} from './bedrock-catalog.ts';
import { ledgerAtBound, } from './bedrock-bound-ledger.ts';
import {
  BEDROCK_COST_UNREPORTED,
  bedrockCostOf,
} from './bedrock-cost.ts';
import type {
  BedrockCredits,
  BedrockLedger,
} from './bedrock-ledger.ts';
import {
  requireBedrockStreamEnd,
  withDoneSentinel,
} from './bedrock-stream-end.ts';
import { armCallDeadline, } from './call-deadline.ts';
import { armStreamBound, } from './stream-bound.ts';
import type {
  ChatTextReply,
  ChatTextRequest,
  ModelCaller,
} from './chat-contract.ts';
import { chatJsonThrough, } from './chat-json-through.ts';
import { isSuccessStatus, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  formatUsageNote,
  type MalformedCompletionError,
  type ModelTransport,
  type TransportReply,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { failureForReply, } from './request-size-refusal.ts';
import { bedrockIdFor, } from './roster-reach.ts';
import { withSchemaInSystemPrompt, } from './schema-prompt.ts';
import { reportSpend, } from './spend-line.ts';
import { extractStreamedCompletion, } from './stream-completion.ts';
import {
  DEFAULT_RETRY_POLICY,
  exchangeWithRetry,
  type RetryPolicy,
} from './transient-retry.ts';

//region Bedrock client
// Fourth provider's client, over the same transport seam as the first three.
//
// IT SPEAKS THE OPENAI-COMPATIBLE PROTOCOL THE OTHER CLIENTS SPEAK, by
// measurement on 2026-09-07 (`bedrock-catalog.ts` records the probes): chat
// completions with `stream` and `stream_options.include_usage`, a json_schema
// `response_format`, `max_tokens` (on every call since 2026-09-09, at the
// measured ceiling in `completion-cap.ts`). What this file adds
// are the things that ARE different: the route each model answers on, the
// terminator each route ends with, a cost computed here rather than read off
// the wire, and a meter that is a file rather than an endpoint.
//
// RAW FETCH BY THE OWNER'S INSTRUCTION of 2026-09-07 ("Please do not
// introduce Amazon Bedrock SDK, Anthropic SDK, OpenAI SDK because these are
// poorly written. Use raw fetch."): the transport seam is `fetch`.
//
// IT THROWS THE SAME FAILURE CLASS AS THE FIRST PROVIDER, for the reason
// `hyper-client.ts` records: `benchmark.ts` branches on it to read a status
// off a failed call, and a fresh class here would make that site blind to
// exactly the provider added to survive the others' exhaustion.
//
// NO REQUEST PACER AND NO PER-MODEL CEILING BY DEFAULT. The mantle endpoint
// enforces no requests-per-minute quota and publishes no token quota for
// these models ("their throughput is governed by internal service capacity",
// its quota page, read 2026-09-07); it asks for retry with backoff on
// throttling, which the transient ladder does, and for a gradual ramp, which
// the pipeline's slice fan-out is. A width past what a pass drives is
// unmeasured, which is why the limiter seam stays injectable.

/**
 Local representation of this provider's absence of a per-model ceiling.

 @example
 ```ts
 const width = BEDROCK_PER_MODEL_CONCURRENCY;
 ```
 */
export const BEDROCK_PER_MODEL_CONCURRENCY: number = Number.POSITIVE_INFINITY;

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
 throw new BedrockModelNotServedError({ modelId, },);
 ```
 */
export class BedrockModelNotServedError extends Error {
  /**
   Declares this message safe to forward: it names a model.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds failure naming the model this provider has no spelling for.

   @param modelId - roster model that was addressed here

   @example
   ```ts
   new BedrockModelNotServedError({ modelId: 'hf:Qwen/Qwen3.8-27B', },);
   ```
   */
  public constructor({ modelId, }: { readonly modelId: string; },) {
    super(`Bedrock does not serve ${modelId}; route it to another provider or pick another model`,);
    this.name = 'BedrockModelNotServedError';
  }
}

/**
 Client surface for the prepaid per-token USD provider.

 @example
 ```ts
 const client: BedrockClient = createBedrockClient({ apiKey, ledger, transport, },);
 ```
 */
export type BedrockClient = ModelCaller & {
  /**
   Credit, spend and what is left, off the ledger, which is this provider's
   whole budget signal.
   */
  readonly credits: (args: { readonly signal: AbortSignal; },) => Promise<BedrockCredits>;
};

/**
 Spells one roster model the way this provider names it.

 @param modelId - roster model the caller addressed

 @returns Wire identifier for the request body

 @throws {@link BedrockModelNotServedError} when this provider serves no such model

 @example
 ```ts
 const served = servedIdFor({ modelId, },);
 ```
 */
function servedIdFor(
  { modelId, }: { readonly modelId: RosterModelId; },
): BedrockServedId {
  /**
   Spelling this provider uses, or that it serves no such model.
   */
  const spelling = bedrockIdFor({ modelId, },);

  if (!spelling.served)
    throw new BedrockModelNotServedError({ modelId, },);
  return spelling.id;
}

/**
 Builds one client over injected transport, speaking chat completions.

 @param apiKey - bearer token; never logged

 @param ledger - durable spend record the meter reads and every priced call
 writes

 @param transport - HTTP the client calls over: `fetchTransport` in a run, a
 recorded one in a test. REQUIRED, since a test that left it out would reach
 the provider (ledger M43, X24)

 @param baseUrl - host both routes hang off, overridable for tests

 @param perModelConcurrency - optional local test or caller bound; normal
 operation is unbounded because the provider publishes no ceiling

 @param retryPolicy - transient-retry pacing; tests pass tiny backoffs

 @param streamBoundMsOverride - one stream bound for every model this client
 serves in place of each catalog entry's, or `'unbounded'` for none; absent
 reads the catalog, whose bound is armed on the real clock around every
 attempt. Recorded tests set a number so a bound can be crossed in
 milliseconds, and `'unbounded'` where the case reads something else, so no
 card's bound races a loaded machine (ledger B416)

 @returns Client surface with chatText, chatJson, and credits

 @example
 ```ts
 const client = createBedrockClient({ apiKey: process.env['TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY'] ?? '', ledger, transport: fetchTransport, },);
 ```
 */
export function createBedrockClient(
  {
    apiKey,
    ledger,
    transport,
    baseUrl = BEDROCK_MANTLE_BASE_URL,
    perModelConcurrency = BEDROCK_PER_MODEL_CONCURRENCY,
    retryPolicy = DEFAULT_RETRY_POLICY,
    streamBoundMsOverride,
  }: {
    readonly apiKey: string;
    readonly ledger: BedrockLedger;
    readonly transport: ModelTransport;
    readonly baseUrl?: string;
    readonly perModelConcurrency?: number;
    readonly retryPolicy?: RetryPolicy;
    readonly streamBoundMsOverride?: number | 'unbounded';
  },
): BedrockClient {
  /**
   Per-model limiter, created lazily for each model (`per-model-limiter.ts`).
   */
  const limiterFor = perModelLimiter({ perModelConcurrency, },);

  /**
   Headers shared by every exchange, auth included.
   */
  const headers: Readonly<Record<string, string>> = {
    [BEDROCK_AUTH_HEADER]: `Bearer ${apiKey}`,
    'content-type': 'application/json',
  };

  /**
   Free-text chat exchange; bounded per model where a bound was given.

   @param request - exchange to perform

   @mutates request - `JSON.stringify` may invoke toJSON methods or getters while serializing messages and response format

   @returns Content text and usage when reported

   @throws {@link BedrockModelNotServedError} when this provider serves no such model

   @throws {@link ProviderHttpError} on non-success status

   @throws {@link MalformedCompletionError} on a stream that never ended the way its route ends

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

    /**
     How this model's stream says it is whole, and how long a call to it may
     run before Bedrock is read as having queued it (class one hundred
     forty-eight).
     */
    const {
      streamEnd,
      streamBoundMs: catalogBoundMs,
    } = BEDROCK_MODELS[servedId];

    /**
     Bound this call runs under: the test override where one is set, a
     number or none at all, else the catalog's.
     */
    const streamBoundMs = streamBoundMsOverride ?? catalogBoundMs;

    /**
     Refuses a success reply whose stream carried an error chunk or never
     ended the way this route ends, so the ladder retries the attempt as the
     transport failure it is, or returns a refusal of the request unretried.

     @param attemptReply - one attempt's reply, read before the ladder returns it

     @throws InStreamRefusalError - when the stream carries an error chunk
     with a 4xx code other than 408 and 429

     @throws InStreamProviderError - when the stream carries any other error
     chunk

     @throws MalformedCompletionError - when the stream stops before its end
     */
    function wholeMessage(attemptReply: TransportReply,): void {
      if (!isSuccessStatus({ status: attemptReply.status, },))
        return;
      requireBedrockStreamEnd({
        bodyText: attemptReply.bodyText,
        streamEnd,
      },);
    }

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
       Signal the deadline leaves: deadline-joined when armed.
       */
      const deadlineSignal = deadline === undefined
        ? request.signal
        : deadline.callSignal;

      /**
       Transport each attempt of the ladder runs through: the card's stream
       bound armed around that one attempt, where the card measured one. Its
       cut is a `StreamBoundError`, which the ladder never retries and the
       router reads to hold Bedrock out for this model.

       PER ATTEMPT, NOT PER LADDER (ledger P13, 2026-09-28): the bound is a
       measurement of one stream (class one hundred forty-eight), and armed
       once around the ladder it counted a failed attempt and its backoff
       against the retry, so a healthy second stream could be cut and read as
       Bedrock queueing the model. The five cuts in the run logs each ran a
       single attempt to 60 s, so none was hit yet.
       */
      const attemptTransport: ModelTransport = (streamBoundMs === 'unbounded')
        ? transport
        : async function boundedAttempt(exchange,): Promise<TransportReply> {
          /**
           This attempt's bound, forwarding the deadline's abort into it.
           */
          using bound = armStreamBound({
            signal: exchange.signal,
            boundMs: streamBoundMs,
            label: servedId,
          },);
          return await transport({
            ...exchange,
            signal: bound.callSignal,
          },);
        };

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
       Ceiling this call sends, which also bounds what an abandoned attempt
       is ledgered at (ledger P1).
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

       NO THINKING PARAMETER, NO TOKEN BUDGET AND NO REASONING EFFORT: the
       owner's rulings of 2026-10-22, recorded in full at the Synthetic body,
       clear thinking parameters for first and second rate providers and keep
       them off third tier, and name no tier for Amazon Bedrock, so it is
       treated as third tier and not cleared. The Gemma 4 cards recommend a
       reasoning effort and this client sends none. A value that disables
       thinking is never sent, here or anywhere.
       */
      const bodyJson = JSON.stringify({
        model: servedId,
        messages: asked,
        stream: true,
        stream_options: { include_usage: true, },
        // THE MEASURED CEILING ON EVERY CALL since 2026-09-09
        // (`completion-cap.ts`): the credit behind this provider is never
        // topped up, and a runaway reply spends it.
        max_tokens: maxTokens,
        // Conditional spread keeps the optional knob absent instead of undefined.
        ...(request.responseFormat === undefined
          ? {}
          : { response_format: request.responseFormat, }),
      },);

      /**
       Raw reply from the transport seam, retried on transient statuses;
       every attempt Bedrock accepted and this pipeline did not finish is
       ledgered at its bound as it fails (ledger P1).
       */
      const reply = await exchangeWithRetry({
        provider: 'bedrock',
        onAbandonedAttempt: async function ledgerAbandoned(): Promise<void> {
          await ledgerAtBound({
            servedId,
            requestBodyBytes: Buffer.byteLength(bodyJson,),
            maxTokens,
            reckoning: 'abandoned-bound',
            ledger,
          },);
        },
        transport: attemptTransport,
        exchange: {
          url: bedrockChatUrlFor({
            baseUrl,
            servedId,
          },),
          label: servedId,
          method: 'POST',
          headers,
          bodyJson,
          signal: deadlineSignal,
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
          provider: 'bedrock',
          status: reply.status,
          bodyText: reply.bodyText,
          requestBodyBytes: Buffer.byteLength(bodyJson,),
        },);
      }

      /**
       Content and usage reassembled from the drained event stream, the
       sentinel supplied where this route ends on its usage chunk.
       */
      const extracted = extractStreamedCompletion({
        bodyText: withDoneSentinel({
          bodyText: reply.bodyText,
          streamEnd,
        },),
      },);

      /**
       USD this call cost, off the usage and the catalog's prices.
       */
      const cost = bedrockCostOf({
        servedId,
        extracted,
      },);

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
      /**
       Usage the provider reported, absent where it did not.
       */
      const { usage, } = extracted;
      // A WHOLE CALL WITH NO USAGE WAS STILL BILLED (ledger P1): it is written
      // at its bound, on the SPEND line and in the ledger, rather than
      // printed with its counts unreported and left out of the ledger.
      if ((cost === BEDROCK_COST_UNREPORTED) || (usage === undefined)) {
        await ledgerAtBound({
          servedId,
          requestBodyBytes: Buffer.byteLength(bodyJson,),
          maxTokens,
          reckoning: 'unreported-bound',
          ledger,
        },);
        return extracted;
      }
      reportSpend({
        provider: 'bedrock',
        label: servedId,
        extracted,
        costUsd: cost,
      },);
      await ledger.note({
        at: new Date().toISOString(),
        model: servedId,
        usd: cost,
        promptTokens: usage.prompt_tokens,
        completionTokens: usage.completion_tokens,
      },);
      return extracted;
    },);
  }

  /**
   JSON exchange over this client's `chatText` (`chat-json-through.ts`).
   */
  const chatJson = chatJsonThrough({ chatText, },);

  /**
   Reads credit, spend and what is left off the ledger, which is this
   provider's whole budget signal.

   @param signal - abort signal honored before the read

   @returns Typed credits

   @throws {@link import('./bedrock-ledger.ts').BedrockLedgerShapeError} on a ledger line that will not read

   @example
   ```ts
   const { remainingUsd, } = await client.credits({ signal, },);
   ```
   */
  async function credits(
    { signal, }: { readonly signal: AbortSignal; },
  ): Promise<BedrockCredits> {
    /**
     Logger pre-tagged with this function's name.
     */
    const rl = tagged({
      tag: credits.name,
      l,
    },);

    signal.throwIfAborted();

    /**
     What the ledger says.
     */
    const read = await ledger.read();

    /**
     What is left, for the log line.
     */
    const { remainingUsd, } = read;

    rl.debug(`remaining ${remainingUsd.toFixed(2,)} USD`,);
    return read;
  }

  return {
    chatText,
    chatJson,
    credits,
  };
}

//endregion Bedrock client
