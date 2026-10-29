/**
 Tests for what the changed slices of a recorded consistency section say beside
 the places that were replaced: one row per slice, a row for each slice that
 holds a replaced place and for none other, one fate for the places of a slice,
 and what an overlapped place names as the place it gave way to (generation
 seventeen) or the weaker check an older generation was written under.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  type ArtifactConsistencyGroup,
  assertSlicesCoherent,
} from '../../dist/final/node/index.mjs';

/**
 The place an overlapped place gave way to, as the record names it.
 */
type GaveWayTo = { readonly group: number; readonly label: string; };

/**
 A group whose places stand in the slices and end in the outcomes a case names.

 @param placed - slice and outcome of each place, in page order, and for an overlapped place the place it gave way to
 when the generation records one

 @param located - whether each place's rendering is located, unlocated when absent

 @returns Group of a repeated sentence

 @example
 ```ts
 const group = groupOf({ placed: [[0, 'chosen',], [1, 'replaced',],], },);
 ```
 */
function groupOf(
  {
    placed,
    located = false,
  }: {
    readonly placed: readonly (readonly [number, string] | readonly [number, string, GaveWayTo])[];
    readonly located?: boolean;
  },
): ArtifactConsistencyGroup {
  return {
    repeat: { kind: 'maximal', wording: '窗台上的阳光让小猫睡着了', },
    occurrences: placed.map(function placeOf(
      [sliceIndex, outcome, gaveWayTo,],
      at,
    ) {
      return {
        label: `OCCURRENCE ${String(at + 1,)}`,
        sliceIndex,
        startOffset: 6 + (at * 30),
        endOffset: 18 + (at * 30),
        line: 3,
        rendering: located
          ? { kind: 'located', quote: 'The kitten naps', start: 6 + (at * 30), }
          : { kind: 'unlocated', reason: 'not-found', },
        outcome,
        ...((gaveWayTo === undefined) ? {} : { gaveWayTo, }),
      };
    },),
    unplaced: [],
    verdict: { kind: 'not-asked', reason: 'too-few-placed', },
  } as unknown as ArtifactConsistencyGroup;
}

/**
 What reading the slices refuses with, or a note that nothing was refused.

 @param groups - recorded groups

 @param slices - recorded changed slices

 @param generation - generation the section was written under

 @returns Refusal's message

 @example
 ```ts
 const message = refusalOf({ groups: [], slices: [], generation: 17, },);
 ```
 */
function refusalOf(
  {
    groups,
    slices,
    generation,
  }: {
    readonly groups: readonly ArtifactConsistencyGroup[];
    readonly slices: readonly { readonly sliceIndex: number; readonly text: string; }[];
    readonly generation: 16 | 17;
  },
): string {
  try {
    assertSlicesCoherent({
      groups,
      slices,
      path: 'cat.consistency.slices',
      generation,
    },);
  }
  catch (error) {
    if (error instanceof ArtifactParseError)
      return error.message;
    throw error;
  }
  return 'not refused';
}

/**
 The words a refusal of an overlapped place begins with.

 @param place - group and label of the overlapped place, counted from one for the group
 and the slice it stands in

 @returns Refusal's message up to what was expected

 @example
 ```ts
 const message = `${refusalStart({ place: 'group 2 OCCURRENCE 1 (overlapped, slice 2)', },)} to name the place it gave way to.`;
 ```
 */
