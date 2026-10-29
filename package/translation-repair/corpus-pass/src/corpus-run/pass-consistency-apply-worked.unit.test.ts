/**
 Tests for reading the slices of a page that carry English with their replacements: which replacements each slice has,
 which of them are code splices, which change the words another group located and so are not candidates, and that a slice
 that ships no English is not read.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type { PageLayout, } from '@monochromatic-dev/translation-repair-consistency';
import {
  type ConsistencySliceFacts,
  type SettledGroup,
  workedPageOf,
} from '../../dist/final/node/index.mjs';
import { pairsOver, } from '@monochromatic-dev/translation-repair-slice-reading/ts/span-pairs.test-fixture.ts';
import {
  groupIn,
  withEdit,
} from './consistency-apply-group.test-fixture.ts';
import { factsOf, } from './consistency-round-page.test-fixture.ts';

/**
 The English of the first slice, which holds the chosen words.
 */
const FIRST = 'Fill the parents’ bowl.';

/**
 The English of the third slice, which holds the receiving words.
 */
const THIRD = 'Fill the family bowl.';

/**
 A group whose chosen place stands in the first slice and whose receiving place a code splice in the third replaces.

 @returns Group with its pending splice

 @example
 ```ts
 const group = splicingGroup();
 ```
 */
function splicingGroup(): SettledGroup {
  return withEdit({
    group: groupIn({
      places: [
        { slice: 0, english: FIRST, quote: 'the parents’', },
        { slice: 2, english: THIRD, quote: 'the family', },
      ],
    },),
    edit: { kind: 'splice', placePositions: [1,], sliceIndex: 2, start: 5, end: 15, text: 'the parents’', },
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: workedPageOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS EVERY SLICE THAT CARRIES ENGLISH, in page order, and none that ships nothing, with its replacements, its '
            + 'candidates and the names of those that are code splices',
          fn: async () => {
            const { slices, } = workedPageOf({
              settled: [splicingGroup(),],
              slices: factsOf({
                englishes: [
                  { sliceIndex: 0, english: FIRST, },
                  { sliceIndex: 1, english: '', },
                  { sliceIndex: 2, english: THIRD, },
                ],
              },),
            },);
            expect(slices.map(function seenOf({ facts, currentText, named, candidates, splices, leftOut, },): unknown {
              return {
                slice: facts.sliceIndex,
                text: currentText,
                edits: named.edits.length,
                candidates: candidates.map(function nameOf({ id, },): string {
                  return id;
                },),
                splices: [...splices,],
                leftOut: leftOut.length,
              };
            },),).toEqual([
              { slice: 0, text: FIRST, edits: 0, candidates: [], splices: [], leftOut: 0, },
              {
                slice: 2,
                text: THIRD,
                edits: 1,
                candidates: ['group 1 OCCURRENCE 2',],
                splices: ['group 1 OCCURRENCE 2',],
                leftOut: 0,
              },
            ],);
          },
        },),
        it({
          name: 'LEAVES OUT A REPLACEMENT THAT CHANGES THE WORDS ANOTHER GROUP LOCATED, naming the replacement and the place, '
            + 'and makes it no candidate',
          fn: async () => {
            /**
             A group that locates the words `family bowl` in the third slice and replaces nothing.
             */
            const locating = groupIn({ places: [{ slice: 2, english: THIRD, quote: 'family bowl', },], },);
            const { slices, } = workedPageOf({
              settled: [splicingGroup(), locating,],
              slices: factsOf({
                englishes: [
                  { sliceIndex: 0, english: FIRST, },
                  { sliceIndex: 2, english: THIRD, },
                ],
              },),
            },);
            expect(slices.map(function seenOf({ leftOut, candidates, },): unknown {
              return { leftOut, candidates: candidates.length, };
            },),).toEqual([
              { leftOut: [], candidates: 0, },
              {
                leftOut: [{ id: 'group 1 OCCURRENCE 2', other: { group: 1, label: 'OCCURRENCE 1', start: 9, }, },],
                candidates: 0,
              },
            ],);
          },
        },),
        it({
          name: 'HANDS ON THE LAYOUT THE FACTS SHARE and reads the text around the slices off the page assembled from it with the '
            + 'wording the slices carry, the span of a slice that ships nothing joining the text on both sides of it; and hands on '
            + 'no layout and no text when the slices carry different layouts',
          fn: async () => {
            const note = '\n\n<!-- ‘note -->\n\n';
            const targetText = `${FIRST}${note}Gone.\n\n${THIRD}`;
            const goneAt = targetText.indexOf('Gone.',);
            const thirdAt = targetText.indexOf(THIRD,);
            const layout: PageLayout = {
              targetText,
              chunks: pairsOver({
                targetText,
                spans: [[0, FIRST.length,], [goneAt, goneAt + 5,], [thirdAt, targetText.length,],],
              },),
              silent: [{ sliceIndex: 1, replacementText: '', },],
            };
            const laidOut = factsOf({
              englishes: [
                { sliceIndex: 0, english: FIRST, },
                { sliceIndex: 2, english: THIRD, },
              ],
            },)
              .map(function withLayout(facts,): ConsistencySliceFacts {
                return { ...facts, layout, };
              },);
            const page = workedPageOf({ settled: [splicingGroup(),], slices: laidOut, },);
            expect(page.layout,).toBe(layout,);
            expect(page.between,).toEqual(['', `${note}\n\n`, '',],);
            const other = workedPageOf({
              settled: [splicingGroup(),],
              slices: [nonNullishOrThrow(laidOut[0],), { ...nonNullishOrThrow(laidOut[1],), layout: { ...layout, }, },],
            },);
            expect([other.layout, other.between,],).toEqual([undefined, undefined,],);
          },
        },),
        it({
          name: 'READS A PAGE OF NO SLICE THAT CARRIES ENGLISH AS A PAGE OF NO SLICE, with no text between',
          fn: async () => {
            expect(workedPageOf({
              settled: [],
              slices: factsOf({ englishes: [{ sliceIndex: 0, english: '', },], },),
            },),).toEqual({ slices: [], },);
          },
        },),
      ],
    },),
  ],
},);
