/**
 Tests that a fence or a display math right after a footnote definition that opens with a tag alone is written as text.

 A tag alone on the definition's line starts a block of raw HTML that holds the lines up to the next blank line, so a
 reader of the source sees a fence mark or the mark of a display math on one of them as text. The plain parse reads two
 such marks as the delimiters of a code span or of inline math that runs over the lines, and a comment opener among the
 lines as code to take out of it, which dropped the marks from the page. The page must show the marks, and the opener as
 the characters it is.

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
    'A FENCE OPENED AND CLOSED RIGHT AFTER A FOOTNOTE THAT OPENS WITH A TAG ALONE IN A LIST ITEM',
    '- [^1]: <b>\n      ```\n      ```\n',
  ],
  [
    'A FENCE HOLDING A BRACE RIGHT AFTER A FOOTNOTE THAT OPENS WITH A TAG ALONE IN A BLOCK QUOTE',
    '> [^1]: <b>\n>     ```\n>     {a}\n>     ```\n',
  ],
  [
    'A DISPLAY MATH HOLDING A BRACE RIGHT AFTER A FOOTNOTE THAT OPENS WITH A TAG ALONE IN A LIST ITEM',
    '- [^1]: <b>\n      $$\n      {a}\n      $$\n',
  ],
  [
    'A FENCE OPENED AFTER A TAG ALONE IN A LIST ITEM, THEN A LINE THAT STARTS A DEFINITION',
    '- [^1]: <b>\n      ```\n  [r]: u{b}\n',
  ],
  [
    'A FENCE WITH A COMMENT OPENER THAT NEVER CLOSES RIGHT AFTER A TAG ALONE IN A LIST ITEM',
    '- [^1]:<b>\n      ```\n      <!-- never\n      ```\n',
  ],
  [
    'A FENCE WITH A COMMENT OPENER THAT NEVER CLOSES RIGHT AFTER A TAG ALONE AT THE TOP',
    '[^1]:<b>\n    ```\n    <!-- never\n    ```\n',
  ],
  [
    'A FENCE WITH A COMMENT OPENER THAT NEVER CLOSES RIGHT AFTER A TAG ALONE IN A BLOCK QUOTE, THEN TEXT',
    '> [^1]:<b>\n>     ```\n>     <!-- never\n>     ```\n> tail {z}\n',
  ],
  [
    'A DISPLAY MATH WITH A COMMENT OPENER THAT NEVER CLOSES RIGHT AFTER A TAG ALONE IN A LIST ITEM',
    '- [^1]:<b>\n      $$\n      <!-- never\n      $$\n',
  ],
  [
    'A FENCE AFTER A TAG ALONE IN A BLOCK QUOTE, THEN A LINE THAT STARTS A DEFINITION',
    '> [^1]: <b>\n>      ```\n> [r]: u{b}\n',
  ],
];

await describe({
  name: 'a fence or display math right after a footnote definition that opens with a tag alone',
  children: readerEqualityCases({ cases: CASES, },),
},);
