/**
 Tests for telling which places of a group were carried under one bench record, which a reader of a unit carried in part
 uses to know that a splice candidate under a place code cannot move was built for another place, and for checking each
 carried place's outcome against its carry.

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
  type ArtifactConsistencyCarry,
  type ArtifactConsistencyOccurrence,
  assertCarriedOutcomes,
  type CarriedPlace,
  carriesSplice,
  spliceBuiltForSomePlace,
} from '../../dist/final/node/index.mjs';
import { refusalTextOf, } from './consistency-refusal.test-fixture.ts';

/**
 A carry for the second place whose bench chose the splice.
 */
const SPLICED: ArtifactConsistencyCarry = {
  label: 'OCCURRENCE 2',
  candidates: [
    { label: 'CANDIDATE 1', kind: 'current', writers: [], text: 'Later the kitten naps in the sun.', },
    { label: 'CANDIDATE 2', kind: 'splice', writers: [], text: 'Later the cat naps in the sun.', },
  ],
  ballots: [
    { seat: 'hyper-cat', reason: 'It words the original best.', choice: 'CANDIDATE 2', },
    { seat: 'bedrock-cat', reason: 'It words the original best.', choice: 'CANDIDATE 2', },
  ],
  usable: 2,
  quorum: 2,
  decision: { kind: 'chosen', label: 'CANDIDATE 2', },
};

/**
 A carry that put no question to the bench.
 */
const ALONE: ArtifactConsistencyCarry = {
  label: 'OCCURRENCE 2',
  candidates: [{ label: 'CANDIDATE 1', kind: 'current', writers: [], text: 'Later the kitten naps in the sun.', },],
  ballots: [],
  usable: 0,
  quorum: 0,
  decision: { kind: 'none', reason: 'no-alternative', },
};

/**
 A place the chosen words are carried to.

 @param label - place's label

 @param plan - plan recomputed for it

 @param carry - carry recorded under it

 @param outcome - what became of it, replaced by default

 @param keptBecause - why the pass left it as it was, none by default

 @returns Place

 @example
 ```ts
 const place = carriedAs({ label: 'OCCURRENCE 3', plan: 'replace', carry: SPLICED, outcome: 'replaced', },);
 ```
 */
function carriedAs(
  {
    label,
    plan,
    carry,
    outcome = 'replaced',
    keptBecause,
  }: {
    readonly label: string;
    readonly plan: CarriedPlace['plan'];
    readonly carry: ArtifactConsistencyCarry;
    readonly outcome?: ArtifactConsistencyOccurrence['outcome'];
    readonly keptBecause?: ArtifactConsistencyOccurrence['keptBecause'];
  },
): CarriedPlace {
  return {
    occurrence: {
      label,
      sliceIndex: 1,
      startOffset: 30,
      endOffset: 42,
      line: 5,
      rendering: { kind: 'located', quote: 'the kitten naps', start: 6, },
      outcome,
      ...(keptBecause === undefined ? {} : { keptBecause, }),
    },
    here: `cat.consistency.groups[0].occurrences[${label.slice(-1,)}]`,
    plan,
    carry: { ...carry, label, },
  };
}

/**
 The refusal text of checking the places.

 @param carriedTo - places

 @param partialUnits - whether the section records a unit carried in part

 @returns Refusal as text, or a note that nothing was refused

 @example
 ```ts
 const text = refusalOf({ carriedTo: [], partialUnits: true, },);
 ```
 */
