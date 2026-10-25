/**
 Tests for the finder of the openers of a comment that never closes and for the cut of the openers out of raw text.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  openerStarts,
  rangesWithoutOpeners,
} from '../../dist/final/node/index.mjs';

await describe({
  name: 'the openers of a comment that never closes',
  children: [
    describe({
      name: openerStarts.name,
      children: [
        it({
          name: 'FINDS EACH OPENER FROM AN INDEX ON, in text order',
          fn: async () => {
            expect(openerStarts({ text: '<!-- a <!-- b', from: 0, },),).toEqual([0, 7,],);
            expect(openerStarts({ text: '<!-- a <!-- b', from: 1, },),).toEqual([7,],);
            expect(openerStarts({ text: '猫 {', from: 0, },),).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: rangesWithoutOpeners.name,
      children: [
        it({
          name: 'CUTS THE FOUR CHARACTERS OF EACH OPENER OUT OF A STRETCH THAT HOLDS IT',
          fn: async () => {
            expect(rangesWithoutOpeners({ ranges: [{ start: 0, end: 20, },], starts: [0, 10,], },),).toEqual([
              { start: 4, end: 10, },
              { start: 14, end: 20, },
            ],);
            expect(rangesWithoutOpeners({ ranges: [{ start: 2, end: 20, },], starts: [6,], },),).toEqual([
              { start: 2, end: 6, },
              { start: 10, end: 20, },
            ],);
          },
        },),
        it({
          name: 'LEAVES A STRETCH THAT HOLDS NO OPENER, and drops one that is only an opener',
          fn: async () => {
            expect(rangesWithoutOpeners({ ranges: [{ start: 0, end: 5, },], starts: [9,], },),).toEqual([{ start: 0, end: 5, },],);
            expect(rangesWithoutOpeners({ ranges: [{ start: 3, end: 7, },], starts: [3,], },),).toEqual([],);
            expect(rangesWithoutOpeners({ ranges: [{ start: 0, end: 5, },], starts: [], },),).toEqual([{ start: 0, end: 5, },],);
          },
        },),
      ],
    },),
  ],
},);
