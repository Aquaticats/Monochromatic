/**
 Tests for finding a surrogate that has no partner, the one kind of text a
 UTF-8 stream cannot carry unchanged.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { loneSurrogateIndex, } from '../dist/final/node/index.mjs';

await describe({
  name: loneSurrogateIndex.name,
  children: [
    it({
      name: 'READS -1 FOR TEXT WITH NONE: plain text, an empty text, and a pair of surrogates, which is one '
        + 'character outside the Basic Multilingual Plane and not two halves',
      fn: async () => {
        expect({
          plain: loneSurrogateIndex({ text: 'The cat naps.', },),
          empty: loneSurrogateIndex({ text: '', },),
          pair: loneSurrogateIndex({ text: 'Astral \u{1F408} naps.', },),
          pairs: loneSurrogateIndex({ text: '\u{1F408}\u{1F344}\u{1D4F7}', },),
        },).toEqual({
          plain: -1,
          empty: -1,
          pair: -1,
          pairs: -1,
        },);
      },
    },),

    it({
      name: 'READS THE UTF-16 INDEX OF A HIGH SURROGATE WITH NO LOW ONE after it, of a low one with no high '
        + 'one before it, and of the first of two',
      fn: async () => {
        expect({
          highInside: loneSurrogateIndex({ text: 'It naps \uD83D and wakes.', },),
          highAtEnd: loneSurrogateIndex({ text: 'It naps\uD83D', },),
          low: loneSurrogateIndex({ text: 'It naps \uDC08 and wakes.', },),
          afterPair: loneSurrogateIndex({ text: '\u{1F408}\uD83D', },),
          first: loneSurrogateIndex({ text: 'ab\uDC08\uD83D', },),
          reversedPair: loneSurrogateIndex({ text: '\uDC08\uD83D', },),
        },).toEqual({
          highInside: 8,
          highAtEnd: 7,
          low: 8,
          afterPair: 2,
          first: 2,
          reversedPair: 0,
        },);
      },
    },),
  ],
},);
