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

 @returns The comparison's answer

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
  ],
},);
