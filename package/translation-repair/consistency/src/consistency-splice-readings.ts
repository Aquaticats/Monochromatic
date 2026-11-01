import type { ConsistencyRendering, } from './consistency-count-types.ts';
import { markReadingsInEnglish, } from './consistency-fold-apostrophes.ts';
import {
  holdsEdgeApostrophe,
  holdsEdgeMark,
  isCountedSingleMark,
} from './consistency-loose-marks.ts';
import type { MarkReading, } from './consistency-proven-apostrophes.ts';
import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Consistency splice readings
// WHAT A SPLICE MUST LEAVE AS IT FOUND IT, besides the words and the loose marks inside the
// span (`consistency-splice.ts`, `consistency-loose-marks.ts`): every single mark outside the
// span read as it was, and no second apostrophe beside a mark the receiving place keeps.
//
// A GAIN CASE (defined in `consistency-splice.ts`) is a splice that brings a single mark to an
// edge of the span where the receiving words held no edge apostrophe (`the parents’` over `the family`), WHATEVER
// STANDS INSIDE THE MARK (`holdsEdgeMark`, `consistency-loose-marks.ts`): the words `_cats_ʼ` bring a mark though they hold
// no real emphasis for it to follow, and the page, not the words, says how that mark reads. The planner, the apply stage
// (`gainedSidesOf`, `corpus-run/pass-consistency-apply-gains.ts`) and the reader of a record ask this one definition
// (`bringsAtEdge`). It is planned as a replacement and decided on the page by the pass, but it is never planned where
// any single mark stands outside the span on that side, directly beside it (`markBefore`,
// `markAfter`) or one refusal run away (`markPastBefore`, `markPastAfter`, in
// `consistency-count-types.ts`; all eight single marks, `isCountedSingleMark`), whatever that mark reads as: the arriving mark would stand
// beside it and write two marks for one (`*dogs’*’`; the words `''cats` over `猫` after a kept `’`). It is
// also never planned when the words carry two single marks at an edge where the receiving words
// do not: one of the pair would stand outside the span, where the locator put it when the other
// was read as an apostrophe. `gainsBesideKeptMark` asks both of the side the gain is on.
//
// A MARK OUTSIDE THE SPAN IS READ AS IT WAS (`readingsChangeOutside`). The proof reads each
// apostrophe-like mark as attached (a Latin letter or an ASCII digit on both sides), joint
// (a mark after closing markup that makes one word of the word before the markup and the letter
// after it, read as attached; `consistency-word-joints.ts` defines the joints), trailing, leading or free
// (`consistency-proven-apostrophes.ts`), and the reading depends on the character beside the
// mark, which the words that go in can change (`🐱's` is a leading mark; after words that end in
// `)` it is a joint). The edge-class test of the splice compares the class of an edge character
// only, which is coarser than that reading. So once the splice is built, the English with and
// without it is read, every single mark outside the span is read both ways with the same reading
// the proof uses, and a splice that changes the reading of any of them is not a code splice.
// The English is the one the place holds (its slice), which holds every mark of its block.

/**
 Whether the words that would go in bring a single mark to one edge where the receiving words hold no edge apostrophe: the
 one definition of a gain that the planner, the apply stage and the reader of a record share.

 @internal

 @param text - the words that would go in

 @param core - the receiving words

 @param side - the edge asked about

 @returns Whether the words hold a single mark at that edge, whatever stands inside it, and the receiving words hold no edge apostrophe there

 @example
 ```ts
 const brings = bringsAtEdge({ text: 'the dogs’', core: 'kittens', side: 'end', },); // true
 ```
 */
export function bringsAtEdge(
  {
    text,
    core,
    side,
  }: {
    readonly text: string;
    readonly core: string;
    readonly side: 'start' | 'end';
  },
): boolean {
  if (!holdsEdgeMark({
    text,
    side,
  },))
    return false;
  return !holdsEdgeApostrophe({
    text: core,
    side,
  },);
}

/**
 Whether the words that would go in bring a single mark to an edge where the
 receiving words hold no edge apostrophe, the gain case. The mark is a proven apostrophe in the
 English it stood in (`consistency-proven-apostrophes.ts`), or any single mark at the edge
 of the words, whatever stands inside it (`holdsEdgeMark`), and the receiving English
 is not held here: a mark brought into an English that holds an opener before it, or a
 closer after it, is one that opener could pair with, and the page would then hold a
 loose mark that was not there. So a plan with this answer is a replacement the pass
 must check on the page it makes (`corpus-run/pass-consistency-apply-gains.ts`), and
 that a reader of the record, which holds no English, can only accept as either made
 or left out. A mark the receiving words already hold at that edge is replaced where
 it stands, so it keeps its place among the marks of its English, and a mark that goes
 out leaves the others as they were.

 @param text - the words that would go in

 @param target - the receiving place's located rendering

 @returns Whether the words would put an apostrophe at the start or the end of the span where the receiving words have none

 @example
 ```ts
 const brings = bringsEdgeApostrophe({ text: 'the parents’', target, },);
 ```
 */
export function bringsEdgeApostrophe(
  {
    text,
    target,
  }: {
    readonly text: string;
    readonly target: Extract<ConsistencyRendering, { readonly kind: 'located'; }>;
  },
): boolean {
  return ([
    'start',
    'end',
  ] as const).some(function bringsAt(side,): boolean {
    return bringsAtEdge({
      text,
      core: target.core,
      side,
    },);
  },);
}

