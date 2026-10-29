/**
 Tests that a fragment landing outside its slice, or with an end before its start, is refused by name (n160 finding Y7).

 `fragmentProseMask` once returned a mask of length zero for a start past the slice, and `refused: true` with a mask of
 zeros for an end before the start, which `restoreTypography` then reported as "refused by the page grammar" about a slice
 that parses. A landing is built by `apply-patch.ts` only after the envelope's current text is compared with its base
 text, so no run reaches the throw; it is the invariant that keeps a wrong landing from reading as no prose.

 Cat-themed invention throughout.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { fragmentProseMask, } from '@monochromatic-dev/translation-repair-typography';
import { restoreTypography, } from '../dist/final/node/index.mjs';
import { warnLinesDuring, } from '@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts';

/**
 A slice that parses, with a bold stretch the landings of these tests point into.
 */
const SLICE = 'He said <b>It\'s here</b>, and the cat’s bowl was full.';

/**
 Landings that lie outside the slice or end before they start.
 */
const BAD_LANDINGS: readonly (readonly [name: string, start: number, end: number,])[] = [
  ['a start past the slice', SLICE.length + 5, SLICE.length + 6,],
  ['an end past the slice', 8, SLICE.length + 1,],
  ['an end before the start', 12, 8,],
  ['a negative start', -1, 4,],
  // n166, Z8: offsets that are no whole numbers fail every comparison of the range check, so each needs its own refusal.
  ['a start that is not a number', Number.NaN, 4,],
  ['an end that is not a number', 4, Number.NaN,],
  ['a fraction for each offset', 1.5, 3.5,],
  ['a fractional end', 4, 8.5,],
];

/**
 What a call threw, absent when it returned.

 @param run - call
 @returns Thrown value as a list of zero or one

 @example
 ```ts
 const [thrown,] = thrownBy({ run: function nothing(): void {}, },);
 ```
 */
function thrownBy({ run, }: { readonly run: () => unknown; },): readonly unknown[] {
  try {
    run();
    return [];
  }
  catch (error) {
    return [error,];
  }
}

await describe({
  name: 'a fragment landing',
  concurrency: 1,
  children: [
    ...BAD_LANDINGS.map(function badLanding([name, start, end,],) {
      return describe({
        name: `with ${name}`,
        children: [
          it({
            name: 'IS REFUSED by fragmentProseMask with a FragmentLandingError that names the offsets',
            fn: async () => {
              const [thrown,] = thrownBy({
                run: function read(): unknown {
                  return fragmentProseMask({ slice: SLICE, start, end, replacement: '<b>It\'s here', },);
                },
              },);
              expect(Error.isError(thrown,),).toBe(true,);
              expect((thrown as Error).name,).toBe('FragmentLandingError',);
              expect((thrown as Error).message,).toContain(String(start,),);
              expect((thrown as Error).message,).toContain(String(end,),);
              expect((thrown as Error).message,).toContain(String(SLICE.length,),);
            },
          },),

          it({
            name: 'IS REFUSED by restoreTypography with the same error and never reported as a refusal of the page grammar',
            fn: async () => {
              /**
               What the call threw and warned.
               */
              const { warned, } = await warnLinesDuring({
                run: async () => {
                  const [thrown,] = thrownBy({
                    run: function restore(): unknown {
                      return restoreTypography({
                        replacement: '<b>It\'s here',
                        replaced: '<b>It’s here',
                        convention: SLICE,
                        landing: { slice: SLICE, start, end, },
                      },);
                    },
                  },);
                  expect(Error.isError(thrown,),).toBe(true,);
                  expect((thrown as Error).name,).toBe('FragmentLandingError',);
                },
              },);
              expect(
                warned.filter(function namesGrammar(line,): boolean {
                  return line.includes('page grammar',);
                },),
              ).toEqual([],);
            },
          },),
        ],
      },);
    },),

    describe({
      name: 'at the edges of the slice',
      children: [
        it({
          name: 'READS a landing over the whole slice and an insertion at either end, one flag per unit of the fragment',
          fn: async () => {
            for (const [start, end, replacement,] of [
              [0, SLICE.length, 'The cat’s bowl.'],
              [0, 0, 'Oh, '],
              [SLICE.length, SLICE.length, ' Yes.'],
            ] as const) {
              const reading = fragmentProseMask({ slice: SLICE, start, end, replacement, },);
              expect(reading.mask.length,).toBe(replacement.length,);
              expect(reading.visible.length,).toBe(replacement.length,);
              expect(reading.refused,).toBe(false,);
            }
          },
        },),
      ],
    },),
  ],
},);
