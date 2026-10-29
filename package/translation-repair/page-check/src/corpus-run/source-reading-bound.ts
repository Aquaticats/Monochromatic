import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  firstAtLeast,
  matchesAt,
  type Segment,
  type WordIndex,
} from './source-reading-words.ts';

//region Source reading bound
// A COST SOME WAY OF PLACING THE BLOCKS NO RANGE REACHES IS KNOWN TO HAVE, found before the blocks are placed: no place that
// costs as much is worth writing. The words of the source, those of the blocks a range reaches among them, stand in the
// page at an offset from the start of the source where the page shows the source from its start, and at another where it
// shows it up to its end, the two being the same where it shows all of it and differing by what the blocks a range reaches
// add or lose between. A way that keeps to these two offsets, and goes from the one to the other, or on to the first place
// the next block stands at, where a stretch of blocks a range reaches stands, is one way among those the placing considers,
// so what it costs is a bound on the cheapest. The few cheapest places each block can end at on such ways are kept, and
// the search for the first place the next block stands at gives up after a few, so the bound is found in the time of the
// words of the source; where it is not found the blocks are placed with no bound.

/**
 How many places a block is kept at when the bound is found.
 */
const KEPT_PLACES = 8;

/**
 How many places the first word of a block stands at are tried when looking for the first that it stands at.
 */
const PLACES_TRIED = 64;

/**
 What the blocks a range reaches show when there are none.
 */
const NO_BLOCKS: Segment = {
  reached: true,
  words: [],
  blocks: 0,
};

/**
 A place a block can end at on a way that keeps to the offsets, and what getting there cost.
 */
type Standing = {
  /**
   Index of the page word the block ends after.
   */
  readonly end: number;

  /**
   What getting there cost.
   */
  readonly cost: number;
};

/**
 How many words of the source stand before each run of blocks, and in all.
 */
type Counted = {
  /**
   For each run, the number of words before it.
   */
  readonly starts: readonly number[];

  /**
   The number of words in all the runs.
   */
  readonly total: number;
};

/**
 Counts the words of the source before each run of blocks.

 @param segments - runs of the source, in order

 @returns For each run, the number of words before it, and the number of words in all

 @example
 ```ts
 const { starts, total, } = startsOf({ segments: [{ reached: false, words: ['a',], blocks: 0, },], },); // [0], 1
 ```
 */
function startsOf({ segments, }: { readonly segments: readonly Segment[]; },): Counted {
  /**
   Words before each run.
   */
  const starts: number[] = [];
  /**
   Words counted so far.
   */
  const counted = { total: 0, };
  for (const { words, } of segments) {
    starts.push(counted.total,);
    counted.total += words.length;
  }
  return {
    starts,
    total: counted.total,
  };
}

/**
 Finds the first place from an index on at which a block stands, among the first few places its first word stands at.

 @param page - words the page shows

 @param index - where each word of the page stands

 @param words - words of the block

 @param from - first index the place may stand at

 @param mayStand - whether the block may end after the place it stands at

 @returns Place; infinity where none of the first few is one

 @example
 ```ts
 const place = firstStandingFrom({ page, index, words, from: 0, mayStand: () => true, },);
 ```
 */
function firstStandingFrom(
  {
    page,
    index,
    words,
    from,
    mayStand,
  }: {
    readonly page: readonly string[];
    readonly index: WordIndex;
    readonly words: readonly string[];
    readonly from: number;
    readonly mayStand: (place: number,) => boolean;
  },
): number {
  /**
   Where the first word of the block stands in the page.
   */
  const positions = index.get(nonNullishOrThrow(words[0],),) ?? [];
  /**
   Where the search starts in them.
   */
  const first = firstAtLeast({
    positions,
    from,
  },);
  /**
   Where it stops.
   */
  const last = Math.min(
    positions.length,
    first + PLACES_TRIED,
  );
  for (let at = first; at < last; at += 1) {
    /**
     A place the first word stands at.
     */
    const place = nonNullishOrThrow(positions[at],);
    if (mayStand(place,) && matchesAt({
      page,
      at: place,
      words,
    },))
      return place;
  }
  return Number.POSITIVE_INFINITY;
}

/**
 Finds the places a block can end at after a place a block before it ended at, and what getting there costs.

 @param page - words the page shows

 @param index - where each word of the page stands

 @param segment - block

 @param held - what the blocks a range reaches show since the last block no range reaches

 @param from - place the block before ended at, with the cost

 @param isLast - whether the block is the last of the source, which must end the page

 @param guesses - places the block is tried at along the offsets

 @returns Places, each with the cost of getting there

 @example
 ```ts
 const ends = placedAfterStanding({ page, index, segment, held, from: { end: 0, cost: 0, }, isLast: false, guesses: [], },);
 ```
 */
