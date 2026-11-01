import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { isInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  buildLaneSliceTexts,
  type LaneSliceText,
  type UndecidedSlicePolicy,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import {
  assertUnheardKeptArchive,
  heardNobodyAbout,
  type RepairVoiceRecord,
} from './repair-unheard.ts';

//region Repair lane wordings
// The repair lane's settled outcomes turned into the shared per-slice wording
// shape, with each slice's outcome named rather than inferred.
//
// The mirror of `translate-lane-wordings.ts`, and it exists for the same reason
// that one does: the translation from a lane's own vocabulary into the shared
// one is not mechanical, and doing it inline let a silence pass as a choice.
//
// Here the silence is structural. This lane mends existing English, so at a
// passage the archive never translated there is nothing for it to work on: it
// logs that, records an outcome carrying the empty string, and moves on. Handed
// straight to the builder, every one of those became `decided` with an empty
// wording, and a lane comparison then reported the two lanes choosing DIFFERENT
// wordings wherever the translate lane filled the passage. The repair lane had
// no opinion there at all.
//
// THE OTHER SILENCE IS A LOST STAGE, and it once looked identical.
// Every producing voice can fail: with all six critics erroring and the
// naturalness lane silent, the lane settles the archive's own wording and
// reported that as a DECISION too. Nobody looked at that slice. It is
// `incumbent-fallback`, the archive standing by default, and the difference
// matters to every rate taken over these rows: a lane that heard nobody all
// document otherwise measures as one that examined everything and approved.

/**
 Builds the repair lane's per-slice wordings from its settled outcomes.

 @param slices - preparation the lane ran over, which supplies every incumbent
 and says which slices the archive never translated

 @param outcomes - what the lane settled, anchors included, in any order,
 each carrying who was heard about it

 @param undecided - what an unnamed gap means; `refuse` after assembly, where
 every slice was visited, and `not-evaluated` at the blocked exit, which stops
 partway through by design

 @returns One wording per prepared slice, in document order

 @throws {@link LaneSliceCoverageError} when the outcomes do not cover the
 preparation as the policy requires

 @throws {@link RepairUnheardError} when a slice nobody spoke about carries a
 wording that is not the archive's

 @example
 ```ts
 const wordings = repairLaneWordings({ slices, outcomes, undecided: 'refuse', },);
 ```
 */
export function repairLaneWordings(
  {
    slices,
    outcomes,
    undecided,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly outcomes: readonly RepairVoiceRecord[];
    readonly undecided: UndecidedSlicePolicy;
  },
): readonly LaneSliceText[] {
  /**
   Slices the archive never translated, which this lane cannot work on.
   */
  const anchored = new Set(slices
    .filter(function hasNoWording(slice,): boolean {
      return isInsertionChunk(slice.target,);
    },)
    .map(function toIndex(slice,): number {
      return slice.target
        .sliceIndex;
    },),);

  /**
   Archive's own wording per slice, which an unheard outcome has to match.
   */
  const incumbents = new Map(slices.map(function toEntry(slice,): [
    number,
    string,
  ] {
    return [
      slice.target
        .sliceIndex,
      slice.target
        .text,
    ];
  },),);

  /**
   Outcomes at a slice the archive does translate, which are the only ones
   that can be a decision or a fallback: an anchor is neither.
   */
  const mendable = outcomes.filter(function hadSomethingToRepair(outcome,): boolean {
    return !anchored.has(outcome.sliceIndex,);
  },);

  // CHECKED BEFORE CLASSIFYING, so a contradiction is refused rather than
  // recorded as whichever outcome the classification happens to pick. A slice
  // no stage spoke about cannot carry a wording, and one that does means
  // something produced text without being recorded as having produced it.
  for (const outcome of mendable) {
    /**
     Archive wording at this outcome's slice, absent where preparation made
     no such slice.
     */
    const incumbentText = incumbents.get(outcome.sliceIndex,);
    // AN INDEX PREPARATION NEVER MADE is refused by the coverage check in
    // `buildLaneSliceTexts`, which says so. Read against an empty archive
    // instead, a silent outcome carrying any text was refused as one that had
    // not kept the archive's wording, where there was no archive to keep.
    if (incumbentText === undefined)
      continue;
    assertUnheardKeptArchive({
      outcome,
      incumbentText,
    },);
  }

  // INTERSECTED WITH WHAT THE LANE REACHED, rather than taken from the
  // preparation whole. The blocked exit settles a prefix and stops, so an
  // anchor after the crossing was never reached and is `not-evaluated`; naming
  // every anchor here would report the lane as having visited slices it never
  // got to.
  return buildLaneSliceTexts({
    slices,
    undecided,
    notApplicableChunkIndices: outcomes
      .filter(function atAnAnchor(outcome,): boolean {
        return anchored.has(outcome.sliceIndex,);
      },)
      .map(function toIndex(outcome,): number {
        return outcome.sliceIndex;
      },),
    unheardChunkIndices: mendable
      .filter(function nobodySpoke(outcome,): boolean {
        return heardNobodyAbout({ outcome, },);
      },)
      .map(function toIndex(outcome,): number {
        return outcome.sliceIndex;
      },),
    decided: mendable
      .filter(function somebodySpoke(outcome,): boolean {
        return !heardNobodyAbout({ outcome, },);
      },)
      .map(function toDecision(outcome,): {
        readonly sliceIndex: number;
        readonly text: string;
      } {
        return {
          sliceIndex: outcome.sliceIndex,
          text: outcome.repairedText,
        };
      },),
  },);
}

//endregion Repair lane wordings
