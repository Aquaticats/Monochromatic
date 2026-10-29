/**
 Tests for the corpus pass's slice overlap: read once by the pass's entry file
 from the environment it hands in, and logged on each entry's own line.

 EVERY CASE HANDS ITS OWN ENVIRONMENT and none writes the process's; the
 entry line is read through a divert of `console.log` that each case's own
 sandbox holds.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import type { OverlapSetting, } from '@monochromatic-dev/translation-repair-slice-cache';
import {
  entryOverlap,
  readPassOverlap,
} from '../../dist/final/node/index.mjs';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { PASS_FALLBACK_OVERLAP, } from '@monochromatic-dev/translation-repair-slice-cache/ts/corpus-run/slice-overlap-dial.test-fixture.ts';

/**
 Instant the injected clock reads, so no case waits on the real one.
 */
const PRINTED_AT = new Date('2026-10-22T08:10:11.123Z',);

/**
 Clock that reads the same instant every time it is asked.

 @returns Instant the line is printed at

 @example
 ```ts
 const overlap = entryOverlap({ entryId: 'Tabby', setting, attempt: 2, wallClock: fixedWallClock, },);
 ```
 */
function fixedWallClock(): Date {
  return PRINTED_AT;
}

await describe({
  name: '',
  children: [
    describe({
      name: readPassOverlap.name,
      children: [
        it({
          name: 'KEEPS FOUR SLICES IN FLIGHT from the pass\'s own fallback when the environment sets no overlap, '
            + 'the decision of 2026-09-06',
          fn: async () => {
            expect(readPassOverlap({ env: {}, },),).toStrictEqual(PASS_FALLBACK_OVERLAP,);
          },
        },),

        it({
          name: 'READS the overlap the environment it is handed sets, and names the variable as its source',
          fn: async () => {
            expect(readPassOverlap({ env: { TRANSLATION_REPAIR_SLICE_OVERLAP: '2', }, },),).toStrictEqual({
              overlap: 2,
              source: 'TRANSLATION_REPAIR_SLICE_OVERLAP',
            },);
          },
        },),

        it({
          name: 'REFUSES an overlap nothing can read as a stated refusal, before any entry is settled',
          fn: async () => {
            /**
             What the read threw on a word where a count belongs.
             */
            const refusal = caught(function readWord(): OverlapSetting {
              return readPassOverlap({ env: { TRANSLATION_REPAIR_SLICE_OVERLAP: 'several', }, },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(
              'StatedRefusalError: TRANSLATION_REPAIR_SLICE_OVERLAP must be a whole number written in digits, '
                + `at most ${String(Number.MAX_SAFE_INTEGER,)}, and several is not one`,
            );
          },
        },),
      ],
    },),

    describe({
      name: entryOverlap.name,
      children: [
        it({
          name: 'LOGS the entry\'s OVERLAP line with the clock time, the attempt, the value and its source, and returns the '
            + 'value',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            expect(entryOverlap({
              entryId: 'Tabby',
              setting: {
                overlap: 2,
                source: 'TRANSLATION_REPAIR_SLICE_OVERLAP',
              },
              attempt: 2,
              wallClock: fixedWallClock,
            },),).toBe(2,);
            expect(printed.lines,).toEqual([
              'OVERLAP Tabby at=2026-10-22T08:10:11.123Z attempt=2 value=2 source=TRANSLATION_REPAIR_SLICE_OVERLAP',
            ],);
          },
        },),
      ],
    },),
  ],
},);
