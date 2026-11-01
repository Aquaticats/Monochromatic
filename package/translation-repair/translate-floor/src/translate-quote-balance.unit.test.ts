/**
 Guards class one hundred sixty-five (TianqiChen6665, 2026-09-26): the
 archive's rendering of a three-line final message carried one closing
 quotation mark and no opening one, where the original quotes every line with
 「」, and the page shipped it unendorsed. A closing double quotation mark
 with no opening mark before it is refused where the original's own
 quotation marks close nothing they did not open.

 Class one hundred seventy-five (TianqiChen66610, 2026-09-26): the same
 stray closer shipped as a straight double quote, which the floor left alone
 and the typography restore left straight on its odd count. A straight
 double quote is read by its shape: closing after a word or punctuation,
 opening before one.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { validateTranslatedSlice, } from '../dist/final/node/index.mjs';

/**
 Original in which the kitten says one quoted line.
 */
const QUOTED = '「小猫还想更可爱一点。」';

/**
 Original whose one quotation runs over two paragraphs.
 */
const LONG_QUOTE = '「小猫还想更可爱一点。\n\n小猫还想更可靠一点。」';

/**
 Original closing a quotation a slice before it opened.
 */
const TAIL_OF_QUOTE = '小猫还想更可靠一点。」';

await describe({
  name: 'closing quotation marks with no opening one (class one hundred sixty-five)',
  children: [
    it({
      name: 'REFUSES a closing mark with no opening mark before it',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: QUOTED,
          candidateText: 'The kitten still wants to be a little cuter.”',
        },).kind,).toBe('invalid',);
      },
    },),
    it({
      name: 'PASSES a quotation opened and closed',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: QUOTED,
          candidateText: '“The kitten still wants to be a little cuter.”',
        },).kind,).toBe('valid',);
      },
    },),
    it({
      name: 'PASSES a quotation over two paragraphs that opens each and closes the last',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: LONG_QUOTE,
          candidateText: '“The kitten still wants to be a little cuter.\n\n“The kitten still wants to be more reliable.”',
        },).kind,).toBe('valid',);
      },
    },),
    it({
      name: 'STANDS ASIDE where the original itself closes a quotation it did not open',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: TAIL_OF_QUOTE,
          candidateText: 'The kitten still wants to be more reliable.”',
        },).kind,).toBe('valid',);
      },
    },),
    it({
      name: 'REFUSES a straight closing double quote with no opening mark before it (class one hundred seventy-five)',
      fn: async () => {
        /**
         Verdict on a line whose straight double quote closes nothing.
         */
        const verdict = validateTranslatedSlice({
          sourceText: QUOTED,
          candidateText: 'The kitten still wants to be a little cuter."',
        },);
        expect(verdict.kind,).toBe('invalid',);
        expect((verdict.kind === 'invalid') && verdict.findings.some(function namesStray(finding,): boolean {
          return finding.includes('closes a quotation it never opened',);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'PASSES straight double quotes opened and closed',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: QUOTED,
          candidateText: '"The kitten still wants to be a little cuter."',
        },).kind,).toBe('valid',);
      },
    },),
    it({
      name: 'PASSES a curly opening mark closed by a straight one',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: QUOTED,
          candidateText: '“The kitten still wants to be a little cuter."',
        },).kind,).toBe('valid',);
      },
    },),
    it({
      name: 'STANDS ASIDE for a straight double quote after a digit, an inch mark',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: '小猫的窗台有二十英寸宽。',
          candidateText: 'The kitten’s windowsill is 20" wide.',
        },).kind,).toBe('valid',);
      },
    },),
  ],
},);
