/**
 Tests for the reading of the text after a top-level footnote on its own.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { parseMarkdownMathBody, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { rootWithFreshTails, } from '../../dist/final/node/index.mjs';

/**
 Reads the blocks of a text with the tails after its footnotes read on their own.

 @param body - text to read

 @returns Type, first line and first offset of each block

 @example
 ```ts
 const blocks = freshBlocks({ body: '[^1]: <b>\n<q>\n', },);
 ```
 */
function freshBlocks({ body, }: { readonly body: string; },): readonly string[] {
  return rootWithFreshTails({
    readable: body,
    root: parseMarkdownMathBody({ body, },),
  },).children.map(function describeBlock(block,): string {
    return `${block.type}@${String(block.position?.start.line,)}:${String(block.position?.start.offset,)}`;
  },);
}

await describe({
  name: rootWithFreshTails.name,
  children: [
    it({
      name: 'READS A TAG ALONE AFTER A FOOTNOTE THAT OPENS WITH A TAG ALONE AS RAW HTML, at the offsets of the whole text',
      fn: async () => {
        expect(freshBlocks({ body: '[^1]: <b>\n<q>\n`y`\n', },),).toEqual(['definition@1:0', 'html@2:10',],);
      },
    },),
    it({
      name: 'LEAVES A TEXT WHOSE FOOTNOTE ENDS WHERE THE PLAIN PARSE ENDS ITS BLOCK',
      fn: async () => {
        /**
         The plain parse of the text.
         */
        const plain = parseMarkdownMathBody({ body: '[^1]: x\n\ntail `y`\n', },).children.map(function describeBlock(block,): string {
          return `${block.type}@${String(block.position?.start.line,)}:${String(block.position?.start.offset,)}`;
        },);
        expect(freshBlocks({ body: '[^1]: x\n\ntail `y`\n', },),).toEqual(plain,);
      },
    },),
    it({
      name: 'LEAVES A FOOTNOTE IN A BLOCK QUOTE AND A FOOTNOTE WITH NO LINE AFTER IT',
      fn: async () => {
        expect(freshBlocks({ body: '> [^1]: <b>\n> <q>\n> `y`\n', },),).toEqual(['blockquote@1:0',],);
        expect(freshBlocks({ body: '[^1]: <b>', },),).toEqual(['definition@1:0',],);
      },
    },),
  ],
},);
