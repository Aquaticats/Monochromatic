import { makeInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { ChunkPair, } from '../dist/final/node/index.mjs';

//region Insertion anchor pair
// A PAIR WHOSE TARGET SIDE IS AN INSERTION ANCHOR AT AN OFFSET, the target
// naming a boundary rather than a span, for cases exercising placement or
// splicing at a point where text is missing rather than over text that is
// there.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The placement-layout and splice-slices
// tests kept their own copy of this pair builder; both now import it from
// here.

/**
 Builds one pair whose target is an anchor at an offset.

 @param sliceIndex - position of this slice

 @param offset - boundary it names

 @returns Pair whose target names that boundary

 @example
 ```ts
 const pair = anchorAt({ sliceIndex: 1, offset: 17, },);
 ```
 */
export function anchorAt(
  {
    sliceIndex,
    offset,
  }: {
    readonly sliceIndex: number;
    readonly offset: number;
  },
): ChunkPair {
  return {
    source: {
      sliceIndex,
      nodes: [],
      startOffset: 0,
      endOffset: 0,
      text: '猫',
    },
    target: makeInsertionChunk({
      sliceIndex,
      offset,
    },),
  };
}

//endregion Insertion anchor pair
