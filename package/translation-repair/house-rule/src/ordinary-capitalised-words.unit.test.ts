/**
 Tests the words the whole-name arrival check lets stand beside a declared
 form (`ordinary-capitalised-words.ts`): that the set is lower-case, holds the
 sentence openers, titles and months the pinned pages show beside a name, and
 holds no word that is also a romanised Chinese or Japanese syllable, since
 one on it would let a fused name piece through.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { ORDINARY_CAPITALISED_WORDS, } from '../dist/final/node/index.mjs';

await describe({
  name: 'ORDINARY_CAPITALISED_WORDS',
  children: [
    it({
      name: 'HOLDS ONLY LOWER-CASE ASCII WORDS, since a neighbour is compared lower-cased',
      fn: async () => {
        for (const word of ORDINARY_CAPITALISED_WORDS) {
          expect(word,).toBe(word.toLowerCase(),);
          expect(word.length,).toBeGreaterThan(0,);
        }
      },
    },),
    it({
      name: 'HOLDS THE OPENERS, TITLES AND MONTHS the census found beside a declared form',
      fn: async () => {
        for (const word of [
          'when',
          'but',
          'after',
          'although',
          'was',
          'as',
          'perhaps',
          'sister',
          'and',
          'in',
          'for',
          'the',
          'dear',
          'sis',
          'september',
        ]) {
          expect(ORDINARY_CAPITALISED_WORDS.has(word,),).toBe(true,);
        }
      },
    },),
    it({
      name: 'HOLDS NO WORD THAT IS ALSO A ROMANISED SYLLABLE OF A NAME, nor an article, so a fused name '
        + 'piece such as `Xing` or `He` still blocks',
      fn: async () => {
        for (const word of [
          'xing',
          'he',
          'she',
          'can',
          'an',
          'a',
          'ah',
          'long',
          'sun',
          'man',
          'ran',
          'yan',
          'tian',
        ]) {
          expect(ORDINARY_CAPITALISED_WORDS.has(word,),).toBe(false,);
        }
      },
    },),
  ],
},);
