/**
 Tests for how applying the consistency replacements names what it records: the
 name of a replacement, the key of a place's outcome, a rendering as recorded, the
 pending replacement a name stands for, the place an overlapped place gave way to,
 the replacement that won an overlap, and the groups once every place has its
 final outcome.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { ConsistencyInvariantError, } from '@monochromatic-dev/translation-repair-consistency';
import type { ArtifactConsistencyGroup, } from '@monochromatic-dev/translation-repair-artifact';
import {
  carriedPlaceName,
  editId,
  finalGroupsOf,
  type GroupEdit,
  pendingNamed,
  placeKey,
  recordedRendering,
  type SettledGroup,
  settledIndexOf,
  sliceEditsOf,
  winnerOver,
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
 A group of two places, the first chosen and the second a receiving place.

 @returns Group, whose second place is provisionally to be replaced

 @example
 ```ts
 const group = twoPlaceGroup();
 ```
 */
function twoPlaceGroup(): SettledGroup {
  return {
    ...groupIn({
      places: [
        { slice: 1, english: SECOND, quote: 'in the sun', },
        { slice: 3, english: FOURTH, quote: 'in the sun', },
      ],
    },),
    outcomes: ['chosen', 'replaced',],
  };
}

/**
 What each place of the groups was recorded as, with the place it gave way to.

 @param groups - recorded groups

 @returns One list per group, naming each place's label, slice, outcome and the place it gave way to or none

 @example
 ```ts
 const recorded = recordedAs({ groups, },);
 ```
 */
