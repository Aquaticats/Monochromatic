import {
  MODEL_CARDS,
  type RosterModelId,
  routeTimeHoldsOf,
  type SeatHold,
  type SeatRole,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { providerServing, } from './budget-routing.ts';
import type { BudgetView, } from './provider-budget.ts';
import { reachOf, } from './roster-reach.ts';

//region Models withheld from a role in a reading
// ONE QUESTION, ONE ANSWER: IS THIS MODEL WITHHELD FROM THIS ROLE IN THIS READING (ledger B1330, the owner's "Honour
// holds (Recommended)" and the lead's ruling that a withheld model is never that role's alternate). The run withholds
// `hyper-slow` and `hyper-slow-select` models from a bench while Hyper is the provider that would serve them and seats
// them otherwise (`judgeSeatsFor` in the run package), and the backfill of the writing benches and the re-ask of a
// runaway both ask this one function whether a model is withheld, so they cannot part.

/**
 The route-time holds of a model's card that keep it out of a role in a reading of the meters: the holds the role
 honours at seating time, while Hyper is the provider that would take the model's calls.

 @param modelId - roster model under question

 @param role - role of the seat

 @param dry - which providers are out of budget in the reading

 @returns Holds in the role's order; empty where the reading seats the model in the role

 @example
 ```ts
 // The holds are ['hyper-slow'] with Hyper wet and none with Synthetic wet.
 const holds = routeTimeHoldsInReading({ modelId: 'hf:Qwen/Qwen3.8-27B', role: 'editor', dry, },);
 ```
 */
export function routeTimeHoldsInReading(
  {
    modelId,
    role,
    dry,
  }: {
    readonly modelId: RosterModelId;
    readonly role: SeatRole;
    readonly dry: BudgetView;
  },
): readonly SeatHold[] {
  /**
   Provider the router would send the model's calls to in this reading.
   */
  const serving = providerServing({
    reach: reachOf({ modelId, },),
    dry,
  },);
  if (serving !== 'hyper')
    return [];
  /**
   Holds the card carries.
   */
  const { holds, } = MODEL_CARDS[modelId];
  return routeTimeHoldsOf({ role, },)
    .filter(function carried(hold,): boolean {
      return holds.includes(hold,);
    },);
}

/**
 Whether the run withholds a model from a role in a reading of the meters.

 @param modelId - roster model under question

 @param role - role of the seat

 @param dry - which providers are out of budget in the reading

 @returns True when a route-time hold of the model's card that the role honours applies in the reading

 @example
 ```ts
 const withheld = isWithheldFromRole({ modelId: 'hf:Qwen/Qwen3.8-27B', role: 'refiner', dry, },);
 ```
 */
export function isWithheldFromRole(
  {
    modelId,
    role,
    dry,
  }: {
    readonly modelId: RosterModelId;
    readonly role: SeatRole;
    readonly dry: BudgetView;
  },
): boolean {
  return routeTimeHoldsInReading({
    modelId,
    role,
    dry,
  },)
    .length
    > 0;
}

//endregion Models withheld from a role in a reading
