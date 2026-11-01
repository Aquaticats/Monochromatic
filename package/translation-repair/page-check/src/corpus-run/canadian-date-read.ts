import {
  closingComma,
  isDateSpace,
  isWordCharacter,
  keptPeriod,
  monthStartsName,
  readDay,
  readMonth,
  yearAfter,
} from './canadian-date-parts.ts';
import { runEnd, } from './text-runs.ts';
import {
  type DateReading,
  type DayPart,
  type MonthPart,
  NO_PART,
  type NoPart,
  RANGE_DASHES,
  RANGE_WORDS,
} from './canadian-date-words.ts';

//region Canadian date readers
// Each reader starts at one offset and reads one date shape the page may
// carry, returning the month-first form Canadian English writes (ledger K1,
// K2, K4): day first ("4 May 2024"), and a range or pair of days sharing one
// month ("1st to 3rd June"). `canadian-date-read-leading.ts` reads the dates
// a month or a year opens. A reader that finds no such date returns no part;
// one whose date needs no change returns the date as written.

/**
 The word that may join an ordinal day to its month ("the 4th of May").
 */
const OF = 'of';

/**
 The part of a day-first date after its month.
 */
type DayFirstTail = Readonly<{
  kind: 'tail';

  /**
   Offset past the date as written.
   */
  end: number;

  /**
   What the month-first form writes after the day.
   */
  tail: string;
}>;

/**
 The join between two days of one month.
 */
type RangeJoin = Readonly<{
  kind: 'join';

  /**
   Offset of the second day.
   */
  end: number;
}>;

/**
 Whether one character may stand in the space around a range word or dash.

 @param character - one UTF-16 unit

 @returns Whether it is a space, a no-break space, a tab or a line break

 @example
 ```ts
 isRangeSpace({ character: '\n', },); // true
 ```
 */
function isRangeSpace(
  { character, }: { readonly character: string; },
): boolean {
  return isDateSpace({ character, },) || [
    '\t',
    '\n',
  ].includes(character,);
}

/**
 Reads a month after a day, across one space and an "of" an ordinal day may
 take.

 @param text - text under scan

 @param day - day just read

 @returns The month, or no part where none follows the day

 @example
 ```ts
 monthAfterDay({ text: '4th of May', day: { kind: 'day', day: 4, end: 3, ordinal: true, }, },);
 ```
 */
export function monthAfterDay(
  {
    text,
    day,
  }: {
    readonly text: string;
    readonly day: DayPart;
  },
): MonthPart | NoPart {
  /**
   Offset of the word after the space.
   */
  const next = day.end + 1;
  /**
   Whether "of" stands between an ordinal day and its month.
   */
  const joined = day.ordinal && text.startsWith(
    OF,
    next,
  )
    && isDateSpace({ character: text.charAt(next + OF.length,), },);
  if (!isDateSpace({ character: text.charAt(day.end,), },))
    return NO_PART;
  return readMonth({
    text,
    at: joined ? next + OF.length
      + 1 : next,
  },);
}

/**
 Reads what a day-first date writes after its month: a year, closed by a
 comma where the sentence goes on, or an abbreviation's period.

 @param text - text under scan

 @param month - month just read

 @returns The tail, or no part where the month starts a name or runs into
 more digits or letters

 @example
 ```ts
 dayFirstTail({ text: '4 May 2024 in', month: { kind: 'month', month: 'May', end: 5, period: false, }, },);
 ```
 */
function dayFirstTail(
  {
    text,
    month,
  }: {
    readonly text: string;
    readonly month: MonthPart;
  },
): DayFirstTail | NoPart {
  /**
   A year after the month, if any.
   */
  const year = yearAfter({
    text,
    at: month.end,
  },);
  if ((year.kind === 'refused') || monthStartsName({
    text,
    month,
  },))
    return NO_PART;
  if (year.kind === 'none') {
    return {
      kind: 'tail',
      end: month.end,
      tail: keptPeriod({
        text,
        month,
      },),
    };
  }
  return {
    kind: 'tail',
    end: year.end,
    tail: `, ${year.year}${closingComma({
      text,
      end: year.end,
    },)}`,
  };
}

/**
 Reads the range word or dash joining two days, with the space around it.

 @param text - text under scan

 @param at - offset just past the first day

 @returns Where the second day starts, or no part where no range word or dash
 stands there, a word lacks space on either side, or the space around it
 breaks more than one line

 @example
 ```ts
 readJoin({ text: '1st\nto 3rd', at: 3, },); // { kind: 'join', end: 7 }
 ```
 */
