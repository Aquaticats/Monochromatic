import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type {
  CensusCarve,
  EntryCensus,
} from './slice-census-entry.ts';

//region Slice census carve
// WHICH SLICING THE SIZES DESCRIBE, said before any size is. A settled entry
// is carved through its artifact's recipe, so its slices are the ones the
// lanes judged; an entry no artifact records is carved by the deterministic
// aligner, which is a baseline the pass no longer runs, and a reader sizing a
// lane over these numbers needs to know how many rows are which.

/**
 Counts the rows whose sizes describe one carve.

 @param rows - census rows

 @param carve - carve to count

 @returns How many rows carry it

 @example
 ```ts
 const complete = countCarve({ rows, carve: 'settled-complete', },);
 ```
 */
function countCarve(
  {
    rows,
    carve,
  }: {
    readonly rows: readonly EntryCensus[];
    readonly carve: CensusCarve;
  },
): number {
  return rows
    .filter(function carries(row,): boolean {
      return row.carve === carve;
    },)
    .length;
}

/**
 The census line saying how many rows each carve describes.

 @param rows - measured entries

 @param legacyCount - entries whose settled artifact predates the recipe,
 counted by the gatherer over every entry it listed, so one of them may be an
 entry that was later found incomplete

 @returns The line, with "entry" in the singular for exactly one complete
 recipe

 @example
 ```ts
 const line = sliceCensusCarveLine({ rows, legacyCount: 0, },);
 ```
 */
export function sliceCensusCarveLine(
  {
    rows,
    legacyCount,
  }: {
    readonly rows: readonly EntryCensus[];
    readonly legacyCount: number;
  },
): string {
  /**
   Rows by which carve their sizes describe.
   */
  const carved = {
    complete: countCarve({
      rows,
      carve: 'settled-complete',
    },),
    partial: countCarve({
      rows,
      carve: 'settled-partial',
    },),
    moved: countCarve({
      rows,
      carve: 'settled-moved',
    },),
    deterministic: countCarve({
      rows,
      carve: 'deterministic',
    },),
  };
  return `CENSUS carve: ${String(carved.complete,)} settled ${
    wordForCount({
      count: carved.complete,
      one: 'entry',
      many: 'entries',
    },)
  } with a complete recipe, ${
    String(carved.partial,)
  } settled with a defaulted half, ${
    String(carved.moved,)
  } settled with a recorded block pairing that does not fit the text, carved by the deterministic aligner, ${
    String(carved.deterministic,)
  } deterministic baseline (${
    String(legacyCount,)
  } of those hold a legacy artifact)`;
}

//endregion Slice census carve
