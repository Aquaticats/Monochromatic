/**
 Tests for the alternative text of an image that holds an angle autolink or a code span.

 The plain parse holds the alternative text of an image as a string: an angle autolink in it is its address alone, and a
 code span is its content as typed. The page must show the same, so the autolink is written as its address without the
 angle brackets MDX reads as a tag, and a code span is copied as it stands, since a backslash written inside one is a
 character of the code.

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

 @returns What the encoder writes and what the comparison says

 @example
 ```ts
 const { written, kind, } = encodedReading({ source: '![<https://example.net/a>](u.png)', },);
 ```
 */
function encodedReading(
  { source, }: { readonly source: string; },
): { readonly written: string; readonly kind: string; } {
  /**
   The page text the encoder writes.
   */
  const written = encodeAsLiteralMdx({ text: source, },);
  return {
    written,
    kind: readingDifferenceOf({
      source,
      carried: written,
    },).kind,
  };
}

await describe({
  name: 'source ships image alternative text',
  children: [
    it({
      name: 'WRITES AN ANGLE AUTOLINK IN THE ALTERNATIVE TEXT AS ITS ADDRESS ALONE, which the page shows as the source does',
      fn: async () => {
        expect(encodedReading({ source: '![<https://example.net/a>](u.png)', },),).toEqual({
          written: '![https://example.net/a](u.png)',
          kind: 'same',
        },);
        expect(encodedReading({ source: '![<a@b.net>](u.png)', },),).toEqual({
          written: '![a@b.net](u.png)',
          kind: 'same',
        },);
      },
    },),

    it({
      name: 'WRITES THE ADDRESS WITH A BACKSLASH BEFORE EACH CHARACTER MARKDOWN READS, and the text around it as any text',
      fn: async () => {
        expect(encodedReading({ source: '![x {y} <https://example.net/a_b>](u.png) {', },),).toEqual({
          written: String.raw`![x \{y\} https://example.net/a\_b](u.png) \{`,
          kind: 'same',
        },);
      },
    },),

    it({
      name: 'WRITES AN ANGLE AUTOLINK AS ITS ADDRESS INSIDE EMPHASIS IN THE ALTERNATIVE TEXT, and in an image reference',
      fn: async () => {
        expect(encodedReading({ source: '![*<https://example.net/a>*](u.png)', },),).toEqual({
          written: '![*https://example.net/a*](u.png)',
          kind: 'same',
        },);
        expect(encodedReading({ source: '![<https://example.net/a>][r]\n\n[r]: u.png\n', },),).toEqual({
          written: '![https://example.net/a][r]\n\n[r]: u.png\n',
          kind: 'same',
        },);
      },
    },),

    it({
      name: 'COPIES A CODE SPAN IN THE ALTERNATIVE TEXT AS IT STANDS, whatever it holds, since a backslash in it would show',
      fn: async () => {
        expect(encodedReading({ source: '![`<https://example.net/a>`](u.png)', },),).toEqual({
          written: '![`<https://example.net/a>`](u.png)',
          kind: 'same',
        },);
        expect(encodedReading({ source: '![`{x}` y {z}](u.png)', },),).toEqual({
          written: String.raw`![` + '`{x}`' + String.raw` y \{z\}](u.png)`,
          kind: 'same',
        },);
      },
    },),
  ],
},);
