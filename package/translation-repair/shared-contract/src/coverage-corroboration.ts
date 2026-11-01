import { codePointCount, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Coverage corroboration
// The second signature an insertion needs, which consults no model.
//
// `doc/decision/translation-repair-absence-verdict.md` requires TWO independent
// readings before anything is written into a page: the coverage roster must
// call the passage absent, AND the page must be measurably too short to hold
// it. A false insertion duplicates text in a memorial page, which is the
// expensive error; a missed one leaves a gap the archive already had.
//
// WHY A SIZE TEST SEPARATES THE CASE THE ROSTER CANNOT. The ambiguity the whole
// question turns on is a MERGE against an OMISSION: an aligner reports both as
// an unpaired source passage. A merge leaves the content somewhere in a page of
// ordinary length, so a merged page is not short. An omission leaves the page
// short by about what the passage would have contributed. The size test is
// blind to which passage is which and answers only "is this page missing
// roughly this much English", which is exactly the half the roster is weakest on.
//
// MEASURED, not tuned. See the decision document: over all 92 pinned pairs the
// ratio of English code points to source code points reads p5 1.42, p25 2.28,
// p50 2.65, p75 3.00, p95 4.52. The three entries where the roster refused to
// call any candidate covered are short of a median expectation by 8970, 676 and
// 10891 code points; the three where it called passages carried and noticed
// their deletion are OVER it by 322, 1484 and 1112. No overlap, and no
// threshold was fitted to produce that: the corpus median is used directly.

/**
 English code points a source page of ordinary completeness renders into, per
 source code point.

 THE CORPUS MEDIAN, measured over all 92 pinned pairs rather than chosen.
 Chinese becoming English expands, and this is by how much on this corpus.

 Using the median rather than a lower percentile makes the shortfall a
 statement about a TYPICAL page, so a page that is merely terse reads as
 slightly short rather than as missing a passage, and the shortfall budget `admitWithinShortfall` spends keeps
 that slight shortness from admitting anything of real size.
 */
export const CORPUS_EXPANSION = 2.65;

/**
 One passage an insertion is being considered for.
 */
export type CandidatePassage = {
  /**
   Where the passage stands in the caller's numbering, handed back for each
   passage admitted so the caller can match an admission to its candidate.
   */
  readonly position: number;

  /**
   Original-side text whose absence is in question.
   */
  readonly sourceText: string;
};

/**
 English size a source passage of ordinary completeness would render into.

 @param sourceText - original-side text

 @returns Code points its translation would be expected to occupy

 @example
 ```ts
 const points = expectedTranslationPoints({ sourceText, },);
 ```
 */
export function expectedTranslationPoints(
  { sourceText, }: { readonly sourceText: string; },
): number {
  return codePointCount({ text: sourceText, },) * CORPUS_EXPANSION;
}

/**
 How much English a page is missing against what its source predicts.

 FLOORED AT ZERO rather than reported negative, because a page LONGER than
 predicted is not evidence of anything: translations run long for reasons that
 have nothing to do with coverage, and a negative shortfall would otherwise
 subtract from a later page's budget if these were ever summed.

 @param sourceText - whole original page

 @param targetText - whole translation as it stands

 @returns Code points of English the page lacks, zero when it lacks none

 @example
 ```ts
 const shortfall = pageShortfall({ sourceText, targetText, },);
 ```
 */
export function pageShortfall(
  {
    sourceText,
    targetText,
  }: {
    readonly sourceText: string;
    readonly targetText: string;
  },
): number {
  /**
   What a page of ordinary completeness would carry.
   */
  const expected = expectedTranslationPoints({ sourceText, },);

  /**
   What it actually carries.
   */
  const standing = codePointCount({ text: targetText, },);

  return Math.max(
    0,
    expected - standing,
  );
}

/**
 Chooses which absent-voted passages the page has room to be missing.

 A BUDGET RATHER THAN A PER-PASSAGE TEST. A page is short by a definite
 amount, and admitting passages whose translations would together exceed it
 would write in more English than the page is missing. On an entry with a
 large shortfall and forty candidates that distinction is the difference
 between restoring a page and rewriting one.

 TAKEN IN THE ORDER GIVEN, which callers supply in document order. Ordering by
 some strength of evidence would need a strength this has no way to measure,
 and document order is at least neutral and reproducible.

 @param sourceText - whole original page

 @param targetText - whole translation as it stands

 @param passages - candidates the roster already voted absent on, in document
 order

 @param shortfall - English the page is missing, when a caller has read it
 off more than the whole page (the untranslated tail's own budget,
 `coverage-tail.ts`); the whole-page shortfall otherwise

 @returns Positions of the passages the shortfall has room for, in the order given

 @example
 ```ts
 const admitted = admitWithinShortfall({ sourceText, targetText, passages, },);
 ```
 */
export function admitWithinShortfall(
  {
    sourceText,
    targetText,
    passages,
    shortfall = pageShortfall({
      sourceText,
      targetText,
    },),
  }: {
    readonly sourceText: string;
    readonly targetText: string;
    readonly passages: readonly CandidatePassage[];
    readonly shortfall?: number;
  },
): readonly number[] {

  // ONE PASS IN DOCUMENT ORDER THAT APPENDS, so each decision reads the total
  // the passages before it committed, and copying the positions admitted so
  // far at every passage costs nothing it buys (ledger B70).
  /**
   Positions of the passages admitted so far, in the order they were weighed.
   */
  const admitted: number[] = [];
  /**
   Expected code points already promised to admitted passages.
   */
  const spent = { points: 0, };
  for (const passage of passages) {
    /**
     What this passage's translation would occupy.
     */
    const wants = expectedTranslationPoints({ sourceText: passage.sourceText, },);
    if ((spent.points + wants) > shortfall)
      continue;
    spent.points += wants;
    admitted.push(passage.position,);
  }
  return admitted;
}

//endregion Coverage corroboration
