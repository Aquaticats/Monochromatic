/**
 Tests for recounting a recorded group against its own ballots: the usable
 count, each place's located quote and slot or the reason it was not located,
 the winner, and what became of each place; and for tying the changed slices to
 the places that were replaced.

 EVERY CASE BREAKS ONE VALID GROUP IN ONE WAY. The valid groups are a repeated
 sentence in three places (the first chosen, the second replaced, the third
 already carrying the chosen words) and a refrain in two, so what fails is a
 relation the ballots decide, and the whole refusal is pinned.

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
  type ArtifactConsistencyBallot,
  type ArtifactConsistencyGroup,
  type ArtifactConsistencySlice,
  assertAskedGroupCoherent,
  assertSlicesCoherent,
} from '../../dist/final/node/index.mjs';
import {
  askedOf,
  firstPlaceBallot,
  frameGroup,
  napsGroup,
  withPlace,
} from './consistency-group.test-fixture.ts';
import { refusalTextOf, } from './consistency-refusal.test-fixture.ts';

/**
 The path of the group every refusal here is built against.
 */
const GROUP = 'cat.groups[0]';

/**
 The refusal text of the winner recount.
 */
const WINNER_REFUSAL = `ArtifactParseError: artifact parse failed at ${GROUP}.verdict.winner: `
  + 'expected the winner a strict majority of the usable ballots, reaching the quorum, give.';

/**
 The refusal text of a replacement a place's outcome does not name.
 */
const NOT_REPLACING = `ArtifactParseError: artifact parse failed at ${GROUP}.occurrences[1].outcome: `
  + 'expected replaced, kept-floor or overlapped, for a place the chosen words would replace.';

/**
 The refusal text of a place that carries the chosen words and is not recorded consistent, or the reverse.

 @param place - position of the place in the group

 @returns Refusal as text

 @example
 ```ts
 const text = notConsistent({ place: 2, },);
 ```
 */
function notConsistent({ place, }: { readonly place: number; },): string {
  return `ArtifactParseError: artifact parse failed at ${GROUP}.occurrences[${String(place,)}].outcome: `
    + 'expected consistent for a place that already carries the chosen words, and for no other.';
}

/**
 The recount of a group.

 @param group - group, asked of the bench

 @returns What the recount makes of a group it accepts

 @example
 ```ts
 const nothing = coherent({ group: napsGroup(), },);
 ```
 */
function coherent({ group, }: { readonly group: ArtifactConsistencyGroup; },): unknown {
  return assertAskedGroupCoherent({
    group: askedOf({ group, },),
    path: GROUP,
    generation: 15,
  },);
}

/**
 The text of the refusal the recount raises for a group.

 @param group - group, asked of the bench

 @returns Refusal as text

 @example
 ```ts
 const text = coherenceRefusal({ group: napsGroup(), },);
 ```
 */
function coherenceRefusal({ group, }: { readonly group: ArtifactConsistencyGroup; },): string {
  return refusalTextOf({
    read: function recounts(): unknown {
      return coherent({ group, },);
    },
  },);
}

/**
 A group whose verdict takes other values.

 @param group - group, asked of the bench

 @param over - fields of the asked verdict that take other values

 @returns Group with that verdict

 @example
 ```ts
 const short = withVerdict({ group: napsGroup(), over: { usable: 2, }, },);
 ```
 */
function withVerdict(
  {
    group,
    over,
  }: {
    readonly group: ArtifactConsistencyGroup;
    readonly over: {
      readonly ballots?: readonly ArtifactConsistencyBallot[];
      readonly usable?: number;
      readonly quorum?: number;
      readonly winner?: ReturnType<typeof askedOf>['verdict']['winner'];
    };
  },
): ArtifactConsistencyGroup {
  /**
   The group as asked, whose verdict the fields replace.
   */
  const asked = askedOf({ group, },);
  return {
    ...asked,
    verdict: {
      ...asked.verdict,
      ...over,
    },
  };
}

/**
 A group heard from other ballots, with the usable count they come to.

 @param group - group, asked of the bench

 @param ballots - ballots heard

 @param usable - how many of them can be counted

 @returns Group with those ballots

 @example
 ```ts
 const heard = withBallots({ group: napsGroup(), ballots, usable: 3, },);
 ```
 */
