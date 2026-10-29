/**
 Tests for the floor rule refusing a translation that keeps the corpus's
 neutral pronoun in Latin letters.

 WHAT THESE PIN: every spelling the sources use counts; the letters inside a
 word, a handle, a path or an address do not; an apostrophe, a han character,
 a dash, an ellipsis, an arrow, emphasis or a slash after han beside the
 pronoun still leaves it a word of its own, in the original and in the
 rendering (ledger B23); the finding names each spelling with its count and
 says what English renders it as; a candidate carrying none yields nothing.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { neutralPronounFindings, } from '../dist/final/node/index.mjs';

/**
 Original writing the neutral pronoun, so an untranslated one is owed its
 rendering.
 */
const WRITES_TA = 'TA 在窗台上打盹，Ta 的尾巴垂下来。我们给 ta 留了一个房间。';

await describe({
  name: neutralPronounFindings.name,
  children: [
    it({
      name: 'stays quiet on a translation that renders the pronoun, which is the case that must not fire',
      fn: async () => {
        expect(neutralPronounFindings({
          sourceText: WRITES_TA,
          candidateText: 'They dozed on the windowsill, and their tail hung down. We set a room for them.',
        },),).toStrictEqual([],);
        expect(neutralPronounFindings({
          sourceText: WRITES_TA,
          candidateText: '',
        },),).toStrictEqual([],);
      },
    },),

    it({
      name: 'READS THE ORIGINAL: an English TA or Ta where the original writes no neutral pronoun is English, '
        + 'not the pronoun left untranslated (ledger F-9: 助教 as "The TA", a "Ta!" of thanks)',
      fn: async () => {
        expect(neutralPronounFindings({
          sourceText: '助教批改了猫的作业。',
          candidateText: 'The TA graded the cat\'s homework.',
        },),).toStrictEqual([],);
        expect(neutralPronounFindings({
          sourceText: '「谢啦！」橘猫说。',
          candidateText: '“Ta!” said the ginger cat.',
        },),).toStrictEqual([],);
      },
    },),

    it({
      name: 'NAMES each spelling kept and its count, in the order TA, Ta, ta',
      fn: async () => {
        const findings = neutralPronounFindings({
          sourceText: WRITES_TA,
          candidateText: 'TA dozed. Then Ta woke, and ta stretched; Ta purred.',
        },);

        expect(findings.length,).toBe(1,);
        expect(findings[0],).toContain('untranslated as "TA" (1 time) and "Ta" (2 times) and "ta" (1 time)',);
        expect(findings[0],).toContain('English renders it as singular they (they, them, their)',);
      },
    },),

    it({
      name: 'COUNTS the pronoun beside an apostrophe, a comma, a quotation mark or a han character, '
        + 'since none of those makes it part of a longer word',
      fn: async () => {
        const findings = neutralPronounFindings({
          sourceText: WRITES_TA,
          candidateText: 'A room for Ta, to give Ta\'s memorial warmth. “Ta” 的 (Ta)',
        },);

        expect(findings[0],).toContain('"Ta" (4 times)',);
      },
    },),

    it({
      name: 'READS THE PRONOUN AFTER A DASH OR A SLASH IN THE ORIGINAL (ledger B23): a doubled Chinese dash '
        + 'or a slash after han leaves it a word of its own, so the rendering owes its rendering',
      fn: async () => {
        expect(neutralPronounFindings({
          sourceText: '猫说——TA 睡着了。',
          candidateText: 'The cat said TA was asleep.',
        },),).toHaveLength(1,);
        expect(neutralPronounFindings({
          sourceText: '猫/Ta 睡着了。',
          candidateText: 'Ta slept.',
        },),).toHaveLength(1,);
      },
    },),

    it({
      name: 'COUNTS THE PRONOUN BESIDE A DASH, AN ELLIPSIS, AN ARROW, EMPHASIS OR A CLOSING MARK (ledger B23), '
        + 'none of which joins it into a longer token',
      fn: async () => {
        /**
         Finding on a rendering keeping the pronoun beside each of those.
         */
        const findings = neutralPronounFindings({
          sourceText: WRITES_TA,
          candidateText: 'The cat said—TA slept, and ta—curled up. We said *Ta* twice…Ta, then asked: Ta? '
            + '『TA』 Ta→home.',
        },);
        expect(findings[0],).toContain('"TA" (2 times) and "Ta" (4 times) and "ta" (1 time)',);
      },
    },),

    it({
      name: 'IGNORES the letters inside a word, a handle, a path segment, an address or a hyphenated '
        + 'form, which are what a bare word-boundary check would have counted',
      fn: async () => {
        expect(neutralPronounFindings({
          sourceText: WRITES_TA,
          candidateText: 'DATA and STATION stay with @ta_cat at https://example.org/ta/ and meta.ta, ta-da, ta9.',
        },),).toStrictEqual([],);
      },
    },),
  ],
},);
