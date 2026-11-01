import {
  type SliceCostRow,
  SLICE_COST_LANES,
  type SliceCostLane
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import {
  COUNT_WIDTH,
  MS_PER_MINUTE,
} from './slice-cost-bands.ts';

//region Slice cost lanes
// WHAT EACH LANE SPENT, in the order a pass runs the lanes.

/**
 Width a lane's total minutes are padded to.
 */
const TOTAL_WIDTH = 8;

/**
 Width a lane name is padded to.
 */
const LANE_WIDTH = 13;

/**
 Where each lane stands in the order a pass runs them.

 A RECORD OVER EVERY LANE, so a lane the cost log gains cannot be left out of
 the report by a list that was never told about it: the compiler refuses the
 record until the new lane has a place in the order.
 */
const LANE_RUN_ORDER: Readonly<Record<SliceCostLane, number>> = {
  repair: 0,
  translate: 1,
  consolidation: 2,
};

/**
 Lanes a pass reports, in the order it runs them: every lane the cost log
 names, sorted by `LANE_RUN_ORDER`.
 */
const LANES: readonly SliceCostLane[] = [...SLICE_COST_LANES,].toSorted(function byRunOrder(
  left,
  right,
): number {
  return LANE_RUN_ORDER[left] - LANE_RUN_ORDER[right];
},);

/**
 Prints what each lane's priced slices spent, one line per lane that has any.

 @param rows - every parsed cost line

 @example
 ```ts
 printSliceCostLanes({ rows, },);
 ```
 */
export function printSliceCostLanes({ rows, }: { readonly rows: readonly SliceCostRow[]; },): void {
  console.log('\nBY LANE',);
  LANES.forEach(function perLane(lane,): void {
    /**
     This lane's computed slices.
     */
    const mine = rows.filter(function isMine(row,): boolean {
      return (row.lane === lane) && (row.exit === 'computed');
    },);
    if (mine.length === 0)
      return;

    /**
     What it spent.
     */
    const ms = mine.reduce(
      function addMs(
        sum,
        row,
      ): number {
        return sum + row.elapsedMs;
      },
      0,
    );
    /**
     Minutes this lane spent, rendered.
     */
    const spent = (ms / MS_PER_MINUTE).toFixed(1,);

    /**
     Slice count for this lane, rendered.
     */
    const count = String(mine.length,);

    console.log(
      `  ${lane.padEnd(LANE_WIDTH,)} slices ${count.padStart(COUNT_WIDTH,)}`
        + `  total ${spent.padStart(TOTAL_WIDTH,)} min`,
    );
  },);
}

//endregion Slice cost lanes