function withBallots(
  {
    group,
    ballots,
    usable,
  }: {
    readonly group: ArtifactConsistencyGroup;
    readonly ballots: readonly ArtifactConsistencyBallot[];
    readonly usable: number;
  },
): ArtifactConsistencyGroup {
  return withVerdict({
    group,
    over: {
      ballots,
      usable,
    },
  },);
}

/**
 The ballots of a group, each changed alike.

 @param group - group, asked of the bench

 @param change - how each ballot changes

 @returns Changed ballots

 @example
 ```ts
 const keeping = ballotsOf({ group, change: function keeps(ballot) { return { ...ballot, keep: ['OCCURRENCE 2',], }; }, },);
 ```
 */
function ballotsOf(
  {
    group,
    change,
  }: {
    readonly group: ArtifactConsistencyGroup;
    readonly change: (ballot: ArtifactConsistencyBallot,) => ArtifactConsistencyBallot;
  },
): readonly ArtifactConsistencyBallot[] {
  return askedOf({ group, },)
    .verdict
    .ballots
    .map(change,);
}

/**
 The refusal text of the recount of the changed slices.

 @param groups - every recorded group

 @param slices - recorded changed slices

 @returns Refusal as text

 @example
 ```ts
 const text = slicesRefusal({ groups: [napsGroup(),], slices: [], },);
 ```
 */
