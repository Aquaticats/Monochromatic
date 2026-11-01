import type { ChunkPair, } from '../../dist/final/node/index.mjs';

//region Archive slice pair
// ONE SLICE OVER A FIXED ARCHIVE SENTENCE, with an archive-side text the
// case supplies, for restore and unify tests that only need the target side
// to vary.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. Several corpus-run restore and unify
// tests kept their own copy of these builders; all now import them from
// here.

/**
 One slice over an archive text.

 @param sliceIndex - where the slice stands

 @param target - archive text

 @returns Prepared pair

 @example
 ```ts
 const slice = pair({ sliceIndex: 0, target: 'She napped.', },);
 ```
 */
export function pair(
  {
    sliceIndex,
    target,
  }: {
    readonly sliceIndex: number;
    readonly target: string;
  },
): ChunkPair {
  return {
    source: {
      kind: 'content',
      sliceIndex,
      nodes: [],
      startOffset: 0,
      endOffset: 4,
      text: '她打盹。',
    },
    target: {
      kind: 'content',
      sliceIndex,
      nodes: [],
      startOffset: 0,
      endOffset: target.length,
      text: target,
    },
  };
}

/**
 One slice over an archive text, starting at an offset.

 @param sliceIndex - where the slice stands

 @param target - archive text

 @param startOffset - where the archive text starts in the page

 @returns Prepared pair

 @example
 ```ts
 const slice = pairAt({ sliceIndex: 0, target: 'She napped.', startOffset: 0, },);
 ```
 */
export function pairAt(
  {
    sliceIndex,
    target,
    startOffset,
  }: {
    readonly sliceIndex: number;
    readonly target: string;
    readonly startOffset: number;
  },
): ChunkPair {
  return {
    source: {
      kind: 'content',
      sliceIndex,
      nodes: [],
      startOffset: 0,
      endOffset: 4,
      text: '她打盹。',
    },
    target: {
      kind: 'content',
      sliceIndex,
      nodes: [],
      startOffset,
      endOffset: startOffset + target.length,
      text: target,
    },
  };
}

//endregion Archive slice pair
