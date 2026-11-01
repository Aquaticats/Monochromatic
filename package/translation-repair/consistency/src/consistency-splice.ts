import { isCasedLetter, } from '@monochromatic-dev/translation-repair-typography/ts';
import { inReceivingForm, } from './consistency-apostrophe-forms.ts';
import { blockBreaksIn, } from './consistency-block-breaks.ts';
import type { ConsistencyRendering, } from './consistency-count-types.ts';
import { ConsistencyInvariantError, } from './consistency-invariant-error.ts';
import { sameEdgeClass, } from './consistency-edge-class.ts';
import {
  looseMarksOf,
  sameBesideMark,
  withLooseMarksOf,
} from './consistency-loose-marks.ts';
import {
  gainsBesideKeptMark,
  readingsChangeOutside,
} from './consistency-splice-readings.ts';

//region Consistency splice
// Moving a chosen place's quoted words over another place's, by code.
//
// NO MODEL WRITES NEW TEXT HERE (the owner's ruling of 2026-10-11). The words
// a replacement carries are words a majority of judges quoted from the page.
// Two edges of the quotes are the only things this code adjusts, and each is
// a rule about the place the words go to rather than a rewrite: the closing
// punctuation of a quote stays where the receiving place has it, and a first
// letter (the first past any opening mark) takes the receiving place's capital
// or small form when the two quotes begin with the same letter.
//
// A PROVEN APOSTROPHE IS PART OF ITS WORD. The span a replacement covers is the
// words and the apostrophe beside them that is proven one
// (`consistency-fold.ts`, `consistency-proven-apostrophes.ts`): `the parents’` and
// `the folks’` are the words that two places hold, and a replacement of one by the
// other carries the apostrophe in with the word, as `don't` moves with `don't`.
// Such a mark is inside the span, is not recorded beside it, and counts as attached
// for the loose marks of the words (`looseMarksOf`). A replacement may take the
// words of a place that holds one to a place that holds one at the same edge (the
// receiving mark is replaced where it stands, so it keeps its place among the marks
// of its English), and may take them to a place that holds none at that edge only
// by taking the mark out, which leaves the other marks of the English as they were.
//
// THE GAIN CASE IS PLANNED AND DECIDED ON THE PAGE. Bringing a proven apostrophe to an
// edge where the receiving words hold none is a replacement here, and the plan alone
// cannot say whether the mark stays an apostrophe: that depends on the single marks of
// the receiving English, which the located rendering does not record (an opener before
// the place would make the new mark the closer of it, and two gains in one English can
// each be proven alone and neither together). The pass decides it where it holds every
// replacement it makes (`corpus-run/pass-consistency-apply-gains.ts`): a replacement
// whose carried mark is not proven on that page is left out and the place keeps its
// words, recorded `kept-current`. `bringsEdgeApostrophe` (`consistency-splice-readings.ts`)
// names the plans that have that
// question to answer, for the pass and for a reader of the record, which cannot answer
// it because it holds no English.
//
// A SPLICE IS REFUSED WHEN IT WOULD WRITE A PAIRING OF WORDS AND MARKS NEITHER
// PLACE WROTE. Every single mark the English holds directly beside the span that is
// no proven apostrophe (half of a quotation, an apostrophe with a possible partner,
// the opening curly mark) stays on the page whatever words go in, and a quotation
// mark inside the words goes with them. Moving `goin` to a place that held `going`
// would drop the apostrophe the chosen place wrote when a quotation mark stands in
// its English; moving `The cat` over `The cats'` where an opener stands before it
// would leave the apostrophe after a word that wants none; moving words that hold a
// quotation mark over words that hold none, or the reverse, would delete one or leave
// one without its pair. So `planConsistencyEdit` calls a splice unavailable on four
// tests of the marks beside and inside the words, and on readings of the English and the
// page that are made once the words that would go in are built. The four tests: when the
// two places differ in the single mark beside the span (`markBefore`, `markAfter`);
// when the words that would go in hold different loose marks
// (`consistency-loose-marks.ts`) from the receiving words; when the two hold a
// different number of block breaks (`consistency-block-breaks.ts`), which no page can
// keep, the structure floor refusing a page that lost a block; or when a single mark
// stands beside the receiving words and the edge character of the words that would go
// in, on that side, is of another class than the edge character of the receiving
// words. The two classes are a Latin letter with its accent, an ASCII digit or a compatibility
// character of those (the neighbour test of the consistency pass,
// `continuesLatinWordOrCompatibilityForm` in `latin-letters.ts`, which `looseMarksOf` and the
// proof read as attaching a mark to a word) and anything else, and an empty edge is a
// class of its own. Why the edge-class test: a
// single mark is attached when such a letter or digit stands on both sides of it, a
// replacement of the span leaves the far side of the mark alone, so words whose
// edge is of the same class leave the mark attached or loose as it was, and words
// whose edge is of another class can turn a loose apostrophe into one a letter
// now attaches to (`**cat**'s` given `cats` reads `cats's`). The far side is not
// recorded with the mark, so the classes are compared whether or not a letter
// stands there: a splice is refused where it could not have changed the pairing
// as well as where it could.
//
// THE TWO READINGS OF THE ENGLISH (`consistency-splice-readings.ts`) NEED THE ENGLISH,
// which the rendering does not record. The first, `gainsBesideKeptMark` (decision D1): the words
// bring a proven apostrophe to an edge, and any single mark stands outside the span on that
// side, directly beside it (`markBefore`, `markAfter`) or past a run of markup
// beyond that edge (emphasis, a backtick, a closing bracket or double quotation mark, a
// link's destination), whatever that mark reads as, so the replacement would write two marks
// for the eye (`*dogs’*’`); or the words carry two single marks at an edge where the receiving
// words do not, so one of the pair would stand outside the span. The rendering carries the mark
// past the markup as `markPastBefore`/`markPastAfter`
// (`consistency-count.ts`), and the plan is unavailable. The second,
// `readingsChangeOutside`: every single mark of the English outside the span is read
// the way the proof reads it (attached, joined to the word after it across closing
// markup, trailing, leading or free) with the receiving words in place and again with
// the words that would go in, and if any mark reads differently the plan is
// unavailable. A THIRD READING OF THE PAGE (`pageReference`, `quoteReferencesIn`,
// `consistency-quote-references.ts`): the page holds a character reference that decodes to a
// quotation-like character (`&#39;`, `&rsquo;`), which no reader of the raw English sees as a mark,
// wherever it stands on the page, so every splice is unavailable, whether the words bring a gain or not
// (the chosen words may start or end in the reference, and the receiving words may too). The page is
// the English of every slice of it, a slice read alone included. All three
// make the plan unavailable and not `kept-current`: a reader of the
// record, which holds no English, cannot make them and recomputes the plan from the
// words, so it accepts `kept-current` for a replacement plan whose splice was chosen
// (`artifact-two-lane-read-consistency-span-shared.ts`). With the four tests and the
// readings passed, replacing the span
// leaves every loose mark of the receiving place where it is, brings none in, and
// leaves every single mark of the English attached or loose as it was; the one
// exception is the gain case (`bringsEdgeApostrophe`), whose mark is decided on the page. Two
// single marks in a row beside a span (`’’`, `‘’`, `''`) are covered the same way: the mark
// directly beside the span is the one recorded and compared, the farther one stands outside
// the span and is read by `readingsChangeOutside`. A record
// written before the marks beside a span were kept is planned by its words alone.
//
// THE LOOSE MARKS THAT GO IN ARE WRITTEN IN THE RECEIVING PLACE'S OWN FORM. The
// loose marks of the words that go in and of the receiving words are equal once
// folded, so they correspond one to one, and the first loose mark that goes in
// takes the character of the first loose mark the receiving words hold, and so on
// (`withLooseMarksOf`): straight quotation marks carried onto a place written
// with curly ones leave it curly, and the reverse. Nothing is guessed, since the
// sequences are the same length by the loose-marks test of `planConsistencyEdit`.
// The opening curly single mark still equals only itself, so it never changes.
//
// THE APOSTROPHES THAT GO IN ARE WRITTEN IN IT TOO (`consistency-apostrophe-forms.ts`).
// An apostrophe inside a word, or a proven one at an edge, is no loose mark, and goes in
// in the form the apostrophes of the receiving words share when they share one, else the
// form the apostrophes of the receiving English outside the span share when they share
// one (asked of the caller, who holds the English), else as the chosen place wrote it.
// Words that differ from the receiving words in no more than the form of their loose
// marks and of their apostrophes are the same words, and the plan is `same`. A record
// that compared no marks keeps the words as the chosen place wrote them.

