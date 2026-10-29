/**
 Tests for the cost some way of placing the blocks no range reaches is known to have, what the blocks after a block ask of the
 places it ends at, which stretches of a text share a character with a list, and how the comparison of the blocks of a
 source with a page grows with their number.

 The growth is measured as the least of several runs at each of two sizes four times apart. On a quiet machine the least run
 at four times the blocks took 3.1 times the time of the smaller (1.7 to 1.9 for each doubling), and under the load of the
 whole suite 5.6 times; a placing that costs the square of the blocks takes 16 times, so the larger size may take less than
 eight times the time of the smaller.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  blockDifferenceOf,
} from '../../dist/final/node/index.mjs';

/**
 How many times the time of the smaller size the larger may take, at four times the blocks.
 */
const MOST_GROWTH = 8;

/**
 The blocks of a source that has every block the same two words and a range reaching every fourth, and a page that shows them
 all: the shape the comparison took a quarter of an hour over at a hundred thousand blocks.

 @param blocks - how many blocks the source has

 @returns Arguments of the comparison

 @example
 ```ts
 const input = sameWordsOf({ blocks: 8, },);
 ```
 */
function sameWordsOf({ blocks, }: { readonly blocks: number; },): Parameters<typeof blockDifferenceOf>[0] {
  /**
   What each block shows.
   */
  const accounts = Array.from({ length: blocks, }, function account(): string {
    return '猫 睡';
  },);
  /**
   Where each block stands in the source.
   */
  const sourceBlocks = accounts.map(function placed(
    account,
    at,
  ): { readonly account: string; readonly start: number; readonly end: number; } {
    return {
      account,
      start: at * 10,
      end: (at * 10) + 5,
    };
  },);
  return {
    sourceBlocks,
    pageBlocks: [accounts.join(' ',),],
    disagreements: sourceBlocks
      .filter(function everyFourth(
        _block,
        at,
      ): boolean {
        return (at % 4) === 1;
      },)
      .map(function insideBlock({ start, },): { readonly start: number; readonly end: number; } {
        return {
          start: start + 1,
          end: start + 2,
        };
      },),
  };
}

/**
 The least time of several runs of the comparison.

 @param input - arguments of the comparison

 @param runs - how many times to run it

 @returns Least time of a run, in milliseconds

 @example
 ```ts
 const least = leastTimeOf({ input: sameWordsOf({ blocks: 8, },), runs: 3, },);
 ```
 */
function leastTimeOf(
  {
    input,
    runs,
  }: {
    readonly input: Parameters<typeof blockDifferenceOf>[0];
    readonly runs: number;
  },
): number {
  /**
   The times.
   */
  const times = Array.from({ length: runs, }, function timed(): number {
    /**
     When the run started.
     */
    const started = performance.now();
    blockDifferenceOf(input,);
    return performance.now() - started;
  },);
  return Math.min(...times,);
}

await describe({
  name: 'the cost some way of placing the blocks is known to have, and how the comparison of the blocks grows',
  children: [
    describe({
      name: blockDifferenceOf.name,
      children: [
        it({
          name: 'TAKES LESS THAN EIGHT TIMES THE TIME WHEN THE BLOCKS QUADRUPLE, for blocks that all show the same words',
          fn: async () => {
            // A run first, to warm the code up.
            blockDifferenceOf(sameWordsOf({ blocks: 12_800, },),);
            /**
             The least time at each of two sizes four times apart.
             */
            const [small, large,] = [25_600, 102_400,].map(function timeAt(blocks,): number {
              return leastTimeOf({
                input: sameWordsOf({ blocks, },),
                runs: 11,
              },);
            },);
            /**
             What the comparison says of the blocks that show the same words.
             */
            const answer = blockDifferenceOf(sameWordsOf({ blocks: 51_200, },),);
            expect(answer,).toEqual({ differs: false, undecidedBlocks: 0, },);
            expect(large,).toBeLessThan((small ?? 0) * MOST_GROWTH,);
          },
        },),
      ],
    },),
  ],
},);
