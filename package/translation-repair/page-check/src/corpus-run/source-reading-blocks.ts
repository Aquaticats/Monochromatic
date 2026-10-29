import type { SourceBlock, } from './source-reading-account.ts';
import { cheapestPlacing, } from './source-reading-cheapest.ts';
import {
  overlapIndexOf,
  reachesAny,
} from './source-reading-overlap.ts';
import type { Segment, } from './source-reading-words.ts';
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
//
// THE WORK. Each anchor is placed in one pass over the places that matter (`cheapestPlacing` in `source-reading-cheapest.ts`),
// read off an index of where each word of the page stands, so the work grows with the places an anchor can end at that no
// other place makes useless and not with the blocks times the words of the page, nor with their cube, however often the
// same words repeat.

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
 Cuts the blocks of the source into runs of blocks a range reaches and of blocks none reaches.

 @param sourceBlocks - what each top-level block of the source shows

 @param reached - for each block, whether a range reaches it

 @returns Runs in order; a block that shows nothing and no range reaches is left out, since it can stand anywhere on
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
 Compares the blocks of a source with the blocks of the page that carries it.

 @param sourceBlocks - what each top-level block of the source shows and where it stands

 @param pageBlocks - what each top-level block of the page shows

 @param disagreements - footnote ranges one reading has and the other lacks that start inside a block quote or a list
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
   The blocks of the source and the ranges, put in the order of their starts to be asked which share a character.
   */
  const blockIndex = overlapIndexOf({ ranges: sourceBlocks, },);
  /**
   The same for the ranges.
   */
  const rangeIndex = overlapIndexOf({ ranges: disagreements, },);
  /**
   Whether a range reaches no block of the source, so that no block can be set apart as the one it is in.
   */
  const unplaced = disagreements.some(function reachesNone(range,): boolean {
    return !reachesAny({
      index: blockIndex,
      range,
    },);
  },);
  /**
   For each block of the source, whether a range reaches it.
   */
  const reached = sourceBlocks.map(function reachedOne(block,): boolean {
    return unplaced || reachesAny({
      index: rangeIndex,
      range: block,
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
   The fewest blocks the readings cannot be asked about over every way the words can be placed.
   */
  const cheapest = cheapestPlacing({
    page,
    segments,
  },);
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
