/**
 Tests for who won a group and what each place is to receive: the winner after
 a run-off, the winning place with its located rendering, and each place's
 provisional outcome with the receiver it makes.

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

import {
  type ConsistencyCount,
  ConsistencyInvariantError,
  type ConsistencyRendering,
} from '@monochromatic-dev/translation-repair-consistency';
import {
  afterRunOff,
  chosenPlaceOf,
  type ConsistencyPlace,
  plannedPlace,
} from '../../dist/final/node/index.mjs';
import { runOffOf, } from './consistency-settle.test-fixture.ts';

/**
 A place a slice holds whole.

 @param position - position of the place in its group, which is also its slice

 @returns Place

 @example
 ```ts
 const place = placeIn({ position: 0, },);
 ```
 */
function placeIn({ position, }: { readonly position: number; },): ConsistencyPlace {
  return {
    startOffset: position * 10,
    endOffset: (position * 10) + 4,
    line: position + 1,
    slice: {
      sliceIndex: position,
      sourceIndex: position,
      sourceText: '小猫打盹',
      sourceStart: position * 10,
      nodes: [],
      current: { kind: 'present', text: 'The cat naps.', },
      lineStructured: false,
      disputedWordings: [],
    },
  };
}

/**
 Three places.
 */
const PLACES: readonly ConsistencyPlace[] = [placeIn({ position: 0, },), placeIn({ position: 1, },), placeIn({ position: 2, },),];

/**
 Labels of the three places.
 */
const LABELS = ['OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 3',] as const;

/**
 The rendering of the first place.
 */
const FIRST: ConsistencyRendering = { kind: 'located', quote: 'The cat naps', core: 'The cat naps', start: 0, };

/**
 A rendering the second place has, that differs from the first.
 */
const DIFFERENT: ConsistencyRendering = { kind: 'located', quote: 'the kitten naps', core: 'the kitten naps', start: 6, };

/**
 What the first round came to, with the facts a case replaces.

 @param over - fields this case replaces

 @returns Count

 @example
 ```ts
 const count = countOf({ over: { finalists: ['OCCURRENCE 1', 'OCCURRENCE 2',], }, },);
 ```
 */
