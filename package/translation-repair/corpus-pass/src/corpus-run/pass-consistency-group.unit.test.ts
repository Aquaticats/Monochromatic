/**
 Tests for what the consistency pass makes of one repeat group before any judge
 is asked: the places a slice holds whole, how the repeat is worded for the
 sheet and the artifact, whether the bench is asked at all, and what the sheet
 shows of one place.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { repeatGroupsOf, } from '@monochromatic-dev/translation-repair-corpus-source';
import {
  type ConsistencyPlace,
  type ConsistencySliceFacts,
  consistencySlicesOf,
  dispositionOf,
  occurrenceViewOf,
  placesOf,
  repeatOf,
} from '../../dist/final/node/index.mjs';
import {
  archiveEnglishOf,
  PAGE_SOURCE,
  preparedCatPage,
  projectedCatPage,
} from '@monochromatic-dev/translation-repair-run/ts/corpus-run/consistency-page.test-fixture.ts';

/**
 What the pass knows of each slice of the cat page, the archive standing.

 @returns One entry per slice

 @example
 ```ts
 const slices = catSlices();
 ```
 */
function catSlices(): readonly ConsistencySliceFacts[] {
  const prepared = preparedCatPage();
  return consistencySlicesOf({
    prepared,
    projected: projectedCatPage({ prepared, english: archiveEnglishOf({ prepared, },), },),
    contestSlices: [],
    consolidateSlices: [],
    archiveDisputes: new Map(),
  },);
}

/**
 The repeat groups of the cat page's original.

 @returns Sentence group and refrain group, in page order

 @example
 ```ts
 const [sentence, refrain,] = catGroups();
 ```
 */
function catGroups(): ReturnType<typeof repeatGroupsOf> {
  return repeatGroupsOf({
    text: PAGE_SOURCE,
    slices: preparedCatPage().slices
      .map(function sourceOf(slice,) {
        return slice.source;
      },),
  },);
}

/**
 One group of the cat page.

 @param position - position of the group on the page

 @returns Group

 @example
 ```ts
 const refrain = catGroup({ position: 1, },);
 ```
 */
function catGroup({ position, }: { readonly position: number; },): ReturnType<typeof repeatGroupsOf>[number] {
  return nonNullishOrThrow(catGroups()[position],);
}

/**
 The places of one group of the cat page.

 @param position - position of the group on the page

 @returns Its places, each with its slice

 @example
 ```ts
 const places = catPlaces({ position: 0, },);
 ```
 */
