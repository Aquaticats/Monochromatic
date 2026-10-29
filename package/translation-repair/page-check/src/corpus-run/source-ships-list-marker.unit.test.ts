/**
 Tests for the marker of a list item that follows indented code: which blocks are indented code, and the edit that writes
 a marker with a backslash.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import { parseMarkdownMathBody, } from '@monochromatic-dev/translation-repair-document-structure';

import {
  isIndentedCodeBlock,
  markerEscapeEdits,
} from '../../dist/final/node/index.mjs';

await describe({
  name: 'the marker of a list item after indented code',
  children: [
    describe({
      name: isIndentedCodeBlock.name,
      children: [
        it({
          name: 'IS TRUE OF CODE NO FENCE OPENS AND FALSE OF A FENCE AND OF OTHER BLOCKS',
          fn: async () => {
            const source = '    猫\n\n```\nf\n```\n\n鱼\n';
            const [indented, fenced, paragraph,] = parseMarkdownMathBody({ body: source, },).children;
            expect(isIndentedCodeBlock({ source, node: nonNullishOrThrow(indented,), },),).toBe(true,);
            expect(isIndentedCodeBlock({ source, node: nonNullishOrThrow(fenced,), },),).toBe(false,);
            expect(isIndentedCodeBlock({ source, node: nonNullishOrThrow(paragraph,), },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: markerEscapeEdits.name,
      children: [
        it({
          name: 'WRITES THE CHARACTER THAT MAKES THE MARKER WITH A BACKSLASH',
          fn: async () => {
            expect(markerEscapeEdits({ source: '2.\n', start: 0, },),).toEqual([{ start: 1, end: 2, replacement: String.raw`\.`, },],);
            expect(markerEscapeEdits({ source: '*\n', start: 0, },),).toEqual([{ start: 0, end: 1, replacement: String.raw`\*`, },],);
          },
        },),
        it({
          name: 'WRITES NOTHING FOR A LINE THAT STARTS WITH NO MARKER',
          fn: async () => {
            expect(markerEscapeEdits({ source: '猫\n', start: 0, },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
