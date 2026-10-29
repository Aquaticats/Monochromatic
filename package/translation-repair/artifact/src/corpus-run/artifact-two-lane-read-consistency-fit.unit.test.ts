/**
 Tests for checking the places of a recorded group beside the kind of repeat
 and the slices the comparison names: labels in order, slices that exist, slot
 offsets lying inside their place and a slot in the rendering on the places of
 a frame alone, two places at least unless the group was skipped for too few,
 and a bench that was not asked meaning every place says so, with a slice of
 its own for each place of a group said to read alike.

 What the ballots say is the coherence check's question
 (`artifact-two-lane-read-consistency-coherence.unit.test.ts`).

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

import {
  type ArtifactConsistencyCarry,
  type ArtifactConsistencyGroup,
  type ArtifactConsistencyOccurrence,
  assertGroupFits,
} from '../../dist/final/node/index.mjs';
import {
  frameGroup,
  napsGroup,
  withPlace,
  withPlaceReplaced,
} from './consistency-group.test-fixture.ts';
import { refusalTextOf, } from './consistency-refusal.test-fixture.ts';

/**
 Slices the cat page's comparison names.
 */
const SLICES: ReadonlySet<number> = new Set([0, 1, 2,],);

/**
 The reason a place carrying a slot offset alone, or none on a frame, is refused
 for, which every offset case shares.
 */
const SLOT_OFFSETS_REASON = 'expected both slot offsets on a place of a frame, and neither on any other.';

/**
 The reason a slot that does not lie inside its place is refused for.
 */
const SLOT_BOUNDS_REASON = 'expected a slot that starts before it ends and lies inside its place.';

/**
 The refusal of a group with fewer than two places that is not a skip for too few.
 */
const TWO_PLACES_REFUSAL = 'ArtifactParseError: artifact parse failed at cat.groups[0].occurrences: '
  + 'expected two places at least, since only a skip for too few places has fewer.';

/**
 The text of the refusal the fit check raises for a group.

 @param group - group read

 @param sliceIndexes - slices the comparison names

 @returns Refusal as text

 @example
 ```ts
 const text = fitRefusal({ group: napsGroup(), sliceIndexes: SLICES, },);
 ```
 */
function fitRefusal(
  {
    group,
    sliceIndexes = SLICES,
    generation = 15,
  }: {
    readonly group: ArtifactConsistencyGroup;
    readonly sliceIndexes?: ReadonlySet<number>;
    readonly generation?: 15 | 16;
  },
): string {
  return refusalTextOf({
    read: function checksFit(): unknown {
      return assertGroupFits({
        group,
        path: 'cat.groups[0]',
        sliceIndexes,
        generation,
      },);
    },
  },);
}

/**
 What the fit check makes of a group it accepts.

 @param group - group read

 @returns Nothing, since the check only refuses

 @example
 ```ts
 const nothing = fitRead({ group: napsGroup(), },);
 ```
 */
function fitRead(
  {
    group,
    generation = 15,
  }: {
    readonly group: ArtifactConsistencyGroup;
    readonly generation?: 15 | 16;
  },
): unknown {
  return assertGroupFits({
    group,
    path: 'cat.groups[0]',
    sliceIndexes: SLICES,
    generation,
  },);
}

/**
 A place without its slot offsets, as a maximal repeat's place is written.

 @param place - place

 @returns Place with no slot offset

 @example
 ```ts
 const bare = withoutSlotOffsets({ place: frameGroup().occurrences[0], },);
 ```
 */
function withoutSlotOffsets(
  { place, }: { readonly place: ArtifactConsistencyOccurrence; },
): ArtifactConsistencyOccurrence {
  return {
    label: place.label,
    sliceIndex: place.sliceIndex,
    startOffset: place.startOffset,
    endOffset: place.endOffset,
    line: place.line,
    rendering: place.rendering,
    outcome: place.outcome,
  };
}

/**
 The group a bench was not asked about, every place saying so.

 @param reason - why the bench was not asked

 @param outcome - outcome every place carries

 @param places - how many places the group has

 @returns Recorded group

 @example
 ```ts
 const group = unaskedGroup({ reason: 'consistent', outcome: 'consistent', places: 2, },);
 ```
 */
