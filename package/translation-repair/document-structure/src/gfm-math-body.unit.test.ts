/**
 Tests for the reading of plain Markdown in the flavour the package parses pages in.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseGfmMathBody, } from '../dist/final/node/index.mjs';

/**
 The types of the blocks of a text, in order.

 @param body - text to parse

 @returns Type of each child of the root

 @example
 ```ts
 const types = blockTypesOf({ body: 'a\n', },);
 ```
 */
function blockTypesOf({ body, }: { readonly body: string; },): readonly string[] {
  return parseGfmMathBody({ body, },).children.map(function typeOf({ type, },): string {
    return type;
  },);
}

await describe({
  name: parseGfmMathBody.name,
  children: [
    it({
      name: 'READS A FOOTNOTE DEFINITION AND A TABLE, which the plain reading has no construct for',
      fn: async () => {
        expect(blockTypesOf({ body: '猫[^1]\n\n[^1]: 注\n\n| a | b |\n| - | - |\n| c | d |\n', },),).toEqual([
          'paragraph',
          'footnoteDefinition',
          'table',
        ],);
      },
    },),
    it({
      name: 'READS DISPLAY MATH AS A MATH NODE AND A TAG ALONE ON A LINE AS RAW HTML',
      fn: async () => {
        expect(blockTypesOf({ body: '$$\na < b\n$$\n\n<q>\n', },),).toEqual([
          'math',
          'html',
        ],);
      },
    },),
  ],
},);
