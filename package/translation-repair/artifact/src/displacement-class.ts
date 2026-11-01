import {
  documentBaseline,
  MIN_RATIO_SOURCE_CHARS,
  type SliceRatio,
  sliceRatios,
  type SliceSize,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { isPlausibleSlice, } from '@monochromatic-dev/translation-repair-translate-floor/ts';

//region Displacement classification
// What a size anomaly MEANS, which the first version of this instrument
// conflated into one bucket called `movedPairs`.
//
// THE MISTAKE THAT PRODUCED THIS FILE. A screen built to find relocation
// reported 44 pairs across the corpus, and the number went into the relocation
// finding and into Question 5. Hand-checking entries whose expansion was anomalous showed the
// bucket held at least four different phenomena, three of which are not
// relocation and one of which is not evidence at all. A single count over a
// mixture cannot be a floor on any of its parts.
//
// EVERYTHING HERE IS A CANDIDATE, never a verdict. Size cannot separate a
// relocation from an adjacent omission and addition that happen to balance, nor
// an untranslated section from a deliberately terse one. These names say where
// to look and what to look for; a reader decides.

/**
 What a slice's size says about it, before any neighbour is considered.

 @example
 ```ts
 const sliceClass: SliceClass = 'untranslated';
 ```
 */
export type SliceClass =
  /**
   Long original, negligible translation. The absence verdict's subject: this is the
   positive verdict that a passage was never rendered, and a ratio of 0.01 is
   not an ambiguous one.

   THE ONE CASE THIS CLASS READS WRONG is a WHOLE-SECTION move: where a
   translator rendered an entire section inside its neighbour, the emptied
   slice looks untranslated and the neighbour looks high, and the guard that
   refuses an untranslated donor suppresses exactly that pair. So this class is
   decided from the slice alone while its own discriminator sits in the
   neighbour, which is the mistake this file made twice already.

   IT IS LEFT AS IT IS BECAUSE THE CORPUS SAYS IT COSTS NOTHING TODAY: at the
   pinned commit, ZERO untranslated slices sit beside a flagged high. Changing
   the rule on no evidence would trade a measured zero for an unmeasured guess.
   What the finding earns is a reading instruction rather than a code change:
   an untranslated slice next to a high one is a possible whole-section move
   and wants a hand-check.
   */
  | 'untranslated'
  /**
   Ordinary: a translation of an original, at whatever density. Includes the
   anomalies that need a neighbour to interpret, since `target-only` and
   relocation share one shape and are told apart on the document rather than
   on the slice.
   */
    | 'translated';

/**
 Longest original a slice can carry and still be called untranslated.

 Paired with {@link MAX_UNTRANSLATED_TARGET_CHARS} rather than used alone: a
 short original with a short translation is ordinary, and only a SUBSTANTIAL
 original left almost unrendered is evidence. `shi_Yumiaoya` carries 715, 1016
 and 1313 original characters against 14, 13 and 12 translated.
 */
const MIN_UNTRANSLATED_SOURCE_CHARS = 150;

/**
 Most translation a slice can carry and still be called untranslated.

 Set above zero because a heading survives translation even when its section
 does not, and that heading is exactly what the empty slices carry.
 */
const MAX_UNTRANSLATED_TARGET_CHARS = 60;

/**
 Longest original a slice can carry and still be called target-only.

 `Zha_Ke`'s slice 1 carries 41 original characters against 3652 translated.
 */
const MAX_TARGET_ONLY_SOURCE_CHARS = 80;

/**
 Least translation a slice must carry to be called target-only.

 A short original with a moderate translation is an ordinary heading plus
 gloss; the class is for a passage the original does not contain.
 */
const MIN_TARGET_ONLY_TARGET_CHARS = 400;

/**
 Least deficit, in translated characters, worth calling a donor.

 A LENGTH FLOOR ON THE SLICE CANNOT DO THIS JOB, which a draft of this file
 learned the hard way: `lintong`'s verified donor carries 43 original
 characters against 25 translated, so any rule that dismissed short slices
 dismissed one of the two relocations this instrument was built from. What
 makes that slice evidence is not its length but its residual, an English side
 114 characters shorter than its own original implies. A clause or two is the
 floor; below that a deficit is how one translator happened to phrase
 something.
 */
const MIN_RELOCATION_DEFICIT = 60;

/**
 How far above the baseline a slice must sit to be a relocation candidate.
 */
const HIGH_FACTOR = 2;

/**
 Least surplus, in translated characters, worth calling a relocation.

 Below a sentence or so, a surplus is ordinary variation in how a translator
 renders a phrase. `Dethelly/0` carries a surplus of 290 and `lintong/3` one
 of 261.
 */
const MIN_RELOCATION_SURPLUS = 120;

/**
 How much of a surplus the neighbour's deficit must account for.

 MEASURED ON BOTH VERIFIED RELOCATIONS, against the baseline this file computes
 rather than an earlier draft's: `Dethelly` runs a surplus of 297 against a
 deficit of 121, and `lintong` 281 against 99, which are ratios of 0.41 and
 0.35. The deficit is consistently the SMALLER side, because the slice that
 gave text up still renders its own original while the slice that took it on
 carries the expanded English of both. A symmetric "similar magnitudes" test,
 which is what conservation suggests and what a reviewer proposed, would reject
 both cases this instrument was built from.

 A QUARTER RATHER THAN A THIRD, and the margin is thinner than it looks. Both
 candidates verified by hand as TRANSCRIPTIONS rather than moves sit at 0.28,
 and the two verified relocations at 0.35 and 0.41, so the band just above this
 floor is where the false ones have so far been found. That is five data points
 and not grounds to retune; it is grounds to hand-check a near-floor candidate
 rather than count it, which is what the window trial is told to do.
 */
const MIN_CONSERVED_FRACTION = (1 / 2) * (1 / 2);

/**
 One slice, measured and classified.

 @example
 ```ts
 const slice: ClassifiedSlice = { slicePosition: 0, sourceChars: 35, targetChars: 403, ratio: 11.51, residual: 290, sliceClass: 'translated', };
 ```
 */
export type ClassifiedSlice = SliceRatio & {
  /**
   Translated characters beyond what the baseline explains, negative where the
   slice carries less than its original implies.
   */
  readonly residual: number;

  /**
   What this slice's size says about it on its own.
   */
  readonly sliceClass: SliceClass;
};

/**
 A slice that took text on beside one that gave text up.

 @example
 ```ts
 const candidate: RelocationCandidate = { high: 0, low: 1, surplus: 290, deficit: 149, };
 ```
 */
export type RelocationCandidate = {
  /**
   Slice carrying more translation than its original explains.
   */
  readonly high: number;

  /**
   Neighbour carrying less, which is the other end of the same passage.
   */
  readonly low: number;

  /**
   Translated characters the high slice carries beyond its original.
   */
  readonly surplus: number;

  /**
   Translated characters the neighbour is short by.
   */
  readonly deficit: number;
};

/**
 One document's reading, with each class kept apart.

 @example
 ```ts
 const reading: DocumentDisplacement = classifyDisplacement({ slices, },);
 ```
 */
export type DocumentDisplacement = {
  /**
   Expansion every residual was computed against.
   */
  readonly baseline: number;

  /**
   Whether that expansion came from this document or from the corpus.
   */
  readonly baselineFrom: 'document' | 'corpus-reference';

  /**
   Every slice, measured and classified, in slice order.
   */
  readonly slices: readonly ClassifiedSlice[];

  /**
   Slices whose original was left essentially unrendered.
   */
  readonly untranslated: readonly number[];

  /**
   Slices carrying translation the original does not account for.
   */
  readonly targetOnly: readonly number[];

  /**
   High slices whose neighbour gave up enough to account for them.
   */
  readonly relocationCandidates: readonly RelocationCandidate[];

  /**
   High slices with no neighbour that gave anything up, which are anomalies
   without a second end and so are not relocation on this evidence.
   */
  readonly otherImbalances: readonly number[];
};

/**
 Classifies one slice on its own size, before any neighbour is read.

 ONLY THE ONE CLASS A NEIGHBOUR CANNOT OVERTURN IS DECIDED HERE. A section with
 a long original and no translation is untranslated whatever sits beside it.
 Everything else waits, because `target-only` and relocation share one shape:
 a tiny original against a long translation describes `Zha_Ke`'s English-only
 letter AND `Dethelly/0`'s relocation, and size alone cannot tell them apart.
 A first draft of this file did claim `target-only` here, and it silently
 reclassified the one case this whole instrument was built from.

 @param reading - one slice's measured ratio

 @returns What that slice's size says about it on its own

 @example
 ```ts
 const sliceClass = classifySlice({ reading, },);
 ```
 */
function classifySlice({ reading, }: { readonly reading: SliceRatio; },): SliceClass {
  if (reading.sourceChars < MIN_UNTRANSLATED_SOURCE_CHARS)
    return 'translated';
  if (reading.targetChars > MAX_UNTRANSLATED_TARGET_CHARS)
    return 'translated';
  return 'untranslated';
}

/**
 Slice positions carrying one class.

 @param classified - every slice, already classified

 @param sliceClass - class to collect

 @returns Positions in slice order

 @example
 ```ts
 const untranslated = indicesOf({ classified, sliceClass: 'untranslated', },);
 ```
 */
function indicesOf(
  {
    classified,
    sliceClass,
  }: {
    readonly classified: readonly ClassifiedSlice[];
    readonly sliceClass: SliceClass;
  },
): readonly number[] {
  return classified
    .filter(function isWanted(slice,) {
      return slice.sliceClass === sliceClass;
    },)
    .map(function toIndex(slice,) {
      return slice.slicePosition;
    },);
}

/**
 Reads one document's slice sizes and says what each anomaly looks like.

 @param slices - prepared slice pairs, each with both sides' character counts

 @returns Baseline used, every slice classified, and each anomaly class apart

 @example
 ```ts
 const reading = classifyDisplacement({ slices, },);
 ```
 */
export function classifyDisplacement(
  { slices, }: { readonly slices: readonly SliceSize[]; },
): DocumentDisplacement {
  /**
   Every slice's ratio, none dropped.
   */
  const readings = sliceRatios({ slices, },);

  /**
   What each slice's own size says about it.
   */
  const classes = readings.map(function toClass(reading,) {
    return classifySlice({ reading, },);
  },);

  /**
   Expansion read from slices that are plausibly translations, long enough on
   the original side for their ratio to mean something, and not themselves
   implausible.

   ALL THREE CONDITIONS ARE LOAD-BEARING, and they were measured separately
   over the 89 documents that offer a baseline at all. Dropping short originals
   moves 62 of them, which is the largest of the three: it stops one slice with
   a 41-character original and 3652 translated characters from counting as an
   ordinary reading, which is `Zha_Ke`, whose document-wide aggregate is 16.85
   for that reason alone. Dropping the implausible slices moves 15 and flips 2
   onto the corpus reference. Dropping untranslated sections moves 2, and it
   stays because its case is definitional rather than statistical: a section
   nobody rendered is not a translation, so it has no standing to say what a
   translation's density is, whatever the number would have done.

   THE IMPLAUSIBILITY FILTER IS NOT CIRCULAR, which is the thing to check
   before believing any of this. Every predicate behind it reads fixed
   endpoints and never the baseline, so nothing it removes was chosen by the
   quantity it goes on to compute. A filter that read the baseline would be
   defining the centre in terms of itself.

   EVERY EXCLUDED SLICE IS STILL CLASSIFIED. They just do not get to say what
   normal is.
   */
  const baseline = documentBaseline({
    slices: readings.filter(function setsTheBar(
      reading,
      slicePosition,
    ) {
      if (classes[slicePosition] !== 'translated')
        return false;
      if (reading.sourceChars < MIN_RATIO_SOURCE_CHARS)
        return false;
      return isPlausibleSlice({ slice: reading, },);
    },),
  },);

  /**
   Every slice with what the baseline leaves unexplained.
   */
  const classified: readonly ClassifiedSlice[] = readings
    .map(function toClassified(
      reading,
      slicePosition,
    ): ClassifiedSlice {
      return {
        ...reading,
        residual: reading.targetChars - (baseline.expansion * reading.sourceChars),
        sliceClass: nonNullishOrThrow(classes[slicePosition],),
      };
    },);

  /**
   Slices carrying more translation than their original explains, by enough to
   be a passage rather than a phrase.
   */
  const highIndices = classified
    .filter(function isHigh(slice,) {
      if (slice.sliceClass !== 'translated')
        return false;
      if (slice.residual < MIN_RELOCATION_SURPLUS)
        return false;
      return slice.ratio >= (baseline.expansion * HIGH_FACTOR);
    },)
    .map(function toIndex(slice,) {
      return slice.slicePosition;
    },);

  /**
   Every high slice paired with whichever neighbours gave enough up.
   */
  const relocationCandidates = highIndices.flatMap(function toCandidates(high,) {
    /**
     Surplus this slice carries, known present by `isHigh`.
     */
    const surplus = nonNullishOrThrow(classified[high],)
      .residual;
    return [
      high - 1,
      high + 1,
    ]
      .flatMap(function toCandidate(low,): readonly RelocationCandidate[] {
        /**
         That neighbour, absent at either end of the document.
         */
        const beside = classified[low];
        if (beside === undefined)
          return [];
        // A NEIGHBOUR THAT IS ITSELF UNTRANSLATED IS NOT A DONOR. It is short
        // because nobody rendered it, which is the absence verdict's finding rather than
        // this one, and pairing with it would report every untranslated section
        // as a relocation.
        if (beside.sliceClass !== 'translated')
          return [];

        /**
         Translated characters this neighbour is short by.
         */
        const deficit = -beside.residual;
        if (deficit < MIN_RELOCATION_DEFICIT)
          return [];
        if (deficit < (surplus * MIN_CONSERVED_FRACTION))
          return [];
        return [{
          high,
          low,
          surplus: Math.round(surplus,),
          deficit: Math.round(deficit,),
        },];
      },);
  },);

  /**
   High slices no neighbour accounts for, which are surpluses with one end.
   */
  const unpaired = highIndices.filter(function noDonor(high,) {
    return !relocationCandidates.some(function names(candidate,) {
      return candidate.high === high;
    },);
  },);

  /**
   Unpaired surpluses whose original is far too small to have carried them,
   which is content the source side does not contain at all.

   DECIDED HERE RATHER THAN ON THE SLICE, because this shape and a relocation's
   are the same shape. `Zha_Ke`'s slice 1 and `Dethelly`'s slice 0 both carry a
   tiny original against a long translation; what separates them is that
   Dethelly's neighbour gave up text to account for it and Zha_Ke's did not.
   */
  const targetOnly = unpaired.filter(function sourceCannotExplain(high,) {
    /**
     That slice, known present since the index came from the readings.
     */
    const slice = nonNullishOrThrow(classified[high],);
    if (slice.sourceChars > MAX_TARGET_ONLY_SOURCE_CHARS)
      return false;
    return slice.targetChars >= MIN_TARGET_ONLY_TARGET_CHARS;
  },);
  return {
    baseline: baseline.expansion,
    baselineFrom: baseline.from,
    slices: classified,
    untranslated: indicesOf({
      classified,
      sliceClass: 'untranslated',
    },),
    targetOnly,
    relocationCandidates,
    otherImbalances: unpaired.filter(function notTargetOnly(high,) {
      return !targetOnly.includes(high,);
    },),
  };
}

//endregion Displacement classification
