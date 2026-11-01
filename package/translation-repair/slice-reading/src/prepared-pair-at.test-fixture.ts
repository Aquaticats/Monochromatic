import type { ChunkPair, } from '../dist/final/node/index.mjs';

//region Prepared pair at
// ONE PREPARED PAIR CARRYING THE ARCHIVE'S WORDING AT AN INDEX, for cases
// over a repair or translate wrap that read a slice's incumbent text rather
// than any candidate a lane produced.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The repair-wrap and translate-wrap tests
// kept their own copy of this pair builder; both now import it from here.

/**
 Builds one prepared pair carrying the archive's wording at an index.

 @param sliceIndex - slice index

 @param incumbentText - archive wording there

 @returns Pair shaped as preparation produces one

 @example
 ```ts
 const pair = preparedPairAt({ sliceIndex: 0, incumbentText: 'The cat naps.', },);
 ```
 */
export function preparedPairAt(
  {
    sliceIndex,
    incumbentText,
  }: {
    readonly sliceIndex: number;
    readonly incumbentText: string;
  },
): ChunkPair {
  return {
    source: {
      sliceIndex,
      nodes: [],
      startOffset: 0,
      endOffset: 1,
      text: `source of slice ${String(sliceIndex,)}`,
    },
    target: {
      sliceIndex,
      nodes: [],
      startOffset: 0,
      endOffset: incumbentText.length,
      text: incumbentText,
    },
  };
}

//endregion Prepared pair at
