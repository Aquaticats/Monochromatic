/**
 Time-category duration formatters that fall outside `Intl.DurationFormat`.
 
 Two distinct shapes live here:
 
 - {@link formatDuration}: magnitude-adaptive sub-millisecond / ms / s
   for `performance.now()` deltas (test summaries, perf traces).
 - {@link formatTrackedDuration}: ultra-compact human-friendly ladder
   from seconds to years for accumulated tracked work time
   (productivity-app chip text).
 
 Both are gaps in `Intl.DurationFormat`: sub-ms precision is not
 representable (integer Duration Record slots, sync'd with
 `Temporal.Duration`; see proposal-intl-duration-format#199 and
 ecma402#980); magnitude-driven unit selection is out of charter and
 was punted to a hypothetical `Intl.RelativeTimeFormat` v2 (ecma402#498,
 still open); the `'narrow'` style produces space-separated, locale-
 specific suffixes (`Xh Ym` with literal space, `週` in Japanese)
 whereas the app needs ASCII compact without spaces; and the 30-day /
 365-day month / year decomposition is app-domain, not Intl-default.
 
 @module
 */

import {
  DAYS_PER_WEEK,
  DAYS_PER_YEAR,
  MS_PER_SECOND,
  SECONDS_PER_DAY,
  SECONDS_PER_HOUR,
  SECONDS_PER_MINUTE,
} from '@monochromatic-dev/const-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

/**
 Cutoff below which a duration renders with one decimal place. Above
 this, decimals add noise without adding signal (a 51.23ms test is not
 more informative than 51ms when the unit is "ms").
 */
const DECIMAL_BELOW_MS = 10;

/**
 Format an elapsed duration in milliseconds for human-readable logs.
 
 - below {@link DECIMAL_BELOW_MS}: one decimal, e.g. `0.3ms`, `9.9ms`
 - {@link DECIMAL_BELOW_MS} to 999ms: whole ms, e.g. `51ms`, `999ms`
 - {@link MS_PER_SECOND} and above: one decimal in seconds, e.g. `1.2s`
 
 @param durationMs - elapsed time in milliseconds, typically from a
   `performance.now()` delta
 
 @returns formatted duration with unit suffix
 
 @example
 formatDuration(0.34); // "0.3ms"
 formatDuration(9.9); // "9.9ms"
 formatDuration(51); // "51ms"
 formatDuration(1234); // "1.2s"
 */
export function formatDuration(durationMs: number,): string {
  if (durationMs < DECIMAL_BELOW_MS)
    return `${durationMs.toFixed(1,)}ms`;
  if (durationMs < MS_PER_SECOND)
    return `${durationMs.toFixed(0,)}ms`;
  return `${(durationMs / MS_PER_SECOND).toFixed(1,)}s`;
}

/**
 Days assumed per calendar month for tracked-time decomposition. Real
 calendar months range 28 to 31; productivity-app convention rounds to
 30 so `30d` of accumulated work renders as `1m0w` and the user does
 not have to mentally convert. Approximation is intentional and lives
 here (call site) rather than in `module-const` so it is
 visible to readers of this formatter.
 */
const DAYS_PER_MONTH = 30;

/**
 Seconds in one week, derived from {@link SECONDS_PER_DAY} and
 {@link DAYS_PER_WEEK}. Exact (no approximation: 1 week = 7 days
 across all calendars).
 */
const SECONDS_PER_WEEK = SECONDS_PER_DAY * DAYS_PER_WEEK;

/**
 Seconds in one month, derived from {@link SECONDS_PER_DAY} and
 {@link DAYS_PER_MONTH}. Approximation: 30 days. Creates a small cliff
 near the year boundary; see {@link formatTrackedDuration}.
 */
const SECONDS_PER_MONTH = SECONDS_PER_DAY * DAYS_PER_MONTH;

/**
 Seconds in one year, derived from {@link SECONDS_PER_DAY} and
 {@link DAYS_PER_YEAR}. Approximation: 365 days (matches the convention
 documented on {@link DAYS_PER_YEAR} itself).
 */
const SECONDS_PER_YEAR = SECONDS_PER_DAY * DAYS_PER_YEAR;

