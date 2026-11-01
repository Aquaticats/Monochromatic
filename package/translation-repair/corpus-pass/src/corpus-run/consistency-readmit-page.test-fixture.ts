/**
 A page of two slices and the round that dropped some of the replacements of the second, for the tests of what puts a
 dropped splice back: the first slice holds the chosen words, the second the words the splices replace.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  type ConsistencyFloorVerdict,
  plansOnPage,
  type SettledGroup,
  type SettledSlice,
  workedPageOf,
} from '../../dist/final/node/index.mjs';
import {
  groupIn,
  withEdit,
} from './consistency-apply-group.test-fixture.ts';
import { factsOf, } from './consistency-round-page.test-fixture.ts';

/**
 The first slice of a page for the cases that read a round directly: a chosen sentence with two plural possessives.
 */
const CHOSEN = 'Fill the parents’ dish and the pups’ bed.';

/**
 One replacement of the second slice of the page.
 */
export type SecondSliceSplice = {
  /**
   The words the replacement takes out, as the judges quote them.
   */
  readonly quote: string;

  /**
   The chosen words of the first slice the group stands on.
   */
  readonly chosen: string;

  /**
   Where the replaced words start in the second slice.
   */
  readonly start: number;

  /**
   Where they end.
   */
  readonly end: number;

  /**
   The words that go in.
   */
  readonly text: string;
};

/**
 A page of two slices and the round that dropped every replacement of the second slice that is named, for a case to put
 back.

 @param second - the English of the second slice

 @param edits - the replacements of the second slice, each a group of its own in the order given

 @param dropped - the names of the replacements the round dropped, the rest being made

 @returns The page and the round

 @example
 ```ts
 const input = droppedRound({ second: 'Fill the family bowl.', edits, dropped: [], },);
 ```
 */
export function droppedRound(
  {
    second,
    edits,
    dropped,
  }: {
    readonly second: string;
    readonly edits: readonly SecondSliceSplice[];
    readonly dropped: readonly string[];
  },
): {
  readonly page: ReturnType<typeof workedPageOf>;
  readonly round: readonly SettledSlice[];
} {
  /**
   The groups: each has the chosen words in the first slice and its receiving place in the second, with one splice over it.
   */
  const settled: readonly SettledGroup[] = edits.map(function groupOf({
    quote,
    chosen,
    start,
    end,
    text,
  },): SettledGroup {
    return withEdit({
      group: groupIn({
        places: [
          {
            slice: 0,
            english: CHOSEN,
            quote: chosen,
          },
          {
            slice: 1,
            english: second,
            quote,
          },
        ],
      },),
      edit: {
        kind: 'splice',
        placePositions: [1,],
        sliceIndex: 1,
        start,
        end,
        text,
      },
    },);
  },);
  /**
   The facts of the two slices.
   */
  const slices = factsOf({
    englishes: [
      {
        sliceIndex: 0,
        english: CHOSEN,
      },
      {
        sliceIndex: 1,
        english: second,
      },
    ],
  },);
  /**
   The slices that carry English, with their replacements.
   */
  const page = workedPageOf({
    settled,
    slices,
  },);
  /**
   The plans of the slices that have replacements.
   */
  const plans = plansOnPage({
    page,
    refused: new Set(),
  },);
  return {
    page,
    round: plans.map(function roundOf(plan,): SettledSlice {
      /**
       The names of the replacements of the slice.
       */
      const { edits: planned, } = plan;
      return {
        plan: {
          ...plan,
          toMake: planned.filter(function isMade({ id, },): boolean {
            return !dropped.includes(id,);
          },),
          unprovenGains: planned.filter(function isDropped({ id, },): boolean {
            return dropped.includes(id,);
          },),
        },
        gainReasons: new Map(),
      };
    },),
  };
}

/**
 Floors that pass every slice.

 @returns The verdict

 @example
 ```ts
 const verdict = floorsPass();
 ```
 */
export function floorsPass(): ConsistencyFloorVerdict {
  return { kind: 'passes', };
}