/**
 One replacement of words in a place's English.

 @example
 ```ts
 const edit: ConsistencyEdit = { id: 'g0:OCCURRENCE 2', start: 0, end: 12, text: 'the cat naps', };
 ```
 */
export type ConsistencyEdit = {
  /**
   Name the outcome of this edit is reported under.
   */
  readonly id: string;

  /**
   Offset in the English where the replaced words start.
   */
  readonly start: number;

  /**
   Exclusive offset where they end.
   */
  readonly end: number;

  /**
   The words that go in their place.
   */
  readonly text: string;
};

/**
 What moving the chosen place's words over another place amounts to.

 @example
 ```ts
 const plan: ConsistencyPlan = { kind: 'same', };
 ```
 */
export type ConsistencyPlan =
  | {
    /**
     The receiving place already carries these words.
     */
    readonly kind: 'same';
  }
    | {
    /**
     The words cannot be moved by code without writing a pairing of words and
     marks neither place wrote: a frame's chosen place or receiving place has
     no agreed slot, the two places hold different single marks directly beside
     their spans, the words that would go in hold different loose marks from
     the receiving words, the words that would go in hold another number of block
     breaks than the receiving words (a blank line, or a line break the Markdown
     parse reads as a new block), which the structure floor would always refuse,
     or a single mark stands beside the receiving words and the words that would
     go in end on that side in another class of character.
     A writer can carry them where one is asked for the block that holds the
     place; a block no writer is asked about keeps the place as it is.
     */
      readonly kind: 'unavailable';
    }
    | {
    /**
     The receiving place's words are replaced.
     */
      readonly kind: 'replace';

    /**
     Offset where the replaced words start.
     */
      readonly start: number;

    /**
     Exclusive offset where they end.
     */
      readonly end: number;

    /**
     The words that go in.
     */
      readonly text: string;
    };

