/**
 Tests that a span the plain reading would pair across the rows of a table, and the lines that follow a row, are shown as
 the source shows them.

 A reader of the source takes a table for a table: a cell ends at its row, so a backtick, a dollar sign or an asterisk
 that a row opens pairs with nothing in the row after it, and a line that starts a block of raw HTML ends the table.
 The plain reading the encoder walks once had no tables and took the rows and the lines after them for one paragraph, so it
 paired the span across the rows, wrote the brace or the tag on the line after the row into the span as code, and left a
 line of markup after a tag unescaped, where the page reads a list item or a heading the source does not.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { describe, } from '@monochromatic-dev/module-test/ts';

import { readerEqualityCases, } from './source-reader-equality.test-fixture.ts';

/**
 Texts a reader must see as the source shows them: what the case is, and the text.
 */
const CASES: readonly (readonly [string, string])[] = [
  [
    'A ROW WITH A BACKTICK THAT NOTHING CLOSES, THEN A LINE WITH A BRACE AND A BACKTICK',
    '| a | b |\n| - | - |\n| c | `d |\n{d} y` |\n',
  ],
  [
    'A ROW WITH A BACKTICK THAT NOTHING CLOSES, THEN A LINE WITH A TAG AND A BACKTICK',
    '| a | b |\n| - | - |\n| c | `d |\n<b> y` |\n',
  ],
  [
    'A ROW WITH A BACKTICK THAT NOTHING CLOSES, THEN A BLOCK TAG ON A LINE OF ITS OWN AND A BACKTICK',
    '| a | b |\n| - | - |\n| c | `d |\n<div>\ny` $ * "\n',
  ],
  [
    'A ROW STARTING WITH A DOLLAR SIGN THAT NOTHING CLOSES, THEN A LINE WITH A BRACE',
    '| a | b |\n| - | - |\n| $c | d |\n{d} y$\n',
  ],
  [
    'A ROW WITH AN ASTERISK THAT NOTHING CLOSES, THEN A LINE WITH A BRACE',
    '| a | b |\n| - | - |\n| c | *d |\n{d} y* |\n',
  ],
  [
    'A ROW WITH A LINK TITLE THAT NOTHING CLOSES, THEN A LINE WITH A BRACE',
    '| a | b |\n| - | - |\n| c | [t](u "x |\n{d} y") |\n',
  ],
  [
    'A TABLE WITHOUT LEADING PIPES, A ROW WITH A BACKTICK, THEN A LINE WITH A BRACE',
    'a | b\n- | -\nc | `d\n{d} y`\n',
  ],
  [
    'A ROW WITH NO SPAN, THEN A TAG ALONE ON A LINE AND A LIST MARKER',
    '| a | b |\n| - | - |\n| c | d |\n<b>\n- j\n',
  ],
  [
    'A ROW WITH NO SPAN, THEN AN ELEMENT WITH AN ATTRIBUTE ALONE ON A LINE AND A LIST MARKER',
    '| a | b |\n| - | - |\n| c | d |\n<a href="x">\n- j\n',
  ],
  [
    'A ROW WITH NO SPAN, THEN A LINE BREAK TAG ALONE ON A LINE AND A HEADING MARK',
    '| a | b |\n| - | - |\n| c | d |\n<br>\n# j\n',
  ],
  [
    'A ROW WITH NO SPAN, THEN A CUSTOM ELEMENT ALONE ON A LINE AND A LIST MARKER',
    '| a | b |\n| - | - |\n| c | d |\n<custom-el>\n- j\n',
  ],
  [
    'A TABLE IN A BLOCK QUOTE, A ROW WITH A BACKTICK, THEN A LINE WITH A BRACE',
    '> | a | b |\n> | - | - |\n> | c | `d |\n> {d} y` |\n',
  ],
  [
    'A TABLE IN A LIST ITEM, A ROW WITH A BACKTICK, THEN A LINE WITH A TAG',
    '- | a | b |\n  | - | - |\n  | c | `d |\n  <b> y` |\n',
  ],
  [
    'A TABLE WITH CLOSED SPANS IN ITS ROWS, THEN A LINE WITH A BRACE',
    '| a | b |\n| - | - |\n| `c` | $d$ |\n{d} y\n',
  ],
];

await describe({
  name: 'a line after the row of a table, and a span a row opens',
  children: readerEqualityCases({ cases: CASES, },),
},);
