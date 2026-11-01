/**
 Tests for deriving what one slice would ship, across every decider.

 ONE CASE PER PATH THROUGH THE DECIDERS, because the whole point of this
 reader is that no single field answers the question and every stage above a
 lane may replace what it left. A reader that got any one branch wrong would
 report wording no reader would ever see, which is exactly the failure the
 `repairDisposition: 'shipped'` name produced before it.

 NO READING MAY CARRY AN EMPTY STRING AS WORDING. `standingTextFor` returns
 `''` at a declined contest by design, so a reader that mirrored it would
 delete every declined slice from any document assembled off these readings.
 Both declines and the archive-silent case are pinned separately.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  type WouldShipReading,
  type WouldShipSource,
  UnansweredContestSliceError,
  wouldShipTextFor,
  wouldShipTextPerSlice,
} from '../../dist/final/node/index.mjs';
import { SEAT_HYPER_OPENROUTER_VISION_EDITOR, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Archive's own English at the slice every case reads.
 */
const ARCHIVE_NAP = 'The cat sleeps on the window ledge.';

/**
 Repair lane's wording for it.
 */
const REPAIR_NAP = 'The cat is asleep on the sill.';

/**
 Translate lane's wording for it.
 */
const TRANSLATE_NAP = 'The cat naps on the windowsill.';

/**
 Third rendering's wording for it.
 */
const CONSOLIDATED_NAP = 'The cat is napping on the windowsill.';

/**
 Builds one comparison row, defaulting to a slice where the lanes differ.

 @param over - fields this case replaces

 @returns Row as the parsed artifact carries it

 @example
 ```ts
 const row = rowWith({ incumbentKind: 'absent', },);
 ```
 */
function rowWith(over: Record<string, unknown> = {},): Record<string, unknown> {
  return {
    sliceIndex: 0,
    incumbentKind: 'present',
    incumbentText: ARCHIVE_NAP,
    repairText: REPAIR_NAP,
    translateText: TRANSLATE_NAP,
    laneRelation: 'both-differ',
    repairOutcome: {
      kind: 'decided',
      acceptedText: REPAIR_NAP,
    },
    translateOutcome: {
      kind: 'decided',
      acceptedText: TRANSLATE_NAP,
    },
    decisionComparison: {
      kind: 'comparable',
      verdict: 'different',
    },
    repairDelivery: { kind: 'replacement-shipped', },
    translateDelivery: { kind: 'replacement-shipped', },
    ...over,
  };
}

/**
 Builds one consolidation slice under a terminal that replaced nothing.

 @param terminal - how the slice left the stage

 @returns Slice as the parsed artifact carries it

 @example
 ```ts
 const slice = keptStanding({ terminal: 'gate-kept-standing', },);
 ```
 */
function keptStanding(
  { terminal, }: { readonly terminal: string; },
): Record<string, unknown> {
  return {
    sliceIndex: 0,
    terminal,
    shipped: { kind: 'unchanged', },
    rewrapped: false,
    demoted: false,
    verdicts: [],
    gate: { kind: 'not-asked', },
  };
}

/**
 Builds one contested slice under a verdict.

 @param verdict - what the roster settled

 @returns Slice as the parsed artifact carries it

 @example
 ```ts
 const slice = contestedWith({ verdict: { kind: 'settled-neither', }, },);
 ```
 */
function contestedWith(
  { verdict, }: { readonly verdict: Record<string, unknown>; },
): Record<string, unknown> {
  return {
    sliceIndex: 0,
    verdict,
    ballots: [],
    usable: 3,
  };
}

/**
 Builds the three fields a reading is derived from.

 @param row - comparison row to read, defaulting to differing lanes

 @param consolidation - what the third rendering says, defaulting to unasked

 @param laneSelection - which lane ships, defaulting to unasked

 @returns Source the reader accepts

 @example
 ```ts
 const source = sourceWith({ row: rowWith(), },);
 ```
 */
