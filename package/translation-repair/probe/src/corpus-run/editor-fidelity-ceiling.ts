import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type { ChatTextRequest, } from '@monochromatic-dev/translation-repair-provider/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  ANTHROPIC_SERVED_IDS,
  type AnthropicPrices,
  CLAUDE_CARDS,
  CLAUDE_COMPLETION_CAP,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { STAGE_RETRY_ROUNDS, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Editor fidelity ceiling
// WHAT THE PROBE MAY SPEND, printed before any call. The amount the operator allows is held by the spend guard
// (`editor-fidelity-spend-guard.ts`), call by call, and this module prices the most one call can cost for it.
//
// THE CEILING IS BUILT FROM THE TWO THINGS A CALL CANNOT EXCEED: the prompt, sized at one input token to the
// character of the sheet (an overshoot for English, which runs near four characters a token, and close for Chinese),
// and the completion cap every Claude card sends as `max_tokens`, thinking included. A call that spends the whole cap
// is billed for it and asked again, so the ceiling prices the cap in full.
//
// TWO FIGURES, BECAUSE ONE ASK A TRIAL IS NOT THE WORST. The gather asks again when a reply cannot be read: in each of
// its retry rounds, and once more in a recovery round. The FIRST-PASS figure is one ask for each trial; the WORST CASE
// multiplies it by the most asks one gather gives a seat. Both are printed so a reader knows what an arm could cost,
// and neither gates the arm: a pool of forty trials prices past any amount an arm is allowed, and the guard stops the
// arm at the call that would pass the amount instead.
//
// A CARD WITH NO PRICE TABLE IS REFUSED, never priced at zero: only the Claude cards carry one.

/**
 Most asks one gather gives a seat: the first round, the retry rounds, and the recovery round for a reply nothing
 could read.
 */
export const MAX_ASKS_PER_TRIAL: number = STAGE_RETRY_ROUNDS + 2;

/**
 Tokens in a million, the unit the prices are written in.
 */
export const PER_MILLION = 1_000_000;

/**
 The ceiling for one card.

 @example
 ```ts
 const [ceiling,] = editorCeilings({ cardIds: ['claude-sonnet-5-5-effort-low',], sheetChars: [9000,], },);
 ```
 */
export type EditorCeiling = {
  /**
   Card the figures are for.
   */
  readonly cardId: RosterModelId;

  /**
   Trials the figures cover.
   */
  readonly trials: number;

  /**
   Most the trials can cost at one ask each, in US dollars.
   */
  readonly firstPassUsd: number;

  /**
   Most the trials can cost when every one is asked as often as a gather asks, in US dollars.
   */
  readonly worstCaseUsd: number;

  /**
   Prices the figures are reckoned at, tier of the first sheet.
   */
  readonly prices: AnthropicPrices;
};

/**
 Prices of a card, or the refusal that none exist.

 @param cardId - card named

 @returns Card's prices

 @throws StatedRefusalError when the card is not one the Claude route serves, so it has no table

 @example
 ```ts
 const prices = pricesOf({ cardId: 'claude-haiku-5-5', },);
 ```
 */
export function pricesOf({ cardId, }: { readonly cardId: RosterModelId; },): AnthropicPrices {
  /**
   The Claude spelling this id matches, absent for any other seat.
   */
  const served = ANTHROPIC_SERVED_IDS.find(function is(servedId,): boolean {
    return servedId === cardId;
  },);
  if (served === undefined) {
    throw new StatedRefusalError({
      says: `${cardId} has no price table in claude-card-sides.ts, so no ceiling can be printed and the editor `
        + 'fidelity probe will not start; it benches the Claude cards',
    },);
  }
  return nonNullishOrThrow(CLAUDE_CARDS[served]
    .anthropic,)
    .prices;
}

/**
 Rates that price a whole call: the long-prompt tier once the prompt passes its limit.

 @param prices - card's prices

 @param promptTokens - tokens in the prompt

 @returns Tier's rates

 @example
 ```ts
 const { outputUsdPerMillion, } = tierFor({ prices, promptTokens: 9000, },);
 ```
 */
export function tierFor(
  {
    prices,
    promptTokens,
  }: {
    readonly prices: AnthropicPrices;
    readonly promptTokens: number;
  },
): Pick<AnthropicPrices, 'inputUsdPerMillion' | 'outputUsdPerMillion' | 'cacheWriteUsdPerMillion'> {
  return ((prices.longPrompt !== undefined) && (promptTokens
    > prices.longPrompt
      .aboveTokens))
    ? prices.longPrompt
    : prices;
}

/**
 Most one call can cost at a card's prices.

 @param prices - card's prices

 @param promptTokens - tokens in the prompt, at the sizing rule

 @returns US dollars for that prompt and a whole completion cap

 @example
 ```ts
 const usd = callCeilingUsd({ prices, promptTokens: 9000, },);
 ```
 */
export function callCeilingUsd(
  {
    prices,
    promptTokens,
  }: {
    readonly prices: AnthropicPrices;
    readonly promptTokens: number;
  },
): number {
  /**
   Rates of the tier that prices the whole call.
   */
  const tier = tierFor({
    prices,
    promptTokens,
  },);
  return ((promptTokens * tier.inputUsdPerMillion) + (CLAUDE_COMPLETION_CAP * tier.outputUsdPerMillion)) / PER_MILLION;
}

/**
 Characters of the messages a call sends, which the ceiling sizes the prompt by.

 @param messages - request's messages

 @returns Characters of every message, a part list counted as its JSON

 @example
 ```ts
 const chars = messageChars({ messages: request.messages, },);
 ```
 */
export function messageChars(
  { messages, }: { readonly messages: ChatTextRequest['messages']; },
): number {
  return messages.reduce(
    function summed(
      total,
      message,
    ): number {
      return total + (((typeof message.content) === 'string') ? message.content : JSON.stringify(message.content,)).length;
    },
    0,
  );
}

/**
 Prices the most a run can spend, card by card.

 @param cardIds - cards the run seats

 @param sheetChars - characters of the sheet each trial sends, in trial order

 @returns One ceiling for each card

 @throws StatedRefusalError when a card has no price table

 @example
 ```ts
 const ceilings = editorCeilings({ cardIds, sheetChars: [9000, 9500,], },);
 ```
 */
export function editorCeilings(
  {
    cardIds,
    sheetChars,
  }: {
    readonly cardIds: readonly RosterModelId[];
    readonly sheetChars: readonly number[];
  },
): readonly EditorCeiling[] {
  return cardIds.map(function ceilingOf(cardId,): EditorCeiling {
    /**
     The card's prices, refused where it has none.
     */
    const prices = pricesOf({ cardId, },);
    /**
     One ask for each trial at the most it can cost.
     */
    const firstPassUsd = sheetChars.reduce(
      function summed(
        total,
        chars,
      ): number {
        return total + callCeilingUsd({
          prices,
          promptTokens: chars,
        },);
      },
      0,
    );
    return {
      cardId,
      trials: sheetChars.length,
      firstPassUsd,
      worstCaseUsd: firstPassUsd * MAX_ASKS_PER_TRIAL,
      prices,
    };
  },);
}

/**
 Decimals a dollar figure is printed with, enough to tell a cent-fraction ask apart.
 */
const USD_DECIMALS = 4;

/**
 Dollars as the lines print them.

 @param usd - amount

 @returns Four decimals

 @example
 ```ts
 // yields '0.6480'
 const text = usdText({ usd: 0.648, },);
 ```
 */
export function usdText({ usd, }: { readonly usd: number; },): string {
  return usd.toFixed(USD_DECIMALS,);
}

/**
 The line that states one card's ceiling and what it is built from.

 @param ceiling - card's ceiling

 @returns Line, without a newline

 @example
 ```ts
 log.info(ceilingLine({ ceiling, },),);
 ```
 */
export function ceilingLine({ ceiling, }: { readonly ceiling: EditorCeiling; },): string {
  return `ceiling ${ceiling.cardId}: ${String(ceiling.trials,)} ${
    wordForCount({
      count: ceiling.trials,
      one: 'unit',
      many: 'units',
    },)
  }, one ask each, `
    + `at most ${usdText({ usd: ceiling.firstPassUsd, },)} USD; with ${String(MAX_ASKS_PER_TRIAL,)} ${
      wordForCount({
        count: MAX_ASKS_PER_TRIAL,
        one: 'ask',
        many: 'asks',
      },)
    } a trial at most `
    + `${usdText({ usd: ceiling.worstCaseUsd, },)} USD; priced at ${String(ceiling.prices
      .inputUsdPerMillion,)} USD in and `
    + `${String(ceiling.prices
      .outputUsdPerMillion,)} USD out per million tokens, ${String(CLAUDE_COMPLETION_CAP,)} output `
    + 'tokens an ask, one input token to the character of the sheet';
}

//endregion Editor fidelity ceiling
