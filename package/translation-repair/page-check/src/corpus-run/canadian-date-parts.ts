import {
  DAY_DIGITS,
  type DayPart,
  LAST_DAY,
  MONTH_COMPOUNDS,
  MONTH_NAMES,
  type MonthPart,
  NO_PART,
  type NoPart,
  ORDINAL_SUFFIXES,
  REFUSED_PART,
  type RefusedPart,
  YEAR_DIGITS,
  type YearPart,
} from './canadian-date-words.ts';

import {
  codePointAt,
  isAsciiDigit,
  isCombiningMark,
} from '@monochromatic-dev/translation-repair-markup/ts';
import {
  isCapitalLetter,
  isCasedLetter,
  isSmallLetter,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { runEnd, } from './text-runs.ts';

//region Canadian date parts
// Reads one part of a date at a time: a day, a month, a year, and what may
// stand around them. `canadian-date-read.ts` puts the parts together into the
// date shapes the Canadian pass rewrites.

/**
 No-break space, which may stand between a date's words.
 */
const NO_BREAK_SPACE = '\u00A0';

/**
 Whether one character belongs to a word: a letter with case, or a
 combining mark that continues one ("idée" written with a separate accent).
 The Canadian passes read every word through this test, so a word written
 with a combining accent is read whole, as its composed spelling is (audit
 area six, ledger B18): a month such as `May` with a combining accent after
 it is no month, and a date beside such a word is part of it.

 @param character - one whole character, as `codePointAt` reads it

 @returns Whether it continues a word

 @example
 ```ts
 isWordCharacter({ character: '\u{0301}', },); // true
 ```
 */
export function isWordCharacter(
  { character, }: { readonly character: string; },
): boolean {
  return isCasedLetter({ character, },) || isCombiningMark({ character, },);
}

/**
 Whether one character continues a word, so a date touching it is part of a
 longer token.

 @param character - character beside the date, empty at a text edge

 @returns Whether it is a letter, a combining mark or a digit

 @example
 ```ts
 continuesWord({ character: 'a', },); // true
 ```
 */
export function continuesWord(
  { character, }: { readonly character: string; },
): boolean {
  return isAsciiDigit({ character, },) || isWordCharacter({ character, },);
}

/**
 Whether one character separates the words of a date: a space or a
 no-break space.

 @param character - one UTF-16 unit

 @returns Whether a date's words may stand either side of it

 @example
 ```ts
 isDateSpace({ character: ' ', },); // true
 ```
 */
export function isDateSpace(
  { character, }: { readonly character: string; },
): boolean {
  return (character === ' ') || (character === NO_BREAK_SPACE);
}

/**
 Whether a word follows one offset after a space, so the sentence goes on
 past a date ending there.

 @param text - text under scan

 @param end - offset just past the date

 @returns Whether a space and then a letter or digit stand there

 @example
 ```ts
 wordFollows({ text: '2024 in a box', end: 4, },); // true
 ```
 */
export function wordFollows(
  {
    text,
    end,
  }: {
    readonly text: string;
    readonly end: number;
  },
): boolean {
  return isDateSpace({ character: text.charAt(end,), },) && continuesWord({
    character: codePointAt({
      text,
      at: end + 1,
    },),
  },);
}

/**
 Reads a day number, with any ordinal suffix, at one offset.

 @param text - text under scan

 @param at - offset of its first digit

 @returns The day, or no part where no day 1 to 31 stands there on its own

 @example
 ```ts
 readDay({ text: '4th May', at: 0, },); // { kind: 'day', day: 4, end: 3, ordinal: true }
 ```
 */
export function readDay(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): DayPart | NoPart {
  /**
   Where the digits end.
   */
  const digitsEnd = runEnd({
    text,
    from: at,
    keeps: isAsciiDigit,
  },);
  /**
   How many digits the day carries.
   */
  const digits = digitsEnd - at;
  /**
   The day number.
   */
  const day = Number(text.slice(
    at,
    digitsEnd,
  ),);
  /**
   Whether the digits make a day of a month.
   */
  const inMonth = (day >= 1) && (day <= LAST_DAY);
  if ((digits === 0) || (digits > DAY_DIGITS)
    || (!inMonth))
    return NO_PART;
  /**
   Ordinal suffix after the digits, or the empty string.
   */
  const suffix = ORDINAL_SUFFIXES.find(function stands(candidate,): boolean {
    return text.startsWith(
      candidate,
      digitsEnd,
    );
  },) ?? '';
  /**
   Offset past the day.
   */
  const end = digitsEnd + suffix.length;
  if (continuesWord({
    character: codePointAt({
      text,
      at: end,
    },),
  },))
    return NO_PART;
  return {
    kind: 'day',
    day,
    end,
    ordinal: suffix !== '',
  };
}

/**
 Reads a month name, full or abbreviated, at one offset.

 @param text - text under scan

 @param at - offset of its first letter

 @returns The month, or no part where no month name stands there

 @example
 ```ts
 readMonth({ text: 'Sept 2024', at: 0, },); // { kind: 'month', month: 'September', end: 4, period: false }
 ```
 */
export function readMonth(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): MonthPart | NoPart {
  /**
   Where the letters end.
   */
  const lettersEnd = runEnd({
    text,
    from: at,
    keeps: isWordCharacter,
  },);
  /**
   The name as written.
   */
  const written = text.slice(
    at,
    lettersEnd,
  );
  /**
   The full name, where the word is a month.
   */
  const month = MONTH_NAMES.get(written,);
  if (month === undefined)
    return NO_PART;
  /**
   Whether an abbreviation's period follows it.
   */
  const period = (written !== month) && (text.charAt(lettersEnd,) === '.');
  return {
    kind: 'month',
    month,
    end: lettersEnd + (period ? 1 : 0),
    period,
  };
}

/**
 Reads a four-digit year at one offset.

 @param text - text under scan

 @param at - offset of its first digit

 @returns The year, or no part where no clean four-digit year stands there

 @example
 ```ts
 readYear({ text: '2024, the', at: 0, },); // { kind: 'year', year: '2024', end: 4 }
 ```
 */
export function readYear(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): YearPart | NoPart {
  /**
   Where the digits end.
   */
  const end = runEnd({
    text,
    from: at,
    keeps: isAsciiDigit,
  },);
  if (((end - at) !== YEAR_DIGITS) || continuesWord({
    character: codePointAt({
      text,
      at: end,
    },),
  },))
    return NO_PART;
  return {
    kind: 'year',
    year: text.slice(
      at,
      end,
    ),
    end,
  };
}

/**
 What stands after a date's month or day: a year, nothing that belongs to
 the date, or something that makes the whole reading no date.

 @param text - text under scan

 @param at - offset just past the month or day

 @returns The year, no part, or refused where digits or letters run on
 ("4 May2024"), or a year runs into more ("4 May 2024a"), ledger K8

 @example
 ```ts
 yearAfter({ text: 'May 2024 the', at: 3, },); // { kind: 'year', year: '2024', end: 8 }
 ```
 */
export function yearAfter(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): YearPart | NoPart | RefusedPart {
  if (continuesWord({
    character: codePointAt({
      text,
      at,
    },),
  },))
    return REFUSED_PART;
  if ((!isDateSpace({ character: text.charAt(at,), },)) || (!isAsciiDigit({ character: text.charAt(at + 1,), },)))
    return NO_PART;
  /**
   Where the digits after the space end.
   */
  const digitsEnd = runEnd({
    text,
    from: at + 1,
    keeps: isAsciiDigit,
  },);
  if ((digitsEnd - (at + 1)) < YEAR_DIGITS)
    return NO_PART;
  /**
   The year, where the digits make a clean one.
   */
  const year = readYear({
    text,
    at: at + 1,
  },);
  return (year.kind === 'year') ? year : REFUSED_PART;
}

/**
 Whether the word after a day-first date's month makes the month part of a
 name or a noun, not a date: a capitalised word ("2 June Carter", "3 April
 Fools’") or a listed compound ("5 May beetles"), ledger K8. "I" never
 continues a name, and after an abbreviation's period a capital opens the
 next sentence.

 @param text - text under scan

 @param month - month just read

 @returns Whether the reading is no date

 @example
 ```ts
 monthStartsName({ text: '2 June Carter fans', month: { kind: 'month', month: 'June', end: 6, period: false, }, },); // true
 ```
 */
export function monthStartsName(
  {
    text,
    month,
  }: {
    readonly text: string;
    readonly month: MonthPart;
  },
): boolean {
  if (month.period || (!isDateSpace({ character: text.charAt(month.end,), },)))
    return false;
  /**
   Where the next word ends.
   */
  const wordEnd = runEnd({
    text,
    from: month.end + 1,
    keeps: isWordCharacter,
  },);
  /**
   The next word.
   */
  const word = text.slice(
    month.end + 1,
    wordEnd,
  );
  /**
   Its first letter, whole.
   */
  const initial = codePointAt({
    text: word,
    at: 0,
  },);
  if ((word === '') || (word === 'I'))
    return false;
  if (isCapitalLetter({ character: initial, },))
    return true;
  /**
   Nouns the month forms with the word after it.
   */
  const compounds = MONTH_COMPOUNDS.get(month.month,) ?? new Set<string>();
  return compounds.has(word,);
}

/**
 The comma a year takes after it where the sentence goes on (ledger K1): the
 Language Portal of Canada sets a full date's year off on both sides.

 @param text - text under scan

 @param end - offset just past the year

 @returns A comma where a word follows the year, else nothing

 @example
 ```ts
 closingComma({ text: '2024 in a box', end: 4, },); // ','
 ```
 */
export function closingComma(
  {
    text,
    end,
  }: {
    readonly text: string;
    readonly end: number;
  },
): string {
  return wordFollows({
    text,
    end,
  },)
    ? ','
    : '';
}

/**
 The period an abbreviated month carried where the date ends with it: kept
 unless a lower-case word runs on after it, since then it marked only the
 abbreviation.

 @param text - text under scan

 @param month - month just read

 @returns A period to keep after the rewritten date, or nothing

 @example
 ```ts
 keptPeriod({ text: '31 Mar.', month: { kind: 'month', month: 'March', end: 7, period: true, }, },); // '.'
 ```
 */
export function keptPeriod(
  {
    text,
    month,
  }: {
    readonly text: string;
    readonly month: MonthPart;
  },
): string {
  /**
   Whole character after the space that follows, if any.
   */
  const next = codePointAt({
    text,
    at: month.end + 1,
  },);
  /**
   Whether a lower-case word runs on after the period.
   */
  const runsOn = isDateSpace({ character: text.charAt(month.end,), },) && isSmallLetter({ character: next, },);
  return (month.period && (!runsOn)) ? '.' : '';
}

//endregion Canadian date parts
