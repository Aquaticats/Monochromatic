import type { ChunkPair, } from '../dist/final/node/index.mjs';

//region Span pairs
// PAIRS WHOSE TARGET SIDE IS A SPAN OF A TRANSLATION, for tests of the assembly of a page that need spans of their own
// (a blank line kept at a span's edge, a sealed note no span holds) rather than the paragraph spans of a fixed text.
//
// TEST SUPPORT, NOT PACKAGE SOURCE.

/**
 Where a span starts and ends in a translation, as the offset it starts at and the offset it ends at, exclusive.
 */
type SpanRange = readonly [
  number,
  number,
];

/**
 Builds the pairs over some spans of a translation.

 @param targetText - the translation the spans are cut from

 @param spans - where each span starts and ends in the translation, in document order

 @param firstIndex - the index of the first pair; the next pairs follow it; zero by default

 @returns One pair for each span, whose two sides read the span from the translation, so offsets and text cannot disagree

 @example
 ```ts
 const pairs = pairsOver({ targetText: 'One.\n\nTwo.', spans: [[0, 4,], [6, 10,],], },);
 ```
 */
export function pairsOver(
  {
    targetText,
    spans,
    firstIndex = 0,
  }: {
    readonly targetText: string;
    readonly spans: readonly SpanRange[];
    readonly firstIndex?: number;
  },
): readonly ChunkPair[] {
  return spans.map(function pairOf(
    [startOffset, endOffset,],
    at,
  ): ChunkPair {
    /**
     The side both halves of the pair share, since only the target is read by assembly.
     */
    const side = {
      sliceIndex: firstIndex + at,
      nodes: [],
      startOffset,
      endOffset,
      text: targetText.slice(
        startOffset,
        endOffset,
      ),
    } as const;
    return {
      source: side,
      target: side,
    };
  },);
}

//endregion Span pairs