function unaskedGroup(
  {
    reason,
    outcome,
    places,
  }: {
    readonly reason: 'too-few-placed' | 'no-english' | 'consistent';
    readonly outcome: 'consistent' | 'untouched';
    readonly places: number;
  },
): ArtifactConsistencyGroup {
  /**
   The asked group, whose places the unasked group keeps in number.
   */
  const asked = napsGroup();
  return {
    ...asked,
    occurrences: asked.occurrences
      .slice(
      0,
      places,
      )
      .map(function saysNotAsked(place,): ArtifactConsistencyOccurrence {
        return {
          ...place,
          rendering: { kind: 'unlocated', reason: 'not-asked', },
          outcome,
        };
      },),
    verdict: { kind: 'not-asked', reason, },
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: assertGroupFits.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS A REPEATED SENTENCE IN THREE PLACES AND A REFRAIN WITH A SLOT IN EACH, the bench asked about both',
          fn: async () => {
            expect([napsGroup(), frameGroup(),].map(function fits(group,): unknown {
              return fitRead({ group, },);
            },),).toEqual([undefined, undefined,],);
          },
        },),
        it({
          name: 'REFUSES A PLACE LABELLED OUT OF ORDER, naming the label its position carries',
          fn: async () => {
            expect(fitRefusal({
              group: withPlace({ group: napsGroup(), position: 1, over: { label: 'OCCURRENCE 3', }, },),
            },),).toBe(
              'ArtifactParseError: artifact parse failed at cat.groups[0].occurrences[1].label: '
                + 'expected the label OCCURRENCE 2, since the places are labelled in order.',
            );
          },
        },),
        it({
          name: 'REFUSES A PLACE IN A SLICE THE COMPARISON DOES NOT NAME',
          fn: async () => {
            expect(fitRefusal({ group: napsGroup(), sliceIndexes: new Set([0, 1,],), },),).toBe(
              'ArtifactParseError: artifact parse failed at cat.groups[0].occurrences[2].sliceIndex: '
                + 'expected a slice the comparison names.',
            );
          },
        },),
        it({
          name: 'REFUSES A PLACE OF A FRAME WITHOUT BOTH SLOT OFFSETS, whether it has neither, only the start or only the end',
          fn: async () => {
            /**
             The frame's second place with only the start of its slot, and with only its end.
             */
            const [, second,] = frameGroup().occurrences;
            const bareSecond = withoutSlotOffsets({ place: nonNullishOrThrow(second,), },);
            const startOnly = {
              ...bareSecond,
              slotStartOffset: 25,
            };
            const endOnly = {
              ...bareSecond,
              slotEndOffset: 27,
            };
            expect([
              fitRefusal({
                group: {
                  ...frameGroup(),
                  occurrences: frameGroup().occurrences
                    .map(function bare(place,): ArtifactConsistencyOccurrence {
                      return withoutSlotOffsets({ place, },);
                    },),
                },
              },),
              fitRefusal({ group: withPlaceReplaced({ group: frameGroup(), position: 1, place: startOnly, },), },),
              fitRefusal({ group: withPlaceReplaced({ group: frameGroup(), position: 1, place: endOnly, },), },),
            ],).toEqual([
              `ArtifactParseError: artifact parse failed at cat.groups[0].occurrences[0]: ${SLOT_OFFSETS_REASON}`,
              `ArtifactParseError: artifact parse failed at cat.groups[0].occurrences[1]: ${SLOT_OFFSETS_REASON}`,
              `ArtifactParseError: artifact parse failed at cat.groups[0].occurrences[1]: ${SLOT_OFFSETS_REASON}`,
            ],);
          },
        },),
        it({
          name: 'REFUSES A PLACE OF A REPEATED RUN CARRYING BOTH SLOT OFFSETS',
          fn: async () => {
            expect(fitRefusal({
              group: withPlace({ group: napsGroup(), position: 1, over: { slotStartOffset: 1, slotEndOffset: 2, }, },),
            },),).toBe(
              `ArtifactParseError: artifact parse failed at cat.groups[0].occurrences[1]: ${SLOT_OFFSETS_REASON}`,
            );
          },
        },),
        it({
          name: 'REFUSES A PLACE OF A REPEATED RUN CARRYING ONLY THE START OF A SLOT OR ONLY ITS END',
          fn: async () => {
            expect([
              { slotStartOffset: 1, },
              { slotEndOffset: 2, },
            ].map(function refusedFor(over,): string {
              return fitRefusal({ group: withPlace({ group: napsGroup(), position: 1, over, }), },);
            },),).toEqual([
              `ArtifactParseError: artifact parse failed at cat.groups[0].occurrences[1]: ${SLOT_OFFSETS_REASON}`,
              `ArtifactParseError: artifact parse failed at cat.groups[0].occurrences[1]: ${SLOT_OFFSETS_REASON}`,
            ],);
          },
        },),
        it({
          name: 'REFUSES A SLOT THAT ENDS WHERE IT STARTS OR BEFORE, STARTS BEFORE ITS PLACE OR ENDS AFTER IT, and accepts one '
            + 'that fills its whole place',
          fn: async () => {
            /**
             The frame's second place stands at 20 to 34, so these slots are empty, reversed, left of it and right of it.
             */
            const outside: readonly { readonly slotStartOffset: number; readonly slotEndOffset: number; }[] = [
              { slotStartOffset: 25, slotEndOffset: 25, },
              { slotStartOffset: 27, slotEndOffset: 25, },
              { slotStartOffset: 19, slotEndOffset: 27, },
              { slotStartOffset: 25, slotEndOffset: 35, },
            ];
            expect(outside.map(function refusedFor(over,): string {
              return fitRefusal({ group: withPlace({ group: frameGroup(), position: 1, over, }), },);
            },),).toEqual(outside.map(function refusal(): string {
              return `ArtifactParseError: artifact parse failed at cat.groups[0].occurrences[1]: ${SLOT_BOUNDS_REASON}`;
            },),);
            expect(fitRead({
              group: withPlace({ group: frameGroup(), position: 1, over: { slotStartOffset: 20, slotEndOffset: 34, }, },),
            },),).toBeUndefined();
          },
        },),
        it({
          name: 'REFUSES A LOCATED RENDERING OF A FRAME WITH NO SLOT AND A LOCATED RENDERING OF A REPEATED RUN WITH ONE',
          fn: async () => {
            expect([
              fitRefusal({
                group: withPlace({
                  group: frameGroup(),
                  position: 1,
                  over: { rendering: { kind: 'located', quote: 'So this cat must be a chubby little treasure.', }, },
                }),
              },),
              fitRefusal({
                group: withPlace({
                  group: napsGroup(),
                  position: 1,
                  over: { rendering: { kind: 'located', quote: 'The kitten naps', slot: 'kitten', }, },
                }),
              },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.groups[0].occurrences[1].rendering: '
                + 'expected a slot in the rendering of a place of a frame, and in no other.',
              'ArtifactParseError: artifact parse failed at cat.groups[0].occurrences[1].rendering: '
                + 'expected a slot in the rendering of a place of a frame, and in no other.',
            ],);
          },
        },),
        it({
          name: 'ACCEPTS, FROM GENERATION SIXTEEN, A LOCATED RENDERING OF A FRAME WITH NO SLOT, since a slot is no condition '
            + 'of a place being located, and still refuses a slot in the rendering of a repeated run',
          fn: async () => {
            const withoutSlot = withPlace({
              group: frameGroup(),
              position: 1,
              over: { rendering: { kind: 'located', quote: 'So this cat must be a chubby little treasure', start: 4, }, },
            });
            expect([
              fitRead({ group: withoutSlot, generation: 16, },),
              fitRefusal({ group: withoutSlot, generation: 15, },),
              fitRefusal({
                group: withPlace({
                  group: napsGroup(),
                  position: 1,
                  over: { rendering: { kind: 'located', quote: 'The kitten naps', start: 4, slot: 'kitten', }, },
                }),
                generation: 16,
              },),
            ],).toEqual([
              undefined,
              'ArtifactParseError: artifact parse failed at cat.groups[0].occurrences[1].rendering: '
                + 'expected a slot in the rendering of a place of a frame, and in no other.',
              'ArtifactParseError: artifact parse failed at cat.groups[0].occurrences[1].rendering: '
                + 'expected a slot in the rendering of a place of a frame only.',
            ],);
          },
        },),
        it({
          name: 'REFUSES CARRIES BESIDE A BENCH THAT WAS NOT ASKED OR WHOSE HANDLING FAILED, since there is nothing to carry',
          fn: async () => {
            const carry: ArtifactConsistencyCarry = {
              label: 'OCCURRENCE 2',
              candidates: [],
              ballots: [],
              usable: 0,
              quorum: 0,
              decision: { kind: 'failed', detail: 'refused by ConsistencyInvariantError', },
            };
            const groups: readonly ArtifactConsistencyGroup[] = [
              { ...unaskedGroup({ reason: 'consistent', outcome: 'consistent', places: 3, }), carries: [carry,], },
              {
                ...unaskedGroup({ reason: 'no-english', outcome: 'untouched', places: 3, }),
                verdict: { kind: 'failed', detail: 'refused by ConsistencyInvariantError', },
                carries: [carry,],
              },
            ];
            expect(groups.map(function refusedFor(group,): string {
              return fitRefusal({ group, generation: 16, },);
            },),).toEqual([
              'ArtifactParseError: artifact parse failed at cat.groups[0].carries: expected no carries, since the bench was not asked.',
              'ArtifactParseError: artifact parse failed at cat.groups[0].carries: expected no carries, since the bench was not asked.',
            ],);
          },
        },),
        it({
          name: 'ACCEPTS AN UNLOCATED RENDERING OF A FRAME WITH NO SLOT, since there is nothing located to hold one',
          fn: async () => {
            expect(fitRead({
              group: withPlace({
                group: frameGroup(),
                position: 1,
                over: { rendering: { kind: 'unlocated', reason: 'not-found', }, outcome: 'unlocated', },
              }),
            },),).toBeUndefined();
          },
        },),
        it({
          name: 'ACCEPTS A BENCH NOT ASKED WHEN EVERY PLACE SAYS SO: consistent places for a group that read alike, untouched '
            + 'places for a group with no English and a group of too few places',
          fn: async () => {
            expect([
              unaskedGroup({ reason: 'consistent', outcome: 'consistent', places: 3, }),
              unaskedGroup({ reason: 'no-english', outcome: 'untouched', places: 3, }),
              unaskedGroup({ reason: 'too-few-placed', outcome: 'untouched', places: 1, }),
              unaskedGroup({ reason: 'too-few-placed', outcome: 'untouched', places: 0, }),
            ].map(function fits(group,): unknown {
              return fitRead({ group, },);
            },),).toEqual([undefined, undefined, undefined, undefined,],);
          },
        },),
        it({
          name: 'REFUSES A GROUP WITH FEWER THAN TWO PLACES THAT THE BENCH WAS ASKED ABOUT OR SKIPPED FOR ANOTHER REASON THAN '
            + 'TOO FEW, since a group with fewer is skipped for too few',
          fn: async () => {
            expect([
              { ...napsGroup(), occurrences: napsGroup().occurrences.slice(0, 1,), },
              unaskedGroup({ reason: 'consistent', outcome: 'consistent', places: 1, }),
              unaskedGroup({ reason: 'no-english', outcome: 'untouched', places: 1, }),
              unaskedGroup({ reason: 'no-english', outcome: 'untouched', places: 0, }),
            ].map(function refusedFor(group,): string {
              return fitRefusal({ group, },);
            },),).toEqual([TWO_PLACES_REFUSAL, TWO_PLACES_REFUSAL, TWO_PLACES_REFUSAL, TWO_PLACES_REFUSAL,],);
          },
        },),
        it({
          name: 'REFUSES A GROUP WHOSE HANDLING FAILED WITH NO PLACE OR ONE, since a group is put to the bench only with two '
            + 'places and only a group put to the bench can fail',
          fn: async () => {
            expect([0, 1,].map(function refusedFor(places,): string {
              return fitRefusal({
                group: {
                  ...unaskedGroup({ reason: 'too-few-placed', outcome: 'untouched', places, }),
                  verdict: { kind: 'failed', detail: 'refused by ConsistencyInvariantError', },
                },
              },);
            },),).toEqual([TWO_PLACES_REFUSAL, TWO_PLACES_REFUSAL,],);
          },
        },),
        it({
          name: 'REFUSES TWO PLACES OF ONE SLICE IN A GROUP SAID TO READ ALIKE, since the pass asks the bench when two places '
            + 'stand in one slice, and accepts them in a group with no English',
          fn: async () => {
            expect(fitRefusal({
              group: withPlace({
                group: unaskedGroup({ reason: 'consistent', outcome: 'consistent', places: 3, }),
                position: 1,
                over: { sliceIndex: 0, },
              }),
            },),).toBe(
              'ArtifactParseError: artifact parse failed at cat.groups[0].occurrences[1].sliceIndex: '
                + 'expected a slice of its own, since the pass asks the bench when two places stand in one slice.',
            );
            expect(fitRead({
              group: withPlace({
                group: unaskedGroup({ reason: 'no-english', outcome: 'untouched', places: 3, }),
                position: 1,
                over: { sliceIndex: 0, },
              }),
            },),).toBeUndefined();
          },
        },),
        it({
          name: 'ACCEPTS A GROUP WHOSE HANDLING FAILED WHEN EVERY PLACE IS UNTOUCHED AND SAYS THE BENCH WAS NOT ASKED',
          fn: async () => {
            expect(fitRead({
              group: {
                ...unaskedGroup({ reason: 'too-few-placed', outcome: 'untouched', places: 3, }),
                verdict: { kind: 'failed', detail: 'refused by ConsistencyInvariantError', },
              },
            },),).toBeUndefined();
          },
        },),
        it({
          name: 'REFUSES A PLACE OF A BENCH NOT ASKED THAT SAYS MORE THAN SO: another outcome, a located rendering, or another '
            + 'reason for no rendering',
          fn: async () => {
            const consistent = unaskedGroup({ reason: 'consistent', outcome: 'consistent', places: 3, });
            const untouched = unaskedGroup({ reason: 'no-english', outcome: 'untouched', places: 3, });
            expect([
              fitRefusal({ group: withPlace({ group: consistent, position: 1, over: { outcome: 'untouched', }, }), },),
              fitRefusal({ group: withPlace({ group: untouched, position: 2, over: { outcome: 'consistent', }, }), },),
              fitRefusal({
                group: withPlace({
                  group: untouched,
                  position: 0,
                  over: { rendering: { kind: 'located', quote: 'The cat naps.', }, },
                }),
              },),
              fitRefusal({
                group: withPlace({
                  group: untouched,
                  position: 0,
                  over: { rendering: { kind: 'unlocated', reason: 'no-english', }, },
                }),
              },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.groups[0].occurrences[1]: expected a place with outcome '
                + 'consistent and no rendering looked for, since the bench was not asked.',
              'ArtifactParseError: artifact parse failed at cat.groups[0].occurrences[2]: expected a place with outcome '
                + 'untouched and no rendering looked for, since the bench was not asked.',
              'ArtifactParseError: artifact parse failed at cat.groups[0].occurrences[0]: expected a place with outcome '
                + 'untouched and no rendering looked for, since the bench was not asked.',
              'ArtifactParseError: artifact parse failed at cat.groups[0].occurrences[0]: expected a place with outcome '
                + 'untouched and no rendering looked for, since the bench was not asked.',
            ],);
          },
        },),
        it({
          name: 'REFUSES A PLACE OF A FAILED GROUP THAT CLAIMS THE PLACES READ ALIKE',
          fn: async () => {
            expect(fitRefusal({
              group: {
                ...unaskedGroup({ reason: 'consistent', outcome: 'consistent', places: 3, }),
                verdict: { kind: 'failed', detail: 'refused by ConsistencyInvariantError', },
              },
            },),).toBe(
              'ArtifactParseError: artifact parse failed at cat.groups[0].occurrences[0]: expected a place with outcome '
                + 'untouched and no rendering looked for, since the bench was not asked.',
            );
          },
        },),
        it({
          name: 'REFUSES A GROUP SKIPPED FOR TOO FEW PLACES THAT HAS TWO, and reads the same count in a group skipped for '
            + 'another reason',
          fn: async () => {
            expect(fitRefusal({
              group: unaskedGroup({ reason: 'too-few-placed', outcome: 'untouched', places: 2, }),
            },),).toBe(
              'ArtifactParseError: artifact parse failed at cat.groups[0].occurrences: '
                + 'expected fewer than two places, since the group was skipped for too few.',
            );
            expect(fitRead({
              group: unaskedGroup({ reason: 'no-english', outcome: 'untouched', places: 2, }),
            },),).toBeUndefined();
          },
        },),
      ],
    },),
  ],
},);
