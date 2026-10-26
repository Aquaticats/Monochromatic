/**
 Tests that a pipe written with a backslash in an address the encoder writes as a link is shown as the source shows it in
 a table.

 A table ends a cell at every pipe that is not written with a backslash. The encoder writes an angle autolink, and a bare
 address that a less-than sign ends, as a link in Markdown's own syntax, with the address as the text of the link and as its
 target. A pipe the original wrote with a backslash stood in a cell of the source; written bare into the text or the target
 of the link, it ended the cell on the page and cut the link in two. The pipe is written with a backslash there.

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
    'AN ANGLE AUTOLINK HOLDING A PIPE WRITTEN WITH A BACKSLASH IN A CELL',
    '| a | b |\n| - | - |\n| <https://example.net/a\\|b> | c |\n',
  ],
  [
    'AN ANGLE AUTOLINK HOLDING TWO PIPES WRITTEN WITH A BACKSLASH IN A CELL, WITH A BRACE AFTER IT',
    '| a | b |\n| - | - |\n| <https://example.net/a\\|b\\|c> {d} | c |\n',
  ],
  [
    'A BARE ADDRESS HOLDING A PIPE WRITTEN WITH A BACKSLASH, THEN A LESS-THAN SIGN, IN A CELL',
    '| a | b |\n| - | - |\n| https://example.net/a\\|b<c> | d |\n',
  ],
  [
    'AN ANGLE AUTOLINK HOLDING A PIPE WRITTEN WITH A BACKSLASH IN THE TEXT OF A LINK IN A CELL',
    '| a | b |\n| - | - |\n| [<https://example.net/a\\|b>](u) | d |\n',
  ],
  [
    'AN ANGLE AUTOLINK HOLDING A PIPE WRITTEN WITH A BACKSLASH IN A CELL OF A TABLE IN A BLOCK QUOTE',
    '> | a | b |\n> | - | - |\n> | <https://example.net/a\\|b> | c |\n',
  ],
  [
    'AN ANGLE AUTOLINK HOLDING A PIPE WRITTEN WITH A BACKSLASH IN A CELL OF A TABLE WITHOUT LEADING PIPES',
    'a | b\n- | -\n<https://example.net/a\\|b> | c\n',
  ],
  [
    'AN ANGLE AUTOLINK HOLDING A PIPE WRITTEN WITH A BACKSLASH OUTSIDE A TABLE',
    'a <https://example.net/a\\|b> b\n',
  ],
  [
    'A BARE ADDRESS HOLDING A PIPE IN A CELL, WHICH THE TABLE ENDS THE CELL AT',
    '| a | b |\n| - | - |\n| https://example.net/a|b<c> | d |\n',
  ],
];

await describe({
  name: 'a pipe written with a backslash in an address written as a link, in a table',
  children: readerEqualityCases({ cases: CASES, },),
},);
