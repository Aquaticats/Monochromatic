/**
 Tests the package's Han character tests (audit area six): every block of Han
 ideographs counts, nothing else does, and the tokenizer's test agrees with
 the floors' on every character one UTF-16 unit holds. The two once read
 different blocks: the floors' the unified block alone, the tokenizer's
 Extension A as well, and neither the compatibility block.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  carriesAsciiLetter,
  isHanCharacter,
} from '@monochromatic-dev/translation-repair-markup';
import { isIdeograph, } from '../dist/final/node/index.mjs';

/**
 One character from each block of Han ideographs, one UTF-16 unit each.
 */
const SINGLE_UNIT_HAN = [
  // CJK Unified Ideographs.
  '猫',
  // Extension A, the first code point.
  '\u{3400}',
  // Compatibility Ideographs, the first code point.
  '\u{F900}',
];

/**
 A Han ideograph outside the Basic Multilingual Plane (Extension B).
 */
const ASTRAL_HAN = '\u{20000}';

/**
 Characters that sit near the Han blocks and are not Han: a Latin letter,
 katakana, a CJK radical, a fullwidth comma and a Hangul syllable.
 */
const NOT_HAN = ['a', 'ア', '\u{2E80}', '，', '한',];

await describe({
  name: 'the Han character tests',
  children: [
    it({
      name: 'COUNT EVERY BLOCK OF HAN IDEOGRAPHS and nothing near them, so a floor reads one page one way',
      fn: async () => {
        expect({
          han: [...SINGLE_UNIT_HAN, ASTRAL_HAN,].map(function tested(character,): boolean {
            return isHanCharacter({ character, },);
          },),
          notHan: NOT_HAN.map(function tested(character,): boolean {
            return isHanCharacter({ character, },);
          },),
        },).toEqual({
          han: [true, true, true, true,],
          notHan: [false, false, false, false, false,],
        },);
      },
    },),
    it({
      name: 'AGREE BETWEEN THE FLOORS AND THE TOKENIZER on every character one unit holds',
      fn: async () => {
        expect([...SINGLE_UNIT_HAN, ...NOT_HAN,].filter(function disagree(character,): boolean {
          return isHanCharacter({ character, },) !== isIdeograph(character,);
        },),).toEqual([],);
      },
    },),

    it({
      name: 'READS AN EMPTY STRING as no character at all, and CARRIES an ASCII letter where one is in '
        + 'the range and not where the text\'s one letter stands outside it',
      fn: async () => {
        expect(isHanCharacter({ character: '', },),).toBe(false,);
        expect(carriesAsciiLetter({ text: '猫a猫', from: 0, to: 3, },),).toBe(true,);
        expect(carriesAsciiLetter({ text: '猫猫猫', from: 0, to: 3, },),).toBe(false,);
        expect(carriesAsciiLetter({ text: 'a猫猫', from: 1, to: 3, },),).toBe(false,);
      },
    },),
  ],
},);