function recordedAs({ groups, }: { readonly groups: readonly ArtifactConsistencyGroup[]; },): readonly (readonly unknown[])[] {
  return groups.map(function placesOf({ occurrences, },): readonly unknown[] {
    return occurrences.map(function recordedOf(occurrence,): unknown {
      return [occurrence.label, occurrence.sliceIndex, occurrence.outcome, occurrence.gaveWayTo ?? 'none',];
    },);
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: editId.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES A REPLACEMENT BY ITS GROUP COUNTED FROM ONE AND THE LABELS OF THE PLACES IT CARRIES, joined by and',
          fn: async () => {
            expect([
              editId({ group: 0, places: [1,], },),
              editId({ group: 2, places: [1, 2,], },),
            ],).toEqual(['group 1 OCCURRENCE 2', 'group 3 OCCURRENCE 2 and OCCURRENCE 3',],);
          },
        },),
      ],
    },),
    describe({
      name: placeKey.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEYS A PLACE BY THE POSITION OF ITS GROUP AND ITS OWN POSITION, from zero',
          fn: async () => {
            expect(placeKey({ group: 2, place: 10, },),).toBe('2:10',);
          },
        },),
      ],
    },),
    describe({
      name: recordedRendering.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RECORDS AN UNLOCATED RENDERING WITH ITS REASON, A LOCATED ONE WITH ITS QUOTE AND START, and the slot when it has one',
          fn: async () => {
            expect([
              recordedRendering({ rendering: { kind: 'unlocated', reason: 'not-found', }, },),
              recordedRendering({ rendering: { kind: 'located', quote: 'in the sun', core: 'in the sun', start: 22, }, },),
              recordedRendering({
                rendering: {
                  kind: 'located',
                  quote: 'So this cat is lazy',
                  core: 'So this cat is lazy',
                  start: 4,
                  slot: 'lazy',
                  slotStart: 15,
                },
              },),
            ],).toEqual([
              { kind: 'unlocated', reason: 'not-found', },
              { kind: 'located', quote: 'in the sun', start: 22, },
              { kind: 'located', quote: 'So this cat is lazy', start: 4, slot: 'lazy', },
            ],);
          },
        },),
        it({
          name: 'RECORDS THE SINGLE MARKS A LOCATED RENDERING HOLDS BESIDE ITS WORDS, each on its own side and only when present, '
            + 'with the slot',
          fn: async () => {
            expect([
              recordedRendering({
                rendering: { kind: 'located', quote: 'the cats', core: 'the cats', start: 5, markAfter: '\'', },
              },),
              recordedRendering({
                rendering: { kind: 'located', quote: 'the cats', core: 'the cats', start: 5, markBefore: '‘', },
              },),
              recordedRendering({
                rendering: {
                  kind: 'located',
                  quote: 'the cats nap',
                  core: 'the cats nap',
                  start: 5,
                  markBefore: '‘',
                  markAfter: '’',
                  slot: 'nap',
                  slotStart: 9,
                },
              },),
            ],).toEqual([
              { kind: 'located', quote: 'the cats', start: 5, markAfter: '\'', },
              { kind: 'located', quote: 'the cats', start: 5, markBefore: '‘', },
              { kind: 'located', quote: 'the cats nap', start: 5, markBefore: '‘', markAfter: '’', slot: 'nap', },
            ],);
          },
        },),
        it({
          name: 'RECORDS THE DEFINITIONS OF THE PAGE A LOCATED RENDERING WAS READ WITH, only when it has them, beside its words '
            + 'and no slot',
          fn: async () => {
            expect([
              recordedRendering({
                rendering: {
                  kind: 'located',
                  quote: 'a[b]c',
                  core: 'a[b]c',
                  start: 0,
                  definitions: '[b]: https://x.example/b',
                },
              },),
              recordedRendering({
                rendering: { kind: 'located', quote: 'a[b]c', core: 'a[b]c', start: 0, slot: 'b', slotStart: 2, },
              },),
            ],).toEqual([
              { kind: 'located', quote: 'a[b]c', start: 0, definitions: '[b]: https://x.example/b', },
              { kind: 'located', quote: 'a[b]c', start: 0, slot: 'b', },
            ],);
          },
        },),
      ],
    },),
    describe({
      name: sliceEditsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE REPLACEMENTS OF ONE SLICE, EARLIER GROUPS FIRST, with the keys of the places each carries and the group it belongs to, and leaves out those of other slices',
          fn: async () => {
            const block = {
              kind: 'block',
              placePositions: [0, 1,],
              sliceIndex: 2,
              start: 0,
              end: 20,
              text: 'Evening.',
            } as const;
            const splice = {
              kind: 'splice',
              placePositions: [1,],
              sliceIndex: 2,
              start: 26,
              end: 36,
              text: 'the window',
            } as const;
            const elsewhere = {
              ...splice,
              sliceIndex: 3,
            } as const;
            const { edits, carriers, pendings, } = sliceEditsOf({
              index: settledIndexOf({
                settled: [
                  {
                    ...twoPlaceGroup(),
                    edits: [block,],
                  },
                  {
                    ...twoPlaceGroup(),
                    edits: [elsewhere, splice,],
                  },
                ],
              },),
              sliceIndex: 2,
            },);
            expect({
              edits,
              carriers: [...carriers,],
              pendings: [...pendings,],
            },).toEqual({
              edits: [
                { id: 'group 1 OCCURRENCE 1 and OCCURRENCE 2', start: 0, end: 20, text: 'Evening.', },
                { id: 'group 2 OCCURRENCE 2', start: 26, end: 36, text: 'the window', },
              ],
              carriers: [
                ['group 1 OCCURRENCE 1 and OCCURRENCE 2', ['0:0', '0:1',],],
                ['group 2 OCCURRENCE 2', ['1:1',],],
              ],
              pendings: [
                ['group 1 OCCURRENCE 1 and OCCURRENCE 2', { group: 0, edit: block, },],
                ['group 2 OCCURRENCE 2', { group: 1, edit: splice, },],
              ],
            },);
          },
        },),
        it({
          name: 'REFUSES TWO REPLACEMENTS OF ONE GROUP WHOSE WORDS OVERLAP, naming the slice and both, and accepts replacements '
            + 'that only touch or that belong to different groups',
          fn: async () => {
            const first = { kind: 'splice', placePositions: [0,], sliceIndex: 2, start: 0, end: 20, text: 'Evening.', } as const;
            const overlapping = { kind: 'block', placePositions: [1,], sliceIndex: 2, start: 10, end: 40, text: 'Dawn.', } as const;
            const touching = { ...overlapping, start: 20, } as const;
            const refusal = caught(function overlapsInOneGroup(): unknown {
              return sliceEditsOf({
                index: settledIndexOf({ settled: [{ ...twoPlaceGroup(), edits: [first, overlapping,], },], },),
                sliceIndex: 2,
              },);
            },);
            expect(refusal,).toBeInstanceOf(ConsistencyInvariantError,);
            expect(String(refusal,),).toBe(
              'ConsistencyInvariantError: unreachable: the replacements group 1 OCCURRENCE 1 and group 1 OCCURRENCE 2 of slice 2 '
                + 'overlap, and no two replacements of one group address the same words',
            );
            expect([
              sliceEditsOf({
                index: settledIndexOf({ settled: [{ ...twoPlaceGroup(), edits: [first, touching,], },], },),
                sliceIndex: 2,
              },).edits.length,
              sliceEditsOf({
                index: settledIndexOf({
                  settled: [{ ...twoPlaceGroup(), edits: [first,], }, { ...twoPlaceGroup(), edits: [overlapping,], },],
                },),
                sliceIndex: 2,
              },).edits.length,
              sliceEditsOf({
                index: settledIndexOf({ settled: [{ ...twoPlaceGroup(), edits: [first, overlapping,], },], },),
                sliceIndex: 3,
              },).edits.length,
            ],).toEqual([2, 2, 0,],);
          },
        },),
        it({
          name: 'READS ONLY THE INDEX ENTRIES OF THE SLICE ASKED ABOUT, so a page of many slices does not cost a scan of every '
            + 'group for each slice: an entry for another slice that cannot be read is never read',
          fn: async () => {
            /**
             A replacement of the slice asked about.
             */
            const splice = { kind: 'splice', placePositions: [1,], sliceIndex: 2, start: 26, end: 36, text: 'the window', } as const;
            /**
             The index of the page, with an entry added for a slice nothing asks about, whose list raises when it is read.
             */
            const real = settledIndexOf({ settled: [{ ...twoPlaceGroup(), edits: [splice,], },], },);
            const poisoned = {
              ...real,
              edits: new Map([
                ...real.edits,
                [9, new Proxy(
                  [],
                  {
                    get(): never {
                      throw new Error('the entry of slice 9 was read',);
                    },
                  },
                ),],
              ],),
            };
            expect(sliceEditsOf({ index: poisoned, sliceIndex: 2, },).edits,).toEqual([
              { id: 'group 1 OCCURRENCE 2', start: 26, end: 36, text: 'the window', },
            ],);
          },
        },),
      ],
    },),
    describe({
      name: pendingNamed.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'GIVES BACK THE PENDING REPLACEMENT A NAME STANDS FOR, and refuses a name no pending replacement has',
          fn: async () => {
            const pending: GroupEdit = {
              group: 1,
              edit: { kind: 'splice', placePositions: [1,], sliceIndex: 2, start: 26, end: 109, text: 'the sunny window', },
            };
            const pendings = new Map([['group 2 OCCURRENCE 2', pending,],],);
            const refusal = caught(function namesNone(): unknown {
              return pendingNamed({ pendings, id: 'group 1 OCCURRENCE 2', },);
            },);
            expect(pendingNamed({ pendings, id: 'group 2 OCCURRENCE 2', },),).toBe(pending,);
            expect(refusal,).toBeInstanceOf(ConsistencyInvariantError,);
            expect(String(refusal,),).toBe(
              'ConsistencyInvariantError: unreachable: edit group 1 OCCURRENCE 2 was made and no pending replacement has its '
                + 'name, and every edit made is a pending replacement',
            );
          },
        },),
      ],
    },),
    describe({
      name: carriedPlaceName.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES A REPLACEMENT\'S PLACE BY ITS GROUP AND THE LABEL OF THE FIRST PLACE IT CARRIES, and refuses one that carries none',
          fn: async () => {
            const edit = { kind: 'block', placePositions: [2, 3,], sliceIndex: 2, start: 8, end: 40, text: 'Evening.', } as const;
            const refusal = caught(function carriesNone(): unknown {
              return carriedPlaceName({ group: 1, edit: { ...edit, placePositions: [], }, },);
            },);
            expect(carriedPlaceName({ group: 1, edit, },),).toEqual({ group: 1, label: 'OCCURRENCE 3', },);
            expect(refusal,).toBeInstanceOf(ConsistencyInvariantError,);
            expect(String(refusal,),).toBe(
              'ConsistencyInvariantError: unreachable: a replacement of group 2 carries no receiving place, and every '
                + 'replacement carries at least one',
            );
          },
        },),
      ],
    },),
    describe({
      name: winnerOver.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'GIVES THE FIRST REPLACEMENT MADE WHOSE SPAN MEETS THE ONE LEFT OUT, not one that only touches it, and refuses '
            + 'when none meets it',
          fn: async () => {
            const loser = { id: 'group 2 OCCURRENCE 3', start: 83, end: 155, text: 'So this cat must be a sweet little treasure', };
            const touching = { id: 'group 1 OCCURRENCE 1', start: 0, end: 83, text: 'In the evening, ', };
            const first = { id: 'group 1 OCCURRENCE 2', start: 26, end: 109, text: 'the sunny window', };
            const second = { id: 'group 3 OCCURRENCE 2', start: 90, end: 100, text: 'the lamp', };
            const refusal = caught(function meetsNone(): unknown {
              return winnerOver({ loser, made: [touching,], },);
            },);
            expect(winnerOver({ loser, made: [touching, first, second,], },),).toBe(first,);
            expect(refusal,).toBeInstanceOf(ConsistencyInvariantError,);
            expect(String(refusal,),).toBe(
              'ConsistencyInvariantError: unreachable: edit group 2 OCCURRENCE 3 was left out for overlapping an earlier edit, '
                + 'and no edit that was made overlaps it',
            );
          },
        },),
      ],
    },),
    describe({
      name: finalGroupsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RECORDS A DECIDED OUTCOME OVER THE PROVISIONAL ONE AND THE PLACE AN OVERLAPPED PLACE GAVE WAY TO, and no place for '
            + 'any other outcome, however a name was kept for it',
          fn: async () => {
            const group = twoPlaceGroup();
            const gaveWayTo = new Map([
              ['0:1', { group: 1, label: 'OCCURRENCE 1', },],
              ['1:0', { group: 0, label: 'OCCURRENCE 2', },],
            ],);
            expect({
              overlapped: recordedAs({
                groups: finalGroupsOf({
                  settled: [group,],
                  outcomes: new Map([['0:1', 'overlapped',],],),
                  gaveWayTo,
                },),
              },),
              replacedWithStaleName: recordedAs({
                groups: finalGroupsOf({
                  settled: [group, group,],
                  outcomes: new Map([['0:1', 'replaced',], ['1:1', 'kept-floor',],],),
                  gaveWayTo,
                },),
              },),
              provisional: recordedAs({
                groups: finalGroupsOf({
                  settled: [{ ...group, outcomes: ['chosen', 'kept-current',], },],
                  outcomes: new Map(),
                  gaveWayTo,
                },),
              },),
            },).toEqual({
              overlapped: [
                [['OCCURRENCE 1', 1, 'chosen', 'none',], ['OCCURRENCE 2', 3, 'overlapped', { group: 1, label: 'OCCURRENCE 1', },],],
              ],
              replacedWithStaleName: [
                [['OCCURRENCE 1', 1, 'chosen', 'none',], ['OCCURRENCE 2', 3, 'replaced', 'none',],],
                [['OCCURRENCE 1', 1, 'chosen', 'none',], ['OCCURRENCE 2', 3, 'kept-floor', 'none',],],
              ],
              provisional: [
                [['OCCURRENCE 1', 1, 'chosen', 'none',], ['OCCURRENCE 2', 3, 'kept-current', 'none',],],
              ],
            },);
          },
        },),
        it({
          name: 'RECORDS WHY A KEPT-CURRENT PLACE WAS LEFT AS IT WAS, the reason the apply stage gave a gain it left out over the '
            + 'reason the group holds of a splice refused at the edge, and no reason for a place of any other outcome',
          fn: async () => {
            const group = twoPlaceGroup();
            /**
             The reasons of the second place of the group, as the apply stage and as the decision of the group give them.
             */
            const reasons = {
              fromApply: new Map([['0:1', 'gain-pairs' as const,],],),
              fromGroup: ['edge-reading-changes' as const, 'edge-second-mark' as const,],
            };
            /**
             The reason recorded under the second place of a final group.
             */
            function reasonRecorded({ groups, }: { readonly groups: readonly ArtifactConsistencyGroup[]; },): unknown {
              return groups.flatMap(function placesOf({ occurrences, },): readonly unknown[] {
                return occurrences.map(function reasonOf({ keptBecause, },): unknown {
                  return keptBecause ?? 'none';
                },);
              },);
            }
            expect({
              fromApply: reasonRecorded({
                groups: finalGroupsOf({
                  settled: [{ ...group, outcomes: ['chosen', 'kept-current',], },],
                  outcomes: new Map([['0:1', 'kept-current',],],),
                  gaveWayTo: new Map(),
                  keptBecause: reasons.fromApply,
                },),
              },),
              fromGroup: reasonRecorded({
                groups: finalGroupsOf({
                  settled: [{ ...group, outcomes: ['chosen', 'kept-current',], leftBecause: reasons.fromGroup, },],
                  outcomes: new Map(),
                  gaveWayTo: new Map(),
                  keptBecause: new Map(),
                },),
              },),
              applyWins: reasonRecorded({
                groups: finalGroupsOf({
                  settled: [{ ...group, outcomes: ['chosen', 'kept-current',], leftBecause: ['edge-second-mark', 'edge-second-mark',], },],
                  outcomes: new Map([['0:1', 'kept-current',],],),
                  gaveWayTo: new Map(),
                  keptBecause: reasons.fromApply,
                },),
              },),
              notKeptCurrent: reasonRecorded({
                groups: finalGroupsOf({
                  settled: [{ ...group, leftBecause: reasons.fromGroup, },],
                  outcomes: new Map([['0:1', 'replaced',],],),
                  gaveWayTo: new Map(),
                  keptBecause: reasons.fromApply,
                },),
              },),
              none: reasonRecorded({
                groups: finalGroupsOf({
                  settled: [{ ...group, outcomes: ['chosen', 'kept-current',], },],
                  outcomes: new Map(),
                  gaveWayTo: new Map(),
                  keptBecause: new Map(),
                },),
              },),
            },).toEqual({
              fromApply: ['none', 'gain-pairs',],
              fromGroup: ['none', 'edge-second-mark',],
              applyWins: ['none', 'gain-pairs',],
              notKeptCurrent: ['none', 'none',],
              none: ['none', 'none',],
            },);
          },
        },),
        it({
          name: 'REFUSES A PLACE WITH NO RENDERING OR OUTCOME, A PLACE TO BE REPLACED THAT NO REPLACEMENT REACHED, AND AN OVERLAPPED '
            + 'PLACE WITH NO PLACE IT GAVE WAY TO, naming the place and the group',
          fn: async () => {
            const group = twoPlaceGroup();
            /**
             The refusal finalizing the groups raises.
             */
            function refusalOf(
              { settled, outcomes, }: { readonly settled: SettledGroup; readonly outcomes: ReadonlyMap<string, 'overlapped'>; },
            ): unknown {
              return caught(function finalizes(): unknown {
                return finalGroupsOf({ settled: [settled,], outcomes, gaveWayTo: new Map(), },);
              },);
            }
            const refusals = [
              refusalOf({ settled: { ...group, renderings: group.renderings.slice(0, 1,), }, outcomes: new Map(), },),
              refusalOf({ settled: group, outcomes: new Map(), },),
              refusalOf({ settled: group, outcomes: new Map([['0:1', 'overlapped',],],), },),
            ];
            expect(refusals.every(function isInvariant(refusal,): boolean {
              return refusal instanceof ConsistencyInvariantError;
            },),).toBe(true,);
            expect(refusals.map(String,),).toEqual([
              'ConsistencyInvariantError: unreachable: OCCURRENCE 2 of group 1 has no rendering or outcome, and both are made '
                + 'for every place',
              'ConsistencyInvariantError: unreachable: OCCURRENCE 2 of group 1 was to be replaced and no replacement was applied, '
                + 'left out or refused, and every pending replacement is one of the three',
              'ConsistencyInvariantError: unreachable: OCCURRENCE 2 of group 1 ended overlapped and no place was recorded for it '
                + 'to have given way to, and every overlap is of a replacement or a located place',
            ],);
          },
        },),
      ],
    },),
  ],
},);
