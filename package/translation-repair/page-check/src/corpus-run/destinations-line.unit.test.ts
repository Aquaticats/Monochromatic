/**
 Tests for the per-entry destinations line.

 WHAT THESE PIN: the line carries the three counts under stable names a grep
 can total, carries no address, appends findings after the counts, and opens with the clock time it was printed at
 and the attempt it belongs to, so a reader windowing a log by time can place it.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { destinationsLine, } from '../../dist/final/node/index.mjs';

/**
 Instant the injected clock reads, so no case waits on the real one.
 */
const PRINTED_AT = new Date('2026-10-22T08:10:11.123Z',);

/**
 Clock that reads the same instant every time it is asked.

 @returns Instant the line is printed at

 @example
 ```ts
 const line = destinationsLine({ entryId, destinations, attempt: 2, wallClock: fixedWallClock, },);
 ```
 */
function fixedWallClock(): Date {
  return PRINTED_AT;
}

await describe({
  name: destinationsLine.name,
  children: [
    it({
      name: 'carries the three counts and no address, after the clock time and the attempt',
      fn: async () => {
        const line = destinationsLine({
          entryId: 'BookshopCat',
          attempt: 2,
          wallClock: fixedWallClock,
          destinations: {
            source: [
              'https://example.org/tabby',
              'https://example.org/album',
            ],
            page: ['https://example.org/album',],
            dropped: ['https://example.org/tabby',],
            findings: [],
          },
        },);

        expect(line,).toBe('DESTINATIONS BookshopCat at=2026-10-22T08:10:11.123Z attempt=2 source=2 page=1 dropped=1',);
      },
    },),

    it({
      name: 'appends each finding after the counts',
      fn: async () => {
        const line = destinationsLine({
          entryId: 'BookshopCat',
          attempt: 2,
          wallClock: fixedWallClock,
          destinations: {
            source: [],
            page: [],
            dropped: [],
            findings: ['destinations-mdx-downgraded (page)',],
          },
        },);

        expect(line,).toBe(
          'DESTINATIONS BookshopCat at=2026-10-22T08:10:11.123Z attempt=2 source=0 page=0 dropped=0 '
            + 'destinations-mdx-downgraded (page)',
        );
      },
    },),
  ],
},);
