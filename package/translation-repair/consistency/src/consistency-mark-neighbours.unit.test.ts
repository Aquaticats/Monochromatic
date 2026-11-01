/**
 Tests for what stands directly beside a single mark: the eight characters that count as a single mark beside a single mark,
 and the characters that part a mark from a word (white space, the edge of the text, punctuation that is no single mark).

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isNeighbouringSingleMark,
  partsMarkFromWord,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: isNeighbouringSingleMark.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS EACH OF THE EIGHT SINGLE MARKS AS ONE: the three apostrophe-like characters, the opening curly single mark, '
            + 'the low and the reversed single mark and both single angle quotation marks',
          fn: async () => {
            expect([
              '\'',
              '‘',
              '’',
              '\u{02BC}',
              '\u{201A}',
              '\u{201B}',
              '‹',
              '›',
            ].map(function asked(character,): boolean {
              return isNeighbouringSingleMark({ character, },);
            },),).toEqual([true, true, true, true, true, true, true, true,],);
          },
        },),
        it({
          name: 'DOES NOT COUNT A DOUBLE MARK, A BRACKET, A LETTER, A PRIME OR THE EDGE OF A TEXT',
          fn: async () => {
            expect([
              '"',
              '”',
              ')',
              'a',
              '′',
              '`',
              '',
            ].map(function asked(character,): boolean {
              return isNeighbouringSingleMark({ character, },);
            },),).toEqual([false, false, false, false, false, false, false,],);
          },
        },),
      ],
    },),
    describe({
      name: partsMarkFromWord.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PARTS A MARK FROM A WORD BY THE EDGE OF THE TEXT, WHITE SPACE AND PUNCTUATION THAT IS NO SINGLE MARK',
          fn: async () => {
            expect([
              '',
              ' ',
              '\n',
              '\t',
              '\u{00A0}',
              '.',
              ',',
              ')',
              '"',
              '”',
              '*',
              '」',
            ].map(function asked(character,): boolean {
              return partsMarkFromWord({ character, },);
            },),).toEqual([true, true, true, true, true, true, true, true, true, true, true, true,],);
          },
        },),
        it({
          name: 'DOES NOT PART A MARK FROM A WORD BY A SINGLE MARK, A LETTER OR DIGIT OF ANY SCRIPT, A SYMBOL OR A PICTOGRAPH',
          fn: async () => {
            expect([
              '\'',
              '’',
              '‹',
              'a',
              '5',
              '猫',
              'ω',
              '™',
              '²',
              '+',
              '🐱',
              '\u{02BC}',
            ].map(function asked(character,): boolean {
              return partsMarkFromWord({ character, },);
            },),).toEqual([false, false, false, false, false, false, false, false, false, false, false, false,],);
          },
        },),
      ],
    },),
  ],
},);
