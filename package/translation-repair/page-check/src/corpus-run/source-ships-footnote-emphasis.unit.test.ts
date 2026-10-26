/**
 Tests that emphasis, strong emphasis, the text of a link and the alternative text of an image that run across a line
 starting a footnote definition, indented four columns or more, are shown whole.

 The strict grammar disables indented code, so a line `[^1]:` indented four columns or more after a paragraph starts a
 footnote definition there, where CommonMark and GFM read a lazy line of the paragraph. The emphasis, the link text or the
 alternative text the original pairs across that line was cut there on the page, and the page showed the asterisk or the
 bracket that opened it. A line ending and the indentation after it are white space in all of them on any render, so the
 line is joined to the one before it, as it is for a code span, inline math or a link title
 (`source-ships-lines.ts`, `source-ships-footnote-code-span.unit.test.ts`), and the page shows what the original shows.

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
    'EMPHASIS CUT BY A FOOTNOTE START INDENTED FOUR SPACES',
    'a *x\n    [^1]: y* z {d}\n',
  ],
  [
    'EMPHASIS WITH UNDERSCORES CUT BY A FOOTNOTE START INDENTED FOUR SPACES',
    'a _x\n    [^1]: y_ z {d}\n',
  ],
  [
    'STRONG EMPHASIS CUT BY A FOOTNOTE START INDENTED FOUR SPACES',
    'a **x\n    [^1]: y** z {d}\n',
  ],
  [
    'EMPHASIS CUT BY A FOOTNOTE START INDENTED FOUR SPACES, WITH A TAG AFTER THE LABEL',
    'a *x\n    [^1]: <b>* z {d}\n',
  ],
  [
    'EMPHASIS CUT BY A FOOTNOTE START INDENTED BY A TAB',
    'a *x\n\t[^1]: y* z {d}\n',
  ],
  [
    'EMPHASIS CUT BY A FOOTNOTE START INDENTED FOUR SPACES ON A LAZY LINE OF A BLOCK QUOTE',
    '> a *x\n    [^1]: y* z {d}\n',
  ],
  [
    'EMPHASIS CUT BY A FOOTNOTE START INDENTED SIX SPACES UNDER THE TEXT OF A LIST ITEM',
    '- a *x\n      [^1]: y* z {d}\n',
  ],
  [
    'EMPHASIS HOLDING A CODE SPAN, BOTH CUT BY A FOOTNOTE START INDENTED FOUR SPACES',
    'a *x `y\n    [^1]: z` w* v {d}\n',
  ],
  [
    'THE TEXT OF A LINK CUT BY A FOOTNOTE START INDENTED FOUR SPACES',
    'a [t\n    [^1]: y](u) z {d}\n',
  ],
  [
    'THE TEXT OF A LINK CUT BY A FOOTNOTE START INDENTED FOUR SPACES, WITH A TITLE',
    'a [t\n    [^1]: y](u "v") z {d}\n',
  ],
  [
    'THE TEXT OF A LINK REFERENCE CUT BY A FOOTNOTE START INDENTED FOUR SPACES',
    'a [t\n    [^1]: y][r] z {d}\n\n[r]: u\n',
  ],
  [
    'THE ALTERNATIVE TEXT OF AN IMAGE CUT BY A FOOTNOTE START INDENTED FOUR SPACES',
    'a ![t\n    [^1]: y](u) z {d}\n',
  ],
  [
    'THE ALTERNATIVE TEXT OF AN IMAGE REFERENCE CUT BY A FOOTNOTE START INDENTED FOUR SPACES',
    'a ![t\n    [^1]: y][r] z {d}\n\n[r]: u\n',
  ],
  [
    'EMPHASIS THAT A FOOTNOTE START INDENTED THREE SPACES CUTS FOR BOTH READINGS',
    'a *x\n   [^1]: y* z {d}\n',
  ],
];

await describe({
  name: 'emphasis and link text across a footnote definition line indented four columns or more',
  children: readerEqualityCases({ cases: CASES, },),
},);
