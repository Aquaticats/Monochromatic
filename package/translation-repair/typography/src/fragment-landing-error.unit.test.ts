/**
 Tests for the error a fragment landing outside its slice throws, and the check that throws it (`fragment-landing-error.ts`).

 Fixtures are cat-themed invention.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  assertLandingInside,
  FragmentLandingError,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: FragmentLandingError.name,
      concurrency: 1,
      children: [
        it({
          name: 'IS an Error named for its class',
          fn: async () => {
            /**
             The error for a landing that ends past its slice.
             */
            const error = new FragmentLandingError({
              slice: 24,
              start: 30,
              end: 31,
            },);
            expect(Error.isError(error,),).toBe(true,);
            expect(error.name,).toBe('FragmentLandingError',);
          },
        },),

        it({
          name: 'NAMES the offsets and the length of the slice in its message, and no text',
          fn: async () => {
            /**
             The error for a landing that ends before it starts.
             */
            const { message, } = new FragmentLandingError({
              slice: 24,
              start: 9,
              end: 3,
            },);
            expect(message,).toContain('lands at 9 to 3 in a slice of 24 units',);
          },
        },),

        it({
          name: 'COUNTS A SLICE OF ONE UNIT IN THE SINGULAR, since the noun agrees with the count',
          fn: async () => {
            /**
             The error for a landing that ends past a slice of one unit.
             */
            const { message, } = new FragmentLandingError({
              slice: 1,
              start: 0,
              end: 2,
            },);
            expect(message,).toContain('lands at 0 to 2 in a slice of 1 unit, and a landing starts',);
          },
        },),

        it({
          name: 'SAYS THAT A LANDING STARTS AND ENDS AT WHOLE NUMBERS, beside the rest of what a landing must be, so a reader of the '
            + 'message knows a fractional offset is one of the faults',
          fn: async () => {
            const { message, } = new FragmentLandingError({
              slice: 24,
              start: 2.5,
              end: 7,
            },);
            expect(message,).toContain(
              'and a landing starts and ends at whole numbers, starts at 0 or later, ends at or after its start, and ends within the slice',
            );
          },
        },),
      ],
    },),

    describe({
      name: assertLandingInside.name,
      concurrency: 1,
      children: [
        it({
          name: 'ACCEPTS a landing inside the slice, empty or whole',
          fn: async () => {
            assertLandingInside({
              slice: 24,
              start: 8,
              end: 20,
            },);
            assertLandingInside({
              slice: 24,
              start: 0,
              end: 24,
            },);
            assertLandingInside({
              slice: 24,
              start: 5,
              end: 5,
            },);
          },
        },),

        it({
          name: 'THROWS for a landing whose start or end is not a whole number, which no comparison of the range check refuses (n166, Z8)',
          fn: async () => {
            for (const [start, end,] of [[Number.NaN, 3,], [1, Number.NaN,], [1.5, 3.5,], [1, 3.5,], [0.5, 3,],] as const) {
              expect(caught(function landBetweenUnits(): void {
                assertLandingInside({
                  slice: 24,
                  start,
                  end,
                },);
              },) instanceof FragmentLandingError,).toBe(true,);
            }
          },
        },),

        it({
          name: 'THROWS for a landing that starts before the slice, ends past it, or ends before it starts',
          fn: async () => {
            for (const [start, end,] of [[-1, 3,], [20, 25,], [9, 3,],] as const) {
              expect(caught(function landOutside(): void {
                assertLandingInside({
                  slice: 24,
                  start,
                  end,
                },);
              },) instanceof FragmentLandingError,).toBe(true,);
            }
          },
        },),
      ],
    },),
  ],
},);
