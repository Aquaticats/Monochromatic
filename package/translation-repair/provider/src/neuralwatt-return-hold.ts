import {
  MS_PER_DAY,
  neuralWattDayStartOfInstant,
} from './neuralwatt-day.ts';
import type { NeuralWattEnergy, } from './neuralwatt-energy.ts';
import { NEURALWATT_SMALLEST_ESTIMATE_WH, } from './neuralwatt-estimate.ts';
import { neuralWattIsDry, } from './neuralwatt-meter.ts';

//region NeuralWatt return hold
// WHY A NEURALWATT READING IS DRY, and when it comes back. The case is XingZ6019 (2026-10-22): NeuralWatt dry at
// 19:14:20 UTC because the day's 550 Wh was spent, its day restarting at 19:15:08 UTC, 48 s later; the bench stood
// short of its floor and the entry stopped 48 s before the energy came back. A NeuralWatt dry reading carries its
// cause, and a spent day carries the time of day it starts again, which the refusal that ends the run states so the
// operator knows when to relaunch. NOTHING WAITS FOR THE RETURN (the owner's answer of 2026-10-29, "Never wait,
// relaunch": "Stop at once and relaunch later."): it is not a hold, so the router, the seat reader and the pass stop
// at once when NeuralWatt is the only way forward.
//
// THE DAY STARTS AT A TIME OF DAY IN UTC (`neuralwatt-day.ts`), and a UTC day is exactly 24 hours, so the distance
// is the clock's time of day to the day start's time of day, around midnight where it must be. No instant of the wall
// clock is read as a number (`source-scan/src/wall-clock-reads.unit.test.ts`), and no calendar is needed.

/**
 Milliseconds added to a return, so a reading taken a hair before the day starts, or a day start given to the second,
 still reads the new day: one second, the granularity of the quota's `current_period_start`.
 */
export const NEURALWATT_RETURN_MARGIN_MS = 1_000;

/**
 Why a NeuralWatt reading is dry, with what each cause comes back at.

 FIVE CAUSES, because the words and the waits differ: a spent day comes back when the day starts again (the distance
 carries the margin, and is absent where the account still reports a day that should have ended, which names no
 return), a blocked key at its own time, energy held by calls in flight when they settle, and an unknown day start when
 one is learned.

 @example
 ```ts
 const dryness: NeuralWattDryness = { kind: 'spent-day', dayStart: '2026-10-22T19:15:08.000Z', returnsInMs: 48_000, };
 ```
 */
export type NeuralWattDryness =
  | {
    /**
     NeuralWatt is not dry, or its meter was not read.
     */
    readonly kind: 'wet';
  }
    | {
    /**
     The day's energy is spent and the day starts again.
     */
      readonly kind: 'spent-day';

    /**
     First instant of the spent day, as the reading spells it.
     */
      readonly dayStart: string;

    /**
     Milliseconds until the day starts again, margin included; absent where the account still reports the day that
     should have ended.
     */
      readonly returnsInMs?: number;
    }
    | {
    /**
     The key's allowance is blocked and lets requests through again at its own time.
     */
      readonly kind: 'key-blocked';

    /**
     When the block lifts, as the reading spells it.
     */
      readonly until: string;
    }
    | {
    /**
     The energy the settled calls leave would pay for a call, and the calls in flight hold it.
     */
      readonly kind: 'reserved';
    }
    | {
    /**
     The reading has no day start, so nothing can be sent.
     */
      readonly kind: 'no-day-start';
    };

/**
 What a budget layer says of NeuralWatt's dryness beside its holds: the cause the last reading found.

 @example
 ```ts
 const cause: NeuralWattCause = budgets;
 ```
 */
export type NeuralWattCause = {
  /**
   Why NeuralWatt reads dry as the last reading found it, which the refusal states as the cause: a spent day with the
   distance counted down to its return, a blocked key with its own time, energy held by calls in flight, or no day
   start. `wet` where NeuralWatt is not dry or has no client.
   */
  readonly neuralwattDryness: () => NeuralWattDryness;
};

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
 Milliseconds since 00:00 UTC at an instant.

 @param at - instant

 @returns Milliseconds after the UTC midnight that began its day

 @example
 ```ts
 msOfDayUtc({ at: new Date('2026-10-23T19:14:20.000Z',), },); // 69_260_000
 ```
 */
function msOfDayUtc({ at, }: { readonly at: Date; },): number {
  return (at.getUTCHours() * MS_PER_HOUR)
    + (at.getUTCMinutes() * MS_PER_MINUTE)
    + (at.getUTCSeconds() * MS_PER_SECOND)
    + at.getUTCMilliseconds();
}

/**
 Why NeuralWatt reads dry, and for a spent day how long until it starts again.

 NOTHING FOR WHAT A DAY START DOES NOT RETURN: a day with energy left is not dry, an unknown day start names no time,
 and a blocked key's allowance comes back at its own time (`keyBlockedUntil`) and not at the day start. A READING DRY
 ONLY THROUGH THE CALLS IN FLIGHT NAMES NO RETURN either: the settled spend alone leaves what a call needs, and the
 holds release in seconds, so waiting to the day start would wait hours for energy that is there. A day that has just
 started reads as a whole day away, since the energy of that day is what is spent.

 @param energy - reading the dryness was drawn from

 @param now - wall clock's instant, whose time of day the distance is taken from

 @param marginMs - milliseconds added to a return that is waited for

 @returns Cause, with the distance to the return and the margin for a spent day

 @example
 ```ts
 neuralWattDrynessOf({ energy, now: new Date('2026-10-23T19:14:20.000Z',), marginMs: 0, },);
 // => { kind: 'spent-day', dayStart: '2026-10-22T19:15:08.000Z', returnsInMs: 48_000, }
 ```
 */
export function neuralWattDrynessOf(
  {
    energy,
    now,
    marginMs,
  }: {
    readonly energy: NeuralWattEnergy;
    readonly now: Date;
    readonly marginMs: number;
  },
): NeuralWattDryness {
  if (!neuralWattIsDry({ energy, },))
    return { kind: 'wet', };
  if (energy.dayStart === undefined)
    return { kind: 'no-day-start', };
  if (energy.keyBlockedUntil !== undefined) {
    return {
      kind: 'key-blocked',
      until: energy.keyBlockedUntil,
    };
  }
  if ((energy.remainingWh + energy.reservedWh) >= NEURALWATT_SMALLEST_ESTIMATE_WH)
    return { kind: 'reserved', };

  /**
   The time of day the day starts at, when the reading's day start parses.
   */
  const { msOfDay, } = neuralWattDayStartOfInstant({ instant: energy.dayStart, },);
  if (msOfDay === undefined)
    return { kind: 'no-day-start', };

  /**
   Milliseconds from the clock's time of day to the next time of day the day starts at, zero meaning it starts now.
   */
  const untilStart = (((msOfDay - msOfDayUtc({ at: now, },)) % MS_PER_DAY) + MS_PER_DAY) % MS_PER_DAY;
  return {
    kind: 'spent-day',
    dayStart: energy.dayStart,
    returnsInMs: ((untilStart === 0) ? MS_PER_DAY : untilStart) + marginMs,
  };
}

//endregion NeuralWatt return hold
