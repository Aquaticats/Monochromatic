/**
 TOML lexical grammar for the date and time payloads a tagged wrapper carries.

 A wrapper payload is written into the document between the `=` and the line
 end, so it is only safe to interpolate once it provably consists of the
 characters that grammar allows. Offset date-times never arrive here: they are
 encoded from JS `Date` values, whose `toISOString` spelling is already valid.

 @module
 */

import { isDigit, } from '@monochromatic-dev/agent-harness-shared-text-scan/ts';

//region Field widths and separators

/**
 Digits in a calendar-year field.
 */
const YEAR_DIGITS = 4;

/**
 Digits in a month, day, hour, minute, or second field.
 */
const CLOCK_DIGITS = 2;

/**
 Separators a `YYYY-MM-DD` date holds.
 */
const DATE_SEPARATORS = 2;

/**
 Characters in a local date, derived from its field widths and separators.
 */
const LOCAL_DATE_LENGTH = (YEAR_DIGITS
  + (CLOCK_DIGITS
    * 2))
  + DATE_SEPARATORS;

/**
 Index of the first `-` in a local date.
 */
const FIRST_DATE_SEPARATOR = YEAR_DIGITS;

/**
 Index of the second `-` in a local date.
 */
const SECOND_DATE_SEPARATOR = (FIRST_DATE_SEPARATOR
  + CLOCK_DIGITS)
  + 1;

/**
 Index of the `:` between hours and minutes in a local time.
 */
const HOUR_SEPARATOR = CLOCK_DIGITS;

/**
 Index just past the minutes in a local time.
 */
const MINUTE_END = (HOUR_SEPARATOR
  + CLOCK_DIGITS)
  + 1;

/**
 Date-time separators TOML accepts between a date and a time.
 */
const LOCAL_DATE_TIME_DELIMITERS = new Set<string>([
  'T',
  't',
  ' ',
],);

/**
 Separator between the year, month, and day of a local date.
 */
const DATE_SEPARATOR = '-';

/**
 Separator between the hours, minutes, and seconds of a local time.
 */
const TIME_SEPARATOR = ':';

/**
 Separator between whole seconds and their fraction.
 */
const FRACTION_SEPARATOR = '.';

//endregion Field widths and separators

//region Field ranges

/**
 Lowest four-digit year TOML accepts.
 */
const YEAR_MIN = 0;

/**
 Highest four-digit year TOML accepts.
 */
const YEAR_MAX = 9_999;

/**
 Lowest month number.
 */
const MONTH_MIN = 1;

/**
 Highest month number.
 */
const MONTH_MAX = 12;

/**
 Month number of February, the only month whose length varies.
 */
const FEBRUARY = 2;

/**
 Lowest day-of-month number.
 */
const DAY_MIN = 1;

/**
 Lowest hour number.
 */
const HOUR_MIN = 0;

/**
 Highest hour number.
 */
const HOUR_MAX = 23;

/**
 Lowest minute number.
 */
const MINUTE_MIN = 0;

/**
 Highest minute number.
 */
const MINUTE_MAX = 59;

/**
 Lowest second number.
 */
const SECOND_MIN = 0;

/**
 Highest second number, TOML allowing a leap second.
 */
const SECOND_MAX = 60;

/**
 Days in a month holding 31 days.
 */
const LONG_MONTH_DAYS = 31;

/**
 Days in a month holding 30 days.
 */
const SHORT_MONTH_DAYS = 30;

/**
 Days in February of a common year.
 */
const COMMON_FEBRUARY_DAYS = 28;

/**
 Days in February of a leap year.
 */
const LEAP_FEBRUARY_DAYS = 29;

/**
 Interval in years at which a leap year recurs.
 */
const LEAP_YEAR_INTERVAL = 4;

/**
 Interval in years at which a leap year is skipped.
 */
const CENTURY_INTERVAL = 100;

/**
 Interval in years at which the skipped leap year returns.
 */
const LEAP_CENTURY_INTERVAL = 400;

/**
 Days in each month of a common year, in calendar order.
 */
