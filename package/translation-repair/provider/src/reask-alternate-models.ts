import {
  reaskHoldsKeepingOutOf,
  isDecisionSeat,
  MODEL_CARDS,
  reaskPoolOf,
  ROSTER_MODEL_IDS,
  type RosterModelId,
  type SeatRole,
  standingOrderOf,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  RUNAWAY_READINGS,
  type RunawayReading,
  runawayShareOf,
} from './reask-runaway-readings.ts';

//region Re-ask alternate models
// WHICH MODELS A REASONING RUNAWAY IS RE-ASKED OF (ledger B1072, B1074, B1075, B1076). A reply that ended in a
// length stop with no content is a model that spent its whole completion cap thinking; asking the same weights
// again on another route repeats it (6 of the 7 direct DeepSeek calls that ran to the cap, in the runs of
// 2026-10-22 to 2026-10-24, were the re-ask of a NeuralWatt runaway of the same model, and each ran away again). The
// re-ask therefore goes to another roster model, and the owner's answers of 2026-10-30 decide which and in what
// order.
//
// ORDER: THE LOWEST SHARE OF RUNAWAYS FIRST ("Lowest runaway share (Recommended)"). `reask-runaway-readings.ts`
// holds the census the order rests on, the way a model with too few calls is placed, and why. FOR AN EDITOR OR A REFINER
// THE ORDER IS THE STANDING IN THAT ROLE FIRST ("Order by role standing (Recommended)", 2026-10-31, ledger B1144),
// from the table of `seat-role-standing.ts`, and the models never measured in the role follow by the share of runaways.
// `alternateOrderOf` is the one place the order is decided.
//
// ROLE: A MODEL OUTSIDE THE POOL OF THE SEAT'S ROLE IS NOT ASKED ("Honour holds (Recommended)"). A hold names the
// roles it empties, and the seat that ran away says which role it stands in (`ChatTextRequest.reaskSeat`). The pool of
// a role is read in the shared contract (`rolePoolOf` in `seat-role.ts`): the roster less the models the role's holds
// keep out, and for the checker role only the models the checker benchmark measured in, the list the run seats its
// checkers from (ledger B1100, the lead's ruling of 2026-10-31). A model the measurements put out of a role is never its
// alternate.
//
// BENCH: A MODEL SEATED ON THE QUESTION IS NOT ASKED (the lead's adoption of 2026-10-30 from "one judge alone never
// decides", 2026-10-25, "Two judges must name it", 2026-10-25, and `roster-id.ts` on one model in two seats). The
// answer is recorded as the voice of the seat that ran away; the same model's own answer is already a voice of its
// own seat, and two of them would count one model twice. A caller that knows its bench passes it.
//
// ONE VOICE PER QUESTION: A MODEL ASKED FOR ANOTHER SEAT OF THE QUESTION IS NOT ASKED (the owner's rule of 2026-10-30,
// ledger B1080). Two seats that ran away together would each take the first model left, and the plain question to a
// model is its own first ask's digest, so one claimed reply would be two voices. `reask-claims.ts` holds the record
// the seats share; this list leaves out what the record holds, and the asking seat takes its model from the record
// before it asks.
//
// THE ROLE HAS NO DEFAULT (ledger B1079). A caller that left it out was asked as the roster-wide role, which honours the
// fewest holds, so a wide gather after a runaway was re-asked of exactly the models its card drops from the wide seats.
//
// THREE KINDS OF SEAT ARE ALWAYS LEFT OUT and are not named in the log, since no run changes them: the model that
// ran away; a decision-only seat, which answers a typed question and never a chat completion; and the Claude route,
// which is metered on the owner's key ("This key would have $200 every month so be careful what you use", the lead's
// adoption of 2026-10-30 keeps it left out). Every other metered route is asked, since the router already spends on
// them for a first ask.

/**
 One model left out of a re-ask, with the reason a reader of the log needs.

 @example
 ```ts
 const left: LeftOutModel = { modelId: 'glm-5.3', why: 'held out of the wide-judge role by wide-seat-dropped', };
 ```
 */
export type LeftOutModel = {
  /**
   Model that is not asked.
   */
  readonly modelId: RosterModelId;

  /**
   Why it is not asked: its place on the bench, an earlier ask for the seat, or the holds of its card.
   */
  readonly why: string;
};

/**
 The models a reasoning runaway may be re-asked of, and the ones that were left out.

 @example
 ```ts
 const { alternates, leftOut, } = alternateModelsFor({ modelId: 'deepseek-v4.1-flash', role: 'roster', },);
 ```
 */
