/**
 Tests that a footnote definition in a container that the plain reading takes for a paragraph is shown as the source
 shows it, whatever its lines are.

 The plain reading takes a footnote line for a paragraph when the text after the label is no destination (a brace, a tag
 with text), and for a link reference definition otherwise; the lines after it are lazy lines, a line of raw HTML among
 them, and a definition line that follows is text of the footnote's paragraph. The body of the footnote is read as a text
 of its own whichever the plain reading says, the marks of the container given up by the lines that carry them, so that a
 line joined to the one before it does not leave its quote mark in the page.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { encodeAsLiteralMdx, } from '../../dist/final/node/index.mjs';
import { readerEqualityCases, } from './source-reader-equality.test-fixture.ts';

/**
 Texts a reader must see as the source shows them: what the case is, and the text.
 */
const CASES: readonly (readonly [string, string])[] = [
  [
    'A TAG WITH AN ATTRIBUTE AFTER A FOOTNOTE LINE IN A BLOCK QUOTE, THEN A DEFINITION LINE WITHOUT THE QUOTE MARK',
    '>[^1]:x\n><a f="">\n[r]:}',
  ],
  [
    'A TAG WITH AN ATTRIBUTE AFTER A FOOTNOTE LINE IN A BLOCK QUOTE, THEN A DEFINITION LINE WITH THE QUOTE MARK, THEN A SCRIPT',
    '> [^1]: x\n> <a href="x">\n> [r]: u{b}\n<script>\n',
  ],
  [
    'A FOOTNOTE LINE THAT HOLDS A BRACE AND A CODE SPAN IN A BLOCK QUOTE, THEN A SCRIPT WITHOUT THE QUOTE MARK',
    '> a `x\n> [^1]: {c}` {d}\n<script>\n',
  ],
  [
    'A FOOTNOTE IN A LIST ITEM WHOSE LINE HOLDS A TAG ALONE, A DEFINITION LINE, AND FOOTNOTES AT THE TOP WITH BRACES',
    '- [^1]: <b>\n  [r]: u{b}\n[^a}]: x {\n\n[^a}] y\n[^1]: 注\n',
  ],
  [
    'INLINE MATH ACROSS THREE LINES OF A BLOCK QUOTE AFTER A LAZY DEFINITION LINE OF A FOOTNOTE, WITH LONE CARRIAGE RETURNS',
    '> [^1]: 注\n[b]: u\n> $\r> {a}\r> $\r\n',
  ],
  [
    'A LINE WITH A WORD OF AN ES MODULE AFTER A LAZY DEFINITION LINE, AND A QUOTE MARK AFTER TWO SPACES ON THE NEXT LINE',
    '> [^1]: 注\n[b]: u import a from "b" - > [^1]: <b>\n  > [r]: u{b}\n',
  ],
  [
    'A FOOTNOTE LINE WITH A LAZY LINE THAT HOLDS A FENCE MARK, IN A BLOCK QUOTE',
    '> [^1]: x\n> abc\n>     ```\n>     {a}\n>     ```\n',
  ],
];

await describe({
  name: 'a footnote definition in a container read as the body of a footnote',
  children: [
    ...readerEqualityCases({ cases: CASES, },),
    it({
      name: 'KEEPS THE QUOTE MARK OF THE LINE AFTER THE BLANK LINE THE ENCODER WRITES BETWEEN A TAG AND A DEFINITION LINE',
      fn: async () => {
        // The blank line ends the quote on the page, and the line after it starts another with its own mark.
        expect(encodeAsLiteralMdx({ text: '> a\n>     ```\n> [^1]: x\n> <a href="x">\n> [r]: u{b}\n', },),).toBe(
          '> a\n>     &#96;&#96;&#96;\n> [^1]: x\n> \\<a href="x">\n\n> [r]: u{b}\n',
        );
      },
    },),
  ],
},);
