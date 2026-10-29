/**
 Tests that a footnote definition in a container, beside a table, a comment opener or a raw HTML block, shows on the page
 as it does in the source.

 These are the minimal texts of the re-check of the containers: each one read the same on the page before the footnote
 readings of the encoder were changed, or was not compared, and differed after. The causes are four. A comment opener on the
 label line of the definition is written as a character reference before the definition is asked whether it opens with
 raw HTML. The nested reading takes a table header on the first line of a footnote body, and the plain reading takes a
 footnote label line right after a table row as a row of that table. A line such as `<!r` starts a block of raw HTML that
 holds the lines of the footnote after it. And a quote mark left unescaped in a deep quote shows as a quote on the page.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { describe, } from '@monochromatic-dev/module-test/ts';

import { sameReadingCases, } from './source-reading-same.test-fixture.ts';

/**
 Texts a reader must see as the source shows them: what the case is, and the text.
 */
const CASES: readonly (readonly [string, string])[] = [
  [
    'A COMMENT OPENER ON THE LABEL LINE OF A DEFINITION IN A QUOTE, THEN A FENCE MARK INDENTED FOUR COLUMNS',
    '>[^1]:<!-- r\n>     ```',
  ],
  [
    'A COMMENT OPENER AFTER A BLANK LINE OF A DEFINITION IN A QUOTE, THEN A DISPLAY MATH MARK',
    '>[^1]:\n>\n>     <!--\n>     $$',
  ],
  [
    'A DECLARATION-LIKE LINE, THEN A DEFINITION WITH A FENCE MARK AND A COMMENT OPENER',
    '<!r\n[^1]:\n    ```\n    <!--',
  ],
  [
    'A DEFINITION WHOSE LABEL LINE HOLDS A CODE SPAN AND A TABLE HEADER, THEN A DELIMITER ROW',
    '[^1]:`{||\n-|-\n`',
  ],
  [
    'A TAG, THEN A DEFINITION WITH A DISPLAY MATH MARK AND A COMMENT OPENER',
    '<v>\n[^1]:\n    $$\n    <!--',
  ],
  [
    'A COMMENT OPENER THAT CLOSES AT A QUOTE MARK ON THE LABEL LINE OF A DEFINITION IN A QUOTE, THEN A FENCE MARK',
    '>[^1]:<!-- >\n>     ```',
  ],
  [
    'A TABLE IN THE LABEL LINE OF A DEFINITION IN A QUOTE, THEN A DISPLAY MATH MARK',
    '>[^1]:||\n>-|-\n>     $$',
  ],
  [
    'A DECLARATION-LIKE LINE, THEN A DEFINITION WITH A DISPLAY MATH MARK AND A COMMENT OPENER',
    '<!r\n[^1]:\n    $$\n    <!--',
  ],
  [
    'A TABLE IN A LIST ITEM, THEN A DEFINITION WITH A BACKTICK INDENTED BY A TAB',
    '* a||\n  -|-\n\t[^1]:`\n      `',
  ],
  [
    'A DEFINITION IN A LIST ITEM, A BLANK LINE, THEN A COMMENT OPENER AND A DISPLAY MATH MARK',
    '1. [^1]:\n\n       <!--\n       $$',
  ],
  [
    'A TABLE, THEN A DEFINITION LABEL LINE AND A DISPLAY MATH MARK',
    'a||\n-|-\n[^1]:\n    $$',
  ],
  [
    'A COMMENT OPENER ON THE LABEL LINE OF A DEFINITION IN A QUOTE, THEN A BLANK QUOTE LINE AND A FENCE MARK',
    '>[^1]:<!r\n>\n>     ```',
  ],
  [
    'A DEFINITION WHOSE LABEL LINE HOLDS A BRACE, A CODE SPAN AND A TABLE HEADER',
    '[^1]:`}||\n-|-\n`',
  ],
  [
    'A DEFINITION LABEL LINE, THEN A TABLE ROW WITH A CODE SPAN OPENING',
    '[^1]:\n-|\n`<\n`',
  ],
  [
    'A TABLE IN A LIST ITEM, THEN A DEFINITION WITH A BACKTICK INDENTED BY A TAB AND A BACKTICK',
    '1. a||\n   -|-\n\t[^1]:`\n       `',
  ],
  [
    'A TABLE, THEN A DEFINITION LABEL LINE AND A FENCE MARK',
    'a||\n-|-\n[^1]:\n    ```',
  ],
  [
    'A COMMENT OPENER ON THE LABEL LINE OF A DEFINITION IN A LIST ITEM, THEN A FENCE MARK',
    '- [^1]:<!-- r\n      ```',
  ],
  [
    'INDENTED DISPLAY MATH WITH A COMMENT OPENER RIGHT AFTER A LINK REFERENCE DEFINITION, THEN A FOOTNOTE DEFINITION',
    '[r]:u\n    $$\n    <!--\n    $$\n[^1]:',
  ],
  [
    'INDENTED DISPLAY MATH WITH A COMMENT OPENER RIGHT AFTER A LINK REFERENCE DEFINITION, THEN A FOOTNOTE WITH TEXT',
    '[r]:u\n    $$\n    <!--\n    $$\n[^1]: x',
  ],
  [
    'A BACKTICK, A DEFINITION OF ANOTHER LABEL INDENTED FOUR COLUMNS, THEN A TABLE AND A BACKTICK',
    '`\n    [^2]: w\na||\n-|-\n`',
  ],
  [
    'A DEFINITION LABEL LINE, THEN A DELIMITER ROW LIKE LINE, A BACKTICK AND A BRACE',
    '[^1]:\n|-\n`\n{`',
  ],
  [
    'A DEFINITION WHOSE LABEL LINE HOLDS A BACKTICK AND A BRACE, THEN A DELIMITER ROW LIKE LINE',
    '[^1]:`{\n-|\n`',
  ],
  [
    'A DEFINITION LABEL LINE, THEN A DELIMITER ROW LIKE LINE WITH A CODE SPAN',
    '[^1]:\n-|\n`|{`',
  ],
  [
    'A DEFINITION WHOSE LABEL LINE HOLDS A BACKTICK, A LESS-THAN SIGN AND A TABLE HEADER',
    '[^1]:`<||\n-|-\n`',
  ],
  [
    'A DEFINITION IN A LIST ITEM, A BLANK LINE, THEN A COMMENT OPENER AND A DISPLAY MATH MARK IN A BULLET ITEM',
    '* [^1]:\n\n      <!--\n      $$',
  ],
  [
    'A COMMENT OPENER ON THE LABEL LINE OF A DEFINITION IN A NUMBERED ITEM, THEN A FENCE MARK',
    '1. [^1]:<!-- r\n       ```',
  ],
  [
    'A BRACE ON THE LABEL LINE OF A DEFINITION, A RAW HTML TAG, THEN A FENCE MARK AND A COMMENT OPENER',
    '[^1]:}\n<div\n    ```\n    <!--',
  ],
];

await describe({
  name: 'a footnote definition in a container beside a table or a comment opener',
  children: sameReadingCases({ cases: CASES, },),
},);
