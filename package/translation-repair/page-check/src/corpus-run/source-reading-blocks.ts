import type { SourceBlock, } from './source-reading-account.ts';
import type { TextRange, } from './source-ships-ranges.ts';

//region Source reading blocks
// THE COMPARISON OF A PAGE WITH ITS SOURCE, BLOCK BY BLOCK. The two-readings ruling takes the page's reading of a footnote
// inside a block quote or a list item, which a reader of plain Markdown cannot give, so a difference in a block that
// holds such a footnote may be the ruling's own choice. That is a property of the blocks the footnote reaches and not of
// the whole text: every other top-level block of the source has a block of the page that must show the same, and a block
// that does not is a difference whatever stands beside it.
//
// WHICH BLOCKS THE FOOTNOTE REACHES. A range one reading has and the other lacks may run over blocks after the container
// that holds its start (the page goes on with a footnote over a line of raw HTML where the source ends it), so a block is
// reached when it overlaps the range, not only when it holds its first character.
//
// HOW THE BLOCKS ARE PAIRED. The blocks of the source that no such range reaches are anchors: each is looked for in the
// words the page shows, in order, wherever the page breaks them into blocks (the two readings split a list item from a
// line after it, or one block of raw HTML into paragraphs, with the same words). The words between two anchors are a
// stretch: what the blocks the ranges reach show against what the page shows there, which differ in length where the two
// readings end the footnote at different places, so a stretch is compared as a whole. An anchor whose words the page does
// not show where it comes is a difference, and so is a stretch that holds no block of the source and for which the page
// shows words; a stretch that does not show the same, with a block of the source in it, is undecided.

/**
 What the blocks of a source and of a page say of each other.
 */
export type BlockDifference = {
  /**
   Whether a block that no disagreeing range reaches has no block of the page to match, or the page has blocks the source
   has none for.
   */
  readonly differs: boolean;

  /**
   How many blocks of the source a disagreeing range reaches and show other text than the page's blocks in their stretch.
   */
  readonly undecidedBlocks: number;
};

/**
 Joins what blocks show into one account, as one block would show it.

 @param accounts - what each block shows

 @returns Accounts that show something, joined with a single space

 @example
 ```ts
 const joined = joinedAccounts({ accounts: ['a', '', 'b',], },); // 'a b'
 ```
 */
function joinedAccounts({ accounts, }: { readonly accounts: readonly string[]; },): string {
  return accounts
    .filter(function shows(account,): boolean {
      return account !== '';
    },)
    .join(' ',);
}

/**
 Whether two stretches of a text share a character.

 @param left - one stretch

 @param right - other stretch

 @returns Whether they overlap

 @example
 ```ts
 const overlap = overlaps({ left: { start: 0, end: 5, }, right: { start: 4, end: 9, }, },); // true
 ```
 */
function overlaps({
  left,
  right,
}: {
  readonly left: TextRange;
  readonly right: TextRange
},): boolean {
  return (left.start < right.end) && (right.start < left.end);
}

/**
 A run of blocks of the source that a disagreeing range reaches, or that none reaches, and the words they show.
 */