/**
 Marks that may open a quote before its first letter: quotation marks,
 brackets and the characters that open emphasis, code or strikethrough.
 */
const OPENING_MARKS = '"\'“‘„«‹([{*_~`¿¡「『（《〈';

/**
 Where a text's first letter stands: past any opening marks, as the character
 there and its offset.

 @param text - words whose first letter is wanted

 @returns The first character after the opening marks and its offset in UTF-16
 units, as a list of one; empty for a text made of opening marks alone

 @example
 ```ts
 const [first,] = firstLetterOf({ text: '“The cat', },); // { character: 'T', at: 1 }
 ```
 */
function firstLetterOf(
  { text, }: { readonly text: string; },
): readonly {
  readonly character: string;
  readonly at: number;
}[] {
  /**
   Offset reached so far, in UTF-16 units.
   */
  const reached = { at: 0, };
  for (const character of text) {
    if (!OPENING_MARKS.includes(character,)) {
      return [
        {
          character,
          at: reached.at,
        },
      ];
    }
    reached.at += character.length;
  }
  return [];
}

/**
 A letter in the case of another, when the two are one letter.

 @param text - words whose first letter may change case

 @param like - words whose first letter sets the case

 @returns The text with its first letter, whole and past any opening mark, in
 the case it has in `like`, or the text itself when the two first letters are
 not one letter in two cases

 @example
 ```ts
 const adapted = firstLetterLike({ text: 'The cat naps', like: 'the cat sleeps', },); // 'the cat naps'
 ```
 */
