/**
 Tests for planning the replacements of every slice of a page before any is recorded: which replacements a slice makes,
 and which code splices are left out because the apostrophe they carry in would not be one on the page, a page whose
 other slices hold the quotation mark that could pair with it included, and the rounds of the check and the floors that
 settle which slices are made.

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

import { pageOfSlices, } from '@monochromatic-dev/translation-repair-consistency';
import {
  type ConsistencyFloorVerdict,
  type ConsistencySliceFacts,
  plansOnPage,
  type SettledGroup,
  settledSlicesOf,
  workedPageOf,
} from '../../dist/final/node/index.mjs';
import {
  groupIn,
  withEdit,
} from './consistency-apply-group.test-fixture.ts';
import {
  factsOf,
  roundPage,
  roundPageWithTwoEdits,
  THIRD_WITH_CLOSER,
} from './consistency-round-page.test-fixture.ts';

/**
 A group whose first place stands in the first slice and whose second place a code splice in the second slice replaces.

 @param first - English of the first slice

 @param second - English of the second slice

 @param splice - where the splice starts and ends in the second slice and the words it puts in

 @returns Group

 @example
 ```ts
 const group = splicedGroup({ first: 'Fill the parents’ bowl.', second: 'Then the family naps.', splice: { start: 5, end: 15, text: 'the parents’', }, },);
 ```
 */
function splicedGroup(
  {
    first,
    second,
    splice,
  }: {
    readonly first: string;
    readonly second: string;
    readonly splice: { readonly start: number; readonly end: number; readonly text: string; };
  },
): SettledGroup {
  return withEdit({
    group: groupIn({
      places: [
        { slice: 0, english: first, quote: 'the parents', },
        { slice: 2, english: second, quote: 'the family', },
      ],
    },),
    edit: { kind: 'splice', placePositions: [1,], sliceIndex: 2, ...splice, },
  },);
}

/**
 The facts of a page, each slice that carries English given the page as it will be published: the slices' English and the
 text the case says the page carries around them.

 @param slices - facts of the slices, in page order
 @param between - text before the first slice that carries English, between each two and after the last
 @returns Facts, slice that carries English now holding the page and where it starts in it
 @example
 ```ts
 const slices = withPageBetween({ slices: factsOf({ englishes, },), between: ['', '\n\n', '',], },);
 ```
 */
function withPageBetween(
  {
    slices,
    between,
  }: {
    readonly slices: readonly ConsistencySliceFacts[];
    readonly between: readonly string[];
  },
): readonly ConsistencySliceFacts[] {
  /**
   The page made of the slices that carry English.
   */
  const page = pageOfSlices({
    slices: slices.flatMap(function presentIn({ sliceIndex, current, },): readonly { readonly sliceIndex: number; readonly english: string; }[] {
      return (current.kind === 'present') ? [{ sliceIndex, english: current.text, },] : [];
    },),
    between,
  },);
  return slices.map(function pagedFacts(facts,): ConsistencySliceFacts {
    /**
     Where the slice starts in the page, none for a slice that carries no English.
     */
    const offset = page.offsets.get(facts.sliceIndex,);
    if (offset === undefined)
      return facts;
    return { ...facts, page: { text: page.text, offset, }, };
  },);
}

/**
 Plans the slices of a page from its groups and the facts of its slices.

 @param settled - groups in page order, with their pending replacements
 @param slices - what the pass knows of each slice, in page order
 @param refused - indexes of the slices the floors refused; none by default
 @returns Plan of each slice that has replacements
 @example
 ```ts
 const plans = slicePlansOf({ settled, slices, },);
 ```
 */
function slicePlansOf(
  {
    settled,
    slices,
    refused = new Set(),
  }: {
    readonly settled: readonly SettledGroup[];
    readonly slices: readonly ConsistencySliceFacts[];
    readonly refused?: ReadonlySet<number>;
  },
): ReturnType<typeof plansOnPage> {
  return plansOnPage({ page: workedPageOf({ settled, slices, },), refused, },);
}

