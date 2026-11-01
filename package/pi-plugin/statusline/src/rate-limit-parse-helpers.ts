/**
 Shared parser helpers for provider usage headers.
 
 @module
 */

import {
  MILLISECONDS_PER_SECOND,
  PERCENT_BASE,
  type RateLimitSnapshot,
} from './rate-limit-types.ts';

/**
 Sentinel returned when a header or object property is missing or invalid.
 */
const INVALID_VALUE: unique symbol = Symbol('usage header value missing or invalid',);

/**
 Sentinel returned when a provider value cannot produce a snapshot.
 */
const INVALID_RATE_LIMIT_SNAPSHOT: unique symbol = Symbol('usage rate limit snapshot missing or invalid',);

/**
 Invalid value sentinel type.
 */
type InvalidValue = typeof INVALID_VALUE;

/**
 Invalid snapshot sentinel type.
 */
type InvalidRateLimitSnapshot = typeof INVALID_RATE_LIMIT_SNAPSHOT;

/**
 Unknown object record used while parsing provider JSON.
 */
type UnknownRecord = Readonly<Record<string, unknown>>;

/**
 Normalizes provider headers to lowercase names.
 
 @param headers - provider response headers from Pi
 
 @returns header record keyed by lowercase header names
 
 @example
 ```ts
 normalizeHeaders({ 'X-Codex-Primary-Used-Percent': '50' });
 ```
 */
function normalizeHeaders(
  headers: Readonly<Record<string, string>>,
): Record<string, string> {
  /**
   Mutable accumulator filled with lowercase header names.
   */
  const normalizedHeaders: Record<string, string> = {};

  for (const entry of Object.entries(headers,)) {
    /**
     Header name and value before case normalization.
     */
    const [name, value,] = entry;
    normalizedHeaders[name.toLowerCase()] = value;
  }

  return normalizedHeaders;
}

/**
 Parses a non-negative numeric header.
 
 @param headers - lowercase provider response headers
 
 @param headerName - lowercase header name to parse
 
 @returns numeric value, or {@link INVALID_VALUE} when missing or invalid
 
 @example
 ```ts
 parseNumberHeader({ headers: { count: '10' }, headerName: 'count' });
 ```
 */
function parseNumberHeader({
  headers,
  headerName,
}: Readonly<{
  headers: Readonly<Record<string, string>>;
  headerName: string;
}>,): number | InvalidValue {
  /**
   Raw header value, if the provider supplied it.
   */
  const rawValue = headers[headerName];
  if (rawValue === undefined)
    return INVALID_VALUE;

  /**

   Numeric header value parsed from the raw string.

   */
  const value = Number(rawValue,);
  if ((!Number.isFinite(value,)) || (value < 0))
    return INVALID_VALUE;

  return value;
}

/**
 Parses a string header and trims whitespace.
 
 @param headers - lowercase provider response headers
 
 @param headerName - lowercase header name to parse
 
 @returns trimmed string value, or {@link INVALID_VALUE} when missing or empty
 
 @example
 ```ts
 parseStringHeader({ headers: { name: ' codex ' }, headerName: 'name' });
 ```
 */
function parseStringHeader({
  headers,
  headerName,
}: Readonly<{
  headers: Readonly<Record<string, string>>;
  headerName: string;
}>,): string | InvalidValue {
  /**
   Raw header value, if the provider supplied it.
   */
  const rawValue = headers[headerName];
  if (rawValue === undefined)
    return INVALID_VALUE;

  /**

   Trimmed header value.

   */
  const value = rawValue.trim();
  if (value.length === 0)
    return INVALID_VALUE;

  return value;
}

/**
 Parses an RFC 3339 reset header into epoch milliseconds.
 
 @param headers - lowercase provider response headers
 
 @param headerName - lowercase reset header name to parse
 
 @returns epoch milliseconds, or {@link INVALID_VALUE} when missing or invalid
 
 @example
 ```ts
 parseResetHeader({ headers: { reset: '2026-06-01T12:00:00Z' }, headerName: 'reset' });
 ```
 */
