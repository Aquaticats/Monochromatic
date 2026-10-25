/**
 Tests for the reading of the blocks under a footnote definition in a block quote or a list item.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { parseMarkdownMathBody, } from '@monochromatic-dev/translation-repair-document-structure';

import {
  containerCharactersAt,
  continuesAsParagraph,
  continuesTheFootnote,
  rangeOf,
  withoutContainerAndIndentation,
} from '../../dist/final/node/index.mjs';

/**
 The state of a container whose last block is the footnote definition of a quote that puts two characters before it.
 */
const OPEN_IN_QUOTE = {
  open: true,
  endLine: 1,
  endsInLiteral: false,
  prefixLength: 2,
  opensRawHtml: false,
  opensFootnote: true,
};

/**
 The block of a block quote at an index among its children, with the text it was parsed from.

 @param source - text that is a block quote
 @param at - index of the child

 @returns The child and where it stands

 @example
 ```ts
 const { node, range, } = childOfQuote({ source: '> a\n', at: 0, },);
 ```
 */
function childOfQuote(
  {
    source,
    at,
  }: {
    readonly source: string;
    readonly at: number;
  },
): {
  readonly node: Parameters<typeof continuesTheFootnote>[0]['node'];
  readonly range: ReturnType<typeof rangeOf>;
} {
  /**
   The block quote.
   */
  const quote = nonNullishOrThrow(parseMarkdownMathBody({ body: source, },).children[0],);
  if (!('children' in quote))
    throw new Error('the text is no block quote',);
  /**
   The child.
   */
  const node = nonNullishOrThrow(quote.children[at],);
  return {
    node,
    range: rangeOf({ node, },),
  };
}

