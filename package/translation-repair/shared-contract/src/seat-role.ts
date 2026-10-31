import { recordOver, } from './model-card-derive.ts';
import type { SeatHold, } from './model-card.ts';
import { MODEL_CARDS, } from './model-cards.ts';
import {
  ROSTER_MODEL_IDS,
  type RosterModelId,
} from './roster-id.ts';

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
// - The editors and the refiners are hand-picked benches of the writers: the editor calibration seats the roster's
//   writers in both chairs (`editor-calibrate-lane.ts`), so their pool is the writers' (ledger B1100).
//
// A ROLE ALSO HAS A POOL (ledger B1100, the lead's ruling of 2026-10-31): the models a re-ask of that role may ask. A
// re-ask of the checker seat asks only a model the checker benchmark measured in, and no hold of a card says so, since
// the cards carry no measurement of that role: the pool of the checker role is the measured order itself, which lives
// here ({@link MEASURED_CHECKER_ORDER}) and which `run-config.ts` takes its `RUN_CHECKER_ORDER` from, so the bench
// and the re-ask read one list. Every other role's pool is the roster less the models its holds keep out.
//
// HOLDS OF A ROUTE ARE NOT HERE: `openrouter-dropped`, `openrouter-withheld`, `synthetic-withheld` and
// `hyper-slow-select` keep a model off a provider or off the select seats while one provider serves it, which the
// router and `run-seats.ts` read at the call. A re-ask is routed like a first ask, so they apply to it without a
// table.

/**
 Every role a re-ask can be asked in, widest first.

 `roster` is the role of a seat whose bench is the run roster and nothing narrower (pairing and admission seats).
 `editor` and `refiner` are the hand-picked benches of the repair lane, whose pool is the writers the editor calibration
 seats in those chairs (ledger B1100). No caller leaves its role out (ledger B1079).

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
  'select-judge',
  'slate-judge',
  'reader',
  'checker',
  'editor',
  'refiner',
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
  // THE SELECT SEATS ARE THE WIDE SEATS' BENCH AND THE SLATE SEATS THE LATE JUDGES' (ledger B1143): the same static holds.
  'select-judge': [
    'owner-culled',
    'judge-unmeasured',
    'wide-seat-dropped',
  ],
  'slate-judge': [
    'owner-culled',
    'judge-unmeasured',
    'late-judge-dropped',
  ],
  reader: [
    'owner-culled',
    'reader-unmeasured',
  ],
  checker: ['owner-culled',],
  editor: [
    'owner-culled',
    'judge-unmeasured',
    'writer-unmeasured',
  ],
  refiner: [
    'owner-culled',
    'judge-unmeasured',
    'writer-unmeasured',
  ],
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

/**
 Checkers in the order the checker benchmark of 2026-09-27 ranks them, which `run-config.ts` seats its bench from and
 the re-ask of a checker reads its pool from (ledger B1100). The benchmark, why each model is in or out of the order
 and what changed on 2026-10-29 are on `RUN_CHECKER_ORDER` in the run package, which names this list.

 @example
 ```ts
 const first = MEASURED_CHECKER_ORDER[0];
 ```
 */
export const MEASURED_CHECKER_ORDER: readonly RosterModelId[] = [
  'xiaomi/mimo-v2.6-flash',
  'hf:moonshotai/Kimi-K3',
  // HAIKU 5.5 JOINED THE ORDER AT THE OWNER'S "Haiku wide" (2026-10-22), UNMEASURED AS A CHECKER, behind the three the
  // benchmark ranks and in front of the two it ranks lowest, so that it filled the bench only when a provider serving one
  // of those three was dry. The order lost its first seat on 2026-10-29, so it now holds the third place of the static
  // bench of three: the Claude key serves a checker round in every reading in which it is wet, and when it is dry the
  // next seat a wet provider serves takes the place. It sends no effort; the effort bench decides that.
  'claude-haiku-5-5',
  'google.gemma-4-31b',
  'inception/mercury-2.5',
];

/**
 The models a re-ask of a seat in a role may ask, in roster order (the measured order for the checker role): the models
 no hold of the role keeps out, and for the checker role only those the checker benchmark measured in.

 It is the pool the run's benches of that role are drawn from, not the bench: the re-ask leaves out the bench, the
 model that ran away and the other seats' asks itself (ledger B1100). A pool is a reading of the bench derivation in
 `run-config.ts`, and `seat-role-pool.unit.test.ts` in the run package fails if the two drift apart.

 @param role - role of the seat that ran away

 @returns Models the role may take, which include decision-only seats and Claude cards that the re-ask itself never asks

 @example
 ```ts
 const pool = rolePoolOf({ role: 'checker', },);
 ```
 */
export function rolePoolOf({ role, }: { readonly role: SeatRole; },): readonly RosterModelId[] {
  return (role === 'checker' ? MEASURED_CHECKER_ORDER : ROSTER_MODEL_IDS)
    .filter(function heldOutOfNothing(modelId,): boolean {
      return holdsKeepingOutOf({
        modelId,
        role,
      },)
        .length
        === 0;
    },);
}

/**
 The models a re-ask of a seat in a role may ask: the pool of the role, less the models the run keeps off the seats of
 that role at the call (ledger B1143).

 @param role - role of the seat that ran away

 @returns Models the re-ask may ask

 @example
 ```ts
 const pool = reaskPoolOf({ role: 'select-judge', },);
 ```
 */
export function reaskPoolOf({ role, }: { readonly role: SeatRole; },): readonly RosterModelId[] {
  return rolePoolOf({ role, },);
}

//endregion Seat role
