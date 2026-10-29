import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { isInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { ArtifactDeliveryRow, } from './artifact-two-lane-vocabulary.ts';

//region Rebuild rows
// WHETHER A REBUILD IS THE RUN'S CARVE, read off the rows the artifact records
// (ledger A12). The recorded identity cannot answer it: it also hashes the
// identity context and the line-structure flags, which today's rules derive
// afresh, so every settled artifact tested differed from its rebuild while
// mikaela17's rows matched slice for slice. The rows are what the run carved:
// each slice's index, original, placement and archive span.

/**
 First way one recorded row differs from the rebuilt slice at its position.

 @param row - row the run recorded

 @param slice - slice the rebuild carved there

 @returns What differs, empty where nothing does

 @example
 ```ts
 const moved = rowDivergence({ row, slice, },);
 ```
 */
function rowDivergence(
  {
    row,
    slice,
  }: {
    readonly row: ArtifactDeliveryRow;
    readonly slice: ChunkPair;
  },
): string {
  /**
   Rebuilt original and archive sides.
   */
  const {
    source,
    target,
  } = slice;
  if (row.sliceIndex !== target.sliceIndex)
    return `index ${String(target.sliceIndex,)} where the run recorded ${String(row.sliceIndex,)}`;
  if (row.sourceText !== source.text)
    return 'another original';
  if ((row.incumbentKind === 'absent') !== isInsertionChunk(target,))
    return (row.incumbentKind === 'absent') ? 'archive text where the run had none' : 'no archive text where the run had some';
  if (row.incumbentText !== target.text)
    return 'another archive span';
  return '';
}

/**
 How a rebuilt carve departs from the one the artifact records.

 TAKES THE ROWS AND THE SLICES, not the artifact and the preparation they
 come from, since nothing else of either is read; the caller names which
 ledger records the run's carve.

 @param rows - rows the run recorded, one per slice it carved: the repair
 lane's delivery ledger

 @param slices - slices the rebuild carved

 @returns First departure, empty when every row matches

 @example
 ```ts
 const divergence = carveDivergence({ rows: artifact.lanes.repair.delivery, slices: prepared.slices, },);
 ```
 */
export function carveDivergence(
  {
    rows,
    slices,
  }: {
    readonly rows: readonly ArtifactDeliveryRow[];
    readonly slices: readonly ChunkPair[];
  },
): string {
  if (rows.length !== slices.length)
    return `${String(slices.length,)} ${
      wordForCount({
        count: slices.length,
        one: 'slice',
        many: 'slices',
      },)
    } rebuilt where the run recorded ${String(rows.length,)}`;
  // The two lists are one length here, so every position has a row and a slice.
  // Read as pairs and stopped at the first departure, which retires a "no
  // recorded row" arm and a fallback for a found departure's text that no
  // position could reach (ledger T8).
  for (const [position, row,] of rows.entries()) {
    /**
     How the slice rebuilt at this position departs from the row, empty where
     it matches.
     */
    const departure = rowDivergence({
      row,
      slice: nonNullishOrThrow(slices[position],),
    },);
    if (departure !== '')
      return `slice at position ${String(position,)} has ${departure}`;
  }
  return '';
}

//endregion Rebuild rows
