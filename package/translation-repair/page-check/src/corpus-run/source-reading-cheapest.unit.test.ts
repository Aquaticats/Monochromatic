/**
 Tests for the cost some way of placing the blocks no range reaches is known to have, what the blocks after a block ask of the
 places it ends at, which stretches of a text share a character with a list, and how the comparison of the blocks of a
 source with a page grows with their number.

 The growth is measured as the least of several runs at each size, since a run alone varies by a third of itself: the
 sizes double, and the time of the least run is taken to at most double and a fifth over.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  blockDifferenceOf,
  boundOf,
  indexOfWords,
  lookaheadsOf,
  overlapIndexOf,
  reachesAny,
} from '../../dist/final/node/index.mjs';

/**
 The blocks of a source that has every block the same two words and a range reaching every fourth, and a page that shows them
 all: the shape the comparison took a quarter of an hour over at a hundred thousand blocks.

 @param blocks - how many blocks the source has

 @returns The arguments of the comparison

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

 @returns The least time of a run, in milliseconds

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
      name: boundOf.name,
      children: [
        it({
          name: 'IS NOTHING WHERE THE PAGE SHOWS THE SOURCE FROM ITS START TO ITS END',
          fn: async () => {
            const page = ['猫', '睡', '猫', '睡',];
            expect(boundOf({
              page,
              index: indexOfWords({ page, },),
              segments: [
                { reached: false, words: ['猫', '睡',], blocks: 0, },
                { reached: true, words: ['猫', '睡',], blocks: 1, },
              ],
            },),).toBe(0,);
          },
        },),
        it({
          name: 'IS THE BLOCKS A RANGE REACHES WHERE THE PAGE LOSES THEIR WORDS AND THE BLOCK AFTER THEM STANDS ON',
          fn: async () => {
            const page = ['猫', '睡', '鱼', '梦',];
            expect(boundOf({
              page,
              index: indexOfWords({ page, },),
              segments: [
                { reached: false, words: ['猫', '睡',], blocks: 0, },
                { reached: true, words: ['碗', '盆',], blocks: 2, },
                { reached: false, words: ['鱼', '梦',], blocks: 0, },
              ],
            },),).toBe(2,);
          },
        },),
        it({
          name: 'IS INFINITE WHERE A BLOCK NO RANGE REACHES STANDS NOWHERE IN THE PAGE',
          fn: async () => {
            const page = ['猫', '睡',];
            expect(boundOf({
              page,
              index: indexOfWords({ page, },),
              segments: [{ reached: false, words: ['鱼',], blocks: 0, },],
            },),).toBe(Number.POSITIVE_INFINITY,);
          },
        },),
      ],
    },),
    describe({
      name: lookaheadsOf.name,
      children: [
        it({
          name: 'GIVES EACH BLOCK THE WORDS OF THE BLOCKS UP TO THE NEXT WITH A STRETCH BEFORE IT, and what that stretch costs',
          fn: async () => {
            expect(lookaheadsOf({
              segments: [
                { reached: false, words: ['a',], blocks: 0, },
                { reached: true, words: ['x',], blocks: 3, },
                { reached: false, words: ['b',], blocks: 0, },
                { reached: false, words: ['c', 'd',], blocks: 0, },
                { reached: true, words: [], blocks: 1, },
                { reached: false, words: ['e',], blocks: 0, },
              ],
            },),).toEqual([
              { words: [], cost: Number.POSITIVE_INFINITY, },
              { words: [], cost: Number.POSITIVE_INFINITY, },
              { words: ['c', 'd',], cost: 1, },
              { words: [], cost: Number.POSITIVE_INFINITY, },
              { words: [], cost: Number.POSITIVE_INFINITY, },
              { words: [], cost: Number.POSITIVE_INFINITY, },
            ],);
          },
        },),
      ],
    },),
    describe({
      name: overlapIndexOf.name,
      children: [
        it({
          name: 'SAYS WHETHER A STRETCH SHARES A CHARACTER WITH ANY OF A LIST, in any order of the list',
          fn: async () => {
            const index = overlapIndexOf({
              ranges: [
                { start: 20, end: 30, },
                { start: 0, end: 5, },
                { start: 9, end: 9, },
              ],
            },);
            expect(reachesAny({ index, range: { start: 4, end: 6, }, },),).toBe(true,);
            expect(reachesAny({ index, range: { start: 5, end: 8, }, },),).toBe(false,);
            expect(reachesAny({ index, range: { start: 8, end: 10, }, },),).toBe(true,);
            expect(reachesAny({ index, range: { start: 9, end: 12, }, },),).toBe(false,);
            expect(reachesAny({ index, range: { start: 29, end: 40, }, },),).toBe(true,);
            expect(reachesAny({ index, range: { start: 30, end: 40, }, },),).toBe(false,);
            expect(reachesAny({ index: overlapIndexOf({ ranges: [], },), range: { start: 0, end: 9, }, },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: blockDifferenceOf.name,
      children: [
        it({
          name: 'TAKES AT MOST TWICE AND A FIFTH THE TIME WHEN THE BLOCKS DOUBLE, for blocks that all show the same words',
          fn: async () => {
            /**
             The least time at each size, after a run to warm the code up.
             */
            blockDifferenceOf(sameWordsOf({ blocks: 12_800, },),);
            const times = [25_600, 51_200, 102_400,].map(function timeAt(blocks,): number {
              return leastTimeOf({
                input: sameWordsOf({ blocks, },),
                runs: 9,
              },);
            },);
            /**
             What the comparison says of the blocks that show the same words.
             */
            const answer = blockDifferenceOf(sameWordsOf({ blocks: 51_200, },),);
            expect(answer,).toEqual({ differs: false, undecidedBlocks: 0, },);
            for (const [at, time,] of times.entries()) {
              if (at === 0)
                continue;
              expect(time,).toBeLessThan((times[at - 1] ?? 0) * 2.2,);
            }
          },
        },),
      ],
    },),
  ],
},);