const COMMON_YEAR_MONTH_DAYS = [
  LONG_MONTH_DAYS,
  COMMON_FEBRUARY_DAYS,
  LONG_MONTH_DAYS,
  SHORT_MONTH_DAYS,
  LONG_MONTH_DAYS,
  SHORT_MONTH_DAYS,
  LONG_MONTH_DAYS,
  LONG_MONTH_DAYS,
  SHORT_MONTH_DAYS,
  LONG_MONTH_DAYS,
  SHORT_MONTH_DAYS,
  LONG_MONTH_DAYS,
] as const;

//endregion Field ranges

//region Field checks

/**
 Whether `text` is a non-empty run of ASCII digits.

 @param text - Candidate field text.

 @returns Whether every character is a digit and at least one is present.

 @example
 ```ts
 isDigitRun({ text: '05', }); // true
 ```
 */
function isDigitRun({ text, }: { readonly text: string; },): boolean {
  return (text.length
    > 0)
    && Array.from(text,)
      .every(function digit(character,) {
        return isDigit(character,);
      },);
}

/**
 Whether a field is exactly `width` digits holding a value within a range.

 @param text - Candidate field text.

 @param width - Exact digit count the field must hold.

 @param min - Lowest accepted value.

 @param max - Highest accepted value.

 @returns Whether the field is a digit run of that width inside the range.

 @example
 ```ts
 isFieldInRange({ text: '05', width: 2, min: 1, max: 12, }); // true
 ```
 */
function isFieldInRange(
  {
    text,
    width,
    min,
    max,
  }: {
    readonly text: string;
    readonly width: number;
    readonly min: number;
    readonly max: number;
  },
): boolean {
  if ((text.length
    !== width)
    || (!isDigitRun({ text, },)))
    return false;
  /**
   Numeric value of a verified digit run.
   */
  const value = Number(text,);
  return (value
    >= min)
    && (value
      <= max);
}

/**
 Whether a calendar year is a leap year under the Gregorian rule.

 @param year - Four-digit year.

 @returns Whether February holds a 29th day that year.

 @example
 ```ts
 isLeapYear({ year: 2024, }); // true
 ```
 */
function isLeapYear({ year, }: { readonly year: number; },): boolean {
  return (((year
    % LEAP_YEAR_INTERVAL)
    === 0)
    && ((year
      % CENTURY_INTERVAL)
      !== 0))
    || ((year
      % LEAP_CENTURY_INTERVAL)
      === 0);
}

/**
 Days in a calendar month.

 @param year - Four-digit year, which decides February's length.

 @param month - Month number already checked as one through twelve.

 @returns Days the month holds that year.

 @example
 ```ts
 daysInMonth({ year: 2024, month: 2, }); // 29
 ```
 */
function daysInMonth(
  {
    year,
    month,
  }: {
    readonly year: number;
    readonly month: number;
  },
): number {
  if ((month
    === FEBRUARY)
    && isLeapYear({ year, },))
    return LEAP_FEBRUARY_DAYS;
  /**
   Month already range-checked by the caller, so the lookup always hits; the
   fallback keeps the absent index from reaching a day comparison.
   */
  return COMMON_YEAR_MONTH_DAYS[month
    - 1] ?? 0;
}

//endregion Field checks

//region Lexeme checks

/**
 Whether `text` is exactly one partial time, without a date or an offset.

 Seconds may be omitted, as TOML 1.1 allows, and a fraction may follow them.

 @param text - Candidate time text.

 @returns Whether the whole text is a `HH:MM`, `HH:MM:SS`, or fractional
          partial time.

 @example
 ```ts
 isPartialTime({ text: '10:00:00.5', }); // true
 ```
 */
function isPartialTime({ text, }: { readonly text: string; },): boolean {
  if (text.charAt(HOUR_SEPARATOR,)
    !== TIME_SEPARATOR)
    return false;
  if (!isFieldInRange({
    text: text.slice(
      0,
      HOUR_SEPARATOR,
    ),
    width: CLOCK_DIGITS,
    min: HOUR_MIN,
    max: HOUR_MAX,
  },))
    return false;
  if (!isFieldInRange({
    text: text.slice(
      HOUR_SEPARATOR
        + 1,
      MINUTE_END,
    ),
    width: CLOCK_DIGITS,
    min: MINUTE_MIN,
    max: MINUTE_MAX,
  },))
    return false;
  /**
   Text after the minutes, which is empty when seconds are omitted.
   */
  const afterMinutes = text.slice(MINUTE_END,);
  if (afterMinutes.length
    === 0)
    return true;
  if (!afterMinutes.startsWith(TIME_SEPARATOR))
    return false;
  /**
   Seconds field, which may carry a fraction after the seconds.
   */
  const secondField = afterMinutes.slice(1,);
  /**
   Index of the fraction separator, or `-1` when only whole seconds are given.
   */
  const fractionIndex = secondField.indexOf(FRACTION_SEPARATOR,);
  if (!isFieldInRange({
    text: fractionIndex
      === (-1) ? secondField : secondField.slice(
        0,
        fractionIndex,
      ),
    width: CLOCK_DIGITS,
    min: SECOND_MIN,
    max: SECOND_MAX,
  },))
    return false;
  if (fractionIndex
    === (-1))
    return true;
  return isDigitRun({
    text: secondField.slice(
      fractionIndex
        + 1,
    ),
  },);
}

