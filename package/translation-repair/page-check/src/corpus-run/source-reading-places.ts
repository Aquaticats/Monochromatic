import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  type Lookahead,
  NO_LOOKAHEAD,
} from './source-reading-lookahead.ts';
import {
  firstAtLeast,
  matchesAt,
  type Placing,
  type Placings,
  type Segment,
  type WordIndex,
} from './source-reading-words.ts';

export {
  indexOfWords,
  matchesAt,
  type Placings,
  type Segment,
  type WordIndex,
} from './source-reading-words.ts';

//region Source reading places
// WHERE THE WORDS OF THE BLOCKS NO RANGE REACHES STAND IN THE WORDS OF THE PAGE, and what it costs to read the page so
// (`source-reading-blocks.ts` says what the blocks are and why). Each such block, an anchor, is looked for in the words the
// page shows after the places the anchors before it can end at. The places an anchor can stand at are read off an index of
// where each word of the page stands, from the first place the anchors before it can end at on, and the places the anchors
// before it can end at are kept in the order of the page, so no pass reads the whole page for one anchor. Where blocks a
// range reaches stand before an anchor, the places it ends at are not all written: a place that no later anchor can use
// better than another is left out (`placedAfter` says which), so the work grows with the places that matter and not with
// the pairs of an anchor and a place it can end at, however often the same words repeat.

/**
 A place for a list that has none left: at infinity, so that it is never the next.
 */
const NO_PLACING: Placing = {
  place: Number.POSITIVE_INFINITY,
  cost: Number.POSITIVE_INFINITY,
};

/**
 Where the place at an index of a list stands.

 @param list - places in the order of the page

 @param at - index in the list

 @returns Index of the page word the place stands at; infinity past the end of the list

 @example
 ```ts
 const place = placeAt({ list: [{ place: 3, cost: 0, },], at: 1, },); // Infinity
 ```
 */
function placeAt(
  {
    list,
    at,
  }: {
    readonly list: readonly Placing[];
    readonly at: number;
  },
): number {
  return (list[at] ?? NO_PLACING).place;
}

/**
 Where the first place from an index on stands at which a block may stand.

 @param positions - where the block's first word stands in the page, in order

 @param from - first index the place may stand at

 @param mayStand - whether the block may stand at a place

 @returns Place; infinity where there is none

 @example
 ```ts
 const place = firstPlaceAt({ positions: [2, 5,], from: 3, mayStand: () => true, },); // 5
 ```
 */
function firstPlaceAt(
  {
    positions,
    from,
    mayStand,
  }: {
    readonly positions: readonly number[];
    readonly from: number;
    readonly mayStand: (place: number,) => boolean;
  },
): number {
  for (let at = firstAtLeast({
    positions,
    from,
  },); at < positions.length; at += 1) {
    /**
     A place the first word stands at.
     */
    const place = nonNullishOrThrow(positions[at],);
    if (mayStand(place,))
      return place;
  }
  return Number.POSITIVE_INFINITY;
}

/**
 Records what one way of placing a block costs, keeping the cheapest cost for the place the block ends at.

 @param next - where the block can end, with the cost, which this adds to

 @param end - place the block ends at

 @param cost - what the way of getting there costs

 @example
 ```ts
 keepCheapest({ next: new Map(), end: 4, cost: 1, },);
 ```
 */
function keepCheapest(
  {
    next,
    end,
    cost,
  }: {
    readonly next: Map<number, number>;
    readonly end: number;
    readonly cost: number;
  },
): void {
  next.set(
    end,
    Math.min(
      next.get(end,) ?? Number.POSITIVE_INFINITY,
      cost,
    ),
  );
}

/**
 Lists the places the blocks before can end at, with their cost, in the order of the page.

 @param placings - where the blocks before can end

 @returns Same as a list

 @example
 ```ts
 const froms = placingsList({ placings: new Map([[3, 1,],],), },); // [{ place: 3, cost: 1 }]
 ```
 */
function placingsList({ placings, }: { readonly placings: Placings; },): readonly Placing[] {
  /**
   The places, as they come.
   */
  const froms: Placing[] = [];
  for (const [place, cost,] of placings) {
    froms.push({
      place,
      cost,
    },);
  }
  return froms;
}

/**
 Reads off the places the blocks before can end at the ones that cost less than every place before them: the cost of
 getting past any place is that of the cheapest place at or before it, and it falls only at these.

 @param froms - places the blocks before can end at, in the order of the page

 @returns Places that cost less than each one before them, in the order of the page

 @example
 ```ts
 const steps = stairsOf({ froms: [{ place: 1, cost: 3, }, { place: 2, cost: 4, }, { place: 5, cost: 1, },], },); // places 1, 5
 ```
 */
