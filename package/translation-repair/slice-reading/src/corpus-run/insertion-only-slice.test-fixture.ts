import { makeInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { ChunkPair, } from '../../dist/final/node/index.mjs';

//region Insertion-only slice
// ONE SOURCE-ONLY SLICE, whose target is an insertion anchor rather than any
// rendering, for cases over a container half, a deficit, or an untranslated
// tail the archive never rendered.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The insertion-container-deficit,
// insertion-container-halves, insertion-tail-split and coverage-tail tests
// kept their own copy of this slice builder; all now import it from here.

/**
 Builds one source-only slice.

 @param sliceIndex - where the slice stands

 @param source - original text with no rendering beside it

 @returns Slice whose target is an insertion

 @example
 ```ts
 const slice = insertion({ sliceIndex: 2, source: MISSING_SOURCE, },);
 ```
 */
export function insertion(
  {
    sliceIndex,
    source,
  }: {
    readonly sliceIndex: number;
    readonly source: string;
  },
): ChunkPair {
  return {
    source: {
      kind: 'content',
      sliceIndex,
      nodes: [],
      startOffset: 0,
      endOffset: source.length,
      text: source,
    },
    target: makeInsertionChunk({
      sliceIndex,
      offset: 0,
    },),
  };
}

//endregion Insertion-only slice
