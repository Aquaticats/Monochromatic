import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';

//region Translate probe coverage
// How much of an aligned section the translation covers, which is what picks
// the section the translate probe demonstrates on.

/**
 Share of a pair's source blocks the translation covers.

 @param pair - aligned section pair

 @returns Target blocks divided by source blocks

 @example
 ```ts
 const ratio = coverageOf({ pair, },);
 ```
 */
export function coverageOf({ pair, }: { readonly pair: ChunkPair; },): number {
  /**
   Blocks on each side.
   */
  const sourceBlocks = pair.source
    .nodes
    .length;

  /**
   Target blocks, which may be far fewer.
   */
  const targetBlocks = pair.target
    .nodes
    .length;

  return targetBlocks / Math.max(
    sourceBlocks,
    1,
  );
}

/**
 The section a probe demonstrates on, or the fact that there is none.

 @example
 ```ts
 const choice: SparsestChoice = { kind: 'none', };
 ```
 */
type SparsestChoice = {
  /**
   Some pair carries source blocks.
   */
  readonly kind: 'found';

  /**
   The pair with the lowest coverage.
   */
  readonly pair: ChunkPair;
} | {
  /**
   No pair carries source blocks.
   */
  readonly kind: 'none';
};

/**
 Picks the aligned section with the lowest coverage among those carrying
 source blocks, the first of equals.

 @param pairs - aligned section pairs in document order

 @returns The sparsest pair, or `none` when no pair carries source blocks

 @example
 ```ts
 const sparsest = sparsestPair({ pairs: alignment.pairs, },);
 ```
 */
export function sparsestPair(
  { pairs, }: { readonly pairs: readonly ChunkPair[]; },
): SparsestChoice {
  /**
   The lowest-coverage pair among those carrying source blocks, absent when
   none carries any.
   */
  const lowest = pairs
    .filter(function hasSource(pair,): boolean {
      /**
       Source blocks this pair carries.
       */
      const { nodes, } = pair.source;

      return nodes.length > 0;
    },)
    .toSorted(function byRatio(
      left,
      right,
    ): number {
      return coverageOf({ pair: left, },) - coverageOf({ pair: right, },);
    },)
    .at(0,);
  if (lowest === undefined)
    return { kind: 'none', };
  return {
    kind: 'found',
    pair: lowest,
  };
}

//endregion Translate probe coverage