function firstLetterLike(
  {
    text,
    like,
  }: {
    readonly text: string;
    readonly like: string;
  },
): string {
  /**
   First letter of each.
   */
  const [first,] = firstLetterOf({ text, },);
  /**
   First letter of the text whose case is followed.
   */
  const [model,] = firstLetterOf({ text: like, },);
  if ((first === undefined) || (model === undefined)
    || (first.character === model.character))
    return text;
  if ((!isCasedLetter({ character: first.character, },)) || (!isCasedLetter({ character: model.character, },)))
    return text;
  if (first.character
    .toLowerCase()
    !== model.character
      .toLowerCase())
    return text;
  return `${text.slice(
    0,
    first.at,
  )}${model.character}${text.slice(first.at
    + first.character
      .length,)}`;
}

/**
 Words a frame's chosen pattern gives a receiving place.

 @param chosen - the chosen place's located rendering

 @param target - the receiving place's located rendering

 @returns The chosen quote's words with its slot words replaced by the
 receiving place's slot words, or the chosen words themselves when the group
 has no slot

 @example
 ```ts
 const words = patternFor({ chosen, target, },);
 ```
 */
function patternFor(
  {
    chosen,
    target,
  }: {
    readonly chosen: Extract<ConsistencyRendering, { readonly kind: 'located'; }>;
    readonly target: Extract<ConsistencyRendering, { readonly kind: 'located'; }>;
  },
): string {
  if ((chosen.slot === undefined) || (chosen.slotStart === undefined)
    || (target.slot === undefined))
    return chosen.core;
  return `${chosen.core
    .slice(
      0,
      chosen.slotStart,
    )}${target.slot}${
    chosen.core
      .slice(chosen.slotStart
        + chosen.slot
          .length,)
  }`;
}

/**
 Whether two places hold the same single marks directly before and after their
 spans.

 @param chosen - the chosen place's located rendering

 @param target - the receiving place's located rendering

 @returns Whether `markBefore` agrees and `markAfter` agrees, with the three
 apostrophe-like marks one and an absent mark equal only to an absent one

 @example
 ```ts
 const agree = sameMarksBeside({ chosen, target, },);
 ```
 */
function sameMarksBeside(
  {
    chosen,
    target,
  }: {
    readonly chosen: Extract<ConsistencyRendering, { readonly kind: 'located'; }>;
    readonly target: Extract<ConsistencyRendering, { readonly kind: 'located'; }>;
  },
): boolean {
  /**
   The marks the chosen place holds before and after its span.
   */
  const {
    markBefore: chosenBefore,
    markAfter: chosenAfter,
  } = chosen;
  /**
   The marks the receiving place holds before and after its span.
   */
  const {
    markBefore: targetBefore,
    markAfter: targetAfter,
  } = target;
  return sameBesideMark({
    left: (chosenBefore === undefined) ? [] : [chosenBefore,],
    right: (targetBefore === undefined) ? [] : [targetBefore,],
  },)
    && sameBesideMark({
      left: (chosenAfter === undefined) ? [] : [chosenAfter,],
      right: (targetAfter === undefined) ? [] : [targetAfter,],
    },);
}

/**
 Whether two texts hold the same loose marks in the same order.

 @param left - one text

 @param right - the other

 @returns Whether the sequences of loose marks agree, each compared in its folded form

 @example
 ```ts
 const agree = sameLooseMarks({ left: '"So kind," she said', right: '"Be kind," she said', },); // true
 ```
 */
