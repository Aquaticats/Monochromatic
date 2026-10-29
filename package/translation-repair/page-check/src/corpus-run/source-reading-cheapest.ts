import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import { boundOf, } from './source-reading-bound.ts';
import { lookaheadsOf, } from './source-reading-lookahead.ts';
import { placedAfter, } from './source-reading-places.ts';
import {
  indexOfWords,
  matchesAt,
  type Placings,
  type Segment,
} from './source-reading-words.ts';

//region Source reading cheapest
// THE FEWEST BLOCKS THE READINGS CANNOT BE ASKED ABOUT over every way the words of the blocks no range reaches can be placed
// in the words of the page (`source-reading-blocks.ts` says what the blocks are). The blocks are placed one after the other
// (`placedAfter` in `source-reading-places.ts`), and a cost that some way of placing them all is known to have is a cost no
// place need reach (`source-reading-bound.ts`).

/**
 The blocks a range reaches when there are none: nothing shown, no block.
 */
const NO_BLOCKS: Segment = {
  reached: true,
  words: [],
  blocks: 0,
};

/**
 Finds the fewest blocks the readings cannot be asked about over every way to place the blocks no range reaches.

 @param page - words the page shows

 @param segments - runs of the source, in order

 @returns Fewest blocks; infinity where the words cannot be placed

 @example
 ```ts
 const cost = cheapestPlacing({ page: ['a',], segments: [{ reached: false, words: ['a',], blocks: 0, },], },);
 ```
 */
export function cheapestPlacing(
  {
    page,
    segments,
  }: {
    readonly page: readonly string[];
    readonly segments: readonly Segment[];
  },
): number {
  /**
   Where each word the page shows stands.
   */
  const index = indexOfWords({ page, },);
  /**
   What the blocks after each block no range reaches ask of the places it ends at.
   */
  const lookaheads = lookaheadsOf({ segments, },);
  /**
   Where the blocks no range reaches can end, and what that cost, as the segments are read in order; the blocks a range
   reaches since the last of them; and the cost of a way known to place them all, which no place need match.
   */
  const reading: {
    placings: Placings;
    held: Segment;
    bound: number;
  } = {
    placings: new Map([[
      0,
      0,
    ],],),
    held: NO_BLOCKS,
    bound: boundOf({
      page,
      index,
      segments,
    },),
  };
  for (const [at, segment,] of segments.entries()) {
    if (segment.reached) {
      reading.held = segment;
      continue;
    }
    reading.placings = placedAfter({
      page,
      index,
      placings: reading.placings,
      segment,
      held: reading.held,
      isLast: at === (segments.length - 1),
      ahead: nonNullishOrThrow(lookaheads[at],),
      bound: reading.bound,
    },);
    reading.held = NO_BLOCKS;
  }
  /**
   What the blocks a range reaches show after the last block no range reaches, and how many they are.
   */
  const {
    words: trailingWords,
    blocks: trailingBlocks,
  } = reading.held;
  /**
   The least cost over the places the last block can end at, with what the page shows after each set against the trailing
   blocks.
   */
  const ends = { least: reading.bound, };
  for (const [from, cost,] of reading.placings) {
    /**
     Whether the page shows after the place what the trailing blocks show, and nothing else.
     */
    const same = ((page.length - from) === trailingWords.length) && matchesAt({
      page,
      at: from,
      words: trailingWords,
    },);
    if (same)
      ends.least = Math.min(
        ends.least,
        cost,
      );
    else if (trailingBlocks !== 0)
      ends.least = Math.min(
        ends.least,
        cost + trailingBlocks,
      );
  }
  return ends.least;
}

//endregion Source reading cheapest
