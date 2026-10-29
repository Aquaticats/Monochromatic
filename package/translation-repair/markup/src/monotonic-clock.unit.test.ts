/**
 Tests the clock every duration, hold, pace and budget in the package is read
 on (ledger B78): whole milliseconds that never go back, which the system
 clock being set leaves alone.

 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { monotonicMs, } from '../dist/final/node/index.mjs';

import {
  HOUR_MS,
  stubWallClock,
  WALL_START_MS,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/wall-clock-stub.test-fixture.ts';

await describe({
  name: monotonicMs.name,
  children: [
    it({
      name: 'READS WHOLE MILLISECONDS THAT NEVER GO BACK, and moves on across a real wait',
      fn: async () => {
        /**
         Two readings with nothing between them.
         */
        const first = monotonicMs();
        const second = monotonicMs();
        await wait(5,);

        /**
         A reading after a real wait.
         */
        const third = monotonicMs();
        expect([
          first,
          second,
          third,
        ].every(function whole(reading,): boolean {
          return Number.isInteger(reading,);
        },),).toBe(true,);
        expect(second,).toBeGreaterThanOrEqual(first,);
        expect(third,).toBeGreaterThan(second,);
      },
    },),
    it({
      name: 'READS ON WHEN THE SYSTEM CLOCK IS SET BACK AN HOUR, where a reading of the wall clock would go back '
        + 'the hour (ledger B78)',
      fn: async ctx => {
        const wall = stubWallClock({ sinon: ctx.sinon, atMs: WALL_START_MS, },);

        /**
         A reading before the clock is set back.
         */
        const before = monotonicMs();
        wall.step({ byMs: -HOUR_MS, },);
        expect(monotonicMs(),).toBeGreaterThanOrEqual(before,);
      },
    },),
  ],
},);