function stairsOf({ froms, }: { readonly froms: readonly Placing[]; },): readonly Placing[] {
  /**
   The steps found.
   */
  const steps: Placing[] = [];
  /**
   The least cost seen so far.
   */
  const seen = { least: Number.POSITIVE_INFINITY, };
  for (const from of froms) {
    if (from.cost < seen.least) {
      steps.push(from,);
      seen.least = from.cost;
    }
  }
  return steps;
}

/**
 Where a block can start so that it follows a place the blocks before end at with exactly the words the blocks a range
 reaches show between them, and what that costs.

 @param page - words the page shows

 @param froms - places the blocks before can end at, in the order of the page

 @param held - what the blocks a range reaches show since the last block no range reaches

 @param canStartAt - whether the block may start at a place

 @returns Places in the order of the page, each with the cost of the place it follows

 @example
 ```ts
 const starts = exactStartsOf({ page, froms, held, canStartAt: () => true, },);
 ```
 */
function exactStartsOf(
  {
    page,
    froms,
    held,
    canStartAt,
  }: {
    readonly page: readonly string[];
    readonly froms: readonly Placing[];
    readonly held: Segment;
    readonly canStartAt: (place: number,) => boolean;
  },
): readonly Placing[] {
  /**
   The words the blocks a range reaches show.
   */
  const { words: heldWords, } = held;
  return froms
    .filter(function showsHeldWords(from,): boolean {
      return matchesAt({
        page,
        at: from.place,
        words: heldWords,
      },);
    },)
    .map(function startAfter(from,): Placing {
      return {
        place: from.place + heldWords.length,
        cost: from.cost,
      };
    },)
    .filter(function mayStart({ place, },): boolean {
      return canStartAt(place,);
    },);
}

/**
 Places the words of one block no range reaches after each place the blocks before it can end at.

 The cost of a way of getting to a place is the cost of the place the blocks before ended at, and the blocks a range
 reaches when the words of the page between the two places are not what those blocks show. Only the place that is exactly
 as many words back as those blocks show can avoid that cost, so the cheapest way to a place is the cheaper of that one
 and the cheapest of every place the words can start after, which is read off the places in order and not asked of each
 pair. The places the blocks before can end at are in the order of the page, since every call writes them in the order
 of the places it reads, which come in the order of the page and end at the place plus the same number of words.

 THE PLACES THAT CAN NOT BE OF USE ARE LEFT UNWRITTEN when the lookahead says what the blocks after this one ask of them. A
 place the next stretch is read from is of use only where the page shows the words between the two blocks right after it;
 and a place that costs as much as the stretch more than a place before it that is of use is not, since the cost of
 getting from that place to wherever this one leads is no more, and it reaches every place after it. That leaves the
 places the cost falls at and the few the exact words lead to, however many places the block's words stand at.

 @param page - words the page shows

 @param index - where each word of the page stands

 @param placings - where the blocks before can end, with their cost, in the order of the page

 @param segment - block's words, and how many blocks a range reaches stand since the last such block

 @param held - what the blocks a range reaches show since the last block no range reaches

 @param isLast - whether no block of the source follows

 @param ahead - what the blocks after this one ask of the places it ends at; nothing, so that every place is written, when
 absent

 @param bound - cost some way of placing all the blocks is known to have: a place that costs as much is not written, since
 no way through it costs less; infinity when absent

 @returns Where the block can end, with the cost, in the order of the page; none when the page shows its words nowhere
 it can come

 @example
 ```ts
 const next = placedAfter({ page, index, placings: new Map([[0, 0,],]), segment, held, isLast: false, },);
 ```
 */