function slicesRefusal(
  {
    groups,
    slices,
  }: {
    readonly groups: readonly ArtifactConsistencyGroup[];
    readonly slices: readonly ArtifactConsistencySlice[];
  },
): string {
  return refusalTextOf({
    read: function recountsSlices(): unknown {
      return assertSlicesCoherent({
        groups,
        slices,
        path: 'cat.slices',
        generation: 17,
      },);
    },
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: assertAskedGroupCoherent.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS A REPEATED SENTENCE AND A REFRAIN THE BALLOTS GIVE, a place replaced and a place already consistent',
          fn: async () => {
            expect([napsGroup(), frameGroup(),].map(function recount(group,): unknown {
              return coherent({ group, },);
            },),).toEqual([undefined, undefined,],);
          },
        },),
        it({
          name: 'RECOUNTS BALLOTS THAT COPIED A MARK BESIDE THEIR WORDS BY THE QUOTE THEY GAVE, as a section of generation '
            + 'fifteen was written: the quote recorded is the quote agreed, mark and all, and no mark evidence enters the recount',
          fn: async () => {
            const group = napsGroup();
            const copying = ballotsOf({
              group,
              change: function opens(ballot,): ArtifactConsistencyBallot {
                return {
                  ...ballot,
                  quotes: ballot.quotes
                    .map(function withMark(entry,): ArtifactConsistencyBallot['quotes'][number] {
                      return { ...entry, quote: `‘${entry.quote}`, };
                    },),
                };
              },
            },);
            const heard = withBallots({ group, ballots: copying, usable: 3, },);
            const recorded: ArtifactConsistencyGroup = {
              ...heard,
              occurrences: heard.occurrences.map(function recordsTheMark(place,): ArtifactConsistencyGroup['occurrences'][number] {
                return (place.rendering.kind === 'located')
                  ? { ...place, rendering: { ...place.rendering, quote: `‘${place.rendering.quote}`, }, }
                  : place;
              },),
            };
            expect([coherent({ group: recorded, },), coherenceRefusal({ group: heard, },),],).toEqual([
              undefined,
              `ArtifactParseError: artifact parse failed at ${GROUP}.occurrences[0].rendering: `
                + 'expected the quote and slot a strict majority of the usable ballots gave.',
            ],);
          },
        },),
        it({
          name: 'REFUSES A USABLE COUNT THAT IS NOT THE BALLOTS THAT CAN BE COUNTED, and leaves an unusable ballot out of it',
          fn: async () => {
            const group = napsGroup();
            const heard = ballotsOf({
              group,
              change: function stray(ballot,): ArtifactConsistencyBallot {
                return (ballot.seat === 'synthetic-cat') ? { ...ballot, choice: 'OCCURRENCE 9', } : ballot;
              },
            },);
            expect(coherenceRefusal({ group: withVerdict({ group, over: { usable: 2, }, },), },),).toBe(
              `ArtifactParseError: artifact parse failed at ${GROUP}.verdict.usable: `
                + 'expected the number of ballots that can be counted against the places.',
            );
            expect(coherenceRefusal({ group: withBallots({ group, ballots: heard, usable: 3, },), },),).toBe(
              `ArtifactParseError: artifact parse failed at ${GROUP}.verdict.usable: `
                + 'expected the number of ballots that can be counted against the places.',
            );
            expect(coherent({ group: withBallots({ group, ballots: heard, usable: 2, },), },),).toBeUndefined();
          },
        },),
        it({
          name: 'REFUSES A LOCATED QUOTE OR SLOT A STRICT MAJORITY OF THE BALLOTS DID NOT GIVE, naming the rendering',
          fn: async () => {
            expect([
              coherenceRefusal({
                group: withPlace({
                  group: napsGroup(),
                  position: 1,
                  over: { rendering: { kind: 'located', quote: 'The lamp naps', }, },
                }),
              },),
              coherenceRefusal({
                group: withPlace({
                  group: frameGroup(),
                  position: 1,
                  over: {
                    rendering: {
                      kind: 'located',
                      quote: 'So this cat must be a chubby little treasure.',
                      slot: 'little',
                    },
                  },
                }),
              },),
            ],).toEqual([
              `ArtifactParseError: artifact parse failed at ${GROUP}.occurrences[1].rendering: `
                + 'expected the quote and slot a strict majority of the usable ballots gave.',
              `ArtifactParseError: artifact parse failed at ${GROUP}.occurrences[1].rendering: `
                + 'expected the quote and slot a strict majority of the usable ballots gave.',
            ],);
          },
        },),
        it({
          name: 'REFUSES A LOCATED RENDERING OF A PLACE NO TEXT HAD A MAJORITY FOR',
          fn: async () => {
            const group = napsGroup();
            const split = ballotsOf({
              group,
              change: function disagree(
                ballot,
              ): ArtifactConsistencyBallot {
                return firstPlaceBallot({
                  seat: ballot.seat,
                  quotes: ballot.quotes
                    .map(function worded(
                      quote,
                    ): ArtifactConsistencyBallot['quotes'][number] {
                      return (quote.label === 'OCCURRENCE 2')
                        ? { ...quote, quote: `The ${ballot.seat} naps`, }
                        : quote;
                    },),
                },);
              },
            },);
            expect(coherenceRefusal({ group: withBallots({ group, ballots: split, usable: 3, },), },),).toBe(
              `ArtifactParseError: artifact parse failed at ${GROUP}.occurrences[1].rendering: `
                + 'expected the quote and slot a strict majority of the usable ballots gave.',
            );
          },
        },),
        it({
          name: 'REFUSES AN UNLOCATED REASON THE BALLOTS CONTRADICT, since a strict majority quoted words for the place',
          fn: async () => {
            expect(coherenceRefusal({
              group: withPlace({
                group: napsGroup(),
                position: 1,
                over: { rendering: { kind: 'unlocated', reason: 'no-agreement', }, outcome: 'unlocated', },
              }),
            },),).toBe(
              `ArtifactParseError: artifact parse failed at ${GROUP}.occurrences[1].rendering.reason: `
                + 'expected a reason the usable ballots do not contradict.',
            );
          },
        },),
        it({
          name: 'ACCEPTS A PLACE UNLOCATED FOR A REASON THE BALLOTS CANNOT CONTRADICT, recorded unlocated beside a winner',
          fn: async () => {
            expect(coherent({
              group: withPlace({
                group: napsGroup(),
                position: 1,
                over: { rendering: { kind: 'unlocated', reason: 'not-found', }, outcome: 'unlocated', },
              }),
            },),).toBeUndefined();
          },
        },),
        it({
          name: 'REFUSES A WINNER THE BALLOTS DO NOT GIVE: another place, none for a reason that is not theirs, or one '
            + 'where the quorum is not reached',
          fn: async () => {
            const group = napsGroup();
            expect([
              withVerdict({ group, over: { winner: { kind: 'chosen', label: 'OCCURRENCE 2', }, }, },),
              withVerdict({ group, over: { winner: { kind: 'none', reason: 'no-majority', }, }, },),
              withVerdict({ group, over: { winner: { kind: 'none', reason: 'chosen-unlocated', }, }, },),
              withVerdict({ group, over: { quorum: 4, }, },),
            ].map(function refusedFor(each,): string {
              return coherenceRefusal({ group: each, },);
            },),).toEqual([WINNER_REFUSAL, WINNER_REFUSAL, WINNER_REFUSAL, WINNER_REFUSAL,],);
            // A winner of none for a reason that is not theirs: a quorum of 4 against 3 usable ballots gives no-quorum.
            expect(coherenceRefusal({
              group: withVerdict({ group, over: { quorum: 4, winner: { kind: 'none', reason: 'no-majority', }, }, },),
            },),).toBe(WINNER_REFUSAL,);
          },
        },),
        it({
          name: 'ACCEPTS NO WINNER BELOW THE QUORUM, every place untouched, and refuses an outcome of a place beside it',
          fn: async () => {
            const group = napsGroup();
            const underQuorum = withVerdict({ group, over: { quorum: 4, winner: { kind: 'none', reason: 'no-quorum', }, }, },);
            const untouched = ['chosen', 'replaced', 'consistent',].reduce(
              function untouch(
                held,
                _outcome,
                position,
              ): ArtifactConsistencyGroup {
                return withPlace({ group: held, position, over: { outcome: 'untouched', }, },);
              },
              underQuorum,
            );
            expect(coherent({ group: untouched, },),).toBeUndefined();
            expect(coherenceRefusal({ group: withPlace({ group: underQuorum, position: 1, over: { outcome: 'replaced', }, },), },),)
              .toBe(
              `ArtifactParseError: artifact parse failed at ${GROUP}.occurrences[0].outcome: `
                + 'expected untouched, since no place won.',
              );
          },
        },),
        it({
          name: 'ACCEPTS NO WINNER WHERE THE PLACE A MAJORITY CHOSE HAS NO LOCATED RENDERING, every place untouched',
          fn: async () => {
            const group = napsGroup();
            const unlocated = withPlace({
              group,
              position: 0,
              over: { rendering: { kind: 'unlocated', reason: 'not-found', }, outcome: 'untouched', },
            },);
            const settled = [1, 2,].reduce(
              function untouch(
                held,
                position,
              ): ArtifactConsistencyGroup {
                return withPlace({ group: held, position, over: { outcome: 'untouched', }, },);
              },
              unlocated,
            );
            expect(coherent({
              group: withVerdict({ group: settled, over: { winner: { kind: 'none', reason: 'chosen-unlocated', }, }, },),
            },),).toBeUndefined();
            expect(coherenceRefusal({ group: settled, },),).toBe(WINNER_REFUSAL,);
          },
        },),
        it({
          name: 'REFUSES AN OUTCOME OF THE WINNING PLACE THAT IS NOT CHOSEN AND AN OUTCOME OF AN UNLOCATED PLACE THAT IS NOT '
            + 'UNLOCATED',
          fn: async () => {
            const unlocated = withPlace({
              group: napsGroup(),
              position: 1,
              over: { rendering: { kind: 'unlocated', reason: 'not-found', }, outcome: 'replaced', },
            },);
            expect([
              coherenceRefusal({ group: withPlace({ group: napsGroup(), position: 0, over: { outcome: 'replaced', }, },), },),
              coherenceRefusal({ group: unlocated, },),
            ],).toEqual([
              `ArtifactParseError: artifact parse failed at ${GROUP}.occurrences[0].outcome: `
                + 'expected chosen, for the winning place.',
              `ArtifactParseError: artifact parse failed at ${GROUP}.occurrences[1].outcome: `
                + 'expected unlocated, for a place whose rendering is not located.',
            ],);
          },
        },),
        it({
          name: 'READS A PLACE A STRICT MAJORITY KEEPS AS KEPT FOR ITS SURROUNDINGS, and refuses any other outcome for it',
          fn: async () => {
            const group = napsGroup();
            const keeping = withBallots({
              group,
              ballots: ballotsOf({
                group,
                change: function keepsTheSecond(ballot,): ArtifactConsistencyBallot {
                  return (ballot.seat === 'synthetic-cat') ? ballot : { ...ballot, keep: ['OCCURRENCE 2',], };
                },
              },),
              usable: 3,
            },);
            expect(coherent({
              group: withPlace({ group: keeping, position: 1, over: { outcome: 'kept-context', }, },),
            },),).toBeUndefined();
            expect(coherenceRefusal({ group: keeping, },),).toBe(
              `ArtifactParseError: artifact parse failed at ${GROUP}.occurrences[1].outcome: `
                + 'expected kept-context, for a place a strict majority keeps.',
            );
          },
        },),
        it({
          name: 'REFUSES A PLACE KEPT BY ONLY HALF OF THE USABLE BALLOTS AS KEPT FOR ITS SURROUNDINGS',
          fn: async () => {
            const group = napsGroup();
            const half = withBallots({
              group,
              ballots: ballotsOf({
                group,
                change: function keepsTheSecond(ballot,): ArtifactConsistencyBallot {
                  return (ballot.seat === 'hyper-cat') ? { ...ballot, keep: ['OCCURRENCE 2',], } : ballot;
                },
              },).slice(0, 2,),
              usable: 2,
            },);
            expect(coherenceRefusal({
              group: withPlace({ group: half, position: 1, over: { outcome: 'kept-context', }, },),
            },),).toBe(NOT_REPLACING,);
          },
        },),
        it({
          name: 'REFUSES A REPLACED PLACE RECORDED CONSISTENT AND A CONSISTENT PLACE RECORDED REPLACED',
          fn: async () => {
            expect([
              coherenceRefusal({ group: withPlace({ group: napsGroup(), position: 1, over: { outcome: 'consistent', }, },), },),
              coherenceRefusal({ group: withPlace({ group: napsGroup(), position: 2, over: { outcome: 'replaced', }, },), },),
            ],).toEqual([notConsistent({ place: 1, },), notConsistent({ place: 2, },),],);
          },
        },),
        it({
          name: 'REFUSES A PLACE THE CHOSEN WORDS WOULD REPLACE RECORDED UNTOUCHED OR CHOSEN, and accepts it kept by the '
            + 'floors or overlapped',
          fn: async () => {
            expect([
              coherenceRefusal({ group: withPlace({ group: napsGroup(), position: 1, over: { outcome: 'untouched', }, },), },),
              coherenceRefusal({ group: withPlace({ group: napsGroup(), position: 1, over: { outcome: 'chosen', }, },), },),
            ],).toEqual([NOT_REPLACING, NOT_REPLACING,],);
            expect([
              withPlace({ group: napsGroup(), position: 1, over: { outcome: 'kept-floor', }, },),
              withPlace({ group: napsGroup(), position: 1, over: { outcome: 'overlapped', }, },),
              withPlace({ group: napsGroup(), position: 1, over: { outcome: 'replaced', }, },),
            ].map(function recount(group,): unknown {
              return coherent({ group, },);
            },),).toEqual([undefined, undefined, undefined,],);
          },
        },),
        it({
          name: 'READS A FRAME PLACE THAT ALREADY CARRIES THE PATTERN WITH ITS OWN SLOT AS CONSISTENT, and refuses it replaced',
          fn: async () => {
            const alike = withPlace({
              group: frameGroup(),
              position: 1,
              over: {
                rendering: { kind: 'located', quote: 'So this cat is a chubby little treasure.', slot: 'chubby', },
                outcome: 'consistent',
              },
            },);
            const ballots = ballotsOf({
              group: alike,
              change: function alikeQuote(ballot,): ArtifactConsistencyBallot {
                return {
                  ...ballot,
                  quotes: ballot.quotes
                    .map(function worded(quote,): ArtifactConsistencyBallot['quotes'][number] {
                      return (quote.label === 'OCCURRENCE 2')
                        ? { ...quote, quote: 'So this cat is a chubby little treasure.', }
                        : quote;
                    },),
                };
              },
            },);
            const agreed = withBallots({ group: alike, ballots, usable: 3, },);
            expect(coherent({ group: agreed, },),).toBeUndefined();
            expect(coherenceRefusal({
              group: withPlace({ group: agreed, position: 1, over: { outcome: 'replaced', }, },),
            },),).toBe(notConsistent({ place: 1, },),);
          },
        },),
        it({
          name: 'REFUSES A LOCATED SLOT THAT DOES NOT STAND EXACTLY ONCE IN THE WORDS OF ITS QUOTE, though every ballot gives it',
          fn: async () => {
            const group = frameGroup();
            const bored = withBallots({
              group: withPlace({
                group,
                position: 1,
                over: {
                  rendering: { kind: 'located', quote: 'So this cat must be a chubby little treasure.', slot: 'bored', },
                },
              },),
              ballots: ballotsOf({
                group,
                change: function boredSlot(ballot,): ArtifactConsistencyBallot {
                  return {
                    ...ballot,
                    quotes: ballot.quotes
                      .map(function slotted(quote,): ArtifactConsistencyBallot['quotes'][number] {
                        return (quote.label === 'OCCURRENCE 2') ? { ...quote, slot: 'bored', } : quote;
                      },),
                  };
                },
              },),
              usable: 3,
            },);
            expect(coherenceRefusal({ group: bored, },),).toBe(
              `ArtifactParseError: artifact parse failed at ${GROUP}.occurrences[1].rendering: `
                + 'expected a slot that stands exactly once in the words of its quote.',
            );
          },
        },),
      ],
    },),
    describe({
      name: assertSlicesCoherent.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS ONE ROW FOR EACH SLICE HOLDING A REPLACED PLACE, in any order, across groups',
          fn: async () => {
            /**
             Slices the two groups replace a place in: the sentence's second place and the refrain's second.
             */
            const rows = [{ sliceIndex: 1, text: 'The kitten naps.', },];
            expect(assertSlicesCoherent({
              groups: [napsGroup(), frameGroup(),],
              slices: rows,
              path: 'cat.slices',
              generation: 17,
            },),).toBeUndefined();
            expect(assertSlicesCoherent({
              groups: [],
              slices: [],
              path: 'cat.slices',
              generation: 17,
            },),).toBeUndefined();
          },
        },),
        it({
          name: 'REQUIRES NO ROW FOR A PLACE THE FLOORS KEPT OR AN OVERLAP LEFT OUT',
          fn: async () => {
            expect(assertSlicesCoherent({
              groups: [
                withPlace({ group: napsGroup(), position: 1, over: { outcome: 'kept-floor', }, },),
                withPlace({
                  group: frameGroup(),
                  position: 1,
                  over: { outcome: 'overlapped', gaveWayTo: { group: 0, label: 'OCCURRENCE 2', }, },
                },),
              ],
              slices: [],
              path: 'cat.slices',
              generation: 17,
            },),).toBeUndefined();
          },
        },),
        it({
          name: 'REFUSES A SLICE LISTED TWICE, naming the slice',
          fn: async () => {
            expect(slicesRefusal({
              groups: [napsGroup(),],
              slices: [
                { sliceIndex: 1, text: 'The kitten naps.', },
                { sliceIndex: 1, text: 'The kitten naps again.', },
              ],
            },),).toBe(
              'ArtifactParseError: artifact parse failed at cat.slices: '
                + 'expected one row per slice; slice 1 appears more than once.',
            );
          },
        },),
        it({
          name: 'REFUSES A CHANGED SLICE WITH NO REPLACED PLACE, naming the slice',
          fn: async () => {
            expect(slicesRefusal({
              groups: [napsGroup(),],
              slices: [{ sliceIndex: 0, text: 'The cat naps.', },],
            },),).toBe(
              'ArtifactParseError: artifact parse failed at cat.slices: '
                + 'expected a replaced place in slice 0, which a changed slice needs.',
            );
          },
        },),
        it({
          name: 'REFUSES A REPLACED PLACE WITH NO CHANGED SLICE, naming the slice',
          fn: async () => {
            expect(slicesRefusal({ groups: [napsGroup(),], slices: [], },),).toBe(
              'ArtifactParseError: artifact parse failed at cat.slices: '
                + 'expected a changed slice row for slice 1, which holds a replaced place.',
            );
          },
        },),
      ],
    },),
  ],
},);
