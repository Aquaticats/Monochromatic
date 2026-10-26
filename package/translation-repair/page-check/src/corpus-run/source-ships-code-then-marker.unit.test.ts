/**
 Tests for a line that looks like a list marker right after indented code.

 A line that starts a list item that cannot interrupt a paragraph (an empty item, or an ordered item that does not start
 at one) is a line of text after indented code to the parse, as it is after a paragraph, with a blank line between the
 two or none. The encoder writes a blank line after indented code, and on the page, which has no indented code, a blank
 line lets such a line open a list that the source does not have: the page lacks the marker the source shows. The marker
 is written with a backslash.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  encodeAsLiteralMdx,
  readingDifferenceOf,
} from '../../dist/final/node/index.mjs';

/**
 What the encoder writes for a text, and what the comparison says of the page.

 @param source - text of a run of slices

 @returns The page text and the answer of the comparison

 @example
 ```ts
 const { written, kind, } = encodedReading({ source: '    注\n2.\n', },);
 ```
 */
function encodedReading(
  { source, }: { readonly source: string; },
): { readonly written: string; readonly kind: string; } {
  /**
   The page text the encoder writes.
   */
  const written = encodeAsLiteralMdx({ text: source, },);
  return {
    written,
    kind: readingDifferenceOf({
      source,
      carried: written,
    },).kind,
  };
}

await describe({
  name: 'source ships a list marker after indented code',
  children: [
    it({
      name: 'WRITES THE MARKER OF AN EMPTY LIST ITEM AFTER INDENTED CODE WITH A BACKSLASH, so the page shows it as the source does',
      fn: async () => {
        for (
          const [source, written,] of [
            ['    注\n2.\n', '    注\n\n2\\.\n',],
            ['    注\n\n2.\n', '    注\n\n2\\.\n',],
            ['    注\n*\n', '    注\n\n\\*\n',],
            ['    注\n-\n', '    注\n\n\\-\n',],
            ['    注\n+\n', '    注\n\n\\+\n',],
            ['    注\n2)\n', '    注\n\n2\\)\n',],
            ['    注\n1.\n', '    注\n\n1\\.\n',],
            ['    注\n*  \n', '    注\n\n\\*  \n',],
            ['    {x}\n\n*\n', '    \\{x\\}\n\n\\*\n',],
          ] as const
        )
          expect(encodedReading({ source, },),).toEqual({
            written,
            kind: 'same',
          },);
      },
    },),

    it({
      name: 'WRITES THE MARKER OF AN ORDERED ITEM THAT DOES NOT START AT ONE WITH A BACKSLASH, since after indented code it is text',
      fn: async () => {
        expect(encodedReading({ source: '    c\n2. x\n', },),).toEqual({
          written: '    c\n\n2\\. x\n',
          kind: 'same',
        },);
        expect(encodedReading({ source: '    c\n\n10) x\n', },),).toEqual({
          written: '    c\n\n10\\) x\n',
          kind: 'same',
        },);
      },
    },),

    it({
      name: 'WRITES THE MARKER WITH A BACKSLASH AFTER INDENTED CODE IN A BLOCK QUOTE AND IN A LIST ITEM',
      fn: async () => {
        expect(encodedReading({ source: '> a\n>\n>     注\n> 2.\n', },),).toEqual({
          written: '> a\n>\n>     注\n>\n> 2\\.\n',
          kind: 'same',
        },);
        expect(encodedReading({ source: '- a\n\n      注\n  2.\n', },),).toEqual({
          written: '- a\n\n      注\n\n  2\\.\n',
          kind: 'same',
        },);
      },
    },),

    it({
      name: 'LEAVES A LIST ITEM THAT CAN INTERRUPT A PARAGRAPH, and one that no indented code precedes, as they stand',
      fn: async () => {
        for (
          const [source, written,] of [
            ['    c\n1. x\n', '    c\n\n1. x\n',],
            ['    c\n- x\n', '    c\n\n- x\n',],
            ['    c\n* x\n', '    c\n\n* x\n',],
            ['a\n\n2.\n', 'a\n\n2.\n',],
            ['a\n\n*\n', 'a\n\n*\n',],
            ['~~~\nc\n~~~\n2.\n', '~~~\nc\n~~~\n2.\n',],
          ] as const
        )
          expect(encodedReading({ source, },),).toEqual({
            written,
            kind: 'same',
          },);
      },
    },),
  ],
},);
