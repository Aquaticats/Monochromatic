/**
 Tests that the blocks under a footnote definition in a container are the footnote's own when the lines of the
 definition's paragraph come first.

 A footnote definition in a block quote or a list item is read by the plain reading as a link reference definition
 followed by a paragraph, or as one paragraph, where the strict grammar reads one paragraph whose lazy lines are the lines
 right after the first. A fence or a display math indented four columns under it is a block of the footnote there, and a
 mark of it is no text, whether a lazy line comes before it or the first line holds a brace that keeps it from being
 a definition.

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
    'A FENCE AFTER A LAZY LINE OF A FOOTNOTE DEFINITION IN A LIST ITEM',
    '- [^1]: x\n  abc\n      ```\n',
  ],
  [
    'A FENCE HOLDING A BRACE AFTER A LAZY LINE OF A FOOTNOTE DEFINITION IN A BLOCK QUOTE',
    '> [^1]: x\n> abc\n>     ```\n>     {a}\n>     ```\n',
  ],
  [
    'A FENCE AFTER A LAZY LINE WITH A BRACE, THEN A LINE AFTER THE FENCE, IN A BLOCK QUOTE',
    '> [^1]: x\n> abc {\n>     ```\n>     ```\n> [r]: u{b}\n',
  ],
  [
    'A FENCE RIGHT AFTER A FOOTNOTE LINE THAT HOLDS AN UNCLOSED BRACE IN A LIST ITEM',
    '- [^1]: x {\n      ```\n',
  ],
  [
    'A FENCE HOLDING A TAG AND A BRACE AFTER A FOOTNOTE LINE THAT HOLDS A BRACE IN A BLOCK QUOTE',
    '> [^1]: x {\n>     ```js\n>     <i>\n>     {b}\n> ```\n',
  ],
  [
    'A FENCE OF TILDES HOLDING AN OPENER AFTER A LAZY LINE IN A BLOCK QUOTE',
    '> [^1]: x\n> abc\n>     ~~~\n>     <!-- never\n>     ~~~\n',
  ],
  [
    'A FENCE AFTER A LAZY LINE OF A FOOTNOTE THAT OPENS WITH A TAG AND A WORD IN A BLOCK QUOTE',
    '> [^1]: <b> abc\n>     ```\n>     a\n> ```\n>     tail {z}\n',
  ],
  [
    'A FOOTNOTE LINE, A TAG ALONE AND A LINE THAT LOOKS LIKE A DEFINITION BUT IS TEXT, IN A BLOCK QUOTE',
    '> [^1]: x\n> <a href="x">\n> [r]: u{b} z\n',
  ],
  [
    'A FENCE AFTER A BLANK LINE AFTER A LAZY LINE OF A FOOTNOTE DEFINITION IN A BLOCK QUOTE',
    '> [^1]: x\n> abc\n>\n>     ```\n>     {a}\n>     ```\n',
  ],
  [
    'A LAZY LINE AND A FENCE INDENTED THREE COLUMNS, WHICH IS NO BLOCK OF THE FOOTNOTE, IN A BLOCK QUOTE',
    '> [^1]: x\n> abc\n>    ```\n>    {a}\n>    ```\n',
  ],
];

await describe({
  name: 'the blocks under a footnote definition in a container after the lines of its paragraph',
  children: readerEqualityCases({ cases: CASES, },),
},);
