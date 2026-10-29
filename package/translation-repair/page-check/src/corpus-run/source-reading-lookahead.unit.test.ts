/**
 Tests for what the blocks after a block ask of the places it ends at.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  lookaheadsOf,
} from '../../dist/final/node/index.mjs';

await describe({
  name: lookaheadsOf.name,
  children: [
    it({
      name: 'GIVES EACH BLOCK THE WORDS OF THE BLOCKS UP TO THE NEXT WITH A STRETCH BEFORE IT, and what that stretch costs',
      fn: async () => {
        expect(lookaheadsOf({
          segments: [
            { reached: false, words: ['a',], blocks: 0, },
            { reached: true, words: ['x',], blocks: 3, },
            { reached: false, words: ['b',], blocks: 0, },
            { reached: false, words: ['c', 'd',], blocks: 0, },
            { reached: true, words: [], blocks: 1, },
            { reached: false, words: ['e',], blocks: 0, },
          ],
        },),).toEqual([
          { words: [], cost: Number.POSITIVE_INFINITY, },
          { words: [], cost: Number.POSITIVE_INFINITY, },
          { words: ['c', 'd',], cost: 1, },
          { words: [], cost: Number.POSITIVE_INFINITY, },
          { words: [], cost: Number.POSITIVE_INFINITY, },
          { words: [], cost: Number.POSITIVE_INFINITY, },
        ],);
      },
    },),
  ],
},);
