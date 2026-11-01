import {
  describeSpread,
  REPORTED_PERCENTILES,
} from './census-spread.ts';
import type { EntryCensus, } from './slice-census-entry.ts';
import { sliceCensusTotal, } from './slice-census-total.ts';

//region Slice census target-only
// HOW MUCH TEXT SITS IN TARGET-ONLY BLOCKS, the class where the English
// carries something the Chinese markdown does not, letters held as images
// being the known case. A translator working from the source has no source
// for it, so whether it can be detected deterministically decides whether it
// can be protected.

/**
 The census lines for the blocks only the translation has.

 @param rows - measured entries

 @returns The totals line, one line for each of the entries holding the most
 such characters (as many as there are reported percentiles), then the spread
 of the sizes of every such block

 @example
 ```ts
 const lines = sliceCensusTargetOnlyLines({ rows, },);
 ```
 */
export function sliceCensusTargetOnlyLines(
  { rows, }: { readonly rows: readonly EntryCensus[]; },
): readonly string[] {
  /**
   Entries carrying blocks only the translation has.
   */
  const targetOnly = rows
    .filter(function hasTargetOnly(row,) {
      return row.targetOnlyBlocks > 0;
    },)
    .toSorted(function byChars(
      left,
      right,
    ) {
      return right.targetOnlyChars - left.targetOnlyChars;
    },);
  return [
    `CENSUS target-only blocks: ${
      String(sliceCensusTotal({
        rows: targetOnly,
        field: 'targetOnlyBlocks',
      },),)
    }; entries: ${String(targetOnly.length,)}; chars: ${
      String(sliceCensusTotal({
        rows: targetOnly,
        field: 'targetOnlyChars',
      },),)
    }`,
    ...targetOnly
      .slice(
      0,
      REPORTED_PERCENTILES.length,
      )
      .map(function targetOnlyLine(row,): string {
        return `CENSUS   ${row.entryId}: blocks ${String(row.targetOnlyBlocks,)}, chars ${String(row.targetOnlyChars,)}`;
      },),
    describeSpread({
      label: 'CENSUS target-only block chars',
      values: rows.flatMap(function toBlockChars(row,) {
        return [...row.targetOnlyBlockChars,];
      },),
    },),
  ];
}

//endregion Slice census target-only
