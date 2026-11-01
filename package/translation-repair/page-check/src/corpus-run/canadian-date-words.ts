//region Canadian date words
// The words and numbers a date is read from, and the shapes its parts take.
// `canadian-date-parts.ts` reads one part at a time with them.

/**
 Month names a date may carry, each mapped to the full name the page writes.
 The Language Portal of Canada spells months out in running text, so an
 abbreviation is written in full (ledger K4).
 */
export const MONTH_NAMES: ReadonlyMap<string, string> = new Map([
  [
    'January',
    'January',
  ],
  [
    'February',
    'February',
  ],
  [
    'March',
    'March',
  ],
  [
    'April',
    'April',
  ],
  [
    'May',
    'May',
  ],
  [
    'June',
    'June',
  ],
  [
    'July',
    'July',
  ],
  [
    'August',
    'August',
  ],
  [
    'September',
    'September',
  ],
  [
    'October',
    'October',
  ],
  [
    'November',
    'November',
  ],
  [
    'December',
    'December',
  ],
  [
    'Jan',
    'January',
  ],
  [
    'Feb',
    'February',
  ],
  [
    'Mar',
    'March',
  ],
  [
    'Apr',
    'April',
  ],
  [
    'Jun',
    'June',
  ],
  [
    'Jul',
    'July',
  ],
  [
    'Aug',
    'August',
  ],
  [
    'Sep',
    'September',
  ],
  [
    'Sept',
    'September',
  ],
  [
    'Oct',
    'October',
  ],
  [
    'Nov',
    'November',
  ],
  [
    'Dec',
    'December',
  ],
],);

/**
 Lower-case words that make a month name before them part of a noun, not a
 date: "5 May beetles" counts beetles (ledger K8).
 */
export const MONTH_COMPOUNDS: ReadonlyMap<string, ReadonlySet<string>> = new Map([
  [
    'May',
    new Set([
      'apple',
      'apples',
      'beetle',
      'beetles',
      'bug',
      'bugs',
      'flies',
      'fly',
      'pole',
      'poles',
      'queen',
      'queens',
      'tree',
      'trees',
    ],),
  ],
  [
    'March',
    new Set([
      'hare',
      'hares',
    ],),
  ],
],);

/**
 Suffixes an ordinal day may carry.
 */
export const ORDINAL_SUFFIXES: readonly string[] = [
  'st',
  'nd',
  'rd',
  'th',
];

/**
 Words that join two days of one month: a range ("1st to 3rd June") or a
 pair ("4th or 5th May"), each written once month first.
 */
export const RANGE_WORDS: ReadonlySet<string> = new Set([
  'and',
  'or',
  'through',
  'till',
  'to',
  'until',
],);

/**
 Dashes that join two days of one month, spaced or not.
 */
export const RANGE_DASHES: ReadonlySet<string> = new Set([
  '-',
  '–',
  '—',
],);

/**
 Highest day of a month.
 */
export const LAST_DAY = 31;

/**
 Digits a year carries.
 */
export const YEAR_DIGITS = 4;

/**
 Most digits a day number carries.
 */
export const DAY_DIGITS = 2;

/**
 What a reader returns where the part it reads does not stand there.
 */
export type NoPart = Readonly<{
  kind: 'none';
}>;

/**
 The one no-part value.
 */
export const NO_PART: NoPart = { kind: 'none', };

/**
 What a reader returns where the text runs on from a date so that the whole
 reading is no date ("4 May2024", "4 May 2024a").
 */
export type RefusedPart = Readonly<{
  kind: 'refused';
}>;

/**
 The one refused-part value.
 */
export const REFUSED_PART: RefusedPart = { kind: 'refused', };

/**
 A day number read from a text.
 */
export type DayPart = Readonly<{
  kind: 'day';

  /**
   The day, 1 to 31.
   */
  day: number;

  /**
   Offset past its digits and any ordinal suffix.
   */
  end: number;

  /**
   Whether it carries an ordinal suffix, which lets an article stand before
   it and "of" after it.
   */
  ordinal: boolean;
}>;

/**
 A month read from a text.
 */
export type MonthPart = Readonly<{
  kind: 'month';

  /**
   Full name the page writes.
   */
  month: string;

  /**
   Offset past the name, and past an abbreviation's period.
   */
  end: number;

  /**
   Whether an abbreviation's period was read with it.
   */
  period: boolean;
}>;

/**
 A year read from a text.
 */
export type YearPart = Readonly<{
  kind: 'year';

  /**
   The four digits.
   */
  year: string;

  /**
   Offset past them.
   */
  end: number;
}>;

/**
 A date one reader found.
 */
export type DateReading = Readonly<{
  kind: 'date';

  /**
   Offset past the date as written.
   */
  end: number;

  /**
   The date month first.
   */
  to: string;

  /**
   Whether an article standing before the date belongs to it ("the 4th
   May"), so the month-first form drops it.
   */
  takesArticle: boolean;
}>;

//endregion Canadian date words
