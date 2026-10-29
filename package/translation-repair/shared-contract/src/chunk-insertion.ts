import type { ForcedAlignStep, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { ContentChunk, } from './chunk-placement.ts';
import {
  admitWithinShortfall,
  pageShortfall,
} from './coverage-corroboration.ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

//region Chunk insertion placement
// Section-scale insertion: which untranslated sections get an anchor to be written at,
// and which are refused.
//
// TWO SIGNATURES, NEVER ONE, per
// `doc/decision/translation-repair-absence-verdict.md`. The aligner must prove
// the section has no partner anywhere and that every optimal alignment skips it
// at the same place, AND the page must be measurably too short to hold what is
// being written into it. A false insertion duplicates text in a memorial page,
// which is the expensive error; a missed one leaves a gap the archive already
// had.
//
// THE SIZE TEST IS THE HALF THE ALIGNER CANNOT DO. The ambiguity is a MERGE
// against an OMISSION, and an aligner reports both as an unpaired source
// section. A merge leaves the content somewhere in a page of ordinary length; an
// omission leaves the page short. Neither signal sees what the other sees, which
// is why both are required rather than whichever is cheaper.
//
// EVERY REFUSAL IS NAMED. Four different things stop an insertion, and they want
// different remedies: a section that may already be on the page, one whose place
// is undetermined, a page that looks complete, and a page already accounted for
// by earlier sections. Reporting them as one "not inserted" is the mistake this
// module exists to avoid, and the same one that made an earlier breadth run
// unreadable.

/**
 Why an untranslated section was not given a place to be written at.
 */
export type InsertionRefusal =
  /**
   Some optimal alignment pairs it with existing translation, so whatever it
   says may already be on the page and writing it in would duplicate content.
   */
  | 'may-pair'
  /**
   Nothing pairs it, but the optimal alignments disagree about where it sits,
   so writing it in risks filing real content under the wrong section.
   */
    | 'several-boundaries'
  /**
   The page carries at least as much English as its source predicts, so it has
   no room to be missing anything and the aligner is likelier to have found a
   merge than an omission.
   */
    | 'page-not-short'
  /**
   The page is short, but earlier sections already account for the whole
   shortfall, so writing this in as well would add more English than the page
   is missing.
   */
    | 'beyond-shortfall';

/**
 What was decided about one untranslated source section.
 */
export type InsertionPlacement =
  | {
    /**
     Both signatures agree, and this is where it goes.
     */
    readonly kind: 'placed';

    /**
     Section this describes.
     */
    readonly sourceIndex: number;

    /**
     Boundary in the translation new text is written at.
     */
    readonly offset: number;
  }
    | {
    /**
     One of the signatures refused.
     */
      readonly kind: 'unplaced';

    /**
     Section this describes.
     */
      readonly sourceIndex: number;

    /**
     Which signature refused, and how.
     */
      readonly refusal: InsertionRefusal;
    };

/**
 Turns a proven section boundary into a document offset.

 A boundary EQUAL to the section count means the section belongs after
 everything the translation carries, which is the end of the last section
 rather than the start of a section that does not exist.

 ONLY THOSE TWO ARRIVE HERE: `anchorFor` (`align-headings-forced.ts`) and
 `anchorBetween` (`pair-sections-steps.ts`) build `beforeTargetIndex` from a
 boundary between 0 and the translation's section count, both included.

 @param beforeTargetIndex - section the insertion precedes

 @param targetChunks - translation sections in document order, never empty

 @returns Offset in the translation text

 @example
 ```ts
 const offset = offsetOfBoundary({ beforeTargetIndex: 2, targetChunks, },);
 ```
 */
function offsetOfBoundary(
  {
    beforeTargetIndex,
    targetChunks,
  }: {
    readonly beforeTargetIndex: number;
    readonly targetChunks: readonly ContentChunk[];
  },
): number {
  /**
   Section the insertion sits before, absent when it sits after everything.
   */
  const following = targetChunks[beforeTargetIndex];
  if (following !== undefined)
    return following.startOffset;

  /**
   Last section, whose end is where a trailing insertion lands. ALWAYS
   PRESENT: `following` is absent only at `beforeTargetIndex ===
   targetChunks.length`, and `alignDocumentSections` returns before placing
   anything when the translation has no sections.
   */
  const last = nonNullishOrThrow(targetChunks.at(-1,),);
  return last.endOffset;
}

/**
 Decides which untranslated sections get an anchor, and names every refusal.

 @param steps - aligner output for the whole document

 @param sourceChunks - original sections in document order

 @param targetChunks - translation sections in document order, never empty

 @param sourceText - whole original page

 @param targetText - whole translation as it stands

 @returns One decision per untranslated section, in document order

 @example
 ```ts
 const placements = placeInsertions({ steps, sourceChunks, targetChunks, sourceText, targetText, },);
 ```
 */
export function placeInsertions(
  {
    steps,
    sourceChunks,
    targetChunks,
    sourceText,
    targetText,
  }: {
    readonly steps: readonly ForcedAlignStep[];
    readonly sourceChunks: readonly ContentChunk[];
    readonly targetChunks: readonly ContentChunk[];
    readonly sourceText: string;
    readonly targetText: string;
  },
): readonly InsertionPlacement[] {
  /**
   Sections the aligner refused to pair, with the anchor it proved or the
   uncertainty it hit.
   */
  const unpaired = steps.flatMap(function toUnpaired(step,) {
    return (step.kind === 'source-only') ? [step,] : [];
  },);

  /**
   Those whose place every optimal alignment agrees on, which are the only
   ones the size test is asked about.
   */
  const anchored = unpaired.filter(function isProven(step,): boolean {
    return step.anchor
      .kind
      === 'proven';
  },);

  /**
   Which of them the page has room to be missing, by source index.
   */
  const admitted = new Set(admitWithinShortfall({
    sourceText,
    targetText,
    passages: anchored.map(function toPassage(step,) {
      return {
        position: step.sourceIndex,
        // A step names a chunk that exists, by construction; an absent one is
        // a fault to raise, not a passage to read as empty.
        sourceText: nonNullishOrThrow(sourceChunks[step.sourceIndex],)
          .text,
      };
    },),
  },),);

  /**
   Whether the page is short at all.

   MEASURED RATHER THAN INFERRED FROM THE ADMISSIONS. Reading "nothing was
   admitted" as "the page is not short" is wrong exactly when the first
   candidate is larger than the whole shortfall: the page is genuinely short,
   nothing fits, and the run would report a complete-looking page. That is the
   same collapse the refusal union exists to prevent, so it must not be
   reintroduced by the code that fills it in.
   */
  const pageIsShort = pageShortfall({
    sourceText,
    targetText,
  },) > 0;

  return unpaired.map(function decide(step,): InsertionPlacement {
    /**
     What the aligner proved about where this section sits.
     */
    const { anchor, } = step;

    if (anchor.kind !== 'proven')
      return {
        kind: 'unplaced',
        sourceIndex: step.sourceIndex,
        refusal: anchor.kind,
      };

    if (!admitted.has(step.sourceIndex,))
      return {
        kind: 'unplaced',
        sourceIndex: step.sourceIndex,
        refusal: pageIsShort ? 'beyond-shortfall' : 'page-not-short',
      };

    return {
      kind: 'placed',
      sourceIndex: step.sourceIndex,
      offset: offsetOfBoundary({
        beforeTargetIndex: anchor.beforeTargetIndex,
        targetChunks,
      },),
    };
  },);
}

/**
 Names one placement for an alignment finding.

 @param placement - what was decided about one section

 @returns Sentence a reader can act on

 @example
 ```ts
 const detail = describePlacement(placement,);
 ```
 */
export function describePlacement(placement: InsertionPlacement,): string {
  if (placement.kind === 'placed')
    return `anchored for insertion at offset ${String(placement.offset,)}`;

  return `not anchored (${placement.refusal})`;
}

//endregion Chunk insertion placement
