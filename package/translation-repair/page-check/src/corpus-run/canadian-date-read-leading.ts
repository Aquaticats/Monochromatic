import { codePointAt, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  closingComma,
  continuesWord,
  isDateSpace,
  keptPeriod,
  monthStartsName,
  readDay,
  readMonth,
  readYear,
  wordFollows,
  yearAfter,
} from './canadian-date-parts.ts';
import {
  monthAfterDay,
  readJoin,
} from './canadian-date-read.ts';
import {
  type DateReading,
  type DayPart,
  type MonthPart,
  NO_PART,
  type NoPart,
  type RefusedPart,
  type YearPart,
} from './canadian-date-words.ts';

//region Canadian date readers led by a month or a year
// Dates that open with their month or their year rather than their day
// (ledger K1, K4): "May 14th, 2023 the cat" loses the suffix and takes a
// comma after the year; "2023 Feb 25th" and "2023, 31 Mar." are written month
// first with the month in full.

/**
 The days a month-first date carries: one, or a range or pair sharing the
 month.
 */
type MonthFirstDays = Readonly<{
  /**
   The days as the month-first form writes them.
   */
  written: string;

  /**
   Offset past the last day.
   */
  end: number;
}>;

/**
 Reads the days after a month: one day, or a range or pair whose second day
 carries no month of its own ("May 11th to 13th"; "May 30th to 2nd June" is
 two dates).

 @param text - text under scan

 @param first - first day after the month

 @returns The days as written month first, suffixes dropped

 @example
 ```ts
 monthFirstDays({ text: 'May 11th to 13th.', first: { kind: 'day', day: 11, end: 8, ordinal: true, }, },);
 ```
 */
function monthFirstDays(
  {
    text,
    first,
  }: {
    readonly text: string;
    readonly first: DayPart;
  },
): MonthFirstDays {
  /**
   The join after the first day.
   */
  const join = readJoin({
    text,
    at: first.end,
  },);
  /**
   The second day, where one stands after the join.
   */
  const last = (join.kind === 'join')
    ? readDay({
      text,
      at: join.end,
    },)
    : NO_PART;
  /**
   A month after the second day, which makes it a date of its own.
   */
  const ownMonth = (last.kind === 'day')
    ? monthAfterDay({
      text,
      day: last,
    },)
    : NO_PART;
  if ((join.kind === 'none') || (last.kind === 'none')
    || (ownMonth.kind === 'month')) {
    return {
      written: String(first.day,),
      end: first.end,
    };
  }
  return {
    written: `${String(first.day,)}${text.slice(
      first.end,
      join.end,
    )}${String(last.day,)}`,
    end: last.end,
  };
}

/**
 Reads the year after a month-first date's days, with or without the comma
 before it.

 @param text - text under scan

 @param at - offset just past the last day

 @returns The year, no part, or refused where a year runs into more digits
 or letters

 @example
 ```ts
 yearAfterDays({ text: '14th, 2023 the', at: 4, },); // { kind: 'year', year: '2023', end: 10 }
 ```
 */
function yearAfterDays(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): YearPart | NoPart | RefusedPart {
  if ((text.charAt(at,) === ',') && isDateSpace({ character: text.charAt(at + 1,), },)) {
    return readYear({
      text,
      at: at + 2,
    },);
  }
  return yearAfter({
    text,
    at,
  },);
}

/**
 Reads a month standing with a year and no day: "Nov 2023", written with the
 month in full as running text spells it; a full name reads back unchanged.

 @param text - text under scan

 @param month - month just read

 @returns The month and year, or no part where no clean year follows

 @example
 ```ts
 monthYear({ text: 'Nov 2023', month: { kind: 'month', month: 'November', end: 3, period: false, }, },); // { kind: 'date', end: 8, to: 'November 2023', takesArticle: false }
 ```
 */
function monthYear(
  {
    text,
    month,
  }: {
    readonly text: string;
    readonly month: MonthPart;
  },
): DateReading | NoPart {
  /**
   The year after the month.
   */
  const year = isDateSpace({ character: text.charAt(month.end,), },)
    ? readYear({
      text,
      at: month.end + 1,
    },)
    : NO_PART;
  if (year.kind === 'none')
    return NO_PART;
  return {
    kind: 'date',
    end: year.end,
    to: `${month.month} ${year.year}`,
    takesArticle: false,
  };
}

/**
 Reads a month-first date: "December 29th", "May 14th, 2023 the cat",
 "Aug. 8th, 2018".

 @param text - text under scan

 @param at - offset of the month's first letter

 @returns The date, or no part where none starts there

 @example
 ```ts
 readMonthFirst({ text: 'May 14th, 2023 the', at: 0, },); // { kind: 'date', end: 14, to: 'May 14, 2023,', takesArticle: false }
 ```
 */
