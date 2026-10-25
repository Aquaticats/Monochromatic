import { MIN_STAGE_VOICES, } from '@monochromatic-dev/translation-repair-stage/ts';
import { MIN_SELECTION_BALLOTS, } from '@monochromatic-dev/translation-repair-translate-stage/ts';
import { WRITING_BENCH_FLOOR, } from './run-seats-floor.ts';
import {
  type BenchName,
  type BenchReading,
  READER_QUORUM,
} from './run-seats-wait.ts';

//region Bench floors and the lift a return gives
// TWO THRESHOLDS DECIDE WHAT A BENCH DOES, AND A RETURN IS WAITED FOR WHEN IT CROSSES EITHER. The floor is the fewest
// reachable voices at which the stages that lean on the bench decide anything; the quorum is the voices a full bench
// settles on. Between them a stage still decides, on the reachable bench and marked short.
//
// THE FLOOR IS TWO VOICES FOR EVERY BENCH, FOR A DIFFERENT REASON EACH, read from the stage code:
//
// - `wide`, `late` and `roster` serve the gathers (`stage-quorum.ts`): below `MIN_STAGE_VOICES` the quorum a
//   short bench closes on, `max(MIN_STAGE_VOICES, majority of the reachable)`, exceeds the reachable seats, so no
//   gather meets its quorum (`reachableQuorum`, `stage-reachable-quorum.ts`). `late` also serves the lane contest,
//   which opens no round under `LANE_CONTEST_QUORUM` and whose verdict tallies need two ballots on a side
//   (`lane-contest-stage.ts`); `roster` serves the preparation's pairing rounds and the admission question.
// - `select` and `slate` serve the selection rounds (`candidate-select.ts`): a leader needs `MIN_SELECTION_BALLOTS`
//   ballots whatever the weight, one judge alone never deciding, and the consolidation buys no slate for a standing
//   under that reach (`consolidate-slice-buy.ts`).
// - `editors`, `refiners` and `translators` write: under `WRITING_BENCH_FLOOR` a slate has no pair to choose between.
// - `readers` read a picture twice (`READER_QUORUM`): a single reading is `one-reader-only`.
//
// BETWEEN THE FLOOR AND THE QUORUM the stage runs on the reachable bench: a gather closes on the reduced voices with
// a `stage-short-bench` finding, a selection scales its minimum weight by reachable over quorum
// (`selectionMinimum`, `candidate-select-minimum.ts`). A return that lifts a bench over its floor changes whether the
// stages call at all; one that lifts it to its quorum changes the voices they settle on. A BENCH AT EXACTLY ITS QUORUM,
// above two voices, KEEPS THE SHORT MARK all the same: a gather closes one voice below the quorum there and carries the
// finding (`reachableQuorum`, `stage-reachable-quorum.ts:112-123`), so only a lift past the quorum drops the mark.
// Both lifts change the result, so both are waited for.

/**
 The fewest reachable voices at which the stages leaning on each bench decide anything.
 */
const BENCH_FLOORS: Readonly<Record<BenchName, number>> = {
  wide: MIN_STAGE_VOICES,
  late: MIN_STAGE_VOICES,
  roster: MIN_STAGE_VOICES,
  select: MIN_SELECTION_BALLOTS,
  slate: MIN_SELECTION_BALLOTS,
  editors: WRITING_BENCH_FLOOR,
  refiners: WRITING_BENCH_FLOOR,
  translators: WRITING_BENCH_FLOOR,
  readers: READER_QUORUM,
};

/**
 Reachable voices a bench needs before the stages leaning on it decide anything.

 @param name - bench asked about

 @returns Floor, in voices

 @example
 ```ts
 const floor = benchFloorOf({ name: 'wide', },); // 2
 ```
 */
export function benchFloorOf({ name, }: { readonly name: BenchName; },): number {
  return BENCH_FLOORS[name];
}

/**
 Which of its two thresholds a return lifts a bench across.

 @example
 ```ts
 const lift: BenchLift = { toFloor: true, toQuorum: false, };
 ```
 */
export type BenchLift = {
  /**
   Whether the bench stands under its floor now and reaches it once the providers are back.
   */
  readonly toFloor: boolean;

  /**
   Whether the bench stands under its quorum now and reaches the quorum it would have once they are back.
   */
  readonly toQuorum: boolean;
};

/**
 Reads what a return does to a bench: the thresholds it lifts the bench across.

 @param now - bench as it reads now

 @param after - same bench read with the providers back

 @returns Thresholds crossed

 @example
 ```ts
 const { toFloor, toQuorum, } = liftOf({ now, after, },);
 ```
 */
export function liftOf(
  {
    now,
    after,
  }: {
    readonly now: BenchReading;
    readonly after: BenchReading;
  },
): BenchLift {
  /**
   Voices under which the stages leaning on the bench decide nothing.
   */
  const floor = benchFloorOf({ name: now.name, },);
  /**
   Seats the bench reads now.
   */
  const { reachable: reachedNow, } = now;
  /**
   Seats the bench reads once the providers are back.
   */
  const { reachable: reachedAfter, } = after;
  /**
   Voices the bench reads now.
   */
  const heardNow = reachedNow.length;
  /**
   Voices the bench reads once the providers are back.
   */
  const heardAfter = reachedAfter.length;
  return {
    toFloor: (heardNow < floor) && (heardAfter >= floor),
    toQuorum: (heardNow < now.quorum) && (heardAfter >= after.quorum),
  };
}

//endregion Bench floors and the lift a return gives
