/**
 Tests for the percentile reader the corpus censuses share.

 THE RANK CONVENTION IS THE WHOLE SUBJECT. `percentileOf` takes
 `floor(p / 100 * n)` and clamps it to the last index, which is one of several
 defensible definitions and the only one the census lines mean. A later change
 to the more common `ceil(p / 100 * n) - 1` would move every published p50 by
 one rank on even-length samples without moving any test, so the cases in this file
 pin the rank itself rather than only the shape of the line.

 THE CLAMP IS NOT DECORATION EITHER. Without it, p99 on a ten-value sample
 reads index 9 and p100 reads index 10, which is past the end; the function
 would return its `?? 0` fallback and report a far tail of zero for a sample
 whose maximum is 200. That is the failure a distribution reader must not have,
 because zero is a plausible-looking number.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  describeSpread,
  percentileOf,
  REPORTED_PERCENTILES,
} from '../../dist/final/node/index.mjs';

//region Census spread tests

/**
 How long each cat of one household slept, in minutes, out of order.

 Deliberately unsorted, and deliberately long-tailed: the largest value is
 more than three times the second largest, so a reader that lost the clamp or
 the sort reports a visibly different tail rather than a near miss.
 */
const NAP_MINUTES: readonly number[] = [
  12,
  3,
  47,
  8,
  200,
  5,
  19,
  61,
  2,
  33,
];

/**
 The same naps in ascending order, which is what `percentileOf` is handed.
 */
const SORTED_NAPS: readonly number[] = [
  2,
  3,
  5,
  8,
  12,
  19,
  33,
  47,
  61,
  200,
];

/**
 What the household's spread comes to, as the census prints it.
 */
const NAP_LINE = 'nap minutes: n 10, p50 19, p90 200, p99 200, max 200';

/**
 What a household with no cats in it comes to.
 */
const EMPTY_LINE = 'nap minutes: n 0, p50 0, p90 0, p99 0, max 0';

/**
 Percentile naming the middle of a sample.
 */
const HALF = 50;

/**
 Percentile naming where a tail starts being worth reporting.
 */
const NINETIETH = 90;

/**
 Percentile naming the far tail, where a per-call deadline is met.
 */
const NINETY_NINTH = 99;

/**
 Whole of a distribution, which is past the last rank and must clamp.
 */
const WHOLE = 100;

/**
 Rank the middle of this ten-value sample lands on under this convention.
 */
const MIDDLE_NAP = 19;

/**
 Longest nap, which is also both reported tails on a sample this small.
 */
const LONGEST_NAP = 200;

/**
 Shortest nap, which percentile zero reads.
 */
const SHORTEST_NAP = 2;

/**
 Percentiles this package reports for every distribution.
 */
const PERCENTILES_REPORTED = 3;

/**
 Sample of one, where every percentile has to land on the same value.
 */
const LONE_NAP: readonly number[] = [7,];

/**
 How long that one cat slept.
 */
const LONE_NAP_MINUTES = 7;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: percentileOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS the middle at the upper of two ranks, which is this package\'s convention',
          fn: async () => {
            expect(percentileOf({
              sorted: SORTED_NAPS,
              percentile: HALF,
            },),).toBe(MIDDLE_NAP,);
          },
        },),
        it({
          name: 'READS the first value at percentile zero',
          fn: async () => {
            expect(percentileOf({
              sorted: SORTED_NAPS,
              percentile: 0,
            },),).toBe(SHORTEST_NAP,);
          },
        },),
        it({
          name: 'CLAMPS a far tail to the last rank instead of reading past the end',
          fn: async () => {
            // Both of these compute a rank at or past the end before the clamp:
            // 0.99 * 10 floors to 9, which is the last index, and 1.00 * 10 is 10,
            // which is not an index at all. Held together because the second is the
            // one that would silently return zero.
            expect(percentileOf({
              sorted: SORTED_NAPS,
              percentile: NINETY_NINTH,
            },),).toBe(LONGEST_NAP,);
            expect(percentileOf({
              sorted: SORTED_NAPS,
              percentile: WHOLE,
            },),).toBe(LONGEST_NAP,);
          },
        },),
        it({
          name: 'REFUSES a percentile below zero or not a number, which names no rank, rather than answering zero '
            + 'for a sample that holds none',
          fn: async () => {
            /**
             What each percentile that names no rank raised.
             */
            const refusals = [
              -10,
              Number.NaN,
            ].map(function refusalOf(percentile,): unknown {
              return caught(function act(): unknown {
                return percentileOf({
                  sorted: SORTED_NAPS,
                  percentile,
                },);
              },);
            },);
            expect(refusals.map(String,),).toEqual([
              'RangeError: percentile -10 is not a number from zero up, so it names no rank in the sample',
              'RangeError: percentile NaN is not a number from zero up, so it names no rank in the sample',
            ],);
          },
        },),
        it({
          name: 'ANSWERS zero for an empty sample rather than refusing',
          fn: async () => {
            expect(percentileOf({
              sorted: [],
              percentile: HALF,
            },),).toBe(0,);
          },
        },),
        it({
          name: 'LANDS every reported percentile on the one value of a sample of one',
          fn: async () => {
            for (const percentile of REPORTED_PERCENTILES) {
              expect(percentileOf({
                sorted: LONE_NAP,
                percentile,
              },),).toBe(LONE_NAP_MINUTES,);
            }
          },
        },),
      ],
    },),

    describe({
      name: describeSpread.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SORTS what it is handed, so a caller need not',
          fn: async () => {
            // NAP_MINUTES is out of order and SORTED_NAPS is the same sample in
            // order. Equal lines is the whole claim: the reader owns the sort.
            expect(describeSpread({
              label: 'nap minutes',
              values: NAP_MINUTES,
            },),).toBe(describeSpread({
              label: 'nap minutes',
              values: SORTED_NAPS,
            },),);
          },
        },),
        it({
          name: 'NAMES count, every reported percentile and maximum, in that order',
          fn: async () => {
            expect(describeSpread({
              label: 'nap minutes',
              values: NAP_MINUTES,
            },),).toBe(NAP_LINE,);
          },
        },),
        it({
          name: 'ANSWERS with zeroes for an empty sample, keeping the label',
          fn: async () => {
            expect(describeSpread({
              label: 'nap minutes',
              values: [],
            },),).toBe(EMPTY_LINE,);
          },
        },),
        it({
          name: 'REPORTS exactly the percentiles the shared list names',
          fn: async () => {
            /**
             Percentile readings the line carries, one per `p` marker.
             */
            const readings = NAP_LINE
              .split(', ',)
              .filter(function isReading(part,): boolean {
                return part.startsWith('p',);
              },);

            expect(REPORTED_PERCENTILES.length,).toBe(PERCENTILES_REPORTED,);
            expect(readings.length,).toBe(PERCENTILES_REPORTED,);
            expect(REPORTED_PERCENTILES,).toEqual([
              HALF,
              NINETIETH,
              NINETY_NINTH,
            ],);
          },
        },),
      ],
    },),
  ],
},);

//endregion Census spread tests
