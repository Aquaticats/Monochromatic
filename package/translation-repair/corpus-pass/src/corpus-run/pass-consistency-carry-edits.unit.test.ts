/**
 Tests for what a winning candidate does to the page: the replacements it makes (none for the current paragraph or no winner,
 the words of each receiving place that has a plan for the splice, the block once for every receiving place for a writer's
 paragraph) and the finding for a block the splice carried in part.

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

import {
  type CarryOffer,
  carryOffersOf,
} from '@monochromatic-dev/translation-repair-consistency';
import {
  type CarryUnit,
  editsOf,
  partlyCarriedFindingOf,
} from '../../dist/final/node/index.mjs';
import {
  carryScene,
  RECEIVING_ENGLISH,
  secondReceiverOf,
  SPLICED_PARAGRAPH,
} from './consistency-carry.test-fixture.ts';
import { choosing, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/consistency-page.test-fixture.ts';

/**
 What a writer wrote.
 */
const WRITTEN = 'Later the cat dozes in the sun.';

/**
 The three candidates: the current paragraph, the splice and one writer's paragraph.
 */
const OFFERS = carryOffersOf({
  current: RECEIVING_ENGLISH,
  splice: [SPLICED_PARAGRAPH,],
  written: [{ seat: 'hyper-cat', text: WRITTEN, },],
  lineStructured: false,
},);

/**
 The candidate of one kind, as a list of one.

 @param kind - which candidate

 @returns Candidate as the winner is passed

 @throws {@link Error} when the candidates hold none of the kind

 @example
 ```ts
 const winner = winnerOfKind({ kind: 'splice', },);
 ```
 */
function winnerOfKind({ kind, }: { readonly kind: CarryOffer['kind']; },): readonly CarryOffer[] {
  /**
   The candidate of the kind.
   */
  const offer = OFFERS.find(function isOfKind(candidate,): boolean {
    return candidate.kind === kind;
  },);
  if (offer === undefined)
    throw new Error(`the fixture's candidates hold no ${kind}`,);
  return [offer,];
}

/**
 The scene's block with two receiving places, the second given the plan the case names.

 @param secondPlan - how the chosen words reach the second place

 @returns Unit

 @example
 ```ts
 const unit = twoPlaceUnit({ secondPlan: { kind: 'unavailable', }, },);
 ```
 */
function twoPlaceUnit({ secondPlan, }: { readonly secondPlan?: CarryUnit['receivers'][number]['plan']; } = {},): CarryUnit {
  /**
   The scene, whose bench is never asked here.
   */
  const scene = carryScene({
    behave: function neverAsked() {
      return choosing({ choice: 'CANDIDATE 1', },);
    },
  },);
  /**
   The second receiving place as the fixture makes it.
   */
  const second = secondReceiverOf({ receiver: scene.receiver, },);
  return {
    ...scene.unit,
    receivers: [scene.receiver, (secondPlan === undefined) ? second : { ...second, plan: secondPlan, },],
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: editsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'MAKES NO REPLACEMENT WHEN NOBODY WON OR THE CURRENT PARAGRAPH WON',
          fn: async () => {
            const unit = twoPlaceUnit();
            expect({
              nobody: editsOf({ unit, winner: [], },),
              current: editsOf({ unit, winner: winnerOfKind({ kind: 'current', },), },),
            },).toEqual({
              nobody: [],
              current: [],
            },);
          },
        },),
        it({
          name: 'REPLACES THE WORDS OF EACH RECEIVING PLACE WITH A PLAN WHEN THE SPLICE WON, and leaves a place without one',
          fn: async () => {
            expect({
              both: editsOf({ unit: twoPlaceUnit(), winner: winnerOfKind({ kind: 'splice', },), },),
              first: editsOf({
                unit: twoPlaceUnit({ secondPlan: { kind: 'unavailable', }, },),
                winner: winnerOfKind({ kind: 'splice', },),
              },),
            },).toEqual({
              both: [
                { kind: 'splice', placePositions: [1,], sliceIndex: 1, start: 6, end: 21, text: 'the cat naps', },
                { kind: 'splice', placePositions: [2,], sliceIndex: 1, start: 25, end: 32, text: 'the warm sun', },
              ],
              first: [
                { kind: 'splice', placePositions: [1,], sliceIndex: 1, start: 6, end: 21, text: 'the cat naps', },
              ],
            },);
          },
        },),
        it({
          name: 'THROWS WHEN THE SPLICE WON FOR A BLOCK NONE OF WHOSE RECEIVING PLACES CODE CAN MOVE THE WORDS INTO, since no '
            + 'splice is offered then',
          fn: async () => {
            const unit = twoPlaceUnit({ secondPlan: { kind: 'unavailable', }, },);
            /**
             The block with neither place carried by code.
             */
            const unplanned: CarryUnit = {
              ...unit,
              receivers: unit.receivers.map(function unavailable(receiver,) {
                return { ...receiver, plan: { kind: 'unavailable', }, } as const;
              },),
            };
            /**
             What the call threw.
             */
            const refusal = caught(function editsAnyway() {
              return editsOf({ unit: unplanned, winner: winnerOfKind({ kind: 'splice', },), },);
            },);
            expect(String(refusal,),).toBe(
              'ConsistencyInvariantError: unreachable: the splice won for OCCURRENCE 2 and OCCURRENCE 3, none of whose receiving '
                + 'places has words that code can move, and a splice is offered only when some receiving place of the block can '
                + 'be spliced',
            );
          },
        },),
        it({
          name: 'REPLACES THE BLOCK ONCE FOR EVERY RECEIVING PLACE WHEN A WRITER\'S PARAGRAPH WON',
          fn: async () => {
            expect(editsOf({ unit: twoPlaceUnit(), winner: winnerOfKind({ kind: 'writer', },), },),).toEqual([
              {
                kind: 'block',
                placePositions: [1, 2,],
                sliceIndex: 1,
                start: 0,
                end: RECEIVING_ENGLISH.length,
                text: WRITTEN,
              },
            ],);
          },
        },),
      ],
    },),
    describe({
      name: partlyCarriedFindingOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REPORTS NOTHING UNLESS THE SPLICE WON AND SOME RECEIVING PLACE KEPT ITS WORDS',
          fn: async () => {
            const unit = twoPlaceUnit({ secondPlan: { kind: 'unavailable', }, },);
            expect({
              nobody: partlyCarriedFindingOf({ unit, winner: [], },),
              writer: partlyCarriedFindingOf({ unit, winner: winnerOfKind({ kind: 'writer', },), },),
              current: partlyCarriedFindingOf({ unit, winner: winnerOfKind({ kind: 'current', },), },),
              allCarried: partlyCarriedFindingOf({ unit: twoPlaceUnit(), winner: winnerOfKind({ kind: 'splice', },), },),
            },).toEqual({
              nobody: [],
              writer: [],
              current: [],
              allCarried: [],
            },);
          },
        },),
        it({
          name: 'NAMES THE PLACES CARRIED AND THE PLACES THAT KEPT THEIR WORDS WHEN THE SPLICE CARRIED THE BLOCK IN PART',
          fn: async () => {
            expect(partlyCarriedFindingOf({
              unit: twoPlaceUnit({ secondPlan: { kind: 'unavailable', }, },),
              winner: winnerOfKind({ kind: 'splice', },),
            },),).toEqual([
              'consistency-partly-carried (OCCURRENCE 2 and OCCURRENCE 3): the splice won, so code moved the chosen words '
                + 'into OCCURRENCE 2 and left OCCURRENCE 3 as it was, since code cannot move them there',
            ],);
          },
        },),
      ],
    },),
  ],
},);