export type AlternateModels = {
  /**
   Models to ask, in the order to try them.
   */
  readonly alternates: readonly RosterModelId[];

  /**
   Models that could have been asked and were not, in roster order, each with why.
   */
  readonly leftOut: readonly LeftOutModel[];
};

/**
 Reasons a chat model that is not the runaway cannot take its re-ask.

 @param candidate - model to check

 @param role - role of the seat that ran away

 @param pool - models the role may take

 @param bench - models seated on the question

 @param askedAlready - models already asked for this seat

 @param askedForOtherSeats - models already asked for other seats of the question

 @returns Reasons, in the order bench, earlier ask, ask for another seat, pool; empty where the model may be asked

 @example
 ```ts
 const reasons = reasonsToLeaveOut({ candidate: 'glm-5.3', role: 'wide-judge', pool: [], bench: [], askedAlready: [], askedForOtherSeats: [], },);
 ```
 */
function reasonsToLeaveOut(
  {
    candidate,
    role,
    pool,
    bench,
    askedAlready,
    askedForOtherSeats,
  }: {
    readonly candidate: RosterModelId;
    readonly role: SeatRole;
    readonly pool: readonly RosterModelId[];
    readonly bench: readonly RosterModelId[];
    readonly askedAlready: readonly RosterModelId[];
    readonly askedForOtherSeats: readonly RosterModelId[];
  },
): readonly string[] {
  /**
   Holds of the candidate's card that the role honours.
   */
  const holds = reaskHoldsKeepingOutOf({
    modelId: candidate,
    role,
  },);
  return [
    ...(bench.includes(candidate,) ? ['seated on the bench of this question',] : []),
    ...(askedAlready.includes(candidate,) ? ['already asked for this seat',] : []),
    ...(askedForOtherSeats.includes(candidate,) ? ['already asked for another seat of this question',] : []),
    ...(pool.includes(candidate,) ? [] : [
      outOfPoolReason({
        role,
        holds,
      },),
    ]),
  ];
}

/**
 Why a model outside the pool of a role is not asked.

 @param role - role of the seat that ran away

 @param holds - holds of the model's card that the role honours

 @returns Holds when the role honours any, else that the role's measured list does not name the model

 @example
 ```ts
 const why = outOfPoolReason({ role: 'checker', holds: [], },);
 ```
 */
function outOfPoolReason(
  {
    role,
    holds,
  }: {
    readonly role: SeatRole;
    readonly holds: readonly string[];
  },
): string {
  return (holds.length === 0)
    ? `not on the measured list of the ${role} role`
    : `held out of the ${role} role by ${holds.join(' and ',)}`;
}

/**
 Orders models by the share of their calls that ran away, lowest first.

 @param modelIds - models to order, in roster order

 @param readings - census table the shares are read from

 @returns Same models: measured ones by share, then by roster position, then the unmeasured ones in roster order

 @example
 ```ts
 const ordered = orderedByRunawayShare({ modelIds: ['glm-5.3', 'minimax-m3',], readings: RUNAWAY_READINGS, },);
 ```
 */
function orderedByRunawayShare(
  {
    modelIds,
    readings,
  }: {
    readonly modelIds: readonly RosterModelId[];
    readonly readings: readonly RunawayReading[];
  },
): readonly RosterModelId[] {
  return modelIds.toSorted(function byShare(
    left,
    right,
  ): number {
    /**
     Measured share of the first, or the words `unmeasured`.
     */
    const leftShare = runawayShareOf({
      modelId: left,
      readings,
    },);

    /**
     Measured share of the second, or the words `unmeasured`.
     */
    const rightShare = runawayShareOf({
      modelId: right,
      readings,
    },);
    /**
     Positions in the roster, which break every tie.
     */
    const inRosterOrder = ROSTER_MODEL_IDS.indexOf(left,) - ROSTER_MODEL_IDS.indexOf(right,);
    if ((leftShare === 'unmeasured') && (rightShare === 'unmeasured'))
      return inRosterOrder;
    if (leftShare === 'unmeasured')
      return 1;
    if (rightShare === 'unmeasured')
      return -1;
    return (leftShare === rightShare) ? inRosterOrder : (leftShare - rightShare);
  },);
}

/**
 The order the models a re-ask may ask are tried in: the ONE PLACE it is decided (ledger B1144).

 The stand-ins of an editor or a refiner are tried by their standing in that role, best first, the owner's "Order by role
 standing (Recommended)" of 2026-10-31; every model never measured in the role, and every model of a role that has no
 standing recorded, follows by the share of its calls that ran away, lowest first ("Lowest runaway share
 (Recommended)", 2026-10-30).

 @param role - role of the seat that ran away

 @param modelIds - models that may be asked, in roster order

 @param readings - census table the share of runaways is read from

 @returns The same models in the order to try them

 @example
 ```ts
 const ordered = alternateOrderOf({ role: 'editor', modelIds, readings: RUNAWAY_READINGS, },);
 ```
 */
