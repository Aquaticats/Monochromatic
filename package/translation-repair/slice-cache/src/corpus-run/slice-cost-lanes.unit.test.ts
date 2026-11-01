/**
 Tests for what the slice cost report prints of each lane's spending.

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

import { printSliceCostLanes, } from '../../dist/final/node/index.mjs';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { costRow, } from '@monochromatic-dev/translation-repair-translate-floor/ts/corpus-run/slice-cost-rows.test-fixture.ts';

await describe({
  name: printSliceCostLanes.name,
  concurrency: 1,
  children: [
    it({
      name: 'PRINTS EACH LANE IN THE ORDER A PASS RUNS THEM, its priced slices and its minutes, whatever order the rows come in',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        printSliceCostLanes({
          rows: [
            costRow({ sliceIndex: 0, sourceChars: 10, elapsedMs: 30_000, lane: 'consolidation', },),
            costRow({ sliceIndex: 1, sourceChars: 10, elapsedMs: 120_000, lane: 'translate', },),
            costRow({ sliceIndex: 2, sourceChars: 10, elapsedMs: 360_000, lane: 'translate', },),
            costRow({ sliceIndex: 3, sourceChars: 10, elapsedMs: 60_000, lane: 'repair', },),
          ],
        },);

        expect(printed.lines,).toEqual([
          '\nBY LANE',
          '  repair        slices    1  total      1.0 min',
          '  translate     slices    2  total      8.0 min',
          '  consolidation slices    1  total      0.5 min',
        ],);
      },
    },),
    it({
      name: 'LEAVES OUT A LANE WITH NO PRICED SLICE, one whose slices were all resumed included',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        printSliceCostLanes({
          rows: [
            costRow({ sliceIndex: 0, sourceChars: 10, elapsedMs: 60_000, lane: 'repair', exit: 'resumed', },),
            costRow({ sliceIndex: 1, sourceChars: 10, elapsedMs: 60_000, lane: 'translate', },),
          ],
        },);

        expect(printed.lines,).toEqual([
          '\nBY LANE',
          '  translate     slices    1  total      1.0 min',
        ],);
      },
    },),
    it({
      name: 'PRINTS ONLY THE HEADING where no row is priced',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        printSliceCostLanes({ rows: [], },);

        expect(printed.lines,).toEqual(['\nBY LANE',],);
      },
    },),
  ],
},);