function sourceWith(
  {
    row = rowWith(),
    consolidation = { kind: 'not-run', },
    laneSelection = { kind: 'pending-human-decision', },
  }: {
    readonly row?: Record<string, unknown>;
    readonly consolidation?: Record<string, unknown>;
    readonly laneSelection?: Record<string, unknown>;
  } = {},
): WouldShipSource {
  return {
    comparison: [row,],
    consolidation,
    laneSelection,
  } as unknown as WouldShipSource;
}

/**
 Reads the first slice of a built source.

 @param source - three fields to read

 @returns What that slice would contribute

 @example
 ```ts
 const reading = firstReadingOf({ source, },);
 ```
 */
function firstReadingOf(
  { source, }: { readonly source: WouldShipSource; },
): WouldShipReading {
  return wouldShipTextFor({
    artifact: source,
    row: nonNullishOrThrow(source.comparison[0],),
  },);
}

/**
 Builds a source whose contest chose the translate lane and whose
 consolidation then kept the archive, because no wording passed the rule
 (owner, 2026-09-27, "Keep archive, ship").

 @param row - comparison row carrying the archive at this slice

 @returns Source the reader accepts

 @example
 ```ts
 const source = archiveKeptOver({ row: rowWith(), },);
 ```
 */
function archiveKeptOver(
  { row, }: { readonly row: Record<string, unknown>; },
): WouldShipSource {
  return sourceWith({
    row,
    consolidation: {
      kind: 'settled',
      slices: [
        {
          ...keptStanding({ terminal: 'incumbent-only', },),
          shipped: { kind: 'archive', },
        },
      ],
    },
    laneSelection: {
      kind: 'contested',
      slices: [
        contestedWith({
          verdict: {
            kind: 'lane-won',
            lane: 'translate',
          },
        },),
      ],
    },
  },);
}

/**
 The wording the consistency pass wrote at the slice every case reads.
 */
const CONSISTENT_NAP = 'The cat naps on the windowsill, as she always does.';

/**
 Final wording the polish approved at that slice, which the consistency pass stands over.
 */
const POLISHED_NAP = 'The cat dozes on the windowsill.';

/**
 What the consistency pass recorded: the one changed slice and nothing else.
 */
const CONSISTENCY_AT_ZERO = {
  groups: [],
  slices: [{ sliceIndex: 0, text: CONSISTENT_NAP, },],
  findings: [],
};

/**
 Builds a source whose contest chose the repair lane and whose consolidation
 kept that choice and polished it, so the polish is the decider below any
 consistency or page assembly record.

 @returns Source the reader accepts

 @example
 ```ts
 const source = polishedSource();
 ```
 */
