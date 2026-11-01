import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  admitWithinShortfall,
  isInsertionChunk,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Insertion admission
// The second signature at BLOCK scale, applied once over the whole page.
//
// `doc/decision/translation-repair-absence-verdict.md` requires two independent
// readings before anything is written into a page: a roster that read both texts
// must leave the passage unplaced, AND the page must be measurably too short to
// hold it. Section scale applies this in `chunk-insertion.ts`, where the whole
// page is in hand at once. Subdivision runs per SECTION, so the block-scale half
// has to be applied here instead, after every section has been carved.
//
// ONE BUDGET FOR THE WHOLE PAGE, not one per section. A page is short by a
// definite amount, and letting each section spend that amount separately would
// admit several times what the page is actually missing. Spending it in document
// order also makes the answer reproducible: two runs over the same page admit
// the same slices.
//
// A REFUSAL HERE IS NOT A FAILURE. The slice keeps the gap the archive already
// had, which the lanes' insertion support already allows: the driver records an unfilled
// passage per slice rather than losing the entry. What this prevents is writing
// English into a page that has no room to be missing it, which usually means the
// pairing found a merge rather than an omission.

/**
 Source-only passage already rendered elsewhere in target page.

 @example
 ```ts
 const carried: CarriedInsertion = {
   position: 2,
   sliceIndex: 4,
   sourceText: '猫。',
   evidence: ['The cat.',],
 };
 ```
 */
export type CarriedInsertion = {
  /**
   Position in prepared slice order.
   */
  readonly position: number;

  /**
   Stable slice index used by lane reports.
   */
  readonly sliceIndex: number;

  /**
   Original passage roster found fully carried.
   */
  readonly sourceText: string;

  /**
   Exact target regions supporting full-coverage votes.
   */
  readonly evidence: readonly string[];
};

/**
 Source-only passage folded into the neighbouring slice whose archive span
 renders it (class one hundred ten): the lanes skip it as carried, and the
 carrier's widened source is what renders it.

 @example
 ```ts
 const folded: FoldedInsertion = { position: 1, sliceIndex: 1, carrierSliceIndex: 2, };
 ```
 */
export type FoldedInsertion = {
  /**
   Position in prepared slice order.
   */
  readonly position: number;

  /**
   Stable slice index used by lane reports.
   */
  readonly sliceIndex: number;

  /**
   Stable index of the slice whose source now covers the passage.
   */
  readonly carrierSliceIndex: number;
};

/**
 Authoritative insertion outcomes beside evidence explaining each.

 @example
 ```ts
 const admission: InsertionAdmission = { positions: new Set([1,]), findings: [], };
 ```
 */
export type InsertionAdmission = {
  /**
   Positions in prepared slice order that may be translated as insertions.
   */
  readonly positions: ReadonlySet<number>;

  /**
   Source-only passages proven fully rendered elsewhere.
   */
  readonly carried?: readonly CarriedInsertion[];

  /**
   Source-only passages folded into the slice that renders them.
   */
  readonly folded?: readonly FoldedInsertion[];

  /**
   Count-only semantic and deterministic evidence for every candidate.
   */
  readonly findings: readonly string[];
};

/**
 Chooses which insertion slices the page has room to be missing.

 @param slices - every prepared slice, in document order

 @param sourceText - whole original page

 @param targetText - whole translation as it stands

 @returns Positions in `slices` whose insertion the shortfall admits, so a
 caller can test membership without matching on chunk indices, which name
 different things depending on who stamped them

 @example
 ```ts
 const admitted = admitInsertions({ slices, sourceText, targetText, },);
 ```
 */
export function admitInsertions(
  {
    slices,
    sourceText,
    targetText,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly sourceText: string;
    readonly targetText: string;
  },
): ReadonlySet<number> {
  /**
   Positions holding a slice with no translation beside it, paired with the
   original that would be written there.
   */
  const proposed = slices.flatMap(function toProposal(
    slice,
    at,
  ) {
    return isInsertionChunk(slice.target,)
      ? [{
        position: at,
        sourceText: slice.source
          .text,
      },]
      : [];
  },);

  /**
   Positions of those the page has room for.
   */
  const admitted = admitWithinShortfall({
    sourceText,
    targetText,
    passages: proposed,
  },);

  return new Set(admitted,);
}

//endregion Insertion admission
