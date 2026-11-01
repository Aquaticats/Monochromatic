import { codePointCount, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Translate alignment guard
// Whether a slice's pairing is trustworthy enough to REPLACE the archive text.
//
// The lane translates a source slice and lets judges compare the result against
// the translation already there. That question is only meaningful if the two
// sides are the same passage. When the aligner pairs a heading against a whole
// section, the judges are asked which of two unrelated texts is the better
// rendering of the heading, and they answer correctly: the rendering of the
// heading. The archive section is then replaced by a sentence.
//
// MEASURED, not supposed. On the roster bench, slice `windward0032#10` paired a
// 3-character source, "其一：", against a 226-character English passage, and the
// lane replaced the passage at every roster width tried.
//
// This guard runs AFTER the stage, never before it: the lane must translate
// every slice unconditionally, and what the judges decided is evidence worth
// recording even when the decision is refused.

/**
 Smallest incumbent worth protecting, in code points.

 Below this a mispairing costs a phrase rather than a passage, and the ratio
 alone would refuse ordinary short slices whose translation happens to run
 long.

 Measured over all 1260 two-sided slices of the pinned corpus: moving this
 floor between 64 and 256 changes how many slices the guard refuses by ONE, so
 the ratio does the work and this only keeps the guard away from short text.
 */
export const MIN_PROTECTED_INCUMBENT = 128;

/**
 Largest incumbent-to-source ratio a replacement may carry.

 Chinese becoming English expands, so a ratio around three is what a correctly
 paired slice looks like. Measured over the same 1260 slices: p50 2.95, p90
 4.10, p95 5.36, p99 23.78, max 521.9.

 SIXTEEN, from that distribution rather than from taste. It sits in the flat
 part of the tail, where the count barely moves: 36 slices refused at eight,
 20 at twelve, 16 here, 14 at twenty, 12 at twenty-four. Every one of the 16
 it refuses was inspected, and all are genuine mispairings or target-only
 content: headings paired against whole sections, and English blocks that
 transcribe an image the Chinese carries as a photo. Not one is a slice the
 lane should be allowed to replace.
 */
export const MAX_INCUMBENT_TO_SOURCE_RATIO = 16;

/**
 What the guard measured and decided for one slice.

 @example
 ```ts
 const assessment: SliceAlignmentAssessment = assessSliceAlignment({ sourceText, incumbentText, },);
 ```
 */
export type SliceAlignmentAssessment = {
  /**
   Whether the pairing supports replacing the incumbent.
   */
  readonly kind:
    | 'within-limit'
      | 'incumbent-dominates-source';

  /**
   Source code points the predicate read, whitespace-trimmed.
   */
  readonly sourceCodePoints: number;

  /**
   Incumbent code points, the same way.
   */
  readonly incumbentCodePoints: number;

  /**
   Floor this assessment applied, carried so a record stays readable after the
   constant moves.
   */
  readonly minProtectedIncumbent: number;

  /**
   Ratio limit it applied, for the same reason.
   */
  readonly maxRatio: number;
};

/**
 Measures whether a slice's two sides can be the same passage.

 @param sourceText - original slice

 @param incumbentText - translation paired with it

 @returns Verdict with the numbers behind it

 @example
 ```ts
 const assessment = assessSliceAlignment({ sourceText, incumbentText, },);
 ```
 */
export function assessSliceAlignment(
  {
    sourceText,
    incumbentText,
  }: {
    readonly sourceText: string;
    readonly incumbentText: string;
  },
): SliceAlignmentAssessment {
  /**
   Source size the predicate reads.
   */
  const sourceCodePoints = codePointCount({ text: sourceText, },);

  /**
   Incumbent size.
   */
  const incumbentCodePoints = codePointCount({ text: incumbentText, },);

  /**
   Shared measurements, whichever way the verdict goes.
   */
  const measured = {
    sourceCodePoints,
    incumbentCodePoints,
    minProtectedIncumbent: MIN_PROTECTED_INCUMBENT,
    maxRatio: MAX_INCUMBENT_TO_SOURCE_RATIO,
  };

  // A blank incumbent is the case the lane EXISTS for: there is nothing to
  // protect and everything to write, so no ratio applies.
  if (incumbentCodePoints < MIN_PROTECTED_INCUMBENT) {
    return {
      kind: 'within-limit',
      ...measured,
    };
  }

  // A blank source against a substantial translation cannot justify replacing
  // it, and the ratio is undefined there rather than large.
  if (sourceCodePoints === 0) {
    return {
      kind: 'incumbent-dominates-source',
      ...measured,
    };
  }
  if (incumbentCodePoints > (sourceCodePoints * MAX_INCUMBENT_TO_SOURCE_RATIO)) {
    return {
      kind: 'incumbent-dominates-source',
      ...measured,
    };
  }
  return {
    kind: 'within-limit',
    ...measured,
  };
}

/**
 Names a refusal in scorecard-stable wording.

 @param sliceIndex - slice refused

 @param assessment - measurements behind it

 @returns One finding line

 @example
 ```ts
 const finding = alignmentRefusalFinding({ sliceIndex, assessment, },);
 ```
 */
export function alignmentRefusalFinding(
  {
    sliceIndex,
    assessment,
  }: {
    readonly sliceIndex: number;
    readonly assessment: SliceAlignmentAssessment;
  },
): string {
  return `translate-refused-alignment (slice ${String(sliceIndex,)}: source ${
    String(assessment.sourceCodePoints,)
  } code points, incumbent ${String(assessment.incumbentCodePoints,)}, floor ${
    String(assessment.minProtectedIncumbent,)
  }, ratio limit ${String(assessment.maxRatio,)})`;
}

//endregion Translate alignment guard
