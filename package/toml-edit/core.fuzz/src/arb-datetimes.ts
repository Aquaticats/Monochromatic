/**
 Datetime value arbitraries for the fuzz generators.
 
 Covers all four RFC-3339 / TOML kinds: offset datetime, local datetime, local
 date, and local time. Components are bounded to always-valid ranges (day at
 most 28 to dodge month-length and leap rules, which have their own fixtures)
 so every generated value parses. Samples omit a predicted value because the
 parser projects these to host-zone-shifted `Date`s; properties assert parse
 success and round-trip instead.
 
 @module
 */

import {
  type Arbitrary,
  constantFrom,
  integer,
  oneof,
  tuple,
} from 'fast-check';

import type { ValueSample, } from './arb-types.ts';

/**
 Left-pad `value` with zeros to `width` digits.
 
 @returns Zero-padded decimal string.
 */
function pad({
  value,
  width,
}: {
  readonly value: number;
  readonly width: number
},): string {
  return value.toString(10,)
    .padStart(
    width,
    '0',
    );
}

/**
 Arbitrary valid calendar date as `YYYY-MM-DD`.
 */
const dateTextArbitrary: Arbitrary<string> = tuple(
  integer({
    min: 1,
    max: 9_999,
  },),
  integer({
    min: 1,
    max: 12,
  },),
  integer({
    min: 1,
    max: 28,
  },),
)
  .map(function build([year, month, day,],) {
    return `${pad({
    value: year,
    width: 4,
  },)}-${pad({
    value: month,
    width: 2,
  },)}-${pad({
    value: day,
    width: 2,
  },)}`;
  },);

/**
 Arbitrary valid wall-clock time as `HH:MM:SS`, half with fractional seconds.
 */
const timeTextArbitrary: Arbitrary<string> = tuple(
  integer({
    min: 0,
    max: 23,
  },),
  integer({
    min: 0,
    max: 59,
  },),
  integer({
    min: 0,
    max: 59,
  },),
  constantFrom(
    '',
    '.5',
    '.000001',
    '.123456789',
  ),
)
  .map(function build([hour, minute, second, fraction,],) {
    return `${pad({
    value: hour,
    width: 2,
  },)}:${pad({
    value: minute,
    width: 2,
  },)}:${pad({
    value: second,
    width: 2,
  },)}${fraction}`;
  },);

/**
 Arbitrary timezone designator: Zulu or a signed offset.
 */
const offsetTextArbitrary: Arbitrary<string> = oneof(
  constantFrom(
    'Z',
    'z',
  ),
  tuple(
    constantFrom(
      '+',
      '-',
    ),
    integer({
      min: 0,
      max: 23,
    },),
    integer({
      min: 0,
      max: 59,
    },),
  )
    .map(function build([sign, hour, minute,],) {
      return `${sign}${pad({
      value: hour,
      width: 2,
    },)}:${pad({
      value: minute,
      width: 2,
    },)}`;
    },),
);

/**
 Offset datetimes: date and time joined by `T` or a space, then an offset.
 */
const offsetDateTimeArbitrary: Arbitrary<ValueSample> = tuple(
  dateTextArbitrary,
  timeTextArbitrary,
  constantFrom(
    'T',
    't',
    ' ',
  ),
  offsetTextArbitrary,
)
  .map(function build([date, time, separator, offset,],) {
    return { text: `${date}${separator}${time}${offset}`, };
  },);

/**
 Local datetimes: date and time with no offset.
 */
const localDateTimeArbitrary: Arbitrary<ValueSample> = tuple(
  dateTextArbitrary,
  timeTextArbitrary,
  constantFrom(
    'T',
    't',
    ' ',
  ),
)
  .map(function build([date, time, separator,],) {
    return { text: `${date}${separator}${time}`, };
  },);

/**
 Local dates.
 */
const localDateArbitrary: Arbitrary<ValueSample> = dateTextArbitrary.map(function build(date,) {
  return { text: date, };
},);

/**
 Local times.
 */
const localTimeArbitrary: Arbitrary<ValueSample> = timeTextArbitrary.map(function build(time,) {
  return { text: time, };
},);

/**
 Deterministic datetime examples spanning every kind.
 */
export const DATETIME_EXAMPLES: readonly ValueSample[] = [
  { text: '1979-05-27T07:32:00Z', },
  { text: '1979-05-27 07:32:00.999999-07:00', },
  { text: '1979-05-27T00:32:00.999999', },
  { text: '1979-05-27', },
  { text: '07:32:00', },
  { text: '00:32:00.999999', },
];

/**
 Datetime value arbitrary across all four kinds.
 */
export const datetimeSampleArbitrary: Arbitrary<ValueSample> = oneof(
  offsetDateTimeArbitrary,
  localDateTimeArbitrary,
  localDateArbitrary,
  localTimeArbitrary,
  constantFrom(...DATETIME_EXAMPLES,),
);
