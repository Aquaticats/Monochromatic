import type {
  SliceCostLane,
  SliceCostRow,
} from '../../dist/final/node/index.mjs';

//region Slice cost rows
// ONE PARSED COST LINE FOR THE CASES of the slice cost report's printers.
//
// TEST SUPPORT, NOT PACKAGE SOURCE.

/**
 One slice's cost row, a priced slice of the translate lane unless said
 otherwise.

 @param sliceIndex - slice the row measures

 @param sourceChars - size of what was translated

 @param elapsedMs - time the slice took

 @param lane - lane that paid

 @param exit - how the lane left the slice

 @returns The row

 @example
 ```ts
 const row = costRow({ sliceIndex: 0, sourceChars: 40, elapsedMs: 60_000, },);
 ```
 */
export function costRow(
  {
    sliceIndex,
    sourceChars,
    elapsedMs,
    lane = 'translate',
    exit = 'computed',
  }: {
    readonly sliceIndex: number;
    readonly sourceChars: number;
    readonly elapsedMs: number;
    readonly lane?: SliceCostLane;
    readonly exit?: SliceCostRow['exit'];
  },
): SliceCostRow {
  return {
    lane,
    sliceIndex,
    sourceChars,
    elapsedMs,
    exit,
  };
}

//endregion Slice cost rows
