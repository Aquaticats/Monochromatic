/**
 Guards the Han side of the pairing for a Han name the archive spells in a
 sentence: which stretches of an original's Han are read by their pinyin, and
 which pairs are dropped or kept once every stretch is read. The side is
 reached through `proseNamePairs`. Cat-themed invention throughout; no corpus
 content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { proseNamePairs, } from '../dist/final/node/index.mjs';

await describe({
  name: 'the Han names of an original, read by their pinyin',
  children: [
    describe({
      name: proseNamePairs.name,
      children: [
        it({
          name: 'READS a name inside an unbroken run of Han, and each name of a text in the order the original writes them',
          fn: async () => {
            expect(proseNamePairs({
              sourceText: '昨天糯米糍和雪团子一起睡了。',
              targetText: 'Xue Tuanzi and Nuomici slept.',
            },),).toEqual([
              {
                source: '糯米糍',
                rendering: 'Nuomici',
              },
              {
                source: '雪团子',
                rendering: 'Xue Tuanzi',
              },
            ],);
          },
        },),
        it({
          name: 'KEEPS BOTH SPELLINGS of one name the archive spells two ways',
          fn: async () => {
            expect(proseNamePairs({
              sourceText: '雪团子来了。',
              targetText: 'Xue Tuanzi came. Later Xuetuan Zi left.',
            },),).toEqual([
              {
                source: '雪团子',
                rendering: 'Xue Tuanzi',
              },
              {
                source: '雪团子',
                rendering: 'Xuetuan Zi',
              },
            ],);
          },
        },),
        it({
          name: 'READS NO HAN INSIDE a link address, a code span or a fence, which no reader sees as prose',
          fn: async () => {
            expect(proseNamePairs({
              sourceText: '[文](https://example.invalid/雪团子)\n\n`雪团子`\n\n```\n雪团子\n```\n',
              targetText: 'Xue Tuanzi came.',
            },),).toEqual([],);
          },
        },),
        it({
          name: 'LEAVES a run of Han whose reading no spelling matches, a run past the length bound, and a character beyond the first plane',
          fn: async () => {
            expect(proseNamePairs({
              sourceText: '猫咪来了。',
              targetText: 'Xue Tuanzi came.',
            },),).toEqual([],);
            expect(proseNamePairs({
              sourceText: '猫'.repeat(60,),
              targetText: 'Xue Tuanzi came.',
            },),).toEqual([],);
            expect(proseNamePairs({
              sourceText: '\u{20BB7}\u{20BB7}来了。',
              targetText: 'Xue Tuanzi came.',
            },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
