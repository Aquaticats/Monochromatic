import type { ReaskSeat, } from './chat-contract.ts';

//region Re-ask mode
// WHOSE ANSWER THE RE-ASK OF A REASONING RUNAWAY MAY GIVE (ledger B1141). A run re-asks a runaway of another model
// its role may take, since the question is what matters and the model is not. A measurement (a probe, a bench, a
// calibration, a census) re-asks the model it measures, on another provider, nudged, since the score it reports is that
// model's and a rival's answer recorded as its own would be scored as its own. The three places that re-ask (the
// provider's `reaskElsewhereNudged`, the stage's recovery round and the picture reader's cut re-ask) read the mode
// from the seat through this one reader, so they cannot part on what a measurement is.

/**
 Whether the seat belongs to a measurement, whose re-ask of a runaway goes to the measured model itself.

 @param seat - seat the re-ask stands in, absent where the request names none

 @returns True when the seat names the measured-model mode; false for the run's mode and for a request with no seat

 @example
 ```ts
 const measuring = measuresModelOf({ seat: request.reaskSeat, });
 ```
 */
export function measuresModelOf({ seat, }: { readonly seat: ReaskSeat | undefined; },): boolean {
  return seat?.reask === 'measured-model';
}

//endregion Re-ask mode
