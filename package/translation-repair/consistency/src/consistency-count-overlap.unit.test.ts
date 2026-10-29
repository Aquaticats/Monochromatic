/**
 Tests for taking the rendering of every place whose located words overlap another place of the same slice: the same
 answer as comparing every pair of places, at the cost of one sort for each slice.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ConsistencyRendering,
  unlessOverlapping,
} from '../dist/final/node/index.mjs';
import {
  countedList,
  type ElementReads,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/element-reads.test-fixture.ts';

/**
 A located rendering of some words at an offset.

 @param start - offset where the words start

 @param length - how many units the words take, at least one

 @returns Located rendering

 @example
 ```ts
 const rendering = locatedAt({ start: 4, length: 3, },); // the words at offsets 4 to 7
 ```
 */
function locatedAt(
  {
    start,
    length,
  }: {
    readonly start: number;
    readonly length: number;
  },
): ConsistencyRendering {
  /**
   The words, as many letters as the length says.
   */
  const words = 'c'.repeat(length,);
  return {
    kind: 'located',
    quote: words,
    core: words,
    start,
  };
}

/**
 Which of some renderings come back unlocated.

 @param places - slice of each place, none named for a place whose slice the caller does not know

 @param renderings - each place's rendering

 @returns For each place whether it comes back unlocated for overlapping another

 @example
 ```ts
 const unlocated = unlocatedAmong({ places: [{ sliceIndex: 0, }, { sliceIndex: 0, },], renderings, },); // [true, true] for two renderings that share a unit
 ```
 */
function unlocatedAmong(
  {
    places,
    renderings,
  }: {
    readonly places: readonly { readonly sliceIndex?: number; }[];
    readonly renderings: readonly ConsistencyRendering[];
  },
): readonly boolean[] {
  return unlessOverlapping({
    places,
    renderings,
  },)
    .map(function isOverlapped(rendering,): boolean {
      return (rendering.kind === 'unlocated') && (rendering.reason === 'overlaps-another');
    },);
}

/**
 The places of one slice, as many as the caller says.

 @param slice - slice every place stands in

 @param count - how many places

 @returns Places

 @example
 ```ts
 const places = inSlice({ slice: 0, count: 2, },); // [{ sliceIndex: 0 }, { sliceIndex: 0 }]
 ```
 */
