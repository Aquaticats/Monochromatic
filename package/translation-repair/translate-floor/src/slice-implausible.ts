import type { SliceSize, } from '@monochromatic-dev/translation-repair-corpus-source/ts';

//region Slice implausibility
// Whether one slice pair's sizes are outside what any translation of that
// original plausibly produces, and on which evidence.
//
// ABSOLUTE, DELIBERATELY, and that is the whole reason this is a separate
// predicate rather than a threshold on the document's own expansion. Every
// endpoint here is fixed, so nothing it names was decided by the baseline. That
// is what makes it sound to REMOVE what it names from the set that sets the
// baseline: a relative predicate used the same way would define the centre in
// terms of itself.
//
// WHAT THEY ARE MADE OF, measured over the pinned commit's 92 pairs under
// deterministic pairing. Of the 36 eligible slices tripping anything, block
// disparity is the SOLE cause for 20; the ratio tails are 16 slices across 14
// entries. The two behave differently under production pairing, which is why
// they are reported as separate reasons rather than one boolean: a roster-paired
// sweep of 29 entries took 64 flags down to 40 and never added one, and block
// disparity is what a re-pairing moves.
//
// THE TAILS AND THE GAP THEREFORE ANSWER DIFFERENT QUESTIONS. A ratio tail says
// this translation is the wrong size for this original. A block gap says the
// PAIRING is not to be trusted here, which makes the ratio meaningless rather
// than extreme. Both disqualify a slice from setting a baseline; only the tails
// describe a fault in the rendering itself.

/**
 Lowest translated-to-original ratio a real translation still reaches.

 Chinese into English expands, so a slice below this is not dense prose: it is
 a section left mostly untranslated, or one whose content moved elsewhere.
 */
const IMPLAUSIBLE_MIN_RATIO = 0.8;

/**
 Highest translated-to-original ratio a real translation still reaches.

 Above this the translated side carries text the original cannot account for,
 which is content relocated into this slice or added outright.
 */
const IMPLAUSIBLE_MAX_RATIO = 10;

/**
 Largest block-count difference a trustworthy pairing leaves behind.

 One block of slack absorbs an ordinary paragraph split. Beyond that the two
 sides are not the same passage, so their ratio measures the pairing rather
 than the translation.
 */
const MAX_BLOCK_COUNT_GAP = 1;

/**
 Evidence on which a slice's sizes are called implausible.

 NAMED FOR WHAT WAS OBSERVED rather than for what it implies about whoever
 wrote the text, because these names reach a judge's prompt and a name that
 characterizes the author biases the reading it is meant to inform.
 */
export type SliceImplausibility
  = 'target-far-shorter'
    | 'target-far-longer'
    | 'block-count-gap';

/**
 Evidence that a rendering is the wrong size for its original, read off
 character counts alone.
 */
export type RatioImplausibility = Exclude<SliceImplausibility, 'block-count-gap'>;

/**
 Reads one slice's character counts for a ratio no translation of its
 original plausibly produces.

 A SIDE WITH NO CHARACTERS RAISES NOTHING, for the reason
 {@link sliceImplausibility} gives.

 SEPARATE FROM THE BLOCK GAP so a caller asking only whether the rendering is
 the wrong size, as the contest size note does, counts no blocks and so
 parses nothing (ledger B68).

 @param sourceChars - original's characters

 @param targetChars - rendering's characters

 @returns Every ratio tail the counts reach, empty when they are ordinary

 @example
 ```ts
 const tails = ratioImplausibility({ sourceChars: 56, targetChars: 10_381, },); // ['target-far-longer']
 ```
 */
export function ratioImplausibility(
  {
    sourceChars,
    targetChars,
  }: {
    readonly sourceChars: number;
    readonly targetChars: number;
  },
): readonly RatioImplausibility[] {
  if ((sourceChars === 0) || (targetChars === 0))
    return [];

  /**
   Translated characters per original character.
   */
  const ratio = targetChars / sourceChars;

  return [
    ((ratio < IMPLAUSIBLE_MIN_RATIO) ? 'target-far-shorter' as const : undefined),
    ((ratio > IMPLAUSIBLE_MAX_RATIO) ? 'target-far-longer' as const : undefined),
  ].filter(function raised(
    reason,
  ): reason is RatioImplausibility {
    return reason !== undefined;
  },);
}

/**
 Reads one slice's sizes for every way they fail to be plausible.

 A SLICE WITH AN EMPTY SIDE RAISES NOTHING. No original means no ratio, and no
 translation means the section was never rendered, which is a different
 phenomenon that the displacement classifier already names. Reporting it here
 as well would double-count it and pull the whole class into a rendering fault
 it is not.

 @param slice - sizes of one paired slice

 @returns Every reason its sizes are implausible, empty when they are ordinary

 @example
 ```ts
 const reasons = sliceImplausibility({ slice: { sourceChars: 129, targetChars: 268, sourceBlocks: 2, targetBlocks: 2, }, },);
 ```
 */
export function sliceImplausibility(
  { slice, }: { readonly slice: SliceSize; },
): readonly SliceImplausibility[] {
  if ((slice.sourceChars === 0) || (slice.targetChars === 0))
    return [];

  /**
   How far the two sides disagree about how many blocks they hold.
   */
  const blockGap = Math.abs(slice.sourceBlocks - slice.targetBlocks,);

  return [
    ...ratioImplausibility({
      sourceChars: slice.sourceChars,
      targetChars: slice.targetChars,
    },),
    ...((blockGap > MAX_BLOCK_COUNT_GAP) ? ['block-count-gap' as const,] : []),
  ];
}

/**
 Whether a slice's sizes are ordinary enough for it to say what normal is.

 SEPARATE FROM READING THE REASONS, because the baseline cares only that a
 slice is clean while a judge needs to be told which evidence was seen.

 @param slice - sizes of one paired slice

 @returns True when no implausibility was raised

 @example
 ```ts
 const eligible = slices.filter(function clean(slice,) { return isPlausibleSlice({ slice, },); },);
 ```
 */
export function isPlausibleSlice(
  { slice, }: { readonly slice: SliceSize; },
): boolean {
  /**
   Every reason this slice's sizes are implausible, which the baseline needs
   only the emptiness of.
   */
  const reasons = sliceImplausibility({ slice, },);

  return reasons.length === 0;
}

//endregion Slice implausibility