function placedAfterStanding(
  {
    page,
    index,
    segment,
    held,
    from,
    isLast,
    guesses,
  }: {
    readonly page: readonly string[];
    readonly index: WordIndex;
    readonly segment: Segment;
    readonly held: Segment;
    readonly from: Standing;
    readonly isLast: boolean;
    readonly guesses: readonly number[];
  },
): readonly Standing[] {
  /**
   The block's words.
   */
  const { words, } = segment;
  /**
   What the blocks a range reaches show before the block, and how many they are.
   */
  const {
    words: heldWords,
    blocks: heldBlocks,
  } = held;
  /**
   Where the blocks before ended, and what getting there cost.
   */
  const {
    end: previousEnd,
    cost: previousCost,
  } = from;

  /**
   Whether the block may end after a place it stands at.

   @param place - index of the block's first word

   @returns Whether the block may stand there

   @example
   ```ts
   const may = mayStand(3,);
   ```
   */
  function mayStand(place: number,): boolean {
    return (!isLast) || ((place + words.length) === page.length);
  }

  /**
   Where the page shows what the blocks a range reaches show right after the place.
   */
  const afterHeldWords = previousEnd + heldWords.length;
  /**
   Whether it does show it there.
   */
  const showsHeldWords = matchesAt({
    page,
    at: previousEnd,
    words: heldWords,
  },);
  /**
   Where the block can start after the place: at once where no blocks a range reaches stand between, and otherwise where
   the page shows what they show, along the offsets, and at the first place it stands at.
   */
  const candidates = (heldBlocks === 0)
    ? [previousEnd,]
    : [
      afterHeldWords,
      ...guesses,
      firstStandingFrom({
        page,
        index,
        words,
        from: previousEnd,
        mayStand,
      },),
    ];
  return candidates
    .filter(function stands(start,): boolean {
      if (start < previousEnd)
        return false;
      if (!mayStand(start,))
        return false;
      return matchesAt({
        page,
        at: start,
        words,
      },);
    },)
    .map(function endAt(start,): Standing {
      /**
       Whether the block follows the held words exactly.
       */
      const exact = (start === afterHeldWords) && showsHeldWords;
      return {
        end: start + words.length,
        cost: previousCost + (((heldBlocks === 0) || exact) ? 0 : heldBlocks),
      };
    },);
}

/**
 Keeps the cheapest few of the places a block can end at, one for each place.

 @param placed - places, with the cost, in any order

 @returns Cheapest places, cheapest first, each place once at its cost

 @example
 ```ts
 const kept = cheapestOf({ placed: [{ end: 4, cost: 1, }, { end: 4, cost: 0, },], },); // [{ end: 4, cost: 0, }]
 ```
 */
function cheapestOf({ placed, }: { readonly placed: readonly Standing[]; },): readonly Standing[] {
  /**
   The places, the cheapest first.
   */
  const ordered = placed.toSorted(function cheapestFirst(
    left,
    right,
  ): number {
    return (left.cost - right.cost) || (left.end - right.end);
  },);
  /**
   The ends seen so far.
   */
  const seen = new Set<number>();
  /**
   The places kept.
   */
  const kept: Standing[] = [];
  for (const standing of ordered) {
    if (seen.has(standing.end,))
      continue;
    seen.add(standing.end,);
    kept.push(standing,);
    if (kept.length >= KEPT_PLACES)
      break;
  }
  return kept;
}

/**
 Finds a cost that the blocks no range reaches can be placed for, along ways that keep to the offsets of the page from
 the source that stand for its start and its end, or go on to the first place the next block stands at.

 @param page - words the page shows

 @param index - where each word of the page stands

 @param segments - runs of the source, in order

 @returns Fewest blocks the readings cannot be asked about over those ways, which is no less than over every way;
 infinity where none of them places the blocks

 @example
 ```ts
 const bound = boundOf({ page, index, segments: [{ reached: false, words: ['a',], blocks: 0, },], },); // 0
 ```
 */
export function boundOf(
  {
    page,
    index,
    segments,
  }: {
    readonly page: readonly string[];
    readonly index: WordIndex;
    readonly segments: readonly Segment[];
  },
): number {
  /**
   How many words of the source stand before each run, and in all.
   */
  const counted = startsOf({ segments, },);
  /**
   The offsets of the page from the source, from the start of the source and up to its end.
   */
  const offsets: readonly number[] = [...new Set([
    0,
    page.length - counted.total,
  ],),];
  /**
   Where the blocks no range reaches before can end, with the cost: none placed yet is a place at the start of the page;
   and what the blocks a range reaches show since.
   */
  const reading: {
    standing: readonly Standing[];
    held: Segment;
  } = {
    standing: [
      {
        end: 0,
        cost: 0,
      },
    ],
    held: NO_BLOCKS,
  };
  for (const [at, segment,] of segments.entries()) {
    if (segment.reached) {
      reading.held = segment;
      continue;
    }
    /**
     Where the blocks before can end.
     */
    const { standing, } = reading;
    /**
     Where the block starts along each offset.
     */
    const guesses = offsets.map(function startAlong(offset,): number {
      return offset + (counted.starts[at] ?? 0);
    },);
    /**
     The places the block can end at on the ways, each with the cost of getting there.
     */
    const placed = standing.flatMap(function endsAfter(from,): readonly Standing[] {
      return placedAfterStanding({
        page,
        index,
        segment,
        held: reading.held,
        from,
        isLast: at === (segments.length - 1),
        guesses,
      },);
    },);
    reading.standing = cheapestOf({ placed, },);
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
   What each place the last block can end at costs, with what the page shows after it set against the trailing blocks.
   */
  const { standing: ending, } = reading;
  /**
   The same, with the costs of the trailing blocks.
   */
  const costs = ending.map(function costAt({
    end,
    cost,
  },): number {
    /**
     Whether the page shows after the place what the trailing blocks show, and nothing else.
     */
    const same = ((page.length - end) === trailingWords.length) && matchesAt({
      page,
      at: end,
      words: trailingWords,
    },);
    if (same)
      return cost;
    return (trailingBlocks === 0) ? Number.POSITIVE_INFINITY : (cost + trailingBlocks);
  },);
  return Math.min(...costs,);
}

//endregion Source reading bound
