/**
 Tests for the clock time and attempt a lifecycle line carries (ledger B960): the instant in the spelling
 `Date.prototype.toISOString` writes, which `isIsoStampText` reads back, and the attempt as a whole number from 1.

 The clock is injected, so no case waits on the real one; the cases that ask for the default clock compare what it
 reads with a fixed floor and never read a clock of their own.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isIsoStampText,
  LifecycleAttemptError,
  lifecycleClockStamp,
  lifecycleStamp,
  wallClockNow,
} from '../dist/final/node/index.mjs';

/**
 Instant the injected clock reads.
 */
const PRINTED_AT = new Date('2026-10-22T08:10:11.123Z',);

/**
 An instant before every run of this test, in the spelling of a stamp, which no wall clock reads earlier than and no
 reading counted from the start of a process (the 1970 instants `performance.now()` gives) reaches.
 */
const WALL_CLOCK_FLOOR = '2026-01-01T00:00:00.000Z';

/**
 Clock that reads the same instant every time it is asked.

 @returns Instant the line is printed at

 @example
 ```ts
 const stamp = lifecycleStamp({ attempt: 2, wallClock: fixedWallClock, },);
 ```
 */
function fixedWallClock(): Date {
  return PRINTED_AT;
}

/**
 Cases of the clock time alone.
 */
const clockStampSuite = describe({
  name: lifecycleClockStamp.name,
  children: [
    it({
      name: 'WRITES THE CLOCK TIME ALONE, in the spelling the stamp with an attempt uses, for a line that belongs to '
        + 'the pass and to no attempt',
      fn: async () => {
        expect(lifecycleClockStamp({ wallClock: fixedWallClock, },),).toBe('at=2026-10-22T08:10:11.123Z',);
        expect(lifecycleStamp({
          attempt: 3,
          wallClock: fixedWallClock,
        },),).toBe(`${lifecycleClockStamp({ wallClock: fixedWallClock, },)} attempt=3`,);
      },
    },),
    it({
      name: 'READS THE WALL CLOCK when it is given no clock of its own, a time after the fixed floor',
      fn: async () => {
        /**
         Time the stamp names, cut off its key.
         */
        const time = lifecycleClockStamp({})
          .slice('at='.length,);
        expect(isIsoStampText({ text: time, },),).toBe(true,);
        expect(time > WALL_CLOCK_FLOOR,).toBe(true,);
      },
    },),
  ],
},);

/**
 Cases of the clock time and the attempt.
 */
const stampSuite = describe({
  name: lifecycleStamp.name,
  children: [
    it({
      name: 'WRITES THE CLOCK TIME IN THE LOGGER\'S SPELLING and the attempt after it, as two key=value fields',
      fn: async () => {
        expect(lifecycleStamp({
          attempt: 2,
          wallClock: fixedWallClock,
        },),).toBe('at=2026-10-22T08:10:11.123Z attempt=2',);
      },
    },),
    it({
      name: 'WRITES A TIME THAT isIsoStampText READS BACK, milliseconds kept even when they are zero',
      fn: async () => {
        /**
         The stamp's time, cut off its key.
         */
        const time = lifecycleStamp({
          attempt: 1,
          wallClock: () => new Date('2026-10-22T08:10:11.000Z',),
        },)
          .split(' ',)[0]
          ?.slice('at='.length,) ?? '';
        expect(time,).toBe('2026-10-22T08:10:11.000Z',);
        expect(isIsoStampText({ text: time, },),).toBe(true,);
      },
    },),
    it({
      name: 'READS THE CLOCK WHEN ASKED, not when the stamp function is made, so two stamps name two instants',
      fn: async () => {
        /**
         Instants the clock reads, first first.
         */
        const instants = [
          new Date('2026-10-22T08:10:11.123Z',),
          new Date('2026-10-22T08:10:12.456Z',),
        ];
        /**
         Clock that reads each instant once.
         */
        function wallClock(): Date {
          return instants.shift() ?? PRINTED_AT;
        }
        expect([
          lifecycleStamp({ attempt: 1, wallClock, },),
          lifecycleStamp({ attempt: 1, wallClock, },),
        ],).toEqual([
          'at=2026-10-22T08:10:11.123Z attempt=1',
          'at=2026-10-22T08:10:12.456Z attempt=1',
        ],);
      },
    },),
    it({
      name: 'READS THE WALL CLOCK when it is given no clock of its own, a time after a fixed floor in UTC, which a '
        + 'monotonic reading counted from the process start (an instant in 1970) is not (ledger B78)',
      fn: async () => {
        /**
         Time the stamp names, cut off its key.
         */
        const time = lifecycleStamp({ attempt: 1, },)
          .split(' ',)[0]
          ?.slice('at='.length,) ?? '';
        expect(isIsoStampText({ text: time, },),).toBe(true,);
        expect(time.endsWith('Z',),).toBe(true,);
        // The text sorts after the floor because both spell an instant the same way, most significant part first.
        expect(time > WALL_CLOCK_FLOOR,).toBe(true,);
        expect(Date.parse(time,) > Date.parse(WALL_CLOCK_FLOOR,),).toBe(true,);
        expect(new Date(time,)
          .toISOString(),).toBe(time,);
      },
    },),
    it({
      name: 'READS AN INSTANT AFTER THE FLOOR from the wall clock it names, so the stamp and the clock cannot agree '
        + 'on a reading that is not the wall',
      fn: async () => {
        expect(wallClockNow()
          .toISOString() > WALL_CLOCK_FLOOR,).toBe(true,);
      },
    },),
    it({
      name: 'REFUSES AN ATTEMPT THAT IS NOT A WHOLE NUMBER FROM 1, naming it, since the pass counts an attempt before '
        + 'it starts and a line printing 0, a fraction or NaN would name an attempt that does not exist',
      fn: async () => {
        for (
          const attempt of [
            0,
            -1,
            1.5,
            Number.NaN,
            Number.POSITIVE_INFINITY,
          ]
        ) {
          /**
           What the stamp threw for this number.
           */
          const refusal = caught(function stampOf(): string {
            return lifecycleStamp({ attempt, wallClock: fixedWallClock, },);
          },);
          expect(refusal,).toBeInstanceOf(LifecycleAttemptError,);
          expect(String(refusal,),).toContain(`${String(attempt,)} is not one`,);
        }
      },
    },),
  ],
},);

await describe({
  name: '',
  children: [
    clockStampSuite,
    stampSuite,
  ],
},);