function parseResetHeader({
  headers,
  headerName,
}: Readonly<{
  headers: Readonly<Record<string, string>>;
  headerName: string;
}>,): number | InvalidValue {
  /**
   Raw reset timestamp supplied by the provider.
   */
  const rawValue = headers[headerName];
  if (rawValue === undefined)
    return INVALID_VALUE;

  /**

   Parsed epoch timestamp in milliseconds.

   */
  const resetAtMs = Date.parse(rawValue,);
  if (!Number.isFinite(resetAtMs,))
    return INVALID_VALUE;

  return resetAtMs;
}

/**
 Parses an epoch-seconds reset header into epoch milliseconds.
 
 @param headers - lowercase provider response headers
 
 @param headerName - lowercase reset header name to parse
 
 @returns epoch milliseconds, or {@link INVALID_VALUE} when missing or invalid
 
 @example
 ```ts
 parseEpochSecondsHeader({ headers: { reset: '1704069000' }, headerName: 'reset' });
 ```
 */
function parseEpochSecondsHeader({
  headers,
  headerName,
}: Readonly<{
  headers: Readonly<Record<string, string>>;
  headerName: string;
}>,): number | InvalidValue {
  /**
   Parsed reset value in seconds.
   */
  const seconds = parseNumberHeader({
    headers,
    headerName,
  },);
  if (seconds === INVALID_VALUE)
    return INVALID_VALUE;

  return seconds * MILLISECONDS_PER_SECOND;
}

/**
 Clamps numeric value into inclusive range.
 
 @param value - number to clamp
 
 @param min - lower bound
 
 @param max - upper bound
 
 @returns value inside `[min, max]`
 
 @example
 ```ts
 clampNumber({ value: 12, min: 0, max: 10 });
 ```
 */
function clampNumber({
  value,
  min,
  max,
}: Readonly<{
  value: number;
  min: number;
  max: number;
}>,): number {
  return Math.min(
    max,
    Math.max(
      min,
      value,
    ),
  );
}

/**
 Detects object records while parsing JSON.
 
 @param value - unknown value to inspect
 
 @returns whether value is a non-array object
 
 @example
 ```ts
 isUnknownRecord({});
 ```
 */
