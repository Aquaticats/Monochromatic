/**
 Guards class eighty-three's reading of a handle the archive never rendered
 (XingZ622, 2026-09-22). Owner, 2026-09-22: pinyin in syllable groups, never
 one joined word ("Jiecheng Tianzou" or "Jie Cheng Tian Zou" are fine,
 "Jiechengtianzou" is not); the reading pairs the syllables, capitalised,
 a lone trailing syllable standing by itself. Cat-themed invention
 throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  carriesRendering,
  handleReading,
  withoutGloss,
} from '../../dist/final/node/index.mjs';

await describe({
  name: 'a handle the archive never rendered is read as pinyin in syllable groups (class eighty-three, XingZ622)',
  children: [
    it({
      name: 'READS two syllables as one capitalised group',
      fn: async () => {
        expect(handleReading({ name: '锦猫', },),).toBe('Jinmao',);
      },
    },),
    it({
      name: 'READS four syllables as two capitalised groups, never one joined word (owner 2026-09-22: '
        + '"Jiechengtianzou is not fine")',
      fn: async () => {
        expect(handleReading({ name: '洁澄天猫', },),).toBe('Jiecheng Tianmao',);
      },
    },),
    it({
      name: 'LEAVES a lone trailing syllable standing by itself',
      fn: async () => {
        expect(handleReading({ name: '雨猫花', },),).toBe('Yumao Hua',);
      },
    },),
    it({
      name: 'KEEPS Latin letters the original writes against the handle, a space apart from the reading',
      fn: async () => {
        expect(handleReading({ name: '洁澄天猫Official', },),).toBe('Jiecheng Tianmao Official',);
        expect(handleReading({ name: '白猫 suki', },),).toBe('Baimao suki',);
        expect(handleReading({ name: 'Mikä', },),).toBe('Mikä',);
      },
    },),
    // Pinned before the reading stopped rebuilding its text a character at
    // a time (ledger B29): a character beyond the first plane between two
    // runs moves every index after it by two units.
    it({
      name: 'READS each run on its own side of a character beyond the first plane, and keeps that character',
      fn: async () => {
        expect(handleReading({ name: '猫猫\u{1F408}猫', },),).toBe('Maomao\u{1F408}Mao',);
        expect(handleReading({ name: '\u{1F408}雨猫', },),).toBe('\u{1F408}Yumao',);
      },
    },),
    // The ledger B29 mutation batch: the run's own offset moves by two units
    // past a Han character beyond the first plane too, or the run is cut
    // short and the syllables after it are lost. Only the syllable after it
    // is asserted: pinyin-pro reads no such character, which passes through
    // as written, and the ledger records that rather than this test pinning it.
    it({
      name: 'READS every syllable of a run that opens on a Han character beyond the first plane',
      fn: async () => {
        expect(handleReading({ name: '\u{20BB7}猫 cat', },)
          .toLowerCase()
          .endsWith('mao cat',),).toBe(true,);
      },
    },),
    it({
      name: 'KEEPS AN ACCENTED WORD a space apart too, opening after the reading or ending, accent composed or '
        + 'combining, before it; ASCII letters alone wrote them against the reading (ledger B18)',
      fn: async () => {
        expect([
          handleReading({ name: '雨猫Émile', },),
          handleReading({ name: 'Café雨猫', },),
          handleReading({ name: 'Cafe\u{0301}雨猫', },),
        ],).toEqual([
          'Yumao Émile',
          'Café Yumao',
          'Cafe\u{0301} Yumao',
        ],);
      },
    },),
    it({
      name: 'ACCEPTS a rendering carrying its literal meaning in parentheses as the rendering itself',
      fn: async () => {
        expect(withoutGloss({ rendering: 'Jinmao (Brocade Cat)', },),).toBe('Jinmao',);
        expect(carriesRendering({
          written: 'Jinmao (Brocade Cat)',
          rendering: 'Jinmao',
        },),).toBe(true,);
        expect(carriesRendering({
          written: 'Jin Mao',
          rendering: 'Jinmao',
        },),).toBe(false,);
      },
    },),

    it({
      name: 'KEEPS a rendering whose parenthesis opens with no space before it, since that shape is part '
        + 'of the name rather than a gloss',
      fn: async () => {
        expect(withoutGloss({ rendering: 'Jinmao(Brocade)', },),).toBe('Jinmao(Brocade)',);
      },
    },),
  ],
},);
