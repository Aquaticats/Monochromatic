/**
 Tests for the finder of the opener that starts a block of raw HTML, and for the cut of the openers out of raw text.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  blockOpenerStart,
  openerStarts,
  rangesWithoutOpeners,
} from '../../dist/final/node/index.mjs';

await describe({
  name: 'the openers of a comment that never closes',
  children: [
    describe({
      name: blockOpenerStart.name,
      children: [
        it({
          name: 'FINDS THE FIRST OPENER WHEN IT OPENS THE TEXT OR A LINE, with up to three spaces before it',
          fn: async () => {
            expect(blockOpenerStart({ text: '<!-- a\n', starts: [0, 9,], },),).toBe(0,);
            expect(blockOpenerStart({ text: '猫\n\n   <!-- a\n', starts: [6,], },),).toBe(6,);
          },
        },),
        it({
          name: 'FINDS NONE WHEN TEXT, A QUOTE MARK, FOUR SPACES OR A TAB STAND BEFORE THE OPENER ON ITS LINE',
          fn: async () => {
            expect(blockOpenerStart({ text: '猫 <!-- a\n', starts: [2,], },),).toBe(-1,);
            expect(blockOpenerStart({ text: '> <!-- a\n', starts: [2,], },),).toBe(-1,);
            expect(blockOpenerStart({ text: '    <!-- a\n', starts: [4,], },),).toBe(-1,);
            expect(blockOpenerStart({ text: '\t<!-- a\n', starts: [1,], },),).toBe(-1,);
          },
        },),
        it({
          name: 'FINDS NONE FOR A TEXT WITH NO OPENER THAT NEVER CLOSES',
          fn: async () => {
            expect(blockOpenerStart({ text: '猫\n', starts: [], },),).toBe(-1,);
          },
        },),
      ],
    },),
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
