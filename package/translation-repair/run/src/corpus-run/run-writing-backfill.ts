import { alternateModelsFor, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import { RUN_MODELS, } from './run-config.ts';

//region Writing bench backfill
// A WRITING BENCH (EDITORS OR REFINERS) IS DERIVED FROM THE READING, AS THE CHECKER BENCH IS. The owner's answer of
// 2026-11-01, asked what a run does when a writing bench has fewer than two reachable seats ("Backfill, and add
// sonnet"; `doc/decision/translation-repair-writing-bench-floor.md`): fill the dark seats with the next models of the
// owner's role order that a wet provider serves. Corpus pass XingZ6024 (2026-11-01, build e9b966af55) stopped both
// attempts with `refiners 1 of 3 reachable, floor 2` while Kimi-K3 and Qwen3.8-27B, both measured refiners and both
// served by the wet Synthetic, sat unused.
//
// ONE ORDER, NOT A COPY: the role order is the owner's "Order by role standing (Recommended)" of 2026-10-31 (ledger
// B1144), decided in one place by `alternateModelsFor` (the standing the editor calibration recorded for the role,
// best first, then the models never measured in it by their share of runaways), so the models that stand in for a
// runaway and the models that fill a dark seat are one list. The pool is the role's: the roster less the models the
// role's holds keep out, less the Claude route (metered on the owner's key; the only Claude seat of a writing bench
// is the one the owner named) and the decision-only seats.
//
// A SEAT THE RUN WITHHOLDS IS NEVER A STAND-IN (the owner's standing rule: honour every hold). The caller hands
// the reading's own answer to what a wet provider serves and which seats the run withholds there, so a model Hyper
// would serve too slowly is skipped for the next of the order and not seated.
//
// A CANDIDATE THAT ALSO CHECKS follows the contract in `assertCheckerIndependence` (`assembly`), which refuses a
// checker that also edits or refines unless the run permits it (`checkerSelfCertificationPermitted`, the owner's
// decision of 2026-08-24). A candidate on this reading's checker bench is therefore seated while the run permits the
// overlap, and skipped where it does not. A candidate off the checker bench is seated whatever the flag: the contract
// has nothing to refuse, and the ballot of a model that wrote the text is weighed by the model that answered, which the
// heard voice names even where it answered in place of a seat that ran away (`heardVoiceOf`, ledger B1142), so the
// per-issue discount follows the answerer. The earlier reason for skipping an off-bench candidate (that a re-ask of a
// checker is recorded under the seat that ran away, so the discount would miss the answerer) was false since B1142;
// ledger B1333 records the test that shows it.

/**
 A writing role the bench of which is derived from the reading.

 @example
 ```ts
 const role: WritingRole = 'refiner';
 ```
 */
export type WritingRole = 'editor' | 'refiner';

/**
 The seats a reading took from the role order, per bench, which the `JUDGE SEATS` line and the stop line name.

 @example
 ```ts
 const none: WritingBackfill = { editors: [], refiners: [], };
 ```
 */
export type WritingBackfill = {
  /**
   Editor seats that are not on the static editor bench.
   */
  readonly editors: readonly RosterModelId[];

  /**
   Refiner seats that are not on the static refiner bench.
   */
  readonly refiners: readonly RosterModelId[];
};

/**
 The benches that can take seats from the role order, in the order the words name them.
 */
const WRITING_BENCH_NAMES = [
  'editors',
  'refiners',
] as const;

/**
 One writing bench as one reading derives it.

 @example
 ```ts
 const { bench, backfilled, } = writingBenchFor({
   staticSeats, backfillOrder, served, seatable, checkers, selfCertificationPermitted,
 },);
 ```
 */
export type WritingBench = {
  /**
   The bench: the static seats a wet provider serves, then the backfill, then the static seats nobody serves, cut at the
   static width.
   */
  readonly bench: readonly RosterModelId[];

  /**
   The seats on the bench that came from the role order.
   */
  readonly backfilled: readonly RosterModelId[];
};

/**
 The models that may fill a dark seat of a role's static bench, in the owner's role order: the role's pool less the
 static bench, the Claude route and the decision-only seats.

 @param role - writing role the order is for

 @param staticSeats - static bench of the role, whose members are never candidates

 @returns Candidates, best first

 @example
 ```ts
 const order = backfillOrderOf({ role: 'refiner', staticSeats: RUN_MODELS.refinerModelIds, },);
 ```
 */
export function backfillOrderOf(
  {
    role,
    staticSeats,
  }: {
    readonly role: WritingRole;
    readonly staticSeats: readonly RosterModelId[];
  },
): readonly RosterModelId[] {
  /**
   A seat on the bench already, named as the one the order stands in for: `alternateModelsFor` leaves it out as the
   model that ran away, and the bench leaves every static seat out, so the first is only a name for the question.
   */
  const standingIn = nonNullishOrThrow(staticSeats[0],);
  return alternateModelsFor({
    modelId: standingIn,
    role,
    bench: staticSeats,
    // THE READING DECIDES, NOT THE ORDER: a model withheld while Hyper would serve it (Qwen3.8-27B) stays a candidate,
    // and `writingBenchFor` asks the run's seating, with the same function a re-ask leaves its alternates out by
    // (`routeTimeHoldsInReading`), whether the reading seats it (ledger B1330).
    routeTimeHolds: { applied: 'by-the-caller', },
  },)
    .alternates;
}

/**
 The editor candidates the role order gives, fixed by the static bench.
 */
export const EDITOR_BACKFILL_ORDER: readonly RosterModelId[] = backfillOrderOf({
  role: 'editor',
  staticSeats: RUN_MODELS.editorModelIds,
},);

/**
 The refiner candidates the role order gives, fixed by the static bench.
 */
export const REFINER_BACKFILL_ORDER: readonly RosterModelId[] = backfillOrderOf({
  role: 'refiner',
  staticSeats: RUN_MODELS.refinerModelIds,
},);

/**
 Derives one writing bench for one reading.

 THE STATIC SEATS ARE FILTERED BY `seatable` TOO (ledger B1330): a hold is honoured for every model of a bench, the
 owner's hand-picked first choice included, although no static seat is withheld by a reading today. A static seat a wet
 provider serves and the reading withholds leaves its place open for the role order, as one no provider serves does.

 @param staticSeats - static bench of the role, whose size is the width

 @param backfillOrder - role order of the models that may fill a dark seat

 @param served - whether a wet provider would take a model's calls in this reading

 @param seatable - whether the run seats a model in this reading: served, and not withheld on the provider that would
 serve it

 @param checkers - checkers the same reading seats

 @param selfCertificationPermitted - whether the contract lets a checker confirm text it wrote

 @returns Bench, never wider than the static one, and the seats it took from the role order

 @example
 ```ts
 const refiners = writingBenchFor({
   staticSeats, backfillOrder, served, seatable, checkers, selfCertificationPermitted: true,
 },);
 ```
 */
export function writingBenchFor(
  {
    staticSeats,
    backfillOrder,
    served,
    seatable,
    checkers,
    selfCertificationPermitted,
  }: {
    readonly staticSeats: readonly RosterModelId[];
    readonly backfillOrder: readonly RosterModelId[];
    readonly served: (modelId: RosterModelId,) => boolean;
    readonly seatable: (modelId: RosterModelId,) => boolean;
    readonly checkers: readonly RosterModelId[];
    readonly selfCertificationPermitted: boolean;
  },
): WritingBench {
  /**
   Static seats a wet provider serves and the reading seats, in the static order.
   */
  const servedSeats = staticSeats.filter(function isSeated(modelId,): boolean {
    return served(modelId,) && seatable(modelId,);
  },);
  /**
   Places the served static seats leave open.
   */
  const open = staticSeats.length - servedSeats.length;
  /**
   Candidates in the role order that this reading may seat, cut at the open places.
   */
  const mayFill = backfillOrder.filter(function isSeatable(modelId,): boolean {
    if (!(served(modelId,) && seatable(modelId,)))
      return false;
    // A CANDIDATE THAT IS ALSO A CHECKER IS SKIPPED ONLY WHERE THE CONTRACT WOULD REFUSE THE OVERLAP (ledger B1333).
    return selfCertificationPermitted || (!checkers.includes(modelId,));
  },);
  /**
   The candidates the open places take, in the order of the role.
   */
  const backfilled = mayFill.slice(
    0,
    open,
  );
  return {
    bench: [
      ...servedSeats,
      ...backfilled,
      ...staticSeats.filter(function unserved(modelId,): boolean {
        return !served(modelId,);
      },),
    ].slice(
      0,
      staticSeats.length,
    ),
    backfilled,
  };
}

/**
 Words for the seats a reading took from the role order, for the `JUDGE SEATS` line and the stop line.

 @param backfilled - seats taken, per bench

 @returns `none`, or each bench that took seats, as `editors:a,b|refiners:c`

 @example
 ```ts
 backfillWordsOf({ backfilled: { editors: [], refiners: ['hf:Qwen/Qwen3.8-27B',], }, },);
 // => 'refiners:hf:Qwen/Qwen3.8-27B'
 ```
 */
export function backfillWordsOf(
  { backfilled, }: { readonly backfilled: WritingBackfill; },
): string {
  /**
   Each bench that took seats, with them.
   */
  const parts = WRITING_BENCH_NAMES.flatMap(function wordsOf(bench,): readonly string[] {
    /**
     Seats the bench took.
     */
    const taken = backfilled[bench];
    return (taken.length === 0) ? [] : [`${bench}:${taken.join(',',)}`,];
  },);
  return (parts.length === 0) ? 'none' : parts.join('|',);
}

//endregion Writing bench backfill
