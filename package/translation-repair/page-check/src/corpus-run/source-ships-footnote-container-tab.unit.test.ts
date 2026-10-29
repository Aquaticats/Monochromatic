/**
 Tests that a tab where a block quote or a list item puts a space before a footnote's blocks is read as that space.

 A footnote definition in a container gives its later lines up to the container as the marks of the quote and the white
 space that stands where the pattern of the container has a space. The white space may be a tab, which counts for the
 pattern's space the same as a space does; a reader that matched spaces alone would take one column fewer off each
 line and read the fence marks and the display math of the footnote's body wrongly.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { describe, } from '@monochromatic-dev/module-test/ts';

import { sameReadingCases, } from './source-reading-same.test-fixture.ts';

/**
 Texts whose lines after a footnote's first carry a tab where the container's pattern has a space: what the case is, and
 the text.
 */
const TAB_CASES: readonly (readonly [string, string])[] = [
  [
    'A TAB AFTER THE QUOTE MARK, THEN A FENCE MARK INDENTED FOUR COLUMNS AND A COMMENT OPENER',
    '> [^1]: x\n>\t    ```\n>\t    <!-- never\n>\t    ```\n',
  ],
  [
    'A BLANK QUOTE LINE, THEN A TAB AFTER THE QUOTE MARK AND A FENCE MARK INDENTED FOUR COLUMNS',
    '> [^1]: x\n>\n>\t    ```\n>\t    <!-- never\n>\t    ```\n',
  ],
  [
    'A BLANK QUOTE LINE, THEN A TAB AFTER THE QUOTE MARK AND A DISPLAY MATH MARK INDENTED FOUR COLUMNS',
    '> [^1]: x\n>\n>\t    $$\n>\t    <!-- never\n>\t    $$\n',
  ],
];

await describe({
  name: 'a tab where the container puts a space before the blocks of a footnote',
  children: sameReadingCases({ cases: TAB_CASES, },),
},);
