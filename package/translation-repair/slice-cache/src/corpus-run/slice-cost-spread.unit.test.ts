/**
 Tests for the spread the slice cost report prints beside its bands: the
 cheapest and the dearest priced slice, and how many times more the dearest
 cost.

 What is printed is read off `console.log` through a diverting capture, which
 is process-wide, so the suite runs one case at a time.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { printSliceCostSpread, } from '../../dist/final/node/index.mjs';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { costRow, } from '@monochromatic-dev/translation-repair-translate-floor/ts/corpus-run/slice-cost-rows.test-fixture.ts';
import { SPREAD_NOTE, } from './slice-cost-report.test-fixture.ts';

/**
 Heading the spread opens with.
 */
const HEADING = '\nSPREAD, WHICH THE BANDS AVERAGE AWAY';

await describe({
  name: printSliceCostSpread.name,
  concurrency: 1,
  children: [
    it({
      name: 'PRINTS THE CHEAPEST AND THE DEAREST PRICED SLICE with their sizes, minutes and lanes, and the ratio '
        + 'between them',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        printSliceCostSpread({
          rows: [
            costRow({ sliceIndex: 0, sourceChars: 150, elapsedMs: 120_000, lane: 'repair', },),
            costRow({ sliceIndex: 1, sourceChars: 3_000, elapsedMs: 360_000, lane: 'translate', },),
            costRow({ sliceIndex: 2, sourceChars: 40, elapsedMs: 30_000, lane: 'consolidation', },),
          ],
        },);

        expect(printed.lines,).toEqual([
          HEADING,
          '  cheapest    40 chars    0.50 min  consolidation',
          '  dearest   3000 chars    6.00 min  translate',
          '  ratio     12.0x',
          SPREAD_NOTE,
        ],);
      },
    },),
    it({
      name: 'PRINTS "char" AFTER A SIZE OF ONE and "chars" after any other',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        printSliceCostSpread({
          rows: [
            costRow({ sliceIndex: 0, sourceChars: 1, elapsedMs: 60_000, },),
            costRow({ sliceIndex: 1, sourceChars: 2, elapsedMs: 120_000, },),
          ],
        },);

        expect(printed.lines.slice(1, 3,),).toEqual([
          '  cheapest     1 char    1.00 min  translate',
          '  dearest      2 chars    2.00 min  translate',
        ],);
      },
    },),
    it({
      name: 'COMPARES ONLY PRICED SLICES, so a resumed slice that took no time is not the cheapest',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        printSliceCostSpread({
          rows: [
            costRow({ sliceIndex: 0, sourceChars: 10, elapsedMs: 0, exit: 'resumed', },),
            costRow({ sliceIndex: 1, sourceChars: 20, elapsedMs: 60_000, },),
            costRow({ sliceIndex: 2, sourceChars: 30, elapsedMs: 120_000, },),
          ],
        },);

        expect(printed.lines.slice(1, 4,),).toEqual([
          '  cheapest    20 chars    1.00 min  translate',
          '  dearest     30 chars    2.00 min  translate',
          '  ratio     2.0x',
        ],);
      },
    },),
    it({
      name: 'SAYS THERE IS NO SPREAD where fewer than two slices are priced, and how many are',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        printSliceCostSpread({ rows: [costRow({ sliceIndex: 0, sourceChars: 10, elapsedMs: 60_000, },),], },);
        printSliceCostSpread({ rows: [], },);

        expect(printed.lines,).toEqual([
          HEADING,
          '  NO SPREAD TO SHOW: it takes two priced slices to compare, and this log holds 1 priced slice.',
          HEADING,
          '  NO SPREAD TO SHOW: it takes two priced slices to compare, and this log holds 0 priced slices.',
        ],);
      },
    },),
    it({
      name: 'SAYS THE RATIO IS NOT DEFINED where the cheapest slice took no time, rather than printing nothing',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        printSliceCostSpread({
          rows: [
            costRow({ sliceIndex: 0, sourceChars: 10, elapsedMs: 0, },),
            costRow({ sliceIndex: 1, sourceChars: 20, elapsedMs: 60_000, },),
          ],
        },);

        expect(printed.lines,).toEqual([
          HEADING,
          '  cheapest    10 chars    0.00 min  translate',
          '  dearest     20 chars    1.00 min  translate',
          '  ratio     not defined: the cheapest slice took no measurable time',
        ],);
      },
    },),
  ],
},);
