import type { SourceBlock, } from './source-reading-account.ts';

//region Source reading blocks
// THE COMPARISON OF A PAGE WITH ITS SOURCE, BLOCK BY BLOCK. The two-readings ruling takes the page's reading of a footnote
// inside a block quote or a list item, which a reader of plain Markdown cannot give, so a difference in a block that
// holds such a footnote may be the ruling's own choice. That is a property of the blocks that hold the footnote and not of
// the whole text: every other top-level block of the source has a block of the page that must show the same, and a block
// that does not is a difference whatever stands beside it.
//
// HOW THE BLOCKS ARE PAIRED. The blocks of the source that hold no such footnote are anchors: each is looked for among the
// blocks of the page, in order, by what it shows. The blocks between two anchors are a stretch: the source's blocks that
// hold the footnote against the page's blocks, which differ in number where the two readings end the footnote at
// different places, so a stretch is compared as a whole. An anchor the page has no block for is a difference, and so is a
// stretch with no block of the source that the page has blocks in; a stretch that does not show the same, with a block of
// the source in it, is undecided.

/**
 What the blocks of a source and of a page say of each other.
 */
export type BlockDifference = {
  /**
   Whether a block that holds no footnote the readings disagree on has no block of the page to match, or the page has
   blocks the source has none for.
   */
  readonly differs: boolean;

  /**
   How many blocks of the source hold a footnote the readings disagree on and show other text than the page's blocks in
   their stretch.
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
 Compares the blocks of a source with the blocks of the page that carries it.

 @param sourceBlocks - what each top-level block of the source shows and where it stands

 @param pageBlocks - what each top-level block of the page shows

 @param disagreements - offsets in the source where a footnote range one reading has and the other lacks starts inside a
 block quote or a list item

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
    readonly disagreements: readonly number[];
  },
): BlockDifference {
  /**
   Whether a disagreement stands in no block of the source, so that no block can be set apart as the one it is in.
   */
  const unplaced = disagreements.some(function inNoBlock(at,): boolean {
    return !sourceBlocks.some(function holds({
      start,
      end,
    },): boolean {
      return (start <= at) && (at < end);
    },);
  },);
  /**
   For each block of the source, whether it holds a footnote the readings disagree on.
   */
  const holdsDisagreement = sourceBlocks.map(function holdsOne({
    start,
    end,
  },): boolean {
    return unplaced || disagreements.some(function inside(at,): boolean {
      return (start <= at) && (at < end);
    },);
  },);
  /**
   Where the reading is: the next block of the page to look at, what the stretch since the last anchor holds, and what
   has been found.
   */
  const state = {
    page: 0,
    stretch: [] as string[],
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
    const page = joinedAccounts({ accounts: pageBlocks.slice(
      state.page,
      upTo,
    ), },);
    if (joinedAccounts({ accounts: state.stretch, },) !== page) {
      if (state.stretch
        .length
        === 0)
        state.differs = true;
      else
        state.undecidedBlocks += state.stretch
          .length;
    }
    state.stretch = [];
    state.page = upTo;
  }

  for (const [at, block,] of sourceBlocks.entries()) {
    if (holdsDisagreement[at] === true) {
      state.stretch
        .push(block.account,);
      continue;
    }
    /**
     Index of the block of the page that shows what this anchor shows, none before the stretch's end being taken.
     */
    const found = pageBlocks.findIndex(function showsIt(
      account,
      index,
    ): boolean {
      return (index >= state.page) && (account === block.account);
    },);
    if (found === (-1)) {
      state.differs = true;
      continue;
    }
    settle({ upTo: found, },);
    state.page = found + 1;
  }
  settle({ upTo: pageBlocks.length, },);
  return {
    differs: state.differs,
    undecidedBlocks: state.undecidedBlocks,
  };
}

//endregion Source reading blocks
