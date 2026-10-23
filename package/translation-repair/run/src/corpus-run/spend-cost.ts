import {
  creditsFor,
  HYPER_PRICE_READ_ON,
} from './hyper-price.ts';
import type {
  SeatSpend,
  SpendTally,
} from './spend-read.ts';

//region Spend cost
// WHAT A RUN COST, from what its log recorded and what the provider charges.
//
// FIVE BUCKETS, AND MIXING THEM WOULD BE THE WHOLE BUG. A seat is metered in
// hypercredits and priced, metered in hypercredits and missing from the price
// table, on a flat subscription that bills no credits at all, on NeuralWatt's
// free daily energy (metered in watt-hours by the energy ledger; the nominal
// cost its calls report is no bill), or billed in USD on OpenRouter,
// Bedrock, DeepSeek, Xiaomi or the Claude key with the cost read off the wire. Only the first has a credit figure and only
// the last a USD figure. A total that folded the others in
// would read as a cheaper run for the second and as an invented currency for
// the third and fourth.
//
// SORTED BY WHAT IT COST, not by name and not by call count, because the
// question this answers is which seat is worth reconsidering. On this roster
// the answer is not the one with the most calls: output is priced two to five
// times input and `completion_tokens` counts thinking, so one talkative seat
// outweighs several terse ones.
//
// A FLOOR, AND IT SAYS SO. Calls whose provider sent no usage block contribute
// nothing to either half, so every figure here is a floor over the calls that
// reported. `unreportedCalls` carries how wide that gap is, and a reader who
// ignores it will underread a run by however much the quiet calls cost.

/**
 One metered seat with what its tokens came to.

 @example
 ```ts
 const seat: PricedSeat = { ...spend, inputCredits: 3.4, outputCredits: 6.1, totalCredits: 9.5, };
 ```
 */
export type PricedSeat = SeatSpend & {
  /**
   Credits the prompt half came to at this model's input rate.
   */
  readonly inputCredits: number;

  /**
   Credits the answer half came to, thinking included.
   */
  readonly outputCredits: number;

  /**
   Both halves together, which is what the balance actually moved by.
   */
  readonly totalCredits: number;
};

/**
 Everything a tally cost, with the seats no price could be put on kept apart.

 @example
 ```ts
 const cost = priceTally({ tally, },);
 ```
 */
export type SpendCost = {
  /**
   Metered seats the price table knew, costliest first.
   */
  readonly priced: readonly PricedSeat[];

  /**
   Metered seats the price table had no row for.

   NAMED RATHER THAN COUNTED AT ZERO. A model the provider added after the
   table was read still bills, and a total that skipped it silently would
   report a cheaper run rather than an incomplete one.
   */
  readonly unpriced: readonly SeatSpend[];

  /**
   Seats on the flat subscription, carried with their tokens and no credits.
   */
  readonly subscription: readonly SeatSpend[];

  /**
   Seats on NeuralWatt's free daily energy, with their tokens and the USD
   their lines reported, a nominal figure NeuralWatt reports on free-energy
   requests too.

   A BUCKET OF THEIR OWN AND IN NO TOTAL. Hypercredits and USD are not what
   this energy is metered in, and a USD figure here is no bill: plan spend
   shows only in the plan-energy difference a run prints at its end.
   */
  readonly freeEnergy: readonly SeatSpend[];

  /**
   Seats billed in USD per token, on OpenRouter, Bedrock, DeepSeek or
   Xiaomi or the Claude key, costliest first, each carrying the USD its lines reported.

   A FOURTH BUCKET AND A SECOND CURRENCY. Hypercredits and USD are never
   summed: `totalCredits` stays a credit figure, and this bucket's
   total is `totalUsd`.
   */
  readonly openRouter: readonly SeatSpend[];

  /**
   Credits every priced seat came to.
   */
  readonly totalCredits: number;

  /**
   USD every OpenRouter seat's lines reported, a floor over the calls whose
   line carried a cost.
   */
  readonly totalUsd: number;

  /**
   Calls across every seat whose provider reported no usage at all, so a
   reader can see how much of the run the totals are a floor over.
   */
  readonly unreportedCalls: number;

  /**
   Date the rates came from, carried so a report can print how old they are.
   */
  readonly pricedAsOf: string;
};

/**
 Prices one metered seat, or reports that the table has no row for it.

 @param seat - one seat's totals as `tallySpend` summed them

 @returns Seat with its credits, or that this model is not in the table

 @example
 ```ts
 const priced = priceSeat({ seat, },);
 ```
 */
