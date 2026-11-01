/**
 Guards class one hundred thirty-seven (hulicaijia20, 2026-09-25): the
 archive's translator note glossed a drug's Han name with its pinyin and
 gave one syllable the wrong tone ("wǎn" for 烷, read wán), and the page
 carried the note as the archive wrote it. Where a parenthesis pairs a Han
 run with its tone-marked pinyin, syllable for character, and a character
 has one reading only, a syllable that differs from it in tone alone is
 written with the character's tone. Cat-themed invention throughout; no
 corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { correctPinyinTones, } from '../../dist/final/node/index.mjs';

await describe({
  name: 'correctPinyinTones (class one hundred thirty-seven)',
  children: [
    it({
      name: 'CORRECTS a syllable whose single-reading character takes another tone',
      fn: async () => {
        expect(correctPinyinTones({ text: 'The cat ate yarn (毛线球, máo xiǎn qiú) all night.', },),).toEqual({
          text: 'The cat ate yarn (毛线球, máo xiàn qiú) all night.',
          changed: ['"xiǎn" to "xiàn" for 线',],
        },);
      },
    },),
    it({
      name: 'CORRECTS a capitalised syllable and keeps its capital',
      fn: async () => {
        expect(correctPinyinTones({ text: 'The cat ate yarn (线, Xiǎn) all night.', },),).toEqual({
          text: 'The cat ate yarn (线, Xiàn) all night.',
          changed: ['"Xiǎn" to "Xiàn" for 线',],
        },);
      },
    },),
    it({
      name: 'KEEPS the apostrophe between syllables',
      fn: async () => {
        expect(correctPinyinTones({ text: 'Cat food (鱼罐, yú’guǎn).', },).text,).toBe('Cat food (鱼罐, yú’guàn).',);
      },
    },),
    it({
      name: 'LEAVES a polyphonic character, a count that does not match, another syllable and toneless pinyin',
      fn: async () => {
        /**
         Pairs the pass must stand aside from.
         */
        const text = [
          'A cat (猫, máo).',
          'Yarn (毛线球, máoxiàn qiǔ).',
          'Fish (鱼, niǎo).',
          'Fish (鱼, yu).',
        ].join('\n',);
        expect(correctPinyinTones({ text, },),).toEqual({
          text,
          changed: [],
        },);
      },
    },),
    it({
      name: 'CORRECTS a syllable whose character lies beyond the first plane, which pinyin-pro reads and a scan by '
        + 'UTF-16 unit never took for Han (ledger B21), and matches the syllables to characters, not to units',
      fn: async () => {
        expect(correctPinyinTones({ text: 'A cat toy (\u{20CD0}\u{732B}, b\u{00E0}ng m\u{0101}o).', },),).toEqual({
          text: 'A cat toy (\u{20CD0}\u{732B}, b\u{0101}ng m\u{0101}o).',
          changed: ['"b\u{00E0}ng" to "b\u{0101}ng" for \u{20CD0}',],
        },);
      },
    },),
  ],
},);
