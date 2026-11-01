/**
 Tests for the line `verify-published` prints for a page shipped without the original of some slices.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { sourceDroppedLinesOf, } from '../../dist/final/node/index.mjs';

await describe({
  name: sourceDroppedLinesOf.name,
  children: [
    it({
      name: 'NAMES THE ENTRY, THE DEFECT AND THE SLICES in one line, and no passage',
      fn: async () => {
        expect(sourceDroppedLinesOf({
          entryId: 'CatEntry1',
          slices: [88, 91,],
        },),).toEqual([
          'CatEntry1: page-source-unparseable, the page ships without the source text of slices [88, 91]',
        ],);
      },
    },),

    it({
      name: 'PRINTS NOTHING FOR A PAGE THAT SHIPS EVERY ORIGINAL, the control for the line',
      fn: async () => {
        expect(sourceDroppedLinesOf({
          entryId: 'CatEntry1',
          slices: [],
        },),).toEqual([],);
      },
    },),
  ],
},);
