/**
 Tests for the clock time and attempt a lifecycle line carries (ledger B960): the instant in the spelling
 `Date.prototype.toISOString` writes, which `isIsoStampText` reads back, and the attempt as a whole number from 1.

 The clock is injected, so no case waits on the real one.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  isIsoStampText,
  LifecycleAttemptError,
  lifecycleStamp,
  wallClockNow,
} from '../dist/final/node/index.mjs';

/**
 Instant the injected clock reads.
 */
const PRINTED_AT = new Date('2026-10-22T08:10:11.123Z',);

/**
 Clock that reads the same instant every time it is asked.

 @returns The instant the line is printed at

 @example
 ```ts
 const stamp = lifecycleStamp({ attempt: 2, wallClock: fixedWallClock, },);
 ```
 */
function fixedWallClock(): Date {
  return PRINTED_AT;
}

await describe({
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
      name: 'READS THE WALL CLOCK when it is given no clock of its own',
      fn: async () => {
        /**
         Time the stamp names, cut off its key.
         */
        const time = lifecycleStamp({ attempt: 1, },)
          .split(' ',)[0]
          ?.slice('at='.length,) ?? '';
        expect(isIsoStampText({ text: time, },),).toBe(true,);

        /**
         The wall clock's own stamp, read right after.
         */
        const afterwards = wallClockNow()
          .toISOString();
        expect(Math.abs(Date.parse(time,) - Date.parse(afterwards,),),).toBeLessThan(60_000,);
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