function sameLooseMarks(
  {
    left,
    right,
  }: {
    readonly left: string;
    readonly right: string;
  },
): boolean {
  /**
   The loose marks of one text, as one text.
   */
  const ofLeft = looseMarksOf({ text: left, },)
    .join('',);
  /**
   The loose marks of the other.
   */
  const ofRight = looseMarksOf({ text: right, },)
    .join('',);
  return ofLeft === ofRight;
}

/**
 Whether a replacement of the receiving words leaves every single mark directly
 beside them attached or loose as it was: on each side where the receiving place
 holds a single mark, the edge character of the words that would go in is of the
 same class as the edge character of the receiving words.

 @param text - the words that would go in

 @param target - the receiving place's located rendering

 @returns Whether no mark beside the receiving words can change from attached to
 loose or back

 @example
 ```ts
 const keeps = keepsMarksAttached({ text: 'the cats', target, },);
 ```
 */
function keepsMarksAttached(
  {
    text,
    target,
  }: {
    readonly text: string;
    readonly target: Extract<ConsistencyRendering, { readonly kind: 'located'; }>;
  },
): boolean {
  return ((target.markBefore === undefined) || sameEdgeClass({
    left: text,
    right: target.core,
    side: 'start',
  },))
    && ((target.markAfter === undefined) || sameEdgeClass({
      left: text,
      right: target.core,
      side: 'end',
    },));
}

/**
 Plans moving the chosen place's rendering over a receiving place's.

 With the marks compared, a splice is unavailable when the two places hold
 different single marks directly beside their spans, when the words that
 would go in hold different loose marks (quotation marks, or apostrophes that
 are no part of a word, a proven apostrophe at an edge of the words being part
 of its word) from the receiving place's own words, when the two hold a different
 number of block breaks, or when a single mark stands beside the receiving words
 and the words that would go in end on that side in another class of character (a
 letter or a digit against anything else, which can attach a loose apostrophe to a
 letter or loose one that was attached). All four false, replacing the span leaves
 every loose mark of the receiving place where it is, brings none in, and leaves
 each single mark beside the span attached or loose as it was, so the result pairs
 words and marks the way both places already did; the exception is a proven
 apostrophe brought to an edge where the receiving words hold none
 (`bringsEdgeApostrophe`), which is planned as a replacement and decided on the
 page by the pass. The marks are checked before the places are called the same: two
 places whose words agree and whose marks differ are not one rendering. The words
 that go in carry each loose mark in the form the receiving place wrote it at that
 position (`withLooseMarksOf`), and each apostrophe of a word in the receiving
 place's form (`inReceivingForm`). Words that then equal the receiving words are
 the same.

 @param chosen - the chosen place's located rendering

 @param target - the receiving place's located rendering

 @param framed - whether the group is a frame, whose words move only with both
 places' slots

 @param marks - `compared` to hold the marks beside and inside the words to
 the rule this function states; `unrecorded` for a record written when no mark beside a span
 was kept, whose renderings cannot be compared that way and are planned by
 their words alone, as they were written

 @param pageApostrophe - asked only when the receiving words share no apostrophe form: the form
 the page's prose outside the receiving words shares, as a list of zero or one; absent for a caller
 that holds no English, whose plan kind is the same either way

 @param pageReference - asked for any splice, that is once the words that go in differ from the receiving words: the character
 references of the page that decode to a quotation-like character (`quoteReferencesIn`, `consistency-quote-references.ts`) in
 the English of every slice of it, as a list that is empty for a page with none; the splice is unavailable when it is not
 empty; absent for a caller that holds no page, which cannot make the check, so the pass alone decides it

 @param english - the English the receiving words stand in (`target.start` addresses it), when the caller holds it: once the
 splice is built, every single mark of it outside the receiving words is read as the proof reads it, before and after, and a
 splice that changes the reading of any is unavailable; absent for a caller that holds no English (a reader of the record),
 which cannot make the check, so the pass alone decides it

 @param definitions - the page's link and footnote definitions (`pageDefinitionsOf`), read together with `english` and only with it: the
 English is one slice's wording, and a reference in it whose definition stands in another slice is a link, so words that make it an
 undefined reference change how the emphasis around it pairs and how the marks outside the span read (n173). Absent for a slice
 read alone and for a caller that holds no English. The words of the places are read without them: the reader of a record plans the
 same splice from the recorded words alone and holds no page, so a definition handed to the reading of the words would let the
 pass plan what the reader refuses, and the definitions would not make the words read as the page does (`consistency-loose-marks.ts`, ledger B673)

 @returns That the receiving place already agrees, that the words cannot be
 moved by code (no slot to move a frame's words by, a mark beside the words
 that differs, loose marks inside the words that differ, a different number of
 block breaks, or an edge of the words of another class beside a mark), or the
 replacement

 @example
 ```ts
 const plan = planConsistencyEdit({ chosen, target, framed: false, marks: 'compared', },);
 ```
 */
