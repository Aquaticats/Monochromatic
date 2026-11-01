import type { SliceDeliveryRecord, } from '../../dist/final/node/index.mjs';

//region Lane result evidence
// WHAT ONE LANE'S RAW RESULT REPORTS ABOUT ITS ROWS, projected the way
// version 2 requires, for cases that build a lane's raw JSON from the rows a
// fixture already settled on.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. Several corpus-run artifact-reading
// tests kept their own copy of this projection; all now import it from here.

/**
 What one row contributes to a lane's evidence list.

 @param row - row the lane recorded

 @returns Evidence fields version 2 requires

 @example
 ```ts
 const evidence = toEvidence(row,);
 ```
 */
export function toEvidence(row: SliceDeliveryRecord,): Record<string, unknown> {
  return {
    sliceIndex: row.sliceIndex,
    incumbentKind: row.incumbentKind,
    incumbentText: row.incumbentText,
    outcome: row.outcome,
  };
}

/**
 Raw lane result consistent with the rows.

 @param rows - rows the result reports

 @returns Evidence core the builder projects

 @example
 ```ts
 const result = rawResultFor({ rows, },);
 ```
 */
export function rawResultFor(
  { rows, }: { readonly rows: readonly SliceDeliveryRecord[]; },
): Record<string, unknown> {
  /**
   Slices the rows say shipped a replacement.
   */
  const shipped = rows
    .filter(function wasShipped(row,): boolean {
      /**
       How the lane delivered the row.
       */
      const { kind, } = row.delivery;
      return kind === 'replacement-shipped';
    },)
    .map(function indexOf(row,): number {
      return row.sliceIndex;
    },);
  return {
    sliceCount: rows.length,
    changedSliceIndices: shipped,
    withdrawnSliceIndices: [],
    changedSliceCount: shipped.length,
    withdrawnSliceCount: 0,
    sliceTexts: rows.map(toEvidence,),
  };
}

//endregion Lane result evidence
