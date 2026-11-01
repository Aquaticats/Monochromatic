/**
 Tests for the state the spans reader keeps for each container it walks.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  footnoteDefinitionRangesOf,
  parseMarkdownMathBody,
} from '@monochromatic-dev/translation-repair-document-structure';

import {
  footnoteStateOf,
  rangeOf,
  stateAfter,
  takeContinuation,
  takeFootnote,
  topLevelPending,
} from '../../dist/final/node/index.mjs';

/**
 The state of a container whose last block is part of a footnote definition that ends on the first line.
 */
const OPEN_ON_LINE_ONE = {
  open: true,
  endLine: 1,
  endsInLiteral: false,
  prefix: '',
  opensRawHtml: false,
  opensFootnote: true,
  goneOnLazily: false,
};

/**
 The state of a container nothing has been read in.
 */
const NOTHING_READ = {
  open: false,
  endLine: 0,
  endsInLiteral: false,
  prefix: '',
  opensRawHtml: false,
  opensFootnote: false,
  goneOnLazily: false,
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
              isRootChild: true,
            },),).toEqual(OPEN_ON_LINE_ONE,);
          },
        },),

        it({
          name: 'RECORDS WHAT A CONTAINER PUTS BEFORE THE DEFINITION AND WHETHER IT OPENS WITH RAW HTML, and keeps the footnote '
            + 'open for a lazy line right after it',
          fn: async () => {
            const source = '> [^1]: <b>\n> abc\n';
            const quote = nonNullishOrThrow(parseMarkdownMathBody({ body: source, },).children[0],);
            if (!('children' in quote))
              throw new Error('the text is no block quote',);
            const [definition, lazy,] = quote.children;
            if ((definition === undefined) || (lazy === undefined))
              throw new Error('the quote holds no two blocks',);
            /**
             The state after the definition, and after the lazy line that goes on in its paragraph.
             */
            const opened = stateAfter({
              before: NOTHING_READ,
              source,
              node: definition,
              range: rangeOf({ node: definition, },),
              continuesFootnote: false,
              isRootChild: false,
            },);
            expect(opened,).toEqual({
              open: true,
              endLine: 1,
              endsInLiteral: false,
              prefix: '> ',
              opensRawHtml: true,
              opensFootnote: true,
              goneOnLazily: false,
            },);
            const goneOn = stateAfter({
              before: opened,
              source,
              node: lazy,
              range: rangeOf({ node: lazy, },),
              continuesFootnote: false,
              isRootChild: false,
            },);
            expect(goneOn.open,).toBe(true,);
            expect(goneOn.goneOnLazily,).toBe(true,);
            expect(opened.goneOnLazily,).toBe(false,);
            expect(goneOn.prefix,).toBe('> ',);
            expect(goneOn.opensFootnote,).toBe(false,);
            expect(stateAfter({
              before: opened,
              source,
              node: lazy,
              range: rangeOf({ node: lazy, },),
              continuesFootnote: false,
              isRootChild: true,
            },).open,).toBe(false,);
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
              isRootChild: true,
            },),).toEqual({
              open: false,
              endLine: 1,
              endsInLiteral: false,
              prefix: '',
              opensRawHtml: false,
              opensFootnote: false,
              goneOnLazily: false,
            },);
            expect(stateAfter({
              before: NOTHING_READ,
              source,
              node: paragraph,
              range: rangeOf({ node: paragraph, },),
              continuesFootnote: true,
              isRootChild: true,
            },).open,).toBe(true,);
            expect(stateAfter({
              before: OPEN_ON_LINE_ONE,
              source,
              node: html,
              range: rangeOf({ node: html, },),
              continuesFootnote: false,
              isRootChild: true,
            },),).toEqual({
              open: false,
              endLine: 3,
              endsInLiteral: true,
              prefix: '',
              opensRawHtml: false,
              opensFootnote: false,
              goneOnLazily: false,
            },);
          },
        },),
      ],
    },),

    describe({
      name: topLevelPending.name,
      children: [
        it({
          name: 'LISTS THE BLOCKS OF A TEXT WITH THE LAST FIRST, each standing at the top of the root',
          fn: async () => {
            const {
              root,
              blocks,
            } = blocksOf({ source: '> a\n\nb\n', },);
            const listed = topLevelPending({ root, },);
            expect(listed.map(function nodeOf({ node, },) {
              return node;
            },),).toEqual(blocks.toReversed(),);
            expect(listed.every(function atTop({
              isRootChild,
              insideLink,
              parentType,
              parent,
            },) {
              return isRootChild && (!insideLink) && (parentType === 'root') && (parent === root);
            },),).toBe(true,);
          },
        },),

        it({
          name: 'LISTS NOTHING FOR A NODE THAT HOLDS NO CHILDREN',
          fn: async () => {
            const [, paragraph,] = blocksOf({ source: '> a\n\nb\n', },).blocks;
            const leaf = (paragraph !== undefined) && ('children' in paragraph) ? paragraph.children[0] : undefined;
            if (leaf === undefined)
              throw new Error('the text holds no leaf',);
            expect(topLevelPending({ root: leaf, },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: takeContinuation.name,
      children: [
        it({
          name: 'TAKES THE PARAGRAPH ON THE LINE RIGHT AFTER THE DEFINITION OFF THE NODES STILL TO READ and hands it '
            + 'back',
          fn: async () => {
            const {
              root,
              blocks,
            } = blocksOf({ source: '[^1]: https://example.net/a\n注\n', },);
            const [definition, paragraph,] = blocks;
            if ((definition === undefined) || (paragraph === undefined))
              throw new Error('the text holds no second block',);
            /**
             The nodes still to read, with the paragraph next.
             */
            const pending = [
              {
                node: paragraph,
                isRootChild: true,
                insideLink: false,
                parentType: 'root',
                parent: root,
              },
            ];
            expect(takeContinuation({
              pending,
              definition,
            },),).toBe(paragraph,);
            expect(pending,).toEqual([],);
          },
        },),

        it({
          name: 'HANDS THE DEFINITION BACK AND LEAVES THE NODES ALONE when the next block is parted from it by a blank '
            + 'line, and when nothing is left to read',
          fn: async () => {
            const {
              root,
              blocks,
            } = blocksOf({ source: '[^1]: https://example.net/a\n\n注\n', },);
            const [definition, paragraph,] = blocks;
            if ((definition === undefined) || (paragraph === undefined))
              throw new Error('the text holds no second block',);
            /**
             The nodes still to read, with a paragraph a blank line away next.
             */
            const pending = [
              {
                node: paragraph,
                isRootChild: true,
                insideLink: false,
                parentType: 'root',
                parent: root,
              },
            ];
            expect(takeContinuation({
              pending,
              definition,
            },),).toBe(definition,);
            expect(pending.length,).toBe(1,);
            expect(takeContinuation({
              pending: [],
              definition,
            },),).toBe(definition,);
          },
        },),
      ],
    },),

    describe({
      name: takeFootnote.name,
      children: [
        it({
          name: 'TAKES THE BLOCKS THE STRICT GRAMMAR READS IN THE FOOTNOTE OFF THE NODES STILL TO READ, a heading the '
            + 'plain reading made of its lines and a definition line among them, and closes the top',
          fn: async () => {
            /**
             A footnote whose paragraph holds a line of equals signs and a definition line, then a paragraph of its own.
             */
            const source = '[^1]: x\ny\n===\n[r]: u\n\n注\n';
            const {
              root,
              blocks,
            } = blocksOf({ source, },);
            const [definition, ...rest] = blocks;
            if (definition === undefined)
              throw new Error('the text holds no block',);
            /**
             The nodes still to read, the next one on top.
             */
            const pending = topLevelPending({ root, },).slice(
              0,
              rest.length,
            );
            /**
             The block that stands apart from the footnote.
             */
            const apart = pending.at(0,);
            const reach = takeFootnote({
              pending,
              footnotes: footnoteDefinitionRangesOf({ body: source, },),
              source,
              node: definition,
              range: rangeOf({ node: definition, },),
              state: OPEN_ON_LINE_ONE,
            },);
            expect(reach.end,).toBe(source.indexOf('\n\n',),);
            expect(pending,).toEqual([apart,],);
            expect(reach.state,).toEqual({
              open: false,
              endLine: 4,
              endsInLiteral: false,
              prefix: '',
              opensRawHtml: false,
              opensFootnote: false,
              goneOnLazily: false,
            },);
          },
        },),

        it({
          name: 'HANDS BACK THE END OF THE BLOCK, the state given and the paragraph that goes on after a definition '
            + 'when no footnote starts at the block',
          fn: async () => {
            /**
             A definition with a paragraph on the next line.
             */
            const source = '[^1]: https://example.net/a\n注\n';
            const {
              root,
              blocks,
            } = blocksOf({ source, },);
            const [definition, paragraph,] = blocks;
            if ((definition === undefined) || (paragraph === undefined))
              throw new Error('the text holds no second block',);
            const pending = [
              {
                node: paragraph,
                isRootChild: true,
                insideLink: false,
                parentType: 'root',
                parent: root,
              },
            ];
            const reach = takeFootnote({
              pending,
              footnotes: [],
              source,
              node: definition,
              range: rangeOf({ node: definition, },),
              state: OPEN_ON_LINE_ONE,
            },);
            expect(reach.end,).toBe(rangeOf({ node: paragraph, },).end,);
            expect(reach.state,).toEqual(OPEN_ON_LINE_ONE,);
            expect(pending,).toEqual([],);
          },
        },),

        it({
          name: 'TAKES A NODE THAT IS NOT A BLOCK OF THE ROOT when it starts inside the footnote, and leaves one that '
            + 'starts where the footnote has ended',
          fn: async () => {
            const source = '[^1]: x\ny\n===\n\n注\n';
            const {
              root,
              blocks,
            } = blocksOf({ source, },);
            const [definition, heading, paragraph,] = blocks;
            if ((definition === undefined) || (heading === undefined) || (paragraph === undefined))
              throw new Error('the text holds fewer than three blocks',);
            const pending = [
              {
                node: paragraph,
                isRootChild: true,
                insideLink: false,
                parentType: 'root',
                parent: root,
              },
              {
                node: heading,
                isRootChild: false,
                insideLink: false,
                parentType: 'listItem',
                parent: root,
              },
            ];
            takeFootnote({
              pending,
              footnotes: footnoteDefinitionRangesOf({ body: source, },),
              source,
              node: definition,
              range: rangeOf({ node: definition, },),
              state: NOTHING_READ,
            },);
            expect(pending.length,).toBe(1,);
          },
        },),
      ],
    },),
  ],
},);