function countOf(
  { over = {}, }: {
    readonly over?: {
      readonly usable?: number;
      readonly renderings?: ConsistencyCount['renderings'];
      readonly kept?: ConsistencyCount['kept'];
      readonly winner?: ConsistencyCount['winner'];
      readonly finalists?: ConsistencyCount['finalists'];
    };
  } = {},
): ConsistencyCount {
  return {
    usable: 3,
    renderings: [FIRST, DIFFERENT, FIRST,],
    kept: [false, false, false,],
    winner: { kind: 'chosen', label: 'OCCURRENCE 1', },
    finalists: [],
    ...over,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: afterRunOff.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS THE FIRST ROUND\'S WINNER AND RECORDS NO RUN-OFF when the choices did not tie',
          fn: async () => {
            expect(afterRunOff({ count: countOf(), labels: LABELS, runOff: [], },),).toEqual({
              winner: { kind: 'chosen', label: 'OCCURRENCE 1', },
              runOff: [],
            },);
          },
        },),
        it({
          name: 'TAKES THE WINNER OF THE RUN-OFF AMONG THE FINALISTS and records it with the finalists, the ballots and the counts',
          fn: async () => {
            const count = countOf({
              over: {
                winner: { kind: 'none', reason: 'no-majority', },
                finalists: ['OCCURRENCE 1', 'OCCURRENCE 2',],
              },
            },);
            expect(afterRunOff({
              count,
              labels: LABELS,
              runOff: [runOffOf({ choices: ['OCCURRENCE 2', 'OCCURRENCE 2', 'OCCURRENCE 1',], },),],
            },),).toEqual({
              winner: { kind: 'chosen', label: 'OCCURRENCE 2', },
              runOff: [
                {
                  finalists: ['OCCURRENCE 1', 'OCCURRENCE 2',],
                  ballots: [
                    { seat: 'seat-0', reason: 'It renders the original best.', choice: 'OCCURRENCE 2', },
                    { seat: 'seat-1', reason: 'It renders the original best.', choice: 'OCCURRENCE 2', },
                    { seat: 'seat-2', reason: 'It renders the original best.', choice: 'OCCURRENCE 1', },
                  ],
                  usable: 3,
                  quorum: 2,
                },
              ],
            },);
          },
        },),
        it({
          name: 'REFUSES A TIE THAT WAS NOT RUN OFF, and a run-off for choices that did not tie',
          fn: async () => {
            const tied = countOf({ over: { finalists: ['OCCURRENCE 1', 'OCCURRENCE 2',], }, },);
            const unrun = caught(function settleTie(): unknown {
              return afterRunOff({ count: tied, labels: LABELS, runOff: [], },);
            },);
            const needless = caught(function settleMajority(): unknown {
              return afterRunOff({
                count: countOf(),
                labels: LABELS,
                runOff: [runOffOf({ choices: ['OCCURRENCE 2',], },),],
              },);
            },);
            expect([unrun, needless,].map(function isInvariant(refusal,): boolean {
              return refusal instanceof ConsistencyInvariantError;
            },),).toEqual([true, true,],);
          },
        },),
      ],
    },),
    describe({
      name: chosenPlaceOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE WINNING PLACE WITH ITS LOCATED RENDERING, and none when no place won',
          fn: async () => {
            expect([
              chosenPlaceOf({ places: PLACES, renderings: [FIRST, DIFFERENT, FIRST,], position: 1, },),
              chosenPlaceOf({ places: PLACES, renderings: [FIRST, DIFFERENT, FIRST,], position: -1, },),
            ],).toEqual([[{ place: PLACES[1], rendering: DIFFERENT, },], [],],);
          },
        },),
        it({
          name: 'REFUSES A WINNER WHOSE RENDERING IS NOT LOCATED OR WHOSE PLACE IS NOT IN THE GROUP',
          fn: async () => {
            const unlocated = caught(function chooseUnlocated(): unknown {
              return chosenPlaceOf({
                places: PLACES,
                renderings: [FIRST, { kind: 'unlocated', reason: 'not-found', }, FIRST,],
                position: 1,
              },);
            },);
            const absent = caught(function chooseAbsent(): unknown {
              return chosenPlaceOf({ places: PLACES, renderings: [FIRST, DIFFERENT, FIRST,], position: 7, },);
            },);
            expect([unlocated, absent,].map(String,),).toEqual([
              'ConsistencyInvariantError: unreachable: a winner is named only where its own rendering is located',
              'ConsistencyInvariantError: unreachable: a winner is named only where its own rendering is located',
            ],);
          },
        },),
      ],
    },),
    describe({
      name: plannedPlace.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'MAKES A PLACE THAT RENDERS THE WORDING OTHERWISE A RECEIVER with the plan to move the chosen words in, its '
            + 'provisional outcome being that the page keeps its English',
          fn: async () => {
            const chosen = chosenPlaceOf({ places: PLACES, renderings: [FIRST, DIFFERENT, FIRST,], position: 0, },);
            expect(plannedPlace({
              chosen,
              place: nonNullishOrThrow(PLACES[1],),
              position: 1,
              count: countOf(),
              framed: false,
            },),)
              .toEqual({
                outcome: 'kept-current',
                receiver: [
                  {
                    place: PLACES[1],
                    rendering: DIFFERENT,
                    position: 1,
                    label: 'OCCURRENCE 2',
                    plan: { kind: 'replace', start: 6, end: 21, text: 'the cat naps', },
                  },
                ],
              },);
          },
        },),
        it({
          name: 'PLANS THE SPLICE WITH THE MARKS BESIDE THE WORDS COMPARED: a receiver whose words match the chosen words but '
            + 'whose marks beside them differ is no consistent place, and its plan is unavailable to code',
          fn: async () => {
            const marked: ConsistencyRendering = {
              kind: 'located',
              quote: 'The cat naps',
              core: 'The cat naps',
              start: 0,
              markAfter: '\'',
            };
            const chosen = chosenPlaceOf({ places: PLACES, renderings: [marked, DIFFERENT, FIRST,], position: 0, },);
            const planned = [
              plannedPlace({
                chosen,
                place: nonNullishOrThrow(PLACES[2],),
                position: 2,
                count: countOf({ over: { renderings: [marked, DIFFERENT, FIRST,], }, },),
                framed: false,
              },),
              plannedPlace({
                chosen,
                place: nonNullishOrThrow(PLACES[1],),
                position: 1,
                count: countOf({ over: { renderings: [marked, DIFFERENT, FIRST,], }, },),
                framed: false,
              },),
            ];
            expect(planned.map(function planOf({ outcome, receiver, },): unknown {
              return [outcome, receiver.map(function kindOf({ plan, },): string {
                return plan.kind;
              },),];
            },),).toEqual([
              ['kept-current', ['unavailable',],],
              ['kept-current', ['unavailable',],],
            ],);
          },
        },),
        it({
          name: 'NAMES WHY THE SPLICE WAS REFUSED AT THE EDGE OF THE SPAN, where the recorded words and marks alone would let '
            + 'code move the chosen words in: a second mark past markup, a mark outside the words that would read differently',
          fn: async () => {
            /**
             The chosen words, which bring an apostrophe to their end.
             */
            const dogs: ConsistencyRendering = { kind: 'located', quote: 'dogs’', core: 'dogs’', start: 9, };
            /**
             A place whose English is the one given.
             */
            function placeWith({ text, }: { readonly text: string; },): ConsistencyPlace {
              const place = nonNullishOrThrow(PLACES[1],);
              return { ...place, slice: { ...place.slice, current: { kind: 'present', text, }, }, };
            }
            const chosen = chosenPlaceOf({ places: PLACES, renderings: [dogs, DIFFERENT, FIRST,], position: 0, },);
            /**
             What the second place comes to when its rendering and English are the ones given.
             */
            function plannedWith(
              { rendering, text, }: { readonly rendering: ConsistencyRendering; readonly text: string; },
            ): ReturnType<typeof plannedPlace> {
              return plannedPlace({
                chosen,
                place: placeWith({ text, },),
                position: 1,
                count: countOf({ over: { renderings: [dogs, rendering, FIRST,], }, },),
                framed: false,
              },);
            }
            expect([
              plannedWith({
                rendering: { kind: 'located', quote: 'kittens', core: 'kittens', start: 10, markPastAfter: '’', },
                text: 'Fill the *kittens*’ bowl.',
              },),
              plannedWith({
                rendering: { kind: 'located', quote: 'kittens', core: 'kittens', start: 10, },
                text: 'Fill the *kittens* bowl.',
              },),
            ].map(function edgeOf({ outcome, receiver, edge, },): unknown {
              return [outcome, receiver.map(function kindOf({ plan, },): string {
                return plan.kind;
              },), edge,];
            },),).toEqual([
              ['kept-current', ['unavailable',], 'edge-second-mark',],
              ['kept-current', ['replace',], undefined,],
            ],);
          },
        },),
        it({
          name: 'REFUSES EVERY SPLICE ON A PAGE WHOSE ENGLISH HOLDS A QUOTATION-LIKE REFERENCE IN ANY SLICE, the receiving slice read '
            + 'alone included and a slice read alone elsewhere on the page too, and names the reference',
          fn: async () => {
            /**
             The chosen words, which bring no apostrophe.
             */
            const chosen = chosenPlaceOf({ places: PLACES, renderings: [FIRST, DIFFERENT, FIRST,], position: 0, },);
            /**
             The second place with the facts of its slice the case gives: the English it carries, the references of the English of
             every slice of the page, and the page it stands in, none for a slice read alone.
             */
            function placeWith(
              { references, page, }: { readonly references?: readonly string[]; readonly page?: string; },
            ): ConsistencyPlace {
              const place = nonNullishOrThrow(PLACES[1],);
              return {
                ...place,
                slice: {
                  ...place.slice,
                  current: { kind: 'present', text: 'The cat naps.', },
                  ...((references === undefined) ? {} : { wholePageReferences: references, }),
                  ...((page === undefined) ? {} : { page: { text: page, offset: 0, }, }),
                },
              };
            }
            /**
             What the second place comes to for the facts given.
             */
            function plannedFor(
              { references, page, }: { readonly references?: readonly string[]; readonly page?: string; },
            ): unknown {
              const { outcome, receiver, edge, } = plannedPlace({
                chosen,
                place: placeWith({
                  ...((references === undefined) ? {} : { references, }),
                  ...((page === undefined) ? {} : { page, }),
                },),
                position: 1,
                count: countOf(),
                framed: false,
              },);
              return [outcome, receiver.map(function kindOf({ plan, },): string {
                return plan.kind;
              },), edge,];
            }
            expect([
              plannedFor({ references: ['&#39;',], },),
              plannedFor({ references: ['&#39;',], page: 'The cat naps.', },),
              plannedFor({ references: [], },),
              plannedFor({ references: [], page: 'The cat naps.', },),
              plannedFor({},),
            ],).toEqual([
              ['kept-current', ['unavailable',], 'edge-reference-mark',],
              ['kept-current', ['unavailable',], 'edge-reference-mark',],
              ['kept-current', ['replace',], undefined,],
              ['kept-current', ['replace',], undefined,],
              ['kept-current', ['replace',], undefined,],
            ],);
          },
        },),
        it({
          name: 'NAMES THE OUTCOME OF A PLACE THAT NEEDS NO CARRY: chosen, already consistent, kept by a majority, unlocated, '
            + 'and untouched when no place won',
          fn: async () => {
            const chosen = chosenPlaceOf({ places: PLACES, renderings: [FIRST, DIFFERENT, FIRST,], position: 0, },);
            /**
             The outcome of a place under a count.
             */
            function outcomeOf({ position, count, winners = chosen, }: {
              readonly position: number;
              readonly count: ConsistencyCount;
              readonly winners?: typeof chosen;
            },): unknown {
              return plannedPlace({
                chosen: winners,
                place: nonNullishOrThrow(PLACES[position],),
                position,
                count,
                framed: false,
              },);
            }
            expect([
              outcomeOf({ position: 0, count: countOf(), },),
              outcomeOf({ position: 2, count: countOf(), },),
              outcomeOf({ position: 1, count: countOf({ over: { kept: [false, true, false,], }, },), },),
              outcomeOf({
                position: 1,
                count: countOf({ over: { renderings: [FIRST, { kind: 'unlocated', reason: 'not-found', }, FIRST,], }, },),
              },),
              outcomeOf({ position: 1, count: countOf(), winners: [], },),
            ],).toEqual([
              { outcome: 'chosen', receiver: [], },
              { outcome: 'consistent', receiver: [], },
              { outcome: 'kept-context', receiver: [], },
              { outcome: 'unlocated', receiver: [], },
              { outcome: 'untouched', receiver: [], },
            ],);
          },
        },),
      ],
    },),
  ],
},);