export function planConsistencyEdit(
  {
    chosen,
    target,
    framed,
    marks,
    pageApostrophe,
    pageReference,
    english,
    definitions = '',
  }: {
    readonly chosen: Extract<ConsistencyRendering, { readonly kind: 'located'; }>;
    readonly target: Extract<ConsistencyRendering, { readonly kind: 'located'; }>;
    readonly framed: boolean;
    readonly marks: 'compared' | 'unrecorded';
    readonly pageApostrophe?: () => readonly string[];
    readonly pageReference?: () => readonly string[];
    readonly english?: string;
    readonly definitions?: string;
  },
): ConsistencyPlan {
  /**
   Whether the marks are held to the rule, which a record that kept none cannot be.
   */
  const compared = marks === 'compared';
  if (compared && (!sameMarksBeside({
    chosen,
    target,
  },)))
    return { kind: 'unavailable', };
  if (framed && ((chosen.slot === undefined) || (target.slot === undefined))) {
    return (chosen.core === target.core)
      ? { kind: 'same', }
      : { kind: 'unavailable', };
  }
  /**
   The words the receiving place is given.
   */
  const text = firstLetterLike({
    text: patternFor({
      chosen,
      target,
    },),
    like: target.core,
  },);
  if (compared && (!sameLooseMarks({
    left: text,
    right: target.core,
  },)))
    return { kind: 'unavailable', };
  if (compared && (blockBreaksIn({ text, },) !== blockBreaksIn({ text: target.core, },)))
    return { kind: 'unavailable', };
  if (compared && (!keepsMarksAttached({
    text,
    target,
  },)))
    return { kind: 'unavailable', };
  /**
   The words with each loose mark written as the receiving place wrote the mark at that position, so a straight
   quotation mark carried onto a curly place leaves it curly; the loose marks agree once folded, which is all
   this compares, and a record that compared none keeps the words as they were.
   */
  const marked = compared
    ? withLooseMarksOf({
      text,
      like: target.core,
    },)
    : text;
  // Words already equal to the receiving words are the same whatever the page's form, so the kind never depends on it.
  if (marked === target.core)
    return { kind: 'same', };
  /**
   The words with each apostrophe of a word, inside it or proven at its edge, written in the receiving place's form,
   which the page decides only when the receiving words share no form and a caller that holds the English gave it.
   */
  const written = compared
    ? inReceivingForm({
      text: marked,
      receiving: target.core,
      ...((pageApostrophe === undefined) ? {} : { page: pageApostrophe, }),
    },)
    : marked;
  if (written === target.core)
    return { kind: 'same', };
  // The form an apostrophe is written in can change how the words parse (the modifier letter apostrophe is a letter, so
  // `_cats_ʼ` has no closing underscore where `_cats_’` has), so the loose marks are read again on the words as they go in.
  if (compared && (!sameLooseMarks({
    left: written,
    right: target.core,
  },)))
    return { kind: 'unavailable', };
  if (compared && gainsBesideKeptMark({
    text: written,
    target,
  },))
    return { kind: 'unavailable', };
  // A page that holds a character reference that is a quotation mark shows a mark no reader of the raw English sees, so every
  // splice on it is refused wherever the reference stands, a gain or not: the chosen words may themselves start or end in the
  // reference, and no reader of the raw English sees that mark either. The references are asked for once the words differ.
  if (compared && (pageReference !== undefined)) {
    /**
     The references of the page.
     */
    const references = pageReference();
    if (references.length > 0)
      return { kind: 'unavailable', };
  }
  /**
   Where the receiving words end in the English.
   */
  const end = target.start
    + target.core
      .length;
  /**
   The English the readings are made on: the one the caller holds, when the marks are held to the rule.
   */
  const read = compared ? english : undefined;
  if ((read !== undefined) && readingsChangeOutside({
    english: read,
    start: target.start,
    end,
    written,
    definitions,
  },))
    return { kind: 'unavailable', };
  return {
    kind: 'replace',
    start: target.start,
    end,
    text: written,
  };
}

