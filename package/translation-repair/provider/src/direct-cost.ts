import type {
  DeepSeekCard,
  XiaomiCard,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { ExtractedCompletion, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import type { DeepSeekCacheSplit, } from './direct-usage.ts';

//region Direct cost
// WHAT ONE DEEPSEEK OR XIAOMI CALL COST, in USD, from the usage it reported
// and the card's prices. NEITHER WIRE REPORTS A COST (the parent session's
// probes of 2026-10-22 found a token usage block and no cost field), so this
// figure, priced here, is the `cost=` the SPEND line carries and the one the
// run's own meter adds up.
//
// DEEPSEEK IS PRICED AT THE RATE OF THE HOUR THE REQUEST STARTED IN: at the
// card's peak rates when it started inside a weekday peak window (UTC), at half
// of them otherwise. The page (`https://api-docs.deepseek.com/quick_start/pricing`,
// read 2026-10-25 by the lead; this change fetched nothing) says "Peak hours are
// 01:00 - 04:00 and 06:00 - 10:00 UTC, Monday through Friday, excluding Chinese
// public holidays", and halves every rate off peak, which is every other hour.
// Until 2026-10-25 every call was booked at peak, and a run that ran a Saturday
// night booked 0.147093 USD while the account's balance fell 0.07.
//
// WHY THE FIGURE IS STILL AN UPPER BOUND ON A HOLIDAY. A Chinese public holiday
// only turns a weekday window off peak, so pricing every weekday window at
// peak, with no calendar, can only over-read it; outside the weekday windows
// the page itself says off peak. The weekday is the UTC weekday: both windows
// lie between 09:00 and 18:00 in Beijing on the same calendar date, so the UTC
// weekday and the Beijing weekday agree for every instant a window covers.
//
// THE EDGES. The page writes each window as "01:00 - 04:00" and does not say
// whether its last instant belongs to it. Each window is read as half-open, a
// ruling of the lead of 2026-10-25: 01:00:00 is peak, 04:00:00 is off peak,
// and likewise 06:00:00 and 10:00:00.
//
// WHAT IS NOT BOUNDED. The instant is the one the request started at, read once
// before the first attempt; a call that spans an edge, or a retry that starts
// after one, is booked at the rate of its first start. A Saturday that China
// makes a working day is off peak here, since the page says "Monday through
// Friday". The first is bounded by one call's length, the second by the page's
// own wording.
//
// A COUNT THE WIRE DID NOT SPLIT IS PRICED AT THE DEARER RATE, for the same
// reason: a DeepSeek block with no cache fields prices the whole prompt as
// misses, and a Xiaomi block with no cached count likewise.

/**
 Reading given when the stream carried no usage block, so nothing can be
 priced; the SPEND line still prints with its counts unreported.
 */
export const DIRECT_COST_UNREPORTED = 'unreported';

/**
 Tokens one quoted price covers, since both pages quote per million.
 */
const PRICE_UNIT_TOKENS = 1_000_000;

/**
 Share of a peak rate DeepSeek charges outside a peak window.

 The page (`https://api-docs.deepseek.com/quick_start/pricing`, read 2026-10-25
 by the lead) states the off-peak rates as half of the peak rates, and the card
 (`DEEPSEEK_V4_1_FLASH_ON_DEEPSEEK`) holds the peak rates only.

 @example
 ```ts
 const offPeakHitRate = card.promptCacheHitUsdPerMillion * DEEPSEEK_OFF_PEAK_MULTIPLIER;
 ```
 */
export const DEEPSEEK_OFF_PEAK_MULTIPLIER = 0.5;

/**
 One peak window as whole UTC hours, the end excluded.

 @example
 ```ts
 const morning: PeakWindowUtcHours = { startHour: 1, endHour: 4, };
 ```
 */
type PeakWindowUtcHours = {
  /**
   First hour of the window, whose first instant is peak.
   */
  readonly startHour: number;

  /**
   Hour at whose first instant the window has ended, so that instant is off peak.
   */
  readonly endHour: number;
};

/**
 The two peak windows of a weekday, "01:00 - 04:00 and 06:00 - 10:00 UTC" on the
 page, each half-open.
 */
const PEAK_WINDOWS_UTC: readonly PeakWindowUtcHours[] = [
  { startHour: 1, endHour: 4, },
  { startHour: 6, endHour: 10, },
];

/**
 Whether an instant lies in one of DeepSeek's weekday peak windows, which is
 the upper bound on whether the call was charged peak: a Chinese public holiday
 inside a window makes it cheaper, and this reads no calendar.

 AN INSTANT THAT IS NOT A TIME (an invalid date) IS PEAK, so a clock that
 fails prices a call at the dearer rate and never under-reads the balance.

 @param at - instant the request started at

 @returns Whether it lies on a Monday to Friday (UTC) inside 01:00 to 04:00 or 06:00 to 10:00 UTC, the end of each
 window excluded

 @example
 ```ts
 isDeepSeekPeakAt({ at: new Date('2026-10-21T01:00:00Z',), },); // true, a Wednesday
 ```
 */
export function isDeepSeekPeakAt({ at, }: { readonly at: Date; },): boolean {
  /**
   Hour of the day in UTC, not a number where the date is invalid.
   */
  const hour = at.getUTCHours();
  if (Number.isNaN(hour,))
    return true;

  /**
   Day of the week in UTC, 0 for Sunday and 6 for Saturday.
   */
  const weekday = at.getUTCDay();
  if ((weekday === 0) || (weekday === 6))
    return false;

  return PEAK_WINDOWS_UTC.some(function holdsHour({ startHour, endHour, },): boolean {
    return (hour >= startHour) && (hour < endHour);
  },);
}

/**
 USD one completed DeepSeek exchange cost, at the peak rates when its request
 started inside a weekday peak window and at half of them otherwise.

 @param card - DeepSeek's side of the card, whose peak rates price the call

 @param extracted - completion whose `usage` block DeepSeek filled in, or did not

 @param split - cache split the stream's usage block reported

 @param startedAt - instant the request started at, from the client's clock, whose hour picks the rate

 @returns Cost in USD, or that no usage arrived to price

 @example
 ```ts
 const cost = deepSeekCostOf({ card, extracted, split: deepSeekCacheSplitOf({ bodyText, },), startedAt: new Date(), },);
 ```
 */
export function deepSeekCostOf(
  {
    card,
    extracted,
    split,
    startedAt,
  }: {
    readonly card: DeepSeekCard;
    readonly extracted: ExtractedCompletion;
    readonly split: DeepSeekCacheSplit;
    readonly startedAt: Date;
  },
): number | typeof DIRECT_COST_UNREPORTED {
  /**
   Usage block as the provider reported it, absent where it did not.
   */
  const { usage, } = extracted;
  if (usage === undefined)
    return DIRECT_COST_UNREPORTED;

  /**
   Prompt tokens served from the cache, none where the block named none, and
   never more than the prompt held: a count past it is the wire's inconsistency,
   and the prompt count is the one both prices are quoted against.
   */
  const hitTokens = Math.min(
    split.hitTokens ?? 0,
    usage.prompt_tokens,
  );

  /**
   Prompt tokens not served from the cache: as named, else what the cache
   did not account for, so the whole prompt is priced as misses when the
   block split nothing; and never more than the prompt leaves after the hits.
   */
  const missTokens = Math.min(
    split.missTokens ?? (usage.prompt_tokens - hitTokens),
    usage.prompt_tokens - hitTokens,
  );

  /**
   Share of the card's peak rates this call is charged: all of them inside a
   weekday peak window, half of them outside.
   */
  const rateShare = isDeepSeekPeakAt({ at: startedAt, },) ? 1 : DEEPSEEK_OFF_PEAK_MULTIPLIER;

  return (
    (
      (hitTokens * card.promptCacheHitUsdPerMillion)
      + (missTokens * card.promptCacheMissUsdPerMillion)
      + (usage.completion_tokens * card.completionUsdPerMillion)
    ) * rateShare
  ) / PRICE_UNIT_TOKENS;
}

/**
 USD one completed Xiaomi exchange cost.

 @param card - Xiaomi's side of the card, whose rates price the call

 @param extracted - completion whose `usage` block Xiaomi filled in, or did not

 @param cachedTokens - prompt tokens the stream said were served from cache,
 none where it said nothing

 @returns Cost in USD, or that no usage arrived to price

 @example
 ```ts
 const cost = xiaomiCostOf({ card, extracted, cachedTokens: 0, },);
 ```
 */
export function xiaomiCostOf(
  {
    card,
    extracted,
    cachedTokens,
  }: {
    readonly card: XiaomiCard;
    readonly extracted: ExtractedCompletion;
    readonly cachedTokens: number;
  },
): number | typeof DIRECT_COST_UNREPORTED {
  /**
   Usage block as the provider reported it, absent where it did not.
   */
  const { usage, } = extracted;
  if (usage === undefined)
    return DIRECT_COST_UNREPORTED;

  /**
   Prompt tokens served from the cache, never more than the prompt held.
   */
  const hitTokens = Math.min(
    cachedTokens,
    usage.prompt_tokens,
  );

  /**
   Prompt tokens not served from the cache.
   */
  const missTokens = usage.prompt_tokens - hitTokens;

  return (
    (hitTokens * card.promptCacheHitUsdPerMillion)
      + (missTokens * card.promptCacheMissUsdPerMillion)
      + (usage.completion_tokens * card.completionUsdPerMillion)
  ) / PRICE_UNIT_TOKENS;
}

//endregion Direct cost
