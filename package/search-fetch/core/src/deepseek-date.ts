/**
 Date filter parsing shared by DeepSeek query composition and result filtering.
 
 The repository bans regex literals in source, so date shapes are recognized by
 length and digit checks instead. DeepSeek operator values accept a year, a
 month, or a day; post-filtering also accepts a full timestamp through
 `Date.parse`.
 
 @module
 */

//region Constants

/**
 Canonical date shape used to derive separator positions and component lengths.
 */
const DATE_SHAPE = 'YYYY-MM-DD' as const;

/**
 Position of the first hyphen in a month or day value.
 */
const FIRST_SEPARATOR_INDEX = DATE_SHAPE.indexOf('-',);

/**
 Position of the second hyphen in a day value.
 */
const SECOND_SEPARATOR_INDEX = DATE_SHAPE.lastIndexOf('-',);

/**
 Length of a year-only value.
 */
const YEAR_LENGTH = 'YYYY'.length;

/**
 Length of a month value.
 */
const MONTH_LENGTH = 'YYYY-MM'.length;

/**
 Length of a day value.
 */
const DAY_LENGTH = DATE_SHAPE.length;

/**
 Separator between date components.
 */
const DATE_SEPARATOR = '-' as const;

/**
 Milliseconds in one day.
 */
const MILLISECONDS_PER_DAY = 86_400_000;

//endregion Constants

//region Types

/**
 Recognized shapes of one date filter value.
 */
type DateFilterShape = 'year' | 'month' | 'day' | 'other';

/**
 Resolution of one date filter into a boundary instant.
 */
type DateBoundary = {
  /**
   Whether the filter resolved to an instant.
   */
  readonly found: false;
} | {
  /**
   Whether the filter resolved to an instant.
   */
  readonly found: true;
  /**
   Boundary instant in milliseconds.
   */
  readonly instant: number;
};

//endregion Types

//region Shape checks

/**
 Return whether a value is one of the shapes DeepSeek operators accept.
 
 @param value - candidate operator date
 
 @returns whether the value is a year, month, or day
 
 @example
 ```ts
 isOperatorDate('2026-01-01');
 ```
 */
function isOperatorDate(value: string,): boolean {
  /**
   Recognized shape of the value.
   */
  const shape = dateFilterShape(value,);
  return (shape === 'year') || (shape === 'month')
    || (shape === 'day');
}

/**
 Classify one date filter value by its shape.
 
 @param value - candidate date text
 
 @returns recognized shape, with every other form classified as other
 
 @example
 ```ts
 dateFilterShape('2026-01');
 ```
 */
function dateFilterShape(value: string,): DateFilterShape {
  if ((value.length === YEAR_LENGTH) && isDigitRange({
    value,
    start: 0,
    end: YEAR_LENGTH,
  },))
    return 'year';
  if (
    (value.length === MONTH_LENGTH)
      && isDigitRange({
        value,
        start: 0,
        end: FIRST_SEPARATOR_INDEX,
      },)
      && (value[FIRST_SEPARATOR_INDEX] === DATE_SEPARATOR)
      && isDigitRange({
        value,
        start: FIRST_SEPARATOR_INDEX + 1,
        end: MONTH_LENGTH,
      },)
  )
    return 'month';
  if (
    (value.length === DAY_LENGTH)
      && isDigitRange({
        value,
        start: 0,
        end: FIRST_SEPARATOR_INDEX,
      },)
      && (value[FIRST_SEPARATOR_INDEX] === DATE_SEPARATOR)
      && isDigitRange({
        value,
        start: FIRST_SEPARATOR_INDEX + 1,
        end: SECOND_SEPARATOR_INDEX,
      },)
      && (value[SECOND_SEPARATOR_INDEX] === DATE_SEPARATOR)
      && isDigitRange({
        value,
        start: SECOND_SEPARATOR_INDEX + 1,
        end: DAY_LENGTH,
      },)
  )
    return 'day';
  return 'other';
}

/**
 Return whether every character in a range is an ASCII digit.
 
 @param value - text to scan
 
 @param start - inclusive start index
 
 @param end - exclusive end index
 
 @returns whether the range holds only digits
 
 @example
 ```ts
 isDigitRange({ value: '2026', start: 0, end: 4 });
 ```
 */
function isDigitRange(
  {
    value,
    start,
    end,
  }: {
    readonly value: string;
    readonly start: number;
    readonly end: number;
  },
): boolean {
  for (let index = start; index < end; index += 1) {
    /**
     Character at the current index.
     */
    const character = value[index];
    if (character === undefined)
      return false;
    if (character < '0')
      return false;
    if (character > '9')
      return false;
  }
  return true;
}

//endregion Shape checks

//region Boundary resolution

/**
 Resolve one date filter into an inclusive boundary instant.
 
 @param value - date filter text
 
 @param endOfRange - whether a year, month, or day value covers its whole range
 
 @returns boundary instant resolution
 
 @example
 ```ts
 dateBoundary({ value: '2026-01-01', endOfRange: false });
 ```
 */
function dateBoundary(
  {
    value,
    endOfRange,
  }: {
    readonly value?: string;
    readonly endOfRange: boolean;
  },
): DateBoundary {
  if (value === undefined)
    return { found: false, };
  /**
   Trimmed filter text.
   */
  const trimmed = value.trim();
  if (trimmed === '')
    return { found: false, };
  /**
   Recognized shape of the filter.
   */
  const shape = dateFilterShape(trimmed,);
  if (shape === 'year') {
    /**
     Named year.
     */
    const year = Number(trimmed,);
    return {
      found: true,
      instant: endOfRange
        ? (Date.UTC(
          year + 1,
          0,
          1,
        ) - 1)
        : Date.UTC(
          year,
          0,
          1,
        ),
    };
  }
  if (shape === 'month') {
    /**
     Year and zero-based month from the filter.
     */
    const [year, month,] = trimmed.split(DATE_SEPARATOR,)
      .map(Number,);
    if ((year === undefined) || (month === undefined))
      return { found: false, };
    return {
      found: true,
      instant: endOfRange
        ? (Date.UTC(
          year,
          month,
          1,
        ) - 1)
        : Date.UTC(
          year,
          month - 1,
          1,
        ),
    };
  }
  if (shape === 'day') {
    /**
     First instant of the named day.
     */
    const start = Date.parse(`${trimmed}T00:00:00.000Z`,);
    return {
      found: true,
      instant: endOfRange
        ? ((start + MILLISECONDS_PER_DAY) - 1)
        : start,
    };
  }
  /**
   Parsed full timestamp filter.
   */
  const parsed = Date.parse(trimmed,);
  return Number.isNaN(parsed,)
    ? { found: false, }
    : {
      found: true,
      instant: parsed,
    };
}

//endregion Boundary resolution

export {
  dateBoundary,
  dateFilterShape,
  isOperatorDate,
};
export type {
  DateBoundary,
  DateFilterShape,
};
