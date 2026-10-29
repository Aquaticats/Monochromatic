import type { AnthropicPrices, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type ClaudeAttemptBound,
  claudeAttemptBound,
} from './claude-abandoned-spend.ts';
import type { exchangeWithRetry, } from './transient-retry.ts';

//region Claude attempt admit
// WHAT A CALLER IS TOLD BEFORE THE CLAUDE CLIENT SENDS AN ATTEMPT. The client retries inside one call, up to five
// attempts, and each attempt that delivers a stream and is cut is billed for what it generated, so a caller that holds a
// run to an amount cannot look once per call: the attempts after the one it admitted are paid for without it. The retry
// ladder tells the caller before each attempt (`transient-retry.ts`), and this module turns that telling into the figure
// the attempt's abandoned line would write (`claude-abandoned-spend.ts`), so the amount a caller admits and the amount
// the meter later holds are one number.

/**
 What the Claude client tells a caller before it sends an attempt: the card the call is for and the most the attempt
 can be billed, which is the figure `reportClaudeAbandonedSpend` writes if the attempt is cut.

 @example
 ```ts
 const ask: ClaudeAttemptAsk = { label: 'claude-sonnet-5-5-effort-low', bound: { promptTokens: 10_000, completionTokens: 32_000, usd: 0.345, }, };
 ```
 */
export type ClaudeAttemptAsk = {
  /**
   The card the call is named by, as its SPEND line names it.
   */
  readonly label: string;

  /**
   The most the attempt could be billed.
   */
  readonly bound: ClaudeAttemptBound;
};

/**
 The part of the retry ladder's options that tells a caller before each attempt, for a client given an admit.

 @param admitAttempt - caller's admit, absent where the caller asked to be told of nothing

 @param label - card the call is named by

 @param prices - card's prices

 @param requestBodyBytes - size of the body every attempt of the call sends

 @param maxTokens - `max_tokens` the body carries

 @returns Ladder's `beforeAttempt`, which throws what the admit throws, and nothing at all where there is no admit,
 so the ladder is given nothing different from before

 @example
 ```ts
 await exchangeWithRetry({ provider: 'anthropic', transport, exchange, ...admissionOf({ admitAttempt, label, prices, requestBodyBytes, maxTokens, }), },);
 ```
 */
export function admissionOf(
  {
    admitAttempt,
    label,
    prices,
    requestBodyBytes,
    maxTokens,
  }: {
    readonly admitAttempt?: (attempt: ClaudeAttemptAsk,) => void;
    readonly label: string;
    readonly prices: AnthropicPrices;
    readonly requestBodyBytes: number;
    readonly maxTokens: number;
  },
): Pick<Parameters<typeof exchangeWithRetry>[0], 'beforeAttempt'> {
  if (admitAttempt === undefined)
    return {};
  return {
    beforeAttempt: function admit(): Promise<void> {
      admitAttempt({
        label,
        bound: claudeAttemptBound({
          prices,
          requestBodyBytes,
          maxTokens,
        },),
      },);
      return Promise.resolve();
    },
  };
}

//endregion Claude attempt admit
