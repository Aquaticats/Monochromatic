/**
 Tests for the callers of the consistency planner that hold the English around a splice: a reference whose definition stands in
 another slice of the page is a link there, so words that make it an undefined reference change the pairing of the emphasis
 around it, and the marks outside the words read another way (n173). The edge refusal and the decision of a place both read the
 English with the page's definitions.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ConsistencyCount, } from '@monochromatic-dev/translation-repair-consistency';
import {
  chosenPlaceOf,
  type ConsistencyPlace,
  edgeRefusalOf,
  plannedPlace,
} from '../../dist/final/node/index.mjs';
import { renderingsOf, } from '@monochromatic-dev/translation-repair-consistency/ts/consistency-renderings.test-fixture.ts';

/**
 The receiving place's English: a reference the page defines inside an emphasis that an apostrophe follows.
 */
const ENGLISH = '*a [cat*] d*\'s bowl';

/**
 The definition of the reference, which stands in another slice of the page.
 */
const DEFINITION = '[cat*]: https://cats.example/c';

/**
 Two places that each hold the English whole, the second with the page the case gives.

 @param page - English of the whole page, none for a slice read alone

 @returns Two places

 @example
 ```ts
 const places = placesOn({ page: `${ENGLISH}\n\n${DEFINITION}`, },);
 ```
 */
function placesOn({ page, }: { readonly page?: string; }): readonly ConsistencyPlace[] {
  return [0, 1,].map(function placeAt(position,): ConsistencyPlace {
    return {
      startOffset: position * 10,
      endOffset: (position * 10) + 3,
      line: position + 1,
      slice: {
        sliceIndex: position,
        sourceIndex: position,
        sourceText: '小猫打盹',
        sourceStart: position * 10,
        nodes: [],
        current: { kind: 'present', text: (position === 0) ? 'Fill the dog' : ENGLISH, },
        lineStructured: false,
        disputedWordings: [],
        ...(((position === 1) && (page !== undefined)) ? { page: { text: page, offset: 0, }, } : {}),
      },
    };
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: edgeRefusalOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS THE ENGLISH WITH THE PAGE\'S DEFINITIONS: words that make a defined reference an undefined one change the '
            + 'reading of the apostrophe after the emphasis, which the English alone cannot show',
          fn: async () => {
            const [chosen, target,] = renderingsOf({
              chosen: { english: 'Fill the dog', quote: 'dog', },
              receiving: { english: ENGLISH, quote: 'cat', },
            },);
            expect([
              edgeRefusalOf({ chosen, target, framed: false, english: ENGLISH, },),
              edgeRefusalOf({ chosen, target, framed: false, english: ENGLISH, definitions: DEFINITION, },),
            ],).toEqual(['unrefused', 'edge-reading-changes',],);
          },
        },),
      ],
    },),
    describe({
      name: plannedPlace.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'HANDS THE PAGE\'S DEFINITIONS TO THE PLANNER: a place whose slice stands in a page that defines the reference is left as '
            + 'it was and the refusal is named, and a slice read alone or a page that defines nothing of the kind is replaced',
          fn: async () => {
            const [chosenRendering, rendering,] = renderingsOf({
              chosen: { english: 'Fill the dog', quote: 'dog', },
              receiving: { english: ENGLISH, quote: 'cat', },
            },);
            /**
             The count of the group: the chosen words and the receiving words.
             */
            const count: ConsistencyCount = {
              usable: 3,
              renderings: [chosenRendering, rendering,],
              kept: [false, false,],
              winner: { kind: 'chosen', label: 'OCCURRENCE 1', },
              finalists: [],
            };
            /**
             What the second place comes to for the page the case gives.
             */
            function plannedOn({ page, }: { readonly page?: string; }): unknown {
              const places = (page === undefined) ? placesOn({}) : placesOn({ page, });
              const { outcome, receiver, edge, } = plannedPlace({
                chosen: chosenPlaceOf({ places, renderings: [chosenRendering, rendering,], position: 0, },),
                place: nonNullishOrThrow(places[1],),
                position: 1,
                count,
                framed: false,
              },);
              return [outcome, receiver.map(function kindOf({ plan, },): string {
                return plan.kind;
              },), edge,];
            }
            expect([
              plannedOn({},),
              plannedOn({ page: ENGLISH, },),
              plannedOn({ page: `${ENGLISH}\n\n${DEFINITION}`, },),
            ],).toEqual([
              ['kept-current', ['replace',], undefined,],
              ['kept-current', ['replace',], undefined,],
              ['kept-current', ['unavailable',], 'edge-reading-changes',],
            ],);
          },
        },),
      ],
    },),
  ],
},);