type Segment = {
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
 The blocks a range reaches when there are none: nothing shown, no block.
 */
const NO_BLOCKS: Segment = {
  reached: true,
  words: [],
  blocks: 0,
};

/**
 The words an account is made of.

 @param account - what a block shows

 @returns Its words, none for a block that shows nothing

 @example
 ```ts
 const words = wordsOf({ account: 'a b', },); // ['a', 'b']
 ```
 */
function wordsOf({ account, }: { readonly account: string; },): readonly string[] {
  return account
    .split(' ',)
    .filter(function isWord(word,): boolean {
      return word !== '';
    },);
}

/**
 Whether two lists of words are the same words in the same order.

 @param left - one list

 @param right - other list

 @returns Whether they are equal

 @example
 ```ts
 const same = sameWords({ left: ['a',], right: ['a',], },); // true
 ```
 */
function sameWords({
  left,
  right,
}: {
  readonly left: readonly string[];
  readonly right: readonly string[]
},): boolean {
  return (left.length === right.length) && left.every(function equal(
    word,
    at,
  ): boolean {
    return word === right[at];
  },);
}

/**
 Cuts the blocks of the source into runs of blocks a range reaches and of blocks none reaches.

 @param sourceBlocks - what each top-level block of the source shows

 @param reached - for each block, whether a range reaches it

 @returns The runs in order; a block that shows nothing and no range reaches is left out, since it can stand anywhere on
 the page

 @example
 ```ts
 const segments = segmentsOf({ sourceBlocks, reached: [false,], },);
 ```
 */
function segmentsOf(
  {
    sourceBlocks,
    reached,
  }: {
    readonly sourceBlocks: readonly SourceBlock[];
    readonly reached: readonly boolean[];
  },
): readonly Segment[] {
  /**
   Runs found.
   */
  const segments: Segment[] = [];
  for (const [at, block,] of sourceBlocks.entries()) {
    /**
     Words of the block.
     */
    const words = wordsOf({ account: block.account, },);
    /**
     Whether a range reaches the block.
     */
    const isReached = reached[at] === true;
    if ((!isReached) && (words.length === 0))
      continue;
    /**
     The run read last.
     */
    const last = segments.at(-1,);
    if (isReached && (last?.reached === true)) {
      segments[segments.length - 1] = {
        reached: true,
        words: [
          ...last.words,
          ...words,
        ],
        blocks: last.blocks + 1,
      };
      continue;
    }
    segments.push({
      reached: isReached,
      words,
      blocks: isReached ? 1 : 0,
    },);
  }
  return segments;
}

/**
 Where a run of words stands in a list of words, at every place.

 @param page - words to look in

 @param words - run of words to find

 @returns Index of the first word of each place the run stands, in order

 @example
 ```ts
 const places = placesOf({ page: ['a', 'b', 'a',], words: ['a',], },); // [0, 2]
 ```
 */
function placesOf({
  page,
  words,
}: {
  readonly page: readonly string[];
  readonly words: readonly string[]
},): readonly number[] {
  /**
   Places found.
   */
  const places: number[] = [];
  for (let at = 0; (at + words.length) <= page.length; at += 1) {
    if (sameWords({
      left: page.slice(
        at,
        at + words.length,
      ),
      right: words,
    },))
      places.push(at,);
  }
  return places;
}

/**
 How far the page is read once the words of the blocks no range reaches are placed in it, and what it cost: for each place
 the last such block can end at, the fewest blocks the readings cannot be asked about on the way there.
 */
type Placings = ReadonlyMap<number, number>;

/**
 Places the words of one block no range reaches after each place the blocks before it can end at.

 @param page - words the page shows

 @param placings - where the blocks before can end, with their cost

 @param segment - the block's words, and how many blocks a range reaches stand since the last such block

 @param held - what the blocks a range reaches show since the last block no range reaches

 @param isLast - whether no block of the source follows

 @returns Where the block can end, with the cost; none when the page shows its words nowhere it can come

 @example
 ```ts
 const next = placedAfter({ page, placings: new Map([[0, 0,],],), segment, held, isLast: false, },);
 ```
 */
function placedAfter(
  {
    page,
    placings,
    segment,
    held,
    isLast,
  }: {
    readonly page: readonly string[];
    readonly placings: Placings;
    readonly segment: Segment;
    readonly held: Segment;
    readonly isLast: boolean;
  },
): Placings {
  /**
   Where the block's words stand in the page.
   */
  const places = placesOf({
    page,
    words: segment.words,
  },);
  /**
   Where the block can end, with the cost.
   */
  const next = new Map<number, number>();
  for (const [from, cost,] of placings) {
    for (const place of places) {
      if ((place < from) || ((held.blocks === 0) && (place !== from)))
        continue;
      if (isLast && ((place
        + segment.words
          .length) !== page.length))
        continue;
      /**
       The words of the page between the last block and this one.
       */
      const gap = page.slice(
        from,
        place,
      );
      /**
       What the gap costs: the blocks a range reaches, when they do not show what the page shows there.
       */
      const gapCost = ((held.blocks > 0) && (!sameWords({
        left: gap,
        right: held.words,
      },))) ? held.blocks : 0;
      /**
       Where the block ends.
       */
      const end = place
        + segment.words
          .length;
      next.set(
        end,
        Math.min(
          next.get(end,) ?? Number.POSITIVE_INFINITY,
          cost + gapCost,
        ),
      );
    }
  }
  return next;
}

/**
 Compares the blocks of a source with the blocks of the page that carries it.

 @param sourceBlocks - what each top-level block of the source shows and where it stands

 @param pageBlocks - what each top-level block of the page shows

 @param disagreements - the footnote ranges one reading has and the other lacks that start inside a block quote or a list
 item, as offsets in the source

 @returns Whether a block differs on its own, and how many blocks the readings cannot be asked about

 @example
 ```ts
 const difference = blockDifferenceOf({
   sourceBlocks: [{ account: 'a', start: 0, end: 2, },],
   pageBlocks: ['a',],
   disagreements: [],
 },);
 ```
 */
export function blockDifferenceOf(
  {
    sourceBlocks,
    pageBlocks,
    disagreements,
  }: {
    readonly sourceBlocks: readonly SourceBlock[];
    readonly pageBlocks: readonly string[];
    readonly disagreements: readonly TextRange[];
  },
): BlockDifference {
  /**
   Whether a range reaches no block of the source, so that no block can be set apart as the one it is in.
   */
  const unplaced = disagreements.some(function reachesNone(range,): boolean {
    return !sourceBlocks.some(function reaches(block,): boolean {
      return overlaps({
        left: range,
        right: block,
      },);
    },);
  },);
  /**
   For each block of the source, whether a range reaches it.
   */
  const reached = sourceBlocks.map(function reachedOne(block,): boolean {
    return unplaced || disagreements.some(function reaches(range,): boolean {
      return overlaps({
        left: range,
        right: block,
      },);
    },);
  },);
  /**
   The words the page shows, whatever blocks it shows them in.
   */
  const page = pageBlocks.flatMap(function wordsOfBlock(account,): readonly string[] {
    return wordsOf({ account, },);
  },);
  /**
   The runs of blocks of the source.
   */
  const segments = segmentsOf({
    sourceBlocks,
    reached,
  },);
  /**
   How many blocks of the source a range reaches.
   */
  const reachedBlocks = segments.reduce(
    function counted(
      sum,
      segment,
    ): number {
      return sum + segment.blocks;
    },
    0,
  );
  /**
   Where the blocks no range reaches can end, and what that cost, as the segments are read in order.
   */
  const reading: {
    placings: Placings;
    held: Segment
  } = {
    placings: new Map([[
      0,
      0,
    ],],),
    held: NO_BLOCKS,
  };
  for (const [index, segment,] of segments.entries()) {
    if (segment.reached) {
      reading.held = segment;
      continue;
    }
    reading.placings = placedAfter({
      page,
      placings: reading.placings,
      segment,
      held: reading.held,
      isLast: index === (segments.length - 1),
    },);
    reading.held = NO_BLOCKS;
  }
  /**
   What the blocks a range reaches show after the last block no range reaches.
   */
  const trailing = reading.held;
  /**
   For each place the last block can end, what the page shows after it against what the trailing blocks show.
   */
  const ends = [...reading.placings
    .keys(),].map(function costAtEnd(from,): number {
    /**
     What the blocks cost up to that place.
     */
      const cost = reading.placings
        .get(from,)
        ?? 0;
    /**
     What the page shows after it.
     */
      const rest = page.slice(from,);
      if (trailing.blocks === 0)
        return (rest.length === 0) ? cost : Number.POSITIVE_INFINITY;
      return sameWords({
        left: rest,
        right: trailing.words,
      },) ? cost : (cost + trailing.blocks);
    },);
  /**
   The fewest blocks the readings cannot be asked about over every way the words can be placed.
   */
  const cheapest = Math.min(...ends,);
  if (Number.isFinite(cheapest,)) {
    return {
      differs: false,
      undecidedBlocks: cheapest,
    };
  }
  return {
    differs: true,
    undecidedBlocks: reachedBlocks,
  };
}

//endregion Source reading blocks