export function readMonthFirst(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): DateReading | NoPart {
  /**
   The month.
   */
  const month = readMonth({
    text,
    at,
  },);
  /**
   The first day after it.
   */
  const first = ((month.kind === 'month') && isDateSpace({ character: text.charAt(month.end,), },))
    ? readDay({
      text,
      at: month.end + 1,
    },)
    : NO_PART;
  if (month.kind === 'none')
    return NO_PART;
  if (first.kind === 'none') {
    return monthYear({
      text,
      month,
    },);
  }
  /**
   The days the month carries.
   */
  const days = monthFirstDays({
    text,
    first,
  },);
  /**
   A year after the days, if any.
   */
  const year = yearAfterDays({
    text,
    at: days.end,
  },);
  if (year.kind === 'refused')
    return NO_PART;
  /**
   The year as the month-first form writes it, closed by a comma where the
   sentence goes on.
   */
  const written = (year.kind === 'year')
    ? `, ${year.year}${closingComma({
      text,
      end: year.end,
    },)}`
    : '';
  return {
    kind: 'date',
    end: (year.kind === 'year') ? year.end : days.end,
    to: `${month.month} ${days.written}${written}`,
    takesArticle: false,
  };
}

/**
 Reads a year-first date whose day follows its month: "2023 Feb 25th". Its
 day carries an ordinal suffix or ends the clause, since "In 2021 May 4 cats"
 is no date.

 @param text - text under scan

 @param year - year just read

 @returns The date, or no part where none follows the year

 @example
 ```ts
 yearMonthDay({ text: '2023 Feb 25th:', year: { kind: 'year', year: '2023', end: 4, }, },);
 ```
 */
function yearMonthDay(
  {
    text,
    year,
  }: {
    readonly text: string;
    readonly year: YearPart;
  },
): DateReading | NoPart {
  /**
   The month after the year.
   */
  const month = isDateSpace({ character: text.charAt(year.end,), },)
    ? readMonth({
      text,
      at: year.end + 1,
    },)
    : NO_PART;
  /**
   The day after the month.
   */
  const day = ((month.kind === 'month') && isDateSpace({ character: text.charAt(month.end,), },))
    ? readDay({
      text,
      at: month.end + 1,
    },)
    : NO_PART;
  if ((month.kind === 'none') || (day.kind === 'none'))
    return NO_PART;
  /**
   Whether the sentence runs on past the day.
   */
  const runsOn = wordFollows({
    text,
    end: day.end,
  },);
  if (runsOn && (!day.ordinal))
    return NO_PART;
  return {
    kind: 'date',
    end: day.end,
    to: `${month.month} ${String(day.day,)}, ${year.year}${runsOn ? ',' : ''}`,
    takesArticle: false,
  };
}

/**
 Reads a year-first date whose day comes before its month, after a comma:
 "2023, 31 Mar.". Only a date ending its clause is read, since in "In 2020,
 4 May was a holiday" the comma closes the phrase and "4 May" is a date on
 its own.

 @param text - text under scan

 @param year - year just read

 @returns The date, or no part where none follows the year

 @example
 ```ts
 yearDayMonth({ text: '2023, 31 Mar.', year: { kind: 'year', year: '2023', end: 4, }, },);
 ```
 */
function yearDayMonth(
  {
    text,
    year,
  }: {
    readonly text: string;
    readonly year: YearPart;
  },
): DateReading | NoPart {
  /**
   The day after the comma.
   */
  const day = ((text.charAt(year.end,) === ',') && isDateSpace({ character: text.charAt(year.end + 1,), },))
    ? readDay({
      text,
      at: year.end + 2,
    },)
    : NO_PART;
  /**
   The month after the day.
   */
  const month = (day.kind === 'day')
    ? monthAfterDay({
      text,
      day,
    },)
    : NO_PART;
  if ((day.kind === 'none') || (month.kind === 'none'))
    return NO_PART;
  /**
   Whether the date runs on into a name, more digits or letters, or the rest
   of its sentence.
   */
  const runsOn = monthStartsName({
    text,
    month,
  },) || continuesWord({
    character: codePointAt({
      text,
      at: month.end,
    },),
  },)
    || wordFollows({
      text,
      end: month.end,
    },);
  if (runsOn)
    return NO_PART;
  return {
    kind: 'date',
    end: month.end,
    to: `${month.month} ${String(day.day,)}, ${year.year}${keptPeriod({
      text,
      month,
    },)}`,
    takesArticle: false,
  };
}

/**
 Reads a year-first date: "2023 Feb 25th" or "2023, 31 Mar.".

 @param text - text under scan

 @param at - offset of the year's first digit

 @returns The date, or no part where none starts there

 @example
 ```ts
 readYearFirst({ text: 'On 2021 July 20th, the cat', at: 3, },); // { kind: 'date', end: 17, to: 'July 20, 2021', takesArticle: false }
 ```
 */
export function readYearFirst(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): DateReading | NoPart {
  /**
   The year.
   */
  const year = readYear({
    text,
    at,
  },);
  if (year.kind === 'none')
    return NO_PART;
  /**
   The date with its month right after the year.
   */
  const monthFirst = yearMonthDay({
    text,
    year,
  },);
  return (monthFirst.kind === 'date')
    ? monthFirst
    : yearDayMonth({
      text,
      year,
    },);
}

//endregion Canadian date readers led by a month or a year
