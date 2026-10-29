/**
 Tests for reading the groups of a page once, by slice: the replacements addressed to each slice, the located places in
 each slice and the places a group replaces, so that the plan finds what one slice holds without a scan of every group.

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
  replacedKeyOf,
  type SettledGroup,
  settledIndexOf,
} from '../../dist/final/node/index.mjs';
import { groupIn, } from './consistency-apply-group.test-fixture.ts';

/**
 The English of the second slice.
 */
const SECOND = 'Later the kitten naps in the sun.';

/**
 The English of the fourth slice.
 */
const FOURTH = 'Dusk: the kitten purrs in the sun.';

/**
 A group of two places, the first in the second slice and the second in the fourth.

 @returns Group, whose second place is provisionally to be replaced

 @example
 ```ts
 const group = twoPlaceGroup();
 ```
 */
function twoPlaceGroup(): SettledGroup {
  /**
   The slices the places stand in and the English of each.
   */
  const standing = [
    { slice: 1, english: SECOND, },
    { slice: 3, english: FOURTH, },
  ];
  /**
   The group, with every place quoting the same words.
   */
  const group = groupIn({
    places: standing.map(function placeOf({ slice, english, },) {
      return { slice, english, quote: 'in the sun', };
    },),
  },);
  return { ...group, outcomes: ['chosen', 'replaced',], };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: settledIndexOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LISTS THE REPLACEMENTS AND THE LOCATED PLACES OF EACH SLICE, by the slice, in the order of the groups and of '
            + 'their places, and the places a group replaces',
          fn: async () => {
            const splice = { kind: 'splice', placePositions: [1,], sliceIndex: 3, start: 5, end: 15, text: 'the window', } as const;
            /**
             The page's first group, replacing its second place, and a second group replacing nothing.
             */
            const settled: readonly SettledGroup[] = [
              { ...twoPlaceGroup(), edits: [splice,], },
              { ...twoPlaceGroup(), edits: [], },
            ];
            const index = settledIndexOf({ settled, },);
            expect({
              edits: [...index.edits,].map(function seenEdits([slice, entries,],): unknown {
                return [slice, entries.map(function groupOf({ group, },): number {
                  return group;
                },),];
              },),
              places: [...index.places,].map(function seenPlaces([slice, entries,],): unknown {
                return [slice, entries.map(function placeOf({ group, position, },): string {
                  return `${String(group,)}:${String(position,)}`;
                },),];
              },),
              replaced: [...index.replaced,],
            },).toEqual({
              edits: [[3, [0,],],],
              places: [[1, ['0:0', '1:0',],], [3, ['0:1', '1:1',],],],
              replaced: ['0:1:3',],
            },);
          },
        },),
        it({
          name: 'LEAVES OUT A PLACE THE COUNTER COULD NOT LOCATE, which no replacement can change the words of',
          fn: async () => {
            /**
             A group whose second place is unlocated.
             */
            const settled: readonly SettledGroup[] = [
              {
                ...twoPlaceGroup(),
                renderings: [
                  ...twoPlaceGroup().renderings.slice(0, 1,),
                  { kind: 'unlocated', reason: 'not-found', },
                ],
              },
            ];
            expect([...settledIndexOf({ settled, },).places.keys(),],).toEqual([1,],);
          },
        },),
        it({
          name: 'READS EACH GROUP ONCE AND NEVER AGAIN, however many slices ask: the index is built in one pass over the groups, '
            + 'so a page of many groups and many slices pays for no scan of the groups on each slice',
          fn: async () => {
            /**
             How many times the replacements, the places and the renderings of the groups were read, mutated by the groups.
             */
            const reads = { edits: 0, places: 0, renderings: 0, };
            /**
             Groups that count every read of what the index is built from.
             */
            const settled: readonly SettledGroup[] = Array.from(
              { length: 12, },
              function countedGroup(): SettledGroup {
                const group = { ...twoPlaceGroup(), edits: [{ kind: 'splice', placePositions: [1,], sliceIndex: 3, start: 5, end: 15, text: 'the window', },], } as const;
                return {
                  ...group,
                  get edits() {
                    reads.edits += 1;
                    return group.edits;
                  },
                  get places() {
                    reads.places += 1;
                    return group.places;
                  },
                  get renderings() {
                    reads.renderings += 1;
                    return group.renderings;
                  },
                };
              },
            );
            const index = settledIndexOf({ settled, },);
            /**
             What the reads came to once the index was built.
             */
            const built = { ...reads, };
            // Every question a consumer asks of the index, for every slice the page could name.
            for (let sliceIndex = 0; sliceIndex < 40; sliceIndex += 1) {
              index.edits.get(sliceIndex,);
              index.places.get(sliceIndex,);
              index.replaced.has(replacedKeyOf({ group: 0, position: 1, sliceIndex, },),);
            }
            expect({ built, afterQuestions: reads, },).toEqual({
              built: { edits: 12, places: 12, renderings: 12, },
              afterQuestions: { edits: 12, places: 12, renderings: 12, },
            },);
          },
        },),
        it({
          name: 'HOLDS NOTHING FOR A PAGE OF NO GROUP, and keeps a replacement of a block that carries several places under '
            + 'each of them',
          fn: async () => {
            /**
             A replacement that carries both places of the group, both in the fourth slice.
             */
            const both = { kind: 'block', placePositions: [0, 1,], sliceIndex: 3, start: 0, end: 5, text: 'Dusk.', } as const;
            const index = settledIndexOf({ settled: [{ ...twoPlaceGroup(), edits: [both,], },], },);
            expect([
              [...settledIndexOf({ settled: [], },).edits,],
              [...index.replaced,],
            ],).toEqual([[], ['0:0:3', '0:1:3',],],);
          },
        },),
      ],
    },),
    describe({
      name: replacedKeyOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES A PLACE A GROUP REPLACES BY ITS GROUP, ITS POSITION AND ITS SLICE, in that order',
          fn: async () => {
            expect(replacedKeyOf({ group: 2, position: 1, sliceIndex: 7, },),).toBe('2:1:7',);
          },
        },),
      ],
    },),
  ],
},);
