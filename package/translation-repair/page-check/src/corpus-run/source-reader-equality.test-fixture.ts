import {
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  encodeAsLiteralMdx,
  readerOfPage,
  readerOfSource,
  withoutLinkTargets,
} from '../../dist/final/node/index.mjs';

//region Reader equality cases
// TEST SUPPORT, NOT PACKAGE SOURCE. THE CASES OF A SUITE THAT PUTS WHAT A READER OF THE PAGE SEES BESIDE WHAT A READER OF
// THE SOURCE SEES, one case a text: the text as the encoder writes it, read by the strict grammar, against the text read
// as plain Markdown, link targets aside.

/**
 Builds the cases of a suite that compares what a reader sees of each text.

 @param cases - name and text of each case, the name in capitals

 @returns One test case a text, which passes when the page shows what the source shows

 @example
 ```ts
 const children = readerEqualityCases({ cases: [['A BRACE', '她说 {\n',],], },);
 ```
 */
export function readerEqualityCases(
  {
    cases,
  }: {
    readonly cases: readonly (readonly [
      string,
      string,
    ])[];
  },
): readonly ReturnType<typeof it>[] {
  return cases.map(function readerCase([name, text,],): ReturnType<typeof it> {
    return it({
      name: `SHOWS THE SOURCE TEXT FOR ${name}`,
      fn: function readsAsTheSourceDoes(): Promise<void> {
        /**
         The text as the encoder writes it.
         */
        const written = encodeAsLiteralMdx({ text, },);
        expect(withoutLinkTargets({ account: readerOfPage({ carried: written, },), },),)
          .toBe(withoutLinkTargets({ account: readerOfSource({ source: text, },), },),);
        return Promise.resolve();
      },
    },);
  },);
}

//endregion Reader equality cases