function isUnknownRecord(value: unknown,): value is UnknownRecord {
  return ((typeof value) === 'object')
    && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Reads nested object property from an unknown record.
 
 @param record - source record
 
 @param key - property key to read
 
 @returns nested object, or {@link INVALID_VALUE} when missing or not an object
 
 @example
 ```ts
 readRecordProperty({ record: { nested: {} }, key: 'nested' });
 ```
 */
function readRecordProperty({
  record,
  key,
}: Readonly<{
  record: UnknownRecord;
  key: string;
}>,): UnknownRecord | InvalidValue {
  /**
   Property value before type narrowing.
   */
  const value = record[key];
  if (!isUnknownRecord(value,))
    return INVALID_VALUE;

  return value;
}

/**
 Reads numeric property from an unknown record.
 
 @param record - source record
 
 @param key - property key to read
 
 @returns finite number, or {@link INVALID_VALUE} when missing or invalid
 
 @example
 ```ts
 readNumberProperty({ record: { used: 42 }, key: 'used' });
 ```
 */
function readNumberProperty({
  record,
  key,
}: Readonly<{
  record: UnknownRecord;
  key: string;
}>,): number | InvalidValue {
  /**
   Property value before numeric narrowing.
   */
  const value = record[key];
  if (((typeof value) !== 'number') || (!Number.isFinite(value,)))
    return INVALID_VALUE;

  return value;
}

/**
 ASCII digit characters, each standing for one digit in a text's shape.
 */
const ASCII_DIGITS = '0123456789';

/**
 Shape of an ISO-8601 date and time to the second, with each digit written as `9`.
 */
const ISO_DATE_TIME_SHAPE = '9999-99-99T99:99:99';

/**
 Shapes a zone designator may take once digits are written as `9`: `Z` or a colon-separated offset.
 */
const ISO_ZONE_SHAPES = [
  'Z',
  '+99:99',
  '-99:99',
] as const;

/**
 Index that `findIndex` returns when no character ends the fraction, which means no zone follows.
 */
const NO_ZONE_FOUND = -1;

/**
 Writes each ASCII digit of a text as `9` and leaves every other character as it is.
 
 @param text - text to reduce to its shape
 
 @returns the shape of the text
 
 @example
 ```ts
 shapeOfText('2026-10-23T05:03:47Z');
 ```
 */
function shapeOfText(text: string,): string {
  /**
   Each character of the text, a digit written as `9`.
   */
  const shapedCharacters = Array.from(
    text,
    function shapeOfCharacter(character,): string {
      return ASCII_DIGITS.includes(character,) ? '9' : character;
    },
  );

  return shapedCharacters.join('',);
}

/**
 Days in a month of thirty-one days.
 */
const LONG_MONTH_DAYS = 31;

/**
 Days in a month of thirty days.
 */
const SHORT_MONTH_DAYS = 30;

/**
 Days in February of a year that is no leap year.
 */
const COMMON_FEBRUARY_DAYS = 28;

/**
 Length in days of each month of a year that is no leap year, January first.
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

/**
 Position of the February entry in {@link COMMON_YEAR_MONTH_DAYS}, whose length a leap year raises by one day.
 */
const FEBRUARY_INDEX = 1;

/**
 Years divisible by this are leap years, unless divisible by {@link CENTURY_YEARS} and not by
 {@link LEAP_CENTURY_YEARS}.
 */
const LEAP_CYCLE_YEARS = 4;

/**
 Years divisible by this are no leap years, unless divisible by {@link LEAP_CENTURY_YEARS}.
 */
const CENTURY_YEARS = 100;

/**
 Years divisible by this are leap years even when divisible by {@link CENTURY_YEARS}.
 */
const LEAP_CENTURY_YEARS = 400;

/**
 First minute or second value that is no minute of an hour or second of a minute: 60 is no minute, and a leap second is
 not an instant here.
 */
const FIRST_NON_MINUTE_OR_SECOND = 60;

/**
 First hour value that is no hour of a day: 24 is the end-of-day spelling of midnight, which is not an instant here.
 */
const FIRST_NON_HOUR = 24;

/**
 Tells whether a year of the Gregorian calendar has a 29th of February.
 
 @param year - calendar year
 
 @returns true when divisible by 4, and not by 100 unless by 400
 
 @example
 ```ts
 isLeapYear(2024);
 ```
 */
function isLeapYear(year: number,): boolean {
  return ((year % LEAP_CYCLE_YEARS) === 0)
    && (((year % CENTURY_YEARS) !== 0) || ((year % LEAP_CENTURY_YEARS) === 0));
}

/**
 Tells whether a zone designator already known to have the shape `Z`, `+HH:MM` or `-HH:MM` names a real offset.
 
 @param zone - the zone designator
 
 @returns true for `Z`, and for an offset whose hours are below 24 and minutes below 60
 
 @example
 ```ts
 isRealZone('+24:00'); // false
 ```
 */
function isRealZone(zone: string,): boolean {
  if (zone === 'Z')
    return true;

  /**
   The offset after its sign.
   */
  const unsignedOffset = zone.slice(1,);
  /**
   Hours and minutes of the offset, as text.
   */
  const offsetFields = unsignedOffset.split(':',);
  /**
   Hours and minutes of the offset.
   */
  const [offsetHour = 0, offsetMinute = 0,] = offsetFields.map(Number,);

  return (offsetHour < FIRST_NON_HOUR)
    && (offsetMinute < FIRST_NON_MINUTE_OR_SECOND);
}

/**
 Tells whether every field of text already known to have the shape of an ISO-8601 instant with a zone is in range.
 
 Month 01 to 12, day within the month by the Gregorian leap rule, hour 00 to 23, minute and second 00 to 59, offset
 hours 00 to 23 and offset minutes 00 to 59. The verdict never rests on the engine's `Date.parse`.
 
 @param text - text whose first nineteen characters have the shape `YYYY-MM-DDTHH:MM:SS`
 
 @param zone - the zone designator that ends the text
 
 @returns true when every field is in range
 
 @example
 ```ts
 hasRealFields({ text: '2026-04-31T05:03:47Z', zone: 'Z' }); // false
 ```
 */
function hasRealFields({
  text,
  zone,
}: Readonly<{
  text: string;
  zone: string;
}>,): boolean {
  /**
   Leading `YYYY-MM-DDTHH:MM:SS` of the text.
   */
  const dateTime = text.slice(
    0,
    ISO_DATE_TIME_SHAPE.length,
  );
  /**
   Date and time halves of the leading date and time.
   */
  const [datePart = '', timePart = '',] = dateTime.split('T',);
  /**
   Year, month and day fields of the date half, as text.
   */
  const dateFields = datePart.split('-',);
  /**
   Calendar year, month and day of the date half.
   */
  const [year = 0, month = 0, day = 0,] = dateFields.map(Number,);
  /**
   Hour, minute and second fields of the time half, as text.
   */
  const timeFields = timePart.split(':',);
  /**
   Hour, minute and second of the time half.
   */
  const [hour = 0, minute = 0, second = 0,] = timeFields.map(Number,);
  /**
   Days in the named month of a year that is no leap year, absent for a month outside one to twelve.
   */
  const commonYearDays = COMMON_YEAR_MONTH_DAYS[month - 1];
  if (commonYearDays === undefined)
    return false;

  /**
   Days in the named month of the named year.
   */
  const monthDays = ((month - 1) === FEBRUARY_INDEX) && isLeapYear(year,)
    ? commonYearDays + 1
    : commonYearDays;

  return (day >= 1)
    && (day <= monthDays)
    && (hour < FIRST_NON_HOUR)
    && (minute < FIRST_NON_MINUTE_OR_SECOND)
    && (second < FIRST_NON_MINUTE_OR_SECOND)
    && isRealZone(zone,);
}

/**
 Tells whether text is an ISO-8601 instant with a zone: date, `T`, time with an optional fraction, then `Z` or a
 colon-separated offset.
 
 Checked on the untrimmed text, so a trailing newline, a year alone, a month-name form, a local time without a zone and
 an offset without its colon are all outside it. `Date.parse` reads those leniently, and reads the zone-less ones in the
 process's time zone. Every field range is checked here, so the verdict never depends on the engine: month 01 to 12,
 the day against the month's length by the Gregorian leap rule (`Date.parse` rolls 2026-04-31 into May), hour 00 to 23
 (24 is an end-of-day spelling where an instant is wanted), minute and second 00 to 59, offset hours 00 to 23 and offset
 minutes 00 to 59. The caller's `Date.parse` only turns the accepted text into a number.
 
 @param text - candidate reset text
 
 @returns true when the text has the shape
 
 @example
 ```ts
 isIsoInstantWithZone('2026-10-23T05:03:47.000Z');
 ```
 */
function isIsoInstantWithZone(text: string,): boolean {
  /**
   Shape of the text, digits written as `9`.
   */
  const shape = shapeOfText(text,);
  if (!shape.startsWith(ISO_DATE_TIME_SHAPE,))
    return false;

  /**
   Shape after the seconds: a fraction, then the zone.
   */
  const afterSeconds = shape.slice(ISO_DATE_TIME_SHAPE.length,);
  /**
   Whether a fraction follows the seconds.
   */
  const hasFraction = afterSeconds.startsWith('.',);
  /**
   Fraction digits, when present, followed by the zone.
   */
  const fractionAndZone = hasFraction ? afterSeconds.slice('.'.length,) : afterSeconds;
  /**
   Characters of the fraction and zone shape.
   */
  const fractionCharacters = Array.from(fractionAndZone,);
  /**
   Count of fraction digits before the zone begins.
   */
  const fractionLength = fractionCharacters.findIndex(function endsFraction(character,): boolean {
    return character !== '9';
  },);
  if (fractionLength === NO_ZONE_FOUND)
    return false;

  if (hasFraction && (fractionLength === 0))
    return false;

  if ((!hasFraction) && (fractionLength > 0))
    return false;

  /**
   Shape of the zone designator that ends the text.
   */
  const zoneShape = fractionAndZone.slice(fractionLength,);
  if (!(ISO_ZONE_SHAPES as readonly string[]).includes(zoneShape,))
    return false;

  return hasRealFields({
    text,
    zone: text.slice(text.length - zoneShape.length,),
  },);
}

/**
 Parses reset property from an unknown record.
 
 Accepts only an ISO-8601 instant with a zone (`Z` or an offset) that also names a real time.
 
 @param record - source record
 
 @param key - property key to read
 
 @returns epoch milliseconds, or {@link INVALID_VALUE} when missing or invalid
 
 @example
 ```ts
 readResetProperty({ record: { reset: '2026-06-01T12:00:00Z' }, key: 'reset' });
 ```
 */
function readResetProperty({
  record,
  key,
}: Readonly<{
  record: UnknownRecord;
  key: string;
}>,): number | InvalidValue {
  /**
   Reset string before timestamp parsing.
   */
  const value = record[key];
  if (((typeof value) !== 'string') || (!isIsoInstantWithZone(value,)))
    return INVALID_VALUE;

  /**

   Parsed reset timestamp in milliseconds.

   */
  const resetAtMs = Date.parse(value,);
  if (!Number.isFinite(resetAtMs,))
    return INVALID_VALUE;

  return resetAtMs;
}

/**
 Computes bounded percentage from used and limit values.
 
 @param used - used quota value
 
 @param limit - total quota value
 
 @returns used percentage clamped to non-negative values
 
 @example
 ```ts
 usedPercentFromLimit({ used: 5, limit: 10 });
 ```
 */
function usedPercentFromLimit({
  used,
  limit,
}: Readonly<{
  used: number;
  limit: number;
}>,): number | InvalidValue {
  if (limit <= 0)
    return INVALID_VALUE;
  if (used < 0)
    return INVALID_VALUE;

  return Math.max(
    0,
    (used / limit) * PERCENT_BASE,
  );
}

/**
 Creates a generic {@link RateLimitSnapshot} from provider-specific values.
 
 @param key - stable snapshot key
 
 @param label - footer label
 
 @param resetAtMs - reset timestamp in epoch milliseconds
 
 @param windowSeconds - fixed window duration in seconds
 
 @param paceScale - optional elapsed-pace multiplier
 
 @param sampledAtMs - sample timestamp in epoch milliseconds
 
 @param usedPercent - used capacity percentage
 
 @returns generic {@link RateLimitSnapshot}, or {@link INVALID_RATE_LIMIT_SNAPSHOT} when a field is invalid
 
 @example
 ```ts
 createRateLimitSnapshot({
   key: 'demo',
   label: 'demo',
   resetAtMs,
   windowSeconds: 60,
   paceScale: 1,
   sampledAtMs,
   usedPercent: 50,
 });
 ```
 */
function createRateLimitSnapshot({
  key,
  label,
  resetAtMs,
  windowSeconds,
  paceScale,
  sampledAtMs,
  usedPercent,
}: Readonly<{
  key: string;
  label: string;
  resetAtMs: number;
  windowSeconds: number;
  paceScale: number;
  sampledAtMs: number;
  usedPercent: number;
}>,): RateLimitSnapshot | InvalidRateLimitSnapshot {
  if ((!Number.isFinite(resetAtMs,))
    || (!Number.isFinite(windowSeconds,))
    || (windowSeconds <= 0))
    return INVALID_RATE_LIMIT_SNAPSHOT;
  if ((!Number.isFinite(paceScale,)) || (paceScale <= 0))
    return INVALID_RATE_LIMIT_SNAPSHOT;
  if ((!Number.isFinite(usedPercent,)) || (usedPercent < 0))
    return INVALID_RATE_LIMIT_SNAPSHOT;

  return {
    key,
    label,
    resetAtMs,
    windowSeconds,
    paceScale,
    sampledAtMs,
    usedPercent,
  };
}

/**
 Detects valid parsed rate-limit snapshots.
 
 @param snapshot - optional snapshot from one parser
 
 @returns whether snapshot exists
 
 @example
 ```ts
 isRateLimitSnapshot(INVALID_RATE_LIMIT_SNAPSHOT);
 ```
 */
function isRateLimitSnapshot(
  snapshot: RateLimitSnapshot | InvalidRateLimitSnapshot,
): snapshot is RateLimitSnapshot {
  return snapshot !== INVALID_RATE_LIMIT_SNAPSHOT;
}

export {
  INVALID_RATE_LIMIT_SNAPSHOT,
  INVALID_VALUE,
  clampNumber,
  createRateLimitSnapshot,
  isRateLimitSnapshot,
  isUnknownRecord,
  normalizeHeaders,
  parseEpochSecondsHeader,
  parseNumberHeader,
  parseResetHeader,
  parseStringHeader,
  readNumberProperty,
  readRecordProperty,
  readResetProperty,
  usedPercentFromLimit,
};
export type {
  InvalidRateLimitSnapshot,
  InvalidValue,
  UnknownRecord,
};
