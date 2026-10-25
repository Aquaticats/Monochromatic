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
 Where a list of words stands in another from an index on.

 @param page - words to look in

 @param words - words to find, a run of them

 @param from - first index to look at

 @param mustEnd - whether the run must be the last words of the list

 @returns Index of the first place the run stands, -1 when it stands nowhere

 @example
 ```ts
 const at = placeOf({ page: ['a', 'b',], words: ['b',], from: 0, mustEnd: false, },); // 1
 ```
 */
function placeOf(
  {
    page,
    words,
    from,
    mustEnd,
  }: {
    readonly page: readonly string[];
    readonly words: readonly string[];
    readonly from: number;
    readonly mustEnd: boolean;
  },
): number {
  /**
   Index of the last place a run of that many words can start.
   */
  const last = page.length - words.length;
  for (let at = mustEnd ? last : from; at <= last; at += 1) {
    if (sameWords({
      left: page.slice(
        at,
        at + words.length,
      ),
      right: words,
    },))
      return at;
    if (mustEnd)
      return -1;
  }
  return -1;
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
   Where the reading is: the next word of the page to look at, what the blocks reached since the last text found show and
   how many they are, and what has been found.
   */
  const found: {
    at: number;
    held: readonly string[];
    heldBlocks: number;
    differs: boolean;
    undecidedBlocks: number;
  } = {
    at: 0,
    held: [],
    heldBlocks: 0,
    differs: false,
    undecidedBlocks: 0,
  };

  /**
   Compares what the blocks reached show with the words of the page that stand in their place.

   @param gap - words of the page between the last text found and the next

   @example
   ```ts
   settleGap({ gap: ['a',], });
   ```
   */
  function settleGap({ gap, }: { readonly gap: readonly string[]; },): void {
    if (found.heldBlocks === 0) {
      found.differs ||= gap.length > 0;
    }
    else if (!sameWords({
      left: gap,
      right: found.held,
    },))
      found.undecidedBlocks += found.heldBlocks;
    found.held = [];
    found.heldBlocks = 0;
  }

  for (const [index, segment,] of segments.entries()) {
    if (segment.reached) {
      found.held = segment.words;
      found.heldBlocks = segment.blocks;
      continue;
    }
    /**
     Where the words of the block that no range reaches stand: at once after the last text found when nothing is held
     between, and for the last block at the end of the page.
     */
    const place = placeOf({
      page,
      words: segment.words,
      from: found.at,
      mustEnd: index === (segments.length - 1),
    },);
    if ((place === (-1)) || ((found.heldBlocks === 0) && (place !== found.at))) {
      found.differs = true;
      continue;
    }
    settleGap({
      gap: page.slice(
        found.at,
        place,
      ),
    },);
    found.at = place
      + segment.words
        .length;
  }
  settleGap({ gap: page.slice(found.at,), },);
  return {
    differs: found.differs,
    undecidedBlocks: found.undecidedBlocks,
  };
}

//endregion Source reading blocks
