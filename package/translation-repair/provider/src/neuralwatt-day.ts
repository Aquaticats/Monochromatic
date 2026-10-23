import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { isAsciiDigits, } from '@monochromatic-dev/translation-repair-markup/ts';

//region NeuralWatt day
// WHERE A NEURALWATT DAY STARTS. The owner, 2026-10-20: "The stop logic should
// obviously count total free energy used per day. Every neuralwatt day starts at
// `15:15`". That is 19:15 UTC for an owner in America/New_York (EDT, UTC-4), and it
// is the UTC time of day of the subscription's `current_period_start`
// (`GET /v1/quota` answered `2026-10-20T19:15:08Z` at 23:15 UTC that day), not
// 00:00 UTC, which the first version of this package assumed from the
// documentation's "resets every day at 00:00 UTC".
//
// THE ANCHOR IS A TIME OF DAY IN UTC, never a local time: a local anchor would
// move an hour with daylight saving while the subscription's period start does
// not. The day is the 24 hours from the latest instant at or before now that has
// that time of day, and a UTC day is exactly 24 hours long, so no calendar is
// needed. Where the quota does not give the time of day, the owner's setting
// does (`TRANSLATION_REPAIR_NEURALWATT_DAY_START`, `HH:MM` UTC); where neither
// does, the day is unknown and NeuralWatt is refused for the run, since an
// assumed midnight would count the wrong day's energy.

/**
 Milliseconds in a day, UTC days all being this long.
 */
export const MS_PER_DAY = 86_400_000;

/**
 Milliseconds in a minute.
 */
const MS_PER_MINUTE = 60_000;

/**
 Minutes in an hour.
 */
const MINUTES_PER_HOUR = 60;

/**
 Hours in a day.
 */
const HOURS_PER_DAY = 24;

/**
 Digits an hour or a minute is written with in `HH:MM`.
 */
const TWO_DIGITS = 2;

/**
 Environment variable giving the UTC time of day a NeuralWatt day starts at, `HH:MM`, for a run whose quota reply
 does not carry the subscription's `current_period_start`.
 */
export const NEURALWATT_DAY_START_VAR = 'TRANSLATION_REPAIR_NEURALWATT_DAY_START';

/**
 A UTC time of day a NeuralWatt day starts at; empty where none was given.

 @example
 ```ts
 const start: NeuralWattDayStart = { msOfDay: ((19 * 60) + 15) * 60_000, };
 ```
 */
export type NeuralWattDayStart = {
  /**
   Milliseconds after 00:00 UTC.
   */
  readonly msOfDay?: number;
};

/**
 Raised when the day-start setting is present but is not `HH:MM`.

 @example
 ```ts
 throw new NeuralWattDayStartOverrideError({ value: '7:5', },);
 ```
 */
export class NeuralWattDayStartOverrideError extends StatedRefusalError {
  /**
   Declares this message safe to forward: it names the variable and repeats the value the operator set in it.
   */
  override readonly messageNamesOnly: true = true;

  /**
   Names the variable, what it held, and the form it must take.

   @param value - what the variable held, quoted back

   @example
   ```ts
   new NeuralWattDayStartOverrideError({ value: '25:00', },);
   ```
   */
  public constructor({ value, }: { readonly value: string; },) {
    super({
      says: `${NEURALWATT_DAY_START_VAR} must be a UTC time of day written HH:MM, two digits each, such as 19:15; `
        + `it holds ${JSON.stringify(value,)}`,
    },);
    this.name = 'NeuralWattDayStartOverrideError';
  }
}

/**
 Reads the two digits of an hour or a minute within a bound.

 @param text - the digits as written

 @param limit - exclusive upper bound

 @returns The number in a list of one, or an empty list where the text is not two digits below the bound

 @example
 ```ts
 const [hours,] = twoDigitsBelow({ text: '19', limit: 24, },);
 ```
 */
function twoDigitsBelow({
  text,
  limit,
}: {
  readonly text: string;
  readonly limit: number
},): readonly number[] {
  if ((text.length !== TWO_DIGITS) || (!isAsciiDigits({ text, },)))
    return [];

  /**
   The digits as a number.
   */
  const value = Number(text,);
  return (value < limit) ? [value,] : [];
}

/**
 The day-start setting: the variable's `HH:MM` as a UTC time of day.

 @param env - process environment

 @returns The time of day, or nothing where the variable is absent or blank

 @throws {@link NeuralWattDayStartOverrideError} when the variable holds something that is not `HH:MM`

 @example
 ```ts
 const setting = neuralWattDayStartFrom({ env: process.env, },);
 ```
 */
export function neuralWattDayStartFrom({ env, }: { readonly env: Readonly<NodeJS.ProcessEnv>; },): NeuralWattDayStart {
  /**
   What the variable holds, absent or blank for no setting.
   */
  const raw = env[NEURALWATT_DAY_START_VAR];
  if ((raw === undefined) || (raw.trim() === ''))
    return {};

  /**
   Hours and minutes as written.
   */
  const [hoursText = '', minutesText = '', ...beyond] = raw.trim()
    .split(':',);
  /**
   The hour, if it is two digits below 24.
   */
  const [hours,] = twoDigitsBelow({
    text: hoursText,
    limit: HOURS_PER_DAY,
  },);
  /**
   The minute, if it is two digits below 60.
   */
  const [minutes,] = twoDigitsBelow({
    text: minutesText,
    limit: MINUTES_PER_HOUR,
  },);
  if ((hours === undefined) || (minutes === undefined)
    || (beyond.length > 0))
    throw new NeuralWattDayStartOverrideError({ value: raw, },);
  return { msOfDay: ((hours * MINUTES_PER_HOUR) + minutes) * MS_PER_MINUTE, };
}

