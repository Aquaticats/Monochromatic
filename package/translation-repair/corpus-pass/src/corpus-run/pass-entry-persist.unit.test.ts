/**
 Tests for the lines a pass prints to stdout for one settled entry: its tally, its destination counts and the content
 checks its page shipped failing.

 Every line of an attempt carries the clock time and the attempt it belongs to (ledger B960). The attempt is a later one
 here, so a line that printed a number of its own, or the first attempt's, would read as another attempt's.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { DestinationCheck, } from '@monochromatic-dev/translation-repair-page-check';
import { relayingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { printSettledLines, } from '../../dist/final/node/index.mjs';

/**
 Instant the injected wall clock reads, so no case waits on the real one.
 */
const PRINTED_AT = new Date('2026-10-22T08:10:11.123Z',);

/**
 The clock time and the attempt every line of the cases carries: a fourth attempt, so a line that printed the first
 one's number reads as wrong.
 */
const STAMP = 'at=2026-10-22T08:10:11.123Z attempt=4';

/**
 Wall clock that reads the same instant every time it is asked.

 @returns Instant the lines are printed at

 @example
 ```ts
 printSettledLines({ entryId, tally, published, attempt: 4, wallClock: fixedWallClock, },);
 ```
 */
function fixedWallClock(): Date {
  return PRINTED_AT;
}

/**
 Destinations of a page that dropped none of the source's.
 */
const KEPT_ALL: DestinationCheck = {
  source: ['https://example.org/cats',],
  page: ['https://example.org/cats',],
  dropped: [],
  findings: [],
};

await describe({
  name: printSettledLines.name,
  concurrency: 1,
  children: [
    it({
      name: 'PRINTS THE TALLY AND THE DESTINATIONS LINE for a clean page, the destinations line naming the attempt that '
        + 'published it, and no defects line',
      fn: async (ctx) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        printSettledLines({
          entryId: 'tabby',
          tally: 'TALLY tabby at=2026-10-22T08:10:11.123Z attempt=4 status=ok',
          published: {
            destinations: KEPT_ALL,
            defects: [],
          },
          attempt: 4,
          wallClock: fixedWallClock,
        },);
        expect(printed.lines,).toEqual([
          'TALLY tabby at=2026-10-22T08:10:11.123Z attempt=4 status=ok',
          `DESTINATIONS tabby ${STAMP} source=1 page=1 dropped=0`,
        ],);
      },
    },),
    it({
      name: 'PRINTS THE DEFECTS LINE OF A PAGE THAT SHIPPED WITH DEFECTS at the attempt that published it, a later one '
        + 'than the first, beside the destinations line of the same attempt',
      fn: async (ctx) => {
        using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
        printSettledLines({
          entryId: 'tabby',
          tally: 'TALLY tabby at=2026-10-22T08:10:11.123Z attempt=4 status=ok',
          published: {
            destinations: KEPT_ALL,
            defects: [
              {
                check: 'headings',
                message: 'entry tabby collapses 2 source headings into 1',
              },
              {
                check: 'page-footnotes',
                message: 'entry tabby leaves a footnote unresolved',
              },
            ],
          },
          attempt: 4,
          wallClock: fixedWallClock,
        },);
        expect(printed.lines,).toEqual([
          'TALLY tabby at=2026-10-22T08:10:11.123Z attempt=4 status=ok',
          `DESTINATIONS tabby ${STAMP} source=1 page=1 dropped=0`,
          `DEFECTS tabby ${STAMP} checks=headings,page-footnotes`,
        ],);
      },
    },),
  ],
},);
