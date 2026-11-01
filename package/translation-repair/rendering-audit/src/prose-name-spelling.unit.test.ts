/**
 Guards the Latin side of the pairing for a Han name the archive spells in a
 sentence: which capitalised words are read as one spelling, and which words
 are no spelling at all. The side is reached through `respelledRendering`,
 the one caller that reads a candidate with it. Cat-themed invention
 throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { respelledRendering, } from '../dist/final/node/index.mjs';

await describe({
  name: 'the Latin spellings of a text, read by their letters',
  children: [
    describe({
      name: respelledRendering.name,
      children: [
        it({
          name: 'READS a name after a capital that opens the sentence, and a name of three words',
          fn: async () => {
            expect(respelledRendering({
              text: 'Today Xuetuan Zi came.',
              rendering: 'Xue Tuanzi',
            },),).toEqual(['Xuetuan Zi',],);
            expect(respelledRendering({
              text: 'Then Xue Tuan Zi came.',
              rendering: 'Xue Tuan Zi Mao',
            },),).toEqual([],);
            expect(respelledRendering({
              text: 'Then Xue Tuan Zimao came.',
              rendering: 'Xue Tuan Zi Mao',
            },),).toEqual(['Xue Tuan Zimao',],);
          },
        },),
        it({
          name: 'FOLDS accents and capitals into the reading, and keeps the spelling as written',
          fn: async () => {
            expect(respelledRendering({
              text: 'Xué Tuan Zi came.',
              rendering: 'Xue Tuanzi',
            },),).toEqual(['Xué Tuan Zi',],);
          },
        },),
        it({
          name: 'LEAVES OUT a single word the text also writes in lower case, a reading under four letters, and a spelling in a link address',
          fn: async () => {
            expect(respelledRendering({
              text: 'Take it. We take turns.',
              rendering: 'Take',
            },),).toEqual([],);
            expect(respelledRendering({
              text: 'Mao came.',
              rendering: 'Mao',
            },),).toEqual([],);
            expect(respelledRendering({
              text: 'See [it](https://example.invalid/Xuetuan-Zi) for more.',
              rendering: 'Xue Tuanzi',
            },),).toEqual([],);
          },
        },),
        it({
          name: 'STOPS a run at a word that is no name, at a gap of more than one space, and at the sixth word',
          fn: async () => {
            expect(respelledRendering({
              text: 'Xue and Tuanzi came.',
              rendering: 'Xue Tuanzi',
            },),).toEqual([],);
            expect(respelledRendering({
              text: 'Xue  Tuanzi came.',
              rendering: 'Xue Tuanzi',
            },),).toEqual([],);
            expect(respelledRendering({
              text: 'Ab Cd Ef Gh Ij Kl came.',
              rendering: 'Abcdefghijkl',
            },),).toEqual(['Ab Cd Ef Gh Ij Kl',],);
            expect(respelledRendering({
              text: 'Ab Cd Ef Gh Ij Kl Mn came.',
              rendering: 'Abcdefghijklmn',
            },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
