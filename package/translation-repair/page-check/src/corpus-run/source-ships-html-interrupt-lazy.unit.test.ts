/**
 Tests that a block of raw HTML at the start of a line right after a paragraph of a block quote or a list item cuts that
 paragraph on the page as it does for a reader of the source.

 A reader of the source takes a line that starts an HTML block of the kinds that may interrupt a paragraph for the start of
 that block, which ends the paragraph and with it the block quote or the list item that holds it when the line carries none
 of their marks (a lazy line cannot continue a paragraph an HTML block interrupts), so a backtick or a dollar sign in the
 paragraph that nothing closes opens no code span or math there. The page has no raw HTML, so the paragraph went on over
 the line and the backtick paired with one in the lines of the block. A blank line before the block ends the paragraph on
 the page where it ends for the reader of the source.

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
    'A BLOCK OF RAW HTML AT COLUMN ZERO AFTER A PARAGRAPH OF A BLOCK QUOTE THAT HOLDS A BACKTICK',
    '> q `x\n<div>\ny`\n',
  ],
  [
    'A BLOCK OF RAW HTML AT COLUMN ZERO AFTER A PARAGRAPH OF A LIST ITEM THAT HOLDS A BACKTICK',
    '- i `x\n<div>\ny`\n',
  ],
  [
    'A BLOCK OF RAW HTML AT COLUMN ZERO AFTER A PARAGRAPH OF AN ORDERED LIST ITEM THAT HOLDS A BACKTICK',
    '1. i `x\n<div>\ny`\n',
  ],
  [
    'A BLOCK OF RAW HTML AT COLUMN ZERO AFTER A PARAGRAPH OF A BLOCK QUOTE IN A BLOCK QUOTE',
    '> > q `x\n<div>\ny`\n',
  ],
  [
    'A BLOCK OF RAW HTML AT COLUMN ZERO AFTER THE LAST PARAGRAPH OF A BLOCK QUOTE OF TWO',
    '> a\n>\n> b `x\n<div>\ny`\n',
  ],
  [
    'A BLOCK OF RAW HTML AT COLUMN ZERO AFTER A PARAGRAPH OF A LIST ITEM IN A LIST ITEM',
    '- a\n  - b `x\n<div>\ny`\n',
  ],
  [
    'A BLOCK OF RAW HTML AT COLUMN ZERO AFTER A LIST IN A BLOCK QUOTE',
    '> a\n> - b `x\n<pre>\ny`\n',
  ],
  [
    'A BLOCK OF RAW HTML IN A BLOCK QUOTE AFTER A LIST ITEM THAT HOLDS A BACKTICK',
    '> - b `x\n> <pre>\n> y`\n',
  ],
  [
    'A BLOCK OF RAW HTML AT COLUMN ZERO AFTER A PARAGRAPH OF A BLOCK QUOTE THAT HOLDS A DOLLAR SIGN',
    '> q $x\n<script>\ny$\n',
  ],
  [
    'A BLOCK OF RAW HTML AT COLUMN ZERO AFTER A PARAGRAPH OF A BLOCK QUOTE THAT HOLDS A BRACE ONLY',
    '> q {x}\n<div>\ny\n',
  ],
  [
    'A BLOCK OF RAW HTML AT COLUMN ZERO AFTER A PARAGRAPH OF A BLOCK QUOTE THAT HOLDS NO BACKTICK',
    '> q\n<div>\ny\n',
  ],
  [
    'A BLOCK OF RAW HTML AFTER A BLANK LINE AFTER A PARAGRAPH OF A BLOCK QUOTE THAT HOLDS A BACKTICK',
    '> q `x\n\n<div>\ny`\n',
  ],
  [
    'A LINE OF TEXT AT COLUMN ZERO AFTER A PARAGRAPH OF A BLOCK QUOTE THAT HOLDS A BACKTICK, A LAZY LINE FOR BOTH',
    '> q `x\ny` z\n',
  ],
];

await describe({
  name: 'a block of raw HTML at column zero after a paragraph of a block quote or a list item',
  children: readerEqualityCases({ cases: CASES, },),
},);
