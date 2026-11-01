import type { Segment, } from './source-reading-words.ts';

//region Source reading lookahead
// WHAT THE NEXT BLOCK NO RANGE REACHES THAT HAS A STRETCH BEFORE IT ASKS OF THE PLACES THE ONE BEFORE IT ENDS AT. The anchors
// of a source are placed in order, and an anchor with blocks a range reaches before it (a stretch) may stand anywhere after
// the place the anchor before it ends at, at the cost of those blocks, so a place that ends an anchor is of no use to a
// later one when another place before it, that the anchors between still fit after, ends the anchor at a cost that is at
// most the stretch's less. This module says, for each anchor, which anchors stand between it and the next one with a
// stretch before it, and what that stretch costs, so that `placedAfter` can leave those places unwritten.

/**
 What the places an anchor ends at are asked of, by the anchors after it.
 */
export type Lookahead = {
  /**
   Words of the anchors between this one and the next with a stretch before it, in order: a place stays in use only where
   the page shows them right after it.
   */
  readonly words: readonly string[];

  /**
   What the stretch before that next anchor costs; infinity where no anchor with a stretch follows.
   */
  readonly cost: number;
};

/**
 The lookahead of an anchor no anchor with a stretch before it follows: every place it ends at is kept.
 */
export const NO_LOOKAHEAD: Lookahead = {
  words: [],
  cost: Number.POSITIVE_INFINITY,
};

/**
 Says, for each run of blocks of a source, what the anchors after it ask of the places it ends at.

 @param segments - runs of the source, in order, as the placing reads them

 @returns For each run, its lookahead; that of a run a range reaches is {@link NO_LOOKAHEAD}, since it ends at no place

 @example
 ```ts
 const lookaheads = lookaheadsOf({ segments: [{ reached: false, words: ['a',], blocks: 0, },], },);
 ```
 */
export function lookaheadsOf({ segments, }: { readonly segments: readonly Segment[]; },): readonly Lookahead[] {
  /**
   The lookahead of each run.
   */
  const lookaheads: Lookahead[] = segments.map(function none(): Lookahead {
    return NO_LOOKAHEAD;
  },);
  /**
   The last anchor that has a stretch before it, none yet at a negative index.
   */
  let owner = -1;
  /**
   The words of the anchors read since, one list for each.
   */
  let chained: (readonly string[])[] = [];
  for (const [at, segment,] of segments.entries()) {
    if (segment.reached)
      continue;
    /**
     The run right before the anchor.
     */
    const before = segments[at - 1];
    if (before?.reached !== true) {
      chained.push(segment.words,);
      continue;
    }
    if (owner >= 0) {
      lookaheads[owner] = {
        words: chained.flat(),
        cost: before.blocks,
      };
    }
    owner = at;
    chained = [];
  }
  return lookaheads;
}

//endregion Source reading lookahead
