/**
 Tests for the finders of the openers of a comment that never closes, of those that start a block of raw HTML, and for the cut of the
 openers out of raw text.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  blockOpenerStarts,
  openerStarts,
  rangesWithoutOpeners,
} from '../../dist/final/node/index.mjs';

await describe({
  name: 'the openers of a comment that never closes',
  children: [
    describe({
      name: blockOpenerStarts.name,
      children: [
        it({
          name: 'FINDS AN OPENER THAT OPENS THE TEXT OR A LINE, with up to three spaces before it, and every such opener',
          fn: async () => {
            expect(blockOpenerStarts({ text: '<!-- a\n', starts: [0,], },),).toEqual([0,],);
            expect(blockOpenerStarts({ text: '猫\n\n   <!-- a\n', starts: [6,], },),).toEqual([6,],);
            expect(blockOpenerStarts({ text: '猫 <!-- a\n\n<!-- b\n', starts: [2, 10,], },),).toEqual([10,],);
          },
        },),
        it({
          name: 'FINDS AN OPENER AFTER THE MARKS OF A BLOCK QUOTE OR THE MARKER OF A LIST ITEM',
          fn: async () => {
            expect(blockOpenerStarts({ text: '> <!-- a\n', starts: [2,], },),).toEqual([2,],);
            expect(blockOpenerStarts({ text: '> > a\n> > <!-- b\n', starts: [10,], },),).toEqual([10,],);
            expect(blockOpenerStarts({ text: '- <!-- a\n', starts: [2,], },),).toEqual([2,],);
            expect(blockOpenerStarts({ text: '1. a\n   <!-- b\n', starts: [8,], },),).toEqual([8,],);
            // A tab that a list item takes in part is read as spaces the text does not hold.
            expect(blockOpenerStarts({ text: '- i\n\t<!-- a\n', starts: [5,], },),).toEqual([5,],);
          },
        },),
        it({
          name: 'FINDS NONE WHEN TEXT, FOUR SPACES OR A TAB STAND BEFORE THE OPENER ON ITS LINE, or it stands in code',
          fn: async () => {
            expect(blockOpenerStarts({ text: '猫 <!-- a\n', starts: [2,], },),).toEqual([],);
            expect(blockOpenerStarts({ text: '    <!-- a\n', starts: [4,], },),).toEqual([],);
            expect(blockOpenerStarts({ text: '\t<!-- a\n', starts: [1,], },),).toEqual([],);
            expect(blockOpenerStarts({ text: '```\n<!-- a\n```\n', starts: [4,], },),).toEqual([],);
            // An opener inside a block of raw HTML that starts on an earlier line is no start of a block.
            expect(blockOpenerStarts({ text: '<div>\n<!-- a\n', starts: [6,], },),).toEqual([],);
          },
        },),
        it({
          name: 'FINDS NONE FOR A TEXT WITH NO OPENER THAT NEVER CLOSES, or one nested past what the plain reading admits',
          fn: async () => {
            expect(blockOpenerStarts({ text: '猫\n', starts: [], },),).toEqual([],);
            expect(blockOpenerStarts({ text: `${'> '.repeat(300,)}<!-- a\n`, starts: [600,], },),).toEqual([],);
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
