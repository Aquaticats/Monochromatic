/**
 Tests that a comment opener that never closes, standing in a fence or a display math that a footnote definition holds,
 is shown as the source shows it.

 The strict grammar disables indented code, so a fence or a display math indented four columns under a footnote
 definition is a fence or a display math of the footnote there, and the plain reading the encoder walks takes the lines
 for indented code, which has no opener to take out of its code. The opener was written as a character reference, which
 shows as written inside code, so the page showed `&lt;!--`. The lines that hold an opener are taken out of the block
 as they are for a fence at the top (`source-ships-opener-in-code.ts`): the block is closed before the line, the line is
 written as a paragraph of the footnote, and the block is opened again with the same mark
 (`source-ships-opener-footnote.ts`).

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
    'AN OPENER IN A FENCE OF BACKTICKS INDENTED UNDER A FOOTNOTE DEFINITION',
    '[^1]: x\n\n    ```\n    <!-- never\n    ```\n',
  ],
  [
    'AN OPENER IN A DISPLAY MATH INDENTED UNDER A FOOTNOTE DEFINITION',
    '[^1]: x\n\n    $$\n    <!-- never\n    $$\n',
  ],
  [
    'AN OPENER IN A FENCE OF BACKTICKS WITH AN INFO STRING THAT THE FOOTNOTE LEAVES OPEN',
    '[^1]: x\n\n    ```js\n    <!-- never\n',
  ],
  [
    'TWO OPENERS ON TWO LINES OF A FENCE OF TILDES UNDER A FOOTNOTE DEFINITION',
    '[^1]: x\n\n    ~~~\n    a <!-- one\n    b <!-- two\n    ~~~\n',
  ],
  [
    'AN OPENER IN A FENCE INDENTED BY A TAB UNDER A FOOTNOTE DEFINITION',
    '[^1]: x\n\n\t```\n\t<!-- never\n\t```\n',
  ],
  [
    'AN OPENER IN A FENCE UNDER A FOOTNOTE DEFINITION, WITH A BRACE IN THE PARAGRAPH AFTER IT',
    '[^1]: x\n\n    ```\n    <!-- never\n    ```\n\n    after {d}\n',
  ],
  [
    'AN OPENER ON THE SECOND LINE OF A DISPLAY MATH UNDER A FOOTNOTE DEFINITION',
    '[^1]: x\n\n    $$\n    a\n    <!-- never\n    $$\n',
  ],
  [
    'AN OPENER IN A PARAGRAPH AFTER A CLOSED FENCE UNDER A FOOTNOTE DEFINITION',
    '[^1]: x\n\n    ```\n    a\n    ```\n\n    p <!-- never\n',
  ],
];

await describe({
  name: 'a comment opener in a fence or display math under a footnote definition',
  children: readerEqualityCases({ cases: CASES, },),
},);
