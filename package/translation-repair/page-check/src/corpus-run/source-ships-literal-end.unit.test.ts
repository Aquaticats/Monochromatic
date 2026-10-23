/**
 Tests for the blocks that end in a raw HTML block or in indented code, which the strict grammar reads as a paragraph.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { parseMarkdownMathBody, } from '@monochromatic-dev/translation-repair-document-structure';

import { endsInLiteral, } from '../../dist/final/node/index.mjs';

/**
 Whether the first block of a text ends in a raw HTML block or in indented code.

 @param source - text

 @returns Answer

 @example
 ```ts
 const ends = firstEndsInLiteral({ source: '<div>\n', },);
 ```
 */
function firstEndsInLiteral({ source, }: { readonly source: string; },): boolean {
  const [node,] = parseMarkdownMathBody({ body: source, },).children;
  if (node === undefined)
    throw new Error('the text holds no block',);
  return endsInLiteral({
    source,
    node,
  },);
}

await describe({
  name: endsInLiteral.name,
  children: [
    it({
      name: 'ANSWERS YES FOR A RAW HTML BLOCK AND FOR INDENTED CODE, at the top',
      fn: async () => {
        expect(firstEndsInLiteral({ source: '<div>\n', },),).toBe(true,);
        expect(firstEndsInLiteral({ source: '    code\n', },),).toBe(true,);
      },
    },),

    it({
      name: 'ANSWERS YES FOR A BLOCK QUOTE OR A LIST WHOSE LAST BLOCK IS ONE, however deep',
      fn: async () => {
        expect(firstEndsInLiteral({ source: '> <div>\n', },),).toBe(true,);
        expect(firstEndsInLiteral({ source: '* </details> a\n', },),).toBe(true,);
        expect(firstEndsInLiteral({ source: '- a\n\n  - b\n\n        code\n', },),).toBe(true,);
      },
    },),

    it({
      name: 'ANSWERS NO FOR A PARAGRAPH, A FENCE, AND A CONTAINER WHOSE LAST BLOCK IS ONE OF THOSE',
      fn: async () => {
        expect(firstEndsInLiteral({ source: 'a\n', },),).toBe(false,);
        expect(firstEndsInLiteral({ source: '```\ncode\n```\n', },),).toBe(false,);
        expect(firstEndsInLiteral({ source: '> <div>\n>\n> a\n', },),).toBe(false,);
        expect(firstEndsInLiteral({ source: '- <div>\n\n  a\n', },),).toBe(false,);
      },
    },),
  ],
},);
