/**
 Tests for the comparison of what a reader of the page sees with what a reader of the source sees.

 The comparison reads the source in the flavour the package parses pages in (CommonMark with GFM and the site's math, raw
 HTML on) and the page in the strict MDX grammar, reduces both to the text of their leaves, and says whether the two are
 the same, differ, or cannot be decided. It cannot be decided where the owner's two-readings ruling takes the page's
 reading of a footnote and a reader of plain Markdown cannot give it: a footnote in a block quote or a list item, where the
 source's ranges and the page's differ.

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
  readingDifferenceOf,
} from '../../dist/final/node/index.mjs';

/**
 The comparison of a source with the page carrying it as the encoder writes it.

 @param source - text of a run of slices

 @returns Answer of the comparison

 @example
 ```ts
 const answer = encodedDifference({ source: '她说 { 然后走了。', },);
 ```
 */
function encodedDifference({ source, }: { readonly source: string; },): ReturnType<typeof readingDifferenceOf> {
  return readingDifferenceOf({
    source,
    carried: encodeAsLiteralMdx({ text: source, },),
  },);
}

await describe({
  name: readingDifferenceOf.name,
  children: [
    it({
      name: 'SAYS SAME FOR A TEXT THE PAGE SHOWS AS THE SOURCE DOES, a brace and a footnote among it',
      fn: async () => {
        expect(encodedDifference({ source: '她说 { 然后走了。[^1]\n\n[^1]: 注 <b>\n', },),).toEqual({ kind: 'same', },);
      },
    },),
    it({
      name: 'SAYS SAME FOR A TEXT WITH A BLOCK QUOTE BESIDE A FOOTNOTE WHOSE RANGES THE TWO READINGS AGREE ON',
      fn: async () => {
        expect(encodedDifference({ source: '[^1]: x\n\n> after\n', },),).toEqual({ kind: 'same', },);
      },
    },),
    it({
      name: 'SAYS SAME FOR AN ADDRESS IN A BLOCK OF RAW HTML OR IN INDENTED CODE, which the strict grammar links and plain Markdown does not',
      fn: async () => {
        expect(encodedDifference({ source: '<div>\nhttps://example.net/a {c}\n</div>\n', },),).toEqual({ kind: 'same', },);
        expect(encodedDifference({ source: '    https://example.net/a {c}\n', },),).toEqual({ kind: 'same', },);
      },
    },),
    it({
      name: 'SAYS DIFFERS FOR AN IMAGE WHOSE ALTERNATIVE TEXT IS AN ANGLE AUTOLINK, which the page shows with its brackets',
      fn: async () => {
        expect(encodedDifference({ source: '![<https://example.net/a>](u.png)\n', },).kind,).toBe('differs',);
        expect(encodedDifference({ source: '> a\n\n![<https://example.net/a>](u.png)\n', },).kind,).toBe('differs',);
      },
    },),
    it({
      name: 'SAYS DIFFERS FOR A PAGE THAT HOLDS SYNTAX THE STRICT GRAMMAR READS, since it shows a reader other than the text',
      fn: async () => {
        expect(readingDifferenceOf({ source: '她说 {x}\n', carried: '她说 {x}', },).kind,).toBe('differs',);
      },
    },),
    it({
      name: 'SAYS UNDECIDED FOR A FOOTNOTE IN A BLOCK QUOTE WHOSE RANGES THE TWO READINGS DO NOT AGREE ON, and names why',
      fn: async () => {
        /**
         What the comparison says of a footnote that a tag ends inside a block quote.
         */
        const answer = encodedDifference({ source: '> [^1]: x\n> <q>\n> text\n> ===\n> ', },);
        expect(answer.kind,).toBe('undecided',);
        expect(Object.keys(answer,).toSorted(),).toEqual(['kind', 'reason',],);
      },
    },),
    it({
      name: 'READS A TEXT THAT OPENS WITH A BYTE ORDER MARK AS THE PAGE HOLDS IT, where the mark is a character and starts no block quote',
      fn: async () => {
        /**
         A line that a byte order mark leads, which a parse that drops the mark takes for the start of a block quote holding a
         footnote that a tag ends, and an image after it whose alternative text is an angle autolink. The page holds the mark
         as a character, so the line is a paragraph, the quote after it holds no footnote, and the image differs on its own.
         */
        const answer = encodedDifference({
          source: '\uFEFF> [^1]: x\n> <q>\n> text\n> ===\n![<https://example.net/a>](u.png)\n',
        },);
        expect(answer.kind,).toBe('differs',);
        expect(Object.keys(answer,).toSorted(),).toEqual(['kind', 'reason',],);
        // The same footnote in a block quote that a blank line separates from the mark is a footnote the readings disagree on.
        expect(encodedDifference({ source: '\uFEFFcat\n\n> [^1]: x\n> <q>\n> text\n> ===\n', },).kind,).toBe('undecided',);
      },
    },),
    it({
      name: 'SAYS DIFFERS FOR A BLOCK THAT DIFFERS ON ITS OWN BESIDE A BLOCK THE TWO READINGS CANNOT BE ASKED ABOUT',
      fn: async () => {
        /**
         A block quote whose footnote the two readings end at different places, and after it an image whose alternative text
         is an angle autolink, which the page shows with its brackets whatever stands beside it.
         */
        const after = encodedDifference({
          source: '> [^1]: x\n> <q>\n> text\n> ===\n\n![<https://example.net/a>](u.png)\n',
        },);
        expect(after.kind,).toBe('differs',);
        expect(after.undecidedBlocks,).toBe(1,);
        /**
         The same two blocks in the other order.
         */
        const before = encodedDifference({
          source: '![<https://example.net/a>](u.png)\n\n> [^1]: x\n> <q>\n> text\n> ===\n',
        },);
        expect(before.kind,).toBe('differs',);
        expect(before.undecidedBlocks,).toBe(1,);
      },
    },),
    it({
      name: 'SAYS UNDECIDED FOR A TEXT WHOSE ONLY DIFFERENCE IS IN THE BLOCK THE TWO READINGS DISAGREE ON, a block beside it equal',
      fn: async () => {
        /**
         What the comparison says of a block quote whose footnote a tag ends, beside a paragraph the page shows as it is.
         */
        const answer = encodedDifference({ source: '> [^1]: x\n> <q>\n> text\n> ===\n\n她说 {\n', },);
        expect(answer.kind,).toBe('undecided',);
      },
    },),
    it({
      name: 'SAYS UNDECIDED FOR A BLOCK THE PAGE\'S FOOTNOTE RUNS INTO FROM A BLOCK QUOTE, which no block of the source holds',
      fn: async () => {
        /**
         A block quote whose footnote the page goes on with over the line of raw HTML that follows the quote, so the page has
         one block where the source has two, though the second block starts after the footnote of the source.
         */
        const answer = encodedDifference({ source: '> [^1]: x\n> <a href="x">\n> [r]: u{b}\n<script>\n', },);
        expect(answer.kind,).toBe('undecided',);
        expect(encodedDifference({ source: '> [^1]: x\n> <a href="x">\n> [r]: u{b}\n<details>\n', },).kind,).toBe('undecided',);
      },
    },),
    it({
      name: 'SAYS UNDECIDED, NOT DIFFERENT, FOR TEXT THE PAGE SHOWS AS THE SOURCE DOES IN OTHER BLOCKS, beside a block quote it cannot ask about',
      fn: async () => {
        /**
         The same footnote in a block quote, with after it a list and an element the page reads as one list item with a lazy
         line where the source has two blocks, and a comment that never closes whose paragraphs the page splits where the
         source has one block of raw HTML.
         */
        const quote = '> [^1]: x\n> <a href="x">\n> [r]: u{b}\n';
        expect(encodedDifference({ source: `${quote}1. 项\n<script>\n`, },).kind,).toBe('undecided',);
        expect(encodedDifference({ source: `${quote}\n<!-- a <!-- b\n\n[\n`, },).kind,).toBe('undecided',);
      },
    },),
    it({
      name: 'PLACES THE WORDS OF A BLOCK WHERE THE PAGE SHOWS THEM AFTER THE QUOTE, not at the first place the same words stand',
      fn: async () => {
        // A block quote whose footnote holds the same raw HTML tag as the block after it, so the first place the tag stands on
        // the page is inside the footnote.
        const quote = '> [^1]: x\n> <a href="x">\n> [r]: u{b}\n';
        expect(encodedDifference({ source: `${quote}\n<a href="x">\n\n<script>\n`, },).kind,).toBe('undecided',);
        expect(encodedDifference({ source: `${quote}\n<a href="x">\n\nexport to PDF\n`, },).kind,).toBe('undecided',);
      },
    },),
    it({
      name: 'SAYS UNDECIDED FOR A LINK BESIDE A BLOCK QUOTE IT CANNOT ASK ABOUT, whose target the comparison leaves out as it does for a whole text',
      fn: async () => {
        expect(
          encodedDifference({ source: '[<https://example.net/a>](u)\n> [^1]: x\n> <a href="x">\n> [r]: u{b}\n', },).kind,
        ).toBe('undecided',);
      },
    },),
    it({
      name: 'SAYS DIFFERS FOR TEXT THE PAGE LACKS OR CHANGES BESIDE A BLOCK QUOTE IT CANNOT ASK ABOUT, in whatever blocks the page shows it',
      fn: async () => {
        // A label with a brace, which the page writes with a backslash in the reference and not in the definition.
        expect(
          encodedDifference({ source: '- [^1]: <b>\n  [r]: u{b}\n[^a}]: x {\n\n[^a}] y\n[^1]: 注\n', },).kind,
        ).toBe('differs',);
      },
    },),
  ],
},);
