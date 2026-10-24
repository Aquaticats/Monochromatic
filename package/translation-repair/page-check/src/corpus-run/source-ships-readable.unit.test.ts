/**
 Tests for a text as the parse reads it: its comments that close blanked and a byte order mark that opens it an ordinary
 character.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  asTheParseReadsIt,
  blanking,
} from '../../dist/final/node/index.mjs';

await describe({
  name: 'source ships readable',
  children: [
    describe({
      name: blanking.name,
      children: [
        it({
          name: 'REPLACES EVERY CHARACTER OF A STRETCH WITH A SPACE AND KEEPS ITS LINE ENDINGS, so the text keeps its '
            + 'length and its lines',
          fn: async () => {
            expect(blanking({
              text: 'a<!-- b\n-->c',
              ranges: [
                {
                  start: 1,
                  end: 11,
                },
              ],
            },),).toBe(`a${' '.repeat(6,)}\n   c`,);
          },
        },),

        it({
          name: 'BLANKS EACH OF SEVERAL STRETCHES AND COPIES THE TEXT BETWEEN THEM AND AFTER THEM',
          fn: async () => {
            expect(blanking({
              text: '猫<!-- 一 -->鱼<!-- 二 -->',
              ranges: [
                {
                  start: 1,
                  end: 11,
                },
                {
                  start: 12,
                  end: 22,
                },
              ],
            },),).toBe(`猫${' '.repeat(10,)}鱼${' '.repeat(10,)}`,);
          },
        },),

        it({
          name: 'RETURNS THE TEXT ITSELF FOR NO STRETCH',
          fn: async () => {
            expect(blanking({ text: '猫\n', ranges: [], },),).toBe('猫\n',);
          },
        },),
      ],
    },),

    describe({
      name: asTheParseReadsIt.name,
      children: [
        it({
          name: 'PUTS A LETTER IN PLACE OF A BYTE ORDER MARK THAT OPENS THE TEXT, and keeps the length',
          fn: async () => {
            expect(asTheParseReadsIt({ text: '\uFEFF- a', },),).toBe('x- a',);
          },
        },),

        it({
          name: 'LEAVES A TEXT WITH NO MARK AT ITS START, and a mark in the middle of it, as they are',
          fn: async () => {
            expect(asTheParseReadsIt({ text: '- a', },),).toBe('- a',);
            expect(asTheParseReadsIt({ text: 'a\uFEFF- b', },),).toBe('a\uFEFF- b',);
          },
        },),
      ],
    },),
  ],
},);
