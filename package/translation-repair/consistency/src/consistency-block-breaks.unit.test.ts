/**
 Tests for counting the block breaks inside some words: the blank lines and the
 line breaks that the Markdown parse reads as the start of a new block, which a
 replacement of words by words with another count cannot keep.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { blockBreaksIn, } from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: blockBreaksIn.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS NO BREAK IN WORDS ON ONE LINE OR WRAPPED BY A LINE BREAK THAT STARTS NO BLOCK, and none in no words',
          fn: async () => {
            expect([
              blockBreaksIn({ text: '', },),
              blockBreaksIn({ text: 'The kitten naps in the sun', },),
              blockBreaksIn({ text: 'The kitten naps\nin the sun', },),
              blockBreaksIn({ text: '> The kitten naps\n> in the sun', },),
            ],).toEqual([0, 0, 0, 0,],);
          },
        },),
        it({
          name: 'COUNTS A BLANK LINE, however long, as one break, and a line break that the parse reads as a new block',
          fn: async () => {
            expect([
              blockBreaksIn({ text: 'The kitten naps\n\nin the sun', },),
              blockBreaksIn({ text: 'The kitten naps\n\n\n\nin the sun', },),
              blockBreaksIn({ text: 'The kitten naps\n \t \nin the sun', },),
              blockBreaksIn({ text: 'One\n\nTwo\n\nThree', },),
              blockBreaksIn({ text: 'The kitten naps\n- in the sun', },),
              blockBreaksIn({ text: 'The kitten naps\n# In the sun', },),
            ],).toEqual([1, 1, 1, 2, 1, 1,],);
          },
        },),
        it({
          name: 'COUNTS BLANK LINES IN TEXT THE GRAMMAR CANNOT READ, where a parse is not to be had',
          fn: async () => {
            expect([
              blockBreaksIn({ text: 'The kitten {naps\n\nin the sun', },),
              blockBreaksIn({ text: '{ The kitten naps\n\n\nin the sun\n\n', },),
            ],).toEqual([1, 1,],);
          },
        },),
      ],
    },),
  ],
},);
