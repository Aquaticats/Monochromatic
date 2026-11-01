import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  MODEL_CARDS,
  type ProviderName,
  type RosterModelId,
  routeTimeHoldsOf,
  type SeatHold,
  type SeatRole,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type NO_PROVIDER,
  providerServing,
} from './budget-routing.ts';
import type { ChatTextRequest, } from './chat-contract.ts';
import type { BudgetView, } from './provider-budget.ts';
import type { RouteTimeHoldsRule, } from './reask-alternate-models.ts';
import { measuresModelOf, } from './reask-mode.ts';
import { reachOf, } from './roster-reach.ts';

//region Models withheld from a role in a reading
// ONE QUESTION, ONE ANSWER: IS THIS MODEL WITHHELD FROM THIS ROLE IN THIS READING (ledger B1330, the owner's "Honour
// holds (Recommended)" and the lead's ruling that a withheld model is never that role's alternate). The run withholds
// `hyper-slow` and `hyper-slow-select` models from a bench while Hyper is the provider that would serve them and seats
// them otherwise (`judgeSeatsFor` in the run package), and the backfill of the writing benches and the re-ask of a
// runaway both ask this one function whether a model is withheld, so they cannot part. The router asks the same card the
// question of the provider it means to send a request to (`holdsKeepingRequestOff`, ledger B1365).

/**
 The role whose route-time holds are the ones every role honours, for a request that names no seat: the roster-wide role
 honours the fewest holds, `hyper-slow` among them, and that hold is in the list of every role. It is no default role for
 choosing among models (ledger B1079); it only says that a request with no seat is held to what no role is free of.
 */
const ROLE_WITHOUT_A_SEAT: SeatRole = 'roster';

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
  return routeTimeHoldsOnProvider({
    modelId,
    role,
    provider: providerServing({
      reach: reachOf({ modelId, },),
      dry,
    },),
  },);
}

/**
 The route-time holds of a model's card that keep it off a provider for a role: the holds the role honours at seating
 time, when the provider is Hyper, which is the provider every such hold names.

 This is the one place the run's holds are tied to a provider, so a reading that says Hyper would serve a model
 ({@link routeTimeHoldsInReading}) and a re-ask that would send the same model to Hyper
 ({@link routeTimeHoldsOnProvider} from the router) cannot part (ledger B1350).

 @param modelId - roster model under question

 @param role - role of the seat; absent for a request that names no seat, which is held to the holds every role honours

 @param provider - provider that would take the model's call

 @returns Holds in the role's order; empty where the provider is not one a hold names or the card carries none the
 role honours

 @example
 ```ts
 // The holds are ['hyper-slow'] for Hyper and none for Synthetic.
 const holds = routeTimeHoldsOnProvider({ modelId: 'hf:Qwen/Qwen3.8-27B', role: 'editor', provider: 'hyper', },);
 ```
 */
export function routeTimeHoldsOnProvider(
  {
    modelId,
    role = ROLE_WITHOUT_A_SEAT,
    provider,
  }: {
    readonly modelId: RosterModelId;
    readonly role?: SeatRole;
    readonly provider: ProviderName | typeof NO_PROVIDER;
  },
): readonly SeatHold[] {
  if (provider !== 'hyper')
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
 The route-time holds of a model's card that keep one request off the provider the router means to send it to: the
 holds the role of the request's seat honours, none for a measurement, and for a request that names no seat the holds
 every role honours (ledger B1365, B1373).

 THE ONE QUESTION BOTH ROUTER PATHS ASK. The first ask is refused where the policy chose a provider the hold names, and
 the hinted re-ask of the same model elsewhere leaves such a provider out. A request that names no seat is held to the
 holds every role honours on both: every production caller names its seat or its measurement
 (`request-seat-named.unit.test.ts` in the source-scan package reads that), so a request with none is a path nobody has
 named, and it may not reach a held provider (ledger B1373, the owner's "Honour holds (Recommended)"). A measurement is
 exempt on both, since it scores the model on the provider that carries it (ledger B1141, the owner's "Keep exempt
 (Recommended)").

 @param request - request being routed, read for its model and its seat

 @param provider - provider the router means to send it to

 @returns Holds in the role's order; empty where the request is not held off that provider

 @example
 ```ts
 const holds = holdsKeepingRequestOff({ request, provider: 'hyper', },);
 ```
 */
export function holdsKeepingRequestOff(
  {
    request,
    provider,
  }: {
    readonly request: ForeignBorrowed<ChatTextRequest>;
    readonly provider: ProviderName | typeof NO_PROVIDER;
  },
): readonly SeatHold[] {
  /**
   Seat the request names, which carries the role and the mode of the re-ask; absent for a request that names none.
   */
  const { reaskSeat, } = request;
  if (reaskSeat === undefined) {
    return routeTimeHoldsOnProvider({
      modelId: request.modelId,
      provider,
    },);
  }
  if (measuresModelOf({ seat: reaskSeat, }))
    return [];
  return routeTimeHoldsOnProvider({
    modelId: request.modelId,
    role: reaskSeat.role,
    provider,
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

/**
 Whether the run withholds a model from a role under the rule a re-ask reads its holds by (ledger B1372).

 The rule `always`, which a client with no reading and a failed read of the meters both give, withholds a model that
 carries any hold the role honours at seating time, as the pool of alternates does, so a seat is never asked on a guess.
 The rule `in-reading` withholds it only while the reading has Hyper serving it, and `by-the-caller` leaves the question
 to a caller that applies the reading itself.

 @param modelId - roster model under question

 @param role - role of the seat

 @param rule - how the holds applied at seating time are read

 @returns True when the rule withholds the model from the role

 @example
 ```ts
 const withheld = isWithheldUnderRule({ modelId: 'hf:Qwen/Qwen3.8-27B', role: 'editor', rule: { applied: 'always', }, },);
 ```
 */
export function isWithheldUnderRule(
  {
    modelId,
    role,
    rule,
  }: {
    readonly modelId: RosterModelId;
    readonly role: SeatRole;
    readonly rule: RouteTimeHoldsRule;
  },
): boolean {
  if (rule.applied === 'in-reading') {
    return isWithheldFromRole({
      modelId,
      role,
      dry: rule.dry,
    },);
  }
  if (rule.applied === 'by-the-caller')
    return false;
  return routeTimeHoldsOnProvider({
    modelId,
    role,
    provider: 'hyper',
  },)
    .length
    > 0;
}

//endregion Models withheld from a role in a reading
