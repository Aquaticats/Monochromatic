/**
 Tests for ASCII character-set constants.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ASCII_DECIMAL_DIGIT_CHARS,
  ASCII_LOWERCASE_ALPHANUMERIC_CHARS,
  ASCII_LOWERCASE_LETTER_CHARS,
} from '../dist/final/neutral/index.mjs';

//region Expected fixtures

/** Empty separator for joining character arrays into comparison strings. */
const EMPTY_SEPARATOR = '';

/** Lowercase ASCII letters in code-point order. */
const EXPECTED_LOWERCASE_LETTERS = 'abcdefghijklmnopqrstuvwxyz';

/** Decimal ASCII digits in code-point order. */
const EXPECTED_DECIMAL_DIGITS = '0123456789';

/** Lowercase ASCII letters followed by decimal ASCII digits. */
const EXPECTED_LOWERCASE_ALPHANUMERIC = `${EXPECTED_LOWERCASE_LETTERS}${EXPECTED_DECIMAL_DIGITS}`;

//endregion Expected fixtures

await describe({
  name: 'ascii',
  children: [
    it({
      name: 'exports lowercase ASCII letters in code-point order',
      fn: async () => {
        expect(ASCII_LOWERCASE_LETTER_CHARS.join(EMPTY_SEPARATOR,),).toBe(
          EXPECTED_LOWERCASE_LETTERS,
        );
      },
    },),
    it({
      name: 'exports decimal ASCII digits in code-point order',
      fn: async () => {
        expect(ASCII_DECIMAL_DIGIT_CHARS.join(EMPTY_SEPARATOR,),).toBe(
          EXPECTED_DECIMAL_DIGITS,
        );
      },
    },),
    it({
      name: 'exports lowercase alphanumeric chars as letters then digits',
      fn: async () => {
        expect(ASCII_LOWERCASE_ALPHANUMERIC_CHARS.join(EMPTY_SEPARATOR,),)
          .toBe(EXPECTED_LOWERCASE_ALPHANUMERIC,);
      },
    },),
  ],
},);