export function readJoin(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): RangeJoin | NoPart {
  /**
   Where the space before the word or dash ends.
   */
  const joinStart = runEnd({
    text,
    from: at,
    keeps: isRangeSpace,
  },);
  /**
   Whether a dash joins the days, which needs no space around it.
   */
  const dashed = RANGE_DASHES.has(text.charAt(joinStart,),);
  /**
   Where the range word or dash ends.
   */
  const joinEnd = dashed
    ? joinStart + 1
    : runEnd({
      text,
      from: joinStart,
      keeps: isWordCharacter,
    },);
  /**
   Where the space after the word or dash ends.
   */
  const end = runEnd({
    text,
    from: joinEnd,
    keeps: isRangeSpace,
  },);
  /**
   Whether a range word stands there with space on both sides.
   */
  const spacedWord = RANGE_WORDS.has(text.slice(
    joinStart,
    joinEnd,
  ),) && (joinStart > at)
    && (end > joinEnd);
  /**
   The whole join as written.
   */
  const join = text.slice(
    at,
    end,
  );
  /**
   Whether the join breaks at most one line.
   */
  const oneBreak = join.indexOf('\n',) === join.lastIndexOf('\n',);
  return ((dashed || spacedWord) && oneBreak)
    ? {
      kind: 'join',
      end,
    }
    : NO_PART;
}


/**
 A day, the month after it, and what follows the month.
 */
type DayFirstParts = Readonly<{
  kind: 'parts';

  /**
   The day.
   */
  day: DayPart;

  /**
   The month after it.
   */
  month: MonthPart;

  /**
   What follows the month.
   */
  tail: DayFirstTail;
}>;

/**
 Reads a day, the month after it and what follows the month.

 @param text - text under scan

 @param at - offset of the day's first digit

 @returns The parts, or no part where no day-first date starts there

 @example
 ```ts
 dayFirstParts({ text: '4 May 2024 in', at: 0, },);
 ```
 */
function dayFirstParts(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): DayFirstParts | NoPart {
  /**
   The day.
   */
  const day = readDay({
    text,
    at,
  },);
  /**
   The month after it.
   */
  const month = (day.kind === 'day')
    ? monthAfterDay({
      text,
      day,
    },)
    : NO_PART;
  /**
   What follows the month.
   */
  const tail = (month.kind === 'month')
    ? dayFirstTail({
      text,
      month,
    },)
    : NO_PART;
  if ((day.kind === 'none') || (month.kind === 'none')
    || (tail.kind === 'none'))
    return NO_PART;
  return {
    kind: 'parts',
    day,
    month,
    tail,
  };
}

/**
 Reads a day-first date: "4 May", "the 4th of May", "4 Sept 2024".

 @param text - text under scan

 @param at - offset of the day's first digit

 @returns The date, or no part where none starts there

 @example
 ```ts
 readDayFirst({ text: '4 May 2024 in', at: 0, },); // { kind: 'date', end: 10, to: 'May 4, 2024,', takesArticle: false }
 ```
 */
export function readDayFirst(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): DateReading | NoPart {
  /**
   The date's parts.
   */
  const parts = dayFirstParts({
    text,
    at,
  },);
  if (parts.kind === 'none')
    return NO_PART;
  /**
   Day, month and tail read.
   */
  const {
    day,
    month,
    tail,
  } = parts;
  return {
    kind: 'date',
    end: tail.end,
    to: `${month.month} ${String(day.day,)}${tail.tail}`,
    takesArticle: day.ordinal,
  };
}

/**
 Reads a range or pair of days sharing one month written after both: "1st to
 3rd June", "1st and\n3rd June", written once with the month first and the
 join kept as it stands (ledger K2).

 @param text - text under scan

 @param at - offset of the first day's first digit

 @returns The date, or no part where no such range starts there

 @example
 ```ts
 readRange({ text: '1st to 3rd June', at: 0, },); // { kind: 'date', end: 15, to: 'June 1 to 3', takesArticle: true }
 ```
 */
export function readRange(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): DateReading | NoPart {
  /**
   The first day.
   */
  const first = readDay({
    text,
    at,
  },);
  /**
   The join after it.
   */
  const join = (first.kind === 'day')
    ? readJoin({
      text,
      at: first.end,
    },)
    : NO_PART;
  /**
   The day-first date the second day opens.
   */
  const last = (join.kind === 'join')
    ? dayFirstParts({
      text,
      at: join.end,
    },)
    : NO_PART;
  if ((first.kind === 'none') || (join.kind === 'none')
    || (last.kind === 'none'))
    return NO_PART;
  /**
   Both days and the join between them, as written month first.
   */
  const days = `${String(first.day,)}${text.slice(
    first.end,
    join.end,
  )}${String(last.day
    .day,)}`;
  return {
    kind: 'date',
    end: last.tail
      .end,
    to: `${last.month
      .month} ${days}${last.tail
        .tail}`,
    takesArticle: first.ordinal,
  };
}

//endregion Canadian date readers
