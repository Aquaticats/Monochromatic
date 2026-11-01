/**
 Tests that a code span, inline math or the title of a link that runs across a line starting a footnote definition, indented
 four columns or more, is shown whole.

 The strict grammar disables indented code, so a line `[^1]:` indented four columns or more after a paragraph starts a
 footnote definition there, where CommonMark and GFM read a lazy line of the paragraph. The span the original pairs across
 that line was cut by the definition on the page, and the page showed the backtick, the dollar sign or the quote that
 opened it. A line ending and the indentation after it are one space inside a span and white space inside a title on
 any render, so the line is joined to the one before it, as it is for a brace, a tag or an indented fence mark
 (`source-ships-lines.ts`), and the page shows the span the original shows.

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
    'A CODE SPAN CUT BY A FOOTNOTE START INDENTED FOUR SPACES, WITH PLAIN TEXT AFTER THE LABEL',
    'a `x\n    [^1]: y` z {d}\n',
  ],
  [
    'A CODE SPAN CUT BY A FOOTNOTE START INDENTED FOUR SPACES, WITH A TAG AFTER THE LABEL',
    'a `x\n    [^1]: <b>` z {d}\n',
  ],
  [
    'A CODE SPAN CUT BY A FOOTNOTE START INDENTED FOUR SPACES, WITH A BRACE AFTER THE LABEL',
    'a `x\n    [^1]: {c}` z {d}\n',
  ],
  [
    'A CODE SPAN CUT BY A FOOTNOTE START INDENTED FIVE SPACES',
    'a `x\n     [^1]: y` z {d}\n',
  ],
  [
    'A CODE SPAN CUT BY A FOOTNOTE START INDENTED BY A TAB',
    'a `x\n\t[^1]: y` z {d}\n',
  ],
  [
    'A CODE SPAN CUT BY A FOOTNOTE START INDENTED BY TWO SPACES AND A TAB',
    'a `x\n  \t[^1]: <b>` z {d}\n',
  ],
  [
    'A CODE SPAN CUT BY A FOOTNOTE START INDENTED FOUR SPACES ON A LAZY LINE OF A BLOCK QUOTE',
    '> a `x\n    [^1]: y` z {d}\n',
  ],
  [
    'A CODE SPAN CUT BY A FOOTNOTE START INDENTED FOUR SPACES AFTER THE MARKS OF A BLOCK QUOTE',
    '> a `x\n>     [^1]: <b>` z {d}\n',
  ],
  [
    'A CODE SPAN CUT BY A FOOTNOTE START INDENTED SIX SPACES UNDER THE TEXT OF A LIST ITEM',
    '- a `x\n      [^1]: y` z {d}\n',
  ],
  [
    'INLINE MATH CUT BY A FOOTNOTE START INDENTED FOUR SPACES',
    'a $x\n    [^1]: y$ z {d}\n',
  ],
  [
    'INLINE MATH CUT BY A FOOTNOTE START INDENTED FOUR SPACES, WITH A TAG AFTER THE LABEL',
    'a $x\n    [^1]: <b>$ z {d}\n',
  ],
  [
    'THE TITLE OF A LINK CUT BY A FOOTNOTE START INDENTED FOUR SPACES',
    'a [t](u "x\n    [^1]: y") z {d}\n',
  ],
  [
    'THE TITLE OF A LINK CUT BY A FOOTNOTE START INDENTED BY A TAB, WITH A BRACE AFTER THE LABEL',
    'a [t](u "x\n\t[^1]: {c}") z {d}\n',
  ],
  [
    'A CODE SPAN CUT BY TWO FOOTNOTE STARTS INDENTED FOUR SPACES, THE SECOND INSIDE THE SPAN TOO',
    'a `x\n    [^1]: y\n    [^2]: z` w {d}\n',
  ],
  [
    'A CODE SPAN THAT A FOOTNOTE START INDENTED THREE SPACES CUTS FOR BOTH READINGS',
    'a `x\n   [^1]: y` z {d}\n',
  ],
];

await describe({
  name: 'a span across a footnote definition line indented four columns or more',
  children: readerEqualityCases({ cases: CASES, },),
},);
