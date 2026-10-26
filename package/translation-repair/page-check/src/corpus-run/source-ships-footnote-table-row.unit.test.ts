/**
 Tests that a table row holding a backtick that nothing closes, followed by a footnote definition line indented four
 columns or more, is shown as the source shows it.

 A reader of the source takes the table for a table and the indented line for a block of its own, so no code span runs
 across the two. The plain reading the encoder walks once had no tables and took the rows and the line for one paragraph
 with a code span across them. Joining the footnote line to the row before it, as the encoder does for a span that really runs
 across the line (`source-ships-lines.ts`), wrote the line into the row: the row lost its closing pipe, or took the
 line's tag and the page did not parse. The line stays where it is, and the footnote it starts is cut as it is for any
 other paragraph.

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
    'A TABLE ROW WITH A BACKTICK THAT NOTHING CLOSES, THEN A FOOTNOTE LINE INDENTED FOUR SPACES CLOSING IT',
    '| a | b |\n| - | - |\n| c | `d |\n    [^1]: y` |\n',
  ],
  [
    'A TABLE ROW WITH A BACKTICK THAT NOTHING CLOSES, THEN A FOOTNOTE LINE WITH A TAG IN IT',
    '| a | b |\n| - | - |\n| c | `d |\n    [^1]: <b>` z\nmore\n',
  ],
  [
    'A TABLE ROW STARTING WITH A BACKTICK THAT NOTHING CLOSES, THEN A FOOTNOTE LINE WITH A BRACE AFTER IT',
    '| a | b |\n| - | - |\n| `c |\n    [^1]: y` | z {d}\n',
  ],
  [
    'A TABLE ROW WITH A BACKTICK, THEN A FOOTNOTE LINE INDENTED BY A TAB',
    '| a | b |\n| - | - |\n| c | `d |\n\t[^1]: y`\n| e |\n',
  ],
  [
    'A TABLE IN A BLOCK QUOTE, ITS LAST ROW HOLDING A BACKTICK, THEN A FOOTNOTE LINE WITH A TAG',
    '> | a | b |\n> | - | - |\n> | c | `d |\n>     [^1]: <b>` z\n',
  ],
  [
    'A TABLE ROW WITH A DOLLAR SIGN THAT NOTHING CLOSES, THEN A FOOTNOTE LINE INDENTED FOUR SPACES',
    '| a | b |\n| - | - |\n| c | $d |\n    [^1]: y$ z |\n',
  ],
];

await describe({
  name: 'a table row before a footnote definition line indented four columns or more',
  children: readerEqualityCases({ cases: CASES, },),
},);