/**
 What the plans of a page come to, for a case to compare.

 @param plans - plans of the slices that have replacements

 @returns For each slice, its index, the names of the replacements it makes and of the splices left out for their apostrophe, and the replacements left out for the words of another group

 @example
 ```ts
 const seen = seenIn({ plans, },);
 ```
 */
function seenIn(
  { plans, }: { readonly plans: ReturnType<typeof slicePlansOf>; },
): readonly unknown[] {
  return plans.map(function seenOf({
    facts,
    toMake,
    unprovenGains,
    leftOut,
  },): unknown {
    return {
      slice: facts.sliceIndex,
      toMake: toMake.map(function nameOf({ id, },): string {
        return id;
      },),
      unprovenGains: unprovenGains.map(function nameOf({ id, },): string {
        return id;
      },),
      leftOut: leftOut.length,
    };
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: plansOnPage.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PLANS THE SLICES THAT CARRY ENGLISH AND HAVE REPLACEMENTS, in page order, and no slice without a replacement',
          fn: async () => {
            const second = 'Later the kitten naps in the sun.';
            const group = withEdit({
              group: groupIn({
                places: [
                  { slice: 0, english: 'Dusk: the kitten purrs in the sun.', quote: 'in the sun', },
                  { slice: 2, english: second, quote: 'naps', },
                ],
              },),
              edit: { kind: 'splice', placePositions: [1,], sliceIndex: 2, start: 17, end: 21, text: 'sleeps', },
            },);
            expect(seenIn({
              plans: slicePlansOf({
                settled: [group,],
                slices: factsOf({
                  englishes: [
                    { sliceIndex: 0, english: 'Dusk: the kitten purrs in the sun.', },
                    { sliceIndex: 1, english: '', },
                    { sliceIndex: 2, english: second, },
                  ],
                },),
              },),
            },),).toEqual([{ slice: 2, toMake: ['group 1 OCCURRENCE 2',], unprovenGains: [], leftOut: 0, },],);
          },
        },),
        it({
          name: 'LEAVES OUT A SPLICE WHOSE CARRIED APOSTROPHE A QUOTATION MARK OF ANOTHER SLICE COULD PAIR WITH, and keeps it '
            + 'on the same page with no such mark',
          fn: async () => {
            /**
             The splice that puts a trailing apostrophe in the second slice.
             */
            const splice = { start: 5, end: 15, text: 'the parents’', };
            /**
             The second slice.
             */
            const second = 'Then the family naps.';
            /**
             The page with an opener after the chosen place, and the same page with none.
             */
            const opened = 'Fill the parents’ bowl. She said ‘hello.';
            const plain = 'Fill the parents’ bowl. She said hello.';
            expect([opened, plain,].map(function planned(first,): unknown {
              return seenIn({
                plans: slicePlansOf({
                  settled: [splicedGroup({ first, second, splice, },),],
                  slices: factsOf({
                    englishes: [
                      { sliceIndex: 0, english: first, },
                      { sliceIndex: 2, english: second, },
                    ],
                  },),
                },),
              },);
            },),).toEqual([
              [{ slice: 2, toMake: [], unprovenGains: ['group 1 OCCURRENCE 2',], leftOut: 0, },],
              [{ slice: 2, toMake: ['group 1 OCCURRENCE 2',], unprovenGains: [], leftOut: 0, },],
            ],);
          },
        },),
        it({
          name: 'READS THE PAGE WITHOUT THE REPLACEMENTS OF A SLICE THE FLOORS REFUSED: the closing mark the refused splice took '
            + 'out stands again, and the splice that put `’round` before it is left out',
          fn: async () => {
            const { settled, slices, } = roundPage({ third: THIRD_WITH_CLOSER, },);
            expect([new Set<number>(), new Set([2,],),].map(function planned(refused,): unknown {
              return seenIn({ plans: slicePlansOf({ settled, slices, refused, },), },);
            },),).toEqual([
              [
                { slice: 1, toMake: ['group 1 OCCURRENCE 2',], unprovenGains: [], leftOut: 0, },
                { slice: 2, toMake: ['group 2 OCCURRENCE 2',], unprovenGains: [], leftOut: 0, },
              ],
              [
                { slice: 1, toMake: [], unprovenGains: ['group 1 OCCURRENCE 2',], leftOut: 0, },
                { slice: 2, toMake: [], unprovenGains: [], leftOut: 0, },
              ],
            ],);
          },
        },),
        it({
          name: 'READS THE TEXT NO SLICE HOLDS: a quotation mark in the text the page carries between two slices, a sealed span, '
            + 'is a partner for the apostrophe a splice carries in, and the same page with a blank line there is not',
          fn: async () => {
            /**
             The splice that puts a trailing apostrophe in the second slice.
             */
            const splice = { start: 5, end: 15, text: 'the parents’', };
            const first = 'Fill the parents’ bowl.';
            const second = 'Then the family naps.';
            /**
             What the page carries between its two slices that carry English, and after the last.
             */
            const sealed = ['', '\n\n<!-- note -->\n\nShe said ‘fill it.\n\n', '\n',];
            const blank = ['', '\n\n', '\n',];
            expect([sealed, blank,].map(function planned(between,): unknown {
              return seenIn({
                plans: slicePlansOf({
                  settled: [splicedGroup({ first, second, splice, },),],
                  slices: withPageBetween({
                    slices: factsOf({
                      englishes: [
                        { sliceIndex: 0, english: first, },
                        { sliceIndex: 1, english: '', },
                        { sliceIndex: 2, english: second, },
                      ],
                    },),
                    between,
                  },),
                },),
              },);
            },),).toEqual([
              [{ slice: 2, toMake: [], unprovenGains: ['group 1 OCCURRENCE 2',], leftOut: 0, },],
              [{ slice: 2, toMake: ['group 1 OCCURRENCE 2',], unprovenGains: [], leftOut: 0, },],
            ],);
          },
        },),
      ],
    },),
    describe({
      name: workedPageOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS THE TEXT BETWEEN THE SLICES OFF THE PAGE THE FACTS CARRY: before the first slice that carries English, '
            + 'between each two and after the last, none for a slice that carries none',
          fn: async () => {
            /**
             What the page carries around the two slices that carry English.
             */
            const between = ['<!-- note -->\n\n', '\n\n<!-- seal -->\n\n', '\n',];
            const { settled, } = roundPage({ third: THIRD_WITH_CLOSER, },);
            const slices = withPageBetween({
              slices: factsOf({
                englishes: [
                  { sliceIndex: 0, english: 'The folks ran ’round.', },
                  { sliceIndex: 1, english: '', },
                  { sliceIndex: 2, english: 'Fill the parents’ bowl now.', },
                ],
              },),
              between,
            },);
            expect(workedPageOf({ settled, slices, },).between,).toEqual(between,);
          },
        },),
        it({
          name: 'GIVES NO TEXT BETWEEN THE SLICES WHEN THE FACTS CARRY NO PAGE, or the page of only some of them, since the '
            + 'slices alone are then the page',
          fn: async () => {
            const { settled, slices, } = roundPage({ third: THIRD_WITH_CLOSER, },);
            const partial = slices.map(function pageForTheFirst(facts,): ConsistencySliceFacts {
              return (facts.sliceIndex === 0) ? { ...facts, page: { text: 'The folks ran ’round.', offset: 0, }, } : facts;
            },);
            expect([
              workedPageOf({ settled, slices, },).between,
              workedPageOf({ settled, slices: partial, },).between,
            ],).toEqual([undefined, undefined,],);
          },
        },),
        it({
          name: 'REFUSES FACTS WHOSE PAGE DOES NOT HOLD THE SLICES IN ORDER, naming the slice, since the text between them '
            + 'would be read off the wrong stretch of the page',
          fn: async () => {
            const { settled, slices, } = roundPage({ third: THIRD_WITH_CLOSER, },);
            /**
             The facts with the second slice placed before the first in the page.
             */
            const misplaced = slices.map(function placed(facts,): ConsistencySliceFacts {
              return {
                ...facts,
                page: { text: 'We went round the yard.\n\nThe folks ran ’round.\n\nFill the parents’ bowl now.', offset: (facts.sliceIndex === 0) ? 25 : 0, },
              };
            },);
            const refusal = caught(function readsMisplaced(): unknown {
              return workedPageOf({ settled, slices: misplaced, },);
            },);
            expect(String(refusal,),).toBe(
              'ConsistencyInvariantError: unreachable: slice 1 starts at 0 in the page, before 46, where the slice before it ends, '
                + 'so the slices do not follow each other in the page',
            );
          },
        },),
      ],
    },),
    describe({
      name: settledSlicesOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'DECIDES THE FLOORS AGAIN ON WHAT IS LEFT AFTER THE GAIN CHECK DROPS MORE, round after round until neither '
            + 'changes anything: a refusal puts a closing mark back, the splice that carried `’round` in goes, and the '
            + 'slice it shared with another splice is put through the floors with that splice alone',
          fn: async () => {
            /**
             Every slice the floors were asked about and the text they were asked about, in order.
             */
            const asked: (readonly [number, string])[] = [];
            const { settled, slices, } = roundPageWithTwoEdits();
            const result = settledSlicesOf({
              settled,
              slices,
              floorsOn: function scripted({ plan, candidateText, },): ConsistencyFloorVerdict {
                asked.push([plan.facts.sliceIndex, candidateText,],);
                return ((plan.facts.sliceIndex === 2) || (!candidateText.includes('’round',)))
                  ? { kind: 'refused', findings: ['scripted refusal',], }
                  : { kind: 'passes', };
              },
            },);
            expect(result.map(function seenOf({ plan, made, verdict, },): unknown {
              return {
                slice: plan.facts.sliceIndex,
                toMake: plan.toMake.map(function nameOf({ id, },): string {
                  return id;
                },),
                unprovenGains: plan.unprovenGains.map(function nameOf({ id, },): string {
                  return id;
                },),
                made: made?.text,
                verdict: verdict?.kind,
              };
            },),).toEqual([
              {
                slice: 1,
                toMake: ['group 3 OCCURRENCE 2',],
                unprovenGains: ['group 1 OCCURRENCE 2',],
                made: 'We went round the garden.',
                verdict: 'refused',
              },
              {
                slice: 2,
                toMake: ['group 2 OCCURRENCE 2',],
                unprovenGains: [],
                made: 'Fill the folks bowl now.',
                verdict: 'refused',
              },
            ],);
            expect(asked,).toEqual([
              [1, 'We went ’round the garden.',],
              [2, 'Fill the folks bowl now.',],
              [1, 'We went round the garden.',],
            ],);
          },
        },),
        it({
          name: 'ASKS THE FLOORS ONCE, and keeps every replacement, when no floor refuses a slice',
          fn: async () => {
            /**
             Every slice the floors were asked about, in order.
             */
            const asked: number[] = [];
            const { settled, slices, } = roundPage({ third: THIRD_WITH_CLOSER, },);
            const result = settledSlicesOf({
              settled,
              slices,
              floorsOn: function passing({ plan, },): ConsistencyFloorVerdict {
                asked.push(plan.facts.sliceIndex,);
                return { kind: 'passes', };
              },
            },);
            expect(asked,).toEqual([1, 2,],);
            expect(result.map(function kindOf({ verdict, },): string {
              return verdict?.kind ?? 'not asked';
            },),).toEqual(['passes', 'passes',],);
          },
        },),
      ],
    },),
  ],
},);
