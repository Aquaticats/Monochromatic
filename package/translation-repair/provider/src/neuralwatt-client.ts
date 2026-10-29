import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  contextRoot,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import {
  completionCapFor,
  type NeuralWattServedId,
  perModelLimiter,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { isSuccessStatus, } from '@monochromatic-dev/translation-repair-markup/ts';
import { requireWholeStream, } from './require-whole-stream.ts';
import {
  formatUsageNote,
  type ModelTransport,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { armCallDeadline, } from './call-deadline.ts';
import {
  carriesPicture,
  type ChatTextReply,
  type ChatTextRequest,
  type ModelCaller,
} from './chat-contract.ts';
import { chatJsonThrough, } from './chat-json-through.ts';
import {
  NEURALWATT_AUTH_HEADER,
  NEURALWATT_BASE_URL,
  NEURALWATT_MODELS,
  neuralWattChatUrlFor,
} from './neuralwatt-catalog.ts';
import type { NeuralWattDayStart, } from './neuralwatt-day.ts';
import type { NeuralWattDayAnchor, } from './neuralwatt-day-anchor.ts';
import { createNeuralWattAnchorKeeper, } from './neuralwatt-day-anchor-keeper.ts';
import { createNeuralWattDaySession, } from './neuralwatt-day-session.ts';
import {
  NeuralWattAllowanceError,
  type NeuralWattEnergy,
} from './neuralwatt-energy.ts';
import { estimateNeuralWattWh, } from './neuralwatt-estimate.ts';
import { promptTokensOfRequest, } from './neuralwatt-picture-tokens.ts';
import type { NeuralWattLedger, } from './neuralwatt-ledger.ts';
import type { NeuralWattPlanSpend, } from './neuralwatt-quota.ts';
import { createNeuralWattQuotaSession, } from './neuralwatt-quota-session.ts';
import { settleNeuralWattCall, } from './neuralwatt-settlement.ts';
import { failureForReply, } from './request-size-refusal.ts';
import { neuralWattIdFor, } from './neuralwatt-reach.ts';
import { withSchemaInSystemPrompt, } from './schema-prompt.ts';
import { extractStreamedCompletion, } from './stream-completion.ts';
import {
  DEFAULT_RETRY_POLICY,
  exchangeWithRetry,
  type RetryPolicy,
} from './transient-retry.ts';

//region NeuralWatt client
// First provider's client, over the same transport seam as the other four.
//
// THE OWNER, 2026-10-20: "Use the neuralwatt free energy front and center."
// The provider catalog (`neuralwatt-catalog.ts`) has the rest of the owner's
// words and the facts about the API.
//
// IT SPEAKS THE OPENAI-COMPATIBLE PROTOCOL the Bedrock and OpenRouter clients
// speak: chat completions with `stream` and `stream_options.include_usage`, a
// json_schema `response_format` and the schema stated in the system prompt,
// `max_tokens` at the measured ceiling, and no thinking parameter beyond the
// documented `reasoning_effort`. NeuralWatt is second rate and cleared for its
// documented thinking parameters (the owner, 2026-10-21: "I'm lifting the ban on
// passing params, but only for NeuralWatt"; 2026-10-22: "NeuralWatt does
// implement what they document correctly, so they're clear on thinking
// params"). THE ONE PARAMETER SENT IS `reasoning_effort`, on a call to a model
// whose card names one: the `pictureReasoningEffort` on a call that carries a
// picture and the `textReasoningEffort` on one that does not. A model whose
// card names none gets none, and a value that disables thinking is never sent
// ("we should never disable thinking on anything"). No other client has that
// field to send.
//
// WHAT THIS CLIENT ADDS IS THE METER'S SIDE OF EVERY CALL, because overage
// "eats into my normal plan which I use for other stuff":
//
//   1. BEFORE ANY REQUEST, the call reserves its estimated energy in the
//      ledger (`neuralwatt-ledger.ts`), in the day the owner's account has:
//      "Every neuralwatt day starts at `15:15`" (America/New_York, `neuralwatt-day.ts`), counting
//      the larger of the ledger's spend and the whole account's covered energy
//      since then (`neuralwatt-day-session.ts`). When the estimate does not fit
//      in what is left of the day, or the day's start is unknown, or the key's
//      cap is blocked, nothing is sent and a `NeuralWattAllowanceError` is
//      thrown, which the router reads as a budget refusal and moves on from.
//   2. AFTER THE STREAM, the `: energy` comment's figure replaces the estimate
//      in the ledger. With no such comment the estimate stands, marked.
//   3. A `: cost` COMMENT IS REPORTED AND NEVER ACTED ON. Its
//      `request_cost_usd` goes on the `SPEND` line, and a figure above zero
//      ends nothing: NeuralWatt reports a nominal cost on free-energy requests
//      too (the owner's dashboard, 2026-10-20, counted a request that reported
//      0.000107 USD wholly against the free energy).
//   4. A QUOTA READING BEFORE THE FIRST CALL AND ONE AT THE END OF THE RUN
//      (`neuralwatt-quota-session.ts`) is the only signal of plan energy
//      spent, since the free energy is reported nowhere; the first also gives
//      the day's anchor.
//
// IT THROWS THE SAME FAILURE CLASS AS THE OTHER PROVIDERS for non-success
// replies (`ProviderHttpError`, provider `'neuralwatt'`), which the router and
// the benchmark branch on.
//
// NO PER-MODEL CEILING BY DEFAULT: NeuralWatt publishes none that was found
// (`https://docs.neuralwatt.com`, read 2026-10-20), and the in-flight hold is
// what bounds how much is risked at once. A width past what a pass drives is
// unmeasured, which is why the limiter seam stays injectable.

/**
 Local representation of this provider's absence of a per-model ceiling.

 @example
 ```ts
 const width = NEURALWATT_PER_MODEL_CONCURRENCY;
 ```
 */
export const NEURALWATT_PER_MODEL_CONCURRENCY: number = Number.POSITIVE_INFINITY;

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
 throw new NeuralWattModelNotServedError({ modelId, },);
 ```
 */
export class NeuralWattModelNotServedError extends Error {
  /**
   Declares this message safe to forward: it names a model.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds failure naming the model this provider has no spelling for.

   @param modelId - roster model that was addressed here

   @example
   ```ts
   new NeuralWattModelNotServedError({ modelId: 'hf:Qwen/Qwen3.8-27B', },);
   ```
   */
  public constructor({ modelId, }: { readonly modelId: string; },) {
    super(`NeuralWatt does not serve ${modelId}; route it to another provider or pick another model`,);
    this.name = 'NeuralWattModelNotServedError';
  }
}

/**
 Client surface for the free-daily-energy provider.

 @example
 ```ts
 const client: NeuralWattClient = createNeuralWattClient({ apiKey, ledger, transport, },);
 ```
 */
export type NeuralWattClient = ModelCaller & {
  /**
   The day's energy off the ledger, floored by the account's figure, and the
   key's cap: this provider's whole budget signal. Each reading refreshes the
   account figure and the cap.
   */
  readonly energy: (args: { readonly signal: AbortSignal; },) => Promise<NeuralWattEnergy>;

  /**
   Where this run's NeuralWatt day starts, as far as it has been decided: the time of day the quota's period start (or
   the day-start setting) gave, or that none is known. READ WITHOUT DECIDING, so no quota is read by asking: it is for
   a message built while the run ends, which states when the free energy returns.
   */
  readonly dayAnchor: () => NeuralWattDayAnchor;

  /**
   Reads the quota again and reports the plan energy spent since the first
   call of the run; call once, at the run's end.
   */
  readonly planSpend: (args: { readonly signal: AbortSignal; },) => Promise<NeuralWattPlanSpend>;

  /**
   Watt-hours this run's own calls drew, off the lines its ledger object appended (`NeuralWattLedger.drawnWh`): the
   figure to print beside `planSpend`, whose difference can include energy this run did not draw.
   */
  readonly drawnWh: () => number;
};

/**
 The system clock, the client's time outside a test.

 @returns Current instant

 @example
 ```ts
 const now = systemClock();
 ```
 */
function systemClock(): Date {
  return new Date();
}

/**
 Spells one roster model the way this provider names it.

 @param modelId - roster model the caller addressed

 @returns Wire identifier for the request body

 @throws {@link NeuralWattModelNotServedError} when this provider serves no such model

 @example
 ```ts
 const served = servedIdFor({ modelId, },);
 ```
 */
function servedIdFor({ modelId, }: { readonly modelId: RosterModelId; },): NeuralWattServedId {
  /**
   Spelling this provider uses, or that it serves no such model.
   */
  const spelling = neuralWattIdFor({ modelId, },);
  if (!spelling.served)
    throw new NeuralWattModelNotServedError({ modelId, },);
  return spelling.id;
}

/**
 Builds one client over injected transport, speaking chat completions.

 @param apiKey - bearer token; never logged

 @param ledger - durable energy record the meter reads and every call
 reserves against and settles into

 @param transport - HTTP the client calls over: `fetchTransport` in a run, a
 recorded one in a test. REQUIRED, since a test that left it out would reach
 the provider (ledger M43, X24)

 @param baseUrl - API base address, overridable for tests

 @param perModelConcurrency - optional local test or caller bound; normal
 operation is unbounded

 @param retryPolicy - transient-retry pacing; tests pass tiny backoffs

 @param now - clock the day is placed by; the system clock outside a test

 @param dayStartSetting - owner's `TRANSLATION_REPAIR_NEURALWATT_DAY_START`
 (`neuralWattDayStartFrom`), used where the quota does not give the day's start

 @returns Client surface with chatText, chatJson, energy, planSpend and drawnWh

 @example
 ```ts
 const client = createNeuralWattClient({ apiKey: process.env['TRANSLATION_REPAIR_NEURALWATT_API_KEY'] ?? '', ledger, transport: fetchTransport, },);
 ```
 */
export function createNeuralWattClient(
  {
    apiKey,
    ledger,
    transport,
    baseUrl = NEURALWATT_BASE_URL,
    perModelConcurrency = NEURALWATT_PER_MODEL_CONCURRENCY,
    retryPolicy = DEFAULT_RETRY_POLICY,
    now = systemClock,
    dayStartSetting = {},
  }: {
    readonly apiKey: string;
    readonly ledger: NeuralWattLedger;
    readonly transport: ModelTransport;
    readonly baseUrl?: string;
    readonly perModelConcurrency?: number;
    readonly retryPolicy?: RetryPolicy;
    readonly now?: () => Date;
    readonly dayStartSetting?: NeuralWattDayStart;
  },
): NeuralWattClient {
  /**
   Per-model limiter, created lazily for each model (`per-model-limiter.ts`).
   */
  const limiterFor = perModelLimiter({ perModelConcurrency, },);

  /**
   Headers shared by every exchange, auth included.
   */
  const headers: Readonly<Record<string, string>> = {
    [NEURALWATT_AUTH_HEADER]: `Bearer ${apiKey}`,
    'content-type': 'application/json',
  };

  /**
   The two quota readings of this client's run.
   */
  const quota = createNeuralWattQuotaSession({
    headers,
    baseUrl,
    transport,
    retryPolicy,
    now,
  },);

  /**
   Keeper of the day anchor, which learns it again while it is unknown and reports what it has decided.
   */
  const anchors = createNeuralWattAnchorKeeper({
    quota,
    setting: dayStartSetting,
    now,
  },);

  /**
   The day the account has, which the ledger counts in.
   */
  const days = createNeuralWattDaySession({
    headers,
    baseUrl,
    transport,
    retryPolicy,
    quota,
    now,
    anchors,
  },);

  /**
   Free-text chat exchange; reserved against the day's energy first.

   @param request - exchange to perform

   @mutates request - `JSON.stringify` may invoke toJSON methods or getters while serializing messages and response format

   @returns Content text and usage when reported

   @throws {@link NeuralWattModelNotServedError} when this provider serves no such model

   @throws {@link NeuralWattAllowanceError} when the day cannot take the call

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
     fails at once instead of queueing behind live ones.
     */
    const servedId = servedIdFor({ modelId: request.modelId, },);

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
       The efforts this model's card names, if any.
       */
      const {
        pictureReasoningEffort,
        textReasoningEffort,
      } = NEURALWATT_MODELS[servedId];

      /**
       Reasoning effort this call sends: the card's picture effort when the
       conversation carries a picture, else its text effort, and none where
       the card names none for the kind of call.
       */
      const effort = carriesPicture({ messages: request.messages, },) ? pictureReasoningEffort : textReasoningEffort;

      /**
       Exactly what goes on the wire, hoisted so its size can be measured.

       NeuralWatt is SECOND RATE AND CLEARED for its documented thinking
       parameters (the owner's rulings of 2026-10-22, recorded in full at the
       Synthetic body), and it sends one. `thinking_token_budget` stays out
       because its support table (2026-08-26) does not name `glm-5.3-flash`.

       THE ONE PARAMETER IS `reasoning_effort`, since 2026-10-21, when the owner
       lifted the ban for this provider, and only where the card's
       NeuralWatt side names one for the kind of call. `low` keeps thinking on,
       and no value that disables it is ever sent. `glm-5.3-flash` spent its
       whole completion cap reasoning at the default `max` on a dense
       screenshot (XingZ6016, ledger B719) and, on text, on 11 of 288 calls
       over the pass logs from zheermao19 to zheermao22 (ledger B807).
       `deepseek-v4.1-flash` gets none: the repo records no efforts for it.
       */
      const bodyJson = JSON.stringify({
        model: servedId,
        messages: asked,
        stream: true,
        stream_options: { include_usage: true, },
        // THE MEASURED CEILING ON EVERY CALL since 2026-09-09
        // (`completion-cap.ts`): a runaway reply spends the day's energy.
        max_tokens: completionCapFor({
          modelId: request.modelId,
          // Conditional spread keeps the knob absent instead of undefined.
          ...(request.maxTokens === undefined ? {} : { requested: request.maxTokens, }),
        },),
        // Conditional spread keeps the optional knob absent instead of undefined.
        ...(request.responseFormat === undefined ? {} : { response_format: request.responseFormat, }),
        // Conditional spread keeps the knob absent where the card names no effort.
        ...(effort === undefined ? {} : { reasoning_effort: effort, }),
      },);

      /**
       Prompt tokens the body stands for, a picture counted by its patches and
       not by its base64 (`neuralwatt-picture-tokens.ts`), and the energy held
       for them.
       */
      const promptTokens = promptTokensOfRequest({
        messages: asked,
        bodyBytes: Buffer.byteLength(bodyJson,),
      },);

      /**
       Energy this call holds while it is in flight.
       */
      const { estimateWh, } = estimateNeuralWattWh({
        servedId,
        promptTokens,
      },);

      /**
       The day the call is made in, which reads the quota once for its anchor
       when the run has not.
       */
      const day = await days.dayFor({
        signal: request.signal,
        refresh: false,
      },);

      /**
       Whether the day can take this call; THE HOLD IS TAKEN BEFORE ANYTHING IS
       SENT.
       */
      const outcome = await ledger.reserve({
        model: servedId,
        promptTokens,
        estimateWh,
        day,
      },);
      if (!outcome.granted)
        throw new NeuralWattAllowanceError({
          model: servedId,
          estimateWh,
          energy: outcome.energy,
        },);

      /**
       The hold this call took, given back when the call ends however it ends.
       */
      using held = outcome.reservation;

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
       Raw reply from the transport seam, retried on transient statuses; every
       attempt NeuralWatt accepted and this pipeline did not finish is noted in
       the ledger at its estimate as it fails.
       */
      const reply = await exchangeWithRetry({
        provider: 'neuralwatt',
        onAbandonedAttempt: held.noteAbandoned,
        transport,
        exchange: {
          url: neuralWattChatUrlFor({ baseUrl, },),
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
          provider: 'neuralwatt',
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
        }${formatUsageNote({ extracted, },)}`,
      );
      await settleNeuralWattCall({
        reservation: held,
        servedId,
        extracted,
        bodyText: reply.bodyText,
        estimateWh,
        promptTokens,
      },);
      return extracted;
    },);
  }

  /**
   JSON exchange over this client's `chatText` (`chat-json-through.ts`).
   */
  const chatJson = chatJsonThrough({ chatText, },);

  /**
   Reads the day's energy off the ledger, refreshing the account figure and
   the key's cap first.

   @param signal - abort signal honored before the read

   @returns Day's reading, holds in flight included

   @throws {@link import('./neuralwatt-ledger-config.ts').NeuralWattLedgerShapeError} on a ledger line that will not read

   @example
   ```ts
   const { remainingWh, } = await client.energy({ signal, },);
   ```
   */
  async function energy({ signal, }: { readonly signal: AbortSignal; },): Promise<NeuralWattEnergy> {
    signal.throwIfAborted();

    /**
     The day, with the account figure and the cap as they are now.
     */
    const day = await days.dayFor({
      signal,
      refresh: true,
    },);
    return await ledger.read({ day, },);
  }

  return {
    chatText,
    chatJson,
    energy,
    dayAnchor: anchors.decided,
    planSpend: quota.planSpend,
    drawnWh: ledger.drawnWh,
  };
}

//endregion NeuralWatt client
