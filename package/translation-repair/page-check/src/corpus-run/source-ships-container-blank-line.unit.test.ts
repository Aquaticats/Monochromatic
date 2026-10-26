/**
 Tests the exact text the encoder writes for a block quote or a list item whose raw HTML block or indented code is
 followed by a definition.

 The page writes such a block as a paragraph, so the line after it needs a blank line, and a blank line inside a block
 quote carries the quote mark: without the mark the blank line ends the quote and the definition stands outside it. A
 reader of the page by leaf text cannot tell the two writings apart (it sees the same words in both), so these cases
 assert the written text.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { encodeAsLiteralMdx, } from '../../dist/final/node/index.mjs';

await describe({
  name: 'the blank line the encoder writes in a container after a block the page writes as a paragraph',
  children: [
    it({
      name: 'KEEPS THE QUOTE MARK ON THE BLANK LINE AFTER A RAW HTML BLOCK IN A BLOCK QUOTE',
      fn: async () => {
        expect(encodeAsLiteralMdx({ text: '> <pre>\n> x\n> </pre>\n> [r]: u\n', },),).toBe(
          '> \\<pre\\>\n> x\n> \\</pre\\>\n>\n> [r]: u\n',
        );
      },
    },),
    it({
      name: 'KEEPS THE QUOTE MARK ON THE BLANK LINE AFTER INDENTED CODE IN A BLOCK QUOTE',
      fn: async () => {
        expect(encodeAsLiteralMdx({ text: '> a\n>\n>     code {c}\n> [r]: u{b}\n', },),).toBe(
          '> a\n>\n>     code \\{c\\}\n>\n> [r]: u\\{b\\}\n',
        );
      },
    },),
    it({
      name: 'KEEPS BOTH QUOTE MARKS ON THE BLANK LINE AFTER A RAW HTML BLOCK IN A BLOCK QUOTE INSIDE A BLOCK QUOTE',
      fn: async () => {
        expect(encodeAsLiteralMdx({ text: '> > <?x?>\n> > [r]: u{b}\n', },),).toBe(
          '> > \\<?x?\\>\n> >\n> > [r]: u\\{b\\}\n',
        );
      },
    },),
    it({
      name: 'WRITES AN EMPTY BLANK LINE AFTER A RAW HTML BLOCK IN A LIST ITEM, whose hanging indent the line needs none of',
      fn: async () => {
        expect(encodeAsLiteralMdx({ text: '- <?x?>\n  [r]: u{b}\n', },),).toBe(
          '- \\<?x?\\>\n\n  [r]: u\\{b\\}\n',
        );
      },
    },),
  ],
},);