function catPlaces({ position, }: { readonly position: number; },): readonly ConsistencyPlace[] {
  return placesOf({ group: catGroup({ position, },), slices: catSlices(), },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: placesOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'TIES EACH OCCURRENCE TO THE FACTS OF ITS SLICE, with its offsets and line, and a frame\'s place to its slot',
          fn: async () => {
            const sentence = catPlaces({ position: 0, },);
            const refrain = catPlaces({ position: 1, },);
            expect({
              sentence: sentence.map(function shape(place,): unknown {
                return [place.slice.sliceIndex, place.startOffset, place.endOffset, place.line, place.slotStartOffset,];
              },),
              refrain: refrain.map(function shape(place,): unknown {
                return [place.slice.sliceIndex, place.slotStartOffset, place.slotEndOffset,];
              },),
            },).toEqual({
              sentence: [[0, 6, 18, 3, undefined,], [2, 85, 97, 11, undefined,],],
              refrain: [[0, 27, 29,], [1, 64, 66,], [2, 113, 115,], [3, 150, 152,],],
            },);
          },
        },),
        it({
          name: 'REFUSES A GROUP THAT NAMES A SLICE THE PASS KNOWS NOTHING OF, since the groups are found over the same slices',
          fn: async () => {
            const refusal = caught(function act(): unknown {
              return placesOf({
                group: catGroup({ position: 0, },),
                slices: catSlices()
                  .filter(function notThird(slice,): boolean {
                    return slice.sourceIndex !== 2;
                  },),
              },);
            },);
            expect(String(refusal,),).toBe(
              'ConsistencyInvariantError: unreachable: a repeat group names source slice 2, and the groups are found '
                + 'over the same slices the facts are gathered from',
            );
          },
        },),
      ],
    },),
    describe({
      name: repeatOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SAYS A MAXIMAL REPEAT AS THE ORIGINAL WRITES IT and a refrain as its frame',
          fn: async () => {
            expect([
              repeatOf({ group: catGroup({ position: 0, },), text: PAGE_SOURCE, },),
              repeatOf({ group: catGroup({ position: 1, },), text: PAGE_SOURCE, },),
            ],).toEqual([
              { kind: 'maximal', wording: '窗台上的阳光让小猫睡着了', },
              { kind: 'refrain', prefix: '所以这只猫一定是', suffix: '的小宝贝', },
            ],);
          },
        },),
        it({
          name: 'READS A MAXIMAL REPEAT FROM A PLACE NO SLICE HOLDS WHEN IT STANDS FIRST, since the wording is the same',
          fn: async () => {
            const [first, second,] = catGroup({ position: 0, },).occurrences;
            expect(repeatOf({
              group: {
                kind: 'maximal',
                length: 12,
                occurrences: [nonNullishOrThrow(second,),],
                unplaced: [
                  {
                    startOffset: nonNullishOrThrow(first,).startOffset,
                    endOffset: nonNullishOrThrow(first,).endOffset,
                    line: nonNullishOrThrow(first,).line,
                    reason: 'in-no-slice',
                  },
                ],
              },
              text: PAGE_SOURCE,
            },),).toEqual({ kind: 'maximal', wording: '窗台上的阳光让小猫睡着了', },);
          },
        },),
        it({
          name: 'REFUSES A MAXIMAL GROUP THAT LISTS NO PLACE AT ALL',
          fn: async () => {
            const refusal = caught(function act(): unknown {
              return repeatOf({
                group: { kind: 'maximal', length: 12, occurrences: [], unplaced: [], },
                text: PAGE_SOURCE,
              },);
            },);
            expect(String(refusal,),).toBe(
              'ConsistencyInvariantError: unreachable: a repeat group lists no place, and the detector reports a '
                + 'repeat only where it stands in two',
            );
          },
        },),
      ],
    },),
    describe({
      name: dispositionOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ASKS THE BENCH ABOUT PLACES THAT MAY RENDER THE WORDING DIFFERENTLY',
          fn: async () => {
            expect(dispositionOf({ places: catPlaces({ position: 1, },), },),).toEqual({ kind: 'ask', },);
          },
        },),
        it({
          name: 'SKIPS A GROUP WITH FEWER THAN TWO PLACES A SLICE HOLDS WHOLE',
          fn: async () => {
            expect(dispositionOf({ places: catPlaces({ position: 0, },).slice(0, 1,), },),)
              .toEqual({ kind: 'skip', reason: 'too-few-placed', },);
          },
        },),
        it({
          name: 'SKIPS A GROUP WITH ENGLISH IN FEWER THAN TWO PLACES',
          fn: async () => {
            const places = catPlaces({ position: 0, },).map(function bare(place, at,): ConsistencyPlace {
              return (at === 0) ? place : { ...place, slice: { ...place.slice, current: { kind: 'absent', }, }, };
            },);
            expect(dispositionOf({ places, },),).toEqual({ kind: 'skip', reason: 'no-english', },);
          },
        },),
        it({
          name: 'SKIPS A GROUP WHOSE PLACES EACH SIT IN A SLICE OF ONE ENGLISH, since nothing is rendered two ways',
          fn: async () => {
            const [first, second,] = catPlaces({ position: 0, },);
            const alike = { kind: 'present', text: 'The sunny window put the kitten to sleep.', } as const;
            expect(dispositionOf({
              places: [
                { ...nonNullishOrThrow(first,), slice: { ...nonNullishOrThrow(first,).slice, current: alike, }, },
                { ...nonNullishOrThrow(second,), slice: { ...nonNullishOrThrow(second,).slice, current: alike, }, },
              ],
            },),).toEqual({ kind: 'skip', reason: 'consistent', });
          },
        },),
        it({
          name: 'ASKS ABOUT TWO PLACES IN ONE SLICE even where the slice has one English, since it may render the wording twice',
          fn: async () => {
            const [first,] = catPlaces({ position: 0, },);
            expect(dispositionOf({
              places: [nonNullishOrThrow(first,), { ...nonNullishOrThrow(first,), startOffset: 30, endOffset: 40, },],
            },),).toEqual({ kind: 'ask', },);
          },
        },),
      ],
    },),
    describe({
      name: occurrenceViewOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SHOWS THE BLOCK HOLDING THE PLACE WITH THE WORDING MARKED, and the English of its slice',
          fn: async () => {
            const [first,] = catPlaces({ position: 0, },);
            expect(occurrenceViewOf({ place: nonNullishOrThrow(first,), text: PAGE_SOURCE, },),).toEqual({
              sliceIndex: 0,
              paragraph: '窗台上的阳光让小猫睡着了。所以这只猫一定是胖胖的小宝贝。',
              markStart: 0,
              markEnd: 12,
              // The whole original of the slice, so the sheets keep its comments in the English.
              sliceSource: nonNullishOrThrow(first,).slice
                .sourceText,
              english: {
                kind: 'present',
                text: '## One\n\nThe sunny window put the kitten to sleep. So this cat must be a chubby little treasure.',
              },
            },);
          },
        },),
        it({
          name: 'SHOWS A FRAME\'S SLOT IN THE ORIGINAL BESIDE THE MARKED PATTERN',
          fn: async () => {
            const [, second,] = catPlaces({ position: 1, },);
            const view = occurrenceViewOf({ place: nonNullishOrThrow(second,), text: PAGE_SOURCE, },);
            expect([view.slotSource, view.paragraph.slice(view.markStart, view.markEnd,),],)
              .toEqual(['懒猫', '所以这只猫一定是懒猫的小宝贝',],);
          },
        },),
        it({
          name: 'SHOWS THE WHOLE ORIGINAL OF THE SLICE WHERE NO BLOCK HOLDS THE PLACE, with the offsets inside it',
          fn: async () => {
            const [first,] = catPlaces({ position: 0, },);
            const place = nonNullishOrThrow(first,);
            const view = occurrenceViewOf({
              place: { ...place, slice: { ...place.slice, nodes: [], }, },
              text: PAGE_SOURCE,
            },);
            expect([view.paragraph, view.markStart, view.markEnd,],).toEqual([place.slice.sourceText, 6, 18,],);
          },
        },),
        it({
          name: 'REFUSES A PLACE WHOSE OFFSETS READ OTHER WORDS IN THE BLOCK THAN IN THE WHOLE ORIGINAL',
          fn: async () => {
            const [first,] = catPlaces({ position: 0, },);
            const place = nonNullishOrThrow(first,);
            const refusal = caught(function act(): unknown {
              return occurrenceViewOf({ place, text: PAGE_SOURCE.replace('窗台上的阳光', '窗台下的阳光',), },);
            },);
            expect(String(refusal,),).toBe(
              'ConsistencyInvariantError: unreachable: the offsets of a place point at other words in the text shown '
                + 'than in the whole original (slice 0), and both are cut from the same offsets',
            );
          },
        },),
      ],
    },),
  ],
},);
