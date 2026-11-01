import type { ChunkPair, } from '../../dist/final/node/index.mjs';

//region Slice pair of texts
// ONE SLICE PAIR CARRYING GIVEN TEXTS, source and target both ordinary
// content chunks, for window-trial cases that need a slice without a live
// parse.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The window-trial-pick and
// window-trial-slice tests kept their own copy of this pair builder; both
// now import it from here.

/**
 Builds one slice pair carrying given texts.

 @param sliceIndex - position in the document

 @param source - original wording

 @param target - archive wording

 @returns Pair shaped like one preparation produces

 @example
 ```ts
 const pair = slicePairOf({ sliceIndex: 0, source: '猫。', target: 'Cat.', },);
 ```
 */
export function slicePairOf(
  {
    sliceIndex,
    source,
    target,
  }: {
    readonly sliceIndex: number;
    readonly source: string;
    readonly target: string;
  },
): ChunkPair {
  return {
    source: {
      sliceIndex,
      nodes: [],
      startOffset: 0,
      endOffset: source.length,
      text: source,
    },
    target: {
      sliceIndex,
      nodes: [],
      startOffset: 0,
      endOffset: target.length,
      text: target,
    },
  };
}

//endregion Slice pair of texts
