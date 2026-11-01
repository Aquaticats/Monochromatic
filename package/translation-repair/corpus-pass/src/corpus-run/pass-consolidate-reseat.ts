import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import type { ConsolidateSliceSeating, } from '@monochromatic-dev/translation-repair-artifact/ts';
import { configuredConsolidationPolish, } from '@monochromatic-dev/translation-repair-consolidate/ts';
import { MIN_SELECTION_BALLOTS, } from '@monochromatic-dev/translation-repair-translate-stage/ts';
import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  type Reseated,
  reseatHookFor,
} from './pass-reseat-hook.ts';
import {
  type JudgeSeats,
  reachableSeats,
  type SeatReadingClient,
} from '@monochromatic-dev/translation-repair-run/ts';

//region Consolidation re-seating
// THE CONSOLIDATION'S PER-SLICE HOOK, split out of `pass-consolidate.ts` so a
// test can drive it with the reading rig the lanes' hooks are tested with
// (ledger H5). Every slice waits out a named hold that keeps the slate from
// quorum (the thirteenth class's second face, the fourth hakureico pass).
//
// IT RE-SEATS THE SLICE TOO (ledger H5, 2026-09-28), as the lanes' hooks do
// (`pass-reseat.ts`). It only waited, so a dry-out inside the consolidation
// left every later slice on the writers, slate judges and naturalness roles
// read before it, the stage classes one hundred three, one hundred nine and
// one hundred thirteen had each fixed for one repair stage at a time. While a
// hold runs it reads the seats again, which waits out the hold where the bench
// cannot reach quorum, builds the roster `pass-consolidate.ts` builds from that
// reading, and keeps handing it over once the hold has ended
// (`pass-reseat-hook.ts`). While nothing is held and the slate reach is at or above the floor it costs one synchronous
// read of the holds; with the reach under the floor it also reads the seats again for every slice, on a budget reading
// trusted for 60 seconds (`BUDGET_FRESH_MS` in `provider-budget.ts`), since a provider returning after its hold ended
// would lift it.

/**
 Hook the consolidation driver calls before each slice.

 @example
 ```ts
 const hooks: ConsolidationHooks = consolidationHooksFor({ client, signal, prepared, l, },);
 ```
 */
export type ConsolidationHooks = {
  readonly beforeSlice: () => Promise<ConsolidateSliceSeating>;
};

/**
 Slate judges of one reading a wet provider would serve, which an eligible
 standing is held to the selection's floor by before a slate is bought.

 @param seats - benches as of this slice

 @returns Reachable slate judges, counted

 @example
 ```ts
 const reachable = slateReachOf({ seats, },);
 ```
 */
export function slateReachOf(
  { seats, }: { readonly seats: JudgeSeats; },
): number {
  return reachableSeats({
    seats: seats.slateJudges,
    dry: seats.dry,
  },)
    .length;
}

/**
 Whether the seating last handed over states a slate reach below the floor.

 READ AGAIN EVEN WHEN NO HOLD RUNS, since a provider whose hold ended is wet
 again and the reach handed over would otherwise stay the one read before it.

 @param seating - seating the hook last handed over

 @returns True when an eligible standing would buy no slate on it

 @example
 ```ts
 const stale = slateReachBelowFloor({ slateReachable: 1, },);
 ```
 */
function slateReachBelowFloor(seating: ConsolidateSliceSeating,): boolean {
  return (seating.slateReachable !== undefined) && (seating.slateReachable < MIN_SELECTION_BALLOTS);
}

/**
 What the consolidation takes from a seat reading: the writers, slate judges
 and naturalness roles, as `pass-consolidate.ts` seats them.

 @param seats - benches as of this slice

 @param prepared - page whose guard facts the naturalness roles carry

 @returns Roster the slice runs on, beside the line naming it

 @example
 ```ts
 const { seating, line, } = consolidationSeatingOf({ seats, prepared, },);
 ```
 */
function consolidationSeatingOf(
  {
    seats,
    prepared,
  }: {
    readonly seats: JudgeSeats;
    readonly prepared: PreparedDocumentPair;
  },
): Reseated<ConsolidateSliceSeating> {
  /**
   Writers the slice runs on, for the line.
   */
  const writers = seats.writers
    .join(',',);
  /**
   Slate judges the slice runs on, for the line.
   */
  const judges = seats.slateJudges
    .join(',',);
  /**
   Naturalness gate the slice runs on, for the line: the reading re-seats it,
   while the polish itself is always configured, the run's roster seating its
   refiners (`RunRepairModels`), so the line names the gate rather than a kind
   that never varies.
   */
  const polishGate = seats.lateJudges
    .join(',',);
  return {
    seating: {
      slateReachable: slateReachOf({ seats, },),
      roster: {
        modelIds: seats.writers,
        judgeModelIds: seats.slateJudges,
        polishConfig: configuredConsolidationPolish({
          prepared,
          models: seats.repairModels,
          gateModelIds: seats.lateJudges,
        },),
      },
    },
    line: `slice re-seated under a hold: writers=${writers} slateJudges=${judges} polishGate=${polishGate}`,
  };
}

/**
 Builds the consolidation's per-slice hook.

 @param client - run client whose dryness view and holds are the router's own

 @param signal - entry abort the readings honour

 @param prepared - page whose guard facts the naturalness roles carry, as the
 consolidation's starting roster carries them

 @param l - entry logger

 @param startedSlateReachable - reachable slate judges in the reading the
 consolidation started on, which a slice keeps until a hold or a reach below
 the floor makes the hook read again; absent where the caller did not count

 @returns Per-slice wait, handing the slice the roster it runs on

 @example
 ```ts
 const hooks = consolidationHooksFor({ client, signal, prepared, l, },);
 ```
 */
export function consolidationHooksFor(
  {
    client,
    signal,
    prepared,
    l,
    startedSlateReachable,
  }: {
    readonly client: SeatReadingClient;
    readonly signal: AbortSignal;
    readonly prepared: PreparedDocumentPair;
    readonly l: Logger;
    readonly startedSlateReachable?: number;
  },
): ConsolidationHooks {
  return {
    beforeSlice: reseatHookFor<ConsolidateSliceSeating>({
      client,
      signal,
      phase: 'consolidation',
      unseated: (startedSlateReachable === undefined) ? {} : { slateReachable: startedSlateReachable, },
      staleWhen: slateReachBelowFloor,
      seatingOf: function seatingOfThisPage(
        { seats, }: { readonly seats: JudgeSeats; },
      ): Reseated<ConsolidateSliceSeating> {
        return consolidationSeatingOf({
          seats,
          prepared,
        },);
      },
      l,
    },),
  };
}

//endregion Consolidation re-seating
