import type { AnthropicPrices, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { ClaudeUsageSplit, } from './claude-stream-facts.ts';

//region Claude cost
// WHAT ONE CLAUDE CALL COST, in USD, from the four counts its usage block reported and the card's prices. THE WIRE
// REPORTS NO COST, only tokens, so this figure, priced here, is the `cost=` the SPEND line carries and the one the
// run's own meter adds up. The owner ruled the key needs no monthly cap of its own ("a month is defined very
// vaguely"), so this figure is read, never enforced.
//
// FOUR RATES, because Anthropic bills the four counts apart: the prompt past the last cache breakpoint, the prompt
// written to the five-minute cache, the prompt read back from it, and the completion with its thinking.
//
// HAIKU'S TIER IS PICKED BY THE WHOLE PROMPT, the three prompt counts together, and prices the whole call: at or under
// 100,000 prompt tokens the base rates, over them the longer tier's rates for every count, completion included
// (the pricing page as the brief carries it, read 2026-10-22).

/**
 Tokens one quoted price covers, since the page quotes per million.
 */
const PRICE_UNIT_TOKENS = 1_000_000;

/**
 USD one completed Claude exchange cost.

 @param prices - the card's prices, whose tier the prompt size picks

 @param usage - the four counts the stream reported

 @returns Cost in USD

 @example
 ```ts
 const cost = claudeCostOf({ prices, usage: { input: 1_000, cacheWrite: 0, cacheRead: 0, output: 500, }, },);
 ```
 */
export function claudeCostOf(
  {
    prices,
    usage,
  }: {
    readonly prices: AnthropicPrices;
    readonly usage: ClaudeUsageSplit;
  },
): number {
  /**
   Prompt tokens the model read, which pick the tier.
   */
  const prompt = usage.input + usage.cacheWrite
    + usage.cacheRead;
  /**
   Rates the call is priced at.
   */
  const tier = ((prices.longPrompt !== undefined) && (prompt
    > prices.longPrompt
      .aboveTokens))
    ? prices.longPrompt
    : prices;
  return (
    (usage.input * tier.inputUsdPerMillion)
      + (usage.cacheWrite * tier.cacheWriteUsdPerMillion)
      + (usage.cacheRead * tier.cacheHitUsdPerMillion)
      + (usage.output * tier.outputUsdPerMillion)
  ) / PRICE_UNIT_TOKENS;
}

//endregion Claude cost