function refusalStart({ place, }: { readonly place: string; },): string {
  return `artifact parse failed at cat.consistency.slices: expected ${place}`;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: assertSlicesCoherent.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS ONE ROW FOR EACH SLICE THAT HOLDS A REPLACED PLACE, and no row for a slice that holds none',
          fn: async () => {
            expect(refusalOf({
              groups: [groupOf({ placed: [[0, 'chosen',], [2, 'replaced',], [3, 'unlocated',],], },),],
              slices: [{ sliceIndex: 2, text: 'The kitten naps.', },],
              generation: 17,
            },),).toBe('not refused',);
          },
        },),
        it({
          name: 'REFUSES A ROW FOR A SLICE WITH NO REPLACED PLACE, A REPLACED PLACE WITH NO ROW, AND A SLICE NAMED TWICE',
          fn: async () => {
            const groups = [groupOf({ placed: [[0, 'chosen',], [2, 'replaced',],], },),];
            expect([
              refusalOf({ groups, slices: [{ sliceIndex: 2, text: 'a', }, { sliceIndex: 0, text: 'b', },], generation: 17, },),
              refusalOf({ groups, slices: [], generation: 17, },),
              refusalOf({ groups, slices: [{ sliceIndex: 2, text: 'a', }, { sliceIndex: 2, text: 'b', },], generation: 17, },),
            ],).toEqual([
              'artifact parse failed at cat.consistency.slices: expected a replaced place in slice 0, which a changed slice needs.',
              'artifact parse failed at cat.consistency.slices: expected a changed slice row for slice 2, which holds a replaced place.',
              'artifact parse failed at cat.consistency.slices: expected one row per slice; slice 2 appears more than once.',
            ],);
          },
        },),
        it({
          name: 'REFUSES A SLICE THAT HOLDS BOTH A REPLACED PLACE AND A PLACE THE FLOORS REFUSED, since they answer for it once',
          fn: async () => {
            expect(refusalOf({
              groups: [groupOf({ placed: [[0, 'chosen',], [2, 'replaced',],], },), groupOf({ placed: [[2, 'kept-floor',], [3, 'chosen',],], },),],
              slices: [{ sliceIndex: 2, text: 'a', },],
              generation: 17,
            },),).toBe(
              'artifact parse failed at cat.consistency.slices: expected one fate for slice 2, which holds a replaced place and a place the floors refused.',
            );
          },
        },),
        it({
          name: 'ACCEPTS AN OVERLAP BESIDE A REPLACED PLACE, A REFUSED ONE OR A LOCATED PLACE OF ANOTHER GROUP IN A SECTION OF '
            + 'GENERATION SIXTEEN, which names no place it gave way to, and refuses one with none of them in its slice',
          fn: async () => {
            expect([
              refusalOf({
                groups: [groupOf({ placed: [[0, 'chosen',], [2, 'replaced',],], },), groupOf({ placed: [[2, 'overlapped',], [3, 'chosen',],], },),],
                slices: [{ sliceIndex: 2, text: 'a', },],
                generation: 16,
              },),
              refusalOf({
                groups: [groupOf({ placed: [[0, 'chosen',], [2, 'kept-floor',],], },), groupOf({ placed: [[2, 'overlapped',], [3, 'chosen',],], },),],
                slices: [],
                generation: 16,
              },),
              refusalOf({
                groups: [
                  groupOf({ placed: [[0, 'chosen',], [2, 'overlapped',],], },),
                  groupOf({ placed: [[2, 'chosen',], [3, 'replaced',],], located: true, },),
                ],
                slices: [{ sliceIndex: 3, text: 'a', },],
                generation: 16,
              },),
              refusalOf({
                groups: [
                  groupOf({ placed: [[0, 'chosen',], [2, 'overlapped',],], },),
                  groupOf({ placed: [[2, 'chosen',], [3, 'replaced',],], },),
                ],
                slices: [{ sliceIndex: 3, text: 'a', },],
                generation: 16,
              },),
              refusalOf({
                groups: [groupOf({ placed: [[0, 'chosen',], [2, 'overlapped',],], located: true, },),],
                slices: [],
                generation: 16,
              },),
            ],).toEqual([
              'not refused',
              'not refused',
              'not refused',
              'artifact parse failed at cat.consistency.slices: expected a replacement, or a located place of another group, '
                + 'in slice 2 for the overlapped place to overlap.',
              'artifact parse failed at cat.consistency.slices: expected a replacement, or a located place of another group, '
                + 'in slice 2 for the overlapped place to overlap.',
            ],);
          },
        },),
        it({
          name: 'STILL ACCEPTS, IN A SECTION OF GENERATION SIXTEEN, AN OVERLAPPED PLACE BESIDE ANY LOCATED PLACE OF ANOTHER GROUP '
            + 'in its slice, the check that generation was written under',
          fn: async () => {
            expect([
              refusalOf({
                groups: [
                  groupOf({ placed: [[2, 'overlapped',],], },),
                  groupOf({ placed: [[2, 'untouched',],], located: true, },),
                ],
                slices: [],
                generation: 16,
              },),
              refusalOf({
                groups: [
                  groupOf({ placed: [[2, 'overlapped',],], located: true, },),
                  groupOf({ placed: [[2, 'chosen',],], located: true, },),
                ],
                slices: [],
                generation: 16,
              },),
              refusalOf({
                groups: [
                  groupOf({ placed: [[2, 'overlapped',],], located: true, },),
                  groupOf({ placed: [[2, 'overlapped',],], located: true, },),
                ],
                slices: [],
                generation: 16,
              },),
            ],).toEqual(['not refused', 'not refused', 'not refused',],);
          },
        },),
        it({
          name: 'REFUSES, IN A SECTION OF GENERATION SEVENTEEN, AN OVERLAPPED PLACE THAT NAMES NO PLACE IT GAVE WAY TO, whatever '
            + 'another group located in its slice',
          fn: async () => {
            expect([
              refusalOf({
                groups: [
                  groupOf({ placed: [[2, 'overlapped',],], },),
                  groupOf({ placed: [[2, 'untouched',],], located: true, },),
                ],
                slices: [],
                generation: 17,
              },),
              refusalOf({
                groups: [
                  groupOf({ placed: [[2, 'overlapped',],], located: true, },),
                  groupOf({ placed: [[2, 'chosen',],], located: true, },),
                ],
                slices: [],
                generation: 17,
              },),
              refusalOf({
                groups: [
                  groupOf({ placed: [[2, 'overlapped',],], located: true, },),
                  groupOf({ placed: [[2, 'overlapped',],], located: true, },),
                ],
                slices: [],
                generation: 17,
              },),
              refusalOf({
                groups: [groupOf({ placed: [[2, 'overlapped',],], },),],
                slices: [],
                generation: 17,
              },),
            ],).toEqual([
              `${refusalStart({ place: 'group 1 OCCURRENCE 1 (overlapped, slice 2)', },)} to name the place it gave way to.`,
              `${refusalStart({ place: 'group 1 OCCURRENCE 1 (overlapped, slice 2)', },)} to name the place it gave way to.`,
              `${refusalStart({ place: 'group 1 OCCURRENCE 1 (overlapped, slice 2)', },)} to name the place it gave way to.`,
              `${refusalStart({ place: 'group 1 OCCURRENCE 1 (overlapped, slice 2)', },)} to name the place it gave way to.`,
            ],);
          },
        },),
        it({
          name: 'ACCEPTS, IN A SECTION OF GENERATION SEVENTEEN, AN OVERLAPPED PLACE THAT NAMES A REPLACED PLACE, A REFUSED ONE OR '
            + 'A LOCATED PLACE OF ANOTHER GROUP in its slice, and a chain of overlapped places that ends in one of them',
          fn: async () => {
            expect([
              refusalOf({
                groups: [
                  groupOf({ placed: [[0, 'chosen',], [2, 'replaced',],], },),
                  groupOf({ placed: [[2, 'overlapped', { group: 0, label: 'OCCURRENCE 2', },], [3, 'chosen',],], },),
                ],
                slices: [{ sliceIndex: 2, text: 'a', },],
                generation: 17,
              },),
              refusalOf({
                groups: [
                  groupOf({ placed: [[0, 'chosen',], [2, 'kept-floor',],], },),
                  groupOf({ placed: [[2, 'overlapped', { group: 0, label: 'OCCURRENCE 2', },], [3, 'chosen',],], },),
                ],
                slices: [],
                generation: 17,
              },),
              refusalOf({
                groups: [
                  groupOf({ placed: [[0, 'chosen',], [2, 'overlapped', { group: 1, label: 'OCCURRENCE 1', },],], },),
                  groupOf({ placed: [[2, 'chosen',], [3, 'replaced',],], located: true, },),
                ],
                slices: [{ sliceIndex: 3, text: 'a', },],
                generation: 17,
              },),
              refusalOf({
                groups: [
                  groupOf({ placed: [[2, 'overlapped', { group: 1, label: 'OCCURRENCE 1', },],], located: true, },),
                  groupOf({ placed: [[2, 'overlapped', { group: 2, label: 'OCCURRENCE 1', },],], located: true, },),
                  groupOf({ placed: [[2, 'replaced',],], },),
                ],
                slices: [{ sliceIndex: 2, text: 'a', },],
                generation: 17,
              },),
            ],).toEqual(['not refused', 'not refused', 'not refused', 'not refused',],);
          },
        },),
        it({
          name: 'ACCEPTS, IN A SECTION OF GENERATION SEVENTEEN, TWO OVERLAPPED PLACES THAT NAME EACH OTHER WHEN BOTH ARE LOCATED, '
            + 'since two replacements that each change the other\'s located words are both left out, and refuses the pair '
            + 'when either has no located rendering',
          fn: async () => {
            expect([
              refusalOf({
                groups: [
                  groupOf({ placed: [[2, 'overlapped', { group: 1, label: 'OCCURRENCE 1', },],], located: true, },),
                  groupOf({ placed: [[2, 'overlapped', { group: 0, label: 'OCCURRENCE 1', },],], located: true, },),
                ],
                slices: [],
                generation: 17,
              },),
              refusalOf({
                groups: [
                  groupOf({ placed: [[2, 'overlapped', { group: 1, label: 'OCCURRENCE 1', },],], located: true, },),
                  groupOf({ placed: [[2, 'overlapped', { group: 0, label: 'OCCURRENCE 1', },],], },),
                ],
                slices: [],
                generation: 17,
              },),
            ],).toEqual([
              'not refused',
              `${refusalStart({ place: 'group 1 OCCURRENCE 1 (overlapped, slice 2)', },)} to name a replaced or refused place, `
                + 'or a place with a located rendering, and group 2 OCCURRENCE 1 is neither.',
            ],);
          },
        },),
        it({
          name: 'REFUSES, IN A SECTION OF GENERATION SEVENTEEN, AN OVERLAPPED PLACE WHOSE NAMED PLACE IS NOT A GROUP OF THE '
            + 'SECTION, NOT A PLACE OF THAT GROUP, THE PLACE\'S OWN GROUP OR IN ANOTHER SLICE, or is no located, replaced or '
            + 'refused place',
          fn: async () => {
            /**
             The groups of a section whose second group holds the place the first group's overlapped place names.
             */
            const named = [
              groupOf({ placed: [[2, 'overlapped', { group: 1, label: 'OCCURRENCE 1', },],], },),
              groupOf({ placed: [[2, 'chosen',],], located: true, },),
            ];
            /**
             The same groups with the overlapped place naming another place.
             */
            function namingOther({ gaveWayTo, }: { readonly gaveWayTo: GaveWayTo; },): readonly ArtifactConsistencyGroup[] {
              return [groupOf({ placed: [[2, 'overlapped', gaveWayTo,],], },), ...named.slice(1,),];
            }
            expect([
              refusalOf({ groups: namingOther({ gaveWayTo: { group: 5, label: 'OCCURRENCE 1', }, },), slices: [], generation: 17, },),
              refusalOf({ groups: namingOther({ gaveWayTo: { group: 1, label: 'OCCURRENCE 4', }, },), slices: [], generation: 17, },),
              refusalOf({ groups: namingOther({ gaveWayTo: { group: 0, label: 'OCCURRENCE 1', }, },), slices: [], generation: 17, },),
              refusalOf({
                groups: [
                  groupOf({ placed: [[1, 'chosen',], [2, 'overlapped', { group: 0, label: 'OCCURRENCE 1', },],], located: true, },),
                ],
                slices: [],
                generation: 17,
              },),
              refusalOf({
                groups: [
                  groupOf({ placed: [[2, 'overlapped', { group: 1, label: 'OCCURRENCE 1', },],], },),
                  groupOf({ placed: [[3, 'chosen',],], located: true, },),
                ],
                slices: [],
                generation: 17,
              },),
              refusalOf({
                groups: [
                  groupOf({ placed: [[2, 'overlapped', { group: 1, label: 'OCCURRENCE 1', },],], },),
                  groupOf({ placed: [[2, 'untouched',],], },),
                ],
                slices: [],
                generation: 17,
              },),
            ],).toEqual([
              `${refusalStart({ place: 'group 1 OCCURRENCE 1 (overlapped, slice 2)', },)} to name a group the section holds, `
                + 'and group 6 is not one.',
              `${refusalStart({ place: 'group 1 OCCURRENCE 1 (overlapped, slice 2)', },)} to name a place group 2 holds, `
                + 'and OCCURRENCE 4 is not one.',
              `${refusalStart({ place: 'group 1 OCCURRENCE 1 (overlapped, slice 2)', },)} to name a place of another group, `
                + 'and group 1 is its own.',
              `${refusalStart({ place: 'group 1 OCCURRENCE 2 (overlapped, slice 2)', },)} to name a place of another group, `
                + 'and group 1 is its own.',
              `${refusalStart({ place: 'group 1 OCCURRENCE 1 (overlapped, slice 2)', },)} to name a place in its own slice, `
                + 'and group 2 OCCURRENCE 1 stands in slice 3.',
              `${refusalStart({ place: 'group 1 OCCURRENCE 1 (overlapped, slice 2)', },)} to name a replaced or refused place, `
                + 'or a place with a located rendering, and group 2 OCCURRENCE 1 is neither.',
            ],);
          },
        },),
      ],
    },),
  ],
},);
