import {
  encodeAsLiteralMdx,
  readingDifferenceOf,
} from '../../dist/final/node/index.mjs';

/**
 Fixtures for the tests that need a page which shows other text than its source, now that the encoder writes the pages it
 builds so that they do not.

 The page is built by hand: what the encoder writes, with each image whose alternative text is an angle autolink written
 the way the encoder wrote it before the alternative text was read as inline content, with a backslash before the
 less-than sign, so that the page shows the brackets and the source shows the address alone.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

/**
 The addresses the fixtures of the tests use as the alternative text of an image.
 */
const ADDRESSES: readonly string[] = [
  'https://example.net/a',
  'https://example.net/b',
];

/**
 The page of a text with each image whose alternative text is an angle autolink written with its brackets.

 @param text - text whose images have an angle autolink of an address of `example.net` as their alternative text

 @returns Page text that a reader sees as other text than the source

 @example
 ```ts
 const page = bracketedAltPage({ text: '![<https://example.net/a>](u.png)', },);
 ```
 */
function bracketedAltPage({ text, }: { readonly text: string; },): string {
  return ADDRESSES.reduce(
    function bracketed(
      page,
      address,
    ): string {
      return page.replaceAll(
        `![${address}]`,
        String.raw`![\<${address}>]`,
      );
    },
    encodeAsLiteralMdx({ text, },),
  );
}

/**
 The comparison of a text with its page built by hand, the way the page check compares a run with the page that carries
 it.

 @param source - text of a run of slices

 @returns Answer of the comparison

 @example
 ```ts
 const answer = compareWithBracketedAlt({ source: '![<https://example.net/a>](u.png)', },);
 ```
 */
export function compareWithBracketedAlt(
  { source, }: { readonly source: string; },
): ReturnType<typeof readingDifferenceOf> {
  return readingDifferenceOf({
    source,
    carried: bracketedAltPage({ text: source, },),
  },);
}
