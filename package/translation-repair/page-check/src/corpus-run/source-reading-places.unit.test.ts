/**
 Tests for where the words of the blocks no range reaches stand in the words of the page, and what reading the page so costs.

 The placing is checked against an enumeration of every way to place the blocks, which takes the cheapest of them as the
 definition does, on texts too small for the enumeration to be slow.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  indexOfWords,
  matchesAt,
  placedAfter,
} from '../../dist/final/node/index.mjs';
import {
  integerBelow,
  randomOf,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/seeded-draws.test-fixture.ts';

/**
 A run of blocks of the source, as the placing is given it.
 */
type Run = {
  readonly reached: boolean;
  readonly words: readonly string[];
  readonly blocks: number;
};

/**
 The blocks a range reaches when there are none.
 */
const NO_BLOCKS: Run = { reached: true, words: [], blocks: 0, };

/**
 The cheapest way to place the unreached runs of a list in a page, read through {@link placedAfter} as the comparison reads it.

 @param page - words the page shows

 @param runs - runs of the source, in order

 @returns The fewest blocks the readings cannot be asked about; infinity where the words cannot be placed

 @example
 ```ts
 const cost = cheapestByPlacing({ page: ['a',], runs: [{ reached: false, words: ['a',], blocks: 0, },], },);
 ```
 */
function cheapestByPlacing(
  {
    page,
    runs,
  }: {
    readonly page: readonly string[];
    readonly runs: readonly Run[];
  },
): number {
  /**
   Where the unreached runs read so far can end, and the reached run since the last of them.
   */
  const reading: { placings: ReadonlyMap<number, number>; held: Run; } = {
    placings: new Map([[0, 0,],],),
    held: NO_BLOCKS,
  };
  /**
   Where each word of the page stands.
   */
  const index = indexOfWords({ page, },);
  for (const [at, run,] of runs.entries()) {
    if (run.reached) {
      reading.held = run;
      continue;
    }
    reading.placings = placedAfter({
      page,
      index,
      placings: reading.placings,
      segment: run,
      held: reading.held,
      isLast: at === (runs.length - 1),
    },);
    reading.held = NO_BLOCKS;
  }
  /**
   What each place the last run can end at costs, with what the page shows after it set against the trailing blocks.
   */
  const costs = [...reading.placings,].map(function costAt([from, cost,],): number {
    /**
     Whether the page shows after the place the words of the trailing blocks and nothing else.
     */
    const same = ((page.length - from) === reading.held.words.length)
      && matchesAt({ page, at: from, words: reading.held.words, },);
    if (reading.held.blocks === 0)
      return same ? cost : Number.POSITIVE_INFINITY;
    return same ? cost : (cost + reading.held.blocks);
  },);
  return Math.min(...costs,);
}

/**
 The same cheapest cost found by trying every way to place the unreached runs.

 @param page - words the page shows

 @param runs - runs of the source, in order

 @returns The fewest blocks the readings cannot be asked about; infinity where the words cannot be placed

 @example
 ```ts
 const cost = cheapestByEnumeration({ page: ['a',], runs: [{ reached: false, words: ['a',], blocks: 0, },], },);
 ```
 */
