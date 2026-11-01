/**
 Tests the wall-clock stub (ledger B78, M100): a case that sets the clock
 reads its own reading through its awaits, and a case the suite runs beside it
 reads the real clock. The first form of the stub replaced `Date.now` for the
 whole process, and a pacer in a sibling case read the stepped clock and
 slept on it.

 THE REAL CLOCK IS READ WITH `new Date()`, which takes the current time from
 the engine rather than through the `Date.now` property (ECMA-262, the `Date`
 constructor called with no arguments), so its stamps bracket a reading of
 `Date.now` whatever the stub does. This file and the stub are the two that
 `wall-clock-reads.unit.test.ts` lets name the wall clock.

 @module
 */

import { setTimeout as sleepFor, } from 'node:timers/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  HOUR_MS,
  stubWallClock,
  WALL_START_MS,
} from './wall-clock-stub.test-fixture.ts';
import { HANG_STOP_MS, } from './hang-stop.test-fixture.ts';

/**
 The reading the stepping case holds while its sibling reads.
 */
const STEPPED_MS = WALL_START_MS - HOUR_MS;

/**
 Settles once the stepping case has set its clock, so its sibling reads while
 the stub is live.
 */
const clockSet = Promise.withResolvers<void>();

/**
 Settles once the sibling has read, so the stepping case holds its stub until
 then.
 */
const siblingRead = Promise.withResolvers<void>();

await describe({
  name: '',
  children: [
    describe({
      name: stubWallClock.name,
      children: [
        it({
          name: 'ANSWERS THE CASE THE READING IT SETS, before and after a step and across an await of a timer',
          fn: async ctx => {
            const wall = stubWallClock({ sinon: ctx.sinon, atMs: WALL_START_MS, },);
            expect(Date.now(),).toBe(WALL_START_MS,);
            wall.step({ byMs: -HOUR_MS, },);
            await sleepFor(1,);
            expect(Date.now(),).toBe(STEPPED_MS,);
          },
        },),
      ],
    },),

    describe({
      name: 'a case beside one that set the clock (ledger M100)',
      children: [
        it({
          name: 'HOLDS THE STEPPED READING while a sibling case reads the clock',
          // How long either case of the pair waits for the other. They meet
          // within a turn of the event loop when the suite runs them side by
          // side; the bound turns a suite that ran them one after the other
          // into a failure rather than a hang.
          timeout: HANG_STOP_MS,
          fn: async ctx => {
            const wall = stubWallClock({ sinon: ctx.sinon, atMs: WALL_START_MS, },);
            wall.step({ byMs: -HOUR_MS, },);
            clockSet.resolve();
            await siblingRead.promise;
            expect(Date.now(),).toBe(STEPPED_MS,);
          },
        },),
        it({
          name: 'READS THE REAL CLOCK while a sibling case holds a stepped one, where a stub replacing Date.now for '
            + 'the whole process answered every case its reading',
          timeout: HANG_STOP_MS,
          fn: async () => {
            await clockSet.promise;
            /**
             Real clock just before the read, as a stamp.
             */
            const before = new Date().toISOString();
            /**
             What this case reads while its sibling's stub is live, as a stamp.
             */
            const read = new Date(Date.now(),).toISOString();
            /**
             Real clock just after the read, as a stamp.
             */
            const after = new Date().toISOString();
            siblingRead.resolve();
            // STAMPS OF ONE LENGTH SORT AS TIME, so the read keeps its place
            // between the two real readings only when it came from the real clock.
            expect([before, read, after,].toSorted(),).toEqual([before, read, after,],);
          },
        },),
      ],
    },),
  ],
},);
