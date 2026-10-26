/**
 Tests that a fence or a display math indented under a footnote definition is shown as the source shows it, in a block
 quote and in a list item as at the top, and that a comment opener in it is written outside the code.

 The strict grammar disables indented code, so the blocks indented four columns under a footnote definition are blocks of
 the footnote there, and a fence or display math among them is code or math, which the page shows as the source does.
 The plain reading has no footnotes and takes those lines for indented code, or for the lazy lines of the paragraph
 they follow, so the encoder reads them as the footnote's blocks. In a block quote or a list item the lines carry the marks and
 the hanging indent of the container before the four columns, and the encoder once wrote the marks of such a fence with
 a backslash, so that the page showed them, and found no opener that stood in one.

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
    'A FENCE UNDER A FOOTNOTE DEFINITION IN A BLOCK QUOTE, HOLDING A BRACE',
    '> [^1]: x\n>\n>     ```\n>     {a}\n>     ```\n',
  ],
  [
    'A FENCE UNDER A FOOTNOTE DEFINITION IN A LIST ITEM, HOLDING A BRACE',
    '- [^1]: x\n\n      ```\n      {a}\n      ```\n',
  ],
  [
    'A FENCE UNDER A FOOTNOTE DEFINITION IN AN ORDERED LIST ITEM, HOLDING A BRACE',
    '1. [^1]: x\n\n       ```\n       {a}\n       ```\n',
  ],
  [
    'A FENCE OF TILDES UNDER A FOOTNOTE DEFINITION IN A BLOCK QUOTE, HOLDING A BRACE',
    '> [^1]: x\n>\n>     ~~~\n>     {a}\n>     ~~~\n',
  ],
  [
    'A DISPLAY MATH UNDER A FOOTNOTE DEFINITION IN A BLOCK QUOTE, HOLDING A BRACE',
    '> [^1]: x\n>\n>     $$\n>     {a}\n>     $$\n',
  ],
  [
    'A FENCE AND A PARAGRAPH UNDER A FOOTNOTE DEFINITION IN A BLOCK QUOTE',
    '> [^1]: x\n>\n>     ```\n>     a\n>     ```\n>\n>     tail {t}\n',
  ],
  [
    'A FENCE UNDER A FOOTNOTE DEFINITION IN A BLOCK QUOTE IN A BLOCK QUOTE',
    '> > [^1]: x\n> >\n> >     ```\n> >     {a}\n> >     ```\n',
  ],
  [
    'A FENCE UNDER A FOOTNOTE DEFINITION IN A LIST ITEM IN A BLOCK QUOTE',
    '> - [^1]: x\n>\n>       ```\n>       {a}\n>       ```\n',
  ],
  [
    'A FENCE UNDER A FOOTNOTE DEFINITION IN A BLOCK QUOTE, HOLDING AN OPENER',
    '> [^1]: x\n>\n>     ```\n>     <!-- never\n>     ```\n',
  ],
  [
    'A FENCE UNDER A FOOTNOTE DEFINITION IN A LIST ITEM, HOLDING AN OPENER',
    '- [^1]: x\n\n      ```\n      <!-- never\n      ```\n',
  ],
  [
    'A FENCE UNDER A FOOTNOTE DEFINITION IN A BLOCK QUOTE IN A BLOCK QUOTE, HOLDING AN OPENER',
    '> > [^1]: x\n> >\n> >     ```\n> >     <!-- never\n> >     ```\n',
  ],
  [
    'A FENCE UNDER A FOOTNOTE DEFINITION IN A BLOCK QUOTE, HOLDING TEXT BEFORE AN OPENER',
    '> [^1]: x\n>\n>     ```\n>     x <!-- never\n>     ```\n',
  ],
  [
    'A DISPLAY MATH UNDER A FOOTNOTE DEFINITION IN A BLOCK QUOTE, HOLDING AN OPENER',
    '> [^1]: x\n>\n>     $$\n>     <!-- never\n>     $$\n',
  ],
  [
    'A FENCE RIGHT AFTER A FOOTNOTE DEFINITION AT THE TOP, HOLDING TEXT BEFORE AN OPENER',
    '[^1]: x\n    ```\n    x <!-- never\n    ```\n',
  ],
  [
    'A FENCE RIGHT AFTER A FOOTNOTE DEFINITION AT THE TOP, HOLDING A CLOSED COMMENT BEFORE AN OPENER',
    '[^1]: x\n    ```\n    <!-- a --> <!-- never\n    ```\n',
  ],
  [
    'A DISPLAY MATH RIGHT AFTER A FOOTNOTE DEFINITION AT THE TOP, HOLDING TEXT BEFORE AN OPENER',
    '[^1]: x\n    $$\n    x <!-- never\n    $$\n',
  ],
  [
    'A FENCE RIGHT AFTER A FOOTNOTE DEFINITION IN A BLOCK QUOTE, HOLDING AN OPENER',
    '> [^1]: x\n>     ```\n>     <!-- never\n>     ```\n',
  ],
  [
    'A FENCE UNDER A FOOTNOTE DEFINITION AT THE TOP, HOLDING AN OPENER, A FOOTNOTE FOR BOTH READERS',
    '[^1]: x\n\n    ```\n    <!-- never\n    ```\n',
  ],
  [
    'A FENCE UNDER A FOOTNOTE DEFINITION IN A LIST ITEM THAT HOLDS A LIST',
    '- [^1]: x\n  - y\n\n        ```\n        {a}\n        ```\n',
  ],
  [
    'A PARAGRAPH UNDER A FOOTNOTE DEFINITION IN A BLOCK QUOTE, HOLDING A BRACE',
    '> [^1]: x\n>\n>     second {b}\n',
  ],
];

await describe({
  name: 'a fence or a display math under a footnote definition, and an opener in it',
  children: readerEqualityCases({ cases: CASES, },),
},);