export function placedAfter(
  {
    page,
    index,
    placings,
    segment,
    held,
    isLast,
    ahead = NO_LOOKAHEAD,
    bound = Number.POSITIVE_INFINITY,
  }: {
    readonly page: readonly string[];
    readonly index: WordIndex;
    readonly placings: Placings;
    readonly segment: Segment;
    readonly held: Segment;
    readonly isLast: boolean;
    readonly ahead?: Lookahead;
    readonly bound?: number;
  },
): Placings {
  /**
   The block's words.
   */
  const { words, } = segment;
  /**
   How many blocks a range reaches stand before the block.
   */
  const { blocks: heldBlocks, } = held;
  /**
   Where the block can end, with the cost.
   */
  const next = new Map<number, number>();

  /**
   Whether the block may end at a place after it stands there, which the last block does only where it ends the page.

   @param place - index of the block's first word

   @returns Whether the block may stand there

   @example
   ```ts
   const may = canEnd({ place: 3, },);
   ```
   */
  function canEnd({ place, }: { readonly place: number; },): boolean {
    return (!isLast) || ((place + words.length) === page.length);
  }

  /**
   Whether the block's words stand at a place, and the block may end there.

   @param place - index of the block's first word

   @returns Whether the block may stand there

   @example
   ```ts
   const may = canStandAt(3,);
   ```
   */
  function canStandAt(place: number,): boolean {
    return canEnd({ place, },) && matchesAt({
      page,
      at: place,
      words,
    },);
  }

  if (heldBlocks === 0) {
    // No block a range reaches stands between: the block starts where the blocks before it ended.
    for (const [place, cost,] of placings) {
      if ((cost < bound) && canStandAt(place,)) {
        keepCheapest({
          next,
          end: place + words.length,
          cost,
        },);
      }
    }
    return next;
  }
  /**
   The places the blocks before can end at, with their cost, in the order of the page.
   */
  const froms = placingsList({ placings, },);
  /**
   The first place the blocks before can end at.
   */
  const [first,] = froms;
  if (first === undefined)
    return next;
  /**
   The places the cost of getting past a place falls at.
   */
  const stairs = stairsOf({ froms, },);
  /**
   Where the block can start after exactly the words the blocks a range reaches show.
   */
  const exacts = exactStartsOf({
    page,
    froms,
    held,
    canStartAt: canStandAt,
  },);
  /**
   Where the first word of the block stands in the page.
   */
  const positions = index.get(nonNullishOrThrow(words[0],),) ?? [];
  /**
   Where the walk over the places has got to: the next place that may be read, how many exact starts and steps of the
   stairs are behind it, the least cost of a place kept, and the first place the words stand at from the walk on.
   */
  const walk = {
    at: first.place,
    exact: 0,
    step: 0,
    least: Number.POSITIVE_INFINITY,
    standing: -1,
  };
  for (;;) {
    if (walk.standing < walk.at) {
      walk.standing = firstPlaceAt({
        positions,
        from: walk.at,
        mayStand: canStandAt,
      },);
    }
    /**
     The next exact start, none left where it stands at infinity.
     */
    const exact = exacts[walk.exact] ?? NO_PLACING;
    /**
     The next place the cost is asked at: where the words stand, or an exact start.
     */
    const place = Math.min(
      walk.standing,
      exact.place,
    );
    if (!Number.isFinite(place,))
      break;
    while (placeAt({
      list: stairs,
      at: walk.step + 1,
    },) <= place)
      walk.step += 1;
    /**
     Whether the place follows exactly the words the blocks a range reaches show.
     */
    const isExact = exact.place === place;
    /**
     The cost of getting there from any place before, and from the place exactly the held words back.
     */
    const step = nonNullishOrThrow(stairs[walk.step],);
    /**
     The cost of getting there from any place before.
     */
    const reachedFromAny = step.cost + heldBlocks;
    /**
     The cheaper of the two.
     */
    const cost = Math.min(
      reachedFromAny,
      isExact ? exact.cost : Number.POSITIVE_INFINITY,
    );
    if (isExact)
      walk.exact += 1;
    walk.at = place + 1;
    if (reachedFromAny >= bound) {
      // The places after this one cost as much from any place before, up to where the cost falls or an exact start.
      walk.at = Math.min(
        placeAt({
          list: stairs,
          at: walk.step + 1,
        },),
        placeAt({
          list: exacts,
          at: walk.exact,
        },),
      );
    }
    if (cost >= bound)
      continue;
    if (!Number.isFinite(ahead.cost,)) {
      keepCheapest({
        next,
        end: place + words.length,
        cost,
      },);
      continue;
    }
    // A place that the next block asked of it is not shown after is of no use.
    if (!matchesAt({
      page,
      at: place + words.length,
      words: ahead.words,
    },))
      continue;
    if (cost >= (walk.least + ahead.cost)) {
      // The places after this one that cost the same are of no use either, up to where the cost falls or an exact start.
      if (!isExact) {
        walk.at = Math.min(
          placeAt({
            list: stairs,
            at: walk.step + 1,
          },),
          placeAt({
            list: exacts,
            at: walk.exact,
          },),
        );
      }
      continue;
    }
    walk.least = Math.min(
      walk.least,
      cost,
    );
    keepCheapest({
      next,
      end: place + words.length,
      cost,
    },);
  }
  return next;
}

//endregion Source reading places
