import { durationWordsOf, } from './duration-words.ts';
import {
  neuralWattDayStartOfInstant,
  neuralWattTimeOfDayShort,
} from './neuralwatt-day.ts';
import type { NeuralWattDryness, } from './neuralwatt-return-hold.ts';

//region NeuralWatt return words
// THE WORDS THE REFUSAL THAT ENDS THE RUN GIVES FOR NEURALWATT: what the run knows of its day, why it reads dry, and
// when its energy comes back, only the true cause of each.

/**
 What the run knows of when NeuralWatt's free energy returns, for the refusal that ends the run to state.

 THREE CASES, because the answers differ: a run with no NeuralWatt client has no energy to return, a run whose client
 never learned its day start (no usable quota reading and no day-start setting) can name no time, and a run that
 decided the day start at launch states it. THE OWNER'S RULING OF 2026-10-20: the day is "not 00:00 UTC".

 @example
 ```ts
 const day: NeuralWattReturn = { kind: 'at', msOfDay: 69_308_000, };
 ```
 */
export type NeuralWattReturn =
  | {
    /**
     The day start was decided: it recurs at this time of day in UTC.
     */
    readonly kind: 'at';

    /**
     Milliseconds after 00:00 UTC the day starts at.
     */
    readonly msOfDay: number;
  }
    | {
    /**
     NeuralWatt is configured, but the run learned no day start.
     */
      readonly kind: 'unanchored';
    }
    | {
    /**
     NeuralWatt is off for this run: it has no client.
     */
      readonly kind: 'off';
    };

/**
 The time of day a spent NeuralWatt day starts again, as the refusal states it, from the reading's own day start where
 it has one and from the run's decided day otherwise.

 @param day - what the run knows of NeuralWatt's day

 @param dryness - why NeuralWatt reads dry, where the caller knows

 @returns Time of day with its zone as `text`, absent where neither source names one

 @example
 ```ts
 returnTimeOf({ day: { kind: 'at', msOfDay: 69_308_000, }, },); // => { text: '19:15:08 UTC' }
 ```
 */
export function returnTimeOf(
  {
    day,
    dryness,
  }: {
    readonly day: NeuralWattReturn;
    readonly dryness?: NeuralWattDryness;
  },
): { readonly text?: string; } {
  /**
   The time of day the reading's own day start gives, when the reading has one that parses.
   */
  const { msOfDay: fromReading, } = (dryness?.kind === 'spent-day')
    ? neuralWattDayStartOfInstant({ instant: dryness.dayStart, },)
    : {};
  /**
   The time of day, from the reading or else from the day the run decided.
   */
  const msOfDay = fromReading ?? ((day.kind === 'at') ? day.msOfDay : undefined);
  return (msOfDay === undefined) ? {} : { text: `${neuralWattTimeOfDayShort({ msOfDay, },)} UTC`, };
}

/**
 The time a blocked key lets requests through again, as a refusal states it: the reading's own text read as an instant
 and spelled in UTC, never the reply's text itself.

 @param until - instant as the quota reply spelled it

 @returns Instant spelled in UTC, or that the account stated none that reads as one

 @example
 ```ts
 keyUnblockTextOf({ until: '2026-10-23T05:00:00Z', },); // => '2026-10-23T05:00:00.000Z'
 ```
 */
export function keyUnblockTextOf({ until, }: { readonly until: string; },): string {
  /**
   Epoch milliseconds, NaN where the text is not an instant.
   */
  const epochMs = Date.parse(until,);
  return Number.isFinite(epochMs,) ? new Date(epochMs,).toISOString() : 'a time the account did not state';
}

/**
 The cause the refusal states: the one the caller named, else a spent day when the caller named only its distance.

 @param neuralwattDryness - why NeuralWatt reads dry, where the caller knows

 @param neuralwattReturnsInMs - milliseconds until a spent day starts again, where the caller counted them

 @returns Cause as `dryness`, absent where the caller named neither

 @example
 ```ts
 namedDrynessOf({ neuralwattReturnsInMs: 301_000, },); // => { dryness: { kind: 'spent-day', ... } }
 ```
 */
