import type { BlockPair, } from './pair-blocks-wire.ts';

//region Block pairing counts
// Many-to-many pairing has more relations than blocks whenever one block spans
// several counterparts. Diagnostics therefore report unique reach separately
// from relation count.

/**
 Unique block reach and relation count for one pairing.

 @example
 ```ts
 const counts: BlockPairCounts = { source: 1, target: 2, relations: 2, };
 ```
 */
export type BlockPairCounts = {
  /**
   Unique original blocks reached.
   */
  readonly source: number;

  /**
   Unique translation blocks reached.
   */
  readonly target: number;

  /**
   Pair relations connecting those blocks.
   */
  readonly relations: number;
};

/**
 Counts unique blocks separately from many-to-many relations.

 @param pairs - committed block correspondences

 @returns Unique reach on each side beside relation count

 @example
 ```ts
 const counts = countPairedBlocks({
   pairs: [{ source: 0, target: 0, }, { source: 0, target: 1, },],
 });
 ```
 */
export function countPairedBlocks(
  { pairs, }: { readonly pairs: readonly BlockPair[]; },
): BlockPairCounts {
  /**
   Original indexes reached by each relation.
   */
  const sourceIndexes = pairs.map(function sourceIndex(pair,): number {
    return pair.source;
  },);
  /**
   Translation indexes reached by each relation.
   */
  const targetIndexes = pairs.map(function targetIndex(pair,): number {
    return pair.target;
  },);
  return {
    source: new Set(sourceIndexes,).size,
    target: new Set(targetIndexes,).size,
    relations: pairs.length,
  };
}

//endregion Block pairing counts
