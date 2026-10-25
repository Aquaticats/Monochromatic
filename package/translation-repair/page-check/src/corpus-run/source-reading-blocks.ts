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
// HOW THE BLOCKS ARE PAIRED. The blocks of the source that no such range reaches are anchors: each is looked for among the
// blocks of the page, in order, by what it shows. The blocks between two anchors are a stretch: the source's blocks the
// ranges reach against the page's blocks, which differ in number where the two readings end the footnote at different
// places, so a stretch is compared as a whole. An anchor the page has no block for is a difference, and so is a stretch
// that holds no block of the source and for which the page has blocks; a stretch that does not show the same, with a block
// of the source in it, is undecided.

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
   What the blocks of the source that are reached show, since the last anchor.
   */
  const stretch: string[] = [];
  /**
   Where the reading is: the next block of the page to look at, and what has been found.
   */
  const found = {
    page: 0,
    differs: false,
    undecidedBlocks: 0,
  };

  /**
   Compares the stretch read so far with the blocks of the page up to an index, and starts the next stretch there.

   @param upTo - index of the first block of the page that is no part of the stretch

   @example
   ```ts
   settle({ upTo: 3, });
   ```
   */
  function settle({ upTo, }: { readonly upTo: number; },): void {
    /**
     What the page's blocks of the stretch show.
     */
    const page = joinedAccounts({
      accounts: pageBlocks.slice(
        found.page,
        upTo,
      ),
    },);
    if (joinedAccounts({ accounts: stretch, },) !== page) {
      if (stretch.length === 0)
        found.differs = true;
      else
        found.undecidedBlocks += stretch.length;
    }
    stretch.length = 0;
    found.page = upTo;
  }

  for (const [at, block,] of sourceBlocks.entries()) {
    if (reached[at] === true) {
      stretch.push(block.account,);
      continue;
    }
    /**
     Index of the block of the page that shows what this anchor shows, none before the stretch's end being taken.
     */
    const anchor = pageBlocks.findIndex(function showsIt(
      account,
      index,
    ): boolean {
      return (index >= found.page) && (account === block.account);
    },);
    if (anchor === (-1)) {
      found.differs = true;
      continue;
    }
    settle({ upTo: anchor, },);
    found.page = anchor + 1;
  }
  settle({ upTo: pageBlocks.length, },);
  return {
    differs: found.differs,
    undecidedBlocks: found.undecidedBlocks,
  };
}

//endregion Source reading blocks
