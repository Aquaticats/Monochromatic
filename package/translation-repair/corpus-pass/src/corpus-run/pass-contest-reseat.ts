import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import { LANE_CONTEST_QUORUM, } from '@monochromatic-dev/translation-repair-lane-contest/ts';
import type { BenchSeating, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import {
  type Reseated,
  reseatHookFor,
} from './pass-reseat-hook.ts';
import {
  type JudgeSeats,
  reachableSeats,
  type SeatReadingClient,
} from '@monochromatic-dev/translation-repair-run/ts';

//region Lane contest re-seating
// THE LANE CONTEST'S PER-SLICE HOOK (ledger X12), beside the consolidation's
// (`pass-consolidate-reseat.ts`), so a test can drive it with the reading rig
// the other phases' hooks are tested with.
//
// THE CONTEST READ ITS JUDGES ONCE and had no per-slice wait: `e17d0c487`
// (2026-09-07) reached the lanes and the consolidation only. Holds began
// inside a contest 222 times in 12 of 4,122 run logs (measured 2026-09-28), and
// each left every later slice on the judges read before it. The hook now
// re-reads the judges while a hold runs, waiting it out where the bench cannot
// reach quorum, and keeps handing them over once it has ended
// (`pass-reseat-hook.ts`).

/**
 Hook the lane contest driver calls before each slice.

 @example
 ```ts
 const hooks: ContestHooks = contestHooksFor({ client, signal, l, },);
 ```
 */
export type ContestHooks = {
  readonly beforeSlice: () => Promise<BenchSeating>;
};

/**
 Seats of the late bench a wet provider would serve in one reading, which the
 contest holds to its floor before it opens a round.

 @param seats - benches as of this slice

 @returns Reachable late judges, counted

 @example
 ```ts
 const reachable = contestReachOf({ seats, },);
 ```
 */
export function contestReachOf(
  { seats, }: { readonly seats: JudgeSeats; },
): number {
  return reachableSeats({
    seats: seats.lateJudges,
    dry: seats.dry,
  },)
    .length;
}

/**
 Whether the seating last handed over states a reach below the contest's floor.

 READ AGAIN EVEN WHEN NO HOLD RUNS, since a provider whose hold ended is wet
 again and the reach handed over would otherwise stay the one read before it.

 @param seating - seating the hook last handed over

 @returns True when the contest would open no round on it

 @example
 ```ts
 const stale = reachBelowFloor({ reachable: 1, },);
 ```
 */
function reachBelowFloor(seating: BenchSeating,): boolean {
  return (seating.reachable !== undefined) && (seating.reachable < LANE_CONTEST_QUORUM);
}

/**
 What the lane contest takes from a seat reading: the late judges, as
 `pass-contest.ts` seats them.

 @param seats - benches as of this slice

 @returns Judges the slice runs on, beside the line naming them

 @example
 ```ts
 const { seating, line, } = contestSeatingOf({ seats, },);
 ```
 */
function contestSeatingOf(
  { seats, }: { readonly seats: JudgeSeats; },
): Reseated<BenchSeating> {
  /**
   Judges the slice runs on, for the line.
   */
  const judges = seats.lateJudges
    .join(',',);
  return {
    seating: {
      modelIds: seats.lateJudges,
      reachable: contestReachOf({ seats, },),
    },
    line: `slice re-seated under a hold: judges=${judges}`,
  };
}

/**
 Builds the lane contest's per-slice hook.

 @param client - run client whose dryness view and holds are the router's own

 @param signal - entry abort the readings honour

 @param l - entry logger

 @param startedReachable - reachable late judges in the reading the contest
 started on, which a slice keeps until a hold or a reach below the contest's
 floor makes the hook read again; absent where the caller did not count

 @returns Per-slice reader, handing the slice the judges it runs on

 @example
 ```ts
 const hooks = contestHooksFor({ client, signal, l, },);
 ```
 */
export function contestHooksFor(
  {
    client,
    signal,
    l,
    startedReachable,
  }: {
    readonly client: SeatReadingClient;
    readonly signal: AbortSignal;
    readonly l: Logger;
    readonly startedReachable?: number;
  },
): ContestHooks {
  return {
    beforeSlice: reseatHookFor<BenchSeating>({
      client,
      signal,
      phase: 'lane contest',
      unseated: (startedReachable === undefined) ? {} : { reachable: startedReachable, },
      seatingOf: contestSeatingOf,
      staleWhen: reachBelowFloor,
      l,
    },),
  };
}

//endregion Lane contest re-seating
