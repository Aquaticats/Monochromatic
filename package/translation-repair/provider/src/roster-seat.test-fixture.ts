import type { ReaskSeat, } from '../dist/final/node/index.mjs';

//region Roster seat
// THE SEAT A CASE NAMES WHEN IT CARES FOR NEITHER THE ROLE NOR THE BENCH (ledger B1373). Every ask of a stage names its
// seat, since the router holds a model off the provider its card names for the seat's role, so a case that drives a
// round or a call directly names one too: the roster-wide role, which honours the fewest holds, no bench and the mode of
// a pass.
//
// TEST SUPPORT, NOT PACKAGE SOURCE.

/**
 The seat of a case that cares for neither the role nor the bench of the question.

 @example
 ```ts
 const voice = await attemptStageCall({ ..., reaskSeat: ROSTER_SEAT, },);
 ```
 */
export const ROSTER_SEAT: ReaskSeat = {
  role: 'roster',
  reask: 'other-model',
  bench: [],
};

//endregion Roster seat
