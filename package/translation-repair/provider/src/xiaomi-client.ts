import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type {
  ExtractedCompletion,
  ModelTransport,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';
import type {
  JsonSchemaResponseFormat,
  ModelCaller,
} from './chat-contract.ts';
import {
  XIAOMI_BASE_URL,
  XIAOMI_MODELS,
  xiaomiChatUrlFor,
} from './direct-catalog.ts';
import {
  createDirectClient,
  type DirectRoute,
} from './direct-client.ts';
import {
  DIRECT_COST_UNREPORTED,
  xiaomiCostOf,
} from './direct-cost.ts';
import { xiaomiIdFor, } from './direct-reach.ts';
import {
  CACHED_UNREPORTED,
  openRouterCachedTokensOf,
} from './openrouter-cached-tokens.ts';
import {
  createRequestPace,
  type RequestPace,
} from './request-pace.ts';
import {
  DEFAULT_RETRY_POLICY,
  type RetryPolicy,
} from './transient-retry.ts';
import {
  xiaomiSpendNow,
  type XiaomiSpend,
} from './xiaomi-spend.ts';

//region Xiaomi client
// THE CLIENT OF XIAOMI'S OWN API, on the owner's pay-as-you-go key, for ONE
// MODEL.
//
// THE OWNER, 2026-10-22: "I've added TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY in
// main worktree's json file. ... You may only use v2.6-flash with this key."
// The key is not model-scoped (its `GET /v1/models` lists `mimo-v2.6-pro` and
// the speech models), so `servedIdFor` in this file is the lock: a roster model other
// than MiMo V2.6 Flash throws `XiaomiModelNotServedError` before any request is
// built. The key is a pay-as-you-go key, not a Token Plan `tp-` key, which
// Xiaomi's error list says it rejects with a 401.
//
// WHAT IS DIFFERENT FROM THE OTHER OPENAI-COMPATIBLE CLIENTS, all measured by
// the parent session on 2026-10-22:
//
//   - `response_format` of type `json_schema` with `strict: true` is accepted
//     (HTTP 200, a conforming reply), so a call's format goes on the wire
//     unchanged, as on the OpenRouter route; the schema is also in the system
//     prompt.
//   - Reasoning is on by default, in `delta.reasoning_content`; a stream may
//     carry a `:` comment line, which the reader skips.
//   - There is no balance endpoint (see `xiaomi-spend.ts`): the meter is this
//     run's own priced spend, and exhaustion is learned from `402 -
//     Insufficient Balance`.
//
// THE LIMIT: 100 requests and 10 million tokens a minute for `mimo-v2.6-flash`,
// across all keys of one account (`https://mimo.mi.com/docs/en-US/api/guidance/rate-limit`,
// read 2026-10-22), with a concurrency limit per account that the page gives
// no number for. The request rate is paced here, as Hyper's is
// (`request-pace.ts`): every attempt, retries included, takes a turn from a
// 100-in-a-minute window, and a full window reads as saturated to the router,
// which overflows the call to OpenRouter, the next provider serving the model.
// The token limit is far above anything one run sends.
//
// NO ZERO DATA RETENTION is promised by this route (unlike the OpenRouter route
// of the same model, which sends `zdr: true`). The owner's ruling of 2026-10-05
// (the corpus is public, PII is allowed) and the grant quoted here are what permit it.

/**
 Length of Xiaomi's request window: a minute, in milliseconds.
 */
export const XIAOMI_PACE_WINDOW_MS = 60_000;

/**
 Requests in any minute Xiaomi grants `mimo-v2.6-flash` on one account:
 `https://mimo.mi.com/docs/en-US/api/guidance/rate-limit`, read 2026-10-22.
 */
export const XIAOMI_REQUESTS_PER_MINUTE = 100;

/**
 Refusal raised when a roster model is not the one the Xiaomi key is granted
 for.

 A THROW RATHER THAN A DATA OUTCOME, because it is a routing mistake in our
 own code and not a thing a model did, and it is THE OWNER'S LOCK: the call
 never reaches the wire.

 @example
 ```ts
 throw new XiaomiModelNotServedError({ modelId, },);
 ```
 */
export class XiaomiModelNotServedError extends Error {
  /**
   Declares this message safe to forward: it names a model.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds failure naming the model this key is not granted for.

   @param modelId - roster model that was addressed here

   @example
   ```ts
   new XiaomiModelNotServedError({ modelId: 'hf:Qwen/Qwen3.8-27B', },);
   ```
   */
  public constructor({ modelId, }: { readonly modelId: string; },) {
    super(
      `Xiaomi's key is granted for MiMo V2.6 Flash only, so Xiaomi does not serve ${modelId}; `
        + 'route it to another provider or pick another model',
    );
    this.name = 'XiaomiModelNotServedError';
  }
}

/**
 Client surface for Xiaomi's own API.

 @example
 ```ts
 const client: XiaomiClient = createXiaomiClient({ apiKey, transport, },);
 ```
 */
export type XiaomiClient = ModelCaller & {
  /**
   What this run has spent here, which is this provider's meter: Xiaomi
   documents no balance, so the reading never decides dryness.
   */
  readonly spend: (args: { readonly signal: AbortSignal; },) => Promise<XiaomiSpend>;

  /**
   How long this client's request window would make a call wait now, which
   the router reads to send the model to OpenRouter while the window is full.
   */
  readonly pace: Pick<RequestPace, 'waitMs'>;
};

/**
 Spells one roster model the way Xiaomi names it, which is THE LOCK.

 @param modelId - roster model the caller addressed

 @returns Wire identifier for the request body

 @throws {@link XiaomiModelNotServedError} when the key is not granted for the model

 @example
 ```ts
 const served = servedIdFor({ modelId, },);
 ```
 */
function servedIdFor({ modelId, }: { readonly modelId: RosterModelId; },): string {
  /**
   Spelling this provider uses, or that it serves no such model.
   */
  const spelling = xiaomiIdFor({ modelId, },);
  if (!spelling.served)
    throw new XiaomiModelNotServedError({ modelId, },);
  return spelling.id;
}

/**
 The `response_format` member Xiaomi accepts: the schema, unchanged.

 THE SCHEMA THROUGH UNCHANGED: json_schema with strict was accepted live.

 @param responseFormat - the call's schema, borrowed from the caller

 @returns The same member, for the wire

 @example
 ```ts
 const member = schemaUnchanged({ responseFormat, },);
 ```
 */
function schemaUnchanged(
  { responseFormat, }: { readonly responseFormat: ForeignBorrowed<JsonSchemaResponseFormat>; },
): Readonly<Record<string, unknown>> {
  return responseFormat;
}

/**
 Prices one finished Xiaomi call with the cached count its stream reported.

 @param extracted - completion whose usage block Xiaomi filled in, or did not

 @param bodyText - whole drained stream, whose usage block names the cached count

 @returns The cost and cache fields the spend line carries, each absent where the stream reported nothing

 @example
 ```ts
 const fields = xiaomiSpendFields({ extracted, bodyText, },);
 ```
 */
function xiaomiSpendFields(
  {
    extracted,
    bodyText,
  }: {
    readonly extracted: ExtractedCompletion;
    readonly bodyText: string;
  },
): ReturnType<DirectRoute['spendFieldsOf']> {
  /**
   Prompt tokens served from the cache, where the stream said.
   */
  const cached = openRouterCachedTokensOf({ bodyText, },);
  /**
   Cache count the price reads: none where the stream said nothing.
   */
  const cachedTokens = (cached === CACHED_UNREPORTED) ? 0 : cached;
  /**
   The call priced, or that nothing was reported.
   */
  const cost = xiaomiCostOf({
    card: XIAOMI_MODELS['mimo-v2.6-flash'],
    extracted,
    cachedTokens,
  },);
  return {
    ...((cost === DIRECT_COST_UNREPORTED) ? {} : { costUsd: cost, }),
    ...((cached === CACHED_UNREPORTED) ? {} : { cachedTokens, }),
  };
}

/**
 Builds the route of Xiaomi's API.

 @param baseUrl - API base address, overridable for tests

 @returns The route the shared client calls

 @example
 ```ts
 const route = xiaomiRoute({ baseUrl: XIAOMI_BASE_URL, },);
 ```
 */
function xiaomiRoute({ baseUrl, }: { readonly baseUrl: string; },): DirectRoute {
  return {
    provider: 'xiaomi',
    chatUrl: xiaomiChatUrlFor({ baseUrl, },),
    servedIdFor,
    responseFormatOnWire: schemaUnchanged,
    spendFieldsOf: xiaomiSpendFields,
  };
}

/**
 Reads what this process has spent on Xiaomi, which is this provider's meter.

 @param signal - abort signal honored for the read

 @returns The priced spend

 @throws The signal's reason when it has already aborted

 @example
 ```ts
 const { spentUsd, } = await readSpend({ signal, },);
 ```
 */
async function readSpend({ signal, }: { readonly signal: AbortSignal; },): Promise<XiaomiSpend> {
  signal.throwIfAborted();
  await Promise.resolve();
  return xiaomiSpendNow();
}

/**
 Builds one client over injected transport, speaking chat completions to
 Xiaomi.

 @param apiKey - bearer token; never logged

 @param transport - HTTP the client calls over: `fetchTransport` in a run, a
 recorded one in a test. REQUIRED, since a test that left it out would reach
 the provider (ledger M43, X24)

 @param baseUrl - API base address, overridable for tests

 @param perModelConcurrency - optional local test or caller bound; normal
 operation is unbounded because the documentation gives the account's
 concurrency limit no number

 @param retryPolicy - transient-retry pacing; tests pass tiny backoffs

 @param requestsPerMinute - request starts allowed in any minute, retries
 included; the documented 100, a whole number of one or more, which
 `createRequestPace` refuses otherwise

 @returns Client surface with chatText, chatJson, spend and pace

 @example
 ```ts
 const client = createXiaomiClient({ apiKey: process.env['TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY'] ?? '', transport: fetchTransport, },);
 ```
 */
export function createXiaomiClient(
  {
    apiKey,
    transport,
    baseUrl = XIAOMI_BASE_URL,
    perModelConcurrency = Number.POSITIVE_INFINITY,
    retryPolicy = DEFAULT_RETRY_POLICY,
    requestsPerMinute = XIAOMI_REQUESTS_PER_MINUTE,
  }: {
    readonly apiKey: string;
    readonly transport: ModelTransport;
    readonly baseUrl?: string;
    readonly perModelConcurrency?: number;
    readonly retryPolicy?: RetryPolicy;
    readonly requestsPerMinute?: number;
  },
): XiaomiClient {
  /**
   Request-rate pacer every attempt on this client takes a turn from.
   */
  const pace = createRequestPace({
    perWindow: requestsPerMinute,
    windowMs: XIAOMI_PACE_WINDOW_MS,
  },);

  /**
   The transport behind the pacer: every attempt, retries included, waits
   for a place in the window before it goes out.

   @param exchange - request the retry ladder is sending

   @returns The transport's reply

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
   The chat surface, shared with DeepSeek's client.
   */
  const chat = createDirectClient({
    route: xiaomiRoute({ baseUrl, },),
    apiKey,
    transport: pacedTransport,
    perModelConcurrency,
    retryPolicy,
  },);

  return {
    ...chat,
    spend: readSpend,
    pace: { waitMs: pace.waitMs, },
  };
}

//endregion Xiaomi client
