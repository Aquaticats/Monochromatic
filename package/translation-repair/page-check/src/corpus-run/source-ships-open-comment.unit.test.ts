/**
 Tests that a comment which never closes, opened at the start of a line, leaves the page showing what the source shows.

 A reader of the source takes a line that starts with the four characters of a comment opener, when nothing closes it, for a
 block of raw HTML that holds the rest of the text, so the lines after it are text there: a quote mark, a label, a heading
 mark or a list marker among them is no syntax. The encoder writes the opener as a character reference so that the package's
 own parser, which hides everything after such an opener, still reads the page, and the lines after it were then read as
 Markdown on the page: the quote, the footnote, the definition and the heading showed as the structures they are, which hide
 the marks the source shows. The rest of the text from such an opener is written as the text a reader of the source sees.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { describe, } from '@monochromatic-dev/test-harness-core/ts';

import { readerEqualityCases, } from './source-reader-equality.test-fixture.ts';

/**
 Texts a reader must see as the source shows them: what the case is, and the text.
 */
const CASES: readonly (readonly [string, string])[] = [
  [
    'A QUOTE, A FOOTNOTE AND A DEFINITION AFTER AN OPENER ALONE ON ITS LINE',
    '<!--\n> [^1]: 注\n[b]: u\n',
  ],
  [
    'A DEFINITION AFTER AN OPENER AND A WORD',
    '<!-- a\n[a]: u\n',
  ],
  [
    'A BRACE, A HEADING AND A CODE SPAN AFTER AN OPENER THAT FOLLOWS A PARAGRAPH',
    '猫\n\n<!-- a {b}\n# head\n`x` <b>\n',
  ],
  [
    'A LIST ITEM AND A NUMBERED ITEM AFTER AN OPENER INDENTED TWO SPACES',
    '  <!-- open\n- item\n1. n {x}\n',
  ],
  [
    'A DEFINITION WITH A BRACE AFTER AN OPENER THAT FOLLOWS A FOOTNOTE',
    '[^1]: x\n\n<!--\n[r]: u{b}\n',
  ],
  [
    'A DEFINITION AFTER AN OPENER THAT FOLLOWS A COMMENT THAT CLOSES',
    '<!-- 注释 -->\n\n<!-- a\n[a]: u\n',
  ],
];

await describe({
  name: 'the page after a comment that never closes, opened at the start of a line',
  children: readerEqualityCases({ cases: CASES, },),
},);