await describe({
  name: 'the blocks under a footnote definition in a container',
  children: [
    describe({
      name: containerCharactersAt.name,
      children: [
        it({
          name: 'COUNTS THE MARKS AND WHITE SPACE A LINE STARTS WITH, up to as many as the container puts before its content',
          fn: async () => {
            expect(containerCharactersAt({ source: '>     a', at: 0, most: 2, },),).toBe(2,);
            expect(containerCharactersAt({ source: '>     a', at: 0, most: 9, },),).toBe(6,);
            expect(containerCharactersAt({ source: '\t> a', at: 0, most: 4, },),).toBe(3,);
          },
        },),
        it({
          name: 'COUNTS NONE FOR A LINE THAT STARTS WITH NO MARK OR WHITE SPACE, or when the container puts none',
          fn: async () => {
            expect(containerCharactersAt({ source: 'a  ', at: 0, most: 2, },),).toBe(0,);
            expect(containerCharactersAt({ source: '>  a', at: 0, most: 0, },),).toBe(0,);
            expect(containerCharactersAt({ source: 'x\n> a', at: 2, most: 2, },),).toBe(2,);
          },
        },),
      ],
    },),
    describe({
      name: withoutContainerAndIndentation.name,
      children: [
        it({
          name: 'KEEPS WHAT IS LEFT OF EACH LINE AFTER THE MARKS OF THE CONTAINER AND THE FOUR COLUMNS OF THE FOOTNOTE',
          fn: async () => {
            /**
             A block quote whose footnote holds a fence, the fence starting after the mark and the space of its first line.
             */
            const source = '> [^1]: x\n>\n>     ```\n>     a\n>     ```\n';
            /**
             The characters kept.
             */
            const kept = withoutContainerAndIndentation({
              source,
              range: { start: 14, end: source.length - 1, },
              prefixLength: 2,
            },)
              .map(function characterAt(at,): string {
                return source.charAt(at,);
              },)
              .join('',);
            expect(kept,).toBe('```\na\n```',);
          },
        },),
        it({
          name: 'GIVES UP A TAB AS THE INDENTATION, and the hanging indent of a list item as the marks of the container',
          fn: async () => {
            /**
             A list item whose footnote holds a paragraph of two lines, the second indented by a tab.
             */
            const source = '- [^1]: x\n\n      a\n  \tb\n';
            const kept = withoutContainerAndIndentation({
              source,
              range: { start: 13, end: source.length - 1, },
              prefixLength: 2,
            },)
              .map(function characterAt(at,): string {
                return source.charAt(at,);
              },)
              .join('',);
            expect(kept,).toBe('a\nb',);
          },
        },),
      ],
    },),
    describe({
      name: continuesAsParagraph.name,
      children: [
        it({
          name: 'IS TRUE FOR A PARAGRAPH INDENTED FOUR COLUMNS BEYOND THE MARKS, with spaces or with a tab',
          fn: async () => {
            for (const source of ['> [^1]: x\n>     p\n', '> [^1]: x\n>\t\tp\n',]) {
              const { node, range, } = childOfQuote({ source, at: 1, },);
              expect(continuesAsParagraph({ source, node, range, prefixLength: 2, },),).toBe(true,);
            }
          },
        },),
        it({
          name: 'IS FALSE FOR A PARAGRAPH INDENTED LESS, one that follows a mark with no white space, or a block that is none',
          fn: async () => {
            for (const source of ['> [^1]: x\n>    p\n', '> [^1]: x\n> p\n', '> [^1]: x\n>p\n',]) {
              const { node, range, } = childOfQuote({ source, at: 1, },);
              expect(continuesAsParagraph({ source, node, range, prefixLength: 2, },),).toBe(false,);
            }
            const source = '> [^1]: x\n>\n>     code\n';
            const { node, range, } = childOfQuote({ source, at: 1, },);
            expect(node.type,).toBe('code',);
            expect(continuesAsParagraph({ source, node, range, prefixLength: 2, },),).toBe(false,);
          },
        },),
        it({
          name: 'IS FALSE WHERE A CHARACTER THAT IS NO WHITE SPACE STANDS BETWEEN THE MARKS AND THE PARAGRAPH',
          fn: async () => {
            /**
             A quote whose second line holds a nested quote mark before the paragraph.
             */
            const source = '> [^1]: x\n>   > p\n';
            const { node, range, } = childOfQuote({ source, at: 1, },);
            expect(node.type,).toBe('blockquote',);
            expect(continuesAsParagraph({ source, node, range, prefixLength: 2, },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: continuesTheFootnote.name,
      children: [
        it({
          name: 'IS TRUE FOR INDENTED CODE AFTER AN OPEN FOOTNOTE, and for a paragraph indented four columns in a container',
          fn: async () => {
            /**
             A quote whose footnote holds indented code after a blank line.
             */
            const code = '> [^1]: x\n>\n>     c\n';
            const codeChild = childOfQuote({ source: code, at: 1, },);
            expect(continuesTheFootnote({ source: code, ...codeChild, state: OPEN_IN_QUOTE, isRootChild: false, },),)
              .toBe(true,);
            expect(continuesTheFootnote({ source: code, ...codeChild, state: OPEN_IN_QUOTE, isRootChild: true, },),)
              .toBe(true,);
            /**
             A quote whose footnote holds a paragraph indented four columns on the next line.
             */
            const paragraph = '> [^1]: x\n>     p\n';
            expect(continuesTheFootnote({
              source: paragraph,
              ...childOfQuote({ source: paragraph, at: 1, },),
              state: OPEN_IN_QUOTE,
              isRootChild: false,
            },),).toBe(true,);
          },
        },),
        it({
          name: 'IS TRUE FOR A PARAGRAPH RIGHT AFTER THE DEFINITION ITSELF, which goes on in the definition as a lazy line',
          fn: async () => {
            /**
             A quote whose definition is followed right away by a line of text, by a line of text after a blank line, and by a
             line that opens a footnote of its own.
             */
            const lazy = '> [^1]: x\n> abc\n>     ```\n';
            const apart = '> [^1]: x\n>\n> abc\n';
            const opener = '> [^1]: x\n> [^2]: y z\n';
            expect(continuesTheFootnote({
              source: lazy,
              ...childOfQuote({ source: lazy, at: 1, },),
              state: OPEN_IN_QUOTE,
              isRootChild: false,
            },),).toBe(true,);
            expect(continuesTheFootnote({
              source: lazy,
              ...childOfQuote({ source: lazy, at: 1, },),
              state: { ...OPEN_IN_QUOTE, opensFootnote: false, },
              isRootChild: false,
            },),).toBe(false,);
            expect(continuesTheFootnote({
              source: apart,
              ...childOfQuote({ source: apart, at: 1, },),
              state: OPEN_IN_QUOTE,
              isRootChild: false,
            },),).toBe(false,);
            expect(continuesTheFootnote({
              source: opener,
              ...childOfQuote({ source: opener, at: 1, },),
              state: OPEN_IN_QUOTE,
              isRootChild: false,
            },),).toBe(false,);
          },
        },),
        it({
          name: 'IS FALSE FOR A BLOCK RIGHT AFTER A DEFINITION THAT OPENS WITH RAW HTML, true after a blank line, in a container',
          fn: async () => {
            /**
             The state after a definition that opens with a tag alone.
             */
            const html = { ...OPEN_IN_QUOTE, opensRawHtml: true, };
            /**
             A quote whose definition is followed right away by indented text, and the same after a blank line.
             */
            const right = '> [^1]: <b>\n>     p\n';
            const after = '> [^1]: <b>\n>\n>     p\n';
            expect(continuesTheFootnote({
              source: right,
              ...childOfQuote({ source: right, at: 1, },),
              state: html,
              isRootChild: false,
            },),).toBe(false,);
            expect(continuesTheFootnote({
              source: after,
              ...childOfQuote({ source: after, at: 1, },),
              state: html,
              isRootChild: false,
            },),).toBe(true,);
            expect(continuesTheFootnote({
              source: right,
              ...childOfQuote({ source: right, at: 1, },),
              state: OPEN_IN_QUOTE,
              isRootChild: false,
            },),).toBe(true,);
          },
        },),
        it({
          name: 'IS FALSE WHERE THE BLOCK READ BEFORE IS NO FOOTNOTE, or for a paragraph at the top, which the footnote\'s range takes',
          fn: async () => {
            /**
             The quote of the paragraph case.
             */
            const paragraph = '> [^1]: x\n>     p\n';
            const child = childOfQuote({ source: paragraph, at: 1, },);
            expect(continuesTheFootnote({
              source: paragraph,
              ...child,
              state: { ...OPEN_IN_QUOTE, open: false, },
              isRootChild: false,
            },),).toBe(false,);
            expect(continuesTheFootnote({ source: paragraph, ...child, state: OPEN_IN_QUOTE, isRootChild: true, },),)
              .toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
