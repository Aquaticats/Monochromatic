/**
 Tests for the lines `score-agreement` prints: rates to three places, lists
 that name an empty list, and the precision and agreement lines themselves.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  agreementRate,
  positionsOrNone,
  printAgreement,
  printPrecision,
} from '../../dist/final/node/index.mjs';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';

await describe({
  name: 'score-agreement-print',
  concurrency: 1,
  children: [
    describe({
      name: agreementRate.name,
      concurrency: 1,
      children: [
        it({
          name: 'RENDERS a rate to three places',
          fn: async () => {
            expect(agreementRate({ numerator: 37, denominator: 47, },),).toBe('0.787',);
          },
        },),

        it({
          name: 'NAMES a rate over no items n/a rather than dividing by zero',
          fn: async () => {
            expect(agreementRate({ numerator: 0, denominator: 0, },),).toBe('n/a',);
          },
        },),
      ],
    },),

    describe({
      name: positionsOrNone.name,
      concurrency: 1,
      children: [
        it({
          name: 'JOINS positions by commas, one position alone and several',
          fn: async () => {
            expect(positionsOrNone({ positions: [4,], },),).toBe('4',);
            expect(positionsOrNone({ positions: [3, 4,], },),).toBe('3,4',);
          },
        },),

        it({
          name: 'NAMES no position none',
          fn: async () => {
            expect(positionsOrNone({ positions: [], },),).toBe('none',);
          },
        },),
      ],
    },),

    describe({
      name: printPrecision.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS the three rates and the positions that were declined or repeated',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printPrecision({
              items: 5,
              precision: {
                scored: 3,
                realDefects: 2,
                unscored: [4,],
                duplicates: [3, 5,],
                gradeable: 3,
              },
            },);

            expect(printed.lines,).toStrictEqual([
              'PRECISION items=5 gradeable=3 scored=3 realDefects=2 strict=0.667 excluded=0.667 lenient=1.000 '
                + 'duplicates=3,5 unscored=4',
            ],);
          },
        },),

        it({
          name: 'NAMES every rate n/a and both lists none for a sheet with no item',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printPrecision({
              items: 0,
              precision: {
                scored: 0,
                realDefects: 0,
                unscored: [],
                duplicates: [],
                gradeable: 0,
              },
            },);

            expect(printed.lines,).toStrictEqual([
              'PRECISION items=0 gradeable=0 scored=0 realDefects=0 strict=n/a excluded=n/a lenient=n/a '
                + 'duplicates=none unscored=none',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: printAgreement.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS the rate and the positions where the pre-grade differed',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printAgreement({
              agreement: {
                compared: 3,
                agreed: 2,
                disagreed: [2,],
                unscored: [],
              },
            },);

            expect(printed.lines,).toStrictEqual(['AGREEMENT compared=3 agreed=2 rate=0.667 disagreed=2',],);
          },
        },),

        it({
          name: 'NAMES the rate n/a and the disagreements none where nothing was compared',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printAgreement({
              agreement: {
                compared: 0,
                agreed: 0,
                disagreed: [],
                unscored: [],
              },
            },);

            expect(printed.lines,).toStrictEqual(['AGREEMENT compared=0 agreed=0 rate=n/a disagreed=none',],);
          },
        },),
      ],
    },),
  ],
},);