function polishedSource(): WouldShipSource {
  return sourceWith({
    consolidation: {
      kind: 'settled',
      slices: [
        {
          ...keptStanding({ terminal: 'gate-kept-standing', },),
          polish: {
            kind: 'settled',
            baseText: REPAIR_NAP,
            proposedText: POLISHED_NAP,
            text: POLISHED_NAP,
            changed: true,
            refinersHeard: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
            contributors: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
            roundCount: 1,
            findings: [],
          },
        },
      ],
    },
    laneSelection: {
      kind: 'contested',
      slices: [
        contestedWith({
          verdict: {
            kind: 'lane-won',
            lane: 'repair',
          },
        },),
      ],
    },
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: wouldShipTextFor.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS THE PAGE ASSEMBLY FIRST: a trimmed slice ships the guard\'s text and a withdrawn slice '
            + 'stands as the archive, ahead of whatever the polish, the consolidation and the contest chose '
            + '(the twenty-second hakureico pass of 2026-09-09)',
          fn: async () => {
            /**
             Source whose page assembly trimmed slice 0.
             */
            const trimmedSource = {
              ...sourceWith(),
              pageAssembly: {
                trimmed: [{ sliceIndex: 0, replacementText: 'The cat naps[^1].', },],
                withdrawn: [],
                findings: ['assembly-footnote-trimmed orphan-definition gfm 2 (slice 0)',],
              },
            } as unknown as WouldShipSource;
            const [trimmedRow,] = trimmedSource.comparison;
            expect(wouldShipTextFor({ artifact: trimmedSource, row: nonNullishOrThrow(trimmedRow,), },),).toEqual({
              kind: 'wording',
              text: 'The cat naps[^1].',
              decidedBy: 'page-assembly',
            },);

            /**
             Source whose page assembly withdrew slice 0.
             */
            const withdrawnSource = {
              ...sourceWith(),
              pageAssembly: { trimmed: [], withdrawn: [0,], findings: [], },
            } as unknown as WouldShipSource;
            const [withdrawnRow,] = withdrawnSource.comparison;
            const reading = wouldShipTextFor({ artifact: withdrawnSource, row: nonNullishOrThrow(withdrawnRow,), },);
            expect(['archive', 'page-assembly-withdrew-and-archive-silent',],)
              .toContain((reading.kind === 'wording') ? reading.decidedBy : reading.reason,);
          },
        },),
        it({
          name: 'TAKES FINAL BODY POLISH before consolidation, contest, or archive wording',
          fn: async () => {
          /**
           Final idiomatic wording approved by polish gate.
           */
            const polished = 'The cat maintained a positive outlook on life.';
            const reading = firstReadingOf({
              source: sourceWith({
                consolidation: {
                  kind: 'settled',
                  slices: [
                    {
                      ...keptStanding({ terminal: 'gate-kept-standing', },),
                      polish: {
                        kind: 'settled',
                        baseText: REPAIR_NAP,
                        proposedText: polished,
                        text: polished,
                        changed: true,
                        refinersHeard: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
                        contributors: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
                        roundCount: 1,
                        findings: [],
                      },
                    },
                  ],
                },
                laneSelection: {
                  kind: 'contested',
                  slices: [
                    contestedWith({
                      verdict: {
                        kind: 'lane-won',
                        lane: 'repair',
                      },
                    },),
                  ],
                },
              },),
            },);
            expect(reading.kind,).toBe('wording',);
            expect(reading.kind === 'wording' ? reading.text : '',).toBe(polished,);
            expect(reading.kind === 'wording' ? reading.decidedBy : '',).toBe('polish',);
          },
        },),

        it({
          name: 'TAKES THE CONSISTENCY PASS\'S TEXT over the polish, the consolidation and the contest it was '
            + 'settled after, and attributes it to the pass',
          fn: async () => {
            expect(firstReadingOf({
              source: {
                ...polishedSource(),
                consistency: CONSISTENCY_AT_ZERO,
              } as unknown as WouldShipSource,
            },),).toEqual({
              kind: 'wording',
              text: CONSISTENT_NAP,
              decidedBy: 'consistency',
            },);
          },
        },),

        it({
          name: 'READS THE PAGE ASSEMBLY ABOVE THE CONSISTENCY PASS, since the assembly describes the page the '
            + 'pass\'s text is composed into',
          fn: async () => {
            expect(firstReadingOf({
              source: {
                ...polishedSource(),
                consistency: CONSISTENCY_AT_ZERO,
                pageAssembly: {
                  trimmed: [{ sliceIndex: 0, replacementText: 'The cat naps[^1].', },],
                  withdrawn: [],
                  findings: [],
                },
              } as unknown as WouldShipSource,
            },),).toEqual({
              kind: 'wording',
              text: 'The cat naps[^1].',
              decidedBy: 'page-assembly',
            },);
          },
        },),

        it({
          name: 'WALKS THE DECIDERS FROM THE PAGE ASSEMBLY DOWN, each surfacing at one slice when the one above it is '
            + 'taken away: the assembly, the consistency pass, the polish',
          fn: async () => {
          /**
           What the page assembly trimmed at the slice.
           */
            const assembly = {
              trimmed: [{ sliceIndex: 0, replacementText: 'The cat naps[^1].', },],
              withdrawn: [],
              findings: [],
            };
          /**
           Reads the polished source with the deciders a case keeps.
           */
            function ladderRung(
              { assemblyKept, consistencyKept, }: { readonly assemblyKept: boolean; readonly consistencyKept: boolean; },
            ): WouldShipReading {
              return firstReadingOf({
                source: {
                  ...polishedSource(),
                  ...(consistencyKept ? { consistency: CONSISTENCY_AT_ZERO, } : {}),
                  ...(assemblyKept ? { pageAssembly: assembly, } : {}),
                } as unknown as WouldShipSource,
              },);
            }
            expect([
              ladderRung({ assemblyKept: true, consistencyKept: true, },),
              ladderRung({ assemblyKept: false, consistencyKept: true, },),
              ladderRung({ assemblyKept: false, consistencyKept: false, },),
            ],).toEqual([
              { kind: 'wording', text: 'The cat naps[^1].', decidedBy: 'page-assembly', },
              { kind: 'wording', text: CONSISTENT_NAP, decidedBy: 'consistency', },
              { kind: 'wording', text: POLISHED_NAP, decidedBy: 'polish', },
            ],);
          },
        },),

        it({
          name: 'GIVES THE ARCHIVE BACK WHERE THE PAGE ASSEMBLY WITHDREW THE SLICE, whatever the consistency pass wrote there',
          fn: async () => {
            expect(firstReadingOf({
              source: {
                ...polishedSource(),
                consistency: CONSISTENCY_AT_ZERO,
                pageAssembly: { trimmed: [], withdrawn: [0,], findings: [], },
              } as unknown as WouldShipSource,
            },),).toEqual({ kind: 'wording', text: ARCHIVE_NAP, decidedBy: 'archive', },);
          },
        },),

        it({
          name: 'LEAVES THE POLISH IN FORCE at a slice the consistency pass did not change, however many other '
            + 'slices it did',
          fn: async () => {
            expect(firstReadingOf({
              source: {
                ...polishedSource(),
                consistency: {
                  groups: [],
                  slices: [{ sliceIndex: 7, text: CONSISTENT_NAP, },],
                  findings: [],
                },
              } as unknown as WouldShipSource,
            },),).toEqual({
              kind: 'wording',
              text: POLISHED_NAP,
              decidedBy: 'polish',
            },);
          },
        },),

        it({
          name:
            'TAKES THE THIRD RENDERING where it settled wording, because it ran after both other '
              + 'deciders and was free to replace what either left. On the two artifacts that exist it '
              + 'overrode 4 rows, including the one the contest had kept',
          fn: async () => {
            const reading = firstReadingOf({
              source: sourceWith({
                consolidation: {
                  kind: 'settled',
                  slices: [
                    {
                      ...keptStanding({ terminal: 'consolidated', },),
                      shipped: {
                        kind: 'consolidated',
                        text: CONSOLIDATED_NAP,
                      },
                    },
                  ],
                },
                laneSelection: {
                  kind: 'contested',
                  slices: [
                    contestedWith({
                      verdict: {
                        kind: 'lane-won',
                        lane: 'translate',
                      },
                    },),
                  ],
                },
              },),
            },);

            expect(reading.kind,).toBe('wording',);
            expect(reading.kind === 'wording' ? reading.text : '',).toBe(CONSOLIDATED_NAP,);
            expect(reading.kind === 'wording' ? reading.decidedBy : '',).toBe('consolidation',);
          },
        },),

        it({
          name:
            'TAKES THE INCUMBENT THAT STOOD IN under a kept-standing terminal, since the lane the contest '
              + 'chose left a standing the gate refused and the archive\'s text is what the settlement kept '
              + '(owner, 2026-09-09)',
          fn: async () => {
            const reading = firstReadingOf({
              source: sourceWith({
                consolidation: {
                  kind: 'settled',
                  slices: [
                    {
                      ...keptStanding({ terminal: 'gate-kept-standing', },),
                      shipped: {
                        kind: 'incumbent',
                        text: ARCHIVE_NAP,
                      },
                    },
                  ],
                },
                laneSelection: {
                  kind: 'contested',
                  slices: [
                    contestedWith({
                      verdict: {
                        kind: 'lane-won',
                        lane: 'translate',
                      },
                    },),
                  ],
                },
              },),
            },);

            expect(reading.kind,).toBe('wording',);
            expect(reading.kind === 'wording' ? reading.text : '',).toBe(ARCHIVE_NAP,);
            expect(reading.kind === 'wording' ? reading.decidedBy : '',).toBe('consolidation',);
          },
        },),

        it({
          name:
            'STANDS THE ARCHIVE where no wording passed the rule, over the lane the contest chose, and '
              + 'leaves a silent archive silent rather than writing an empty string into the page (owner, '
              + '2026-09-27, "Keep archive, ship")',
          fn: async () => {
            expect(firstReadingOf({ source: archiveKeptOver({ row: rowWith(), },), },),).toStrictEqual({
              kind: 'wording',
              text: ARCHIVE_NAP,
              decidedBy: 'archive',
            },);
            expect(firstReadingOf({
              source: archiveKeptOver({
                row: rowWith({
                  incumbentKind: 'absent',
                  incumbentText: '',
                },),
              },),
            },),).toStrictEqual({
              kind: 'nothing-ships',
              reason: 'no-valid-wording-and-archive-silent',
              incumbentKind: 'absent',
            },);
          },
        },),

        it({
          name:
            'FALLS THROUGH EVERY TERMINAL THAT REPLACED NOTHING, tested by shape rather than by '
              + 'enumerating names, so a terminal added later cannot silently start yielding text. Covers '
              + 'the retired spelling 11 rows across four settled entries still carry',
          fn: async () => {
            const terminals = [
              'incumbent-only',
              'no-standing-text',
              'slate-endorsed-standing',
              'slate-unjudged-standing',
              'slate-declined-standing',
              'gate-kept-standing',
              'wrap-erased-difference',
              'slate-kept-standing',
            ];

            for (const terminal of terminals) {
              const reading = firstReadingOf({
                source: sourceWith({
                  consolidation: {
                    kind: 'settled',
                    slices: [keptStanding({ terminal, },),],
                  },
                  laneSelection: {
                    kind: 'contested',
                    slices: [
                      contestedWith({
                        verdict: {
                          kind: 'lane-won',
                          lane: 'repair',
                        },
                      },),
                    ],
                  },
                },),
              },);

              expect(reading.kind === 'wording' ? reading.text : '',).toBe(REPAIR_NAP,);
              expect(reading.kind === 'wording' ? reading.decidedBy : '',).toBe('contest',);
            }
          },
        },),

        it({
          name:
            'SHIPS THE APPLICABLE LANE at a sole-lane slice, the archive-absent insertion the translate lane '
              + 'filled, since no roster backed that verdict and the lane that applies is the wording it ships',
          fn: async () => {
          /**
           Insertion both lanes read as absent from the archive, with only the translate lane applying.
           */
            const insertion = rowWith({
              incumbentKind: 'absent',
              incumbentText: '',
              repairText: '',
              laneRelation: 'translate-only',
              repairOutcome: { kind: 'not-applicable', },
              decisionComparison: {
                kind: 'not-comparable',
                undecidedLanes: ['repair',],
              },
              repairDelivery: { kind: 'gap-remains', },
            },);
            const reading = firstReadingOf({
              source: sourceWith({
                row: insertion,
                laneSelection: {
                  kind: 'contested',
                  slices: [
                    {
                      sliceIndex: 0,
                      verdict: {
                        kind: 'sole-lane',
                        lane: 'translate',
                      },
                      ballots: [],
                      usable: 0,
                    },
                  ],
                },
              },),
            },);

            expect(reading,).toEqual({
              kind: 'wording',
              text: TRANSLATE_NAP,
              decidedBy: 'contest',
            },);
          },
        },),

        it({
          name:
            'TAKES THE LANE THE CONTEST NAMED, on each side, since reading the winning lane off either '
              + 'lane ledger alone was wrong at every measurable row: 6 rows claimed replacement-shipped '
              + 'and 0 reached the page',
          fn: async () => {
            const wins = [
              {
                lane: 'repair',
                text: REPAIR_NAP,
              },
              {
                lane: 'translate',
                text: TRANSLATE_NAP,
              },
            ];

            for (const win of wins) {
              const reading = firstReadingOf({
                source: sourceWith({
                  laneSelection: {
                    kind: 'contested',
                    slices: [
                      contestedWith({
                        verdict: {
                          kind: 'lane-won',
                          lane: win.lane,
                        },
                      },),
                    ],
                  },
                },),
              },);

              expect(reading.kind,).toBe('wording',);
              expect(reading.kind === 'wording' ? reading.text : '',).toBe(win.text,);
              expect(reading.kind === 'wording' ? reading.decidedBy : '',).toBe('contest',);
            }
          },
        },),

        it({
          name:
            'KEEPS THE ARCHIVE STANDING WHEN THE CONTEST DECLINED, on both declines, and REFUSES TO '
              + 'SHIP THE EMPTY STRING. standingTextFor returns empty on a decline on purpose, because it '
              + 'answers what a slate must beat; a document assembled off that would delete the slice',
          fn: async () => {
            const declines = [
              { kind: 'settled-neither', },
              { kind: 'quorum-not-met', },
            ];

            for (const verdict of declines) {
              const reading = firstReadingOf({
                source: sourceWith({
                  laneSelection: {
                    kind: 'contested',
                    slices: [contestedWith({ verdict, },),],
                  },
                },),
              },);

              expect(reading.kind,).toBe('wording',);
              expect(reading.kind === 'wording' ? reading.text : '',).toBe(ARCHIVE_NAP,);
              expect(reading.kind === 'wording' ? reading.decidedBy : '',).toBe('archive',);
            }
          },
        },),

        it({
          name:
            'NAMES THE SILENCE WHERE A DECLINED CONTEST MEETS AN ARCHIVE THAT HELD NOTHING, rather '
              + 'than returning an empty string that a caller would write into a document as wording',
          fn: async () => {
            const reading = firstReadingOf({
              source: sourceWith({
                row: rowWith({
                  incumbentKind: 'absent',
                  incumbentText: '',
                },),
                laneSelection: {
                  kind: 'contested',
                  slices: [contestedWith({ verdict: { kind: 'settled-neither', }, },),],
                },
              },),
            },);

            expect(reading.kind,).toBe('nothing-ships',);
            expect(reading.kind === 'nothing-ships' ? reading.reason : '',)
              .toBe('contest-declined-and-archive-silent',);
          },
        },),

        it({
          name:
            'SHIPS THE WORDING BOTH LANES AGREED ON where the contest never saw the slice, since '
              + 'contestEligibleIndexes makes a slice eligible exactly where the lane texts differ, so an '
              + 'unlisted slice is one where they match and agreement needs no decider',
          fn: async () => {
            const reading = firstReadingOf({
              source: sourceWith({
                row: rowWith({
                  repairText: TRANSLATE_NAP,
                  laneRelation: 'both-agree',
                },),
                laneSelection: {
                  kind: 'contested',
                  slices: [],
                },
              },),
            },);

            expect(reading.kind === 'wording' ? reading.text : '',).toBe(TRANSLATE_NAP,);
            expect(reading.kind === 'wording' ? reading.decidedBy : '',).toBe('lanes-agreed',);
          },
        },),

        it({
          name:
            'NAMES THE SILENCE WHERE BOTH LANES AGREED ON NOTHING, and does NOT revive the archive '
              + 'underneath it. Two lanes removing wording is a decision, and republishing what they both '
              + 'dropped would undo it',
          fn: async () => {
            const reading = firstReadingOf({
              source: sourceWith({
                row: rowWith({
                  repairText: '',
                  translateText: '',
                  laneRelation: 'gap-remains',
                },),
                laneSelection: {
                  kind: 'contested',
                  slices: [],
                },
              },),
            },);

            expect(reading.kind,).toBe('nothing-ships',);
            expect(reading.kind === 'nothing-ships' ? reading.reason : '',)
              .toBe('lanes-agreed-on-no-wording',);
          },
        },),

        it({
          name:
            'KEEPS THE ARCHIVE STANDING WHERE NO CONTEST HAS RUN, and names that silence separately '
              + 'from a decline, because an entry nobody has judged and an entry the judges could not '
              + 'settle are different facts about the roster',
          fn: async () => {
            const standing = firstReadingOf({ source: sourceWith(), },);
            expect(standing.kind === 'wording' ? standing.text : '',).toBe(ARCHIVE_NAP,);
            expect(standing.kind === 'wording' ? standing.decidedBy : '',).toBe('archive',);

            const silent = firstReadingOf({
              source: sourceWith({
                row: rowWith({
                  incumbentKind: 'absent',
                  incumbentText: '',
                },),
              },),
            },);
            expect(silent.kind === 'nothing-ships' ? silent.reason : '',)
              .toBe('contest-unasked-and-archive-silent',);
          },
        },),

        it({
          name:
            'READS BOTH CONSOLIDATION ABSENCES the same way and neither as a decision. A pass that '
              + 'chose not to ask and an artifact written before the field existed both leave whatever '
              + 'the contest settled standing',
          fn: async () => {
            const absences = [
              { kind: 'not-run', },
              { kind: 'unrecorded', },
            ];

            for (const consolidation of absences) {
              const reading = firstReadingOf({
                source: sourceWith({
                  consolidation,
                  laneSelection: {
                    kind: 'contested',
                    slices: [
                      contestedWith({
                        verdict: {
                          kind: 'lane-won',
                          lane: 'translate',
                        },
                      },),
                    ],
                  },
                },),
              },);

              expect(reading.kind === 'wording' ? reading.text : '',).toBe(TRANSLATE_NAP,);
            }
          },
        },),

        it({
          name:
            'REFUSES A SLICE WHOSE LANES DIFFER THAT THE CONTEST RECORD NAMES NOWHERE, rather than '
              + 'reading it as agreement and picking one lane with nothing behind it. The parser forbids '
              + 'that artifact, so reaching it means the record contradicts itself',
          fn: async () => {
          /**
           What unanswered raised, read for its class as well as its wording.
           */
            const refusalOfUnanswered = caught(function unanswered() {
              firstReadingOf({
                source: sourceWith({
                  laneSelection: {
                    kind: 'contested',
                    slices: [],
                  },
                },),
              },);
            },);

            expect(refusalOfUnanswered,).toBeInstanceOf(UnansweredContestSliceError,);
            expect((refusalOfUnanswered as Error).message,).toContain('names it nowhere',);
          },
        },),
      ],
    },),

    describe({
      name: wouldShipTextPerSlice.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'ANSWERS EVERY COMPARISON ROW IN ORDER, so a consumer counting subjects reads a '
              + 'denominator over the whole document rather than over whichever stage it happened to open',
          fn: async () => {
            const source = {
              comparison: [
                rowWith({ sliceIndex: 0, },),
                rowWith({
                  sliceIndex: 1,
                  repairText: TRANSLATE_NAP,
                  laneRelation: 'both-agree',
                },),
              ],
              consolidation: { kind: 'not-run', },
              laneSelection: {
                kind: 'contested',
                slices: [
                  contestedWith({
                    verdict: {
                      kind: 'lane-won',
                      lane: 'repair',
                    },
                  },),
                ],
              },
            } as unknown as WouldShipSource;

            const slices = wouldShipTextPerSlice({ artifact: source, },);

            expect(slices.length,).toBe(2,);

          /**
           Reading of the contested slice, first in comparison-row order.
           */
            const contestedSlice = nonNullishOrThrow(slices[0],);

          /**
           Reading of the slice both lanes agreed on, second in that order.
           */
            const agreedSlice = nonNullishOrThrow(slices[1],);

            expect(contestedSlice.sliceIndex,).toBe(0,);
            expect(contestedSlice.reading.kind === 'wording' ? contestedSlice.reading.decidedBy : '',)
              .toBe('contest',);
            expect(agreedSlice.sliceIndex,).toBe(1,);
            expect(agreedSlice.reading.kind === 'wording' ? agreedSlice.reading.decidedBy : '',)
              .toBe('lanes-agreed',);
          },
        },),

        it({
          name:
            'RESTORES the archive\'s quote convention on every wording a stage wrote, since the repair lane '
              + 'already does and the translate lane and consolidation did not: yulianNyanner shipped five '
              + 'straight apostrophes on a curly page on 2026-09-06, and Uekawakuyuurei two on 2026-09-04',
          fn: async () => {
            const source = {
              comparison: [
                rowWith({
                  sliceIndex: 0,
                  incumbentText: 'The cat’s asleep on the ledge.',
                  translateText: 'The cat isn\'t asleep on the sill.',
                },),
                rowWith({
                  sliceIndex: 1,
                  incumbentText: 'The cat’s awake.',
                  repairText: 'The cat\'s awake now.',
                  translateText: 'The cat\'s awake now.',
                  laneRelation: 'both-agree',
                },),
              ],
              consolidation: { kind: 'not-run', },
              laneSelection: {
                kind: 'contested',
                slices: [
                  contestedWith({
                    verdict: {
                      kind: 'lane-won',
                      lane: 'translate',
                    },
                  },),
                ],
              },
              preparation: {
                archiveText: {
                  kind: 'stored',
                  text: 'The cat’s asleep on the ledge.\n\nThe cat’s awake.\n',
                },
              },
            } as unknown as WouldShipSource;

            const slices = wouldShipTextPerSlice({ artifact: source, },);

          /**
           Wording the translate lane won, its apostrophe curled to the page's.
           */
            const contestedSlice = nonNullishOrThrow(slices[0],);

          /**
           Wording both lanes agreed on, curled the same way.
           */
            const agreedSlice = nonNullishOrThrow(slices[1],);

            expect(contestedSlice.reading.kind === 'wording' ? contestedSlice.reading.text : '',)
              .toBe('The cat isn’t asleep on the sill.',);
            expect(agreedSlice.reading.kind === 'wording' ? agreedSlice.reading.text : '',)
              .toBe('The cat’s awake now.',);
          },
        },),

        it({
          name: 'RESTORES the archive\'s quote convention on the consistency pass\'s text as on any stage\'s, '
            + 'so the page and the artifact agree on the apostrophe',
          fn: async () => {
            const source = {
              comparison: [
                rowWith({
                  sliceIndex: 0,
                  incumbentText: 'The cat’s asleep on the ledge.',
                },),
              ],
              consolidation: { kind: 'not-run', },
              laneSelection: { kind: 'pending-human-decision', },
              consistency: {
                groups: [],
                slices: [{ sliceIndex: 0, text: 'The cat\'s asleep on the sill.', },],
                findings: [],
              },
              preparation: {
                archiveText: {
                  kind: 'stored',
                  text: 'The cat’s asleep on the ledge.\n',
                },
              },
            } as unknown as WouldShipSource;

            const slice = nonNullishOrThrow(wouldShipTextPerSlice({ artifact: source, },)[0],);

            expect(slice.reading,).toEqual({
              kind: 'wording',
              text: 'The cat’s asleep on the sill.',
              decidedBy: 'consistency',
            },);
          },
        },),

        it({
          name: 'LEAVES a straight-quoted page alone, since the convention is the archive\'s and not this rule\'s',
          fn: async () => {
            const source = {
              comparison: [
                rowWith({
                  sliceIndex: 0,
                  incumbentText: 'The cat\'s asleep on the ledge.',
                  translateText: 'The cat isn\'t asleep on the sill.',
                },),
              ],
              consolidation: { kind: 'not-run', },
              laneSelection: {
                kind: 'contested',
                slices: [
                  contestedWith({
                    verdict: {
                      kind: 'lane-won',
                      lane: 'translate',
                    },
                  },),
                ],
              },
              preparation: {
                archiveText: {
                  kind: 'stored',
                  text: 'The cat\'s asleep on the ledge.\n',
                },
              },
            } as unknown as WouldShipSource;

            const slice = nonNullishOrThrow(wouldShipTextPerSlice({ artifact: source, },)[0],);

            expect(slice.reading.kind === 'wording' ? slice.reading.text : '',)
              .toBe('The cat isn\'t asleep on the sill.',);
          },
        },),
      ],
    },),
  ],
},);
