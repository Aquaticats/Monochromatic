import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';

//region Source reading words
// THE WORDS OF A PAGE AND WHERE EACH STANDS: the runs of blocks of a source as the placing reads them, the places the blocks
// end at, an index of the page, and the questions asked of it (`source-reading-places.ts` places the words with them).

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
export type Placing = {
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
export function firstAtLeast(
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

//endregion Source reading words
