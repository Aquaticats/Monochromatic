import { tagged, } from '@monochromatic-dev/module-logger/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  contextRoot,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import {
  ANTHROPIC_EFFORTS,
  ANTHROPIC_WIRE_MODELS,
  claudeCallDeadlineMs,
  completionCapFor,
  perModelLimiter,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { isSuccessStatus, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  formatUsageNote,
  type ModelTransport,
  type TransportReply,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { extractAnthropicCompletion, } from './anthropic-completion.ts';
import { requireWholeAnthropicMessage, } from './anthropic-whole-message.ts';
import { armCallDeadline, } from './call-deadline.ts';
import type {
  ChatTextReply,
  ChatTextRequest,
  ModelCaller,
} from './chat-contract.ts';
import { chatJsonThrough, } from './chat-json-through.ts';
import { buildClaudeBody, } from './claude-body.ts';
import {
  CLAUDE_MODELS,
  type ClaudeModelInfo,
  CLAUDE_API_VERSION,
  CLAUDE_AUTH_HEADER,
  CLAUDE_BASE_URL,
  CLAUDE_VERSION_HEADER,
  claudeMessagesUrlFor,
} from './claude-catalog.ts';
import {
  claudeCostOf,
} from './claude-cost.ts';
import { claudeIdFor, } from './claude-reach.ts';
import { claudeStreamFactsOf, } from './claude-stream-facts.ts';
import {
  createRequestPace,
  type RequestPace,
} from './request-pace.ts';
import {
  claudeSpendNow,
  type ClaudeSpend,
} from './claude-spend.ts';
import { isPaymentRefusal, } from './provider-budget-refusal.ts';
import { RETRY_AFTER_HEADER, } from './retry-after-header.ts';
import { failureForReply, } from './request-size-refusal.ts';
import { reportSpend, } from './spend-line.ts';
import {
  DEFAULT_RETRY_POLICY,
  exchangeWithRetry,
  type RetryPolicy,
} from './transient-retry.ts';
import { withoutRefusedKeyBody, } from './refused-key-body.ts';

//region Claude client
// THE CLIENT OF ANTHROPIC'S OWN API, on the owner's Claude key, for TWO MODELS: Sonnet 5.5 and Haiku 5.5.
//
// THE OWNER, 2026-10-22: "I've added TRANSLATION_REPAIR_CLAUDE_API_KEY . You may use latest Haiku and latest Sonnet
// with this key. This key would have $200 every month so be careful what you use." The key is not model-scoped (it
// lists Opus, Fable and older models), so the lock is in this file: a roster model that is not one of the eight Claude
// cards is refused before any request is built.
//
// A STREAMED MESSAGES CALL over the transport seam, read by the Messages reader Hyper's client uses
// (`anthropic-completion.ts`): `text_delta` is the answer and `thinking_delta` is dropped.

/**
 Logger root for this package's model-facing shell.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Length of the request window: a minute, in milliseconds.
 */
export const CLAUDE_PACE_WINDOW_MS = 60_000;

/**
 Requests in any minute the owner's key is stated to allow: 1000, beside 2,000,000 input and 400,000 output tokens a
 minute (the parent session's read of the key's limits, 2026-10-22). The client logs the limits the first reply
 actually carries, so a changed tier shows in the run's own log.
 */
export const CLAUDE_REQUESTS_PER_MINUTE = 1_000;

/**
 Response-header family Anthropic states its rate limits in.
 */
const RATE_LIMIT_HEADER_PREFIX = 'anthropic-ratelimit-';

/**
 Refusal raised when a roster model is not one of the eight Claude cards.

 A THROW RATHER THAN A DATA OUTCOME, because it is a routing mistake in our own code and not a thing a model did, and it
 is THE OWNER'S LOCK: the call never reaches the wire. The key lists Opus, Fable and older models too, and a request
 for any of them stops here.

 @example
 ```ts
 throw new ClaudeModelNotServedError({ modelId, },);
 ```
 */
export class ClaudeModelNotServedError extends Error {
  /**
   Declares this message safe to forward: it names a model.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds failure naming the model this key is not granted for.

   @param modelId - roster model that was addressed here

   @example
   ```ts
   new ClaudeModelNotServedError({ modelId: 'hf:Qwen/Qwen3.8-27B', },);
   ```
   */
  public constructor({ modelId, }: { readonly modelId: string; },) {
    super(
      `The Claude key is granted for Sonnet 5.5 and Haiku 5.5 only, so the Claude route does not serve ${modelId}; `
        + 'route it to another provider or pick another model',
    );
    this.name = 'ClaudeModelNotServedError';
  }
}

/**
 Client surface of the Claude route: the shared one every route has, and the request pace.

 @example
 ```ts
 const client: ClaudeClient = createClaudeClient({ apiKey, transport, },);
 ```
 */
export type ClaudeClient = ModelCaller & {
  /**
   Spend meter: the run's priced dollars on this key and whether billing has refused a call, read by the budget layer.
   */
  readonly spend: (args: { readonly signal: AbortSignal; },) => Promise<ClaudeSpend>;

  /**
   Request-rate view the router reads to see a full window as saturation.
   */
  readonly pace: Pick<RequestPace, 'waitMs'>;
};

/**
 A flag that can be raised and never lowered, so a fact met once in a run (a billing error, the first limits line)
 cannot be forgotten by a later call.

 CARRIED BY AN ABORT CONTROLLER, whose signal goes aborted once and stays: raising again changes nothing, and nothing
 here grows or rebinds.

 @returns The flag's reader and its raiser

 @example
 ```ts
 const refused = oneWayFlag();
 refused.raise();
 ```
 */
function oneWayFlag(): {
  readonly isRaised: () => boolean;
  readonly raise: () => void;
} {
  /**
   Aborted once the flag is raised.
   */
  const raised = new AbortController();
  return {
    isRaised: function isRaised(): boolean {
      return raised.signal
        .aborted;
    },
    raise: function raise(): void {
      raised.abort();
    },
  };
}

/**
 Refuses a success reply whose event stream stopped before its terminator or reported an error.

 @param attemptReply - one attempt's reply, read before the ladder returns it

 @example
 ```ts
 const reply = await exchangeWithRetry({ provider: 'anthropic', transport, exchange, policy, verify: wholeMessage, },);
 ```
 */
function wholeMessage(attemptReply: TransportReply,): void {
  if (isSuccessStatus({ status: attemptReply.status, },))
    requireWholeAnthropicMessage({
      provider: 'anthropic',
      bodyText: attemptReply.bodyText,
    },);
}

/**
 Finds the Claude card one roster model is, which is THE LOCK.

 @param modelId - roster model the caller addressed

 @returns The card, whose wire model is one of the two the key is granted for

 @throws {@link ClaudeModelNotServedError} when the model is not one of the eight Claude cards

 @example
 ```ts
 const card = cardFor({ modelId, },);
 ```
 */
function cardFor({ modelId, }: { readonly modelId: RosterModelId; },): ClaudeModelInfo {
  /**
   Spelling this route uses, or that it serves no such model.
   */
  const spelling = claudeIdFor({ modelId, },);
  if (!spelling.served)
    throw new ClaudeModelNotServedError({ modelId, },);

  /**
   The card's row of the catalog.
   */
  const row = CLAUDE_MODELS[spelling.id];
  // THE LOCK A SECOND TIME, AT RUN TIME: the types already hold every row to the two granted wire models and the three
  // efforts, and the cards' unit tests fail on a third, so these two branches cannot run unless a card is edited past
  // both. They stop the call before a request is built, which is the only place a stray model id would cost money;
  // `claude-client.unit.test.ts` edits a row past the types to run each.
  if (!(ANTHROPIC_WIRE_MODELS as readonly string[]).includes(row.wireModel,))
    throw new ClaudeModelNotServedError({ modelId: row.wireModel, },);
  if ((row.effort !== undefined) && (!(ANTHROPIC_EFFORTS as readonly string[]).includes(row.effort,)))
    throw new ClaudeModelNotServedError({ modelId: `${row.id} at effort ${row.effort}`, },);
  return row;
}

/**
 Builds one client over injected transport, speaking the Messages protocol to Anthropic.

 @param apiKey - key sent as `x-api-key`; never logged

 @param transport - HTTP the client calls over: `fetchTransport` in a run, a recorded one in a test. REQUIRED, since a
 test that left it out would reach the provider

 @param baseUrl - API base address, overridable for tests

 @param perModelConcurrency - bound on concurrent calls to one model; unbounded by default, since the account states
 request and token rates and no concurrency

 @param retryPolicy - transient-retry pacing; tests pass tiny backoffs

 @param requestsPerMinute - request starts allowed in any minute, retries included; the stated 1000, a whole number of
 one or more, which `createRequestPace` refuses otherwise

 @returns Client surface with chatText, chatJson, spend and pace

 @example
 ```ts
 const client = createClaudeClient({ apiKey: process.env['TRANSLATION_REPAIR_CLAUDE_API_KEY'] ?? '', transport: fetchTransport, },);
 ```
 */
export function createClaudeClient(
  {
    apiKey,
    transport,
    baseUrl = CLAUDE_BASE_URL,
    perModelConcurrency = Number.POSITIVE_INFINITY,
    retryPolicy = DEFAULT_RETRY_POLICY,
    requestsPerMinute = CLAUDE_REQUESTS_PER_MINUTE,
  }: {
    readonly apiKey: string;
    readonly transport: ModelTransport;
    readonly baseUrl?: string;
    readonly perModelConcurrency?: number;
    readonly retryPolicy?: RetryPolicy;
    readonly requestsPerMinute?: number;
  },
): ClaudeClient {
  /**
   Request-rate pacer every attempt on this client takes a turn from.
   */
  const pace = createRequestPace({
    perWindow: requestsPerMinute,
    windowMs: CLAUDE_PACE_WINDOW_MS,
  },);

  /**
   Raised when a call of this client meets a billing error, which holds the key dry for the rest of the run.
   */
  const billingRefused = oneWayFlag();

  /**
   Reads the spend meter.

   @param signal - abort signal of the reading

   @returns The priced spend and the billing flag

   @example
   ```ts
   const { billingRefused: dry, } = await readSpend({ signal, },);
   ```
   */
  async function readSpend({ signal, }: { readonly signal: AbortSignal; },): Promise<ClaudeSpend> {
    signal.throwIfAborted();
    await Promise.resolve();
    return claudeSpendNow({ billingRefused: billingRefused.isRaised(), },);
  }

  /**
   Raised once the rate-limit headers have been logged: once per client, which is once per run.
   */
  const limitsLogged = oneWayFlag();

  /**
   Logs the rate-limit headers a reply carries, the first time any reply does.

   @param attemptReply - one attempt's reply as the transport returned it

   @example
   ```ts
   logLimitsOnce(reply,);
   ```
   */
  function logLimitsOnce(attemptReply: TransportReply,): void {
    if (limitsLogged.isRaised())
      return;

    /**
     The rate-limit headers this reply carries, as `name=value` pairs.
     */
    const limits = Object.keys(attemptReply.headers ?? {},)
      .filter(function isLimit(name,): boolean {
        return name.startsWith(RATE_LIMIT_HEADER_PREFIX,);
      },)
      .map(function pairOf(name,): string {
        return `${name}=${attemptReply.headers?.[name] ?? ''}`;
      },);
    if (limits.length === 0)
      return;
    limitsLogged.raise();
    tagged({
      tag: createClaudeClient.name,
      l,
    },)
      .info(`rate limits on the first reply: ${limits.join(', ',)}`,);
  }

  /**
   The transport behind the pacer: every attempt, retries included, waits for a place in the window before it goes
   out, and the first reply that states limits has them logged.

   @param exchange - request the retry ladder is sending

   @returns The transport's reply

   @example
   ```ts
   const reply = await pacedTransport(exchange,);
   ```
   */
  async function pacedTransport(
    exchange: Parameters<ModelTransport>[0],
  ): Promise<TransportReply> {
    await pace.take({ signal: exchange.signal, },);

    /**
     The transport's reply, headers included when they were asked for.
     */
    const attemptReply = await transport(exchange,);
    logLimitsOnce(attemptReply,);
    return attemptReply;
  }

  /**
   Holds the key dry for the run when a failure is a billing error, whichever way it reached the client: as the reply
   of a non-success status, or as a thrown refusal (an `error` event under a success status, which the retry ladder
   returns unretried as a 402).

   @param failure - whatever the exchange produced

   @param label - what the call is named by in logs

   @example
   ```ts
   holdDryOnBilling({ failure, label, },);
   ```
   */
  function holdDryOnBilling({
    failure,
    label,
  }: {
    readonly failure: unknown;
    readonly label: string
  },): void {
    if (!isPaymentRefusal({ error: failure, },))
      return;
    billingRefused.raise();
    tagged({
      tag: holdDryOnBilling.name,
      l,
    },)
      .warn(`<- ${label}: billing error; the Claude key is held dry for the rest of the run`,);
  }

  /**
   Runs the retry ladder over one exchange, holding the key dry when it throws a billing error.

   @param ladder - everything the ladder takes

   @param label - what the call is named by in logs

   @returns The reply the ladder ended on

   @throws The ladder's own failure, after the billing flag has read it

   @example
   ```ts
   const reply = await exchangeHoldingDryOnBilling({ ladder, label, },);
   ```
   */
  async function exchangeHoldingDryOnBilling(
    {
      ladder,
      label,
    }: {
      readonly ladder: Parameters<typeof exchangeWithRetry>[0];
      readonly label: string
    },
  ): Promise<TransportReply> {
    try {
      return await exchangeWithRetry(ladder,);
    } catch (error) {
      holdDryOnBilling({
        failure: error,
        label,
      },);
      throw error;
    }
  }

  /**
   Transport that gives a refused key's 401 back without the provider's words.
   */
  const guardedTransport = withoutRefusedKeyBody({ transport: pacedTransport, },);

  /**
   Per-model limiter, created lazily for each model.
   */
  const limiterFor = perModelLimiter({ perModelConcurrency, },);

  /**
   Headers shared by every exchange, auth included.
   */
  const headers: Readonly<Record<string, string>> = {
    [CLAUDE_AUTH_HEADER]: apiKey,
    [CLAUDE_VERSION_HEADER]: CLAUDE_API_VERSION,
    'content-type': 'application/json',
  };

  /**
   Free-text chat exchange.

   @param request - exchange to perform

   @mutates request - `JSON.stringify` may invoke toJSON methods or getters while serializing messages and response
   format

   @returns Content text and usage when reported

   @throws {@link ClaudeModelNotServedError} when the model is not one of the eight Claude cards, before any request is
   built

   @throws {@link ClaudePrefillError} when the conversation ends on an assistant turn

   @throws {@link EmptyConversationError} when the messages cannot form a conversation

   @throws {@link ClaudeSchemaKeywordError} when the response format carries a keyword the structured-output feature
   refuses, before any exchange is made

   @throws {@link ProviderHttpError} on non-success status

   @throws {@link StreamErrorEventError} on an error event under a success status, which stands for the refusal it
   names

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
     Card the call is for, resolved BEFORE the slot is taken so a misrouted call fails at once, with no request built,
     instead of queueing behind live ones.
     */
    const card = cardFor({ modelId: request.modelId, },);

    /**
     What the call is named by in logs, on the exchange and on the SPEND line: the card, so the effort cards that
     share a wire model stay apart in the record.
     */
    const label = card.id;

    return await limiterFor(request.modelId,)(async function performExchange() {
      rl.debug(`-> ${label}: ${String(request.messages
        .length,)} ${
        wordForCount({
          count: request.messages
            .length,
          one: 'message',
          many: 'messages',
        },)
      }`,);

      /**
       Ceiling the call sends as `max_tokens`: the card's cap, or the caller's lower ask.
       */
      const maxTokens = completionCapFor({
        modelId: request.modelId,
        // Conditional spread keeps the knob absent instead of undefined.
        ...(request.maxTokens === undefined ? {} : { requested: request.maxTokens, }),
      },);

      /**
       Exactly what goes on the wire, hoisted so its size can be measured.
       */
      const bodyJson = JSON.stringify(buildClaudeBody({
        card,
        messages: request.messages,
        // Conditional spread keeps the member absent instead of undefined.
        ...(request.responseFormat === undefined ? {} : { responseFormat: request.responseFormat, }),
        maxTokens,
      },),);

      // THE CALLER'S DEADLINE IS A FLOOR UNDER A CLAUDE CALL'S OWN (R1 of the second review of the Claude route, kept
      // in `package/translation-repair/doc/claude-route-probe-figures-and-review-handles.md`). The run hands every call
      // 360,000 ms (`RUN_PER_CALL_TIMEOUT_MS`), and a completion of the 32,000-token cap needs 460,000 ms at the floor
      // rate of 80 tokens a second (`claudeCallDeadlineMs`), so below 89 tokens a second a call that thinks up to the
      // cap would be cut, billed for what it generated and answer nothing. The larger of the two is armed: a caller's
      // longer deadline stands, and a caller that set none still gets none, as in every client.
      /**
       Per-exchange deadline armed inside the slot, absent when the caller set none.
       */
      using deadline = request.exchangeTimeoutMs === undefined
        ? undefined
        : armCallDeadline({
          signal: request.signal,
          timeoutMs: Math.max(
            request.exchangeTimeoutMs,
            claudeCallDeadlineMs({ maxTokens, },),
          ),
          label,
        },);

      /**
       Raw reply from the transport seam, retried on transient statuses.
       */
      const reply = await exchangeHoldingDryOnBilling({
        label,
        ladder: {
          provider: 'anthropic',
          transport: guardedTransport,
          exchange: {
            url: claudeMessagesUrlFor({ baseUrl, },),
            label,
            method: 'POST',
            headers,
            bodyJson,
            signal: (deadline === undefined) ? request.signal : deadline.callSignal,
            wireFormat: 'anthropic',
            captureHeaders: [
              RETRY_AFTER_HEADER,
              `${RATE_LIMIT_HEADER_PREFIX}*`,
            ],
            // Conditional spread keeps the knob absent instead of undefined.
            ...(request.maxAnswerChars === undefined ? {} : { maxAnswerChars: request.maxAnswerChars, }),
          },
          policy: retryPolicy,
          verify: wholeMessage,
        },
      },);
      if (!isSuccessStatus({ status: reply.status, },)) {
        rl.warn(`<- ${label}: HTTP ${String(reply.status,)}`,);

        // BYTES RATHER THAN CHARACTERS: this corpus is Chinese.
        /**
         The failure this reply is, thrown once the billing flag has read it.
         */
        const failure = failureForReply({
          provider: 'anthropic',
          status: reply.status,
          bodyText: reply.bodyText,
          requestBodyBytes: Buffer.byteLength(bodyJson,),
          // Conditional spread keeps the member absent instead of undefined.
          ...((reply.headers === undefined) ? {} : { headers: reply.headers, }),
        },);
        // A BILLING ERROR (402, or the older 400 saying the credit balance is too low) EMPTIES THE KEY FOR THE RUN: the
        // flag is this client's own, so a call already in flight that settles cannot lift it.
        holdDryOnBilling({
          failure,
          label,
        },);
        throw failure;
      }

      /**
       Answer and usage reassembled from the drained event stream.
       */
      const extracted = extractAnthropicCompletion({
        provider: 'anthropic',
        bodyText: reply.bodyText,
      },);

      /**
       How the stream ended, beyond its text.
       */
      const facts = claudeStreamFactsOf({ bodyText: reply.bodyText, },);

      /**
       The cost and cache fields the SPEND line carries, each absent where the stream reported no usage to price.
       */
      const spendFields = (facts.usage === undefined)
        ? {}
        : {
          costUsd: claudeCostOf({
            prices: card.prices,
            usage: facts.usage,
          },),
          cachedTokens: facts.usage
            .cacheRead,
        };

      if (facts.refusal !== undefined) {
        /**
         Category the refusal named, in words for the log and the record.
         */
        const named = (facts.refusal
          .category
          === undefined)
          ? 'no category named'
          : `category ${facts.refusal
            .category}`;
        // A REFUSAL IS A 200 AND NOT AN OUTAGE: the model's classifier declined this one call, so the seat has no
        // answer for it and the provider stays where its meter puts it. Whatever text streamed before a mid-stream
        // decline is incomplete and is dropped. The tokens it spent are billed and reported as any call's are.
        rl.warn(`<- ${label}: refusal (${named}); the seat has no answer for this call`,);
        reportSpend({
          provider: 'anthropic',
          label,
          extracted,
          ...spendFields,
        },);
        return {
          text: '',
          refusal: `${label} refused the call (${named})`,
          finishReason: 'refusal',
          ...((extracted.usage === undefined) ? {} : { usage: extracted.usage, }),
        };
      }

      /**
       Answer length for the completion log line.
       */
      const textLength = extracted.text
        .length;
      rl.debug(`<- ${label}: ${String(textLength,)} ${
        wordForCount({
          count: textLength,
          one: 'char',
          many: 'chars',
        },)
      }${formatUsageNote({ extracted, },)}`,);
      reportSpend({
        provider: 'anthropic',
        label,
        extracted,
        ...spendFields,
      },);
      return extracted;
    },);
  }

  return {
    chatText,
    chatJson: chatJsonThrough({ chatText, },),
    spend: readSpend,
    pace: { waitMs: pace.waitMs, },
  };
}

//endregion Claude client
