import type { BedrockServedId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { BEDROCK_MODELS, } from './bedrock-catalog.ts';
import type { ExtractedCompletion, } from '@monochromatic-dev/translation-repair-provider-stream/ts';

//region Bedrock cost
// What one Bedrock call cost, in USD, from the usage it reported and the
// catalog's prices. THE WIRE REPORTS NO COST, unlike OpenRouter's final chunk,
// and the account exposes no balance to a bearer key, so this figure is the
// only one the ledger and the meter have.

/**
 Reading given when the stream carried no usage block, so nothing can be
 priced and nothing is written to the ledger; the SPEND line still prints
 with its counts unreported, as `spend-line.ts` argues it must.
 */
export const BEDROCK_COST_UNREPORTED = 'unreported';

/**
 Tokens one quoted price covers, since the pricing page quotes per million.
 */
const PRICE_UNIT_TOKENS = 1_000_000;

/**
 USD one completed exchange cost, off the usage the last chunk reported.

 @param servedId - model the call went to, whose row carries the prices

 @param extracted - completion whose `usage` block the provider filled in,
 or did not

 @returns Cost in USD, or that no usage arrived to price

 @example
 ```ts
 const cost = bedrockCostOf({ servedId: 'google.gemma-4-e2b', extracted, },);
 ```
 */
export function bedrockCostOf(
  {
    servedId,
    extracted,
  }: {
    readonly servedId: BedrockServedId;
    readonly extracted: ExtractedCompletion;
  },
): number | typeof BEDROCK_COST_UNREPORTED {
  /**
   Usage block as the provider reported it, absent where it did not.
   */
  const { usage, } = extracted;
  if (usage === undefined)
    return BEDROCK_COST_UNREPORTED;

  /**
   Prices for this model, per million tokens each way.
   */
  const {
    promptUsdPerMillion,
    completionUsdPerMillion,
  } = BEDROCK_MODELS[servedId];

  /**
   What the prompt cost.
   */
  const promptUsd = (usage.prompt_tokens * promptUsdPerMillion) / PRICE_UNIT_TOKENS;

  /**
   What the completion cost.
   */
  const completionUsd = (usage.completion_tokens * completionUsdPerMillion) / PRICE_UNIT_TOKENS;

  return promptUsd + completionUsd;
}

/**
 The most one attempt could have been billed, off what it sent.

 A BOUND, NOT AN ESTIMATE (ledger P1, 2026-09-28). An attempt the endpoint
 accepted and this pipeline stopped reading reports no usage, and whether
 Bedrock bills output past a cancel is unmeasured (OpenRouter's endpoints
 were measured billing to the end, `completion-cap.ts`). This ledger is the
 only guard on the owner's card, where an under-read is the failure, so the
 attempt is priced at no more prompt tokens than its body had bytes and no
 more completion tokens than its `max_tokens`. Abandoned attempts were 874 of
 about 190,000 Bedrock calls in the logs, so the over-read is small.

 @param servedId - model the attempt went to, whose row carries the prices

 @param requestBodyBytes - size of the body the attempt sent

 @param maxTokens - `max_tokens` the body carried

 @returns Token counts and their price at the bound

 @example
 ```ts
 const bound = bedrockAttemptBound({ servedId: 'google.gemma-4-e2b', requestBodyBytes: 30_000, maxTokens: 1_149, },);
 ```
 */
export function bedrockAttemptBound(
  {
    servedId,
    requestBodyBytes,
    maxTokens,
  }: {
    readonly servedId: BedrockServedId;
    readonly requestBodyBytes: number;
    readonly maxTokens: number;
  },
): {
  readonly promptTokens: number;
  readonly completionTokens: number;
  readonly usd: number
} {
  /**
   Prices for this model, per million tokens each way.
   */
  const {
    promptUsdPerMillion,
    completionUsdPerMillion,
  } = BEDROCK_MODELS[servedId];
  return {
    promptTokens: requestBodyBytes,
    completionTokens: maxTokens,
    usd: ((requestBodyBytes * promptUsdPerMillion) + (maxTokens * completionUsdPerMillion)) / PRICE_UNIT_TOKENS,
  };
}

//endregion Bedrock cost
