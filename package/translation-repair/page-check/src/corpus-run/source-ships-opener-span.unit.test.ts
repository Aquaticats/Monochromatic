/**
 Tests that a code span cut before a comment opener keeps the white space that stood between its text and the opener.

 A code span that holds an opener of a comment that never closes is closed before the opener and the rest of it is
 written as text (`source-ships-opener-in-code.ts`). A span that starts with a line ending and ends with a space has one
 space taken off each end by a reader, so the space before the opener, kept inside the shortened span, was taken off with
 it and the page showed the text and the opener without the space between them. The white space is written after the
 closing delimiter, in the text.

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
    'A CODE SPAN OF THREE BACKTICKS ON LINES INDENTED FOUR SPACES, A SPACE BEFORE THE OPENER',
    'p\n    ```\n    x <!-- never\n    ```\n',
  ],
  [
    'A CODE SPAN OF THREE BACKTICKS ON LINES INDENTED FOUR SPACES, TWO WORDS BEFORE THE OPENER',
    'p\n    ```\n    x y <!-- never\n    ```\n',
  ],
  [
    'A CODE SPAN OF THREE BACKTICKS ON LINES INDENTED BY A TAB, A SPACE BEFORE THE OPENER',
    'p\n\t```\n\tx <!-- never\n\t```\n',
  ],
  [
    'A CODE SPAN WITH A SPACE AFTER ITS OPENING DELIMITER, A SPACE BEFORE THE OPENER',
    'p ``` x <!-- never ``` q\n',
  ],
  [
    'A CODE SPAN ON LINES INDENTED FOUR SPACES IN A BLOCK QUOTE, A SPACE BEFORE THE OPENER',
    '> p\n>     ```\n>     x <!-- never\n>     ```\n',
  ],
  [
    'A CODE SPAN ON LINES INDENTED FOUR SPACES IN A LIST ITEM, A SPACE BEFORE THE OPENER',
    '- p\n      ```\n      x <!-- never\n      ```\n',
  ],
  [
    'A CODE SPAN ON LINES INDENTED FOUR SPACES AFTER A FOOTNOTE LINE IN A BLOCK QUOTE, NOTHING BUT MARKS BEFORE THE OPENER',
    '> [^1]: x\n>     ```\n>     <!-- never\n>     ```\n',
  ],
  [
    'A CODE SPAN ON LINES INDENTED FOUR SPACES AFTER A FOOTNOTE LINE IN A LIST ITEM, NOTHING BUT WHITE SPACE BEFORE THE OPENER',
    '- [^1]: x\n      ```\n      <!-- never\n      ```\n',
  ],
  [
    'A CODE SPAN IN A BLOCK QUOTE, CODE AND A MARK BEFORE THE OPENER',
    '> p ```\n> x\n> <!-- never\n> ``` q\n',
  ],
  [
    'A CODE SPAN ON ONE LINE, A SPACE BEFORE THE OPENER',
    'a `x <!-- never` b\n',
  ],
  [
    'A CODE SPAN ON TWO LINES, THE OPENER RIGHT AFTER THE TEXT',
    'p ```\nx<!-- never\n``` q\n',
  ],
  [
    'A CODE SPAN ON LINES INDENTED FOUR SPACES, SEVERAL SPACES BEFORE THE OPENER',
    'p\n    ```\n    x   <!-- never\n    ```\n',
  ],
];

await describe({
  name: 'a code span cut before a comment opener',
  children: readerEqualityCases({ cases: CASES, },),
},);
