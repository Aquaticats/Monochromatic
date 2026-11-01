/**
 Tests that a block the page writes as a paragraph, in a block quote or a list item, is kept from the line after it.

 The strict grammar has no raw HTML and no indented code, so the page writes a raw HTML block of the kinds that end at a
 line of their own, and a block of indented code, as lines of a paragraph. The line right after such a block in the same
 container is a lazy line of that paragraph on the page, where the source starts a block there: a definition shows as
 text and the underline of a heading as a row of equals signs. A blank line of the container, with its marks, between the
 two keeps the blocks apart, as the blank line at the top of a text does (`source-ships-page-goes-on.unit.test.ts`).

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
    'A DEFINITION AFTER A PROCESSING INSTRUCTION IN A BLOCK QUOTE',
    '> <?x?>\n> [r]: u{b}\n',
  ],
  [
    'A DEFINITION AFTER A PROCESSING INSTRUCTION IN A LIST ITEM',
    '- <?x?>\n  [r]: u{b}\n',
  ],
  [
    'A DEFINITION AFTER A PROCESSING INSTRUCTION IN A NUMBERED LIST ITEM',
    '1. <?x?>\n   [r]: u{b}\n',
  ],
  [
    'A DEFINITION AFTER A PROCESSING INSTRUCTION IN A BLOCK QUOTE INSIDE A BLOCK QUOTE',
    '> > <?x?>\n> > [r]: u{b}\n',
  ],
  [
    'A ROW OF EQUALS SIGNS AFTER A PROCESSING INSTRUCTION IN A BLOCK QUOTE',
    '> <?x?>\n> ===\n',
  ],
  [
    'A DEFINITION AFTER A SCRIPT ELEMENT IN A BLOCK QUOTE',
    '> <script>\n> </script>\n> [r]: u{b}\n',
  ],
  [
    'A DEFINITION AFTER INDENTED CODE IN A BLOCK QUOTE',
    '> a\n>\n>     code {c}\n> [r]: u{b}\n',
  ],
  [
    'A DEFINITION AFTER INDENTED CODE IN A LIST ITEM',
    '- a\n\n      code {c}\n  [r]: u{b}\n',
  ],
  [
    'A DEFINITION AFTER A PROCESSING INSTRUCTION IN A BLOCK QUOTE, ON A LINE OUTSIDE THE QUOTE',
    '> <?x?>\n[r]: u{b}\n',
  ],
  [
    'A DEFINITION AFTER A PROCESSING INSTRUCTION IN A LIST ITEM, ON A LINE OUTSIDE THE ITEM',
    '- <?x?>\n[r]: u{b}\n',
  ],
];

await describe({
  name: 'the page after a block written as a paragraph in a container',
  children: readerEqualityCases({ cases: CASES, },),
},);
