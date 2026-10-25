/**
 Tests that a line `[^label]:` indented four columns or more right after a link reference definition is shown as the
 source shows it.

 CommonMark and GFM read that line as a second link reference definition, which shows nothing, because the definition before
 it keeps the paragraph going and a definition may start on any of its lines. The strict grammar disables indented code, so
 it starts a footnote definition at the line, and the footnote shows its label and its text. The encoder writes the caret
 of the label with a backslash there, so that the page reads a link reference definition of another label, which shows
 nothing either, and a brace or a tag after the colon stays the destination of a definition.

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
    'A FOOTNOTE LABEL INDENTED FOUR SPACES RIGHT AFTER A LINK REFERENCE DEFINITION',
    '[r]: u\n    [^1]: y\n',
  ],
  [
    'A FOOTNOTE LABEL INDENTED FOUR SPACES RIGHT AFTER A LINK REFERENCE DEFINITION, WITH A BRACE AFTER THE COLON',
    '[r]: u\n    [^1]: {c}\n',
  ],
  [
    'A FOOTNOTE LABEL INDENTED FOUR SPACES RIGHT AFTER A LINK REFERENCE DEFINITION, WITH A TAG AFTER THE COLON',
    '[r]: u\n    [^1]: <b>\n',
  ],
  [
    'A FOOTNOTE LABEL INDENTED BY A TAB RIGHT AFTER A LINK REFERENCE DEFINITION',
    '[r]: u\n\t[^1]: y\n',
  ],
  [
    'A FOOTNOTE LABEL INDENTED SIX SPACES RIGHT AFTER A LINK REFERENCE DEFINITION AND A PARAGRAPH BEHIND IT',
    '[r]: u\n      [^1]: y\n\nnext {n}\n',
  ],
  [
    'A FOOTNOTE LABEL INDENTED FOUR SPACES AFTER TWO LINK REFERENCE DEFINITIONS',
    '[r]: u\n[s]: v\n    [^1]: y\n',
  ],
  [
    'A FOOTNOTE LABEL INDENTED FOUR SPACES RIGHT AFTER A LINK REFERENCE DEFINITION IN A BLOCK QUOTE',
    '> [r]: u\n>     [^1]: y\n',
  ],
  [
    'A FOOTNOTE LABEL INDENTED FOUR SPACES RIGHT AFTER A LINK REFERENCE DEFINITION IN A LIST ITEM',
    '- [r]: u\n      [^1]: y\n',
  ],
  [
    'A FOOTNOTE LABEL INDENTED FOUR SPACES RIGHT AFTER A LINK REFERENCE DEFINITION, THEN INDENTED CODE WITH A BRACE',
    '[r]: u\n    [^1]: y\n\n    code {c}\n',
  ],
  [
    'A FOOTNOTE LABEL AT THE START OF A LINE RIGHT AFTER A LINK REFERENCE DEFINITION, A FOOTNOTE FOR BOTH READINGS',
    '[r]: u\n[^1]: y {d}\n',
  ],
  [
    'A FOOTNOTE LABEL INDENTED THREE SPACES RIGHT AFTER A LINK REFERENCE DEFINITION, A FOOTNOTE FOR BOTH READINGS',
    '[r]: u\n   [^1]: y {d}\n',
  ],
  [
    'A FOOTNOTE LABEL INDENTED FOUR SPACES AFTER A PARAGRAPH, A FOOTNOTE ONLY THE PAGE STARTS',
    'a\n    [^1]: y {d}\n',
  ],
];

await describe({
  name: 'a footnote label indented four columns or more right after a link reference definition',
  children: readerEqualityCases({ cases: CASES, },),
},);