function refusalOf(
  {
    carriedTo,
    partialUnits,
  }: {
    readonly carriedTo: readonly CarriedPlace[];
    readonly partialUnits: boolean;
  },
): string {
  return refusalTextOf({
    read: function checks(): unknown {
      return assertCarriedOutcomes({
        carriedTo,
        path: 'cat.consistency.groups[0]',
        marks: 'compared',
        partialUnits,
      },);
    },
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: carriesSplice.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'IS TRUE FOR A CARRY WITH A SPLICE AMONG ITS CANDIDATES AND FALSE FOR ONE WITHOUT',
          fn: async () => {
            expect([carriesSplice({ carry: SPLICED, },), carriesSplice({ carry: ALONE, },),],).toEqual([true, false,],);
          },
        },),
      ],
    },),
    describe({
      name: spliceBuiltForSomePlace.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'IS TRUE WHEN ANOTHER PLACE CODE CAN MOVE THE WORDS INTO WAS RECORDED UNDER THE SAME CANDIDATES, BALLOTS AND '
            + 'DECISION, whatever label that place carries',
          fn: async () => {
            expect(spliceBuiltForSomePlace({
              carry: SPLICED,
              carriedTo: [
                carriedAs({ label: 'OCCURRENCE 2', plan: 'unavailable', carry: SPLICED, },),
                carriedAs({ label: 'OCCURRENCE 3', plan: 'replace', carry: SPLICED, },),
              ],
            },),).toBe(true,);
          },
        },),
        it({
          name: 'IS FALSE WHEN NO OTHER PLACE IS ONE CODE CAN MOVE THE WORDS INTO, or the one that is stands under a record of its '
            + 'own, whether its candidates, its ballots, its counts or its decision differ',
          fn: async () => {
            /**
             The places of a group whose second place cannot be spliced and whose third is recorded as given.
             */
            function withThird({ carry, plan = 'replace', }: {
              readonly carry: ArtifactConsistencyCarry;
              readonly plan?: CarriedPlace['plan'];
            },): readonly CarriedPlace[] {
              return [
                carriedAs({ label: 'OCCURRENCE 2', plan: 'unavailable', carry: SPLICED, },),
                carriedAs({ label: 'OCCURRENCE 3', plan, carry, },),
              ];
            }
            expect([
              withThird({ carry: SPLICED, plan: 'unavailable', },),
              withThird({ carry: SPLICED, plan: 'same', },),
              withThird({ carry: { ...SPLICED, usable: 3, }, },),
              withThird({ carry: { ...SPLICED, quorum: 1, }, },),
              withThird({ carry: { ...SPLICED, decision: { kind: 'chosen', label: 'CANDIDATE 1', }, }, },),
              withThird({
                carry: {
                  ...SPLICED,
                  ballots: SPLICED.ballots.map(function other(ballot,): ArtifactConsistencyCarry['ballots'][number] {
                    return { ...ballot, reason: 'A different reason.', };
                  },),
                },
              },),
              withThird({ carry: { ...SPLICED, candidates: ALONE.candidates, }, },),
              [],
            ].map(function built(carriedTo,): boolean {
              return spliceBuiltForSomePlace({ carry: SPLICED, carriedTo, },);
            },),).toEqual([false, false, false, false, false, false, false, false,],);
          },
        },),
      ],
    },),
    describe({
      name: assertCarriedOutcomes.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS A PLACE CODE CANNOT MOVE KEPT-CURRENT UNDER A SPLICE BUILT FOR ANOTHER PLACE, only when the section '
            + 'records a unit carried in part',
          fn: async () => {
            /**
             The two places of a unit carried in part.
             */
            const carriedTo = [
              carriedAs({ label: 'OCCURRENCE 2', plan: 'unavailable', carry: SPLICED, outcome: 'kept-current', },),
              carriedAs({ label: 'OCCURRENCE 3', plan: 'replace', carry: SPLICED, },),
            ];
            expect(assertCarriedOutcomes({
              carriedTo,
              path: 'cat.consistency.groups[0]',
              marks: 'compared',
              partialUnits: true,
            },),).toBeUndefined();
            expect(refusalOf({ carriedTo, partialUnits: false, },),).toBe(
              'ArtifactParseError: artifact parse failed at cat.consistency.groups[0].carries: expected no splice candidate '
                + 'for OCCURRENCE 2, whose words code cannot move: the frame or the chosen frame has no slot to move them by, '
                + 'a single mark beside the words differs between the two places, the quotation marks inside the words '
                + 'differ, the two hold a different number of block breaks (a blank line, or a line break the Markdown parse '
                + 'reads as a new block), or a single mark stands beside the words and the chosen words end on that side in '
                + 'another class of character than the receiving words, a letter or a digit against anything else.',
            );
          },
        },),
        it({
          name: 'ACCEPTS KEPT-CURRENT FOR A REPLACE PLAN WHOSE SPLICE WAS CHOSEN, because the pass leaves the place as it '
            + 'was when the marks around the splice would read differently on its page, and refuses it for a writer\'s '
            + 'paragraph that was chosen',
          fn: async () => {
            /**
             The carry whose bench chose a writer's paragraph.
             */
            const WRITTEN: ArtifactConsistencyCarry = {
              ...SPLICED,
              candidates: [
                ...SPLICED.candidates,
                { label: 'CANDIDATE 3', kind: 'writer', writers: ['hyper-cat',], text: 'Later the cat naps.', },
              ],
              ballots: SPLICED.ballots.map(function chosenWriter(ballot,): ArtifactConsistencyCarry['ballots'][number] {
                return { ...ballot, choice: 'CANDIDATE 3', };
              },),
              decision: { kind: 'chosen', label: 'CANDIDATE 3', },
            };
            /**
             Checks a lone place the way the reader of a unit carried in part does.
             */
            function checkLone({ place, }: { readonly place: CarriedPlace; },): unknown {
              return assertCarriedOutcomes({
                carriedTo: [place,],
                path: 'cat.consistency.groups[0]',
                marks: 'compared',
                partialUnits: true,
              },);
            }
            expect([
              checkLone({
                place: carriedAs({
                  label: 'OCCURRENCE 2',
                  plan: 'replace',
                  carry: SPLICED,
                  outcome: 'kept-current',
                  keptBecause: 'gain-pairs',
                },),
              },),
              checkLone({
                place: carriedAs({
                  label: 'OCCURRENCE 2',
                  plan: 'replace',
                  carry: SPLICED,
                  outcome: 'kept-floor',
                },),
              },),
            ],).toEqual([undefined, undefined,],);
            expect([
              refusalOf({
                carriedTo: [
                  carriedAs({
                    label: 'OCCURRENCE 2',
                    plan: 'replace',
                    carry: WRITTEN,
                    outcome: 'kept-current',
                  },),
                ],
                partialUnits: true,
              },),
            ],).toEqual([
              'ArtifactParseError: artifact parse failed at cat.consistency.groups[0].occurrences[2].outcome: '
                + 'expected replaced, kept-floor, overlapped, for a carry that decided as recorded.',
            ],);
          },
        },),
        it({
          name: 'REFUSES KEPT-CURRENT FOR A REPLACE PLAN IN A SECTION THAT RECORDED NO MARKS BESIDE THE WORDS, which predates '
            + 'the decisions that leave a place as it was',
          fn: async () => {
            expect(refusalTextOf({
              read: function unrecorded(): unknown {
                return assertCarriedOutcomes({
                  carriedTo: [carriedAs({ label: 'OCCURRENCE 2', plan: 'replace', carry: SPLICED, outcome: 'kept-current', },),],
                  path: 'cat.consistency.groups[0]',
                  marks: 'unrecorded',
                  partialUnits: true,
                },);
              },
            },),).toBe(
              'ArtifactParseError: artifact parse failed at cat.consistency.groups[0].occurrences[2].outcome: '
                + 'expected replaced, kept-floor, overlapped, for a carry that decided as recorded.',
            );
          },
        },),
        it({
          name: 'REFUSES A PLACE CODE CANNOT MOVE RECORDED AS REPLACED AFTER THE SPLICE WAS CHOSEN, naming the place, and '
            + 'ACCEPTS A PLACE CODE CAN MOVE RECORDED AS KEPT-CURRENT, since the pass reads the English around a splice '
            + 'and may leave the place as it was when that reading changes a mark outside the splice',
          fn: async () => {
            expect(refusalOf({
              carriedTo: [
                carriedAs({ label: 'OCCURRENCE 2', plan: 'unavailable', carry: SPLICED, outcome: 'replaced', },),
                carriedAs({ label: 'OCCURRENCE 3', plan: 'replace', carry: SPLICED, },),
              ],
              partialUnits: true,
            },),).toBe(
              'ArtifactParseError: artifact parse failed at cat.consistency.groups[0].occurrences[2].outcome: '
                + 'expected kept-current, for a carry that decided as recorded.',
            );
            expect(assertCarriedOutcomes({
              carriedTo: [
                carriedAs({ label: 'OCCURRENCE 2', plan: 'unavailable', carry: SPLICED, outcome: 'kept-current', },),
                carriedAs({
                  label: 'OCCURRENCE 3',
                  plan: 'replace',
                  carry: SPLICED,
                  outcome: 'kept-current',
                  keptBecause: 'edge-reading-changes',
                },),
              ],
              path: 'cat.consistency.groups[0]',
              marks: 'compared',
              partialUnits: true,
            },),).toBeUndefined();
          },
        },),
        it({
          name: 'REFUSES KEPT-CURRENT FOR A REPLACE PLAN WHOSE SPLICE WAS CHOSEN WHEN THE RECORD GIVES NO REASON, naming the place '
            + 'and the closed set of reasons, and still accepts kept-current beside a carry that kept the place',
          fn: async () => {
            expect(refusalOf({
              carriedTo: [carriedAs({ label: 'OCCURRENCE 2', plan: 'replace', carry: SPLICED, outcome: 'kept-current', },),],
              partialUnits: true,
            },),).toBe(
              'ArtifactParseError: artifact parse failed at cat.consistency.groups[0].occurrences[2].keptBecause: expected a '
                + 'reason OCCURRENCE 2 was left as it was for, one of gain-pairs, gain-unproves, gain-floors, gain-overlaps, '
                + 'gain-refused, edge-second-mark, edge-reading-changes, edge-reference-mark; the record has none.',
            );
            expect(assertCarriedOutcomes({
              carriedTo: [carriedAs({ label: 'OCCURRENCE 2', plan: 'replace', carry: ALONE, outcome: 'kept-current', },),],
              path: 'cat.consistency.groups[0]',
              marks: 'compared',
              partialUnits: true,
            },),).toBeUndefined();
          },
        },),
        it({
          name: 'ACCEPTS KEPT-CURRENT FOR A REPLACE PLAN WITH EACH REASON OF THE CLOSED SET, and refuses it in a section that '
            + 'recorded no marks even with a reason',
          fn: async () => {
            /**
             Every reason the pass leaves a place as it was for.
             */
            const reasons = [
              'gain-pairs',
              'gain-unproves',
              'gain-floors',
              'gain-overlaps',
              'gain-refused',
              'edge-second-mark',
              'edge-reading-changes',
            ] as const;
            expect(reasons.map(function accepted(keptBecause,): unknown {
              return assertCarriedOutcomes({
                carriedTo: [
                  carriedAs({ label: 'OCCURRENCE 2', plan: 'replace', carry: SPLICED, outcome: 'kept-current', keptBecause, },),
                ],
                path: 'cat.consistency.groups[0]',
                marks: 'compared',
                partialUnits: true,
              },);
            },),).toEqual(reasons.map(function none(): undefined {
              return undefined;
            },),);
            expect(refusalTextOf({
              read: function unrecorded(): unknown {
                return assertCarriedOutcomes({
                  carriedTo: [
                    carriedAs({
                      label: 'OCCURRENCE 2',
                      plan: 'replace',
                      carry: SPLICED,
                      outcome: 'kept-current',
                      keptBecause: 'gain-pairs',
                    },),
                  ],
                  path: 'cat.consistency.groups[0]',
                  marks: 'unrecorded',
                  partialUnits: true,
                },);
              },
            },),).toBe(
              'ArtifactParseError: artifact parse failed at cat.consistency.groups[0].occurrences[2].outcome: '
                + 'expected replaced, kept-floor, overlapped, for a carry that decided as recorded.',
            );
          },
        },),
      ],
    },),
  ],
},);
