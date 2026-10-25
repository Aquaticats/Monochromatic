/**
 Tests that a footnote whose first line is a tag alone leaves the text after it read as the source reads it.

 A reader of the source in GFM takes `[^1]: <b>` for a footnote holding a raw HTML block that ends at the end of its line
 (a footnote's continuation lines must be indented, and raw HTML is not a lazy line), so what follows starts blocks of the
 top: `<q>` alone on a line starts raw HTML there, and the lines after it are its text. The plain parse of the encoder
 reads the footnote's first line as a definition and the rest as one paragraph, where `<q>` is a tag in text and a code
 span or a heading underline after it is syntax, so the page would show what the source hides as raw HTML. The text after
 the footnote's end is read on its own.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  encodeAsLiteralMdx,
  readerOfPage,
  readerOfSource,
  withoutLinkTargets,
} from '../../dist/final/node/index.mjs';

/**
 Texts a reader must see as the source shows them: what the case is, and the text.
 */
const CASES: readonly (readonly [string, string])[] = [
  [
    'A CODE SPAN AFTER A TAG ALONE THAT FOLLOWS A FOOTNOTE OPENED WITH A TAG ALONE',
    '[^1]: <b>\n<q>\n`y`\n',
  ],
  [
    'A HEADING MARK AFTER A TAG ALONE THAT FOLLOWS A FOOTNOTE OPENED WITH A TAG ALONE',
    '[^1]: <b>\n<q>\n# head\n',
  ],
  [
    'DISPLAY MATH AFTER A TAG ALONE THAT FOLLOWS A FOOTNOTE OPENED WITH A TAG ALONE',
    '[^1]: <b>\n<q>\n$$\nx\n$$\n',
  ],
  [
    'A FENCE AFTER A TAG ALONE THAT FOLLOWS A FOOTNOTE OPENED WITH A TAG ALONE',
    '[^1]: <b>\n<q>\n```\nz{c}\n```\n',
  ],
  [
    'A TABLE AFTER A TAG ALONE THAT FOLLOWS A FOOTNOTE OPENED WITH A TAG ALONE',
    '[^1]: <b>\n<q>\n| a | b |\n| - | - |\n| c | d |\n',
  ],
  [
    'A CODE SPAN AFTER AN ELEMENT WITH AN ATTRIBUTE THAT FOLLOWS A FOOTNOTE OPENED WITH A TAG ALONE',
    '[^1]: <b>\n<a href="x">\n`y`\n',
  ],
  [
    'A CODE SPAN AFTER A TAG ALONE AT THE END OF THE TEXT, WITH NO FINAL LINE ENDING',
    '[^1]: <b>\n<q>\n`y`',
  ],
];

await describe({
  name: 'the page after a footnote opened with a tag alone',
  children: CASES.map(function readerCase([name, text,],) {
    return it({
      name: `SHOWS THE SOURCE TEXT FOR ${name}`,
      fn: async () => {
        /**
         The text as the encoder writes it.
         */
        const written = encodeAsLiteralMdx({ text, },);
        expect(withoutLinkTargets({ account: readerOfPage({ carried: written, },), },),)
          .toBe(withoutLinkTargets({ account: readerOfSource({ source: text, },), },),);
      },
    },);
  },),
},);
