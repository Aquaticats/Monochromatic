/**
 Tests that the lines right after a footnote definition in a container that opens with a tag alone are not taken for the
 footnote's own blocks.

 A tag alone on a line starts a block of raw HTML that holds the lines up to the next blank line, so a reader of the
 source takes a fence mark on one of them for text, and the encoder must write it as text. After a blank line the lines are
 blocks of the footnote, as for any other definition.

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
    'A FENCE OPENER RIGHT AFTER A FOOTNOTE THAT OPENS WITH A TAG ALONE IN A BLOCK QUOTE',
    '> [^1]: <b>\n>     ```js\n',
  ],
  [
    'A FENCE AND A LINE OF TEXT RIGHT AFTER A FOOTNOTE THAT OPENS WITH A TAG ALONE IN A BLOCK QUOTE',
    '> [^1]: <b>\n>     ```\n>     a\n',
  ],
  [
    'A FENCE HOLDING A BRACE RIGHT AFTER A FOOTNOTE THAT OPENS WITH A TAG ALONE IN A LIST ITEM, THEN TEXT',
    '- [^1]: <b>\n      ```\n      {a}\n  tail {z}\n',
  ],
  [
    'A FENCE OF TILDES CLOSED BY BACKTICKS RIGHT AFTER A FOOTNOTE THAT OPENS WITH A TAG ALONE IN A BLOCK QUOTE',
    '> [^1]: <b>\n>     ~~~\n>     {a}\n>     ```\n',
  ],
  [
    'A FENCE HOLDING A TAG AND A BRACE RIGHT AFTER A FOOTNOTE THAT OPENS WITH A TAG ALONE IN A LIST ITEM',
    '- [^1]: <b>\n      ```\n      <i>\n      {b}\n  ```\n  [r]: u{b}\n',
  ],
  [
    'A DISPLAY MATH AFTER A BLANK LINE AFTER A FOOTNOTE THAT OPENS WITH A TAG ALONE IN A BLOCK QUOTE',
    '> [^1]: <b>\n>\n>     $$\n>     {a}\n>     $$\n',
  ],
  [
    'A FENCE AFTER A BLANK LINE AFTER A FOOTNOTE THAT OPENS WITH A TAG ALONE IN A BLOCK QUOTE',
    '> [^1]: <b>\n>\n>     ```\n>     {a}\n>     ```\n',
  ],
  [
    'A FENCE RIGHT AFTER A FOOTNOTE THAT OPENS WITH A WORD IN A BLOCK QUOTE',
    '> [^1]: x\n>     ```\n>     {a}\n>     ```\n',
  ],
  [
    'A FENCE RIGHT AFTER A FOOTNOTE THAT OPENS WITH A TAG AND A WORD IN A BLOCK QUOTE',
    '> [^1]: <b> x\n>     ```\n>     {a}\n>     ```\n',
  ],
];

await describe({
  name: 'the lines right after a footnote definition in a container that opens with a tag alone',
  children: readerEqualityCases({ cases: CASES, },),
},);
