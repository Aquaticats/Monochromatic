/**
 Guards class one hundred forty-five (XingZ6013, 2026-09-26). The original
 credits a song as 《[title](url)》, the title the link's text. The
 consolidated candidate wrote an English title the web lookup offered and
 kept the Han as the link's text in parentheses after it, so the class
 ninety-eight floor read the Han as a gloss beside its English and let it
 through; the title pass then rewrote the Han link text into the heading's
 rendering, and the page credited one song under two English titles. A
 link's text is what the reader sees as the work's name, so a Han title
 standing as a link's whole text is never a gloss. Cat-themed invention
 throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { validateTranslatedSlice, } from '../dist/final/node/index.mjs';

/**
 Original naming the song through a link inside the brackets.
 */
const LINKED = '她唱了《[猫猫摇篮曲](https://example.test/song)》。';

await describe({
  name: 'a Han title kept as a link\'s text (class one hundred forty-five)',
  children: [
    it({
      name: 'REFUSES a candidate that writes the English outside the link and keeps the Han as its text in parentheses',
      fn: async () => {
        /**
         Verdict on the rendering whose link text stayed Han.
         */
        const verdict = validateTranslatedSlice({
          sourceText: LINKED,
          candidateText: 'She sang “Kitten Lullaby” ([猫猫摇篮曲](https://example.test/song)).',
        },);
        expect(verdict.kind,).toBe('invalid',);
        if (verdict.kind !== 'invalid')
          throw new Error('unreachable',);
        expect(verdict.findings.join('\n',),).toContain('leaves the title 《猫猫摇篮曲》',);
      },
    },),
    it({
      name: 'ACCEPTS the English as the link\'s text with the Han glossed after the link',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: LINKED,
          candidateText: 'She sang [Kitten Lullaby](https://example.test/song) (猫猫摇篮曲).',
        },).kind,).toBe('valid',);
      },
    },),
  ],
},);
