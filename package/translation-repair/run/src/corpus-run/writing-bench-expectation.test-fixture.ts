import {
  alternateModelsFor,
  type BudgetView,
  NO_PROVIDER,
  providerServing,
  reachOf,
} from '@monochromatic-dev/translation-repair-provider';
import {
  MEASURED_CHECKER_ORDER,
  type RosterModelId,
  type SeatRole,
} from '@monochromatic-dev/translation-repair-shared-contract';

//region Writing bench expectation
// TEST SUPPORT, NOT PACKAGE SOURCE. WHAT A WRITING BENCH (EDITORS OR REFINERS) MUST BE UNDER ONE READING OF THE METERS,
// worked out from the pieces the owner's answer of 2026-11-01 ("Backfill, and add sonnet") names and nothing the
// derivation under test returns: the static bench, the owner's role order (`alternateModelsFor`, which orders a role's
// pool by standing and then by the share of runaways), who serves what (`providerServing`), the hold on Hyper-slow
// models and the rule for a model that also checks. A case compares the bench the run derives with this one, so the
// derivation cannot be right by agreeing with itself.

/**
 One writing bench as one reading of the meters must derive it.

 @example
 ```ts
 const expected: ExpectedWritingBench = { bench, backfilled, };
 ```
 */
export type ExpectedWritingBench = {
  /**
   The bench in seat order: the static seats a wet provider serves, then the backfill, then the static seats nobody
   serves, cut at the static width.
   */
  readonly bench: readonly RosterModelId[];

  /**
   The seats the bench holds that the static bench does not, in the order they were taken.
   */
  readonly backfilled: readonly RosterModelId[];
};

/**
 Works out the bench a role must have in one reading.

 @param role - writing role, editor or refiner

 @param dry - which providers are dry in the reading

 @param staticSeats - the role's static bench, whose size is the width

 @param checkerBench - checkers the same reading seats

 @param hyperSlow - models withheld while Hyper is the provider that would serve them

 @param selfCertificationPermitted - whether the contract lets a checker confirm text it wrote

 @returns Bench and the seats it took from the role order

 @example
 ```ts
 const { bench, } = expectedWritingBench({ role: 'refiner', dry, staticSeats, checkerBench, hyperSlow, selfCertificationPermitted: true, },);
 ```
 */
export function expectedWritingBench(
  {
    role,
    dry,
    staticSeats,
    checkerBench,
    hyperSlow,
    selfCertificationPermitted,
  }: {
    readonly role: Extract<SeatRole, 'editor' | 'refiner'>;
    readonly dry: BudgetView;
    readonly staticSeats: readonly RosterModelId[];
    readonly checkerBench: readonly RosterModelId[];
    readonly hyperSlow: ReadonlySet<RosterModelId>;
    readonly selfCertificationPermitted: boolean;
  },
): ExpectedWritingBench {
  /**
   Provider that would take a model's calls, or none.

   @param modelId - seat under question

   @returns First wet provider that serves it
   */
  function servedBy(modelId: RosterModelId,): ReturnType<typeof providerServing> {
    return providerServing({
      reach: reachOf({ modelId, },),
      dry,
    },);
  }
  /**
   Whether a wet provider serves a seat.

   @param modelId - seat under question

   @returns True when a call to it would be routed somewhere
   */
  function served(modelId: RosterModelId,): boolean {
    return servedBy(modelId,) !== NO_PROVIDER;
  }
  /**
   Any static seat, which the order's lookup leaves out as the one that ran away, as it is on the bench already.
   */
  const [anyStatic,] = staticSeats;
  if (anyStatic === undefined)
    throw new Error('a writing bench has at least one static seat',);
  /**
   The role's pool less the static bench, the Claude route and the decision-only seats, in the owner's role order.
   */
  const { alternates, } = alternateModelsFor({
    modelId: anyStatic,
    role,
    bench: staticSeats,
  },);
  /**
   Places the static bench has that no static seat a wet provider serves fills.
   */
  const open = Math.max(
    0,
    staticSeats.length - staticSeats.filter(served,).length,
  );
  /**
   Candidates a reading may seat: served, not withheld while Hyper would serve them, and, when also a checker, on the
   reading's own checker bench and only where the contract permits a checker to confirm text it wrote.
   */
  const backfilled = alternates
    .filter(function seatable(modelId,): boolean {
      if (!served(modelId,))
        return false;
      if ((servedBy(modelId,) === 'hyper') && hyperSlow.has(modelId,))
        return false;
      if (!MEASURED_CHECKER_ORDER.includes(modelId,))
        return true;
      return selfCertificationPermitted && checkerBench.includes(modelId,);
    },)
    .slice(
      0,
      open,
    );
  return {
    bench: [
      ...staticSeats.filter(served,),
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

//endregion Writing bench expectation
