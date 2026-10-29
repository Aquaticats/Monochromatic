/**
 Tests for the check of a pinned start of the bench: the starts that name a seat, the starts that name none, and the error that
 names the value and the size of the bench.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  assertBenchStart,
  StageBenchStartError,
} from '../dist/final/node/index.mjs';

/**
 What checking one start of a bench of seven comes to.

 @param start - start checked

 @returns Error thrown, none when the start names a seat
 */
function refusalOf({ start, }: { readonly start: number; },): unknown {
  try {
    assertBenchStart({ start, benchSize: 7, },);
  }
  catch (error) {
    return error;
  }
  return undefined;
}

await describe({
  name: '',
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    describe({
      name: assertBenchStart.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS EVERY WHOLE NUMBER FROM 0 TO THE SIZE LESS ONE',
          fn: async () => {
            expect([0, 1, 3, 6,].map(function refused(start,): unknown {
              return refusalOf({ start, },);
            },),).toEqual([undefined, undefined, undefined, undefined,],);
          },
        },),
        it({
          name: 'REFUSES A NEGATIVE START, NOT A NUMBER, A FRACTION, THE SIZE ITSELF AND AN INFINITY',
          fn: async () => {
            expect([-1, Number.NaN, 1.5, 7, 8, Number.POSITIVE_INFINITY, Number.NEGATIVE_INFINITY,].map(
              function refused(start,): unknown {
                return refusalOf({ start, },) instanceof StageBenchStartError;
              },
            ),).toEqual([true, true, true, true, true, true, true,],);
          },
        },),
      ],
    },),
    describe({
      name: StageBenchStartError.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE VALUE, THE LAST SEAT AND THE SIZE OF THE BENCH, and is marked as a message that names only numbers',
          fn: async () => {
            const error = refusalOf({ start: 1.5, },) as StageBenchStartError;
            expect({
              name: error.name,
              namesValue: error.message.includes('1.5',),
              namesLastSeat: error.message.includes('from 0 to 6',),
              namesSize: error.message.includes('size of the bench is 7',),
              wholeNumber: error.message.includes('whole number',),
              messageNamesOnly: error.messageNamesOnly,
            },).toEqual({
              name: 'StageBenchStartError',
              namesValue: true,
              namesLastSeat: true,
              namesSize: true,
              wholeNumber: true,
              messageNamesOnly: true,
            },);
          },
        },),
      ],
    },),
  ],
},);
