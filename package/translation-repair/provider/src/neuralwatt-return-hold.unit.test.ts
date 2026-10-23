/**
 Tests for the cause of a NeuralWatt dry reading and, for a spent day, how long until its day starts again.

 THE DAY STARTS AT A TIME OF DAY IN UTC (`neuralwatt-day.ts`), so the distance is from the clock's time of day to the
 energy reading's day-start time of day, around midnight where it must be. No calendar and no instant of the wall clock
 is read as a number.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import {
  NEURALWATT_RETURN_MARGIN_MS,
  neuralWattDrynessOf,
  type NeuralWattDryness,
} from '../dist/final/node/index.mjs';

/**
 A reading of a day whose energy is spent, the day starting at 19:15:08 UTC.
 */
const SPENT_DAY = {
  dayStart: '2026-10-22T19:15:08.000Z',
  dailyWh: 550,
  spentWh: 550,
  reckonedWh: 0,
  reservedWh: 0,
  remainingWh: 0,
  calls: 40,
} as const;

/**
 A spent day that starts again after so many milliseconds.

 @param returnsInMs - distance, margin included

 @returns Cause

 @example
 ```ts
 const dryness = spentDayIn({ returnsInMs: 48_000, },);
 ```
 */
function spentDayIn({ returnsInMs, }: { readonly returnsInMs: number; },): NeuralWattDryness {
  return {
    kind: 'spent-day',
    dayStart: SPENT_DAY.dayStart,
    returnsInMs,
  };
}

/**
 The cause for a spent day at a time, with no margin.

 @param at - wall clock's instant, in UTC

 @returns Cause

 @example
 ```ts
 const dryness = drynessAt({ at: '2026-10-23T19:14:20.000Z', },);
 ```
 */
function drynessAt({ at, }: { readonly at: string; },): NeuralWattDryness {
  return neuralWattDrynessOf({
    energy: SPENT_DAY,
    now: new Date(at,),
    marginMs: 0,
  },);
}

await describe({
  name: neuralWattDrynessOf.name,
  concurrency: 1,
  children: [
    it({
      name: 'IS THE 48 SECONDS XINGZ6019 STOOD SHORT FOR: spent at 19:14:20 UTC, the day starting again '
        + 'at 19:15:08 UTC',
      fn: async () => {
        expect(drynessAt({ at: '2026-10-23T19:14:20.000Z', },),).toEqual(spentDayIn({ returnsInMs: 48_000, },),);
        expect(drynessAt({ at: '2026-10-22T19:14:20.000Z', },),).toEqual(spentDayIn({ returnsInMs: 48_000, },),);
      },
    },),

    it({
      name: 'ADDS THE MARGIN to the distance, so a timer that fires a hair early still reads the new day',
      fn: async () => {
        expect(neuralWattDrynessOf({
          energy: SPENT_DAY,
          now: new Date('2026-10-23T19:14:20.000Z',),
          marginMs: NEURALWATT_RETURN_MARGIN_MS,
        },),).toEqual(spentDayIn({ returnsInMs: 48_000 + NEURALWATT_RETURN_MARGIN_MS, },),);
      },
    },),

    it({
      name: 'WRAPS PAST MIDNIGHT: 23:59:50 UTC with a day starting at 00:00:10 UTC is 20 seconds',
      fn: async () => {
        expect(neuralWattDrynessOf({
          energy: { ...SPENT_DAY, dayStart: '2026-10-22T00:00:10.000Z', },
          now: new Date('2026-10-22T23:59:50.000Z',),
          marginMs: 0,
        },),).toEqual({ kind: 'spent-day', dayStart: '2026-10-22T00:00:10.000Z', returnsInMs: 20_000, },);
      },
    },),

    it({
      name: 'READS A DAY THAT HAS JUST STARTED AS A WHOLE DAY AWAY, since the energy of that day is what is spent',
      fn: async () => {
        expect(drynessAt({ at: '2026-10-23T19:15:08.000Z', },),).toEqual(spentDayIn({ returnsInMs: 86_400_000, },),);
      },
    },),

    it({
      name: 'NAMES THE DISTANCE AT ANY RANGE, since the bound a caller waits within is the caller\'s and not the '
        + 'reading\'s: a return 23 hours away is a return',
      fn: async () => {
        expect(drynessAt({ at: '2026-10-22T20:15:08.000Z', },),).toEqual(spentDayIn({ returnsInMs: 82_800_000, },),);
      },
    },),

    it({
      name: 'SAYS WET for a day with energy left',
      fn: async () => {
        expect(neuralWattDrynessOf({
          energy: { ...SPENT_DAY, remainingWh: 400, },
          now: new Date('2026-10-23T19:14:20.000Z',),
          marginMs: 0,
        },),).toEqual({ kind: 'wet', },);
      },
    },),

    it({
      name: 'NAMES THE TRUE CAUSE where the energy is not what is spent: no day start, a blocked key and energy held '
        + 'by calls in flight, none of which comes back at the day start',
      fn: async () => {
        /**
         The instant 48 seconds before the day starts again.
         */
        const now = new Date('2026-10-23T19:14:20.000Z',);
        const { dayStart, ...unanchored } = SPENT_DAY;
        expect(dayStart,).toBe(SPENT_DAY.dayStart,);
        expect([
          neuralWattDrynessOf({
            energy: unanchored,
            now,
            marginMs: 0,
          },),
          neuralWattDrynessOf({
            energy: { ...SPENT_DAY, keyBlockedUntil: '2026-10-23T20:00:00.000Z', },
            now,
            marginMs: 0,
          },),
          neuralWattDrynessOf({
            energy: { ...SPENT_DAY, spentWh: 100, reservedWh: 449.9995, remainingWh: 0.0005, },
            now,
            marginMs: 0,
          },),
        ],).toEqual([
          { kind: 'no-day-start', },
          { kind: 'key-blocked', until: '2026-10-23T20:00:00.000Z', },
          { kind: 'reserved', },
        ],);
      },
    },),
  ],
},);
