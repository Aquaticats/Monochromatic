import type { DrySpan, } from './meter-dry-span.ts';

//region Meter report text
// How the meter report writes a span of time, an instant and the longest
// outage a provider had, with its open ends named.
//
// SPLIT OUT OF `meter-report.ts` so the entry holds only the wiring and each
// writer is read through its own cases.

/**
 Milliseconds in each unit a duration is rendered in.
 */
const UNIT_MS = {
  h: 3_600_000,
  m: 60_000,
  s: 1_000,
} as const;

/**
 Renders a duration in hours, minutes and seconds, dropping empty leaders.

 @param ms - duration to render

 @returns Compact duration, `0s` for nothing

 @example
 ```ts
 spanText({ ms: 3_720_000, },);
 // => '1h2m'
 ```
 */
export function spanText(
  { ms, }: { readonly ms: number; },
): string {
  /**
   Whole hours the duration covers.
   */
  const hours = Math.floor(ms / UNIT_MS.h,);

  /**
   Whole minutes left after those hours.
   */
  const minutes = Math.floor((ms % UNIT_MS.h) / UNIT_MS.m,);

  /**
   Whole seconds left after those minutes.
   */
  const seconds = Math.floor((ms % UNIT_MS.m) / UNIT_MS.s,);

  /**
   Units that carry anything, so `2h0m5s` reads as `2h5s`.
   */
  const carried = ((hours > 0) ? `${String(hours,)}h` : '')
    + ((minutes > 0) ? `${String(minutes,)}m` : '')
    + ((seconds > 0) ? `${String(seconds,)}s` : '');

  if (carried === '')
    return '0s';

  return carried;
}

/**
 Renders an epoch stamp the way the log wrote it.

 @param at - epoch milliseconds

 @returns ISO stamp

 @example
 ```ts
 stampText({ at, },);
 ```
 */
export function stampText(
  { at, }: { readonly at: number; },
): string {
  return new Date(at,).toISOString();
}

/**
 Renders what is known about the longest outage, bounds and open ends both.

 NAMES AN OPEN END RATHER THAN PRINTING A NUMBER FOR IT. A stretch with no
 wet reading before it may have started before the record; one with none
 after it may still be running now. Either way the upper bound is not a
 number, and printing the confirmed length alone would read as the whole
 outage.

 @param span - longest stretch, absent where none was found

 @returns Lines describing it

 @example
 ```ts
 for (const line of outageLines({ span, },)) console.log(line,);
 ```
 */
export function outageLines(
  { span, }: { readonly span: DrySpan | 'no-outage'; },
): readonly string[] {
  if (span === 'no-outage')
    return ['  longest outage: none, no reading found this provider out',];

  /**
   How the upper bound reads, which is a number only when both ends closed.
   */
  const upper = (span.boundedByMs === undefined)
    ? 'and NOT BOUNDED ABOVE'
    : `at most ${spanText({ ms: span.boundedByMs, },)}`;

  /**
   Why the bound is open, named so the reader knows which way to doubt it.
   */
  const openness = [
    ...(span.openBefore
      ? ['    no reading before it found this provider up, so it may have begun earlier than the record',]
      : []),
    ...(span.openAfter
      ? ['    no reading after it found this provider up, so it may still be running',]
      : []),
  ];

  return [
    `  longest outage: at least ${spanText({ ms: span.confirmedMs, },)}, ${upper} `
      + `(${stampText({ at: span.firstAt, },)} .. ${stampText({ at: span.lastAt, },)})`,
    ...openness,
  ];
}

//endregion Meter report text