/**
 The UTC time of day of an instant a reply spelled.

 @param instant - ISO 8601 instant, such as the quota's `current_period_start`

 @returns The time of day, or nothing where the text is not an instant

 @example
 ```ts
 neuralWattDayStartOfInstant({ instant: '2026-10-20T19:15:08Z', },); // { msOfDay: 69_308_000 }
 ```
 */
export function neuralWattDayStartOfInstant({ instant, }: { readonly instant: string; },): NeuralWattDayStart {
  /**
   Epoch milliseconds, NaN where the text is not an instant.
   */
  const epochMs = Date.parse(instant,);
  if (!Number.isFinite(epochMs,))
    return {};
  return { msOfDay: ((epochMs % MS_PER_DAY) + MS_PER_DAY) % MS_PER_DAY, };
}

/**
 The start of the day that contains an instant: the latest instant at or before it with the given UTC time of day.

 @param now - the instant

 @param msOfDay - milliseconds after 00:00 UTC the day starts at

 @returns Start of the day, UTC

 @example
 ```ts
 neuralWattDayStartAt({ now: new Date('2026-10-20T23:15:00.741Z',), msOfDay: 69_300_000, },);
 // 2026-10-20T19:15:00.000Z
 ```
 */
export function neuralWattDayStartAt(
  {
    now,
    msOfDay,
  }: {
    readonly now: Date;
    readonly msOfDay: number;
  },
): Date {
  /**
   The anchor on the instant's UTC date: midnight plus the time of day, which the milliseconds argument carries past
   its usual range.
   */
  const candidate = new Date(Date.UTC(
    now.getUTCFullYear(),
    now.getUTCMonth(),
    now.getUTCDate(),
    0,
    0,
    0,
    msOfDay,
  ),);
  if (candidate <= now)
    return candidate;
  return new Date(Date.UTC(
    now.getUTCFullYear(),
    now.getUTCMonth(),
    now.getUTCDate() - 1,
    0,
    0,
    0,
    msOfDay,
  ),);
}

/**
 The start of the day after the one starting at an instant: the same time of day, a UTC date later.

 @param start - first instant of a day

 @returns First instant of the next, which is also when the first ends

 @example
 ```ts
 neuralWattDayAfter({ start: new Date('2026-10-20T19:15:08.000Z',), },); // 2026-10-21T19:15:08.000Z
 ```
 */
export function neuralWattDayAfter({ start, }: { readonly start: Date; },): Date {
  return new Date(Date.UTC(
    start.getUTCFullYear(),
    start.getUTCMonth(),
    start.getUTCDate() + 1,
    start.getUTCHours(),
    start.getUTCMinutes(),
    start.getUTCSeconds(),
    start.getUTCMilliseconds(),
  ),);
}

/**
 The time of day an anchor places the NeuralWatt day's start at, in UTC.

 @param msOfDay - milliseconds after 00:00 UTC

 @returns The time as `HH:MM:SS.sss`

 @example
 ```ts
 neuralWattTimeOfDayUtc({ msOfDay: 69_308_000, },); // '19:15:08.000'
 ```
 */
export function neuralWattTimeOfDayUtc({ msOfDay, }: { readonly msOfDay: number; },): string {
  /**
   The instant that many milliseconds into 1970-01-01, which spells the time of day.
   */
  const iso = new Date(msOfDay,).toISOString();
  return iso.slice(
    'YYYY-MM-DDT'.length,
    'YYYY-MM-DDTHH:MM:SS.sss'.length,
  );
}

/**
 The milliseconds of a time of day that starts on a whole second.
 */
const WHOLE_SECOND_MILLIS = '.000';

/**
 The time of day an anchor places the NeuralWatt day's start at, in UTC, in whole seconds where it falls on one.

 @param msOfDay - milliseconds after 00:00 UTC

 @returns Time as `HH:MM:SS`, or `HH:MM:SS.sss` where it has milliseconds

 @example
 ```ts
 neuralWattTimeOfDayShort({ msOfDay: 69_308_000, },); // '19:15:08'
 ```
 */
export function neuralWattTimeOfDayShort({ msOfDay, }: { readonly msOfDay: number; },): string {
  /**
   The time of day with its milliseconds.
   */
  const withMillis = neuralWattTimeOfDayUtc({ msOfDay, },);
  return withMillis.endsWith(WHOLE_SECOND_MILLIS,)
    ? withMillis.slice(
      0,
      -WHOLE_SECOND_MILLIS.length,
    )
    : withMillis;
}

/**
 Seconds between a failed read of the quota and the next attempt to read it for the same purpose: the cadence of the
 budget layer's meter readings, shared by the day anchor keeper and the quota session's starting reading.
 */
export const NEURALWATT_QUOTA_RETRY_SECONDS = 60;

/**
 An instant some seconds after another.

 @param from - the instant

 @param seconds - how many seconds later

 @returns The later instant

 @example
 ```ts
 neuralWattSecondsAfter({ from: new Date('2026-10-20T23:15:00.000Z',), seconds: 60, },);
 // 2026-10-20T23:16:00.000Z
 ```
 */
export function neuralWattSecondsAfter(
  {
    from,
    seconds,
  }: {
    readonly from: Date;
    readonly seconds: number;
  },
): Date {
  return new Date(Date.UTC(
    from.getUTCFullYear(),
    from.getUTCMonth(),
    from.getUTCDate(),
    from.getUTCHours(),
    from.getUTCMinutes(),
    from.getUTCSeconds() + seconds,
    from.getUTCMilliseconds(),
  ),);
}

//endregion NeuralWatt day