/**
 Whether a text holds two single marks side by side at one edge, as the pair `’’` or `''` that a quotation or a doubled mark
 leaves at the start or the end of words.

 @internal

 @param text - the words

 @param side - the edge asked about

 @returns Whether the two whole characters at that edge are both single marks

 @example
 ```ts
 const pair = holdsTwoMarksAtEdge({ text: 'the dogs’’', side: 'end', },); // true
 ```
 */
export function holdsTwoMarksAtEdge(
  {
    text,
    side,
  }: {
    readonly text: string;
    readonly side: 'start' | 'end';
  },
): boolean {
  /**
   The whole characters of the words, so a letter beyond the first plane is one.
   */
  const points = Array.from(text,);
  /**
   The two whole characters at the edge.
   */
  const pair = (side === 'start')
    ? points.slice(
      0,
      2,
    )
    : points.slice(-2,);
  return (pair.length === 2) && pair.every(function isMark(character,): boolean {
    return isCountedSingleMark({ character, },);
  },);
}

/**
 Whether the words that would go in put a single mark beside one the receiving place keeps, so the two would stand as a doubled
 mark: the words bring an apostrophe to an edge where the receiving words hold none, and any single mark stands outside the
 span on that side, directly beside it or a wide run away (past markup, a bracket, a quotation mark), whatever the kept mark
 reads as; or the words carry two single marks at an edge where the receiving words do not, since one of the pair would stand
 outside the span, where the locator put it when the other was read as an apostrophe.

 @internal

 @param text - the words that would go in

 @param target - the receiving place's located rendering

 @returns Whether a gain or a doubled pair arrives on a side where a single mark stands outside the span

 @example
 ```ts
 const doubled = gainsBesideKeptMark({ text: 'the dogs’', target, },); // true for `*kittens*’`
 ```
 */
export function gainsBesideKeptMark(
  {
    text,
    target,
  }: {
    readonly text: string;
    readonly target: Extract<ConsistencyRendering, { readonly kind: 'located'; }>;
  },
): boolean {
  /**
   The two edges of the span.
   */
  const sides = [
    'start',
    'end',
  ] as const;
  if (sides.some(function carriesPairOnly(side,): boolean {
    return holdsTwoMarksAtEdge({
      text,
      side,
    },) && (!holdsTwoMarksAtEdge({
      text: target.core,
      side,
    },));
  },))
    return true;
  // Words that bring no edge apostrophe put nothing beside a kept mark.
  if (!bringsEdgeApostrophe({
    text,
    target,
  },))
    return false;
  return sides.some(function gainsOn(side,): boolean {
    // Words that bring no edge apostrophe on this side put nothing beside a kept mark there.
    if (!bringsAtEdge({
      text,
      core: target.core,
      side,
    },))
      return false;
    /**
     The marks the receiving place keeps outside the span on this side: directly beside it, then one refusal run away.
     */
    const kept = (side === 'start')
      ? [
        target.markBefore,
        target.markPastBefore,
      ]
      : [
        target.markAfter,
        target.markPastAfter,
      ];
    return kept.some(function isHeld(mark,): boolean {
      return mark !== undefined;
    },);
  },);
}

/**
 Whether putting words in the place of a span changes how any single mark outside the span is read.

 @param english - the English the span stands in, before the splice

 @param start - offset where the span starts

 @param end - exclusive offset where it ends, counted in UTF-16 units: a span that takes in an emoji ends after both of its units
 (in the example the emoji stands at offsets 13 and 14 and the span ends at 15, right before the `'`, which is the mark outside)

 @param written - the words that go in

 @param definitions - the page's link and footnote definitions (`pageDefinitionsOf`), none for an English read alone: the English
 is one slice's wording, and a reference in it whose definition stands in another slice is a link, so words that make it an
 undefined reference change how the emphasis around it pairs and how the marks outside the span read (n173). Both readings are
 made with them

 @returns Whether a mark outside the span reads another way (attached, joint, trailing, leading or free) once the words are in

 @example
 ```ts
 const changed = readingsChangeOutside({ english: 'He said cats 🐱\'s now.', start: 8, end: 15, written: 'Come (here)', },); // true
 ```
 */
export function readingsChangeOutside(
  {
    english,
    start,
    end,
    written,
    definitions = '',
  }: {
    readonly english: string;
    readonly start: number;
    readonly end: number;
    readonly written: string;
    readonly definitions?: string;
  },
): boolean {
  /**
   The English with the words in.
   */
  const spliced = [
    english.slice(
      0,
      start,
    ),
    written,
    english.slice(end,),
  ].join('',);
  /**
   How many units further on the text after the span stands once the words are in.
   */
  const shift = written.length - (end - start);
  /**
   The marks outside the span, read before the splice, as offset and reading.
   */
  const outsideBefore = [...markReadingsInEnglish({
    english,
    definitions,
  },),].filter(function standsOutside([at,]: ReadonlyEntry<number, MarkReading>,): boolean {
    return (at < start) || (at >= end);
  },);
  /**
   The marks outside the span, read after it.
   */
  const outsideAfter = [...markReadingsInEnglish({
    english: spliced,
    definitions,
  },),].filter(function standsOutside([at,]: ReadonlyEntry<number, MarkReading>,): boolean {
    return (at < start) || (at >= (start + written.length));
  },);
  if (outsideBefore.length !== outsideAfter.length)
    return true;
  return outsideBefore.some(function readsOtherwise(
    [at, reading,]: ReadonlyEntry<number, MarkReading>,
    order,
  ): boolean {
    /**
     The same mark after the splice.
     */
    const counterpart = outsideAfter[order];
    return (counterpart === undefined)
      || (counterpart[0] !== ((at < start) ? at : (at + shift)))
      || (counterpart[1] !== reading);
  },);
}

//endregion Consistency splice readings
