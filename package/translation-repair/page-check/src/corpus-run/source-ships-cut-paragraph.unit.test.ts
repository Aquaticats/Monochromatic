/**
 Tests that a paragraph a footnote start cuts, whose first line is indented four spaces or more, is read as the paragraph it is.

 A line indented four spaces or more that follows a link reference definition opens a paragraph to a reader of the source (indented
 code cannot follow a definition there, as it cannot interrupt a paragraph), and the node of that paragraph starts after the
 indentation. The lines before a footnote start are read again as a text of their own, and read alone the line starts at the
 text's first column: a run of backticks there is the opening of a fence and the lines after it the code of that fence, which
 the encoder lengthened at the opening only, so the page showed the rest of the text as the code of a fence that never closes.
 The piece is read as the paragraph it was.

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
    'A CODE SPAN OF FENCE MARKS INDENTED FOUR SPACES AFTER A DEFINITION, CUT BY A FOOTNOTE',
    '[^1]: x\n<a>\n[r]: u\n    ```\n    {\n    ```\n[^2]: a\n',
  ],
  [
    'THE SAME WITH A BRACE IN THE DEFINITION AND A LABEL OF A WORD',
    '[^1]: <b>\n[r]: u{b}\n    ```\n    {\n    ```\n[^note]: <b>\n',
  ],
  [
    'THE SAME WITH TILDES',
    '[^1]: x\n<a>\n[r]: u\n    ~~~\n    {\n    ~~~\n[^2]: a\n',
  ],
  [
    'THE SAME INDENTED BY A TAB',
    '[^1]: x\n<a>\n[r]: u\n\t```\n\t{\n\t```\n[^2]: a\n',
  ],
];

await describe({
  name: 'the page after a paragraph a footnote start cuts, whose first line is indented',
  children: readerEqualityCases({ cases: CASES, },),
},);
