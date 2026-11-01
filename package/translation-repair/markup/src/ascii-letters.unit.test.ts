/**
 Tests the ASCII letter and digit tests the package's scanners share (audit
 area six): each range at its edges, the characters beside them, and the
 letters an ASCII test does not admit.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isAsciiAlphanumeric,
  isAsciiDigit,
  isAsciiDigits,
  isAsciiLetter,
  isAsciiLowerLetter,
  isLowerHexDigit,
} from '../dist/final/node/index.mjs';

/**
 Characters each test is read against: both ends of each range, the
 characters just outside them, and letters from outside ASCII.
 */
const PROBES: readonly string[] = [
  'a',
  'z',
  'A',
  'Z',
  '0',
  '9',
  'f',
  'g',
  'F',
  '`',
  '{',
  '@',
  '[',
  '/',
  ':',
  'é',
  'ǎ',
  // U+0130 LATIN CAPITAL LETTER I WITH DOT ABOVE, which lowercases to an ASCII i and a mark.
  'İ',
  // U+212A KELVIN SIGN, drawn like an ASCII K and lowercasing to one.
  'K',
  '',
];

await describe({
  name: 'ascii letters',
  children: [
    it({
      name: 'ADMITS a to z in either case and nothing beside them, accented and look-alike letters included',
      fn: async () => {
        expect(PROBES.filter(function letter(character,): boolean {
          return isAsciiLetter({ character, },);
        },),).toEqual(['a', 'z', 'A', 'Z', 'f', 'g', 'F',],);
      },
    },),
    it({
      name: 'ADMITS 0 to 9 as digits, and letters and digits together as alphanumeric',
      fn: async () => {
        expect(PROBES.filter(function digit(character,): boolean {
          return isAsciiDigit({ character, },);
        },),).toEqual(['0', '9',],);
        expect(PROBES.filter(function alphanumeric(character,): boolean {
          return isAsciiAlphanumeric({ character, },);
        },),).toEqual(['a', 'z', 'A', 'Z', '0', '9', 'f', 'g', 'F',],);
      },
    },),
    it({
      name: 'ADMITS 0 to 9 and a to f as lower-case hexadecimal, and refuses upper case and g',
      fn: async () => {
        expect(PROBES.filter(function lowerHex(character,): boolean {
          return isLowerHexDigit({ character, },);
        },),).toEqual(['a', '0', '9', 'f',],);
      },
    },),
    it({
      name: 'ADMITS a to z alone as lower-case letters',
      fn: async () => {
        expect(PROBES.filter(function lower(character,): boolean {
          return isAsciiLowerLetter({ character, },);
        },),).toEqual(['a', 'z', 'f', 'g',],);
      },
    },),
    it({
      name: 'READS a text as digits only when it is non-empty and every character is 0 to 9',
      fn: async () => {
        expect([
          '611',
          '0',
          '',
          '10th',
          '٣',
          '1 2',
        ].map(function digits(text,): boolean {
          return isAsciiDigits({ text, },);
        },),).toEqual([true, true, false, false, false, false,],);
      },
    },),
  ],
},);
