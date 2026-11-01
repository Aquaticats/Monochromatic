import {
  type InFlight,
  type RunTiming,
  summariseRounds,
} from './run-timing-read.ts';

//region Run timing report print
// What the run timing report says, one function per part: a span in the unit
// it fills, the rounds a log reported and the calls it had in flight.
//
// SPLIT OUT OF `run-timing-report.ts` so the entry holds only the wiring and
// each printer is read through its own cases. Each prints with `console.log`,
// which is the report.

/**
 Milliseconds in a second.
 */
const MS_PER_SECOND = 1_000;

/**
 Milliseconds in a minute.
 */
const MS_PER_MINUTE = 60_000;

/**
 Milliseconds in an hour.
 */
const MS_PER_HOUR = 3_600_000;

/**
 Multiplier turning a fraction into a percentage.
 */
const PERCENT = 100;

/**
 Decimal places every span column carries.
 */
const SPAN_PLACES = 2;

/**
 Decimal places the mean-in-flight column carries.
 */
const MEAN_PLACES = 2;

/**
 Decimal places the grace-share column carries.
 */
const SHARE_PLACES = 1;

/**
 Renders a span in the largest unit it fills.

 THREE UNITS RATHER THAN HOURS ALONE. The same report reads a six-hour corpus
 pass and a thirty-second probe, and printing both in hours prints the probe
 as `0.01h`, which is indistinguishable from a run that did nothing. Choosing
 the unit per figure keeps a short span legible without making a long one
 unreadable.

 @param ms - span to render

 @returns Text for a report column

 @example
 ```ts
 console.log(asSpan({ ms: 22_140_000, },),);
 ```
 */
export function asSpan({ ms, }: { readonly ms: number; },): string {
  if (ms >= MS_PER_HOUR)
    return `${(ms / MS_PER_HOUR).toFixed(SPAN_PLACES,)}h`;
  if (ms >= MS_PER_MINUTE)
    return `${(ms / MS_PER_MINUTE).toFixed(SPAN_PLACES,)}min`;
  return `${(ms / MS_PER_SECOND).toFixed(SPAN_PLACES,)}s`;
}

/**
 Reports what the rounds spent, split into work and waiting.

 @param reading - every timing line the log held

 @example
 ```ts
 printRounds({ reading, },);
 ```
 */
export function printRounds({ reading, }: { readonly reading: RunTiming; },): void {
  /**
   How many rounds the log reported.
   */
  const roundCount = reading
    .rounds
    .length;

  if (roundCount === 0) {
    console.log(
      'NO ROUND LINE. This log predates round lines and call durations, so how long each fan-out took and how much of that '
        + 'was spent waiting after quorum are both unrecorded. That is not the same as a run that '
        + 'never waited.',
    );
    return;
  }

  /**
   Totals across every round, of either kind (T8, nineteenth batch).
   */
  const summary = summariseRounds({ rounds: reading.rounds, },);

  console.log(
    `rounds                 ${String(roundCount,)}, `
      + `${asSpan({ ms: summary.totalMs, },)} in total`,
  );
  console.log(`  without quorum       ${String(summary.withoutQuorum,)}`,);
  // NO SHARE OF NO TIME: rounds that all took no time leave nothing to divide
  // the grace by, and a NaN percentage reads as a measurement.
  /**
   Share of round time spent waiting rather than working, where there was any.
   */
  const graceShare = (summary.totalMs === 0)
    ? 'no round time to share'
    : `${((summary.graceMs / summary.totalMs) * PERCENT).toFixed(SHARE_PLACES,)}% of round time`;

  console.log(`  waiting after quorum ${asSpan({ ms: summary.graceMs, },)}, ${graceShare}`,);
  console.log(`  voices never heard   ${String(summary.lost,)}`,);
}

/**
 Reports how many calls the run had in flight.

 @param flight - what the sweep counted

 @example
 ```ts
 printInFlight({ flight, },);
 ```
 */
export function printInFlight({ flight, }: { readonly flight: InFlight; },): void {
  /**
   Mean in flight at the precision a fan-out is read in.
   */
  const meanShown = flight
    .meanInFlight
    .toFixed(MEAN_PLACES,);

  console.log(
    `calls in flight        mean ${meanShown}, `
      + `peak ${String(flight.peakInFlight,)}`,
  );
  console.log(
    `  busy against span    ${asSpan({ ms: flight.busyMs, },)} of calls `
      + `across ${asSpan({ ms: flight.spanMs, },)} of run`,
  );
}

//endregion Run timing report print