function cheapestByEnumeration(
  {
    page,
    runs,
  }: {
    readonly page: readonly string[];
    readonly runs: readonly Run[];
  },
): number {
  /**
   The cost of the words between two places, against the blocks a range reaches that stand there.
   */
  const gapCost = (
    from: number,
    to: number,
    held: Run,
  ): number => {
    if (to < from)
      return Number.POSITIVE_INFINITY;
    if (held.blocks === 0)
      return (to === from) ? 0 : Number.POSITIVE_INFINITY;
    return ((to - from) === held.words.length) && matchesAt({ page, at: from, words: held.words, },) ? 0 : held.blocks;
  };

  /**
   The cheapest way to place the runs from an index of the list on, the previous run ending at a place.
   */
  const best = (
    at: number,
    end: number,
    held: Run,
  ): number => {
    if (at >= runs.length) {
      const rest = page.length;
      return gapCost(end, rest, held);
    }
    const run = runs[at];
    if (run === undefined)
      return Number.POSITIVE_INFINITY;
    if (run.reached)
      return best(at + 1, end, run,);
    let cheapest = Number.POSITIVE_INFINITY;
    for (let start = end; (start + run.words.length) <= page.length; start += 1) {
      if (!matchesAt({ page, at: start, words: run.words, },))
        continue;
      const gap = gapCost(end, start, held,);
      if (!Number.isFinite(gap,))
        continue;
      cheapest = Math.min(cheapest, gap + best(at + 1, start + run.words.length, NO_BLOCKS,),);
    }
    return cheapest;
  };

  return best(0, 0, NO_BLOCKS,);
}

/**
 Runs of the source and a page made at random from a seed.

 @param seed - state of the generator

 @returns Page and runs: unreached runs of one to three words, reached runs of up to three words and one to three blocks

 @example
 ```ts
 const { page, runs, } = configurationFrom({ seed: 7, },);
 ```
 */
function configurationFrom({ seed, }: { readonly seed: number; },): { readonly page: readonly string[]; readonly runs: readonly Run[]; } {
  /**
   The generator.
   */
  const rand = randomOf({ seed, },);

  /**
   Words drawn from a vocabulary of two or three.

   @param count - how many words

   @returns The words

   @example
   ```ts
   const words = wordsOf({ count: 2, });
   ```
   */
  function wordsOf({ count, }: { readonly count: number; },): readonly string[] {
    return Array.from({ length: count, }, function word(): string {
      return ['a', 'b', 'c',][integerBelow({ rand, bound: 3, },)] ?? 'a';
    },);
  }

  /**
   The runs, alternating at random between reached and unreached.
   */
  const runs: Run[] = [];
  for (let at = 0, reached = integerBelow({ rand, bound: 2, },) === 0; at < (1 + integerBelow({ rand, bound: 5, },)); at += 1, reached = !reached) {
    runs.push(reached
      ? { reached, words: wordsOf({ count: integerBelow({ rand, bound: 4, },), },), blocks: 1 + integerBelow({ rand, bound: 3, },), }
      : { reached, words: wordsOf({ count: 1 + integerBelow({ rand, bound: 3, },), },), blocks: 0, },);
  }
  return {
    page: wordsOf({ count: integerBelow({ rand, bound: 9, },), },),
    runs,
  };
}

