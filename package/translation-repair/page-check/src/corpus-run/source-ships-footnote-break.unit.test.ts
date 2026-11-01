/**
 Tests that a hard break written with a backslash, before a line starting a footnote definition indented four columns or
 more, is shown as the source shows it.

 The strict grammar disables indented code, so a line `[^1]:` indented four columns or more after a paragraph starts a
 footnote definition there, where CommonMark and GFM read a lazy line of the paragraph. A backslash at the end of the
 line before is a hard break to a reader of the source, and a backslash and nothing after it on the page, where the
 footnote cut the paragraph: the page showed the backslash. A hard break reads as one space of white space, so the break,
 the line ending and the indentation are written as one space (`source-ships-lines.ts`), and the page shows the text
 without the backslash, as the source does.

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
    'A BACKSLASH HARD BREAK BEFORE A FOOTNOTE START INDENTED FOUR SPACES',
    'a\\\n    [^1]: y {d}\n',
  ],
  [
    'A BACKSLASH HARD BREAK BEFORE A FOOTNOTE START INDENTED BY A TAB, WITH A TAG AFTER THE LABEL',
    'a\\\n\t[^1]: <b> z\n',
  ],
  [
    'A BACKSLASH HARD BREAK BEFORE A FOOTNOTE START AFTER THE MARKS OF A BLOCK QUOTE',
    '> a\\\n>     [^1]: y {d}\n',
  ],
  [
    'A BACKSLASH HARD BREAK BEFORE A FOOTNOTE START UNDER THE TEXT OF A LIST ITEM',
    '- a\\\n      [^1]: y {d}\n',
  ],
  [
    'A BACKSLASH HARD BREAK IN EMPHASIS BEFORE A FOOTNOTE START INDENTED FOUR SPACES',
    'a *x\\\n    [^1]: y* z {d}\n',
  ],
  [
    'A BACKSLASH HARD BREAK BEFORE A FOOTNOTE START INDENTED THREE SPACES, WHICH BOTH READINGS CUT',
    'a\\\n   [^1]: y {d}\n',
  ],
  [
    'A HARD BREAK OF TWO SPACES BEFORE A FOOTNOTE START INDENTED FOUR SPACES',
    'a  \n    [^1]: y {d}\n',
  ],
];

await describe({
  name: 'a hard break before a footnote definition line indented four columns or more',
  children: readerEqualityCases({ cases: CASES, },),
},);