export function namedDrynessOf(
  {
    neuralwattDryness,
    neuralwattReturnsInMs,
  }: {
    readonly neuralwattDryness?: NeuralWattDryness;
    readonly neuralwattReturnsInMs?: number;
  },
): { readonly dryness?: NeuralWattDryness; } {
  if (neuralwattDryness !== undefined)
    return { dryness: neuralwattDryness, };
  if (neuralwattReturnsInMs === undefined)
    return {};
  return {
    dryness: {
      kind: 'spent-day',
      dayStart: '',
      returnsInMs: neuralwattReturnsInMs,
    },
  };
}

/**
 The opening clause about NeuralWatt, which says only what the run knows of it: why it reads dry when the caller knows,
 else spent when a day start is decided, refused until one is learned when none is, and absent when NeuralWatt is off
 and so has no energy to be out of.

 @param day - what the run knows of NeuralWatt's day

 @param dryness - why NeuralWatt reads dry, where the caller knows

 @returns Clause ending in a comma and a space, or nothing for an off provider

 @example
 ```ts
 openingClauseOf({ day: { kind: 'off', }, },); // => ''
 ```
 */
export function openingClauseOf(
  {
    day,
    dryness,
  }: {
    readonly day: NeuralWattReturn;
    readonly dryness?: NeuralWattDryness;
  },
): string {
  if (day.kind === 'off')
    return '';
  if (dryness?.kind === 'key-blocked')
    return 'NeuralWatt\'s key allowance is blocked, ';
  if (dryness?.kind === 'reserved')
    return 'NeuralWatt has no energy to spend while calls in flight hold what is left, ';
  if ((day.kind === 'unanchored') || (dryness?.kind === 'no-day-start'))
    return 'NeuralWatt is refused until its day start is learned, ';
  return 'NeuralWatt has no free daily energy left, ';
}

/**
 The clause saying when NeuralWatt's free energy returns, and only the true cause of it: the day start and the distance
 for a spent day, the key's own time for a blocked key, the calls in flight for energy they hold.

 @param day - what the run knows of NeuralWatt's day

 @param dryness - why NeuralWatt reads dry, where the caller knows

 @param afterWaiting - whether the call waited for the return before this refusal

 @returns Clause ending in a semicolon and a space

 @example
 ```ts
 returnClauseOf({ day: { kind: 'at', msOfDay: 69_308_000, }, afterWaiting: false, });
 // => 'NeuralWatt\'s free energy returns when its day starts again, at 19:15:08 UTC, the day start this run decided or
 // learned; '
 ```
 */
export function returnClauseOf(
  {
    day,
    dryness,
    afterWaiting,
  }: {
    readonly day: NeuralWattReturn;
    readonly dryness?: NeuralWattDryness;
    readonly afterWaiting: boolean;
  },
): string {
  if (day.kind === 'off')
    return 'NeuralWatt is off for this run, so it returns no energy; ';
  if (dryness?.kind === 'key-blocked')
    return `NeuralWatt's key allowance stays blocked until ${keyUnblockTextOf({ until: dryness.until, },)} `
      + 'and is not tied to the day start; ';
  if (dryness?.kind === 'reserved') {
    return 'NeuralWatt\'s remaining energy is held by calls in flight and comes back as they settle, not at the day '
      + 'start; ';
  }
  if ((day.kind === 'unanchored') || (dryness?.kind === 'no-day-start'))
    return 'NeuralWatt\'s day start was not decided for this run, so no time of day can be stated for its return; ';

  /**
   The time of day, whole seconds where the day starts on one.
   */
  const { text: timeOfDay = 'its day start', } = returnTimeOf({
    day,
    ...((dryness === undefined) ? {} : { dryness, }),
  },);
  if ((dryness?.kind === 'spent-day') && (dryness.returnsInMs === undefined)) {
    return `NeuralWatt's day started again at ${timeOfDay}, but its account still reads the day `
      + `spent${afterWaiting ? ' after waiting for it' : ''}; `;
  }
  /**
   The distance to the return, where the caller counted it down.
   */
  const distance = (dryness?.kind === 'spent-day') && (dryness.returnsInMs !== undefined)
    ? ` (in ${durationWordsOf({ ms: dryness.returnsInMs, },)})`
    : '';
  return `NeuralWatt's free energy returns when its day starts again, at ${timeOfDay}${distance}, `
    + 'the day start this run decided or learned; ';
}

//endregion NeuralWatt return words
