/**
 Time and duration ratio constants.
 
 Values express fixed relationships between time units. Composed values are
 derived from primitive ratios so any future correction propagates.
 
 @example
 ```ts
 import {
   MS_PER_HOUR,
   MS_PER_SECOND,
   SECONDS_PER_DAY,
 } from '@monochromatic-dev/const-core';
 ```
 
 @module
 */

//region Primitive ratios

/**
 Milliseconds in one second.
 
 @example
 ```ts
 await wait(5 * MS_PER_SECOND,);
 ```
 */
export const MS_PER_SECOND = 1_000;

/**
 Seconds in one minute.
 
 @example
 ```ts
 const minutes = totalSeconds / SECONDS_PER_MINUTE;
 ```
 */
export const SECONDS_PER_MINUTE = 60;

/**
 Minutes in one hour.
 
 @example
 ```ts
 const hours = totalMinutes / MINUTES_PER_HOUR;
 ```
 */
export const MINUTES_PER_HOUR = 60;

/**
 Hours in one day.
 
 @example
 ```ts
 const days = totalHours / HOURS_PER_DAY;
 ```
 */
export const HOURS_PER_DAY = 24;

/**
 Days in one week.
 
 @example
 ```ts
 const weeks = totalDays / DAYS_PER_WEEK;
 ```
 */
export const DAYS_PER_WEEK = 7;

/**
 Days in a non-leap calendar year. Use the average 365.25 only when leap-year
 accuracy matters; for typical UI display 365 is the conventional choice.
 
 @example
 ```ts
 const years = totalDays / DAYS_PER_YEAR;
 ```
 */
export const DAYS_PER_YEAR = 365;

/**
 Months in one year.
 
 @example
 ```ts
 const years = totalMonths / MONTHS_PER_YEAR;
 ```
 */
export const MONTHS_PER_YEAR = 12;

//endregion Primitive ratios

//region Composed millisecond constants

/**
 Milliseconds in one minute, derived from {@link MS_PER_SECOND} and {@link SECONDS_PER_MINUTE}.
 
 @example
 ```ts
 setTimeout(handler, 5 * MS_PER_MINUTE,);
 ```
 */
export const MS_PER_MINUTE: number = MS_PER_SECOND * SECONDS_PER_MINUTE;

/**
 Milliseconds in one hour, derived from {@link MS_PER_MINUTE} and {@link MINUTES_PER_HOUR}.
 
 @example
 ```ts
 const cacheTtlMs = MS_PER_HOUR;
 ```
 */
export const MS_PER_HOUR: number = MS_PER_MINUTE * MINUTES_PER_HOUR;

/**
 Milliseconds in one day, derived from {@link MS_PER_HOUR} and {@link HOURS_PER_DAY}.
 
 @example
 ```ts
 const sweepEveryMs = MS_PER_DAY;
 ```
 */
export const MS_PER_DAY: number = MS_PER_HOUR * HOURS_PER_DAY;

/**
 Milliseconds in one week, derived from {@link MS_PER_DAY} and {@link DAYS_PER_WEEK}.
 
 @example
 ```ts
 const retentionMs = 4 * MS_PER_WEEK;
 ```
 */
export const MS_PER_WEEK: number = MS_PER_DAY * DAYS_PER_WEEK;

//endregion Composed millisecond constants

//region Composed second constants

/**
 Seconds in one hour, derived from {@link SECONDS_PER_MINUTE} and {@link MINUTES_PER_HOUR}.
 
 @example
 ```ts
 const hours = Math.floor(totalSeconds / SECONDS_PER_HOUR,);
 ```
 */
export const SECONDS_PER_HOUR: number = SECONDS_PER_MINUTE * MINUTES_PER_HOUR;

/**
 Seconds in one day, derived from {@link SECONDS_PER_HOUR} and {@link HOURS_PER_DAY}.
 
 @example
 ```ts
 const days = Math.floor(totalSeconds / SECONDS_PER_DAY,);
 ```
 */
export const SECONDS_PER_DAY: number = SECONDS_PER_HOUR * HOURS_PER_DAY;

//endregion Composed second constants
