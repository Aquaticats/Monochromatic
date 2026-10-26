import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';

//region Source reading places
// WHERE THE WORDS OF THE BLOCKS NO RANGE REACHES STAND IN THE WORDS OF THE PAGE, and what it costs to read the page so
// (`source-reading-blocks.ts` says what the blocks are and why). Each such block, an anchor, is looked for in the words the
// page shows after the places the anchors before it can end at. The places an anchor can stand at are read off an index of
// where each word of the page stands, from the first place the anchors before it can end at on, and the places the anchors
// before it can end at are kept in the order of the page, so no pass reads the whole page for one anchor. What is left is
// the number of pairs of an anchor and a place it can end at, which grows with the anchors times the places for a page
// whose anchors all show the same words, and with the places the anchor's first word stands at otherwise.

/**
 A run of blocks of the source that a disagreeing range reaches, or that none reaches, and the words they show.
 */
export type Segment = {
  /**
   Whether a disagreeing range reaches the blocks.
   */
  readonly reached: boolean;

  /**
   Words the blocks show, in order.
   */
  readonly words: readonly string[];

  /**
   How many blocks of the source are in the segment.
   */
  readonly blocks: number;
};

/**
 How far the page is read once the words of the blocks no range reaches are placed in it, and what it cost: for each place
 the last such block can end at, in the order of the page, the fewest blocks the readings cannot be asked about on the way
 there.
 */
export type Placings = ReadonlyMap<number, number>;

/**
 Where each word of the page stands, in the order of the page.
 */
export type WordIndex = ReadonlyMap<string, readonly number[]>;

/**
 A place the blocks before can end at, and what getting there cost.
 */
type Placing = {
  /**
   Index of the page word the blocks end after.
   */
  readonly place: number;

  /**
   Blocks a range reaches that the readings cannot be asked about on the way there.
   */
  readonly cost: number;
};

/**
 Lists where each word of the page stands.

 @param page - words the page shows

 @returns For each word, the indices it stands at, in order

 @example
 ```ts
 const index = indexOfWords({ page: ['a', 'b', 'a',], },); // a: [0, 2], b: [1]
 ```
 */
export function indexOfWords({ page, }: { readonly page: readonly string[]; },): WordIndex {
  /**
   Indices found.
   */
  const index = new Map<string, number[]>();
  for (const [at, word,] of page.entries()) {
    /**
     The places the word was found at so far.
     */
    const found = index.get(word,);
    if (found === undefined) {
      index.set(
        word,
        [at,],
      );
    }
    else
      found.push(at,);
  }
  return index;
}

/**
 Whether a run of words stands in a list of words at an index.

 @param page - words to look in

 @param at - index to look at

 @param words - run of words

 @returns Whether the words are the ones of the list from the index on; false for a run that does not fit

 @example
 ```ts
 const stands = matchesAt({ page: ['a', 'b',], at: 1, words: ['b',], },); // true
 ```
 */
export function matchesAt(
  {
    page,
    at,
    words,
  }: {
    readonly page: readonly string[];
    readonly at: number;
    readonly words: readonly string[];
  },
): boolean {
  if ((at < 0) || ((at + words.length) > page.length))
    return false;
  return words.every(function stands(
    word,
    offset,
  ): boolean {
    return page[at + offset] === word;
  },);
}

/**
 Where the first of a list of ascending numbers that is not below a number stands.

 @param positions - ascending numbers

 @param from - number to reach

 @returns Index of the first that is at least the number; the length of the list when there is none

 @example
 ```ts
 const at = firstAtLeast({ positions: [1, 4, 9,], from: 4, },); // 1
 ```
 */
function firstAtLeast(
  {
    positions,
    from,
  }: {
    readonly positions: readonly number[];
    readonly from: number;
  },
): number {
  /**
   The stretch of the list still to search: it holds the answer or ends where it stands.
   */
  const search = {
    low: 0,
    high: positions.length,
  };
  while (search.low < search.high) {
    /**
     Middle of the stretch.
     */
    const middle = Math.floor((search.low + search.high) / 2,);
    if (nonNullishOrThrow(positions[middle],) < from)
      search.low = middle + 1;
    else
      search.high = middle;
  }
  return search.low;
}

