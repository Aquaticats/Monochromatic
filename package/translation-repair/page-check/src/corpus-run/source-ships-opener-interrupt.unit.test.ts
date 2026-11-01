/**
 Tests that a comment opener that never closes, standing first on a line right after a paragraph, is shown as the source
 shows it when the paragraph holds a backtick or a dollar sign that nothing closes.

 A reader of the source takes the opener at the start of a line for the start of a block of raw HTML, which cuts the
 paragraph short, so the backtick or dollar sign before it opens no code span or math. The page has no raw HTML: it
 writes the opener as a character reference and the lines after it as the text of a paragraph, so the paragraph went on
 and the backtick paired with one of the lines after the opener, and the page showed a code span the source does not
 have. A blank line before the line of the opener ends the paragraph on the page where it ends for the reader of the
 source (`source-ships-html-interrupt.ts`).

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
    'AN OPENER FIRST ON A LINE AFTER A PARAGRAPH WITH A BACKTICK THAT NOTHING CLOSES',
    'a `x\n<!-- never y` z {d}\n',
  ],
  [
    'AN OPENER INDENTED TWO SPACES AFTER A PARAGRAPH WITH A BACKTICK THAT NOTHING CLOSES',
    'a `x\n  <!-- never y` z {d}\n',
  ],
  [
    'AN OPENER FIRST ON A LINE AFTER A PARAGRAPH WITH A DOLLAR SIGN THAT NOTHING CLOSES',
    'a $x\n<!-- never y$ z {d}\n',
  ],
  [
    'AN OPENER FIRST ON A LINE AFTER A PARAGRAPH OF TWO LINES WITH A BACKTICK THAT NOTHING CLOSES',
    'a `x\nq\n<!-- never y` z {d}\n',
  ],
  [
    'AN OPENER FIRST ON A LINE AFTER A PARAGRAPH, WITH WINDOWS LINE ENDINGS',
    'a `x\r\n<!-- never y` z {d}\r\n',
  ],
  [
    'AN OPENER FIRST ON A LINE AFTER A PARAGRAPH WITH NO BACKTICK',
    'a x\n<!-- never\ny z {d}\n',
  ],
  [
    'AN OPENER AFTER OTHER TEXT ON ITS LINE, WHICH STARTS NO BLOCK',
    'a `x <!-- never y` z {d}\n',
  ],
  [
    'AN OPENER FIRST ON A LINE AFTER A BLANK LINE',
    'a `x\n\n<!-- never y` z {d}\n',
  ],
];

await describe({
  name: 'a comment opener first on a line after a paragraph that holds a backtick nothing closes',
  children: readerEqualityCases({ cases: CASES, },),
},);
