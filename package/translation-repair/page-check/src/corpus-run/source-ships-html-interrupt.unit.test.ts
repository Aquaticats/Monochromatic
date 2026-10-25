/**
 Tests that a block of raw HTML standing first on a line right after a paragraph is shown as the source shows it when the
 paragraph holds a backtick or a dollar sign that nothing closes.

 A reader of the source takes a line that starts an HTML block of the kinds that may interrupt a paragraph (an element of
 the block list such as a `div`, a `pre`, a `script`, a processing instruction) for the start of that block, which ends the
 paragraph before it, so the backtick or dollar sign before it opens no code span or math. The page has no raw HTML: it
 writes the lines of the block as the text of a paragraph, so the paragraph went on and the backtick paired with one of the
 lines of the block, and the page showed a code span the source does not have. A blank line before the line of the block
 ends the paragraph on the page where it ends for the reader of the source (`source-ships-html-interrupt.ts`). A comment
 that never closes is such a block (`source-ships-opener-interrupt.unit.test.ts`).

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
    'A DIV FIRST ON A LINE AFTER A PARAGRAPH WITH A BACKTICK THAT NOTHING CLOSES',
    'a `x\n<div>\ny` z {d}\n',
  ],
  [
    'A DIV AFTER A PARAGRAPH WITH A BACKTICK, A BRACE IN A PARAGRAPH AFTER A BLANK LINE',
    'a `x\n<div>\ny` z\n\n{d}\n',
  ],
  [
    'A DIV AFTER A PARAGRAPH WITH A DOLLAR SIGN THAT NOTHING CLOSES',
    'a $x\n<div>\ny$ z {d}\n',
  ],
  [
    'A PRE ELEMENT AFTER A PARAGRAPH WITH A BACKTICK THAT NOTHING CLOSES',
    'a `x\n<pre>\ny` z {d}\n</pre>\n',
  ],
  [
    'A SCRIPT ELEMENT AFTER A PARAGRAPH WITH A BACKTICK THAT NOTHING CLOSES',
    'a `x\n<script>\ny` z {d}\n',
  ],
  [
    'A PROCESSING INSTRUCTION AFTER A PARAGRAPH WITH A BACKTICK THAT NOTHING CLOSES',
    'a `x\n<?php\ny` z {d}\n?>\n',
  ],
  [
    'A DIV INDENTED TWO SPACES AFTER A PARAGRAPH WITH A BACKTICK THAT NOTHING CLOSES',
    'a `x\n  <div>\ny` z {d}\n',
  ],
  [
    'A DIV AFTER A PARAGRAPH WITH A BACKTICK, WITH WINDOWS LINE ENDINGS',
    'a `x\r\n<div>\r\ny` z {d}\r\n',
  ],
  [
    'A DIV AFTER A PARAGRAPH WITH A BACKTICK, IN A BLOCK QUOTE',
    '> a `x\n> <div>\n> y` z {d}\n',
  ],
  [
    'A DIV AFTER A PARAGRAPH WITH A BACKTICK, IN A LIST ITEM',
    '- a `x\n  <div>\n  y` z {d}\n',
  ],
  [
    'A DIV AFTER A PARAGRAPH WITH A BACKTICK, CLOSED BY ITS END TAG',
    'a `x\n<div>\ny` z {d}\n</div>\n',
  ],
  [
    'A DIV AFTER A PARAGRAPH WITH NO BACKTICK',
    'a x\n<div>\ny z {d}\n',
  ],
  [
    'A DIV AFTER A BLANK LINE',
    'a `x\n\n<div>\ny` z {d}\n',
  ],
  [
    'AN INLINE ELEMENT FIRST ON A LINE, WHICH STARTS NO BLOCK',
    'a `x\n<span>\ny` z {d}\n',
  ],
  [
    'A DIV AFTER THE FIRST PARAGRAPH OF A FOOTNOTE DEFINITION',
    '[^1]: a `x\n<div>\ny` z {d}\n',
  ],
];

await describe({
  name: 'a block of raw HTML first on a line after a paragraph that holds a backtick nothing closes',
  children: readerEqualityCases({ cases: CASES, },),
},);
