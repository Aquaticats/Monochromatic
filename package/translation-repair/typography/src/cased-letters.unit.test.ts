/**
 Tests the cased-letter reading the quote neighbours and the Canadian passes
 share (ledger B22): upper, lower and title case by general category in any
 script, the mathematical script letters handles are written in among them,
 and a Han character, a digit, a modifier letter, a combining mark, nothing
 or two characters never a cased letter.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isCapitalLetter,
  isCasedLetter,
  isSmallLetter,
} from '../dist/final/node/index.mjs';

/**
 Capitals in several scripts: Latin, Latin-1 accented, title case (U+01C5),
 Greek, and a bold script capital M (U+1D4DC) past the first plane.
 */
const CAPITALS = [
  'A',
  '\u{00C9}',
  '\u{01C5}',
  '\u{03A3}',
  '\u{1D4DC}',
] as const;

/**
 Small letters in several scripts: Latin, sharp s (U+00DF), Cyrillic, and a
 bold script small x (U+1D501) past the first plane.
 */
const SMALLS = [
  'a',
  '\u{00DF}',
  '\u{0436}',
  '\u{1D501}',
] as const;

/**
 What is no cased letter: a Han character, a digit, a modifier letter
 (U+02B0), a combining mark (U+0301), nothing, and two letters at once.
 */
const UNCASED = [
  '\u{732B}',
  '1',
  '\u{02B0}',
  '\u{0301}',
  '',
  'ab',
] as const;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: isCasedLetter.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS EVERY CAPITAL AND SMALL LETTER AS CASED, and nothing else',
          fn: async () => {
            for (const character of [
              ...CAPITALS,
              ...SMALLS,
            ]) {
              expect(isCasedLetter({ character, },),).toBe(true,);
            }
            for (const character of UNCASED) {
              expect(isCasedLetter({ character, },),).toBe(false,);
            }
          },
        },),
      ],
    },),

    describe({
      name: isCapitalLetter.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS UPPER AND TITLE CASE AS CAPITAL, and a small letter or anything uncased as not',
          fn: async () => {
            for (const character of CAPITALS) {
              expect(isCapitalLetter({ character, },),).toBe(true,);
            }
            for (const character of [
              ...SMALLS,
              ...UNCASED,
            ]) {
              expect(isCapitalLetter({ character, },),).toBe(false,);
            }
          },
        },),
      ],
    },),

    describe({
      name: isSmallLetter.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS LOWER CASE AS SMALL, and a capital, a title-case letter or anything uncased as not',
          fn: async () => {
            for (const character of SMALLS) {
              expect(isSmallLetter({ character, },),).toBe(true,);
            }
            for (const character of [
              ...CAPITALS,
              ...UNCASED,
            ]) {
              expect(isSmallLetter({ character, },),).toBe(false,);
            }
          },
        },),
      ],
    },),
  ],
},);
