import { recordOver, } from './model-card-derive.ts';
import type { SeatHold, } from './model-card.ts';
import { MODEL_CARDS, } from './model-cards.ts';
import type { RosterModelId, } from './roster-id.ts';

//region Seat role
// A HOLD NAMES A ROLE, AND A RE-ASK ASKS A MODEL OF A ROLE (ledger B1074, the owner's answer of 2026-10-30,
// "Honour holds (Recommended)"). A card says which roles a model is held out of (`SeatHold`), and each hold is read
// by the run's bench derivation for the roles it empties (`run-config.ts`, `run-seats.ts`). A question re-asked of
// another model must not seat that model in a role its card holds it out of, so the re-ask needs the role of the seat
// it stands in. No request carried one, so `ChatTextRequest.reaskSeat` carries it now.
//
// `SEAT_ROLE_HOLDS` IS THE BENCH DERIVATION'S, ROLE BY ROLE, READ ON 2026-10-30 (`run-config.ts`):
//
// - `RUN_ROSTER` drops `judge-unmeasured` and `owner-culled`, and every bench but the checkers and the picture
//   readers derives from it.
// - `RUN_TRANSLATORS` also drops `writer-unmeasured` and `translator-dropped`; `RUN_WRITERS` drops
//   `writer-unmeasured`.
// - `RUN_WIDE_SEATS` drops `wide-seat-dropped`; `RUN_LATE_JUDGES` drops `late-judge-dropped`.
// - `RUN_READER_MODELS` drops `reader-unmeasured` and `owner-culled` only ("Gemma 4 31B already reads without
//   judging"), so a model the judge probe withheld can still read.
// - The checker bench is a static order that only the cull empties.
//
// HOLDS OF A ROUTE ARE NOT HERE: `openrouter-dropped`, `openrouter-withheld`, `synthetic-withheld` and
// `hyper-slow-select` keep a model off a provider or off the select seats while one provider serves it, which the
// router and `run-seats.ts` read at the call. A re-ask is routed like a first ask, so they apply to it without a
// table.

/**
 Every role a re-ask can be asked in, widest first.

 `roster` is the role of a seat whose bench is the run roster and nothing narrower (pairing, admission, editor and
 refiner seats). No caller leaves its role out (ledger B1079).

 @example
 ```ts
 const roles = SEAT_ROLES.map(function nameOf(role,) { return role; },);
 ```
 */
export const SEAT_ROLES = [
  'roster',
  'translator',
  'writer',
  'wide-judge',
  'late-judge',
  'reader',
  'checker',
] as const;

/**
 A role a seat stands in; see {@link SEAT_ROLES}.

 @example
 ```ts
 const role: SeatRole = 'wide-judge';
 ```
 */
export type SeatRole = typeof SEAT_ROLES[number];

/**
 The holds each role honours, in the order a model is checked against them, written once per role.
 */
const HOLDS_OF_ROLE: Readonly<Record<SeatRole, readonly SeatHold[]>> = {
  roster: [
    'owner-culled',
    'judge-unmeasured',
  ],
  translator: [
    'owner-culled',
    'judge-unmeasured',
    'writer-unmeasured',
    'translator-dropped',
  ],
  writer: [
    'owner-culled',
    'judge-unmeasured',
    'writer-unmeasured',
  ],
  'wide-judge': [
    'owner-culled',
    'judge-unmeasured',
    'wide-seat-dropped',
  ],
  'late-judge': [
    'owner-culled',
    'judge-unmeasured',
    'late-judge-dropped',
  ],
  reader: [
    'owner-culled',
    'reader-unmeasured',
  ],
  checker: ['owner-culled',],
};

/**
 The holds each role honours, in the order a model is checked against them, built over the list of roles so the record
 carries exactly those roles and fails at import where a role is listed twice.

 @example
 ```ts
 const honoured = SEAT_ROLE_HOLDS.reader;
 ```
 */
export const SEAT_ROLE_HOLDS: Readonly<Record<SeatRole, readonly SeatHold[]>> = recordOver({
  keys: SEAT_ROLES,
  of: function honouredBy(role,): readonly SeatHold[] {
    return HOLDS_OF_ROLE[role];
  },
},);

/**
 The holds of a model's card that keep it out of a role.

 @param modelId - roster model a re-ask might ask

 @param role - role of the seat the re-ask stands in, which decides which holds count

 @returns Holds the card carries that the role honours, in the role's order; empty where the card holds the model out
 of nothing in that role

 @example
 ```ts
 const holds = holdsKeepingOutOf({ modelId: 'hf:Qwen/Qwen3.8-27B', role: 'wide-judge', },);
 ```
 */
export function holdsKeepingOutOf(
  {
    modelId,
    role,
  }: {
    readonly modelId: RosterModelId;
    readonly role: SeatRole;
  },
): readonly SeatHold[] {
  /**
   Holds the card carries.
   */
  const { holds, } = MODEL_CARDS[modelId];
  return SEAT_ROLE_HOLDS[role]
    .filter(function carried(hold,): boolean {
      return holds.includes(hold,);
    },);
}

//endregion Seat role
