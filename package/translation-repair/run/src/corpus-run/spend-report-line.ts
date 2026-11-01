import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { PricedSeat, } from './spend-cost.ts';
import type { SeatSpend, } from './spend-read.ts';

//region Spend report line
// The lines of the spend report that stand for one seat, and the figures
// they are written from: what a seat cost in credits, in USD, or in tokens
// alone, and what share of the bill it was.
//
// SPLIT OUT OF `spend-report.ts` so the entry holds only the wiring and each
// line is read through its own cases.

/**
 Multiplier turning a fraction into a percentage.
 */
const PERCENT = 100;

/**
 Says how many of a seat's calls were reckoned rather than reported (ledger
 P14), nothing where none were.

 @param seat - seat with its counts

 @returns Suffix for the seat's line

 @example
 ```ts
 console.log(`${line}${reckonedNote({ seat, },)}`,);
 ```
 */
function reckonedNote({ seat, }: { readonly seat: SeatSpend; },): string {
  return (seat.reckonedCalls === 0)
    ? ''
    : `, ${String(seat.reckonedCalls,)} of them reckoned rather than reported`;
}

/**
 Renders a credit figure at the precision the provider quotes balances in.

 @param credits - what something came to

 @returns Text for a report column

 @example
 ```ts
 console.log(asCredits({ credits: 9.5, },),);
 ```
 */
export function asCredits({ credits, }: { readonly credits: number; },): string {
  return credits.toFixed(2,);
}

/**
 Renders one priced seat, and what share of the bill it was.

 @param seat - seat with its credits

 @param totalCredits - what every priced seat came to together

 @returns Line for the report

 @example
 ```ts
 console.log(pricedLine({ seat, totalCredits, },),);
 ```
 */
export function pricedLine(
  {
    seat,
    totalCredits,
  }: {
    readonly seat: PricedSeat;
    readonly totalCredits: number;
  },
): string {
  /**
   Share of the bill this seat was, blank where nothing was billed at all.
   */
  const share = (totalCredits === 0)
    ? 'n/a'
    : `${((seat.totalCredits / totalCredits) * PERCENT).toFixed(1,)}%`;

  return `  ${seat.model}: ${asCredits({ credits: seat.totalCredits, },)} credits (${share}) `
    + `over ${String(seat.calls,)} ${
      wordForCount({
        count: seat.calls,
        one: 'call',
        many: 'calls',
      },)
    }, `
    + `in ${String(seat.promptTokens,)}=${asCredits({ credits: seat.inputCredits, },)} `
    + `out ${String(seat.completionTokens,)}=${asCredits({ credits: seat.outputCredits, },)}${
     reckonedNote({ seat, },)}`;
}

/**
 Decimal places USD figures are rendered at: a corpus call costs tenths of
 a cent, and two places would print most of a run's seats as 0.00.
 */
const USD_PLACES = 4;

/**
 Renders a USD figure at the precision a corpus call costs in.

 @param usd - what something came to

 @returns Text for a report column

 @example
 ```ts
 console.log(asUsd({ usd: 0.0842, },),);
 ```
 */
export function asUsd({ usd, }: { readonly usd: number; },): string {
  return usd.toFixed(USD_PLACES,);
}

/**
 Renders one OpenRouter seat with the USD its lines reported, and what share
 of the run's USD it was.

 @param seat - seat with the USD summed off its `cost=` fields

 @param totalUsd - what every OpenRouter seat came to together

 @returns Line for the report

 @example
 ```ts
 console.log(usdLine({ seat, totalUsd, },),);
 ```
 */
export function usdLine(
  {
    seat,
    totalUsd,
  }: {
    readonly seat: SeatSpend;
    readonly totalUsd: number;
  },
): string {
  /**
   Share of the USD bill this seat was, blank where nothing was billed.
   */
  const share = (totalUsd === 0)
    ? 'n/a'
    : `${((seat.costUsd / totalUsd) * PERCENT).toFixed(1,)}%`;

  /**
   Calls whose line carried no cost, named so the seat's figure reads as a
   floor when any did.
   */
  const uncosted = seat.calls - seat.costedCalls;

  /**
   Floor note, absent when every call was costed.
   */
  const floor = (uncosted === 0)
    ? ''
    : `, a floor: ${String(uncosted,)} ${
      wordForCount({
        count: uncosted,
        one: 'call',
        many: 'calls',
      },)
    } carried no cost`;

  return `  ${seat.model}: ${asUsd({ usd: seat.costUsd, },)} USD (${share}) `
    + `over ${String(seat.calls,)} ${
      wordForCount({
        count: seat.calls,
        one: 'call',
        many: 'calls',
      },)
    }, `
    + `in ${String(seat.promptTokens,)} out ${String(seat.completionTokens,)}${floor}${
     reckonedNote({ seat, },)}`;
}

/**
 Renders a seat carrying tokens but no credit figure.

 @param seat - subscription or unpriced seat

 @returns Line for the report

 @example
 ```ts
 console.log(tokensOnlyLine({ seat, },),);
 ```
 */
export function tokensOnlyLine({ seat, }: { readonly seat: SeatSpend; },): string {
  return `  ${seat.model}: ${String(seat.calls,)} ${
    wordForCount({
      count: seat.calls,
      one: 'call',
      many: 'calls',
    },)
  }, `
    + `in ${String(seat.promptTokens,)} out ${String(seat.completionTokens,)}${
     reckonedNote({ seat, },)}`;
}

//endregion Spend report line
