import type { EntryCensus, } from './slice-census-entry.ts';

//region Slice census total
// One sum the census lines share.

/**
 The counts of a row that the census adds up across rows.

 @example
 ```ts
 const field: SliceCensusCountField = 'targetOnlyChars';
 ```
 */
type SliceCensusCountField =
  | 'unpairedSourceSections'
    | 'unpairedSourceChars'
    | 'unpairedTargetSections'
    | 'unpairedTargetChars'
    | 'targetOnlyBlocks'
    | 'targetOnlyChars';

/**
 Adds one count over every row.

 @param rows - rows to add over

 @param field - the count each row holds that is added

 @returns The sum, zero for no rows

 @example
 ```ts
 const blocks = sliceCensusTotal({ rows, field: 'targetOnlyBlocks', },);
 ```
 */
export function sliceCensusTotal(
  {
    rows,
    field,
  }: {
    readonly rows: readonly EntryCensus[];
    readonly field: SliceCensusCountField;
  },
): number {
  return rows.reduce(
    function add(
      sum,
      row,
    ): number {
      return sum + row[field];
    },
    0,
  );
}

//endregion Slice census total