function inSlice(
  {
    slice,
    count,
  }: {
    readonly slice: number;
    readonly count: number;
  },
): readonly { readonly sliceIndex: number; }[] {
  return Array.from(
    { length: count, },
    function placeOf(): { readonly sliceIndex: number; } {
      return { sliceIndex: slice, };
    },
  );
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: unlessOverlapping.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LEAVES UNLOCATED EXACTLY THE PLACES WHOSE WORDS OVERLAP ANOTHER PLACE OF THEIR SLICE, over every three ranges of '
            + 'a line of eight units, in any order, whatever the ranges share',
          fn: async () => {
            /**
             Every range of the line, as its first and its exclusive last unit.
             */
            const ranges = Array.from(
              { length: 8, },
              function from(_unused, first,): readonly (readonly [number, number])[] {
                return Array.from(
                  { length: 8 - first, },
                  function to(_other, length,): readonly [number, number] {
                    return [first, first + length + 1,];
                  },
                );
              },
            )
              .flat();
            /**
             Whether each triple of ranges comes back as comparing every pair of its places says.
             */
            const agreements = ranges.flatMap(function triplesFrom(first,): readonly boolean[] {
              return ranges.flatMap(function triplesWith(second,): readonly boolean[] {
                return ranges.map(function triple(third,): boolean {
                  /**
                   The three ranges, one for each place.
                   */
                  const chosen = [first, second, third,];
                  /**
                   What comparing every pair says for each place: another place whose range shares a unit with it.
                   */
                  const expected = chosen.map(function sharesAUnit(
                    [from, to,],
                    position,
                  ): boolean {
                    return chosen.some(function overlapsIt(
                      [otherFrom, otherTo,],
                      other,
                    ): boolean {
                      return (other !== position) && (from < otherTo) && (otherFrom < to);
                    },);
                  },);
                  /**
                   What the function says.
                   */
                  const found = unlocatedAmong({
                    places: inSlice({ slice: 0, count: 3, },),
                    renderings: chosen.map(function renderingOf([from, to,],): ConsistencyRendering {
                      return locatedAt({
                        start: from,
                        length: to - from,
                      },);
                    },),
                  },);
                  return found.every(function sameAt(unlocated, position,): boolean {
                    return unlocated === expected[position];
                  },);
                },);
              },);
            },);
            expect([ranges.length, agreements.every(Boolean,),],).toEqual([36, true,],);
          },
        },),
        it({
          name: 'LOCATES WORDS THAT TOUCH, since two ranges that share no unit overlap nothing, and one range inside another '
            + 'overlaps it',
          fn: async () => {
            expect([
              unlocatedAmong({
                places: inSlice({ slice: 0, count: 2, },),
                renderings: [locatedAt({ start: 0, length: 3, },), locatedAt({ start: 3, length: 3, },),],
              },),
              unlocatedAmong({
                places: inSlice({ slice: 0, count: 2, },),
                renderings: [locatedAt({ start: 0, length: 9, },), locatedAt({ start: 3, length: 3, },),],
              },),
              unlocatedAmong({
                places: inSlice({ slice: 0, count: 3, },),
                renderings: [locatedAt({ start: 8, length: 2, },), locatedAt({ start: 0, length: 8, },), locatedAt({ start: 3, length: 3, },),],
              },),
            ],).toEqual([[false, false,], [true, true,], [false, true, true,],],);
          },
        },),
        it({
          name: 'LEAVES ALONE PLACES OF DIFFERENT SLICES, OF NO NAMED SLICE AND THE ONES THAT ARE NOT LOCATED, whatever offsets '
            + 'they hold',
          fn: async () => {
            /**
             A rendering that is not located.
             */
            const nowhere: ConsistencyRendering = { kind: 'unlocated', reason: 'ambiguous', };
            expect([
              unlocatedAmong({
                places: [{ sliceIndex: 0, }, { sliceIndex: 1, },],
                renderings: [locatedAt({ start: 0, length: 5, },), locatedAt({ start: 0, length: 5, },),],
              },),
              unlocatedAmong({
                places: [{}, {},],
                renderings: [locatedAt({ start: 0, length: 5, },), locatedAt({ start: 0, length: 5, },),],
              },),
              unlocatedAmong({
                places: inSlice({ slice: 0, count: 3, },),
                renderings: [locatedAt({ start: 0, length: 5, },), nowhere, locatedAt({ start: 0, length: 5, },),],
              },),
            ],).toEqual([[false, false,], [false, false,], [true, false, true,],],);
          },
        },),
        it({
          name: 'KEEPS EACH PLACE IN ITS POSITION, an unlocated one in place of the one that overlaps, and returns the located '
            + 'rendering it was given where it overlaps nothing',
          fn: async () => {
            /**
             The renderings given.
             */
            const renderings = [locatedAt({ start: 0, length: 4, },), locatedAt({ start: 2, length: 4, },), locatedAt({ start: 9, length: 2, },),];
            /**
             What comes back.
             */
            const separated = unlessOverlapping({
              places: [{ sliceIndex: 4, }, { sliceIndex: 4, }, { sliceIndex: 4, },],
              renderings,
            },);
            expect(separated,).toEqual([
              { kind: 'unlocated', reason: 'overlaps-another', },
              { kind: 'unlocated', reason: 'overlaps-another', },
              renderings[2],
            ],);
          },
        },),
        it({
          name: 'READS EACH PLACE A FEW TIMES, not every other place of its slice, so a slice of thousands of places is '
            + 'separated by a sort and one pass',
          fn: async () => {
            /**
             How many places the one slice holds.
             */
            const size = 5_000;
            /**
             The reads of both lists, counted together.
             */
            const reads: ElementReads = { count: 0, };
            /**
             A slice of places side by side, none touching the next, where comparing every pair would read millions of times.
             */
            const separated = unlessOverlapping({
              places: countedList({
                items: Array.from(
                  { length: size, },
                  function placeOf(): { readonly sliceIndex: number; } {
                    return { sliceIndex: 0, };
                  },
                ),
                reads,
              },),
              renderings: countedList({
                items: Array.from(
                  { length: size, },
                  function renderingOf(_unused, position,): ConsistencyRendering {
                    return locatedAt({
                      start: position * 3,
                      length: 2,
                    },);
                  },
                ),
                reads,
              },),
            },);
            expect([
              separated.every(function isLocated({ kind, },): boolean {
                return kind === 'located';
              },),
              reads.count < (20 * size),
            ],).toEqual([true, true,],);
          },
        },),
      ],
    },),
  ],
},);
