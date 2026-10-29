/**
 Tests for the reading of a block of raw HTML or of code: the lines of an indented block are read as they stand, and a fence
 is copied.

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

import { literalBlockReading, } from '../../dist/final/node/index.mjs';

await describe({
  name: literalBlockReading.name,
  children: [
    it({
      name: 'READS THE LINES OF INDENTED CODE AS THEY STAND, without the indentation',
      fn: async () => {
        const source = '    <b>\n    {a}\n';
        const [node,] = parseMarkdownMathBody({ body: source, },).children;
        const { literal, edits, } = literalBlockReading({
          source,
          readable: source,
          node: nonNullishOrThrow(node,),
          range: { start: 0, end: source.length - 1, },
          isRootChild: true,
        },);
        expect(literal,).toEqual([{ start: 4, end: 7, }, { start: 12, end: 15, },],);
        expect(edits,).toEqual([],);
      },
    },),
    it({
      name: 'READS NO LINE OF A FENCE AS LITERAL TEXT, it is copied',
      fn: async () => {
        const source = '```\nf\n```\n';
        const [node,] = parseMarkdownMathBody({ body: source, },).children;
        const { literal, edits, } = literalBlockReading({
          source,
          readable: source,
          node: nonNullishOrThrow(node,),
          range: { start: 0, end: source.length - 1, },
          isRootChild: true,
        },);
        expect(literal,).toEqual([],);
        expect(edits,).toEqual([],);
      },
    },),
  ],
},);
