/**
 Tests that an indented fence that no footnote definition holds is not taken for one of a footnote when an opener stands
 in it.

 A fence or a display math indented four columns is a fence of a footnote only under a footnote definition. Elsewhere the
 plain reading takes the lines for indented code and the encoder writes every character of them as text, the opener as a
 character reference (`source-ships-opener-footnote.ts` reads only the footnotes' ranges). A footnote that stands after the
 lines, or in another place of the text, must not make the lines its own.

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
    'AN INDENTED FENCE WITH AN OPENER AT THE TOP, NO FOOTNOTE IN THE TEXT',
    '    ```\n    <!-- never\n    ```\n',
  ],
  [
    'AN INDENTED FENCE WITH AN OPENER AT THE TOP, A FOOTNOTE AFTER IT',
    '    ```\n    <!-- never\n    ```\n\n[^2]: w\n',
  ],
  [
    'AN INDENTED FENCE WITH AN OPENER AT THE TOP, A FOOTNOTE BEFORE IT AFTER A PARAGRAPH',
    '[^2]: w\n\nthen\n\n    ```\n    <!-- never\n    ```\n',
  ],
  [
    'AN INDENTED FENCE WITH AN OPENER IN A BLOCK QUOTE, NO FOOTNOTE IN THE TEXT',
    '>     ```\n>     <!-- never\n>     ```\n',
  ],
  [
    'AN INDENTED DISPLAY MATH WITH AN OPENER AT THE TOP, A FOOTNOTE AFTER IT',
    '    $$\n    <!-- never\n    $$\n\n[^2]: w\n',
  ],
];

await describe({
  name: 'an indented fence with an opener that no footnote definition holds',
  children: readerEqualityCases({ cases: CASES, },),
},);