/**
 What applying a place's edits came to.

 @example
 ```ts
 const applied: AppliedEdits = { text: 'The cat naps.', applied: [], overlapped: [], };
 ```
 */
export type AppliedEdits = {
  /**
   The English with the applied edits made.
   */
  readonly text: string;

  /**
   Ids of the edits made.
   */
  readonly applied: readonly string[];

  /**
   Ids of the edits left out because their words overlap an edit that came
   before them.
   */
  readonly overlapped: readonly string[];
};

/**
 Applies edits to one English, the earlier edit winning wherever two overlap.

 Every edit's offsets address the English as it stood when the words were
 located, so the text is cut once, from the start, between the kept edits,
 which overlap none another, and none moves another.

 @param english - the place's English before any edit

 @param edits - edits in priority order

 @returns The edited English and which edits were made

 @throws {@link Error} when an edit's span lies outside the English or is
 empty, since offsets located in that English cannot

 @example
 ```ts
 const { text, } = applyConsistencyEdits({ english, edits, },);
 ```
 */
export function applyConsistencyEdits(
  {
    english,
    edits,
  }: {
    readonly english: string;
    readonly edits: readonly ConsistencyEdit[];
  },
): AppliedEdits {
  /**
   Edits kept so far.
   */
  const kept: ConsistencyEdit[] = [];
  /**
   Ids of edits left out.
   */
  const overlapped: string[] = [];
  for (const edit of edits) {
    if ((edit.start < 0) || (edit.end > english.length)
      || (edit.end <= edit.start)) {
      throw new ConsistencyInvariantError({ invariant:
        `edit ${edit.id} spans ${String(edit.start,)} to ${String(edit.end,)} in an English whose length is ${
          String(english.length,)
        }, and a span located in that English lies inside it and is not empty`, },);
    }
    if (kept.some(function overlaps(other,): boolean {
      return (edit.start < other.end) && (other.start < edit.end);
    },))
      overlapped.push(edit.id,);
    else
      kept.push(edit,);
  }
  /**
   Kept edits in the order their spans stand in the text.
   */
  const fromTheStart = kept.toSorted(function earlierFirst(
    a,
    b,
  ): number {
    return a.start - b.start;
  },);
  /**
   Pieces of the edited text: the English between edits, and each edit's words.
   */
  const pieces: string[] = [];
  /**
   Offset in the English up to which the pieces hold its text.
   */
  const cursor = { at: 0, };
  for (const edit of fromTheStart) {
    pieces.push(
      english.slice(
        cursor.at,
        edit.start,
      ),
      edit.text,
    );
    cursor.at = edit.end;
  }
  pieces.push(english.slice(cursor.at,),);
  return {
    text: pieces.join('',),
    applied: kept.map(function idOf(edit,): string {
      return edit.id;
    },),
    overlapped,
  };
}

//endregion Consistency splice
