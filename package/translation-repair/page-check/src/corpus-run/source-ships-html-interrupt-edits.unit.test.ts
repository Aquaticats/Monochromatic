/**
 Tests the edits that put a blank line before a block of raw HTML that cuts a paragraph, and where those blocks start
 (`htmlInterruptionEdits`).

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { parseMarkdownMathBody, } from '@monochromatic-dev/translation-repair-document-structure';

import { htmlInterruptionEdits, } from '../../dist/final/node/index.mjs';

/**
 What the function answers for a text.

 @param readable - text

 @returns Edits and where the blocks cut off start

 @example
 ```ts
 const cut = cutOf({ readable: 'a `x\n<div>\ny', },);
 ```
 */
function cutOf({ readable, }: { readonly readable: string; },): ReturnType<typeof htmlInterruptionEdits> {
  return htmlInterruptionEdits({
    readable,
    root: parseMarkdownMathBody({ body: readable, },),
  },);
}

await describe({
  name: htmlInterruptionEdits.name,
  children: [
    it({
      name: 'DOUBLES THE LINE ENDING BEFORE A BLOCK OF RAW HTML THAT STARTS RIGHT AFTER A PARAGRAPH, and says where the block starts',
      fn: async () => {
        expect(cutOf({ readable: 'a `x\n<div>\ny`\n', },),).toEqual({
          edits: [{ start: 4, end: 5, replacement: '\n\n', },],
          starts: [5,],
        },);
      },
    },),
    it({
      name: 'WRITES THE LINE ENDING OF THE TEXT AND THE MARKS OF THE CONTAINER ON THE BLANK LINE',
      fn: async () => {
        expect(cutOf({ readable: 'a `x\r\n<div>\r\n', },).edits,).toEqual([{ start: 4, end: 6, replacement: '\r\n\r\n', },],);
        expect(cutOf({ readable: '> a `x\n> <div>\n', },).edits,).toEqual([{ start: 6, end: 7, replacement: '\n>\n', },],);
        expect(cutOf({ readable: '- a `x\n  <div>\n', },).edits,).toEqual([{ start: 6, end: 7, replacement: '\n\n', },],);
      },
    },),
    it({
      name: 'LEAVES A BLOCK ALONE THAT NO PARAGRAPH STANDS RIGHT BEFORE, a blank line or another kind of block between',
      fn: async () => {
        const none = { edits: [], starts: [], };
        expect(cutOf({ readable: 'a `x\n\n<div>\n', },),).toEqual(none,);
        expect(cutOf({ readable: '# a\n<div>\n', },),).toEqual(none,);
        expect(cutOf({ readable: '<div>\n', },),).toEqual(none,);
        expect(cutOf({ readable: 'a `x\n<span>\n', },),).toEqual(none,);
      },
    },),
    it({
      name: 'LEAVES A PARAGRAPH ALONE THAT A FOOTNOTE DEFINITION HOLDS, where the page goes on over a line of raw HTML',
      fn: async () => {
        expect(cutOf({ readable: '[^1]: a `x\n<div>\ny`\n', },),).toEqual({ edits: [], starts: [], },);
      },
    },),
  ],
},);
