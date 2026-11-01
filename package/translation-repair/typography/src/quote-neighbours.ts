import {
  codePointAt,
  codePointBefore,
  isAsciiDigit,
  isAsciiDigits,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { isCasedLetter, } from './cased-letters.ts';

//region Quote neighbours
// WHAT STANDS BESIDE A STRAIGHT QUOTE, read by whole code point: whether a
// letter or digit binds it into a word, and whether an inline span closes
// before it. Shared by the apostrophe reading in `restore-typography.ts` and
// the nested quotation reading in `nested-single-quotes.ts`.
//
// BEFORE A QUOTE, A COMBINING MARK IS READ WITH the character it sits on
// (audit area six, 2026-09-28; ledger B18): `café's` written with a combining
// acute has the mark, not the `e`, beside the quote, and a mark is no cased
// letter, so the composed spelling curled and the combining one stayed
// straight on a curly page.

/**
 Whether a character can sit beside an apostrophe inside one word.

 Restricted to letters and digits so a straight quote acting as a QUOTE, which
 has a space or punctuation on at least one side, is never mistaken for an
 apostrophe inside a contraction.

 @param character - code point after the quote, or the combining sequence
 before it (`sequenceBefore`), whose first code point is its base; empty at
 a text boundary

 @returns Whether it binds the quote into a word

 @example
 ```ts
 const binds = bindsWord({ character: 't', },);
 ```
 */
export function bindsWord({ character, }: { readonly character: string; },): boolean {
  /**
   First code point: the character itself, or a sequence's base.
   */
  const base = character.codePointAt(0,);
  if (base === undefined)
    return false;

  /**
   That code point as a character. A cased letter by general category, not by
   case mapping (`cased-letters.ts`): the mathematical script letters the
   corpus writes handles in are cased letters with no case mapping, and Han
   is neither cased nor a word an apostrophe binds into (class ninety-six).
   */
  const first = String.fromCodePoint(base,);
  return isCasedLetter({ character: first, },) || isAsciiDigit({ character: first, },);
}

// CLASS NINETY-SIX (mikaela_khara, 2026-09-23). The archive writes a handle
// in mathematical script, every letter a surrogate pair, and the bench wrote
// its possessive with a straight apostrophe. The neighbours of a quote were
// read by UTF-16 unit, so the unit before the apostrophe was the low half of
// the last letter, which is no letter at all, and the apostrophe stayed
// straight on a curly page. THE NEIGHBOURS ARE WHOLE CODE POINTS, read by
// `code-points.ts`.

/* oxlint-disable no-restricted-syntax/no-regex -- the input is one code point, anchored at both ends with no quantifier, so the test is bounded and cannot backtrack; the Unicode general category has no string API */
/**
 Whether one code point is a combining mark (general category M, variation
 selectors included), which belongs to the character before it rather than
 standing as one.
 */
const MARK_CODE_POINT = /^\p{M}$/u;
/* oxlint-enable no-restricted-syntax/no-regex */

/**
 Offset where the run of combining marks ending at an offset starts: the
 offset itself when no mark ends there.

 @param text - text being read

 @param at - offset the marks end at

 @returns Offset just after the code point the marks sit on

 @example
 ```ts
 const start = marksStart({ text: 'cafe\u{0301}\'s', at: 5, },); // 4
 ```
 */
function marksStart({
  text,
  at,
}: {
  readonly text: string;
  readonly at: number;
},): number {
  /**
   Offset moved back past each mark.
   */
  let start = at;
  /**
   Code point ending at that offset.
   */
  let before = codePointBefore({
    text,
    at: start,
  },);
  while (MARK_CODE_POINT.test(before,)) {
    start -= before.length;
    before = codePointBefore({
      text,
      at: start,
    },);
  }
  return start;
}

/**
 The combining character sequence ending just before an offset: the whole
 code point there, with every combining mark before the offset read back to
 the code point they sit on, so a letter written with a combining accent
 stands before a quote as its composed spelling does.

 WHOLE, NOT THE BASE ALONE: a mark on a space is a standalone accent, not
 a space, and a mark on a span's closing `)` makes it no closer, so each
 test reads the sequence and only `bindsWord` reads its base.

 @param text - text being read

 @param at - offset of the quote

 @returns The base code point and its marks, empty at the text's start;
 marks alone where nothing but marks precedes them

 @example
 ```ts
 const before = sequenceBefore({ text: 'cafe\u{0301}\'s', at: 5, },); // 'e' plus U+0301
 ```
 */
export function sequenceBefore({
  text,
  at,
}: {
  readonly text: string;
  readonly at: number;
},): string {
  /**
   Offset where the sequence's marks start.
   */
  const start = marksStart({
    text,
    at,
  },);
  /**
   Code point the marks sit on, empty at the text's start.
   */
  const base = codePointBefore({
    text,
    at: start,
  },);
  return text.slice(
    start - base.length,
    at,
  );
}

/**
 Characters that close an inline span the prose mask leaves in place: a
 link's `)`, a reference's `]`, an emphasis or strong `*` and `_`, a
 strikethrough `~`, a code span's backtick and a tag's `>`.

 CLASS SEVENTY (mikaela_khara, 2026-09-19). The archive's closing line reads
 `[name](url)’s chronicle`; the page shipped `)'s` straight on a curly page
 because only a letter or digit before the quote bound it into a word, and a
 possessive after a link, an emphasis span or a tag has a delimiter there.
 */
const SPAN_CLOSERS: ReadonlySet<string> = new Set([
  ')',
  ']',
  '*',
  '_',
  '~',
  '`',
  '>',
],);

/**
 Whether a character closes an inline span, so a possessive quote after it
 belongs to the word the span carries.

 @param character - character before the quote, empty at a text boundary

 @returns Whether a span ends there

 @example
 ```ts
 const closes = closesSpan({ character: ')', },);
 ```
 */
export function closesSpan({ character, }: { readonly character: string; },): boolean {
  return SPAN_CLOSERS.has(character,);
}

// A LEADING ELISION is a straight mark that stands for the letters a word dropped at its start (`'tis`, `'em`, `rock 'n' roll`,
// `'99`, `'90s`). The `'n'` counts only with a word directly before and after it, each across one space (`rock 'n' roll`, `fish 'n'
// chips`), so `The letters 'a', 'n' and 'e'` and `He said 'n'` pair as quotations; `the letter 'n' is silent` reads as the unit
// all the same, an accepted limit, as no test of the neighbours tells it from `rock 'n' roll`. Its curly form is the closing single mark, the apostrophe, never the opening single mark, WHEN NO LATER STRAIGHT
// MARK CLOSES IT as a quotation: an elision written as an opening mark is a wrong mark in the shipped page (`‘tis fine`). A
// mark that a later one does close begins a quotation (`'Tis the season,' she said`, `'Round things are best'`), and the pair
// curls as a quotation, so the closing mark does not stay straight on a curly page. The `'n'` of `rock 'n' roll` is two
// apostrophes that never pair. THE LIST IS CLOSED and matched as a whole word, case
// folded, so a word that only begins like one (`'tisane'`, `'emperor'`, `'rounders'`, `'1990s'`) is not an elision and still
// pairs as a quotation.

/**
 The words, lower case, that a straight mark before them stands in for dropped letters of: it is, it was, until, them, because
 and around.
 */
const ELISION_WORDS: ReadonlySet<string> = new Set([
  'tis',
  'twas',
  'til',
  'em',
  'cause',
  'round',
],);

/**
 The length of the longest word of `ELISION_WORDS`, so a word read after the mark is cut one unit past it.
 */
const LONGEST_ELISION = 5;

/**
 The length of a decade written as an elision: two digits and an `s`.
 */
const DECADE_LENGTH = 3;

/**
 The length of a year written as an elision: two digits and nothing after them.
 */
const YEAR_LENGTH = 2;

/**
 Where the space after the `'n'` stands, counted from the first mark: the letter, the second mark, then the space.
 */
const SPACE_AFTER_UNIT = 3;

/**
 Where the word after the `'n'` starts, counted from the first mark: one past the space after the unit.
 */
const WORD_AFTER_UNIT = 4;

/**
 Whether a straight mark opens the `'n'` of `rock 'n' roll`: the letter, a second straight mark, a word directly before the
 first mark and a word directly after the second, each across one space. A quoted letter between two words (`the letter 'n'
 is silent`) reads the same, an accepted limit, since no test of the neighbours tells the two apart.

 @param text - text being read

 @param at - offset of the first straight mark, whose letter follows it

 @returns Whether a word stands before the first mark and after the second, each across one space

 @example
 ```ts
 const unit = standsBetweenWords({ text: "Rock 'n' roll.", at: 5, },); // true
 ```
 */
function standsBetweenWords(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): boolean {
  if (text.charAt(at + 2,) !== '\'')
    return false;
  if (text.charAt(at - 1,) !== ' ')
    return false;
  if (text.charAt(at + SPACE_AFTER_UNIT,) !== ' ')
    return false;
  /**
   Whether a word stands before the space that precedes the first mark.
   */
  const wordBefore = bindsWord({ character: sequenceBefore({
    text,
    at: at - 1,
  },), },);

  /**
   Whether a word stands after the space that follows the second mark.
   */
  const wordAfter = bindsWord({ character: codePointAt({
    text,
    at: at + WORD_AFTER_UNIT,
  },), },);
  return wordBefore && wordAfter;
}

/**
 Whether the straight mark at an offset is a leading elision by the word after it: one of the closed list, the `n` of `'n'`
 between two words (`standsBetweenWords`), a year of two digits (`'99`) or a decade (two digits and an `s`). Only what
 follows the mark is asked, and for the `'n'` the word before it; the caller asks what else stands before it.

 @param text - text being read

 @param at - offset of the straight mark

 @returns Whether the whole word after the mark is an elision

 @example
 ```ts
 const elides = isLeadingElision({ text: "It was 'tis fine.", at: 7, },); // true
 ```
 */
export function isLeadingElision(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): boolean {
  /**
   Offset where the word after the mark starts.
   */
  const wordStart = at + 1;
  /**
   How many units of the word after the mark are read, whole code points, cut one unit past the longest elision.
   */
  const reading = { length: 0, };
  while (reading.length <= LONGEST_ELISION) {
    /**
     The code point at the end of the word read so far.
     */
    const character = codePointAt({
      text,
      at: wordStart + reading.length,
    },);
    if (!bindsWord({ character, },))
      break;
    reading.length += character.length;
  }
  /**
   The word after the mark with its case folded.
   */
  const folded = text.slice(
    wordStart,
    wordStart + reading.length,
  )
    .toLowerCase();
  if (ELISION_WORDS.has(folded,))
    return true;
  if (folded === 'n')
    return standsBetweenWords({
      text,
      at,
    },);
  // A year of two digits (`'99`) and a decade (`'90s`) are both written with the mark for the dropped century.
  if (folded.length === YEAR_LENGTH)
    return isAsciiDigits({ text: folded, },);
  return (folded.length === DECADE_LENGTH)
    && isAsciiDigits({
      text: folded.slice(
        0,
        DECADE_LENGTH - 1,
      ),
    },)
    && folded.endsWith('s',);
}

/**
 Whether the straight mark at an offset opens the `'n'` of `rock 'n' roll`: a leading elision whose letter is followed by a
 second straight mark, with a word before the first and after the second (`standsBetweenWords`), so the two marks both stand
 for dropped letters and are never the marks of one quotation.

 @param text - text being read

 @param at - offset of the straight mark

 @returns Whether the word after the mark is `n` and a straight mark follows it

 @example
 ```ts
 const pair = isElisionPair({ text: "Rock 'n' roll.", at: 5, },); // true
 ```
 */
export function isElisionPair(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): boolean {
  /**
   The letter after the mark, in lower case.
   */
  const letter = text.charAt(at + 1,)
    .toLowerCase();
  return (letter === 'n') && isLeadingElision({
    text,
    at,
  },);
}

/**
 Whether the straight mark at an offset is the second mark of the `'n'` of `rock 'n' roll`: it follows an `n` that a straight mark
 opens (`isElisionPair`) and is the straight mark that opening one looks for. The unit is one: neither of its marks opens or
 closes a quotation.

 @param text - text being read

 @param at - offset of the straight mark

 @returns Whether the mark closes the unit `'n'`

 @example
 ```ts
 const end = isElisionPairEnd({ text: "Rock 'n' roll.", at: 7, },); // true
 ```
 */
export function isElisionPairEnd(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): boolean {
  return (at >= 2)
    && (text.charAt(at,) === '\'')
    && isElisionPair({
      text,
      at: at - 2,
    },)
    && (text.charAt(at - 2,) === '\'');
}

/**
 The punctuation after which a straight single mark with no word after it is the closing mark of a clause or an apostrophe and
 nothing else: the stops and pauses, the dashes, a closing double quotation mark or guillemet and a closing brace.
 */
const PUNCTUATION_BEFORE_CLOSING: ReadonlySet<string> = new Set(Array.from(',.;:!?…-–—”’»›"}',),);

/**
 Whether the character before a straight single mark with no word after it is punctuation that a closing single mark follows: a
 stop, a pause, a dash or a closing quotation mark.

 @param character - the character before the mark, empty at a text boundary

 @returns Whether it is such punctuation

 @example
 ```ts
 const follows = precedesClosingMark({ character: ',', },); // true
 ```
 */
export function precedesClosingMark({ character, }: { readonly character: string; },): boolean {
  return PUNCTUATION_BEFORE_CLOSING.has(character,);
}

//endregion Quote neighbours
