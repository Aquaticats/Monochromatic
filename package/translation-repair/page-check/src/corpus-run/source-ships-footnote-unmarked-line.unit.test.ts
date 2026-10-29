/**
 Tests that a line of a block quote that carries no quote mark is a lazy line of the paragraph above it, not a line of the
 footnote the paragraph opens.

 A line without the mark of the block quote goes on with the paragraph only as text, and a line that would start a block
 of the footnote with the mark ends the quote. The reader of the source reads those lines as text of the footnote's
 paragraph, a fence mark among them, where the strict grammar of the page would open a fence, so the encoder keeps
 such a line as it stands, and joins it to the line before it where its mark would open one.

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
    'A FENCE MARK ON A LINE WITHOUT THE QUOTE MARK, AFTER A LAZY DEFINITION LINE AND A CODE SPAN THAT NOTHING CLOSES',
    '> [^1]: 注\n[b]: u\n`x\n    ```\n    y`\n',
  ],
  [
    'A FENCE MARK ON A LINE WITHOUT THE QUOTE MARK, AFTER A LAZY DEFINITION LINE AND A LAZY WORD',
    '> [^1]: 注\n[b]: u\nabc\n    ```\n',
  ],
  [
    'A FENCE MARK ON A LINE WITHOUT THE QUOTE MARK, RIGHT AFTER A FOOTNOTE LINE WITH NO SPACE AFTER THE QUOTE MARK',
    '>[^1]:注\n    ```',
  ],
  [
    'A FENCE MARK ON A LINE WITHOUT THE QUOTE MARK, RIGHT AFTER A FOOTNOTE LINE THAT HOLDS BACKTICKS',
    '>[^1]:``\n    ```',
  ],
  [
    'A FENCE MARK ON A LINE WITHOUT THE QUOTE MARK, THE LINES SEPARATED BY LONE CARRIAGE RETURNS',
    '> [^1]: 注\n[b]: u\r`x\n    ```\n    y`\n',
  ],
  [
    'A LINE WITH THE QUOTE MARK AFTER A FOOTNOTE LINE THAT HOLDS A DOLLAR SIGN AND A BRACE',
    '>[^1]:> $\r>{$',
  ],
  [
    'A LINE OF A FOOTNOTE WITH A DOUBLED COLON AND A LINE WITH A BRACE AND NO CLOSING BRACE',
    '>[^b]:: >\n >[r]:{',
  ],
  [
    'A FENCE UNDER A FOOTNOTE IN A BLOCK QUOTE, EACH LINE WITH THE QUOTE MARK',
    '> [^1]: x\n>     ```\n>     {a}\n>     ```\n',
  ],
  [
    'A FENCE MARK ON A LINE WITHOUT THE QUOTE MARK, THREE COLUMNS IN, AFTER A LAZY WORD',
    '> [^1]: x\nabc\n   ```\n',
  ],
];

await describe({
  name: 'a line of a block quote without the quote mark after a footnote definition',
  children: readerEqualityCases({ cases: CASES, },),
},);
