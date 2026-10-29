/**
 Tests for the stretches of a crossing place's words that a writer's paragraph must
 hold at an edge of its block: which stretches a block has, and when a paragraph
 holds them.

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
  crossingOfUnit,
  crossingWordsLost,
  crossingWordsOf,
} from '../../dist/final/node/index.mjs';

/**
 The block the cases rewrite: the second paragraph of a slice whose first paragraph ends
 where the block begins.
 */
const BLOCK = {
  start: 33,
  end: 33 + 'to sleep. The kitchen was warm. Fill the cats bowl.'.length,
  text: 'to sleep. The kitchen was warm. Fill the cats bowl.',
};

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: crossingWordsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'GIVES THE STRETCH INSIDE THE BLOCK AT ITS START FOR A SPAN THAT COMES IN OVER THE START',
          fn: async () => {
            expect(crossingWordsOf({
              block: BLOCK,
              spans: [{ start: 0, core: 'The sunny window put the kitten\n\nto sleep', },],
            },),).toEqual([{ edge: 'start', words: 'to sleep', },],);
          },
        },),
        it({
          name: 'GIVES THE STRETCH INSIDE THE BLOCK AT ITS END FOR A SPAN THAT GOES OUT OVER THE END',
          fn: async () => {
            expect(crossingWordsOf({
              block: BLOCK,
              spans: [{ start: BLOCK.end - 'cats bowl.'.length, core: 'cats bowl.\n\nThen the dogs nap', },],
            },),).toEqual([{ edge: 'end', words: 'cats bowl.', },],);
          },
        },),
        it({
          name: 'GIVES THE WHOLE BLOCK AT BOTH EDGES FOR A SPAN THAT COMES IN OVER THE START AND GOES OUT OVER THE END',
          fn: async () => {
            expect(crossingWordsOf({
              block: BLOCK,
              spans: [{ start: 4, core: 'x'.repeat(BLOCK.end,), },],
            },),).toEqual([{ edge: 'start', words: BLOCK.text, }, { edge: 'end', words: BLOCK.text, },],);
          },
        },),
        it({
          name: 'GIVES NOTHING FOR A SPAN WHOLLY INSIDE THE BLOCK, WHOLLY OUTSIDE IT OR ENDING AT ITS START OR STARTING AT ITS END',
          fn: async () => {
            expect(crossingWordsOf({
              block: BLOCK,
              spans: [
                { start: BLOCK.start + 3, core: 'sleep', },
                { start: 0, core: 'The sunny window', },
                { start: 10, core: 'x'.repeat(BLOCK.start - 10,), },
                { start: BLOCK.end, core: 'Then the dogs nap', },
              ],
            },),).toEqual([],);
          },
        },),
        it({
          name: 'GIVES THE STRETCHES OF SEVERAL SPANS IN THE ORDER OF THE SPANS, and none for no span',
          fn: async () => {
            expect([
              crossingWordsOf({
                block: BLOCK,
                spans: [
                  { start: BLOCK.end - 5, core: 'bowl.\n\nNext', },
                  { start: 17, core: 'put the kitten\n\nto', },
                ],
              },),
              crossingWordsOf({ block: BLOCK, spans: [], },),
            ],).toEqual([
              [{ edge: 'end', words: 'bowl.', }, { edge: 'start', words: 'to', },],
              [],
            ],);
          },
        },),
      ],
    },),
    describe({
      name: crossingOfUnit.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS ONLY THE PLACES OF THE UNIT\'S SLICE THAT ARE NO MEMBER OF THE UNIT AND HAVE NO SPLICE',
          fn: async () => {
            /**
             A held receiving place.

             @param sliceIndex - slice it stands in
             @param kind - how the chosen words move into it
             @param start - where its words start
             @param core - its words

             @returns Held place
             */
            const heldIn = (
              {
                sliceIndex,
                kind,
                start,
                core,
              }: {
                readonly sliceIndex: number;
                readonly kind: string;
                readonly start: number;
                readonly core: string;
              },
            ): { readonly receiver: { readonly place: { readonly slice: { readonly sliceIndex: number; }; }; readonly plan: { readonly kind: string; }; readonly rendering: { readonly start: number; readonly core: string; }; }; } => ({
              receiver: { place: { slice: { sliceIndex, }, }, plan: { kind, }, rendering: { start, core, }, },
            });
            const member = heldIn({ sliceIndex: 1, kind: 'replace', start: 40, core: 'cats', },);
            const comesIn = heldIn({ sliceIndex: 1, kind: 'unavailable', start: 0, core: 'The sunny window put the kitten\n\nto sleep', },);
            const withSplice = heldIn({ sliceIndex: 1, kind: 'replace', start: 0, core: 'The sunny window put the kitten\n\nto sleep', },);
            const elsewhere = heldIn({ sliceIndex: 2, kind: 'unavailable', start: 0, core: 'The sunny window put the kitten\n\nto sleep', },);
            expect([
              crossingOfUnit({ block: BLOCK, members: [member,], held: [member, comesIn,], kept: [], },),
              crossingOfUnit({ block: BLOCK, members: [member,], held: [member, withSplice,], kept: [], },),
              crossingOfUnit({ block: BLOCK, members: [member,], held: [member, elsewhere,], kept: [], },),
              crossingOfUnit({ block: BLOCK, members: [comesIn,], held: [comesIn,], kept: [], },),
              crossingOfUnit({ block: BLOCK, members: [], held: [], kept: [], },),
            ],).toEqual([
              [{ edge: 'start', words: 'to sleep', },],
              [],
              [],
              [],
              [],
            ],);
          },
        },),
        it({
          name: 'READS ALSO THE WORDS OF THE GROUP\'S OWN PLACES THAT MUST STAY, the chosen place, a place kept for its context and '
            + 'a place that already agrees alike, and not those of another group, of another slice, wholly inside the block or '
            + 'wholly outside it',
          fn: async () => {
            /**
             A receiving place of the unit, in the second paragraph of slice one.
             */
            const member = {
              receiver: {
                place: { slice: { sliceIndex: 1, }, },
                plan: { kind: 'replace', },
                rendering: { start: 40, core: 'cats', },
              },
            };
            /**
             The words of a place that must stay, in a slice.
             */
            const staying = (
              {
                sliceIndex = 1,
                start,
                core,
              }: {
                readonly sliceIndex?: number;
                readonly start: number;
                readonly core: string;
              },
            ): { readonly sliceIndex: number; readonly start: number; readonly core: string; } => ({ sliceIndex, start, core, });
            const comesIn = staying({ start: 0, core: 'The sunny window put the kitten\n\nto sleep', });
            const goesOut = staying({ start: BLOCK.end - 'cats bowl.'.length, core: 'cats bowl.\n\nThen the dogs nap', });
            const inside = staying({ start: BLOCK.start + 3, core: 'sleep', });
            const outside = staying({ start: 0, core: 'The sunny window', });
            const elsewhere = staying({ sliceIndex: 2, start: 0, core: 'The sunny window put the kitten\n\nto sleep', });
            const otherGroups = { ...comesIn, ofOtherGroup: true as const, };
            expect([
              crossingOfUnit({ block: BLOCK, members: [member,], held: [member,], kept: [comesIn,], },),
              crossingOfUnit({ block: BLOCK, members: [member,], held: [member,], kept: [goesOut,], },),
              crossingOfUnit({ block: BLOCK, members: [member,], held: [member,], kept: [otherGroups,], },),
              crossingOfUnit({ block: BLOCK, members: [member,], held: [member,], kept: [elsewhere,], },),
              crossingOfUnit({ block: BLOCK, members: [member,], held: [member,], kept: [inside, outside,], },),
            ],).toEqual([
              [{ edge: 'start', words: 'to sleep', },],
              [{ edge: 'end', words: 'cats bowl.', },],
              [],
              [],
              [],
            ],);
          },
        },),
        it({
          name: 'LISTS THE STRETCHES OF RECEIVING PLACES WITH NO SPLICE AND OF PLACES THAT STAY TOGETHER, in the page order of the '
            + 'places whichever list a place came from',
          fn: async () => {
            const member = {
              receiver: {
                place: { slice: { sliceIndex: 1, }, },
                plan: { kind: 'replace', },
                rendering: { start: 40, core: 'cats', },
              },
            };
            const goesOut = {
              receiver: {
                place: { slice: { sliceIndex: 1, }, },
                plan: { kind: 'unavailable', },
                rendering: { start: BLOCK.end - 'cats bowl.'.length, core: 'cats bowl.\n\nThen the dogs nap', },
              },
            };
            const comesIn = { sliceIndex: 1, start: 0, core: 'The sunny window put the kitten\n\nto sleep', };
            expect(crossingOfUnit({ block: BLOCK, members: [member,], held: [member, goesOut,], kept: [comesIn,], },),).toEqual([
              { edge: 'start', words: 'to sleep', },
              { edge: 'end', words: 'cats bowl.', },
            ],);
          },
        },),
      ],
    },),
    describe({
      name: crossingWordsLost.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LOSES NOTHING WHERE THE PARAGRAPH STARTS WITH THE START STRETCH AND ENDS WITH THE END STRETCH',
          fn: async () => {
            expect([
              crossingWordsLost({
                paragraph: 'to sleep. The kitchen felt warm.',
                crossing: [{ edge: 'start', words: 'to sleep', },],
              },),
              crossingWordsLost({
                paragraph: 'Fill the kittens purr bowl.',
                crossing: [{ edge: 'end', words: 'bowl.', },],
              },),
              crossingWordsLost({
                paragraph: 'Fill the cats.',
                crossing: [],
              },),
            ],).toEqual([[], [], [],],);
          },
        },),
        it({
          name: 'LOSES THE STRETCH THE PARAGRAPH CHANGED, MOVED FROM ITS EDGE OR DROPPED',
          fn: async () => {
            expect([
              crossingWordsLost({
                paragraph: 'to doze. The kitchen felt warm.',
                crossing: [{ edge: 'start', words: 'to sleep', },],
              },),
              crossingWordsLost({
                paragraph: 'The kitchen felt warm. to sleep.',
                crossing: [{ edge: 'start', words: 'to sleep', },],
              },),
              crossingWordsLost({
                paragraph: 'to sleep. Fill the bowl.',
                crossing: [{ edge: 'end', words: 'to sleep', },],
              },),
              crossingWordsLost({
                paragraph: 'The kitchen felt warm.',
                crossing: [{ edge: 'end', words: 'bowl.', },],
              },),
            ],).toEqual([
              [{ edge: 'start', words: 'to sleep', },],
              [{ edge: 'start', words: 'to sleep', },],
              [{ edge: 'end', words: 'to sleep', },],
              [{ edge: 'end', words: 'bowl.', },],
            ],);
          },
        },),
        it({
          name: 'READS RUNS OF WHITE SPACE AS ONE SPACE, so words laid out again on other lines are the same words',
          fn: async () => {
            expect(crossingWordsLost({
              paragraph: 'put the\nkitten  to sleep. The kitchen felt warm.',
              crossing: [{ edge: 'start', words: 'put the kitten\n\nto sleep', },],
            },),).toEqual([],);
          },
        },),
        it({
          name: 'LOSES A STRETCH THAT THE PARAGRAPH CONTINUES OR PRECEDES AS PART OF A LONGER WORD',
          fn: async () => {
            expect([
              crossingWordsLost({
                paragraph: 'to sleeping. The kitchen felt warm.',
                crossing: [{ edge: 'start', words: 'to sleep', },],
              },),
              crossingWordsLost({
                paragraph: 'The kitchen felt warm in the unsun on the sill.',
                crossing: [{ edge: 'end', words: 'sun on the sill.', },],
              },),
            ],).toEqual([
              [{ edge: 'start', words: 'to sleep', },],
              [{ edge: 'end', words: 'sun on the sill.', },],
            ],);
          },
        },),
      ],
    },),
  ],
},);
