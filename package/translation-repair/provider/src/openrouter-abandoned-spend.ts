import {
  type OpenRouterServedId,
  servedRecord,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { OPENROUTER_MODELS, } from './openrouter-catalog.ts';
import { reportSpend, } from './spend-line.ts';

//region OpenRouter abandoned spend
// A STREAM THIS PIPELINE STOPPED READING WAS STILL PAID FOR, and until
// 2026-09-09 nothing wrote that down. `SPEND` lines are written off the usage
// block of a completed stream, so a round's straggler cut, a stall, an overrun
// or a degenerate ending left no line at all, and the log summed 141.62 USD
// where the meter had moved 199.92 between the top-up of 2026-09-08 and the
// refusal of 2026-09-09. The missing 58 were 2,135 abandoned streams.
//
// ESTIMATED, AND SAID SO. The wire reports cost only at the end, so an
// abandoned call's cost is reconstructed: completion tokens from the raw
// stream characters delivered at each model's measured ratio, prompt tokens
// from the request body's bytes, both priced at the listing's rates. The line
// carries `estimated=abandoned` so a reader can total it beside the reported
// lines or apart from them. The estimate over-reads a stream on an endpoint
// that honoured the cancel and under-reads one that generated on past the cut,
// which is why the ceiling in `completion-cap.ts` is the bound and
// this is the record.

/**
 Median of recorded model medians for an unmeasured version, not a pooled-stream percentile.
 The cards measured on 2026-09-28 over pass-run logs (ledger P7) carry 0.9, 93, 130, 140, 228, 286, 292
 and 302; the middle of the eight is 184, halfway between 140 and 228.
 This remains an explicitly abandoned-call estimate, never reported usage.
 */
const UNMEASURED_RAW_CHARS_PER_TOKEN = 184;

/**
 Raw stream characters per completion token a card carries,
 or the median of the measured ones where the card carries none.

 @param rawCharsPerToken - Ratio the card carries, `'unmeasured'` where the
 roster holds no observation for the model

 @returns Raw characters one completion token stands for, which the reckoning
 divides the delivered characters by

 @example
 ```ts
 const ratio = rawCharsPerCompletionTokenOf('unmeasured',);
 ```
 */
export function rawCharsPerCompletionTokenOf(rawCharsPerToken: number | 'unmeasured',): number {
  return (rawCharsPerToken === 'unmeasured') ? UNMEASURED_RAW_CHARS_PER_TOKEN : rawCharsPerToken;
}

/**
 Raw stream characters per completion token, the 50th percentile over every
 completed OpenRouter stream of 2026-09-09 whose progress line sat beside
 its spend line (pass logs, re-measured in ledger P7), read off the cards.
 Framing differs by endpoint, which is why one model reads three times
 another.
 */
const RAW_CHARS_PER_COMPLETION_TOKEN: Readonly<Record<OpenRouterServedId, number>> = servedRecord({
  provider: 'openrouter',
  toRow: function ratioOf(card,): number {
    /**
     Ratio the card carries, or the median of the measured ones.
     */
    const { rawCharsPerToken, } = card.openrouter;
    return rawCharsPerCompletionTokenOf(rawCharsPerToken,);
  },
},);

/**
 Request body bytes per prompt token, an order-of-magnitude figure for a
 body that is mostly UTF-8 Chinese at three bytes a character and English
 sheet text at one; the prompt half of an abandoned call is the smaller half
 and this is labelled an estimate.
 */
export const PROMPT_BYTES_PER_TOKEN = 4;

/**
 Tokens in one million, the unit the listing prices in.
 */
const TOKENS_PER_MILLION = 1_000_000;

/**
 What an abandoned call is reckoned to have cost.

 @example
 ```ts
 const estimate: AbandonedSpendEstimate = { promptTokens: 1000, completionTokens: 10, usd: 0.0006, };
 ```
 */
export type AbandonedSpendEstimate = {
  /**
   Prompt tokens reckoned from the request body's bytes.
   */
  readonly promptTokens: number;

  /**
   Completion tokens reckoned from the raw characters delivered.
   */
  readonly completionTokens: number;

  /**
   Both halves at the listing's prices.
   */
  readonly usd: number;
};

/**
 Reckons an abandoned call's cost from what it delivered and what it sent.

 @param servedId - model as this provider spells it

 @param deliveredChars - raw stream characters read before the end

 @param requestBodyBytes - size of the request body that was sent

 @param maxTokens - `max_tokens` the call sent, past which the endpoint bills
 nothing, so neither does the reckoning (ledger P7)

 @returns Token halves and their price

 @example
 ```ts
 const estimate = estimateAbandonedSpend({ servedId, deliveredChars: 3860, requestBodyBytes: 4000, maxTokens: 1149, },);
 ```
 */
export function estimateAbandonedSpend(
  {
    servedId,
    deliveredChars,
    requestBodyBytes,
    maxTokens,
  }: {
    readonly servedId: OpenRouterServedId;
    readonly deliveredChars: number;
    readonly requestBodyBytes: number;
    readonly maxTokens: number;
  },
): AbandonedSpendEstimate {
  /**
   Listing row carrying this model's prices.
   */
  const info = OPENROUTER_MODELS[servedId];

  /**
   Completion tokens the delivered characters stand for, never past the
   ceiling the call sent.
   */
  const completionTokens = Math.min(
    Math.round(deliveredChars / RAW_CHARS_PER_COMPLETION_TOKEN[servedId],),
    maxTokens,
  );

  /**
   Prompt tokens the body's bytes stand for.
   */
  const promptTokens = Math.round(requestBodyBytes / PROMPT_BYTES_PER_TOKEN,);

  /**
   Both halves priced.
   */
  const usd = ((promptTokens * info.promptUsdPerMillion) + (completionTokens * info.completionUsdPerMillion))
    / TOKENS_PER_MILLION;
  return {
    promptTokens,
    completionTokens,
    usd,
  };
}

/**
 Writes the estimated spend line for one abandoned attempt.

 ONE LINE PER ATTEMPT (ledger P1, 2026-09-28): the retry ladder calls this
 for every attempt that delivered something before it failed, the ones it
 retried included. The reckoning used to wrap the whole ladder, so an attempt
 refused and retried inside it left no line.

 @param servedId - model as this provider spells it

 @param deliveredChars - raw stream characters the attempt delivered

 @param requestBodyBytes - size of the request body that was sent

 @param maxTokens - `max_tokens` the call sent

 @returns Line as logged, in the SPEND grammar with the estimated mark

 @example
 ```ts
 const line = reportAbandonedSpend({ servedId, deliveredChars: 3860, requestBodyBytes: 4000, maxTokens: 1149, },);
 ```
 */
export function reportAbandonedSpend(
  {
    servedId,
    deliveredChars,
    requestBodyBytes,
    maxTokens,
  }: {
    readonly servedId: OpenRouterServedId;
    readonly deliveredChars: number;
    readonly requestBodyBytes: number;
    readonly maxTokens: number;
  },
): string {
  /**
   What the attempt is reckoned to have cost.
   */
  const estimate = estimateAbandonedSpend({
    servedId,
    deliveredChars,
    requestBodyBytes,
    maxTokens,
  },);
  return reportSpend({
    provider: 'openrouter',
    label: servedId,
    extracted: {
      text: '',
      usage: {
        prompt_tokens: estimate.promptTokens,
        completion_tokens: estimate.completionTokens,
      },
    },
    costUsd: estimate.usd,
    estimated: 'abandoned',
  },);
}

//endregion OpenRouter abandoned spend
