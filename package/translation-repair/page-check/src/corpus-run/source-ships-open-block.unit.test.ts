/**
 Tests for the finder of the openers of a comment that never closes at which a reader of the source reads a block of raw
 HTML start.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { blockOpenerStarts, } from '../../dist/final/node/index.mjs';

await describe({
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
        // An opener inside a block of raw HTML that began on a line before the opener's own is no start of a block.
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
},);
