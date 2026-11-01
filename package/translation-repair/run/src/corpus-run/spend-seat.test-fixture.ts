import type {
  PricedSeat,
  SeatSpend,
} from '../../dist/final/node/index.mjs';

//region Spend seat fixture
// Seats as the spend tally sums them and the price table prices them, built
// from a base a case overrides in the one field it is about.

/**
 A metered seat that made one call and was costed, reckoned and unreported on
 none.
 */
const BASE_SEAT: SeatSpend = {
  provider: 'hyper',
  model: 'qwen3.8-max',
  calls: 1,
  promptTokens: 1_000,
  completionTokens: 500,
  unreportedCalls: 0,
  costUsd: 0,
  costedCalls: 0,
  reckonedCalls: 0,
};

/**
 The fields of a seat a case changes, each optional because a case names only
 the ones it is about.
 */
type SeatOverrides = {
  /**
   Provider the seat's calls went to.
   */
  readonly provider?: SeatSpend['provider'];

  /**
   Model the seat's calls named.
   */
  readonly model?: SeatSpend['model'];

  /**
   Calls the seat made.
   */
  readonly calls?: SeatSpend['calls'];

  /**
   Prompt tokens across those calls.
   */
  readonly promptTokens?: SeatSpend['promptTokens'];

  /**
   Completion tokens across those calls.
   */
  readonly completionTokens?: SeatSpend['completionTokens'];

  /**
   Calls whose provider reported no usage.
   */
  readonly unreportedCalls?: SeatSpend['unreportedCalls'];

  /**
   USD the wire reported across those calls.
   */
  readonly costUsd?: SeatSpend['costUsd'];

  /**
   Calls whose line carried a cost.
   */
  readonly costedCalls?: SeatSpend['costedCalls'];

  /**
   Calls whose counts and cost were reckoned.
   */
  readonly reckonedCalls?: SeatSpend['reckonedCalls'];
};

/**
 Builds a seat from the base and the fields a case names.

 @param overrides - fields that differ from the base

 @returns The seat

 @example
 ```ts
 const seat = seatOf({ overrides: { calls: 3, }, },);
 ```
 */
export function seatOf({ overrides, }: { readonly overrides: SeatOverrides; },): SeatSpend {
  return {
    ...BASE_SEAT,
    ...overrides,
  };
}

/**
 Builds a priced seat from the base, the credits a case names and any fields
 that differ.

 @param inputCredits - what the prompt tokens came to

 @param outputCredits - what the completion tokens came to

 @param overrides - fields of the seat that differ from the base

 @returns The priced seat, whose total is its two halves together

 @example
 ```ts
 const priced = pricedSeatOf({ inputCredits: 40, outputCredits: 60, overrides: {}, },);
 ```
 */
export function pricedSeatOf(
  {
    inputCredits,
    outputCredits,
    overrides,
  }: {
    readonly inputCredits: number;
    readonly outputCredits: number;
    readonly overrides: SeatOverrides;
  },
): PricedSeat {
  return {
    ...seatOf({ overrides, },),
    inputCredits,
    outputCredits,
    totalCredits: inputCredits + outputCredits,
  };
}

//endregion Spend seat fixture
