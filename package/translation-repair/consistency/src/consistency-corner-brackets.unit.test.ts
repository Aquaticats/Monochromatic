/**
 Tests for telling a white corner bracket from the single marks the fold writes
 it as: the one question left of the reading of what each single quotation mark
 is, which the edge reading of a quote no longer asks.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { isCornerBracket, } from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: isCornerBracket.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'IS TRUE FOR THE TWO WHITE CORNER BRACKETS AND FOR NOTHING ELSE THAT FOLDS TO THE SINGLE MARK, nor for no '
            + 'character at all',
          fn: async () => {
            expect([
              '『',
              '』',
              '‘',
              '’',
              '\'',
              '',
            ].map(function cornered(original,): boolean {
              return isCornerBracket({ original, },);
            },),).toEqual([true, true, false, false, false, false,],);
          },
        },),
      ],
    },),
  ],
},);
