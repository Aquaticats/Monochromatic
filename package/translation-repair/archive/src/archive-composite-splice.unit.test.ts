/**
 Tests the application of several revisions' hunks to the archive block together (ledger B803): the order they land
 in, the lines of the combined text each revision is credited with, and the removal that leaves a revision nothing to
 point at.

 Fixtures are invented lines about a cat; no corpus text appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { spliceHunks, } from '../dist/final/node/index.mjs';

await describe({
  name: 'archive composite splice',
  children: [
    it({
      name: 'APPLIES HUNKS IN BLOCK ORDER WHATEVER ORDER THEY ARE GIVEN IN, an insertion ahead of the hunk that '
        + 'begins at its gap and after the hunk that ends there',
      fn: async () => {
        const spliced = spliceHunks({
          base: ['a\n', 'b\n', 'c\n', 'd\n', 'e',],
          revisions: [
            [{ start: 3, end: 4, lines: ['D\n',], },],
            [{ start: 3, end: 3, lines: ['x\n',], },],
            [{ start: 1, end: 3, lines: ['BC\n',], },],
            [{ start: 4, end: 4, lines: ['y\n',], },],
          ],
        },);
        expect(spliced.lines.join('',),).toBe('a\nBC\nx\nD\ny\ne',);
        expect(spliced.positions,).toEqual([[3,], [2,], [1,], [4,],],);
      },
    },),
    it({
      name: 'PLACES A REMOVAL AT THE LINE THAT NOW FOLLOWS THE CUT, and on the last line when the cut is at the end '
        + 'or the text is left with one line',
      fn: async () => {
        const base = ['a\n', 'b\n', 'c',];
        expect(spliceHunks({
          base,
          revisions: [[{ start: 1, end: 2, lines: [], },],],
        },).positions,).toEqual([[1,],],);
        expect(spliceHunks({
          base,
          revisions: [[{ start: 2, end: 3, lines: [], },],],
        },).positions,).toEqual([[1,],],);
        /** Everything removed, so no line exists to place the cut on. */
        const empty = spliceHunks({
          base,
          revisions: [[{ start: 0, end: 3, lines: [], },],],
        },);
        expect(empty.lines,).toEqual([],);
        expect(empty.positions,).toEqual([[],],);
      },
    },),
    it({
      name: 'COPIES THE BLOCK WHEN NO REVISION HAS A HUNK',
      fn: async () => {
        const base = ['a\n', 'b',];
        expect(spliceHunks({ base, revisions: [[], [],], },),).toEqual({ lines: base, positions: [[], [],], },);
      },
    },),
  ],
},);
