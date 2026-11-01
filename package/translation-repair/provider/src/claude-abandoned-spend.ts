import type { AnthropicPrices, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { reportSpend, } from './spend-line.ts';

//region Claude abandoned spend
// AN ATTEMPT ANTHROPIC ACCEPTED IS SPEND WHETHER OR NOT ITS USAGE ARRIVED, as it is on the Bedrock route (ledger P1,
// `bedrock-bound-ledger.ts`). A Claude stream that ends without its terminator is retried inside one call
// (`transient-retry.ts`), and each attempt that delivered something was billed for what it generated, thinking
// included. The client used to tell nobody, so a call's SPEND line read its last attempt alone and the run meter, which
// every Claude budget reads, was a floor that a cut stream could hold down without limit.
//
// A BOUND, NOT AN ESTIMATE. No characters-per-token ratio is measured for the Claude route (OpenRouter's reckoning
// divides by one), and adaptive thinking is billed whether or not its text is streamed, so delivered characters could
// under-read. The attempt is written at the most it could have been billed: no more prompt tokens than its body had
// bytes (a token is at least a byte), priced at the higher of the input and cache-write rates because the body asks for
// a cache write, and no more completion tokens than the `max_tokens` it sent. The over-read is bounded by one ceiling
// per abandoned attempt, and a 200 stream cut before its end is rare beside the calls that finish.

/**
 Tokens one quoted price covers, since the page quotes per million.
 */
const PRICE_UNIT_TOKENS = 1_000_000;

/**
 The most one attempt could have been billed.

 @example
 ```ts
 const bound: ClaudeAttemptBound = { promptTokens: 10_000, completionTokens: 32_000, usd: 0.345, };
 ```
 */
export type ClaudeAttemptBound = {
  /**
   Prompt tokens at the bound: the body's bytes.
   */
  readonly promptTokens: number;

  /**
   Completion tokens at the bound: the ceiling the call sent.
   */
  readonly completionTokens: number;

  /**
   Both halves at the card's prices.
   */
  readonly usd: number;
};

/**
 The most one attempt could have been billed, off what it sent.

 @param prices - the card's prices, whose tier the prompt size picks

 @param requestBodyBytes - size of the body the attempt sent

 @param maxTokens - `max_tokens` the body carried

 @returns Token counts and their price at the bound

 @example
 ```ts
 const bound = claudeAttemptBound({ prices, requestBodyBytes: 10_000, maxTokens: 32_000, },);
 ```
 */
export function claudeAttemptBound(
  {
    prices,
    requestBodyBytes,
    maxTokens,
  }: {
    readonly prices: AnthropicPrices;
    readonly requestBodyBytes: number;
    readonly maxTokens: number;
  },
): ClaudeAttemptBound {
  /**
   The longer tier, absent where the page prices every prompt size alike.
   */
  const { longPrompt, } = prices;
  /**
   Rates of the tier that prices the whole attempt.
   */
  const tier = ((longPrompt !== undefined) && (requestBodyBytes > longPrompt.aboveTokens))
    ? longPrompt
    : prices;
  return {
    promptTokens: requestBodyBytes,
    completionTokens: maxTokens,
    usd: ((requestBodyBytes * Math.max(
      tier.inputUsdPerMillion,
      tier.cacheWriteUsdPerMillion,
    )) + (maxTokens * tier.outputUsdPerMillion)) / PRICE_UNIT_TOKENS,
  };
}

/**
 Writes one attempt at the most it could have been billed, on the SPEND line and in the run meter.

 @param label - the card the call is named by, as the call's own SPEND line names it

 @param prices - the card's prices

 @param requestBodyBytes - size of the body the attempt sent

 @param maxTokens - `max_tokens` the body carried

 @returns Line as logged, in the SPEND grammar with the abandoned-bound mark

 @example
 ```ts
 const line = reportClaudeAbandonedSpend({ label, prices, requestBodyBytes: 10_000, maxTokens: 32_000, },);
 ```
 */
export function reportClaudeAbandonedSpend(
  {
    label,
    prices,
    requestBodyBytes,
    maxTokens,
  }: {
    readonly label: string;
    readonly prices: AnthropicPrices;
    readonly requestBodyBytes: number;
    readonly maxTokens: number;
  },
): string {
  /**
   The most the attempt could have been billed.
   */
  const bound = claudeAttemptBound({
    prices,
    requestBodyBytes,
    maxTokens,
  },);
  return reportSpend({
    provider: 'anthropic',
    label,
    extracted: {
      text: '',
      usage: {
        prompt_tokens: bound.promptTokens,
        completion_tokens: bound.completionTokens,
      },
    },
    costUsd: bound.usd,
    estimated: 'abandoned-bound',
  },);
}

//endregion Claude abandoned spend
