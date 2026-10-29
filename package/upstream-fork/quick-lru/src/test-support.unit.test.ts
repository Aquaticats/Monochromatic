/**
 Tests for the test-only fake clock helpers.
 
 Every suite here installs the global `Date.now` replacement, so the root
 suite runs sequentially (`concurrency: 1`) and each test restores the real
 clock before the next one starts.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  installFakeClock,
} from './test-support.ts';

await describe({
  name: 'test support',
  concurrency: 1,
  children: [
    describe({
      name: installFakeClock.name,
      children: [
        it({
          name: 'reads fake time starting from the requested instant',
          fn: async () => {
            using clock = installFakeClock({
              startMilliseconds: 1_700_000_000_000,
            },);
            expect(clock.now,).toBe(1_700_000_000_000,);
            expect(Date.now(),).toBe(1_700_000_000_000,);
          },
        },),

        it({
          name: 'advances fake time by the requested amount',
          fn: async () => {
            using clock = installFakeClock({
              startMilliseconds: 1_000,
            },);
            clock.advance(250,);
            expect(clock.now,).toBe(1_250,);
            expect(Date.now(),).toBe(1_250,);
            clock.advance(-50,);
            expect(clock.now,).toBe(1_200,);
          },
        },),

        it({
          name: 'hands the real Date.now back at disposal',
          fn: async () => {
            /**
             Real `Date.now` captured before the fake clock took over.
             */
            const realDateNow = Date.now;
            {
              using _clock = installFakeClock({
                startMilliseconds: 5,
              },);
              expect(Date.now(),).toBe(5,);
            }
            expect(Date.now,).toBe(realDateNow,);
          },
        },),

        it({
          name: 'hands the real Date.now back on restore',
          fn: async () => {
            /**
             Real `Date.now` captured before the fake clock took over.
             */
            const realDateNow = Date.now;
            /**
             Clock handle restored explicitly below.
             */
            const clock = installFakeClock({
              startMilliseconds: 9,
            },);
            expect(Date.now(),).toBe(9,);
            clock.restore();
            expect(Date.now,).toBe(realDateNow,);
          },
        },),
      ],
    },),
  ],
},);
