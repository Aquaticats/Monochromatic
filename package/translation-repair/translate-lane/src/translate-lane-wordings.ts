import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  buildLaneSliceTexts,
  type LaneSliceText,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { TranslateSliceRecord, } from './translate-document-contract.ts';
import { heardNobody, } from './translate-unheard.ts';

//region Translate lane wordings
// The translate lane's per-slice records turned into the shared per-slice
// wording shape, with each slice's outcome named rather than inferred.
//
// Split out of `translate-document.ts` at its line cap, and this is the seam
// worth splitting on: everything here is a TRANSLATION of one vocabulary into
// another, reading only settled records and the passages the lane could not
// fill, while what stays behind buys slices and assembles a document.
//
// The translation is not mechanical, which is why it is worth a file of its
// own. A settled record whose stage heard NO TRANSLATOR carries the incumbent
// as its text, because that is what the document keeps; passing it through as a
// decision said the stage examined the slice and chose the archive's wording,
// which is what a lost voice looks like when nothing separates the two.

/**
 Builds the translate lane's per-slice wordings from its settled records.

 @param slices - preparation the lane ran over, which every wording is stamped
 against rather than taken from the records, since a resumed run may hold
 cache values written under an earlier preparation of the same entry

 @param settled - records the lane settled, unheard ones included

 @param unfilledChunkIndices - passages lane reached and could not fill

 @param belowFloorChunkIndices - those among them the select bench never bought a candidate for, which a name in this
 list marks only where the passage is also unfilled

 @param carriedChunkIndices - source-only passages rendered elsewhere

 @returns One wording per prepared slice, in document order

 @throws {@link LaneSliceCoverageError} when the records do not cover the
 preparation, since this lane visits every slice by contract and a gap it did
 not name is a defect

 @example
 ```ts
 const wordings = translateLaneWordings({ slices, settled, unfilledChunkIndices, },);
 ```
 */
export function translateLaneWordings(
  {
    slices,
    settled,
    unfilledChunkIndices,
    belowFloorChunkIndices = [],
    carriedChunkIndices = [],
  }: {
    readonly slices: readonly ChunkPair[];
    readonly settled: readonly TranslateSliceRecord[];
    readonly unfilledChunkIndices: readonly number[];
    readonly belowFloorChunkIndices?: readonly number[];
    readonly carriedChunkIndices?: readonly number[];
  },
): readonly LaneSliceText[] {
  /**
   Records whose stage heard at least one translator, which are the only ones
   carrying a wording anybody chose.
   */
  const heard = settled.filter(function heardSomebody(record,): boolean {
    return !heardNobody({ record, },);
  },);

  return buildLaneSliceTexts({
    slices,
    // This lane visits every slice by contract and throws rather than returning
    // a partial document, so a gap it has not named is a defect.
    undecided: 'refuse',
    unfilledChunkIndices,
    belowFloorChunkIndices,
    notApplicableChunkIndices: carriedChunkIndices,
    unheardChunkIndices: settled
      .filter(function answeredByNobody(record,): boolean {
        return heardNobody({ record, },);
      },)
      .map(function toIndex(record,): number {
        return record.sliceIndex;
      },),
    decided: heard.map(function toDecision(record,): {
      readonly sliceIndex: number;
      readonly text: string;
    } {
      return {
        sliceIndex: record.sliceIndex,
        text: record.outputText,
      };
    },),
  },);
}

//endregion Translate lane wordings
