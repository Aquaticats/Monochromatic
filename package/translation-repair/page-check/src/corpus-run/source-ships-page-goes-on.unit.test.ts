/**
 Tests that a page which goes on with a paragraph over a line of raw HTML shows what the source shows.

 The strict grammar has no raw HTML, so the page writes a line that starts an HTML block as a line of the paragraph above
 it. Where the source ends a block at that line, the page needs a blank line there, or what follows it is a lazy line of
 the paragraph: a definition shows as text, and the underline of a heading as a row of equals signs. Two places end a block
 at such a line: a footnote whose paragraph a tag alone on a line ends, and an HTML block of the kinds that end at a line
 of their own (a processing instruction, a declaration, a CDATA section, a script, a pre or a style element), and a block
 of indented code, which the page has none of.

 EVERY CASE COMPARES WHAT A READER SEES, as the cases of `source-ships-encoding.unit.test.ts` do: the leaves of the page
 after the note against the leaves of the source read as plain Markdown.

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
    'A HEADING UNDERLINED BY EQUALS SIGNS AFTER A TAG ALONE THAT ENDS A FOOTNOTE',
    '[^1]: x\n<q>\ntext\n===\n',
  ],
  [
    'A DEFINITION AFTER A PROCESSING INSTRUCTION THAT ENDS A FOOTNOTE',
    '[^1]: x\n<?x?>\n[r]: u{b}\n',
  ],
  [
    'A TABLE AFTER A PROCESSING INSTRUCTION THAT ENDS A FOOTNOTE',
    '[^1]: x\n<?x?>\n| a | b |\n| - | - |\n| c | d |\n',
  ],
  [
    'A DEFINITION AFTER A PROCESSING INSTRUCTION AT THE TOP',
    '<?x?>\n[r]: u{b}\n',
  ],
  [
    'A DEFINITION AFTER A CDATA SECTION AT THE TOP',
    '<![CDATA[x]]>\n[r]: u{b}\n',
  ],
  [
    'A DEFINITION AFTER A DECLARATION AT THE TOP',
    '<!DOCTYPE x>\n[r]: u{b}\n',
  ],
  [
    'A DEFINITION AFTER A SCRIPT ELEMENT AT THE TOP',
    '<script>\n</script>\n[r]: u{b}\n',
  ],
  [
    'A DEFINITION AFTER A PRE ELEMENT AT THE TOP',
    '<pre>\nx\n</pre>\n[r]: u{b}\n',
  ],
  [
    'A HEADING UNDERLINED BY EQUALS SIGNS AFTER A PROCESSING INSTRUCTION AT THE TOP',
    '<?x?>\n===\n',
  ],
  [
    'A DEFINITION AFTER INDENTED CODE AT THE TOP',
    'cat\n\n    purr {c}\n[r]: u{b}\n',
  ],
  [
    'A ROW OF EQUALS SIGNS AFTER INDENTED CODE AT THE TOP',
    '    purr\n===\n',
  ],
  [
    'A DEFINITION AFTER INDENTED CODE THAT FOLLOWS A RAW HTML BLOCK',
    '<details>\n[^1]: x\n\n    purr\n[r]: u{b}\n',
  ],
];

await describe({
  name: 'the page after a line of raw HTML that ends a block of the source',
  children: readerEqualityCases({ cases: CASES, },),
},);
