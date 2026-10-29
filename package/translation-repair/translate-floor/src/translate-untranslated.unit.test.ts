/**
 Guards class sixty-five (XingZ615, 2026-09-19): a consolidation writer
 returned the Chinese original of the closing poem with a trailing space
 after each bare `>` line, the untranslated check compared it character for
 character after trimming the ends, the floor accepted it as the slate's
 only valid candidate, three of four judges abstained ("not a translation;
 a verbatim copy of the Chinese original") and the entry stopped. A copy
 that differs from the original only in whitespace is the original. Cat-themed
 invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  untranslatedFindings,
  validateTranslatedSlice,
} from '../dist/final/node/index.mjs';

/**
 Original: a quoted verse with bare quote lines between the verses and a
 closing tag after it.
 */
const SOURCE_TEXT = '> 猫在窗台上打盹\n>\n> 尾巴垂在暖气片旁\n>\n> <p style="text-align: end;">——猫猫 2024.1.1</p>\n\n</details>';

/**
 The same text with a space after each bare quote line and Windows line
 ends, which is what the writer returned.
 */
const RESPACED_COPY = '> 猫在窗台上打盹\r\n> \r\n> 尾巴垂在暖气片旁\r\n> \r\n> <p style="text-align: end;">——猫猫 2024.1.1</p>\r\n\r\n</details>';

/**
 A rendering of it.
 */
const TRANSLATION = '> The cat dozes on the windowsill\n>\n> Tail draped beside the radiator\n>\n> <p style="text-align: end;">——Maomao 2024.1.1</p>\n\n</details>';

/**
 A slice with nothing to translate, returned as it stands.
 */
const COMPONENT = '<TextRing text="⊕⊕⊕⊕" fontSize="1.25rem"/>';

await describe({
  name: 'a copy of the original that differs only in whitespace is untranslated (class sixty-five, XingZ615)',
  children: [
    it({
      name: 'FINDS the respaced copy untranslated',
      fn: async () => {
        expect(untranslatedFindings({
          sourceText: SOURCE_TEXT,
          candidateText: RESPACED_COPY,
        },).length,).toBe(1,);
      },
    },),
    it({
      name: 'STILL FINDS the exact copy untranslated',
      fn: async () => {
        expect(untranslatedFindings({
          sourceText: SOURCE_TEXT,
          candidateText: `\n${SOURCE_TEXT}\n`,
        },).length,).toBe(1,);
      },
    },),
    it({
      name: 'PASSES a rendering and a component returned as it stands',
      fn: async () => {
        expect(untranslatedFindings({
          sourceText: SOURCE_TEXT,
          candidateText: TRANSLATION,
        },),).toEqual([],);
        expect(untranslatedFindings({
          sourceText: COMPONENT,
          candidateText: `${COMPONENT} `,
        },),).toEqual([],);
      },
    },),
    it({
      name: 'REFUSES the respaced copy at the deterministic floor',
      fn: async () => {
        const validation = validateTranslatedSlice({
          sourceText: SOURCE_TEXT,
          candidateText: RESPACED_COPY,
          pageText: '',
          lineStructured: true,
        },);
        expect(validation.kind,).toBe('invalid',);
      },
    },),
  ],
},);
