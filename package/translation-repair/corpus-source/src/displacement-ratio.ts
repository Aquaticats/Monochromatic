import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import { readSliceSkeleton, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Displacement ratio
// Size primitives the displacement classifier reads: per-slice ratios, a
// document's own expansion, and the corpus expansion to fall back on.
//
// WHY SIZE IS WORTH READING AT ALL. Every judge in the translate lane sees one
// slice pair and nothing else. Where a translator carried a passage into a
// neighbouring section, the archive looks like it invented content on one slice
// and dropped content on the next, and a per-slice roster condemns it at both
// ends while preferring a fresh rendering that says only what its own slice's
// original says. Found on `Dethelly/0` by the judge-quality bench's alteration arm and confirmed
// by hand: the English `Description` carries four sentences whose Chinese sits
// in the NEXT slice.
//
// THE CENTRE HAS BEEN A MEDIAN, THEN A POOLED AGGREGATE, AND IS A MEDIAN AGAIN.
// All three readings are recorded here because the second one was right about
// the case that beat the first, and wrong to conclude the statistic was the
// problem.
//
// The FIRST version took a median over EVERY slice, and it was contaminated by
// the thing it exists to detect: `shi_Yumiaoya` carries three untranslated
// sections whose near-zero ratios pulled its median to 0.76, which dropped the
// threshold to 1.51, which then flagged two perfectly ordinary translations.
//
// The SECOND version replaced it with a pooled aggregate over slices believed
// translated. That fixed `shi_Yumiaoya`, but the fix was the ELIGIBILITY FILTER,
// not the pooling, and nothing checked which of the two had done the work.
// Pooling brought its own exposure: a pooled ratio is decided by the longest
// slices, so one long contaminated slice moves it while a hundred short clean
// ones do not.
//
// The THIRD version measured them against each other over the 89
// documents that offer a baseline. Removing the implausible slices moves a
// pooled centre in 7 more documents than it moves a median, and flips 7 onto the
// corpus reference where a median flips 2. `shi_Yumiaoya` was re-read under both
// and falls back to the corpus reference under each, because the eligibility
// filter removes its untranslated sections before either estimator sees them.
// So the median returned, over a filtered set rather than every slice, and the
// filter that made it safe is the one the second version had already added.
//
// THE AGGREGATE IS NOT FULLY INVARIANT UNDER RELOCATION EITHER, and an earlier
// version of this comment claimed it was. A total over ALL slices would be:
// moving text between slices leaves both document totals alone. The aggregate
// this file computes is taken over ELIGIBLE slices, and relocation can move text
// across that boundary. `Dethelly` is the case: its 35-character recipient is
// excluded by the length floor while its 129-character donor is included, so the
// baseline is lower than it would otherwise be. Measured, the ratio of deficit to
// surplus reads 0.41 under this baseline, 0.44 leaving the pair out, and 0.51
// over all slices. THE CONCLUSION SURVIVES ALL THREE, since the deficit is the
// smaller side under every one, but the estimator is endogenous and the honest
// fix is a baseline computed per adjacency with that pair excluded. That fix is
// recorded here with those measurements rather than made at the same time as
// the numbers that depend on it.

/**
 Expansion the corpus works at, used when a document cannot speak for itself.

 MEASURED, not assumed. Over the pinned commit's 92 complete pairs, the 91 that
 carry any original text at all give a median per-document aggregate of 2.86,
 with a p10 to p90 span of 1.95 to 3.47. An earlier draft of this instrument
 said "roughly threefold" from recall; the measurement moved it.

 NINETY-ONE RATHER THAN NINETY-TWO because `XIEPT2` has both files and no
 source text, so it can carry no aggregate. Elsewhere this instrument counts
 92, which is complete PAIRS; the two numbers measure different things.
 */
export const CORPUS_REFERENCE_EXPANSION = 2.86;

/**
 Lowest document aggregate still worth believing as a baseline.

 Below this a document is not translating at a plausible density: it is
 partly untranslated, or mostly markup, or carries long verbatim blocks. Set
 just under the measured p10 of 1.95 so an ordinary low-density document keeps
 its own baseline and a contaminated one does not.
 */
export const PLAUSIBLE_BASELINE_MIN = 1.9;

/**
 Highest document aggregate still worth believing as a baseline.

 Above this the translation carries substantially more than the original says,
 which is a document-scale version of the source-absent class rather than a
 density. Measured p90 is 3.47, so this leaves generous room before refusing.
 */
export const PLAUSIBLE_BASELINE_MAX = 4.5;

/**
 Shortest original worth a ratio.

 RAISED FROM TWENTY, which was far too low: `noname3031`'s flagged slice was
 TWENTY-THREE original characters, one over the old floor, and its ratio was
 arithmetic rather than evidence. At eighty a slice has to carry a sentence or
 two before its ratio is read at all.
 */
export const MIN_RATIO_SOURCE_CHARS = 80;

/**
 One slice's size reading.

 CARRIES BLOCK COUNTS AS WELL AS CHARACTERS, because a slice whose two sides
 disagree about how many blocks they hold is one whose PAIRING is in doubt,
 and a ratio taken across a doubtful pairing measures the pairing rather than
 the translation. Excluding those from the baseline moved 8 of 89 documents
 and flipped one onto the corpus reference, which is why the counts are
 required rather than optional: a caller that omitted them would silently get
 a different estimator from the measured one.

 @example
 ```ts
 const size: SliceSize = { sourceChars: 129, targetChars: 268, sourceBlocks: 2, targetBlocks: 2, };
 ```
 */
export type SliceSize = {
  /**
   Characters of original.
   */
  readonly sourceChars: number;

  /**
   Characters of translation.
   */
  readonly targetChars: number;

  /**
   Top-level blocks the parse reads on the original side.
   */
  readonly sourceBlocks: number;

  /**
   Top-level blocks the parse reads on the translated side.
   */
  readonly targetBlocks: number;
};

/**
 Signals that a side of a slice refused the shared slice grammar, so its
 blocks cannot be counted off the parse.

 THROWN RATHER THAN GUESSED AT (ledger B68): a block count read some other
 way would be a second estimator reported as this one. Only the displacement
 and window-trial probes count blocks; the contest size note reads character
 counts alone, so no live gate meets this refusal.

 @example
 ```ts
 throw new SliceBlockCountRefusalError({ detail: 'unexpected `{`', },);
 ```
 */
export class SliceBlockCountRefusalError extends Error {
  /**
   Declares this message safe to forward: it repeats only the parser's own
   account of where the grammar stopped.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds failure naming the parser's own account.

   @param detail - grammar's refusal, as `readSliceSkeleton` reports it

   @example
   ```ts
   new SliceBlockCountRefusalError({ detail: 'unexpected `{`', },);
   ```
   */
  constructor({ detail, }: { readonly detail: string; },) {
    super(
      `slice block count could not be read off the parse: ${detail}`,
    );
    this.name = 'SliceBlockCountRefusalError';
  }
}

/**
 Counts the top-level blocks the shared slice grammar reads, the same floor
 `readSliceSkeleton` and every other block-aware reader in this package
 measure against (ledger B68). A fenced block carrying a blank line between
 its own lines is one block here, as it is to the parse, where a blank-line
 split would have read it as several.

 @param text - one side of a slice

 @returns How many top-level blocks the parse reads

 @throws {@link SliceBlockCountRefusalError} when the shared slice grammar refuses this side

 @example
 ```ts
 const blocks = contentBlockCount({ text: slice.source.text, },);
 ```
 */
function contentBlockCount(
  { text, }: { readonly text: string; },
): number {
  /**
   The shared slice grammar's reading of this side, or its refusal.
   */
  const read = readSliceSkeleton({ text, },);
  if (read.kind === 'unparseable')
    throw new SliceBlockCountRefusalError({ detail: read.detail, },);
  return read.skeleton
    .blocks
    .length;
}

/**
 Reads both sides of one paired slice into the sizes this instrument wants.

 THE ONE PLACE BLOCKS ARE COUNTED, so that every caller feeding the classifier
 counts them the same way. Two callers splitting blocks slightly differently
 would be running two different estimators while reporting one number, and the
 difference would be invisible in the output.

 A BLOCK IS A TOP-LEVEL NODE OF THE SHARED SLICE GRAMMAR (ledger B68), matching
 how `readSliceSkeleton` and the line-structure reader both count, because a
 blank-line split and the parse disagree about a fenced block carrying a
 blank line between its own lines, or a list tight on one side and loose on
 the other, and this instrument's PAIRING decision has to agree with every
 other reader built on the same parse.

 @param sourceText - original side of this slice

 @param targetText - translated side of this slice

 @returns Characters and blocks on both sides

 @throws {@link SliceBlockCountRefusalError} when the shared slice grammar refuses either side

 @example
 ```ts
 const size = sliceSizeOf({ sourceText: slice.source.text, targetText: slice.target.text, },);
 ```
 */
export function sliceSizeOf(
  {
    sourceText,
    targetText,
  }: {
    readonly sourceText: string;
    readonly targetText: string;
  },
): SliceSize {
  return {
    sourceChars: sourceText.length,
    targetChars: targetText.length,
    sourceBlocks: contentBlockCount({ text: sourceText, },),
    targetBlocks: contentBlockCount({ text: targetText, },),
  };
}

/**
 Sizes of every slice of a preparation, in slice order.

 The displacement probe and the window trial each mapped their slices
 through `sliceSizeOf` with their own copy of this (audit area six,
 2026-09-28).

 @param slices - prepared slices, each with its two texts

 @returns Each slice's size

 @example
 ```ts
 const sizes = sliceSizesOf({ slices: prepared.slices, },);
 ```
 */
export function sliceSizesOf(
  { slices, }: {
    readonly slices: readonly {
      readonly source: { readonly text: string; };
      readonly target: { readonly text: string; };
    }[];
  },
): readonly SliceSize[] {
  return slices.map(function toSizes(slice,): SliceSize {
    return sliceSizeOf({
      sourceText: slice.source
        .text,
      targetText: slice.target
        .text,
    },);
  },);
}

/**
 One slice's size reading with the ratio it implies.

 @example
 ```ts
 const reading: SliceRatio = { slicePosition: 0, sourceChars: 35, targetChars: 403, sourceBlocks: 1, targetBlocks: 1, ratio: 11.51, };
 ```
 */
export type SliceRatio = SliceSize & {
  /**
   Slice this describes.
   */
  readonly slicePosition: number;

  /**
   Translation characters per original character.
   */
  readonly ratio: number;
};

/**
 Reads each slice's ratio, in slice order and without dropping any.

 NOTHING IS FILTERED HERE. The old version discarded every slice under a
 source-character floor before anything else ran, which threw away the
 strongest evidence there is: `Zha_Ke`'s slice 1 carries 41 original
 characters against 3652 translated, a ratio of 89, and a floor on the
 ORIGINAL side deleted it. Classification decides what a short slice means;
 this function only measures.

 @param slices - prepared slice pairs, each with both sides' character counts

 @returns Ratio per slice, one entry per input in the same order

 @example
 ```ts
 const readings = sliceRatios({ slices, },);
 ```
 */
export function sliceRatios(
  { slices, }: { readonly slices: readonly SliceSize[]; },
): readonly SliceRatio[] {
  return slices.map(function toRatio(
    slice,
    slicePosition,
  ): SliceRatio {
    // A SOURCE OF ZERO IS A REAL STATE, not an input to sanitize: a slice can be
    // an insertion anchor with no original at all. Its ratio is the translated
    // length itself, which classification reads as target-only rather than as a
    // density.
    if (slice.sourceChars <= 0) {
      return {
        slicePosition,
        sourceChars: slice.sourceChars,
        targetChars: slice.targetChars,
        sourceBlocks: slice.sourceBlocks,
        targetBlocks: slice.targetBlocks,
        ratio: slice.targetChars,
      };
    }
    return {
      slicePosition,
      sourceChars: slice.sourceChars,
      targetChars: slice.targetChars,
      sourceBlocks: slice.sourceBlocks,
      targetBlocks: slice.targetBlocks,
      ratio: slice.targetChars / slice.sourceChars,
    };
  },);
}

/**
 Expansion to read a document's slices against.

 THE MEDIAN OF PER-SLICE RATIOS RATHER THAN A POOLED AGGREGATE, reversing what
 this file did until the two were measured against each other. A pooled
 ratio is decided by the longest slices, so a single contaminated long slice
 moves it; every slice counts once here. Measured over the pinned commit's 89
 documents that offer a baseline at all, removing the implausible slices moves
 a pooled centre in 7 more documents than it moves this one, and flips 2 onto
 the corpus reference where the pooled centre flips 7. Half the sensitivity to
 the contamination it exists to survive.

 A MEDIAN WAS TRIED FIRST AND REJECTED, and the region comment still records
 why: `shi_Yumiaoya`'s untranslated sections pulled a median to 0.76. That
 median was taken over EVERY slice. This one is taken over what the caller
 passes, and the caller now passes neither the untranslated sections nor the
 implausible ones, so the case that refuted it no longer reaches it: measured,
 that document falls back to the corpus reference under both estimators.

 NO MINIMUM SLICE COUNT, which was proposed and then refuted. Split-half
 stability was read at every count, and a document's own centre beat the
 corpus reference at all of them, including one.

 IT IS NOT INVARIANT UNDER RELOCATION, and neither was the aggregate it
 replaces. Moving text between two slices changes both their ratios, so it
 moves this centre whenever it moves either of them across the middle of the
 order. The region comment gives the measured size of that effect for the
 aggregate and the fix it would take; the median inherits the question.

 @param slices - slices believed to be translated

 @returns Document's own expansion when believable, and the corpus reference
 otherwise, with which one was used

 @example
 ```ts
 const baseline = documentBaseline({ slices: translated, },);
 ```
 */
export function documentBaseline(
  { slices, }: { readonly slices: readonly SliceSize[]; },
): {
  readonly expansion: number;
  readonly from: 'document' | 'corpus-reference';
} {
  /**
   Every offered slice's own ratio, smallest first.

   A slice with no original is dropped rather than floored, since it carries
   no ratio to rank: keeping it at some stand-in value would let the number of
   untranslatable slices decide where the middle of the order falls.
   */
  const ratios = slices
    .filter(function carriesARatio(
      slice,
    ): boolean {
      return slice.sourceChars > 0;
    },)
    .map(function toRatio(
      slice,
    ): number {
      return slice.targetChars / slice.sourceChars;
    },)
    .toSorted(function bySize(
      left,
      right,
    ): number {
      return left - right;
    },);

  /**
   Corpus expansion, named once so every refusal path reads alike.
   */
  const fallback = {
    expansion: CORPUS_REFERENCE_EXPANSION,
    from: 'corpus-reference',
  } as const;
  if (ratios.length === 0)
    return fallback;

  /**
   Where the middle of the order sits.
   */
  const middle = Math.floor(ratios.length / 2,);

  /**
   This document's own expansion, taken as the middle ratio.

   An even count averages the two middle ratios rather than taking either,
   so that adding one slice cannot move the centre further than the slices
   around it sit apart.
   */
  const expansion = ((ratios.length % 2) === 1)
    ? nonNullishOrThrow(ratios[middle],)
    : ((nonNullishOrThrow(ratios[middle - 1],) + nonNullishOrThrow(ratios[middle],)) / 2);

  if (expansion < PLAUSIBLE_BASELINE_MIN)
    return fallback;
  if (expansion > PLAUSIBLE_BASELINE_MAX)
    return fallback;
  return {
    expansion,
    from: 'document',
  };
}

//endregion Displacement ratio