/**
 Where a run of words stands in the page, at every place from an index on.

 @param page - words to look in

 @param index - where each word of the page stands

 @param words - run of words to find

 @param from - first index a place may stand at

 @returns Index of the first word of each place the run stands, in order

 @example
 ```ts
 const places = placesOf({ page, index: indexOfWords({ page, }), words: ['a',], from: 0, },);
 ```
 */
function placesOf(
  {
    page,
    index,
    words,
    from,
  }: {
    readonly page: readonly string[];
    readonly index: WordIndex;
    readonly words: readonly string[];
    readonly from: number;
  },
): readonly number[] {
  /**
   Where the first word of the run stands in the page, in order.
   */
  const positions = index.get(nonNullishOrThrow(words[0],),) ?? [];
  return positions
    .slice(firstAtLeast({
      positions,
      from,
    },),)
    .filter(function standsHere(place,): boolean {
      return matchesAt({
        page,
        at: place,
        words,
      },);
    },);
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
 Places the words of one block no range reaches after each place the blocks before it can end at.

 The cost of a way of getting to a place is the cost of the place the blocks before ended at, and the blocks a range
 reaches when the words of the page between the two places are not what those blocks show. Only the place that is exactly
 as many words back as those blocks show can avoid that cost, so the cheapest way to a place is the cheaper of that one
 and the cheapest of every place the words can start after, which is read off the places in order and not asked of each
 pair. The places the blocks before can end at are in the order of the page, since every call writes them in the order
 of the places it reads, which come in the order of the page and end at the place plus the same number of words.

 @param page - words the page shows

 @param index - where each word of the page stands

 @param placings - where the blocks before can end, with their cost, in the order of the page

 @param segment - block's words, and how many blocks a range reaches stand since the last such block

 @param held - what the blocks a range reaches show since the last block no range reaches

 @param isLast - whether no block of the source follows

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
  }: {
    readonly page: readonly string[];
    readonly index: WordIndex;
    readonly placings: Placings;
    readonly segment: Segment;
    readonly held: Segment;
    readonly isLast: boolean;
  },
): Placings {
  /**
   The block's words.
   */
  const { words, } = segment;
  /**
   What the blocks a range reaches show since the last block no range reaches, and how many they are.
   */
  const {
    words: heldWords,
    blocks: heldBlocks,
  } = held;
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

  if (heldBlocks === 0) {
    // No block a range reaches stands between: the block starts where the blocks before it ended.
    for (const [place, cost,] of placings) {
      if (canEnd({ place, },) && matchesAt({
        page,
        at: place,
        words,
      },)) {
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
  const froms: Placing[] = [];
  for (const [place, cost,] of placings) {
    froms.push({
      place,
      cost,
    },);
  }
  /**
   The first place the blocks before can end at.
   */
  const [first,] = froms;
  if (first === undefined)
    return next;
  /**
   Where the block's words stand, the last block's only where they end the page, from the first place the blocks before it
   can end at.
   */
  const places = placesOf({
    page,
    index,
    words,
    from: first.place,
  },)
    .filter(function mayEnd(place,): boolean {
      return canEnd({ place, },);
    },);
  /**
   How many of those places the words start after, and the cheapest of them.
   */
  const reading = {
    taken: 0,
    cheapest: Number.POSITIVE_INFINITY,
  };
  for (const place of places) {
    for (;;) {
      /**
       The next place the blocks before can end at.
       */
      const from = froms[reading.taken];
      if ((from === undefined) || (from.place > place))
        break;
      reading.cheapest = Math.min(
        reading.cheapest,
        from.cost,
      );
      reading.taken += 1;
    }
    /**
     The place the blocks before ended at if the words of the page between it and this one are what the blocks a range
     reaches show.
     */
    const exact = place - heldWords.length;
    /**
     What getting there from that place costs, absent where the words are other or no block ended there.
     */
    const exactCost = matchesAt({
      page,
      at: exact,
      words: heldWords,
    },)
      ? placings.get(exact,)
      : undefined;
    /**
     The cheapest way to this place: from any place before it at the cost of the blocks a range reaches, or from the exact
     one at none.
     */
    const cost = Math.min(
      reading.cheapest + heldBlocks,
      exactCost ?? Number.POSITIVE_INFINITY,
    );
    if (Number.isFinite(cost,)) {
      keepCheapest({
        next,
        end: place + words.length,
        cost,
      },);
    }
  }
  return next;
}

//endregion Source reading places
