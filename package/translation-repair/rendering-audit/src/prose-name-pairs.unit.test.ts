/**
 Guards the archive's Latin spelling of a Han name written inside a sentence.
 A judged polish re-romanized a cited author's handle that the archive already
 spelled in its sentence ("Han Lianyi" became "Han Lian Yi (Cold Ripples)"),
 because no sheet and no floor knew the archive had a rendering: the glossary
 reads link texts, signatures and headings only. The pairing reads a Han name
 and the archive's capitalised Latin words by their reading letters.
 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  proseNamePairs,
  respelledRendering,
} from '../dist/final/node/index.mjs';

/**
 Original: a cited author named in a sentence, written in Han.
 */
const SOURCE_TEXT = '在她离世后，雪团子为她写下了[悼文](https://example.invalid/memorial)。';

/**
 Archive spelling that author in its sentence.
 */
const ARCHIVE_TEXT = 'After her departure, Xue Tuanzi wrote an [epitaph](https://example.invalid/memorial) for her.';

await describe({
  name: 'the archive spells a Han name in its sentences',
  children: [
    describe({
      name: proseNamePairs.name,
      children: [
        it({
          name: 'PAIRS a Han name with the capitalised Latin words whose reading letters it shares',
          fn: async () => {
            expect(proseNamePairs({ sourceText: SOURCE_TEXT, targetText: ARCHIVE_TEXT, },),).toEqual([
              {
                source: '雪团子',
                rendering: 'Xue Tuanzi',
              },
            ],);
          },
        },),
        it({
          name: 'KEEPS the whole name and drops the part of it another pairing already covers',
          fn: async () => {
            expect(proseNamePairs({
              sourceText: '雪团子来了。团子睡了。',
              targetText: 'Xue Tuanzi came. Tuanzi slept.',
            },),).toEqual([
              {
                source: '雪团子',
                rendering: 'Xue Tuanzi',
              },
            ],);
          },
        },),
        it({
          name: 'FINDS a name after a capital that opens the sentence',
          fn: async () => {
            expect(proseNamePairs({
              sourceText: '今天雪团子来了。',
              targetText: 'Today Xue Tuanzi came.',
            },),).toEqual([
              {
                source: '雪团子',
                rendering: 'Xue Tuanzi',
              },
            ],);
          },
        },),
        it({
          name: 'LEAVES OUT an English word that shares its letters with Han text, when the archive also writes it in lower case',
          fn: async () => {
            expect(proseNamePairs({
              sourceText: '她可以睡了。',
              targetText: 'Take your time. She may take a nap.',
            },),).toEqual([],);
          },
        },),
        it({
          name: 'LEAVES OUT a rendering that more than one Han text reads as',
          fn: async () => {
            expect(proseNamePairs({
              sourceText: '猫去了上海，不想受到伤害。',
              targetText: 'The cat went to Shanghai and wished to stay safe.',
            },),).toEqual([],);
          },
        },),
        it({
          name: 'LEAVES OUT a spelling inside a link address, a single syllable, and capitals throughout',
          fn: async () => {
            expect(proseNamePairs({
              sourceText: '雪团子写了[悼文](https://example.invalid/memorial)。猫来了。',
              targetText: 'She wrote an [epitaph](https://example.invalid/Xuetuanzi). Mao came. XUETUANZI.',
            },),).toEqual([],);
          },
        },),
        it({
          name: 'PAIRS nothing when either text has no Han or no Latin name',
          fn: async () => {
            expect(proseNamePairs({ sourceText: '', targetText: ARCHIVE_TEXT, },),).toEqual([],);
            expect(proseNamePairs({ sourceText: SOURCE_TEXT, targetText: '', },),).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: respelledRendering.name,
      children: [
        it({
          name: 'NAMES the spellings of a text that share the rendering\'s letters and differ from it',
          fn: async () => {
            expect(respelledRendering({
              text: 'Xuetuan Zi (Snow Dumpling) wrote it. Xue Tuan Zi wrote it too.',
              rendering: 'Xue Tuanzi',
            },),).toEqual(['Xuetuan Zi', 'Xue Tuan Zi',],);
          },
        },),
        it({
          name: 'NAMES nothing when the text writes the rendering as it stands, or does not write the name',
          fn: async () => {
            expect(respelledRendering({
              text: 'Xue Tuanzi (Snow Dumpling) wrote it, and Xue Tuan Zi did not.',
              rendering: 'Xue Tuanzi',
            },),).toEqual([],);
            expect(respelledRendering({ text: 'She wrote it.', rendering: 'Xue Tuanzi', },),).toEqual([],);
          },
        },),
        it({
          name: 'READS only capitalised words a single space apart',
          fn: async () => {
            expect(respelledRendering({
              text: 'xue tuanzi, XUE TUANZI, Xue  Tuanzi',
              rendering: 'Xue Tuanzi',
            },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
