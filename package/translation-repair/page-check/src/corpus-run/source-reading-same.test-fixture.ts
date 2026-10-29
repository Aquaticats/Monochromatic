import {
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  encodeAsLiteralMdx,
  readingDifferenceOf,
} from '../../dist/final/node/index.mjs';

//region Same reading cases
// TEST SUPPORT, NOT PACKAGE SOURCE. WHAT THE ENCODER WRITES FOR A TEXT AND WHAT THE COMPARISON SAYS OF THE PAGE, and the
// cases of a suite that asks it of each text of a list: the answer must be that the page reads the same as the source.

/**
 What the encoder writes for a text, and what the comparison says of the page.
 */
type EncodedReading = {
  /**
   The page text the encoder writes.
   */
  readonly written: string;

  /**
   The answer of the comparison.
   */
  readonly kind: string;
};

/**
 What the encoder writes for a text, and what the comparison says of the page.

 @param source - text of a run of slices

 @returns Page text and answer of the comparison

 @example
 ```ts
 const { written, kind, } = encodedReading({ source: 'a\n    $$\n    {a}\n    $$\n', },);
 ```
 */
export function encodedReading(
  { source, }: { readonly source: string; },
): EncodedReading {
  /**
   The page text the encoder writes.
   */
  const written = encodeAsLiteralMdx({ text: source, },);
  /**
   What the comparison says of it.
   */
  const { kind, } = readingDifferenceOf({
    source,
    carried: written,
  },);
  return {
    written,
    kind,
  };
}

/**
 Builds the cases of a suite that asks the comparison of each text.

 @param cases - name and text of each case, the name in capitals

 @returns One test case a text, which passes when the comparison says the page reads the same as the source

 @example
 ```ts
 const children = sameReadingCases({ cases: [['A BRACE', '她说 {\n',],], },);
 ```
 */
export function sameReadingCases(
  {
    cases,
  }: {
    readonly cases: readonly (readonly [
      string,
      string,
    ])[];
  },
): readonly ReturnType<typeof it>[] {
  return cases.map(function sameReadingCase([name, text,],): ReturnType<typeof it> {
    return it({
      name: `SHOWS THE SOURCE TEXT FOR ${name}`,
      fn: function readsAsTheSourceDoes(): Promise<void> {
        /**
         What the comparison says of the page.
         */
        const { kind, } = encodedReading({ source: text, },);
        expect(kind,)
          .toBe('same',);
        return Promise.resolve();
      },
    },);
  },);
}

//endregion Same reading cases