function priceSeat(
  { seat, }: { readonly seat: SeatSpend; },
): PricedSeat | 'unpriced' {
  /**
   What this seat's two halves came to, absent where the model has no row.
   */
  const credits = creditsFor({
    model: seat.model,
    promptTokens: seat.promptTokens,
    completionTokens: seat.completionTokens,
  },);

  if (credits === 'unpriced')
    return 'unpriced';

  return {
    ...seat,
    inputCredits: credits.inputCredits,
    outputCredits: credits.outputCredits,
    totalCredits: credits.inputCredits + credits.outputCredits,
  };
}

/**
 Puts a price on every metered seat a tally holds.

 @param tally - per-seat totals read out of a run log

 @returns Priced seats costliest first, the metered seats no price covered,
 the subscription seats, and what the priced ones came to

 @example
 ```ts
 const cost = priceTally({ tally: tallySpend({ lines, },), },);
 ```
 */
export function priceTally(
  { tally, }: { readonly tally: SpendTally; },
): SpendCost {
  /**
   Seats billed per token, which are the only ones a credit figure applies to.
   */
  const metered = tally
    .seats
    .filter(function isMetered(seat,): boolean {
      return seat.provider === 'hyper';
    },);

  /**
   Seats on the flat subscription, kept with their tokens and no credits.
   */
  const subscription = tally
    .seats
    .filter(function isSubscription(seat,): boolean {
      return seat.provider === 'synthetic';
    },);

  /**
   Seats on the free daily energy, kept with their tokens.
   */
  const freeEnergy = tally
    .seats
    .filter(function isFreeEnergy(seat,): boolean {
      return seat.provider === 'neuralwatt';
    },);

  /**
   Seats billed in USD, costliest first by what their lines reported.
   */
  const openRouter = tally
    .seats
    .filter(function isBilledInUsd(seat,): boolean {
      return (seat.provider === 'openrouter') || (seat.provider === 'bedrock')
        || (seat.provider === 'deepseek')
        || (seat.provider === 'xiaomi')
        || (seat.provider === 'anthropic');
    },)
    .toSorted(function costliestFirst(
      left,
      right,
    ): number {
      return right.costUsd - left.costUsd;
    },);

  /**
   What every OpenRouter seat's lines reported together.
   */
  const totalUsd = openRouter.reduce(
    function addUsd(
      running,
      seat,
    ): number {
      return running + seat.costUsd;
    },
    0,
  );

  /**
   Metered seats paired with what they came to, priced or not.
   */
  const attempted = metered.map(function price(seat,): {
    readonly seat: SeatSpend;
    readonly cost: PricedSeat | 'unpriced';
  } {
    return {
      seat,
      cost: priceSeat({ seat, },),
    };
  },);

  /**
   Seats the table could price, costliest first.
   */
  const priced = attempted
    .flatMap(function keepPriced(attempt,): readonly PricedSeat[] {
      return (attempt.cost === 'unpriced') ? [] : [attempt.cost,];
    },)
    .toSorted(function costliestFirst(
      left,
      right,
    ): number {
      return right.totalCredits - left.totalCredits;
    },);

  /**
   Seats the table had no row for, kept so the total reads as incomplete
   rather than as cheap.
   */
  const unpriced = attempted
    .filter(function isUnpriced(attempt,): boolean {
      return attempt.cost === 'unpriced';
    },)
    .map(function toSeat(attempt,): SeatSpend {
      return attempt.seat;
    },);

  /**
   What every priced seat came to together.
   */
  const totalCredits = priced.reduce(
    function addCredits(
      running,
      seat,
    ): number {
      return running + seat.totalCredits;
    },
    0,
  );

  /**
   Calls across every seat whose provider reported no usage at all.

   COUNTED OVER EVERY SEAT rather than the priced ones, because a quiet call
   on a subscription seat is just as invisible to a reader as a quiet metered
   one, and the figure exists to say how much went unseen.
   */
  const unreportedCalls = tally
    .seats
    .reduce(
    function addQuiet(
      running,
      seat,
    ): number {
      return running + seat.unreportedCalls;
    },
    0,
    );

  return {
    priced,
    unpriced,
    subscription,
    freeEnergy,
    openRouter,
    totalCredits,
    totalUsd,
    unreportedCalls,
    pricedAsOf: HYPER_PRICE_READ_ON,
  };
}

//endregion Spend cost
