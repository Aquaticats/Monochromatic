/**
 Tests for the units a span edge is cut by: a backslash and the ASCII punctuation it escapes, a character reference the Markdown
 parser decodes, and every other character alone, each with the character a reader sees for it.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { edgeUnitsOf, } from '../dist/final/node/index.mjs';

/**
 The units of a text read as written.

 @param text - text

 @returns Units as `from-to:shown`, in order

 @example
 ```ts
 const units = unitsOf({ text: 'a\\.', },); // ['0-1:a', '1-3:.']
 ```
 */
function unitsOf({ text, }: { readonly text: string; },): readonly string[] {
  return edgeUnitsOf({ points: Array.from(text,), },)
    .map(function described({
      from,
      to,
      shown,
    },): string {
      return `${String(from,)}-${String(to,)}:${shown}`;
    },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: edgeUnitsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'MAKES A BACKSLASH AND THE ASCII PUNCTUATION IT ESCAPES ONE UNIT that shows the escaped character, and any other '
            + 'backslash one unit alone',
          fn: async () => {
            expect({
              stop: unitsOf({ text: String.raw`a\.`, },),
              bracket: unitsOf({ text: String.raw`\)`, },),
              escapedBackslash: unitsOf({ text: String.raw`a\\.`, },),
              threeBackslashes: unitsOf({ text: String.raw`\\\*`, },),
              beforeLetter: unitsOf({ text: String.raw`\a`, },),
              atEnd: unitsOf({ text: 'a\\', },),
              beforeLineFeed: unitsOf({ text: '\\\n', },),
            },).toEqual({
              stop: ['0-1:a', '1-3:.',],
              bracket: ['0-2:)',],
              escapedBackslash: ['0-1:a', '1-3:\\', '3-4:.',],
              threeBackslashes: ['0-2:\\', '2-4:*',],
              beforeLetter: ['0-1:\\', '1-2:a',],
              atEnd: ['0-1:a', '1-2:\\',],
              beforeLineFeed: ['0-1:\\', '1-2:\n',],
            },);
          },
        },),
        it({
          name: 'MAKES A CHARACTER REFERENCE THE PARSER DECODES ONE UNIT that shows what the parser decodes it to, and an ampersand '
            + 'that a backslash escapes starts none',
          fn: async () => {
            expect({
              named: unitsOf({ text: '&amp;', },),
              numeric: unitsOf({ text: '&#39;', },),
              hexadecimal: unitsOf({ text: '&#x2019;', },),
              space: unitsOf({ text: 'a&nbsp;b', },),
              sevenDigits: unitsOf({ text: '&#0000039;', },),
              beyondFirstPlane: unitsOf({ text: '&#128049;', },),
              afterEscapedBackslash: unitsOf({ text: String.raw`\\&amp;`, },),
              escapedAmpersand: unitsOf({ text: String.raw`\&amp;`, },),
            },).toEqual({
              named: ['0-5:&',],
              numeric: ['0-5:\'',],
              hexadecimal: ['0-8:’',],
              space: ['0-1:a', '1-7:\u{00A0}', '7-8:b',],
              sevenDigits: ['0-10:\'',],
              beyondFirstPlane: ['0-9:🐱',],
              afterEscapedBackslash: ['0-2:\\', '2-7:&',],
              escapedAmpersand: ['0-2:&', '2-3:a', '3-4:m', '4-5:p', '5-6:;',],
            },);
          },
        },),
        it({
          name: 'MAKES NO UNIT OF A WORD THAT MERELY LOOKS LIKE A REFERENCE, one the parser leaves as text: a name it does not know, a '
            + 'single letter, a number of eleven digits, a missing semicolon',
          fn: async () => {
            expect({
              questionAnswer: unitsOf({ text: 'Q&A;', },),
              unknownName: unitsOf({ text: '&zzzz;', },),
              elevenDigits: unitsOf({ text: '&#00000000039;', },),
              noSemicolon: unitsOf({ text: '&amp', },),
              emptyName: unitsOf({ text: '&;', },),
              wrongCase: unitsOf({ text: '&Amp;', },),
            },).toEqual({
              questionAnswer: ['0-1:Q', '1-2:&', '2-3:A', '3-4:;',],
              unknownName: ['0-1:&', '1-2:z', '2-3:z', '3-4:z', '4-5:z', '5-6:;',],
              elevenDigits: Array.from('&#00000000039;',)
                .map(function single(character, at,): string {
                  return `${String(at,)}-${String(at + 1,)}:${character}`;
                },),
              noSemicolon: ['0-1:&', '1-2:a', '2-3:m', '3-4:p',],
              emptyName: ['0-1:&', '1-2:;',],
              wrongCase: ['0-1:&', '1-2:A', '2-3:m', '3-4:p', '4-5:;',],
            },);
          },
        },),
        it({
          name: 'READS AN ESCAPE FROM THE CHARACTERS AS WRITTEN: a backslash before a curly mark that the fold turned straight escapes '
            + 'nothing, and one before a straight mark escapes it',
          fn: async () => {
            expect({
              curly: edgeUnitsOf({ points: Array.from(String.raw`a\'`,), originals: Array.from(String.raw`a\’`,), },)
                .map(function shownOf({ shown, },): string {
                  return shown;
                },),
              straight: edgeUnitsOf({ points: Array.from(String.raw`a\'`,), originals: Array.from(String.raw`a\'`,), },)
                .map(function shownOf({ shown, },): string {
                  return shown;
                },),
            },).toEqual({
              curly: ['a', '\\', '\'',],
              straight: ['a', '\'',],
            },);
          },
        },),
        it({
          name: 'COUNTS CHARACTERS BEYOND THE FIRST PLANE AS ONE and gives a reference that shows two code points as one unit',
          fn: async () => {
            expect({
              cat: unitsOf({ text: String.raw`🐱\.`, },),
              twoCodePoints: unitsOf({ text: '&NotEqualTilde;', },),
            },).toEqual({
              cat: ['0-1:🐱', '1-3:.',],
              twoCodePoints: ['0-15:\u{2242}\u{0338}',],
            },);
          },
        },),
      ],
    },),
  ],
},);
