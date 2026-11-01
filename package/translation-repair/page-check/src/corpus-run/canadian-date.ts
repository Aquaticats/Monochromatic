import {
  codePointAt,
  codePointBefore,
  isAsciiDigit,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { isCasedLetter, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  continuesWord,
  isDateSpace,
} from './canadian-date-parts.ts';
import {
  readMonthFirst,
  readYearFirst,
} from './canadian-date-read-leading.ts';
import {
  readDayFirst,
  readRange,
} from './canadian-date-read.ts';
import {
  type DateReading,
  NO_PART,
  type NoPart,
} from './canadian-date-words.ts';
import {
  inProse,
  type ProtectedRange,
} from '@monochromatic-dev/translation-repair-glossary/ts';

//region Canadian date
// CLASS ONE HUNDRED THIRTY-FOUR (hulicaijia19, 2026-09-25): the page is
// Canadian English, which writes a date month first ("April 29", "March 13,
// 2024"), yet the archive's "29th April" and "On 4 May" stood beside the
// bench's "March 13" on one page. Every date in the text's prose is read and
// written month first, its month in full, with no ordinal suffix, and with its
// year set off by commas on both sides.
//
// The audit of 2026-09-26 (ledger K1, K2, K4, K8) found the first pass wrote
// "March 13, 2024 in a box" with no closing comma, left a day-first date
// after "until", "to" or a dash, split "1st\nto 3rd June", mixed the orders of
// "3 June to 5 July", left "December 29th", "2023 Feb 25th" and "4 Sept
// 2024", wrote "the May 4" from "the 4th May", and rewrote "5 May beetles"
// and "4 May2024". The readers in `canadian-date-read.ts` and
// `canadian-date-read-leading.ts` read each shape whole instead.

/**
 Characters that may stand right before a date's first digit in prose.
 */
const DAY_OPENERS: ReadonlySet<string> = new Set([
  ' ',
  '\u00A0',
  '\n',
  '\t',
  '(',
  '"',
  '“',
  '‘',
  '\'',
  '*',
  '_',
  '[',
  '>',
  '–',
  '—',
],);

/**
 Article a day-first date may carry before its ordinal day, in either case.
 */
const ARTICLES: ReadonlySet<string> = new Set([
  'the',
  'The',
],);

/**
 Length of the article.
 */
const ARTICLE_LENGTH = 3;

/**
 Readers a date opening with a digit is tried against, in order: a year
 opens a year-first date, and a day a range before it opens a lone date.
 */
const DIGIT_READERS: readonly ((place: {
  readonly text: string;
  readonly at: number;
},) => DateReading | NoPart)[] = [
  readYearFirst,
  readRange,
  readDayFirst,
];

/**
 One date rewritten, with the span it covered.
 */
export type DateRewrite = {
  readonly start: number;
  readonly end: number;
  readonly from: string;
  readonly to: string;
};

/**
 Where the scan stands: the offset under the cursor and the end of the last
 date read, from which a bare hyphen may open the next ("2 June-3 July").
 */
type ScanState = {
  at: number;
  lastEnd: number;
};

/**
 Whether a digit at one offset may open a date: at the text's start, after a
 space, an opening mark, a dash or an emoji, or after a hyphen that directly
 follows the date read before it.

 AN EMOJI, NOT EVERY CHARACTER BEYOND THE FIRST PLANE (ledger B21): the test
 read the unit before the digit and took any low surrogate as an emoji, so a
 date glued to a Deseret letter opened where one glued to a Latin letter did
 not. The whole character before the digit is read, and one beyond the first
 plane opens a date only where it continues no word.

 @param text - text under scan

 @param state - where the scan stands, the digit under the cursor

 @returns Whether a date may start there

 @example
 ```ts
 opensDate({ text: 'napped—4 May', state: { at: 7, lastEnd: -1, }, },); // true
 ```
 */
function opensDate(
  {
    text,
    state,
  }: {
    readonly text: string;
    readonly state: Readonly<ScanState>;
  },
): boolean {
  /**
   Whole character before the digit, empty at the text's start.
   */
  const before = codePointBefore({
    text,
    at: state.at,
  },);
  /**
   Whether the digit follows an emoji or another character beyond the first
   plane that is no part of a word.
   */
  const afterPair = (before.length > 1) && (!continuesWord({ character: before, },));
  /**
   Whether a hyphen joins the digit to the date just read.
   */
  const afterDate = (before === '-') && ((state.at - 1) === state.lastEnd);
  return (state.at === 0) || DAY_OPENERS.has(before,)
    || afterPair
    || afterDate;
}

/**
 Where a date's rewrite starts: at an article that belongs to it ("the"
 or "The" as a word of its own, one space before an ordinal day, after the
 last date read), or at the date.

 @param text - text under scan

 @param state - where the scan stands, the date's first digit under the cursor

 @param reading - date read there

 @returns Offset the rewrite starts at

 @example
 ```ts
 rewriteStart({ text: 'on the 4th May', state: { at: 7, lastEnd: -1, }, reading, },); // 3
 ```
 */
function rewriteStart(
  {
    text,
    state,
    reading,
  }: {
    readonly text: string;
    readonly state: Readonly<ScanState>;
    readonly reading: DateReading;
  },
): number {
  /**
   Where the article would start.
   */
  const start = state.at - 1
    - ARTICLE_LENGTH;
  /**
   Whether the article stands there as a word of its own.
   */
  const article = ARTICLES.has(text.slice(
    start,
    state.at - 1,
  ),) && (!continuesWord({
    character: codePointBefore({
      text,
      at: start,
    },),
  },));
  /**
   Whether the article may be dropped with the date.
   */
  const drops = reading.takesArticle && (start >= 0)
    && (start > state.lastEnd)
    && isDateSpace({ character: text.charAt(state.at - 1,), },);
  return (drops && article) ? start : state.at;
}

/**
 Reads the date starting at one offset, whichever shape it takes.

 @param text - text under scan

 @param state - where the scan stands

 @returns The date, or no part where none starts there

 @example
 ```ts
 readingAt({ text: 'On 4 May', state: { at: 3, lastEnd: -1, }, },);
 ```
 */
function readingAt(
  {
    text,
    state,
  }: {
    readonly text: string;
    readonly state: Readonly<ScanState>;
  },
): DateReading | NoPart {
  /**
   Whole character under the cursor.
   */
  const character = codePointAt({
    text,
    at: state.at,
  },);
  /**
   Where the date would start, for the readers.
   */
  const place = {
    text,
    at: state.at,
  };
  if (isAsciiDigit({ character, },)) {
    if (!opensDate({
      text,
      state,
    },))
      return NO_PART;
    return DIGIT_READERS.reduce<DateReading | NoPart>(
      function firstFound(
        found,
        reader,
      ): DateReading | NoPart {
        return (found.kind === 'date') ? found : reader(place,);
      },
      NO_PART,
    );
  }
  return (isCasedLetter({ character, },) && (!continuesWord({
    character: codePointBefore({
      text,
      at: state.at,
    },),
  },)))
    ? readMonthFirst(place,)
    : NO_PART;
}

/**
 Every date in a text's prose that Canadian English writes differently,
 rewritten month first.

 @param text - text under scan

 @param ranges - the text's non-prose ranges

 @returns Rewrites in order, none overlapping

 @example
 ```ts
 monthFirstDates({ text: 'On 4 May', ranges: [], },);
 ```
 */
export function monthFirstDates(
  {
    text,
    ranges,
  }: {
    readonly text: string;
    readonly ranges: readonly ProtectedRange[];
  },
): readonly DateRewrite[] {
  /**
   Rewrites found so far.
   */
  const rewrites: DateRewrite[] = [];
  /**
   Where the scan stands.
   */
  const state: ScanState = {
    at: 0,
    lastEnd: -1,
  };
  while (state.at < text.length) {
    /**
     The date starting here, if any.
     */
    const reading = readingAt({
      text,
      state,
    },);
    /**
     Where its rewrite would start.
     */
    const start = (reading.kind === 'date')
      ? rewriteStart({
        text,
        state,
        reading,
      },)
      : state.at;
    if ((reading.kind === 'none') || (!inProse({
      ranges,
      start,
      end: reading.end,
    },))) {
      state.at += 1;
      continue;
    }
    /**
     The date as written.
     */
    const from = text.slice(
      start,
      reading.end,
    );
    if (from !== reading.to) {
      rewrites.push({
        start,
        end: reading.end,
        from,
        to: reading.to,
      },);
    }
    state.at = reading.end;
    state.lastEnd = reading.end;
  }
  return rewrites;
}
//endregion Canadian date
