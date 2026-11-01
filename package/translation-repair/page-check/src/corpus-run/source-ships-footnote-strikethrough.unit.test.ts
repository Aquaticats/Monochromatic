/**
 Tests that strikethrough that runs across a line starting a footnote definition, indented four columns or more, is shown
 whole.

 The strict grammar disables indented code, so a line `[^1]:` indented four columns or more after a paragraph starts a
 footnote definition there, where CommonMark and GFM read a lazy line of the paragraph. The strikethrough the original
 pairs across that line was cut there on the page, and the page showed the tildes that opened it. A line ending and the
 indentation after it are white space in strikethrough as they are in emphasis, so the line is joined to the one before
 it (`source-ships-lines.ts`, `source-ships-footnote-emphasis.unit.test.ts`), and the page shows what the original shows.

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
    'STRIKETHROUGH CUT BY A FOOTNOTE START INDENTED FOUR SPACES',
    'a ~~x\n    [^1]: y~~ z {d}\n',
  ],
  [
    'STRIKETHROUGH OF ONE TILDE CUT BY A FOOTNOTE START INDENTED FOUR SPACES',
    'a ~x\n    [^1]: y~ z {d}\n',
  ],
  [
    'STRIKETHROUGH CUT BY A FOOTNOTE START INDENTED FOUR SPACES, WITH A TAG AFTER THE LABEL',
    'a ~~x\n    [^1]: <b>~~ z\n',
  ],
  [
    'STRIKETHROUGH CUT BY A FOOTNOTE START INDENTED BY A TAB',
    'a ~~x\n\t[^1]: y~~ z {d}\n',
  ],
  [
    'STRIKETHROUGH CUT BY A FOOTNOTE START INDENTED FOUR SPACES ON A LAZY LINE OF A BLOCK QUOTE',
    '> a ~~x\n    [^1]: y~~ z {d}\n',
  ],
  [
    'STRIKETHROUGH CUT BY A FOOTNOTE START INDENTED SIX SPACES UNDER THE TEXT OF A LIST ITEM',
    '- a ~~x\n      [^1]: y~~ z {d}\n',
  ],
  [
    'STRIKETHROUGH HOLDING A CODE SPAN, BOTH CUT BY A FOOTNOTE START INDENTED FOUR SPACES',
    'a ~~x `y\n    [^1]: z` w~~ v {d}\n',
  ],
  [
    'STRIKETHROUGH HOLDING EMPHASIS, BOTH CUT BY A FOOTNOTE START INDENTED FOUR SPACES',
    'a ~~x *y\n    [^1]: z* w~~ v {d}\n',
  ],
  [
    'STRIKETHROUGH THAT A FOOTNOTE START INDENTED THREE SPACES CUTS FOR BOTH READINGS',
    'a ~~x\n   [^1]: y~~ z {d}\n',
  ],
  [
    'STRIKETHROUGH ON ONE LINE BEFORE A FOOTNOTE START INDENTED FOUR SPACES',
    'a ~~x~~ b\n    [^1]: y z {d}\n',
  ],
];

await describe({
  name: 'strikethrough across a footnote definition line indented four columns or more',
  children: readerEqualityCases({ cases: CASES, },),
},);