/**
 Format an integer-seconds duration as ultra-compact ASCII suitable for
 productivity-app chip text. Renders top-1 (when only seconds is
 non-zero) or top-2 in strict adjacency with the next smaller unit
 (when any larger unit is non-zero). Single-letter suffix per unit
 (`y`, `m`, `w`, `d`, `h`, `m`, `s`); the secondary always
 disambiguates the reused `m`: `1y2m` is years + months because of
 the `y`, `1h30m` is hours + minutes because of the `h`, and standalone
 `Xm` never occurs because seconds-only renders as `Xs`.
 
 Magnitude ladder (each row pairs the largest non-zero unit with the
 next smaller unit; weeks/days/hours/minutes/seconds are exact, months
 approximate at {@link DAYS_PER_MONTH} days, years approximate at
 {@link DAYS_PER_YEAR} days):
 
 - all zero       → `0s`
 - seconds only   → `Xs`
 - minutes        → `XmYs`     (m means minutes; partner is seconds)
 - hours          → `XhYm`     (m means minutes; partner is hours)
 - days           → `XdYh`
 - weeks          → `XwYd`
 - months         → `XmYw`     (m means months; partner is weeks)
 - years          → `XyYm`     (m means months; partner is years)
 
 Strict adjacency loses precision when the secondary unit is zero but
 a tertiary is non-zero (e.g. 35 days renders `1m0w` and the trailing
 5 days are dropped). Acceptable cost for visual stability during live
 timer ticks. Near the year boundary (12 months = 360d vs 1 year =
 365d) the same task class can render `12m0w` at 12 months and
 `1y0m` at 13 months due to the 30d / 365d approximations not being
 internally consistent (12 × 30 ≠ 365).
 
 @param seconds - non-negative duration in seconds; negative or
   fractional inputs clamp to a non-negative integer
 
 @returns ultra-compact duration string
 
 @example
 formatTrackedDuration(0); // "0s"
 formatTrackedDuration(45); // "45s"
 formatTrackedDuration(90); // "1m30s"
 formatTrackedDuration(5400); // "1h30m"
 formatTrackedDuration(263_400); // "3d1h"  (= 3 days, 1 hour, 10 minutes; trailing minutes dropped)
 formatTrackedDuration(1_468_800); // "2w3d"
 formatTrackedDuration((365 + 60) * 86_400); // "1y2m"
 */
export function formatTrackedDuration(seconds: number,): string {
  /**
   Clamped to non-negative integer seconds so the modulo chain below cannot consume a fraction or produce negative quotients.
   */
  const total = Math.max(
    0,
    Math.floor(seconds,),
  );

  /**
   Top of the ladder, in units of {@link SECONDS_PER_YEAR}; carved off
   first so subsequent units operate on a descending residual.
   */
  const years = Math.floor(total / SECONDS_PER_YEAR,);
  /**
   Residual after dividing out {@link SECONDS_PER_YEAR}; fed to months.
   */
  const remAfterY = total % SECONDS_PER_YEAR;
  /**
   Carved off the post-years residual using the 30-day approximation in {@link SECONDS_PER_MONTH}.
   */
  const months = Math.floor(remAfterY / SECONDS_PER_MONTH,);
  /**
   Residual after dividing out {@link SECONDS_PER_MONTH}; fed to weeks.
   */
  const remAfterMo = remAfterY % SECONDS_PER_MONTH;
  /**
   Carved off the post-months residual, in units of {@link SECONDS_PER_WEEK}.
   */
  const weeks = Math.floor(remAfterMo / SECONDS_PER_WEEK,);
  /**
   Residual after dividing out {@link SECONDS_PER_WEEK}; fed to days.
   */
  const remAfterW = remAfterMo % SECONDS_PER_WEEK;
  /**
   Carved off the post-weeks residual, in units of {@link SECONDS_PER_DAY}.
   */
  const days = Math.floor(remAfterW / SECONDS_PER_DAY,);
  /**
   Residual after dividing out {@link SECONDS_PER_DAY}; fed to hours.
   */
  const remAfterD = remAfterW % SECONDS_PER_DAY;
  /**
   Carved off the post-days residual, in units of {@link SECONDS_PER_HOUR}.
   */
  const hours = Math.floor(remAfterD / SECONDS_PER_HOUR,);
  /**
   Residual after dividing out {@link SECONDS_PER_HOUR}; fed to minutes.
   */
  const remAfterH = remAfterD % SECONDS_PER_HOUR;
  /**
   Carved off the post-hours residual, in units of {@link SECONDS_PER_MINUTE}.
   */
  const minutes = Math.floor(remAfterH / SECONDS_PER_MINUTE,);
  /**
   Tail of the ladder; whatever does not fit in {@link SECONDS_PER_MINUTE} above.
   */
  const remSeconds = remAfterH % SECONDS_PER_MINUTE;

  /**
   Paired (value, suffix) tuples ordered largest-to-smallest so a linear `findIndex` locates the top of the ladder.
   */
  const UNITS = [
    [
      years,
      'y',
    ],
    [
      months,
      'm',
    ],
    [
      weeks,
      'w',
    ],
    [
      days,
      'd',
    ],
    [
      hours,
      'h',
    ],
    [
      minutes,
      'm',
    ],
    [
      remSeconds,
      's',
    ],
  ] as const;

  /**
   Index of the first non-zero unit; defines the top of the rendered two-unit pair.
   */
  const biggestIdx = UNITS.findIndex(function isNonZero([value,],) {
    return value > 0;
  },);

  if (biggestIdx === (-1))
    return '0s';

  /**
   Top unit; primary cell of the rendered string. Extracted with
   {@link nonNullishOrThrow}, which is unreachable here because
   `biggestIdx` was just bounds-checked above.
   */
  const [bigValue, bigSuffix,] = nonNullishOrThrow(UNITS[biggestIdx],);

  if (biggestIdx === (UNITS.length
    - 1))
    return `${bigValue}${bigSuffix}`;

  /**
   Partner unit; immediately adjacent per the strict-adjacency rule (no
   skipping). Extracted with {@link nonNullishOrThrow}, which is
   unreachable here because the prior return covers the last-index case.
   */
  const [smallValue, smallSuffix,] = nonNullishOrThrow(UNITS[biggestIdx + 1],);
  return `${bigValue}${bigSuffix}${smallValue}${smallSuffix}`;
}
