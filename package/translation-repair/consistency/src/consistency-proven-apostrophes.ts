import { partsMarkFromWord, } from './consistency-mark-neighbours.ts';
import { continuesLatinWordOrCompatibilityForm, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Consistency proven apostrophes
// WHICH SINGLE MARKS OF AN ENGLISH ARE APOSTROPHES BEYOND DOUBT, so that a splice
// may move them with their word. Three characters can be an apostrophe (`'`, `’`,
// and the modifier letter apostrophe U+02BC), and the character alone does not say
// whether one is an apostrophe (`parents’`, `’round`) or half of a quotation
// (`‘Mio’`). This file never guesses: a mark is PROVEN an apostrophe only when no
// other quotation-like character of the whole English could have been its partner,
// and a mark wrongly proven is a quotation mark deleted from a page.
//
// THE RULE (decisions D3, D5 and D9 of the independent-oracle round). An apostrophe-like mark is ATTACHED when a
// letter or a digit stands on both sides of it (`don't`), TRAILING when one stands directly before it and what stands
// directly after it PARTS it from a word, LEADING when one stands directly after it and what stands directly before it
// parts it, and FREE in every other case. A character PARTS a mark when it is white space, the edge of the text, or
// punctuation that is no single mark (`consistency-mark-neighbours.ts`). So a letter or a digit of another script, a
// symbol (`™`, `+`), a number that is no decimal digit (`²`, `①`) or a pictograph (`🐱`) beside the mark makes it FREE: it is
// no apostrophe of a Latin word and no edge of a quotation either (`cats’猫`, `猫’rock`, `the 🐱's bowl`, `cat™’s`), the
// safe direction, since a free mark is read as a quotation mark would be and never deleted as an apostrophe. D5: a single
// mark directly beside another single mark (`''`, `’’`, `’ʼ`, and the opening, low, reversed and angle single marks)
// reads FREE, both of them, so a doubled mark is a pair of quotation marks and no apostrophe. D9 is an accepted limit: a
// bare leading mark that no later mark could close reads as an elision (`'tis`) and is proven, though a reader may see an
// unclosed quotation. "A letter or a digit" is ONE TEST FOR EVERY READING OF A SINGLE
// MARK in the consistency pass (this proof, the form reader, the loose-mark reader, the joints that cut
// a word, the closing and edge marks, the counter's class of an edge): a Latin letter with its accent, composed or
// combining, or an ASCII digit, or a whole character that is itself a letter or a decimal digit and whose NFKC form is
// those alone (a mathematical script letter, a fullwidth letter or digit, a ligature;
// `continuesLatinWordOrCompatibilityForm`, `latin-letters.ts`). The rest of the package reads a word with
// `continuesLatinWord`, as written. A trailing mark is a proven apostrophe when
// no mark EARLIER IN THE ENGLISH could have opened a quotation it closes, and a
// leading mark is one when no mark LATER IN THE ENGLISH could close a quotation it
// opens. Which marks could: the opening curly single mark `‘`, the low single mark
// `‚`, the reversed single mark `‛`, both angle marks `‹` `›` and a backtick, each
// wherever it stands (this proof takes none of them for an apostrophe, so a glued `said‘fill`
// and a `cat‘s` between letters are each a possible partner; the locator reads the `‘` of
// `cat‘s` as a joint, so words that run into it are not found, and the loose-mark reader
// counts it as a quotation mark, never as part of a word: three readers, one meaning, a
// quotation mark that lacks its space), and a straight or closing
// curly mark or U+02BC that is leading or free for an opener and trailing or free for a
// closer. An attached mark is a contraction and stands neither for nor against. D6: a leading mark is a possible
// opener for every trailing mark after it, and a trailing mark a possible closer for every leading mark before it, so
// neither is proven by the other's absence (`He said 'cats and the dogs' bowl` proves neither). The
// opening curly single mark is never itself a proven apostrophe.
//
// A MARK AFTER CLOSING MARKUP IS ATTACHED TOO, when the caller says so. In `**cat**'s` the prose
// has blanked the emphasis to spaces, and a reader of the prose alone sees a space before the
// mark and takes it for the opener of a quotation. The joint rule that cuts a word
// (`consistency-word-joints.ts`) reads the same mark as joining the letter after it, across
// emphasis, a closing double quotation mark, a closing bracket and a link's destination, and
// the proof must not read one mark two ways. The caller names those marks in `attached`
// (`joinedMarksOf` in `consistency-joined-marks.ts`, read on the English as written, not on the
// prose where the closing marks are gone) and they are read like `don't`. A mark after an
// OPENING mark (`*'Tis*`) is no such joint and stays what its neighbours make it.
//
// A MARK PAST MARKUP IS AN EDGE MARK, ON THE TRAILING SIDE ONLY (decisions D2 and D4). In `*kittens*’ bowl` the mark
// comes after a run of closing markup and the character before the run is a Latin letter, so the word is `kittens`, the
// markup is no part of it and the mark is its trailing edge (a plural possessive when no opener is earlier). The run holds
// closing delimiters of emphasis, strong emphasis and strikethrough by the parse, and a link's closing `](…)`; a `*`, `_` or
// `~` that the parse makes no delimiter is a character of the text and ends the run (`猫_cats_’`, `'_cats_s`). A closing
// double quotation mark or a bracket is not in the run: after `"ok"'` or `(here)’` a single mark is most often a closing
// quotation mark and is read where it stands. There is no leading mirror: `’**cats**` and `'_cats_` read FREE. The caller
// names the marks that end a run in `pastMarkup` (`marksPastMarkupOf`, `consistency-marks-past-markup.ts`). Only a FREE
// mark is re-read: a mark with a letter beside it keeps what the letters say, a joint stays a joint, a mark with a single
// mark beside it stays free, and a mark that the character after it does not part from a word stays free. Every reader of
// the consistency pass takes the same reading from `markReadingsOf`, so the proof, the form reader and the counter agree
// that the mark is an edge mark.
//
// THE WHOLE ENGLISH, NOT THE PARAGRAPH. A quotation runs over paragraphs: a closing
// `’` after `kittens` whose `‘` opened three paragraphs earlier is the end of a
// quotation, and a reading that stops at the blank line takes it for a plural
// possessive and deletes it. So the partner is looked for in the text the function is
// handed, whole. The caller hands the PROSE of the English, the text with the stretches
// the page does not read as words blanked (`consistency-prose-text.ts`): code, the
// markup and string attributes of a component, a comment, the destination of a link.
// A mark in such a stretch is no partner, and a backtick that opens no code is one. The
// text a caller hands is the English of the WHOLE PAGE, every slice of it joined
// (`consistency-page-english.ts`), so a quotation that runs over a heading into another
// slice is seen from either: the pass holds the English of every slice and reads the
// page, and a caller that holds one English alone gets the proof of that English alone.
//
// WHAT THE CORPUS HOLDS (counted 2026-10-14 on the final build with `count-single.mjs`, a probe kept
// in the scratch directory of the n144 round, over the 92 English pages of the pinned corpus, each read
// as the pass reads it, with the stretches the page does not read as prose blanked, see
// `package/translation-repair/doc/design-commitments-consistency-pass.md`, the bullet "The
// consistency pass carries words and leaves every mark where it was"): 1,190 marks `'` or `’`
// (65 + 1,125; U+02BC stands in no page) are apostrophe-like marks the reading classifies: 1,148 attached to a
// letter or a digit on both sides, 33 trailing, 1 leading, 4 joints (a mark after closing markup that joins
// the word before the markup to the letter after it; defined in `consistency-word-joints.ts`) and 4 free
// (1,148 + 33 + 1 + 4 + 4 = 1,190). A count that reads the 4 joints as leading marks, as the paragraph reading of
// the 2026-10-13 measurement did, finds 5 leading. The other single quotation characters of the prose are
// 9 `‘` (1,199 with them) and 1 backtick that opens no code (`∂z/∂x`` in one page; 1,200 with it), and the 10
// are no apostrophe: no reading classifies them. So the 1,200 characters of the earlier count and the
// 1,190 marks of this one are the same corpus counted to a different boundary, and differ by exactly those 10.
// One prime `′` of the prose is no quotation character and is in none of the counts. The paragraph reading of the 2026-10-13 measurement proved 28 of the 33
// trailing marks and all 5 leading marks, every trailing one a plural possessive
// (`parents’`, `friends’`, `others’`) and every leading one a dropped letter or the
// `s` after emphasis or after a link. The whole-English reading proves 27 of those 28 and
// 1 of the 5, and no mark besides them. The trailing one that drops out is `her friends’
// excellent ideas`, where a backtick of the same page stands earlier in prose. The 4
// leading ones that drop out are the `’s` after `*Yorushika*`, after `*Arcaea*` and after
// the closing bracket of two links, which the joint rule that cuts a word reads as joining
// the letter after it (`joinedMarksOf`, `consistency-joined-marks.ts`): they are attached,
// like `don't`, and the one leading mark left is a dropped letter. The same pages cut into
// the slices the package's own slicer makes (1,329 slices) prove 28 and 1 when each slice
// is read alone, the mark with the backtick among them, since its backtick stands in an
// earlier slice; read as the pass now reads them, joined as one page, they prove 27 and
// 1, the same marks as the whole-file reading, and that one mark is the only difference.
// The counts of this paragraph were taken with a letter of any script beside the mark; under the test
// of a Latin letter as written (`continuesLatinWord`: a Latin letter with its accent or an ASCII digit) the
// same pages prove 27 trailing and 2 leading, and of the 1,199 marks `'`, `’`, U+02BC and `‘` of the prose
// exactly one changes its kind: the `’s` of a name written in mathematical script letters (`𝓠𝓾𝓪𝓷’s`),
// whose letters are no Latin letters as written, so the mark that was attached reads as leading. The
// neighbour test of the pass (`continuesLatinWordOrCompatibilityForm`) reads those letters through their
// NFKC form, so the mark is attached again and the pages prove 27 trailing and 1 leading.
// Read as plain text, with
// the strings of a component not blanked, 6 more of the 28 trailing marks and one
// leading mark drop out, as 5 pages hold a photo list whose straight quotes would then
// count as partners; that is the cost of reading a page as text where a parse is
// refused. Every mark the whole-English reading proves is one of the 33 the
// 2026-10-13 count read as apostrophes.
//
// ONE LINEAR PASS. The text is read once, a code point at a time, with two running
// facts: where the first mark that could open a quotation stands, and where the last
// mark that could close one stands. A trailing mark is proven the moment it is read
// when no opener has been seen; a leading mark is held until the pass ends and is proven
// when it stands after the last closer. The work is a constant number of reads for
// each character and one entry for each leading mark, never a scan of the text for each
// mark.

/**
 The proven apostrophes of an English, by where they stand: the ones that end a
 word (a plural possessive) and the ones that start a word (a dropped letter).
 */
export type ProvenApostrophes = {
  /**
   Offsets in the English of the proven trailing marks.
   */
  readonly trailing: ReadonlySet<number>;

  /**
   Offsets in the English of the proven leading marks.
   */
  readonly leading: ReadonlySet<number>;
};

/**
 The marks that can be a proven apostrophe: the straight apostrophe, the closing
 curly single mark and the modifier letter apostrophe.
 */
const APOSTROPHE_LIKE: ReadonlySet<string> = new Set([
  '\'',
  '’',
  '\u{02BC}',
],);

/**
 The quotation-like characters that are no apostrophe in any use and so pair with
 an apostrophe-like mark wherever they stand, glued to a letter or not: the opening
 curly single mark, the low and the reversed single mark, both angle marks and a
 backtick.
 */
const PARTNER_ANYWHERE: ReadonlySet<string> = new Set([
  '‘',
  '\u{201A}',
  '\u{201B}',
  '\u{2039}',
  '\u{203A}',
  '`',
],);

/**
 How an apostrophe-like mark stands among its neighbours.
 */
type MarkKind = 'attached' | 'trailing' | 'leading' | 'free';

/**
 How a mark reads on the page: as `MarkKind` says, or `joint` for a mark the caller names as joined to the word after it
 across closing markup (`**cat**'s`), which reads as attached but is told apart, since what makes it attached is the
 markup before it and not a letter.
 */
export type MarkReading = MarkKind | 'joint';

/**
 One mark read: its offset in the prose and how it reads.
 */
type ReadMark = readonly [
  number,
  MarkReading,
];

/**
 The marks the caller reads as standing past markup, by their offset in the prose: the ones that end a reading run of closing
 delimiters of emphasis (by the parse) or a link's destination after a letter or a digit (`*kittens*’`). There is no leading
 side: a mark before opening markup (`’*tis*`) is read where it stands. `marksPastMarkupOf`
 (`consistency-marks-past-markup.ts`) reads them on the English as written, where the markup is not yet blanked.
 */
export type MarksPastMarkup = {
  /**
   Offsets of the marks that end a reading run that a letter or a digit stands before.
   */
  readonly trailing: ReadonlySet<number>;
};

/**
 The marks named when the caller names none.
 */
const NO_MARKS_PAST_MARKUP: MarksPastMarkup = { trailing: new Set(), };

/**
 How a mark stands, from the character directly on each side of it. Letters on both sides make it attached. A letter on one
 side makes it trailing (letter before) or leading (letter after) only when the other side parts it from a word (white
 space, punctuation other than a single mark, the edge of the text); a letter of another script, a symbol, a pictograph or
 a single mark on the other side makes it free. A single mark never parts, so two single marks side by side read free, both
 of them, with a letter on either side of the pair or none.

 @param before - the character directly before the mark, a space at the start of the text

 @param after - the character directly after it, empty at the end of the text

 @returns `attached`, `trailing`, `leading` or `free`

 @example
 ```ts
 const kind = kindOf({ before: 's', after: ' ', },); // 'trailing'
 ```
 */
function kindOf({
  before,
  after,
}: {
  readonly before: string;
  readonly after: string;
},): MarkKind {
  /**
   Whether a letter or a digit of a Latin word stands directly before the mark.
   */
  const wordBefore = continuesLatinWordOrCompatibilityForm({ character: before, },);
  /**
   Whether one stands directly after it.
   */
  const wordAfter = continuesLatinWordOrCompatibilityForm({ character: after, },);
  if (wordBefore && wordAfter)
    return 'attached';
  if (wordBefore)
    return partsMarkFromWord({ character: after, },) ? 'trailing' : 'free';
  if (wordAfter)
    return partsMarkFromWord({ character: before, },) ? 'leading' : 'free';
  return 'free';
}

/**
 How one apostrophe-like mark reads: `joint` for a mark the caller names as joined to the word after it, else as the letters
 beside it say. A FREE mark that the caller names as standing past markup reads as `trailing`, since the markup is no part of
 the word and a letter or a digit stands on its far side; a mark before opening markup has no such reading and stays free. A
 mark that a letter or a digit stands directly beside keeps what those letters say.

 @param text - the prose of the English, whole

 @param at - offset of the mark

 @param character - the mark

 @param previous - the character just before the mark, a space at the start of the text

 @param attached - offsets of the marks the caller reads as joined to the word after them

 @param pastMarkup - the marks the caller reads as standing past markup

 @returns The reading of the mark

 @example
 ```ts
 const reading = markReadingOf({ text: 'cats’ bowl', at: 4, character: '’', previous: 's', attached: new Set(), pastMarkup: NO_MARKS_PAST_MARKUP, },); // 'trailing'
 ```
 */
function markReadingOf(
  {
    text,
    at,
    character,
    previous,
    attached,
    pastMarkup,
  }: {
    readonly text: string;
    readonly at: number;
    readonly character: string;
    readonly previous: string;
    readonly attached: ReadonlySet<number>;
    readonly pastMarkup: MarksPastMarkup;
  },
): MarkReading {
  if (attached.has(at,))
    return 'joint';
  /**
   The code point directly after the mark, none at the end of the text.
   */
  const afterPoint = text.codePointAt(at + character.length,);
  /**
   The character directly after the mark, empty at the end of the text.
   */
  const after = (afterPoint === undefined) ? '' : String.fromCodePoint(afterPoint,);
  /**
   How the mark stands among the characters directly beside it.
   */
  const kind = kindOf({
    before: previous,
    after,
  },);
  if (kind !== 'free')
    return kind;
  /**
   The offsets of the marks that end a reading run.
   */
  const { trailing, } = pastMarkup;
  if (!trailing.has(at,))
    return 'free';
  // A free mark that ends a reading run reads as the trailing edge of the word before the run, when what stands after it
  // parts it from a word (not a single mark, a letter of another script, a symbol); none reads as leading.
  return partsMarkFromWord({ character: after, },) ? 'trailing' : 'free';
}

/**
 The proven apostrophes of an English: the trailing marks no earlier opener
 could pair with, and the leading marks no later closer could. The opening curly
 single mark is never proven.

 @param text - the prose of the English, whole, as a caller reads it

 @param attached - offsets of marks the caller reads as part of the word after them whatever stands before them (the `'`
 of `**cat**'s`, where the prose has blanked the emphasis), which stand neither for nor against a quotation and are never
 proven; none by default

 @param pastMarkup - the marks the caller reads as standing past markup (`*kittens*’`): a free one named reads as a trailing
 mark, so it can be proven, and a trailing one that is not proven still opens nothing; none by default

 @returns The offsets of the proven marks, trailing and leading apart

 @example
 ```ts
 const { trailing, } = provenApostrophesOf({ text: 'The parents’ bowl', },); // trailing holds 11
 ```
 */
export function provenApostrophesOf(
  {
    text,
    attached = new Set(),
    pastMarkup = NO_MARKS_PAST_MARKUP,
  }: {
    readonly text: string;
    readonly attached?: ReadonlySet<number>;
    readonly pastMarkup?: MarksPastMarkup;
  },
): ProvenApostrophes {
  /**
   Offsets of the proven trailing marks, mutated only inside this function.
   */
  const trailing = new Set<number>();
  /**
   The leading marks held until the last closer is known, mutated only inside this function.
   */
  const heldLeading: number[] = [];
  /**
   The running facts of the pass: whether a mark that could open a quotation has been read, and where the last mark that
   could close one stands, none until one has been read.
   */
  const seen = {
    opener: false,
    lastCloser: -1,
  };
  /**
   Where the reading is: the offset and the character just read, a space standing before the first character.
   */
  const reading = {
    at: 0,
    previous: ' ',
  };
  while (reading.at < text.length) {
    /**
     The code point at the offset, never absent inside the text.
     */
    const codePoint = text.codePointAt(reading.at,) ?? 0;
    /**
     The character it is.
     */
    const character = String.fromCodePoint(codePoint,);
    if (PARTNER_ANYWHERE.has(character,)) {
      seen.opener = true;
      seen.lastCloser = reading.at;
    }
    else if (APOSTROPHE_LIKE.has(character,)) {
      /**
       How the mark reads, a joint counting as attached here.
       */
      const markReading = markReadingOf({
        text,
        at: reading.at,
        character,
        previous: reading.previous,
        attached,
        pastMarkup,
      },);
      /**
       How the mark stands among its neighbours.
       */
      const kind = (markReading === 'joint') ? 'attached' : markReading;
      if ((kind === 'trailing') && (!seen.opener))
        trailing.add(reading.at,);
      if (kind === 'leading')
        heldLeading.push(reading.at,);
      if ((kind === 'leading') || (kind === 'free'))
        seen.opener = true;
      if ((kind === 'trailing') || (kind === 'free'))
        seen.lastCloser = reading.at;
    }
    reading.previous = character;
    reading.at += character.length;
  }
  return {
    trailing,
    leading: new Set(heldLeading.filter(function standsAfterLastCloser(at,): boolean {
      return at > seen.lastCloser;
    },),),
  };
}

/**
 How every apostrophe-like mark of an English reads: attached, joined after closing markup, trailing, leading or free, from
 the letters directly beside it in the prose and the marks the caller names as joined. The opening curly single mark reads
 as none of them: it is a partner wherever it stands.

 @param text - the prose of the English, whole, as a caller reads it

 @param attached - offsets of the marks the caller reads as joined to the word after them across closing markup; none by default

 @param pastMarkup - the marks the caller reads as standing past markup; a free one named reads as a trailing mark; none by
 default

 @returns The reading of each apostrophe-like mark by its offset, in the order the marks stand

 @example
 ```ts
 const readings = markReadingsOf({ text: 'The parents’ bowl', },); // 11 reads as trailing
 ```
 */
export function markReadingsOf(
  {
    text,
    attached = new Set(),
    pastMarkup = NO_MARKS_PAST_MARKUP,
  }: {
    readonly text: string;
    readonly attached?: ReadonlySet<number>;
    readonly pastMarkup?: MarksPastMarkup;
  },
): ReadonlyMap<number, MarkReading> {
  /**
   The readings as offset and reading, in the order the marks stand, mutated only inside this function. An offset is read
   once, so no mark is written over another.
   */
  const readings: ReadMark[] = [];
  /**
   Where the reading is: the offset and the character just read, a space standing before the first character.
   */
  const reading = {
    at: 0,
    previous: ' ',
  };
  while (reading.at < text.length) {
    /**
     The character at the offset, never absent inside the text.
     */
    const character = String.fromCodePoint(text.codePointAt(reading.at,) ?? 0,);
    if (APOSTROPHE_LIKE.has(character,)) {
      readings.push([
        reading.at,
        markReadingOf({
          text,
          at: reading.at,
          character,
          previous: reading.previous,
          attached,
          pastMarkup,
        },),
      ],);
    }
    reading.previous = character;
    reading.at += character.length;
  }
  return new Map(readings,);
}

//endregion Consistency proven apostrophes
