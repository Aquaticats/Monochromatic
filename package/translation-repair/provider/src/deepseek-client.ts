import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { isSuccessStatus, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  ProviderHttpError,
  type ExtractedCompletion,
  type ModelTransport,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';
import type { ModelCaller, } from './chat-contract.ts';
import {
  type DeepSeekBalance,
  parseDeepSeekBalance,
} from './deepseek-balance.ts';
import {
  DEEPSEEK_BASE_URL,
  DEEPSEEK_MODELS,
  DIRECT_AUTH_HEADER,
  deepSeekBalanceUrlFor,
  deepSeekChatUrlFor,
} from './direct-catalog.ts';
import {
  createDirectClient,
  type DirectRoute,
} from './direct-client.ts';
import {
  DIRECT_COST_UNREPORTED,
  deepSeekCostOf,
} from './direct-cost.ts';
import { deepSeekIdFor, } from './direct-reach.ts';
import { withoutRefusedKeyBody, } from './refused-key-body.ts';
import { deepSeekCacheSplitOf, } from './direct-usage.ts';
import {
  DEFAULT_RETRY_POLICY,
  exchangeWithRetry,
  type RetryPolicy,
} from './transient-retry.ts';

//region DeepSeek client
// THE CLIENT OF DEEPSEEK'S OWN API, on the owner's key, for ONE MODEL.
//
// THE OWNER, 2026-10-22: "You can also use the DEEPSEEK_API_KEY as you wish,
// but you may only use deepseek v4.1 flash 1017 with that key." The key is not
// model-scoped, so `servedIdFor` in this file is the lock: a roster model other than
// DeepSeek V4.1 Flash throws `DeepSeekModelNotServedError` before any request
// is built. The wire id is `deepseek-flash`, which DeepSeek's documentation
// says is served by DeepSeek-V4.1-Flash-1017 (no dated id exists).
//
// WHAT IS DIFFERENT FROM THE OTHER OPENAI-COMPATIBLE CLIENTS, all measured by
// the parent session on 2026-10-22:
//
//   - `response_format` of type `json_schema` is REFUSED (HTTP 400, "This
//     response_format type is unavailable now"). `{type: 'json_object'}` is
//     accepted, so a call that carries a schema sends `json_object` and the
//     schema stays in the system prompt (`schema-prompt.ts`), whose text holds
//     the word "json" that DeepSeek's documentation requires in the prompt for
//     JSON output. The documentation also warns the API "may occasionally
//     return empty content", which reaches the caller as a schema mismatch.
//   - Reasoning is on by default and arrives in `delta.reasoning_content`.
//     Streams end in `[DONE]`; the documentation says a streaming request
//     that waits receives `: keep-alive` comment lines, which the stream
//     reader skips, and that a request not started after 10 minutes is closed.
//   - The meter is `GET /user/balance` (`credits` on the client built here).
//
// THE LIMIT: 2,500 concurrent requests for `deepseek-flash` per account, over
// which DeepSeek answers HTTP 429 (`https://api-docs.deepseek.com/quick_start/rate_limit`,
// read 2026-10-22), which is the per-model concurrency this client grants. No
// request-rate limit is stated.
//
// EXHAUSTION: `402 - Insufficient Balance` ("You have run out of balance",
// `https://api-docs.deepseek.com/quick_start/error_codes`, read 2026-10-22),
// which the router passes to `markRefused` as a payment refusal. A 401 holds
// the provider out for the run (`credential-hold.ts`).
//
// NO ZERO DATA RETENTION is promised by this route. The owner's ruling of
// 2026-10-05 (the corpus is public, PII is allowed) and the grant quoted here are
// what permit it.

/**
 Logger root for this package's model-facing shell.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Concurrent requests DeepSeek grants `deepseek-flash` on one account:
 `https://api-docs.deepseek.com/quick_start/rate_limit`, read 2026-10-22
 ("2500 for `deepseek-flash`"; HTTP 429 when exceeded).

 @example
 ```ts
 const width = DEEPSEEK_PER_MODEL_CONCURRENCY;
 ```
 */
export const DEEPSEEK_PER_MODEL_CONCURRENCY = 2_500;

/**
 Refusal raised when a roster model is not the one the DeepSeek key is granted
 for.

 A THROW RATHER THAN A DATA OUTCOME, because it is a routing mistake in our
 own code and not a thing a model did, and it is THE OWNER'S LOCK: the call
 never reaches the wire.

 @example
 ```ts
 throw new DeepSeekModelNotServedError({ modelId, },);
 ```
 */
export class DeepSeekModelNotServedError extends Error {
  /**
   Declares this message safe to forward: it names a model.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds failure naming the model this key is not granted for.

   @param modelId - roster model that was addressed here

   @example
   ```ts
   new DeepSeekModelNotServedError({ modelId: 'hf:Qwen/Qwen3.8-27B', },);
   ```
   */
  public constructor({ modelId, }: { readonly modelId: string; },) {
    super(
      `DeepSeek's key is granted for DeepSeek V4.1 Flash only, so DeepSeek does not serve ${modelId}; `
        + 'route it to another provider or pick another model',
    );
    this.name = 'DeepSeekModelNotServedError';
  }
}

/**
 Client surface for DeepSeek's own API.

 @example
 ```ts
 const client: DeepSeekClient = createDeepSeekClient({ apiKey, transport, },);
 ```
 */
export type DeepSeekClient = ModelCaller & {
  /**
   The account's balance, which is this provider's whole budget signal.
   */
  readonly credits: (args: { readonly signal: AbortSignal; },) => Promise<DeepSeekBalance>;
};

/**
 Spells one roster model the way DeepSeek names it, which is THE LOCK.

 @param modelId - roster model the caller addressed

 @returns Wire identifier for the request body

 @throws {@link DeepSeekModelNotServedError} when the key is not granted for the model

 @example
 ```ts
 const served = servedIdFor({ modelId, },);
 ```
 */
function servedIdFor({ modelId, }: { readonly modelId: RosterModelId; },): string {
  /**
   Spelling this provider uses, or that it serves no such model.
   */
  const spelling = deepSeekIdFor({ modelId, },);
  if (!spelling.served)
    throw new DeepSeekModelNotServedError({ modelId, },);
  return spelling.id;
}

/**
 The `response_format` member DeepSeek accepts for any schema.

 JSON MODE, NOT A SCHEMA: the json_schema type is refused by this API (HTTP 400),
 and the schema is already stated in the system prompt by the shared client. The
 word "json" the documentation requires is in that text.

 @returns The JSON-mode member

 @example
 ```ts
 const format = jsonObjectFormat();
 ```
 */
function jsonObjectFormat(): Readonly<Record<string, unknown>> {
  return { type: 'json_object', };
}

/**
 Prices one finished DeepSeek call at peak rates.

 @param extracted - completion whose usage block DeepSeek filled in, or did not

 @param bodyText - whole drained stream, whose last usage block names the cache split

 @returns The cost and cache fields the spend line carries, each absent where the stream reported nothing

 @example
 ```ts
 const fields = deepSeekSpendFields({ extracted, bodyText, },);
 ```
 */
function deepSeekSpendFields(
  {
    extracted,
    bodyText,
  }: {
    readonly extracted: ExtractedCompletion;
    readonly bodyText: string;
  },
): ReturnType<DirectRoute['spendFieldsOf']> {
  /**
   Cache split the usage block reported.
   */
  const split = deepSeekCacheSplitOf({ bodyText, },);
  /**
   The call priced at peak rates, or that nothing was reported.
   */
  const cost = deepSeekCostOf({
    card: DEEPSEEK_MODELS['deepseek-flash'],
    extracted,
    split,
  },);
  return {
    ...((cost === DIRECT_COST_UNREPORTED) ? {} : { costUsd: cost, }),
    ...((split.hitTokens === undefined) ? {} : { cachedTokens: split.hitTokens, }),
  };
}

/**
 Builds the route of DeepSeek's API.

 @param baseUrl - API base address, overridable for tests

 @returns The route the shared client calls

 @example
 ```ts
 const route = deepSeekRoute({ baseUrl: DEEPSEEK_BASE_URL, },);
 ```
 */
function deepSeekRoute({ baseUrl, }: { readonly baseUrl: string; },): DirectRoute {
  return {
    provider: 'deepseek',
    chatUrl: deepSeekChatUrlFor({ baseUrl, },),
    servedIdFor,
    responseFormatOnWire: jsonObjectFormat,
    spendFieldsOf: deepSeekSpendFields,
  };
}

/**
 Builds one client over injected transport, speaking chat completions to
 DeepSeek.

 @param apiKey - bearer token; never logged

 @param transport - HTTP the client calls over: `fetchTransport` in a run, a
 recorded one in a test. REQUIRED, since a test that left it out would reach
 the provider (ledger M43, X24)

 @param baseUrl - API base address, overridable for tests

 @param perModelConcurrency - bound on concurrent calls to the model; the
 account's stated 2,500 by default

 @param retryPolicy - transient-retry pacing; tests pass tiny backoffs

 @returns Client surface with chatText, chatJson and credits

 @example
 ```ts
 const client = createDeepSeekClient({ apiKey: process.env['DEEPSEEK_API_KEY'] ?? '', transport: fetchTransport, },);
 ```
 */
export function createDeepSeekClient(
  {
    apiKey,
    transport,
    baseUrl = DEEPSEEK_BASE_URL,
    perModelConcurrency = DEEPSEEK_PER_MODEL_CONCURRENCY,
    retryPolicy = DEFAULT_RETRY_POLICY,
  }: {
    readonly apiKey: string;
    readonly transport: ModelTransport;
    readonly baseUrl?: string;
    readonly perModelConcurrency?: number;
    readonly retryPolicy?: RetryPolicy;
  },
): DeepSeekClient {
  /**
   The chat surface, shared with Xiaomi's client.
   */
  const chat = createDirectClient({
    route: deepSeekRoute({ baseUrl, },),
    apiKey,
    transport,
    perModelConcurrency,
    retryPolicy,
  },);

  /**
   Reads the account's balance, which is this provider's whole budget signal.

   @param signal - abort signal honored for the read

   @returns Typed balance

   @throws {@link ProviderHttpError} on non-success status

   @throws {@link import('./deepseek-balance.ts').DeepSeekBalanceShapeError} on contract-violating bodies

   @example
   ```ts
   const { totalUsd, } = await client.credits({ signal, },);
   ```
   */
  async function credits(
    { signal, }: { readonly signal: AbortSignal; },
  ): Promise<DeepSeekBalance> {
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
      provider: 'deepseek',
      transport: withoutRefusedKeyBody({ transport, },),
      exchange: {
        url: deepSeekBalanceUrlFor({ baseUrl, },),
        label: 'balance',
        method: 'GET',
        headers: {
          [DIRECT_AUTH_HEADER]: `Bearer ${apiKey}`,
          'content-type': 'application/json',
        },
        signal,
      },
      policy: retryPolicy,
    },);

    if (!isSuccessStatus({ status: reply.status, },))
      throw new ProviderHttpError({
        provider: 'deepseek',
        status: reply.status,
        bodyText: reply.bodyText,
        summary: `DeepSeek /user/balance returned HTTP ${String(reply.status,)}:`,
      },);

    /**
     Typed balance parsed from the verified body shape.
     */
    const parsed = parseDeepSeekBalance({ bodyText: reply.bodyText, },);

    rl.debug(`balance ${parsed.totalUsd
      .toFixed(2,)} USD, available ${String(parsed.isAvailable,)}`,);
    return parsed;
  }

  return {
    ...chat,
    credits,
  };
}

//endregion DeepSeek client
