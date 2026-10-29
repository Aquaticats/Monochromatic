/**
 Tests for telling a footnote definition that opens with a block of raw HTML that goes on over the next line.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { footnoteOpensRawHtml, } from '../../dist/final/node/index.mjs';

/**
 Whether the definition a text starts with, after an optional block quote mark, opens with raw HTML.

 @param source - text

 @param start - index of the definition's label

 @returns Answer for the definition held to the end of the text

 @example
 ```ts
 const html = opens({ source: '[^1]: <b>\n', start: 0, },);
 ```
 */
function opens({ source, start, }: { readonly source: string; readonly start: number; },): boolean {
  return footnoteOpensRawHtml({
    source,
    range: {
      start,
      end: source.length,
    },
  },);
}

await describe({
  name: footnoteOpensRawHtml.name,
  children: [
    it({
      name: 'SAYS A TAG ALONE, AN ELEMENT OF THE BLOCK LIST AND AN OPENER OF A COMMENT THAT NEVER CLOSES START A BLOCK THAT GOES ON',
      fn: async () => {
        expect(opens({ source: '[^1]: <b>\n', start: 0, },),).toBe(true,);
        expect(opens({ source: '[^1]: <div>\n', start: 0, },),).toBe(true,);
        expect(opens({ source: '[^1]:   <pre>\n', start: 0, },),).toBe(true,);
        expect(opens({ source: '[^1]: <!-- never\n', start: 0, },),).toBe(true,);
      },
    },),
    it({
      name: 'SAYS A TAG WITH TEXT AFTER IT, A COMMENT THAT CLOSES ON ITS LINE AND PLAIN TEXT START NONE',
      fn: async () => {
        expect(opens({ source: '[^1]: <b> x\n', start: 0, },),).toBe(false,);
        expect(opens({ source: '[^1]: <!-- c -->\n', start: 0, },),).toBe(false,);
        expect(opens({ source: '[^1]: <?x?>\n', start: 0, },),).toBe(false,);
        expect(opens({ source: '[^1]: x\n', start: 0, },),).toBe(false,);
      },
    },),
    it({
      name: 'SAYS A DEFINITION WITH NO BODY OR WITH A LESS-THAN SIGN THAT STARTS NO TAG OPENS NO BLOCK',
      fn: async () => {
        expect(opens({ source: '[^1]:\n', start: 0, },),).toBe(false,);
        expect(opens({ source: '[^1]: < b>\n', start: 0, },),).toBe(false,);
        expect(opens({ source: '[^1]: a <b>\n', start: 0, },),).toBe(false,);
      },
    },),
    it({
      name: 'READS A DEFINITION IN A BLOCK QUOTE OR A LIST ITEM FROM ITS LABEL, and one on the last line of the text',
      fn: async () => {
        expect(opens({ source: '> [^1]: <b>\n>     c\n', start: 2, },),).toBe(true,);
        expect(opens({ source: '- [^1]: <b>\n      c\n', start: 2, },),).toBe(true,);
        expect(opens({ source: '> [^1]: <b>', start: 2, },),).toBe(true,);
        expect(opens({ source: '> [^1]: x\n', start: 2, },),).toBe(false,);
      },
    },),
  ],
},);
