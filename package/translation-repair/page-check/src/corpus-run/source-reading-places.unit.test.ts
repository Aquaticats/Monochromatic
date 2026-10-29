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
} from '@monochromatic-dev/test-harness-core/ts';

import {
  cheapestPlacing,
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
 The cheapest way to place the unreached runs of a list in a page, read through {@link cheapestPlacing} as the comparison reads it.

 @param page - words the page shows

 @param runs - runs of the source, in order

 @returns Fewest blocks the readings cannot be asked about; infinity where the words cannot be placed

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
  return cheapestPlacing({
    page,
    segments: runs,
  },);
}

/**
 The same cheapest cost found by placing each run after every place the runs before it can end at, pair by pair, with no
 place left out: the reading the comparison made before it left out the places that cannot be of use.

 @param page - words the page shows

 @param runs - runs of the source, in order

 @returns Fewest blocks the readings cannot be asked about; infinity where the words cannot be placed

 @example
 ```ts
 const cost = cheapestByPairs({ page: ['a',], runs: [{ reached: false, words: ['a',], blocks: 0, },], },);
 ```
 */
function cheapestByPairs(
  {
    page,
    runs,
  }: {
    readonly page: readonly string[];
    readonly runs: readonly Run[];
  },
): number {
  /**
   Where the unreached runs read so far can end, with the cost, and the reached run since the last of them.
   */
  const reading: { ends: Map<number, number>; held: Run; } = {
    ends: new Map([[0, 0,],],),
    held: NO_BLOCKS,
  };
  for (const [at, run,] of runs.entries()) {
    if (run.reached) {
      reading.held = run;
      continue;
    }
    /**
     Where the run can end after each place the runs before it end at.
     */
    const next = new Map<number, number>();
    for (let start = 0; (start + run.words.length) <= page.length; start += 1) {
      if (!matchesAt({ page, at: start, words: run.words, },))
        continue;
      if ((at === (runs.length - 1)) && ((start + run.words.length) !== page.length))
        continue;
      for (const [from, cost,] of reading.ends) {
        /**
         The cost of the words between the place and the start, against the blocks a range reaches that stand there.
         */
        const gap = (start < from)
          ? Number.POSITIVE_INFINITY
          : (reading.held.blocks === 0)
            ? ((start === from) ? 0 : Number.POSITIVE_INFINITY)
            : ((((start - from) === reading.held.words.length) && matchesAt({ page, at: from, words: reading.held.words, },))
              ? 0
              : reading.held.blocks);
        next.set(start + run.words.length, Math.min(next.get(start + run.words.length,) ?? Number.POSITIVE_INFINITY, cost + gap,),);
      }
    }
    reading.ends = new Map([...next,].filter(function finite([, cost,],): boolean {
      return Number.isFinite(cost,);
    },),);
    reading.held = NO_BLOCKS;
  }
  /**
   What each place the last run can end at costs, with what the page shows after it set against the trailing blocks.
   */
  const costs = [...reading.ends,].map(function costAt([from, cost,],): number {
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

 @returns Fewest blocks the readings cannot be asked about; infinity where the words cannot be placed

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

   @returns Words

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

/**
 Runs of the source and a page made at random from a seed, longer than those of {@link configurationFrom} and, for half the
 seeds, a page that shows the words of the source with a few words changed: the case the certificate of the diagonal that
 ends the page and the lookahead are made for.

 @param seed - state of the generator

 @returns Page and runs: up to sixteen runs of a vocabulary of two words

 @example
 ```ts
 const { page, runs, } = longConfigurationFrom({ seed: 7, },);
 ```
 */
function longConfigurationFrom({ seed, }: { readonly seed: number; },): { readonly page: readonly string[]; readonly runs: readonly Run[]; } {
  /**
   The generator.
   */
  const rand = randomOf({ seed, },);

  /**
   Words drawn from a vocabulary of two.

   @param count - how many words

   @returns Words

   @example
   ```ts
   const words = wordsOf({ count: 2, });
   ```
   */
  function wordsOf({ count, }: { readonly count: number; },): readonly string[] {
    return Array.from({ length: count, }, function word(): string {
      return ['a', 'b',][integerBelow({ rand, bound: 2, },)] ?? 'a';
    },);
  }

  /**
   The runs, alternating at random between reached and unreached, and the words they show in order.
   */
  const runs: Run[] = [];
  for (let at = 0, reached = integerBelow({ rand, bound: 2, },) === 0; at < (3 + integerBelow({ rand, bound: 14, },)); at += 1, reached = !reached) {
    runs.push(reached
      ? { reached, words: wordsOf({ count: integerBelow({ rand, bound: 4, },), },), blocks: 1 + integerBelow({ rand, bound: 3, },), }
      : { reached, words: wordsOf({ count: 1 + integerBelow({ rand, bound: 4, },), },), blocks: 0, },);
  }
  if (integerBelow({ rand, bound: 2, },) === 0) {
    return {
      page: wordsOf({ count: 8 + integerBelow({ rand, bound: 40, },), },),
      runs,
    };
  }
  /**
   The words of the source, changed at a few places.
   */
  const page: string[] = runs.flatMap(function wordsOfRun(run,): readonly string[] {
    return run.words;
  },);
  for (let edits = integerBelow({ rand, bound: 4, },); edits > 0; edits -= 1) {
    /**
     Where, and which of the three changes.
     */
    const at = integerBelow({ rand, bound: page.length + 1, },);
    const change = integerBelow({ rand, bound: 3, },);
    if (change === 0)
      page.splice(at, 1,);
    else if (change === 1)
      page.splice(at, 1, ...wordsOf({ count: 1, },),);
    else
      page.splice(at, 0, ...wordsOf({ count: 1 + integerBelow({ rand, bound: 3, },), },),);
  }
  return {
    page,
    runs,
  };
}

await describe({
  name: 'where the words of the blocks no range reaches stand in the page',
  children: [
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
        it({
          name: 'FINDS THE CHEAPEST WAY TO PLACE THE BLOCKS THAT PLACING EACH AFTER EVERY PLACE THE ONES BEFORE CAN END AT FINDS, over longer texts',
          fn: async () => {
            /**
             How many configurations were compared, how many could be placed, and how many cost nothing.
             */
            const counted = { compared: 0, placeable: 0, free: 0, };
            for (let seed = 1; seed <= 4_000; seed += 1) {
              const { page, runs, } = longConfigurationFrom({ seed, },);
              /**
               The cost by the placing, and by every pair of a run and a place.
               */
              const placed = cheapestByPlacing({ page, runs, },);
              const paired = cheapestByPairs({ page, runs, },);
              expect(placed,).toBe(paired,);
              counted.compared += 1;
              if (Number.isFinite(paired,))
                counted.placeable += 1;
              if (paired === 0)
                counted.free += 1;
            }
            expect(counted.compared,).toBe(4_000,);
            expect(counted.placeable,).toBeGreaterThan(1_000,);
            expect(counted.free,).toBeGreaterThan(100,);
          },
        },),
      ],
    },),
  ],
},);