await describe({
  name: 'where the words of the blocks no range reaches stand in the page',
  children: [
    describe({
      name: indexOfWords.name,
      children: [
        it({
          name: 'LISTS WHERE EACH WORD OF THE PAGE STANDS, in order',
          fn: async () => {
            expect([...indexOfWords({ page: ['a', 'b', 'a', 'c', 'a',], },),],).toEqual([
              ['a', [0, 2, 4,],],
              ['b', [1,],],
              ['c', [3,],],
            ],);
            expect([...indexOfWords({ page: [], },),],).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: matchesAt.name,
      children: [
        it({
          name: 'SAYS WHETHER A RUN OF WORDS STANDS AT AN INDEX, a run that does not fit never',
          fn: async () => {
            expect(matchesAt({ page: ['a', 'b', 'c',], at: 1, words: ['b', 'c',], },),).toBe(true,);
            expect(matchesAt({ page: ['a', 'b', 'c',], at: 1, words: ['b', 'd',], },),).toBe(false,);
            expect(matchesAt({ page: ['a', 'b', 'c',], at: 2, words: ['c', 'd',], },),).toBe(false,);
            expect(matchesAt({ page: ['a', 'b', 'c',], at: -1, words: ['a',], },),).toBe(false,);
          },
        },),
        it({
          name: 'SAYS AN EMPTY RUN STANDS ANYWHERE FROM THE START OF THE PAGE TO ITS END',
          fn: async () => {
            expect(matchesAt({ page: ['a',], at: 0, words: [], },),).toBe(true,);
            expect(matchesAt({ page: ['a',], at: 1, words: [], },),).toBe(true,);
            expect(matchesAt({ page: ['a',], at: 2, words: [], },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: placedAfter.name,
      children: [
        it({
          name: 'PLACES A BLOCK ONLY WHERE THE BLOCKS BEFORE ENDED WHEN NO BLOCK A RANGE REACHES STANDS BETWEEN, in the order of the page',
          fn: async () => {
            /**
             Words of the page, and the places two blocks before can end at.
             */
            const page = ['a', 'b', 'a', 'b',];
            const next = placedAfter({
              page,
              index: indexOfWords({ page, },),
              placings: new Map([[0, 0,], [1, 3,], [2, 1,],],),
              segment: { reached: false, words: ['a', 'b',], blocks: 0, },
              held: NO_BLOCKS,
              isLast: false,
            },);
            expect([...next,],).toEqual([[2, 0,], [4, 1,],],);
          },
        },),
        it({
          name: 'PLACES THE LAST BLOCK ONLY WHERE IT ENDS THE PAGE',
          fn: async () => {
            const page = ['a', 'b', 'a', 'b',];
            const next = placedAfter({
              page,
              index: indexOfWords({ page, },),
              placings: new Map([[0, 0,], [2, 1,],],),
              segment: { reached: false, words: ['a', 'b',], blocks: 0, },
              held: NO_BLOCKS,
              isLast: true,
            },);
            expect([...next,],).toEqual([[4, 1,],],);
          },
        },),
        it({
          name: 'PLACES A BLOCK AFTER BLOCKS A RANGE REACHES AT THE COST OF THEM, or at none where the page shows what they show',
          fn: async () => {
            const page = ['a', 'x', 'b', 'c', 'b',];
            const next = placedAfter({
              page,
              index: indexOfWords({ page, },),
              placings: new Map([[1, 2,],],),
              segment: { reached: false, words: ['b',], blocks: 0, },
              held: { reached: true, words: ['x',], blocks: 2, },
              isLast: false,
            },);
            expect([...next,],).toEqual([[3, 2,], [5, 4,],],);
          },
        },),
        it({
          name: 'PLACES NOTHING WHERE NO BLOCK BEFORE CAN END',
          fn: async () => {
            const page = ['a',];
            expect(placedAfter({
              page,
              index: indexOfWords({ page, },),
              placings: new Map(),
              segment: { reached: false, words: ['a',], blocks: 0, },
              held: { reached: true, words: [], blocks: 1, },
              isLast: false,
            },).size,).toBe(0,);
          },
        },),
        it({
          name: 'FINDS THE CHEAPEST WAY TO PLACE THE BLOCKS THAT AN ENUMERATION OF EVERY WAY FINDS, over thousands of small texts',
          fn: async () => {
            /**
             How many configurations were compared and how many could be placed at all.
             */
            const counted = { compared: 0, placeable: 0, };
            for (let seed = 1; seed <= 6_000; seed += 1) {
              const { page, runs, } = configurationFrom({ seed, },);
              /**
               The cost by the placing, and by enumeration.
               */
              const placed = cheapestByPlacing({ page, runs, },);
              const enumerated = cheapestByEnumeration({ page, runs, },);
              expect(placed,).toBe(enumerated,);
              counted.compared += 1;
              if (Number.isFinite(enumerated,))
                counted.placeable += 1;
            }
            expect(counted.compared,).toBe(6_000,);
            expect(counted.placeable,).toBeGreaterThan(1_000,);
            expect(counted.placeable,).toBeLessThan(5_000,);
          },
        },),
      ],
    },),
  ],
},);
