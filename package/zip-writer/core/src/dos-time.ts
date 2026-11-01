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
 format. The DOS epoch is 1980-01-01; earlier dates are clamped.
 
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
   Floor-clamped at the DOS epoch so the offset subtraction never underflows.
   */
  const year = Math.max(
    date.getUTCFullYear(),
    DOS_EPOCH_YEAR,
  );
  /**
   Packed DOS date word ready to write to the archive.
   */
  const dosDate = ((year - DOS_EPOCH_YEAR) << DOS_DATE_YEAR_SHIFT)
    | ((date.getUTCMonth()
      + MONTH_OFFSET_TO_ONE_BASED) << DOS_DATE_MONTH_SHIFT)
    | date
      .getUTCDate();
  /**
   Packed DOS time word ready to write to the archive.
   */
  const dosTime = (date.getUTCHours()
    << DOS_TIME_HOUR_SHIFT)
    | (date.getUTCMinutes()
      << DOS_TIME_MINUTE_SHIFT)
    | (date.getUTCSeconds()
      >>> DOS_TIME_SECOND_SHIFT);
  return {
    date: dosDate,
    time: dosTime,
  };
}
