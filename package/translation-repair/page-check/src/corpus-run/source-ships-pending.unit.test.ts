/**
 Tests for the state the spans reader keeps for each container it walks.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { parseMarkdownMathBody, } from '@monochromatic-dev/translation-repair-document-structure';

import {
  footnoteStateOf,
  rangeOf,
  stateAfter,
} from '../../dist/final/node/index.mjs';

/**
 The state of a container whose last block is part of a footnote definition that ends on the first line.
 */
const OPEN_ON_LINE_ONE = {
  open: true,
  endLine: 1,
  endsInLiteral: false,
};

/**
 The state of a container nothing has been read in.
 */
const NOTHING_READ = {
  open: false,
  endLine: 0,
  endsInLiteral: false,
};

/**
 The blocks of a text, parsed as the spans reader parses.

 @param source - text

 @returns Root and its blocks

 @example
 ```ts
 const { root, blocks, } = blocksOf({ source: '[^1]: 注\n', },);
 ```
 */
function blocksOf({ source, }: { readonly source: string; },) {
  /**
   The root of the parse.
   */
  const root = parseMarkdownMathBody({ body: source, },);
  return {
    root,
    blocks: root.children,
  };
}

await describe({
  name: 'the state the spans reader keeps',
  children: [
    describe({
      name: footnoteStateOf.name,
      children: [
        it({
          name: 'GIVES A CONTAINER NOTHING WAS READ IN THE STATE OF NO FOOTNOTE DEFINITION',
          fn: async () => {
            const { root, } = blocksOf({ source: '> a\n\nb\n', },);
            expect(footnoteStateOf({
              footnotes: new Map(),
              parent: root,
            },),).toEqual(NOTHING_READ,);
          },
        },),

        it({
          name: 'GIVES THE STATE RECORDED FOR A CONTAINER AND KEEPS THE STATE OF TWO CONTAINERS APART, the root and a '
            + 'block quote in it',
          fn: async () => {
            const {
              root,
              blocks,
            } = blocksOf({ source: '> a\n\nb\n', },);
            const [quote,] = blocks;
            if (quote === undefined)
              throw new Error('the text holds no block',);
            expect(footnoteStateOf({
              footnotes: new Map([[root, OPEN_ON_LINE_ONE,],],),
              parent: root,
            },),).toEqual(OPEN_ON_LINE_ONE,);
            expect(footnoteStateOf({
              footnotes: new Map([[root, OPEN_ON_LINE_ONE,],],),
              parent: quote,
            },),).toEqual(NOTHING_READ,);
          },
        },),
      ],
    },),

    describe({
      name: stateAfter.name,
      children: [
        it({
          name: 'MAKES A FOOTNOTE DEFINITION THE OPEN BLOCK, with the line it ends on',
          fn: async () => {
            const source = '[^1]: 注\n';
            const [definition,] = blocksOf({ source, },).blocks;
            if (definition === undefined)
              throw new Error('the text holds no block',);
            expect(stateAfter({
              before: NOTHING_READ,
              source,
              node: definition,
              range: rangeOf({ node: definition, },),
              continuesFootnote: false,
            },),).toEqual(OPEN_ON_LINE_ONE,);
          },
        },),

        it({
          name: 'MAKES A BLOCK INDENTED UNDER A FOOTNOTE DEFINITION THE OPEN BLOCK TOO, and a paragraph, a raw HTML '
            + 'block and indented code what they are',
          fn: async () => {
            const source = 'a\n\n<div>\n';
            const [paragraph, html,] = blocksOf({ source, },).blocks;
            if ((paragraph === undefined) || (html === undefined))
              throw new Error('the text holds no second block',);
            expect(stateAfter({
              before: NOTHING_READ,
              source,
              node: paragraph,
              range: rangeOf({ node: paragraph, },),
              continuesFootnote: false,
            },),).toEqual({
              open: false,
              endLine: 1,
              endsInLiteral: false,
            },);
            expect(stateAfter({
              before: NOTHING_READ,
              source,
              node: paragraph,
              range: rangeOf({ node: paragraph, },),
              continuesFootnote: true,
            },).open,).toBe(true,);
            expect(stateAfter({
              before: OPEN_ON_LINE_ONE,
              source,
              node: html,
              range: rangeOf({ node: html, },),
              continuesFootnote: false,
            },),).toEqual({
              open: false,
              endLine: 3,
              endsInLiteral: true,
            },);
          },
        },),
      ],
    },),
  ],
},);
