/**
 Tests the line arithmetic and the words the composite step of the archive block review
 says it in (ledger B716, split out of `archive-composite-revision.ts`).

 Fixtures are invented lines about a cat.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  counted,
  linesOf,
  numbered,
  sharedLines,
} from '../dist/final/node/index.mjs';

await describe({
  name: 'archive composite lines',
  children: [
    it({
      name: 'CUTS A TEXT INTO LINES THAT KEEP THEIR ENDINGS, so joining them gives the text back: a text that ends '
        + 'in a line feed has no line after it, and a carriage return stays with its line',
      fn: async () => {
        expect(linesOf({ text: 'a\nb', },),).toEqual(['a\n', 'b',],);
        expect(linesOf({ text: 'a\nb\n', },),).toEqual(['a\n', 'b\n',],);
        expect(linesOf({ text: 'a\r\nb', },),).toEqual(['a\r\n', 'b',],);
        expect(linesOf({ text: 'a\n\nb', },).join('',),).toBe('a\n\nb',);
        expect(linesOf({ text: '', },),).toEqual(['',],);
      },
    },),
    it({
      name: 'SAYS A COUNT WITH ITS NOUN, one form at exactly one',
      fn: async () => {
        expect(counted({ count: 1, one: 'line', many: 'lines', },),).toBe('1 line',);
        expect(counted({ count: 0, one: 'line', many: 'lines', },),).toBe('0 lines',);
        expect(counted({ count: 4, one: 'line', many: 'lines', },),).toBe('4 lines',);
      },
    },),
    it({
      name: 'NUMBERS LINES FROM ONE, in the singular for one position',
      fn: async () => {
        expect(numbered({ indexes: [3,], },),).toBe('line 4',);
        expect(numbered({ indexes: [0, 1, 5,], },),).toBe('lines 1, 2, 6',);
      },
    },),
    it({
      name: 'FINDS THE POSITIONS TWO REVISIONS BOTH CHANGE, in ascending order, and none for disjoint sets',
      fn: async () => {
        expect(sharedLines({ left: [0, 2, 5,], right: [2, 5, 7,], },),).toEqual([2, 5,],);
        expect(sharedLines({ left: [0, 1,], right: [2, 3,], },),).toEqual([],);
      },
    },),
  ],
},);
