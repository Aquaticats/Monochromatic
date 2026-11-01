/**
 Tests that a fence mark indented four columns or more after a line of text shows on the page as it does in the source.

 The plain reading takes such a mark for text, or for the delimiter of a code span, and the strict grammar, which has no
 indented code, opens a fence at it. The mark is written with a backslash before its first character, which the page reads
 as text and which leaves the run of marks whole: a backslash before each character cut the run into runs of one, and a
 backtick on a line before it then paired with the first and made a code span of the lines between that showed a backslash.
 After a link reference definition the mark that opens a code span cannot be joined to the line before it, so the space
 that indents it is written as a character reference.

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
    'A LONE BACKTICK ON A LINE, THEN A FENCE MARK INDENTED FOUR COLUMNS',
    '`\n    ```\n',
  ],
  [
    'A LONE BACKTICK AFTER TEXT, THEN A FENCE MARK INDENTED FOUR COLUMNS AND A LINE',
    'a `\n    ```\nx\n',
  ],
  [
    'A LONE BACKTICK ON A LINE, THEN A FENCE MARK INDENTED WITH A TAB',
    '`\n\t```\n',
  ],
  [
    'A LONE BACKTICK ON A LINE, THEN A FENCE MARK INDENTED FOUR COLUMNS, WITH CARRIAGE RETURNS',
    '`\r    ```\r',
  ],
  [
    'A TWO-BACKTICK RUN ON A LINE, THEN A FENCE MARK INDENTED FOUR COLUMNS',
    '``\n    ```\n',
  ],
  [
    'A LONE BACKTICK IN A BLOCK QUOTE, THEN A FENCE MARK INDENTED FOUR COLUMNS',
    '> `\n>     ```\n',
  ],
  [
    'A LONE BACKTICK IN A LIST ITEM, THEN A FENCE MARK INDENTED FOUR COLUMNS',
    '- `\n      ```\n',
  ],
  [
    'A FENCE MARK OF FIVE BACKTICKS INDENTED FOUR COLUMNS AFTER A LONE BACKTICK',
    '`\n    `````\n',
  ],
  [
    'A FENCE MARK INDENTED FOUR COLUMNS RIGHT AFTER A LINK REFERENCE DEFINITION, THEN A BACKTICK RUN',
    '[b]:x\n    ```\na```\n',
  ],
  [
    'A LONE BACKTICK ON THE LINE OF A FOOTNOTE DEFINITION IN A BLOCK QUOTE, THEN A LINE WITHOUT THE MARK OF THE QUOTE AND A '
      + 'FENCE MARK INDENTED FOUR COLUMNS',
    '>x\n>[^1]:`\n    ```',
  ],
  [
    'A LONE BACKTICK ON THE LINE OF A FOOTNOTE DEFINITION IN A BLOCK QUOTE, THEN A LINE WITHOUT THE MARK OF THE QUOTE AND A '
      + 'FENCE MARK INDENTED WITH A TAB',
    '>x\n>[^1]:`\r\t```',
  ],
  [
    'A FENCE MARK OF TILDES INDENTED FOUR COLUMNS AFTER A LONE BACKTICK',
    '`\n    ~~~\n',
  ],
  [
    'A FENCE MARK INDENTED FOUR COLUMNS AFTER A LINE OF TEXT',
    'x\n    ```\ny\n',
  ],
];

await describe({
  name: 'a fence mark indented four columns after a line',
  children: readerEqualityCases({ cases: CASES, },),
},);
