/**
 Tests that a block of raw HTML indented by a tab inside a list item is shown as the source shows it.

 A tab that a list item's indentation takes in part is read by the parse as spaces: the block's text starts with the spaces
 the tab leaves, which the text of the source does not hold, and the block's position is that of its first character. The
 encoder took the spaces for characters of the source, so it looked for an opener of a comment that never closes, and for
 the lines of the block, two characters off, and wrote the block's marks as text.

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
    'AN OPENER FIRST ON A LINE INDENTED BY A TAB IN A LIST ITEM, AFTER A PARAGRAPH THAT HOLDS A BACKTICK',
    '- i `x\n\t<!-- c\ny`\n',
  ],
  [
    'AN OPENER FIRST ON A LINE INDENTED BY A TAB IN A LIST ITEM IN A BLOCK QUOTE, AFTER A PARAGRAPH THAT HOLDS A BACKTICK',
    '> - i `x\n>\t<!-- c\n> y`\n',
  ],
  [
    'AN ELEMENT OF RAW HTML INDENTED BY A TAB IN A LIST ITEM IN TWO BLOCK QUOTES, A BLANK LINE OF THE QUOTES AFTER IT',
    '> > - i\n> > \t<pre>\n> > \n> > ',
  ],
  [
    'AN ELEMENT OF RAW HTML INDENTED BY A TAB IN A LIST ITEM IN A LIST ITEM IN A BLOCK QUOTE, THEN A PARAGRAPH',
    '> - - i\n>   \t<script>\n>   \n>   z\n>   ',
  ],
  [
    'AN ELEMENT OF RAW HTML INDENTED BY A TAB IN A LIST ITEM, A PARAGRAPH AT COLUMN ZERO AFTER IT',
    '- i\n\t<pre>\nz\n',
  ],
  [
    'AN ELEMENT OF RAW HTML INDENTED BY TWO SPACES IN A LIST ITEM, A BLANK LINE OF TWO BLOCK QUOTES AFTER IT',
    '> > - i\n> >   <pre>\n> > \n> > ',
  ],
];

await describe({
  name: 'a block of raw HTML indented by a tab in a list item',
  children: readerEqualityCases({ cases: CASES, },),
},);