//endregion Lexeme checks

/**
 Whether `text` is exactly one TOML local date.

 @param text - Wrapper payload to check.

 @returns Whether the whole payload is a `YYYY-MM-DD` date naming a real day.

 @example
 ```ts
 isLocalDateText({ text: '2026-05-14', }); // true
 isLocalDateText({ text: '2026-05-14 # x', }); // false
 ```
 */
export function isLocalDateText({ text, }: { readonly text: string; },): boolean {
  if (text.length
    !== LOCAL_DATE_LENGTH)
    return false;
  if (text.charAt(FIRST_DATE_SEPARATOR,)
    !== DATE_SEPARATOR)
    return false;
  if (text.charAt(SECOND_DATE_SEPARATOR,)
    !== DATE_SEPARATOR)
    return false;
  /**
   Year text before the first separator.
   */
  const yearText = text.slice(
    0,
    FIRST_DATE_SEPARATOR,
  );
  /**
   Month text between the two separators.
   */
  const monthText = text.slice(
    FIRST_DATE_SEPARATOR
      + 1,
    SECOND_DATE_SEPARATOR,
  );
  /**
   Day text after the second separator.
   */
  const dayText = text.slice(SECOND_DATE_SEPARATOR
    + 1,);
  if (!isFieldInRange({
    text: yearText,
    width: YEAR_DIGITS,
    min: YEAR_MIN,
    max: YEAR_MAX,
  },))
    return false;
  if (!isFieldInRange({
    text: monthText,
    width: CLOCK_DIGITS,
    min: MONTH_MIN,
    max: MONTH_MAX,
  },))
    return false;
  if (!isFieldInRange({
    text: dayText,
    width: CLOCK_DIGITS,
    min: DAY_MIN,
    max: LONG_MONTH_DAYS,
  },))
    return false;
  return isFieldInRange({
    text: dayText,
    width: CLOCK_DIGITS,
    min: DAY_MIN,
    max: daysInMonth({
      year: Number(yearText,),
      month: Number(monthText,),
    },),
  },);
}

/**
 Whether `text` is exactly one TOML local time.

 @param text - Wrapper payload to check.

 @returns Whether the whole payload is a partial time with no date or offset.

 @example
 ```ts
 isLocalTimeText({ text: '10:00:00', }); // true
 isLocalTimeText({ text: '10:00:00Z', }); // false
 ```
 */
export function isLocalTimeText({ text, }: { readonly text: string; },): boolean {
  return isPartialTime({ text, },);
}

/**
 Whether `text` is exactly one TOML local date-time.

 @param text - Wrapper payload to check.

 @returns Whether the whole payload is a date, a `T`, `t`, or space delimiter,
          and a partial time with no offset.

 @example
 ```ts
 isLocalDateTimeText({ text: '2026-05-14T10:00:00', }); // true
 isLocalDateTimeText({ text: '2026-05-14T10:00:00Z', }); // false
 ```
 */
export function isLocalDateTimeText({ text, }: { readonly text: string; },): boolean {
  if (text.length
    <= LOCAL_DATE_LENGTH)
    return false;
  if (!isLocalDateText({
    text: text.slice(
      0,
      LOCAL_DATE_LENGTH,
    ),
  },))
    return false;
  if (!LOCAL_DATE_TIME_DELIMITERS.has(text.charAt(LOCAL_DATE_LENGTH,),))
    return false;
  return isPartialTime({
    text: text.slice(LOCAL_DATE_LENGTH
      + 1,),
  },);
}
