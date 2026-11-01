/**
 Tests for the size bands the slice cost report groups priced slices into,
 and for the line it prints for each band.

 What is printed is read off `console.log` through a diverting capture, which
 is process-wide, so the printing suite runs one case at a time.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  bucketSliceCostsBySize,
  printSliceCostBucket,
} from '../../dist/final/node/index.mjs';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { costRow, } from '@monochromatic-dev/translation-repair-translate-floor/ts/corpus-run/slice-cost-rows.test-fixture.ts';

/**
 Every band's upper bound, in order, the last open-ended.
 */
const BOUNDS = [
  50,
  200,
  500,
  1_000,
  2_000,
  Number.POSITIVE_INFINITY,
];

await describe({
  name: 'slice-cost bands',
  concurrency: 1,
  children: [
    describe({
      name: bucketSliceCostsBySize.name,
      children: [
        it({
          name: 'MAKES SIX EMPTY BANDS, smallest first and the last open-ended, for a log with no row',
          fn: async () => {
            expect(bucketSliceCostsBySize({ rows: [], },),).toEqual(BOUNDS.map(function emptyBand(upTo,) {
              return {
                upTo,
                slices: 0,
                chars: 0,
                ms: 0,
              };
            },),);
          },
        },),
        it({
          name: 'PUTS A SLICE IN THE BAND ABOVE ITS SIZE, so a size equal to a bound opens the next band, and sums '
            + 'the characters and the time of several in one band',
          fn: async () => {
            expect(bucketSliceCostsBySize({
              rows: [
                costRow({ sliceIndex: 0, sourceChars: 49, elapsedMs: 10, },),
                costRow({ sliceIndex: 1, sourceChars: 1, elapsedMs: 20, },),
                costRow({ sliceIndex: 2, sourceChars: 50, elapsedMs: 30, },),
                costRow({ sliceIndex: 3, sourceChars: 2_000, elapsedMs: 40, },),
              ],
            },).map(function summary({ upTo, slices, chars, ms, },) {
              return [
                upTo,
                slices,
                chars,
                ms,
              ];
            },),).toEqual([
              [50, 2, 50, 30,],
              [200, 1, 50, 30,],
              [500, 0, 0, 0,],
              [1_000, 0, 0, 0,],
              [2_000, 0, 0, 0,],
              [Number.POSITIVE_INFINITY, 1, 2_000, 40,],
            ],);
          },
        },),
        it({
          name: 'LEAVES OUT EVERY SLICE NOT COMPUTED, since a cached or skipped slice prices no work',
          fn: async () => {
            expect(bucketSliceCostsBySize({
              rows: [
                costRow({ sliceIndex: 0, sourceChars: 10, elapsedMs: 1, exit: 'resumed', },),
                costRow({ sliceIndex: 1, sourceChars: 10, elapsedMs: 1, exit: 'failed', },),
              ],
            },).map(function slicesOf({ slices, },): number {
              return slices;
            },),).toEqual([0, 0, 0, 0, 0, 0,],);
          },
        },),
      ],
    },),
    describe({
      name: printSliceCostBucket.name,
      children: [
        it({
          name: 'PRINTS NOTHING for a band no slice landed in',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            printSliceCostBucket({ bucket: { upTo: 200, slices: 0, chars: 0, ms: 0, }, },);

            expect(printed.lines,).toEqual([],);
          },
        },),
        it({
          name: 'PRINTS A BAND BY ITS BOUND with the minutes a slice takes and the milliseconds a character takes',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            printSliceCostBucket({ bucket: { upTo: 200, slices: 12, chars: 1_400, ms: 90_000, }, },);

            expect(printed.lines,).toEqual([
              '  under    200 chars  slices   12  min/slice   0.13  ms/char    64.3',
            ],);
          },
        },),
        it({
          name: 'PRINTS THE OPEN-ENDED BAND AS ANY',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            printSliceCostBucket({
              bucket: { upTo: Number.POSITIVE_INFINITY, slices: 1, chars: 3_000, ms: 360_000, },
            },);

            expect(printed.lines,).toEqual([
              '  under    any chars  slices    1  min/slice   6.00  ms/char   120.0',
            ],);
          },
        },),
        it({
          name: 'PRINTS ZERO MILLISECONDS A CHARACTER for slices that carried no character, rather than dividing by zero',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            printSliceCostBucket({ bucket: { upTo: 50, slices: 2, chars: 0, ms: 60_000, }, },);

            expect(printed.lines,).toEqual([
              '  under     50 chars  slices    2  min/slice   0.50  ms/char     0.0',
            ],);
          },
        },),
      ],
    },),
  ],
},);
