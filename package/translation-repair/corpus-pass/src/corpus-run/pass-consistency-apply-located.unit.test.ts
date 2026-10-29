/**
 Tests for finding the replacements that change the words another group located:
 a block that still holds the words is not one, a block that drops or cuts them
 is, a code splice is one against every place of another group except a place
 that group itself replaces in the slice (which the earlier group wins by order),
 a group's own places never count, and the result is the same whichever order the
 groups come in.

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
  editsChangingLocatedWords,
  type PendingEdit,
  type SettledGroup,
  settledIndexOf,
} from '../../dist/final/node/index.mjs';
import {
  groupIn,
  withEdit,
} from './consistency-apply-group.test-fixture.ts';

/**
 The English of the second slice.
 */
const SECOND = 'Later the kitten naps in the sun.';

/**
 The English of the fourth slice.
 */
const FOURTH = 'Dusk: the kitten purrs in the sun.';

/**
 A replacement of the second slice's whole English.

 @param kind - what the replacement is

 @param start - where the replaced text starts

 @param text - text that goes in

 @returns Replacement, naming the second place of the group

 @example
 ```ts
 const edit = secondSliceEdit({ kind: 'block', start: 0, text: 'Later the kitten sleeps.', },);
 ```
 */
function secondSliceEdit(
  { kind, start, text, }: { readonly kind: PendingEdit['kind']; readonly start: number; readonly text: string; },
): PendingEdit {
  return { kind, placePositions: [1,], sliceIndex: 1, start, end: SECOND.length, text, };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: editsChangingLocatedWords.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES A BLOCK REPLACEMENT THAT DROPS OR CUTS THE WORDS ANOTHER GROUP LOCATED, with the group and the place, '
            + 'and none that still holds them; names a splice against every place its group does not replace in the slice',
          fn: async () => {
            const mine = groupIn({
              places: [
                { slice: 0, english: 'The cat naps. It is warm.', quote: 'The cat naps.', },
                { slice: 1, english: SECOND, quote: 'the kitten naps', },
              ],
            },);
            const other = groupIn({
              places: [
                { slice: 1, english: SECOND, quote: 'in the sun', },
                { slice: 3, english: FOURTH, quote: 'in the sun', },
              ],
            },);
            const receiving = groupIn({
              places: [
                { slice: 3, english: 'Dusk: the kitten purrs by the fire.', quote: 'by the fire', },
                { slice: 1, english: SECOND, quote: 'in the sun', },
              ],
            },);
            /**
             The receiving group with a real pending splice over its place in the second slice.
             */
            const replacing = withEdit({
              group: receiving,
              edit: { kind: 'splice', placePositions: [1,], sliceIndex: 1, start: 22, end: 32, text: 'by the fire', },
            },);
            /**
             What the replacement of the second slice leaves out, for a replacement of mine beside another group.
             */
            function leftOutBy(
              { edit, beside = other, }: { readonly edit: PendingEdit; readonly beside?: SettledGroup; },
            ): unknown {
              return editsChangingLocatedWords({
                index: settledIndexOf({ settled: [withEdit({ group: mine, edit, },), beside,], },),
                sliceIndex: 1,
                english: SECOND,
              },);
            }
            const dropping = secondSliceEdit({ kind: 'block', start: 0, text: 'Later the kitten sleeps.', },);
            const cutting = secondSliceEdit({ kind: 'block', start: 10, text: 'kitten sleeps', },);
            expect({
              dropping: leftOutBy({ edit: dropping, },),
              cutting: leftOutBy({ edit: { ...cutting, end: 25, }, },),
              holding: leftOutBy({
                edit: secondSliceEdit({ kind: 'block', start: 0, text: 'Later the kitten sleeps in the sun.', },),
              },),
              spliceAgainstKept: leftOutBy({ edit: { ...dropping, kind: 'splice', }, },),
              spliceAgainstReplacing: leftOutBy({ edit: { ...dropping, kind: 'splice', }, beside: replacing, },),
              spliceAgainstNotReplacing: leftOutBy({ edit: { ...dropping, kind: 'splice', }, beside: receiving, },),
              spliceAgainstOtherPlaceReplaced: leftOutBy({
                edit: { ...dropping, kind: 'splice', },
                beside: withEdit({
                  group: receiving,
                  edit: { kind: 'splice', placePositions: [0,], sliceIndex: 3, start: 23, end: 34, text: 'by the lamp', },
                },),
              },),
              spliceAgainstPlaceReplacedInAnotherSlice: leftOutBy({
                edit: { ...dropping, kind: 'splice', },
                beside: withEdit({
                  group: receiving,
                  edit: { kind: 'splice', placePositions: [1,], sliceIndex: 3, start: 23, end: 34, text: 'by the lamp', },
                },),
              },),
              blockAgainstReplacing: leftOutBy({ edit: dropping, beside: replacing, },),
            },).toEqual({
              dropping: [{ group: 0, edit: dropping, other: { group: 1, label: 'OCCURRENCE 1', start: 22, }, },],
              cutting: [
                {
                  group: 0,
                  edit: { ...cutting, end: 25, },
                  other: { group: 1, label: 'OCCURRENCE 1', start: 22, },
                },
              ],
              holding: [],
              spliceAgainstKept: [
                {
                  group: 0,
                  edit: { ...dropping, kind: 'splice', },
                  other: { group: 1, label: 'OCCURRENCE 1', start: 22, },
                },
              ],
              spliceAgainstReplacing: [],
              spliceAgainstNotReplacing: [
                {
                  group: 0,
                  edit: { ...dropping, kind: 'splice', },
                  other: { group: 1, label: 'OCCURRENCE 2', start: 22, },
                },
              ],
              spliceAgainstOtherPlaceReplaced: [
                {
                  group: 0,
                  edit: { ...dropping, kind: 'splice', },
                  other: { group: 1, label: 'OCCURRENCE 2', start: 22, },
                },
              ],
              spliceAgainstPlaceReplacedInAnotherSlice: [
                {
                  group: 0,
                  edit: { ...dropping, kind: 'splice', },
                  other: { group: 1, label: 'OCCURRENCE 2', start: 22, },
                },
              ],
              blockAgainstReplacing: [{ group: 0, edit: dropping, other: { group: 1, label: 'OCCURRENCE 2', start: 22, }, },],
            },);
          },
        },),
        it({
          name: 'HOLDS A SPLICE TO THE PLACE OF A GROUP THAT REPLACES ANOTHER PLACE OF ITS OWN IN THE SAME SLICE, sparing only '
            + 'the place the group\'s replacement carries',
          fn: async () => {
            const mine = groupIn({ places: [{ slice: 1, english: SECOND, quote: 'Later', },], },);
            /**
             A group with two places in the second slice, of which its pending splice carries only the first.
             */
            const beside = withEdit({
              group: groupIn({
                places: [
                  { slice: 1, english: SECOND, quote: 'the kitten naps', },
                  { slice: 1, english: SECOND, quote: 'in the sun', },
                ],
              },),
              edit: { kind: 'splice', placePositions: [0,], sliceIndex: 1, start: 6, end: 21, text: 'the cat naps', },
            },);
            const splice = secondSliceEdit({ kind: 'splice', start: 0, text: 'Evening.', },);
            expect(editsChangingLocatedWords({
              index: settledIndexOf({ settled: [withEdit({ group: mine, edit: splice, },), beside,], },),
              sliceIndex: 1,
              english: SECOND,
            },),).toEqual([{ group: 0, edit: splice, other: { group: 1, label: 'OCCURRENCE 2', start: 22, }, },],);
          },
        },),
        it({
          name: 'NEVER COUNTS A GROUP\'S OWN PLACES OR A PLACE IN ANOTHER SLICE, and gives the same replacement whichever '
            + 'order the groups come in',
          fn: async () => {
            const mine = groupIn({ places: [{ slice: 1, english: SECOND, quote: 'in the sun', },], },);
            const other = groupIn({ places: [{ slice: 3, english: FOURTH, quote: 'in the sun', },], },);
            const edit = secondSliceEdit({ kind: 'block', start: 0, text: 'Later the kitten sleeps.', },);
            const pending = withEdit({ group: mine, edit, },);
            const sibling = groupIn({ places: [{ slice: 1, english: SECOND, quote: 'the kitten naps', },], },);
            expect({
              ownAndElsewhere: editsChangingLocatedWords({
                index: settledIndexOf({ settled: [pending, other,], },),
                sliceIndex: 1,
                english: SECOND,
              },),
              forward: editsChangingLocatedWords({
                index: settledIndexOf({ settled: [pending, sibling,], },),
                sliceIndex: 1,
                english: SECOND,
              },),
              backward: editsChangingLocatedWords({
                index: settledIndexOf({ settled: [sibling, pending,], },),
                sliceIndex: 1,
                english: SECOND,
              },),
            },).toEqual({
              ownAndElsewhere: [],
              forward: [{ group: 0, edit, other: { group: 1, label: 'OCCURRENCE 1', start: 6, }, },],
              backward: [{ group: 1, edit, other: { group: 0, label: 'OCCURRENCE 1', start: 6, }, },],
            },);
          },
        },),
        it({
          name: 'NAMES THE FIRST OF SEVERAL PLACES IT CHANGES in page order, once for the replacement',
          fn: async () => {
            const mine = groupIn({ places: [{ slice: 1, english: SECOND, quote: 'Later the kitten', },], },);
            const first = groupIn({ places: [{ slice: 1, english: SECOND, quote: 'in the sun', },], },);
            const second = groupIn({ places: [{ slice: 1, english: SECOND, quote: 'the kitten naps', },], },);
            const edit = secondSliceEdit({ kind: 'block', start: 0, text: 'Evening.', },);
            expect(editsChangingLocatedWords({
              index: settledIndexOf({ settled: [withEdit({ group: mine, edit, },), first, second,], },),
              sliceIndex: 1,
              english: SECOND,
            },),).toEqual([{ group: 0, edit, other: { group: 2, label: 'OCCURRENCE 1', start: 6, }, },],);
          },
        },),
        it({
          name: 'READS ONLY THE INDEX ENTRIES OF THE SLICE ASKED ABOUT, so a page of many groups does not cost a scan of every '
            + 'group for each replacement: entries of other slices that cannot be read are never read',
          fn: async () => {
            const mine = groupIn({ places: [{ slice: 1, english: SECOND, quote: 'Later the kitten', },], },);
            const other = groupIn({ places: [{ slice: 1, english: SECOND, quote: 'in the sun', },], },);
            const edit = secondSliceEdit({ kind: 'block', start: 0, text: 'Evening.', },);
            const real = settledIndexOf({ settled: [withEdit({ group: mine, edit, },), other,], },);
            /**
             A list that raises when it is read.
             */
            const unreadable = new Proxy(
              [],
              {
                get(): never {
                  throw new Error('the entry of another slice was read',);
                },
              },
            );
            /**
             The index with an unreadable entry added for a slice nothing asks about, in both lists.
             */
            const poisoned = {
              ...real,
              edits: new Map([...real.edits, [9, unreadable,],],),
              places: new Map([...real.places, [9, unreadable,],],),
            };
            expect(editsChangingLocatedWords({ index: poisoned, sliceIndex: 1, english: SECOND, },),).toEqual([
              { group: 0, edit, other: { group: 1, label: 'OCCURRENCE 1', start: 22, }, },
            ],);
          },
        },),
      ],
    },),
  ],
},);