function alternateOrderOf(
  {
    role,
    modelIds,
    readings,
  }: {
    readonly role: SeatRole;
    readonly modelIds: readonly RosterModelId[];
    readonly readings: readonly RunawayReading[];
  },
): readonly RosterModelId[] {
  /**
   Models the role has a standing for, best first.
   */
  const measured = standingOrderOf({
    role,
    modelIds,
  },);
  return [
    ...measured,
    ...orderedByRunawayShare({
      modelIds: modelIds.filter(function unmeasured(modelId,): boolean {
        return !measured.includes(modelId,);
      },),
      readings,
    },),
  ];
}

/**
 Roster models a reasoning runaway may be re-asked of, in the order to try them, and the ones left out with why.

 @param modelId - model that ran away, left out

 @param role - role of the seat that ran away, with no default so that a new caller cannot leave it out and ask a model
 its card holds out of the seat (ledger B1079)

 @param bench - models seated on the question, left out so one model is never two voices

 @param askedAlready - models an earlier re-ask for this seat went to, left out so the next ask is another model's

 @param askedForOtherSeats - models asked for other seats of the question, left out so one model gives at most one
 voice per question (ledger B1080)

 @param readings - census table the order is read from

 @returns Chat models other than it that its seat's role may take, that are not on the metered Claude route and not on
 the bench, ordered by share of runaways; empty only where the pool, the bench and the earlier asks leave no model

 @example
 ```ts
 const [first,] = alternateModelsFor({ modelId: 'deepseek-v4.1-flash', role: 'roster', },).alternates;
 ```
 */
export function alternateModelsFor(
  {
    modelId,
    role,
    bench = [],
    askedAlready = [],
    askedForOtherSeats = [],
    readings = RUNAWAY_READINGS,
  }: {
    readonly modelId: RosterModelId;
    readonly role: SeatRole;
    readonly bench?: readonly RosterModelId[];
    readonly askedAlready?: readonly RosterModelId[];
    readonly askedForOtherSeats?: readonly RosterModelId[];
    readonly readings?: readonly RunawayReading[];
  },
): AlternateModels {
  /**
   Chat models other than the runaway that are not on the Claude route, in roster order.
   */
  const candidates = ROSTER_MODEL_IDS.filter(function isCandidate(candidate,): boolean {
    return (candidate !== modelId)
      && (!isDecisionSeat({ modelId: candidate, },))
      && (MODEL_CARDS[candidate]
        .anthropic
        === undefined);
  },);

  /**
   Models the role may take, which the candidates must be among.
   */
  const pool = reaskPoolOf({ role, },);

  /**
   Each candidate beside the reasons it cannot be asked.
   */
  const judged = candidates.map(function judge(candidate,): LeftOutModel & { readonly reasons: readonly string[]; } {
    /**
     Reasons this candidate is not asked.
     */
    const reasons = reasonsToLeaveOut({
      candidate,
      role,
      pool,
      bench,
      askedAlready,
      askedForOtherSeats,
    },);
    return {
      modelId: candidate,
      why: reasons.join('; ',),
      reasons,
    };
  },);
  return {
    alternates: alternateOrderOf({
      role,
      modelIds: judged
        .filter(function mayBeAsked(row,): boolean {
          return row.reasons
            .length
            === 0;
        },)
        .map(function nameOf(row,): RosterModelId {
          return row.modelId;
        },),
      readings,
    },),
    leftOut: judged
      .filter(function isLeftOut(row,): boolean {
        return row.reasons
          .length
          > 0;
      },)
      .map(function withoutReasons(row,): LeftOutModel {
        return {
          modelId: row.modelId,
          why: row.why,
        };
      },),
  };
}

/**
 Writes the models left out as one phrase for a log line.

 @param leftOut - models left out, in the order to be read

 @returns Each model with its reason in brackets, separated by commas, or the word `none`

 @example
 ```ts
 const phrase = describeLeftOut({ leftOut: [{ modelId: 'glm-5.3', why: 'seated on the bench of this question', },], },);
 ```
 */
export function describeLeftOut({ leftOut, }: { readonly leftOut: readonly LeftOutModel[]; },): string {
  if (leftOut.length === 0)
    return 'none';
  return leftOut
    .map(function describe(left,): string {
      return `${left.modelId} (${left.why})`;
    },)
    .join(', ',);
}

//endregion Re-ask alternate models
