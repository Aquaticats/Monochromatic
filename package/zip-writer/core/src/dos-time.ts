/**
 MS-DOS time and date encoding for ZIP modification timestamps.
 
 Reference: PKWARE APPNOTE.txt v6.3.10 section 4.4.6 (last mod file time
 and date).
 
 @module
 */

/**
 Earliest year representable in DOS date encoding.
 */
const DOS_EPOCH_YEAR = 1_980;

/**
 Latest year representable in DOS date encoding: the seven-bit year field
 holds 1980 through 2107.
 */
const DOS_MAX_YEAR = 2_107;

/**
 Zero-based index of January, the month a DOS date range starts in, as
 `Date.UTC` counts months.
 */
const JANUARY_MONTH_INDEX = 0;

/**
 Zero-based index of December, the month a DOS date range ends in, as
 `Date.UTC` counts months.
 */
const DECEMBER_MONTH_INDEX = 11;

/**
 First day of a month; the representable range starts on day one.
 */
const FIRST_DAY_OF_MONTH = 1;

/**
 Last day of December; the representable range ends on December's final day.
 */
const LAST_DAY_OF_DECEMBER = 31;

/**
 Last hour representable in a DOS time word.
 */
const DOS_LAST_HOUR = 23;

/**
 Last minute representable in a DOS time word.
 */
const DOS_LAST_MINUTE = 59;

/**
 Last even second representable in a DOS time word; the two-second field
 cannot hold an odd second, so a timestamp ending at 23:59:59 clamps here.
 */
const DOS_LAST_EVEN_SECOND = 58;

/**
 Earliest instant representable in DOS encoding, as epoch milliseconds.
 */
const DOS_FIRST_INSTANT = Date.UTC(
  DOS_EPOCH_YEAR,
  JANUARY_MONTH_INDEX,
  FIRST_DAY_OF_MONTH,
);

/**
 Latest instant representable in DOS encoding, as epoch milliseconds:
 2107-12-31 23:59:58, the last even second of the last representable day.
 */
const DOS_LAST_INSTANT = Date.UTC(
  DOS_MAX_YEAR,
  DECEMBER_MONTH_INDEX,
  LAST_DAY_OF_DECEMBER,
  DOS_LAST_HOUR,
  DOS_LAST_MINUTE,
  DOS_LAST_EVEN_SECOND,
);

/**
 Bit position of the year field within a DOS date word.
 */
const DOS_DATE_YEAR_SHIFT = 9;

/**
 Bit position of the month field within a DOS date word.
 */
const DOS_DATE_MONTH_SHIFT = 5;

/**
 Bit position of the hour field within a DOS time word.
 */
const DOS_TIME_HOUR_SHIFT = 11;

/**
 Bit position of the minute field within a DOS time word.
 */
const DOS_TIME_MINUTE_SHIFT = 5;

/**
 Bits dropped to encode seconds (2-second resolution).
 */
const DOS_TIME_SECOND_SHIFT = 1;

/**
 JavaScript getUTCMonth is zero-based; ZIP DOS dates are one-based.
 */
const MONTH_OFFSET_TO_ONE_BASED = 1;

/**
 Encoded DOS modification timestamp split into time and date words.
 */
export type DosDateTime = {
  /**
   Time-of-day word: `(hour << 11) | (minute << 5) | (second / 2)`.
   */
  readonly time: number;

  /**
   Date word: `((year - 1980) << 9) | (month << 5) | day`.
   */
  readonly date: number;
};

/**
 Encode a JavaScript Date as MS-DOS time and date words used by the ZIP
 format. DOS dates run from 1980-01-01 through 2107-12-31 and DOS times
 hold even seconds only, so the representable instants are
 1980-01-01T00:00:00Z through 2107-12-31T23:59:58Z. Instants outside that
 range are clamped to the nearest boundary: an earlier date encodes as the
 1980-01-01 epoch, a later one as 2107-12-31 23:59:58.
 
 @param date - Wall-clock instant to encode (interpreted as UTC)
 
 @returns Two 16-bit words (time and date) in DOS encoding
 
 @example
 ```ts
 dosDateTime(new Date(Date.UTC(2024, 5, 15, 12, 30, 30,),),);
 // -> { date: ((2024 - 1980) << 9) | (6 << 5) | 15, time: (12 << 11) | (30 << 5) | 15 }
 ```
 */
export function dosDateTime(date: Date,): DosDateTime {
  /**
   Instant clamped into the representable range, so an out-of-range date
   yields the nearest real timestamp instead of a wrapped date word.
   */
  const instant = new Date(
    Math.min(
      Math.max(
        date.getTime(),
        DOS_FIRST_INSTANT,
      ),
      DOS_LAST_INSTANT,
    ),
  );
  /**
   Packed DOS date word ready to write to the archive.
   */
  const dosDate = ((instant.getUTCFullYear()
    - DOS_EPOCH_YEAR) << DOS_DATE_YEAR_SHIFT)
    | ((instant.getUTCMonth()
      + MONTH_OFFSET_TO_ONE_BASED) << DOS_DATE_MONTH_SHIFT)
    | instant
      .getUTCDate();
  /**
   Packed DOS time word ready to write to the archive.
   */
  const dosTime = (instant.getUTCHours()
    << DOS_TIME_HOUR_SHIFT)
    | (instant.getUTCMinutes()
      << DOS_TIME_MINUTE_SHIFT)
    | (instant.getUTCSeconds()
      >>> DOS_TIME_SECOND_SHIFT);
  return {
    date: dosDate,
    time: dosTime,
  };
}

export {
  DOS_DATE_MONTH_SHIFT,
  DOS_DATE_YEAR_SHIFT,
  DOS_EPOCH_YEAR,
  DOS_MAX_YEAR,
  DOS_TIME_HOUR_SHIFT,
  DOS_TIME_MINUTE_SHIFT,
  DOS_TIME_SECOND_SHIFT,
};
