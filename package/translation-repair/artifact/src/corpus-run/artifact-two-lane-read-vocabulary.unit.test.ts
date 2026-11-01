/**
 Tests for reading version 2's unions and rows back off disk.

 WHAT THESE PIN is the schema-ownership rule, which decides where reading is
 strict and where it is tolerant. Version 2 owns the ledger and the
 comparison, so a key it does not name there means a file this reader cannot
 read. The raw lane results belong to the live pipeline, which has added
 fields before and will again, so a key version 2 does not name THERE is
 evidence a later lane recorded and not a later version of this artifact.

 The pair that is easy to get wrong sits in the middle: inside a raw result, a
 field version 2 never heard of is tolerated, while `acceptedText` on an
 outcome that decided nothing is refused. One is a later pipeline adding
 evidence; the other is this version's own vocabulary used to say something it
 cannot mean.

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

import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  parseComparisonRow,
  parseDecisionComparison,
  parseDeliveryRow,
  parseEvidenceRow,
  parseSliceDelivery,
  parseSliceOutcome,
  SLICE_SPELLED_KEYS,
} from '../../dist/final/node/index.mjs';

/**
 Archive wording every fixture here shares.
 */
const ARCHIVE_NAP = 'The cat sleeps on the sill.';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: parseSliceOutcome.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'reads every member this version describes, and carries a decision`s wording back with it',
          fn: async () => {
            expect(parseSliceOutcome({
              value: {
                kind: 'decided',
                acceptedText: ARCHIVE_NAP,
              },
              unknownKeys: 'refuse',
              path: 'outcome',
            },),).toEqual({
              kind: 'decided',
              acceptedText: ARCHIVE_NAP,
            },);
            expect([
              'not-evaluated',
              'unfilled',
              'incumbent-fallback',
              'not-applicable',
            ].map(function readKind(kind,): unknown {
              return parseSliceOutcome({
                value: { kind, },
                unknownKeys: 'refuse',
                path: 'outcome',
              },);
            },),).toEqual([
              { kind: 'not-evaluated', },
              { kind: 'unfilled', },
              { kind: 'incumbent-fallback', },
              { kind: 'not-applicable', },
            ],);
          },
        },),
        it({
          name:
            'READS THE FLOOR MARK ON AN UNFILLED OUTCOME where the generation records it, so a slice the select '
              + 'bench never bought a candidate for reads apart from one the panel declined twice',
          fn: async () => {
            expect({
              marked: parseSliceOutcome({
                value: {
                  kind: 'unfilled',
                  benchBelowFloor: true,
                },
                unknownKeys: 'refuse',
                path: 'outcome',
                benchFloorRecorded: true,
              },),
              declined: parseSliceOutcome({
                value: { kind: 'unfilled', },
                unknownKeys: 'refuse',
                path: 'outcome',
                benchFloorRecorded: true,
              },),
            },).toEqual({
              marked: {
                kind: 'unfilled',
                benchBelowFloor: true,
              },
              declined: { kind: 'unfilled', },
            },);
          },
        },),
        it({
          name:
            'REFUSES THE FLOOR MARK where the generation does not record it, on a member that decided or fell back, '
              + 'and set to false',
          fn: async () => {
            /**
             Refusal text for one outcome read as a generation that records the mark does, or as one that does not.

             @param value - outcome as recorded

             @param benchFloorRecorded - whether the generation records the mark

             @returns Refusal
             */
            function refusalOf(
              {
                value,
                benchFloorRecorded,
              }: {
                readonly value: unknown;
                readonly benchFloorRecorded: boolean;
              },
            ): string {
              return String(caught(function readMarked() {
                parseSliceOutcome({
                  value,
                  unknownKeys: 'refuse',
                  path: 'outcome',
                  benchFloorRecorded,
                },);
              },),);
            }
            expect({
              older: refusalOf({
                value: {
                  kind: 'unfilled',
                  benchBelowFloor: true,
                },
                benchFloorRecorded: false,
              },),
              decided: refusalOf({
                value: {
                  kind: 'decided',
                  acceptedText: ARCHIVE_NAP,
                  benchBelowFloor: true,
                },
                benchFloorRecorded: true,
              },),
              falsy: refusalOf({
                value: {
                  kind: 'unfilled',
                  benchBelowFloor: false,
                },
                benchFloorRecorded: true,
              },),
            },).toEqual({
              older: 'ArtifactParseError: artifact parse failed at outcome.benchBelowFloor: '
                + 'expected no key here beyond kind.',
              decided: 'ArtifactParseError: artifact parse failed at outcome.benchBelowFloor: '
                + 'expected no key here beyond kind, acceptedText.',
              falsy: 'ArtifactParseError: artifact parse failed at outcome.benchBelowFloor: expected true or absent.',
            },);
          },
        },),
        it({
          name:
            'REFUSES a discriminator this version cannot project, under BOTH key policies: a member named '
              + 'here that version 2 never described has no reading at all, and taking the row anyway would '
              + 'record a slice under a name this reader invented',
          fn: async () => {
            /**
             What unknownKind raised, read for its class as well as its wording.
             */
            const refusalOfUnknownKind = caught(function unknownKind() {
              parseSliceOutcome({
                value: { kind: 'napped-through-it', },
                unknownKeys: 'refuse',
                path: 'outcome',
              },);
            },);

            expect(refusalOfUnknownKind,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfUnknownKind as Error).message,).toContain('outcome.kind',);
            /**
             What unknownKindTolerated raised, read for its class as well as its wording.
             */
            const refusalOfUnknownKindTolerated = caught(function unknownKindTolerated() {
              parseSliceOutcome({
                value: { kind: 'napped-through-it', },
                unknownKeys: 'tolerate',
                path: 'outcome',
              },);
            },);

            expect(refusalOfUnknownKindTolerated,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfUnknownKindTolerated as Error).message,).toContain('outcome.kind',);
          },
        },),
        it({
          name:
            'TOLERATES a field version 2 never described where the live pipeline owns the shape, and REFUSES '
              + 'the same field where version 2 owns it: one is a later lane recording evidence, the other is a '
              + 'file this version cannot read',
          fn: async () => {
            expect(parseSliceOutcome({
              value: {
                kind: 'decided',
                acceptedText: ARCHIVE_NAP,
                confidence: 0.5,
              },
              unknownKeys: 'tolerate',
              path: 'outcome',
            },),).toEqual({
              kind: 'decided',
              acceptedText: ARCHIVE_NAP,
            },);
            /**
             What strictHere raised, read for its class as well as its wording.
             */
            const refusalOfStrictHere = caught(function strictHere() {
              parseSliceOutcome({
                value: {
                  kind: 'decided',
                  acceptedText: ARCHIVE_NAP,
                  confidence: 0.5,
                },
                unknownKeys: 'refuse',
                path: 'outcome',
              },);
            },);

            expect(refusalOfStrictHere,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfStrictHere as Error).message,).toContain('outcome.confidence',);
          },
        },),
        it({
          name:
            'REFUSES a RESERVED field on a member that has no meaning for it, even while tolerating unknown '
              + 'ones: an outcome that decided nothing cannot carry the wording it decided, and reading past '
              + 'that would hand a caller a wording no lane chose',
          fn: async () => {
            /**
             What misplacedText raised, read for its class as well as its wording.
             */
            const refusalOfMisplacedText = caught(function misplacedText() {
              parseSliceOutcome({
                value: {
                  kind: 'not-evaluated',
                  acceptedText: ARCHIVE_NAP,
                },
                unknownKeys: 'tolerate',
                path: 'outcome',
              },);
            },);

            expect(refusalOfMisplacedText,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfMisplacedText as Error).message,).toContain('outcome.acceptedText',);
            /**
             What misplacedTextOnFallback raised, read for its class as well as its wording.
             */
            const refusalOfMisplacedTextOnFallback = caught(function misplacedTextOnFallback() {
              parseSliceOutcome({
                value: {
                  kind: 'incumbent-fallback',
                  acceptedText: ARCHIVE_NAP,
                },
                unknownKeys: 'tolerate',
                path: 'outcome',
              },);
            },);

            expect(refusalOfMisplacedTextOnFallback,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfMisplacedTextOnFallback as Error).message,).toContain('decided nothing',);
          },
        },),
        it({
          name:
            'REFUSES a decision carrying no wording, which is the one field a member of this union owns',
          fn: async () => {
            /**
             What noText raised, read for its class as well as its wording.
             */
            const refusalOfNoText = caught(function noText() {
              parseSliceOutcome({
                value: { kind: 'decided', },
                unknownKeys: 'refuse',
                path: 'outcome',
              },);
            },);

            expect(refusalOfNoText,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfNoText as Error).message,).toContain('outcome.acceptedText',);
          },
        },),
      ],
    },),

    describe({
      name: parseSliceDelivery.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reads every member, and a withdrawal`s mechanism with it',
          fn: async () => {
            expect(parseSliceDelivery({
              value: {
                kind: 'replacement-withdrawn',
                reason: 'assembly-integrity',
              },
              path: 'delivery',
            },),).toEqual({
              kind: 'replacement-withdrawn',
              reason: 'assembly-integrity',
            },);
            expect(parseSliceDelivery({
              value: { kind: 'gap-remains', },
              path: 'delivery',
            },),).toEqual({ kind: 'gap-remains', },);
          },
        },),
        it({
          name:
            'REFUSES a withdrawal whose mechanism this version does not name, and a reason on a member that '
              + 'took nothing back: both would let a reader report a withdrawal that never happened',
          fn: async () => {
            /**
             What unknownReason raised, read for its class as well as its wording.
             */
            const refusalOfUnknownReason = caught(function unknownReason() {
              parseSliceDelivery({
                value: {
                  kind: 'replacement-withdrawn',
                  reason: 'the-cat-said-no',
                },
                path: 'delivery',
              },);
            },);

            expect(refusalOfUnknownReason,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfUnknownReason as Error).message,).toContain('delivery.reason',);
            /**
             What reasonWithoutWithdrawal raised, read for its class as well as its wording.
             */
            const refusalOfReasonWithoutWithdrawal = caught(function reasonWithoutWithdrawal() {
              parseSliceDelivery({
                value: {
                  kind: 'incumbent-retained',
                  reason: 'assembly-integrity',
                },
                path: 'delivery',
              },);
            },);

            expect(refusalOfReasonWithoutWithdrawal,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfReasonWithoutWithdrawal as Error).message,).toContain('delivery.reason',);
          },
        },),
      ],
    },),

    describe({
      name: parseDecisionComparison.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reads both members, and keeps `undecidedLanes` in the order the file names them',
          fn: async () => {
            expect(parseDecisionComparison({
              value: {
                kind: 'not-comparable',
                undecidedLanes: [
                  'translate',
                  'repair',
                ],
              },
              path: 'decisionComparison',
            },),).toEqual({
              kind: 'not-comparable',
              undecidedLanes: [
                'translate',
                'repair',
              ],
            },);
            expect(parseDecisionComparison({
              value: {
                kind: 'comparable',
                verdict: 'different',
              },
              path: 'decisionComparison',
            },),).toEqual({
              kind: 'comparable',
              verdict: 'different',
            },);
          },
        },),
        it({
          name:
            'REFUSES a lane name this pipeline has no lane for, at the position that named it, since a third '
              + 'lane is a generation this reader cannot describe rather than a typo to skip past',
          fn: async () => {
            /**
             What unknownLane raised, read for its class as well as its wording.
             */
            const refusalOfUnknownLane = caught(function unknownLane() {
              parseDecisionComparison({
                value: {
                  kind: 'not-comparable',
                  undecidedLanes: [
                    'repair',
                    'proofread',
                  ],
                },
                path: 'decisionComparison',
              },);
            },);

            expect(refusalOfUnknownLane,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfUnknownLane as Error).message,).toContain('decisionComparison.undecidedLanes[1]',);
          },
        },),
        it({
          name:
            'ACCEPTS an EMPTY undecided list, which is a fact about the writer rather than about this reader: '
              + 'the relations check whether the list agrees with the two outcomes, and a parser that refused '
              + 'it here would report a shape error for a disagreement',
          fn: async () => {
            expect(parseDecisionComparison({
              value: {
                kind: 'not-comparable',
                undecidedLanes: [],
              },
              path: 'decisionComparison',
            },),).toEqual({
              kind: 'not-comparable',
              undecidedLanes: [],
            },);
          },
        },),
      ],
    },),

    describe({
      name: parseDeliveryRow.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reads a whole ledger row, unions included',
          fn: async () => {
            expect(parseDeliveryRow({
              value: {
                sliceIndex: 0,
                sourceText: '猫猫在窗台上睡觉。',
                incumbentKind: 'present',
                incumbentText: ARCHIVE_NAP,
                outcome: {
                  kind: 'decided',
                  acceptedText: ARCHIVE_NAP,
                },
                shippedText: ARCHIVE_NAP,
                delivery: { kind: 'incumbent-retained', },
              },
              path: 'lanes.repair.delivery[0]',
              keys: SLICE_SPELLED_KEYS,
            },),).toEqual({
              sliceIndex: 0,
              sourceText: '猫猫在窗台上睡觉。',
              incumbentKind: 'present',
              incumbentText: ARCHIVE_NAP,
              outcome: {
                kind: 'decided',
                acceptedText: ARCHIVE_NAP,
              },
              shippedText: ARCHIVE_NAP,
              delivery: { kind: 'incumbent-retained', },
            },);
          },
        },),
        it({
          name:
            'REFUSES a key version 2 does not name on the row, and a slice index that is not a slice index: '
              + 'a fraction, a negative, or a value past what JSON carries exactly names no slice any '
              + 'preparation produced',
          fn: async () => {
            /**
             One valid row, which each refusal this case reads breaks in exactly one way.
             */
            const row = {
              sliceIndex: 0,
              sourceText: '猫猫在窗台上睡觉。',
              incumbentKind: 'present',
              incumbentText: ARCHIVE_NAP,
              outcome: { kind: 'not-applicable', },
              shippedText: ARCHIVE_NAP,
              delivery: { kind: 'incumbent-retained', },
            };
            /**
             What extraKey raised, read for its class as well as its wording.
             */
            const refusalOfExtraKey = caught(function extraKey() {
              parseDeliveryRow({
                value: {
                  ...row,
                  whiskers: 3,
                },
                path: 'row',
                keys: SLICE_SPELLED_KEYS,
              },);
            },);

            expect(refusalOfExtraKey,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfExtraKey as Error).message,).toContain('row.whiskers',);
            expect([
              1.5,
              -1,
              Number.NaN,
              Number.POSITIVE_INFINITY,
              Number.MAX_SAFE_INTEGER + 2,
            ].map(function refusesIndex(sliceIndex,): string {
              try {
                parseDeliveryRow({
                  value: {
                    ...row,
                    sliceIndex,
                  },
                  path: 'row',
                  keys: SLICE_SPELLED_KEYS,
                },);
                return 'accepted';
              } catch (error) {
                // The NAME rather than a boolean, so a case that fails some other
                // way reads as that failure instead of as a refusal.
                return Error.isError(error,) ? error.name : 'threw a non-error';
              }
            },),).toEqual([
              'ArtifactParseError',
              'ArtifactParseError',
              'ArtifactParseError',
              'ArtifactParseError',
              'ArtifactParseError',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: parseComparisonRow.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reads a whole comparison row, all four unions and the lane relation included',
          fn: async () => {
            /**
             One row saying both lanes moved to different wordings.
             */
            const row = {
              sliceIndex: 2,
              incumbentKind: 'present',
              incumbentText: ARCHIVE_NAP,
              repairText: 'The cat is asleep on the sill.',
              translateText: 'The cat naps on the windowsill.',
              laneRelation: 'both-differ',
              repairOutcome: {
                kind: 'decided',
                acceptedText: 'The cat is asleep on the sill.',
              },
              translateOutcome: {
                kind: 'decided',
                acceptedText: 'The cat naps on the windowsill.',
              },
              decisionComparison: {
                kind: 'comparable',
                verdict: 'different',
              },
              repairDelivery: { kind: 'replacement-shipped', },
              translateDelivery: { kind: 'replacement-shipped', },
            };
            expect(parseComparisonRow({
              value: row,
              path: 'comparison[0]',
              keys: SLICE_SPELLED_KEYS,
            },),).toEqual(row,);
          },
        },),

        it({
          name:
            'READS A ROW WRITTEN UNDER THE RETIRED SPELLING, because artifacts outlive the pipelines '
              + 'that wrote them. The field was `verdict` until 2026-08-22, sharing a bare key name with '
              + '`laneSelection.slices[].verdict` at a sibling path, where it answers who WON rather than '
              + 'which lanes changed. Refusing the old spelling would have made 25 settled artifacts '
              + 'unreadable to buy a name',
          fn: async () => {
            /**
             One row as a pipeline before the rename wrote it.
             */
            const legacy = {
              sliceIndex: 2,
              incumbentKind: 'present',
              incumbentText: ARCHIVE_NAP,
              repairText: 'The cat is asleep on the sill.',
              translateText: 'The cat naps on the windowsill.',
              verdict: 'both-differ',
              repairOutcome: {
                kind: 'decided',
                acceptedText: 'The cat is asleep on the sill.',
              },
              translateOutcome: {
                kind: 'decided',
                acceptedText: 'The cat naps on the windowsill.',
              },
              decisionComparison: {
                kind: 'comparable',
                verdict: 'different',
              },
              repairDelivery: { kind: 'replacement-shipped', },
              translateDelivery: { kind: 'replacement-shipped', },
            };

            expect(parseComparisonRow({
              value: legacy,
              path: 'comparison[0]',
              keys: SLICE_SPELLED_KEYS,
            },).laneRelation,).toBe('both-differ',);
          },
        },),

        it({
          name:
            'REFUSES A ROW CARRYING BOTH SPELLINGS rather than picking one, since two names for one '
              + 'field means two pipelines wrote the row, and quietly preferring either would hide that '
              + 'from every reader downstream',
          fn: async () => {
            /**
             What bothSpellings raised, read for its class as well as its wording.
             */
            const refusalOfBothSpellings = caught(function bothSpellings() {
              parseComparisonRow({
                value: {
                  sliceIndex: 0,
                  incumbentKind: 'absent',
                  incumbentText: '',
                  repairText: '',
                  translateText: '',
                  laneRelation: 'gap-remains',
                  verdict: 'gap-remains',
                  repairOutcome: { kind: 'not-applicable', },
                  translateOutcome: { kind: 'unfilled', },
                  decisionComparison: {
                    kind: 'not-comparable',
                    undecidedLanes: [
                      'repair',
                      'translate',
                    ],
                  },
                  repairDelivery: { kind: 'gap-remains', },
                  translateDelivery: { kind: 'gap-remains', },
                },
                path: 'comparison[0]',
                keys: SLICE_SPELLED_KEYS,
              },);
            },);

            expect(refusalOfBothSpellings,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfBothSpellings as Error).message,).toContain('verdict',);
          },
        },),
        it({
          name:
            'REFUSES a lane relation this version does not name, so a generation that added one cannot '
              + 'be read as though its rows meant what these do',
          fn: async () => {
            /**
             What unknownVerdict raised, read for its class as well as its wording.
             */
            const refusalOfUnknownVerdict = caught(function unknownVerdict() {
              parseComparisonRow({
                value: {
                  sliceIndex: 0,
                  incumbentKind: 'absent',
                  incumbentText: '',
                  repairText: '',
                  translateText: '',
                  verdict: 'both-napped',
                  repairOutcome: { kind: 'not-applicable', },
                  translateOutcome: { kind: 'unfilled', },
                  decisionComparison: {
                    kind: 'not-comparable',
                    undecidedLanes: [
                      'repair',
                      'translate',
                    ],
                  },
                  repairDelivery: { kind: 'gap-remains', },
                  translateDelivery: { kind: 'gap-remains', },
                },
                path: 'comparison[0]',
                keys: SLICE_SPELLED_KEYS,
              },);
            },);

            expect(refusalOfUnknownVerdict,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfUnknownVerdict as Error).message,).toContain('comparison[0].verdict',);
          },
        },),
      ],
    },),

    describe({
      name: parseEvidenceRow.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'takes the four fields version 2 checks out of a raw slice row and LEAVES THE REST, so a lane '
              + 'that records more evidence next month does not make today`s artifacts unreadable',
          fn: async () => {
            expect(parseEvidenceRow({
              value: {
                sliceIndex: 1,
                incumbentKind: 'present',
                incumbentText: ARCHIVE_NAP,
                outcome: {
                  kind: 'decided',
                  acceptedText: ARCHIVE_NAP,
                },
                ballots: [{ voter: 'Tabby', },],
                elapsedMs: 12,
              },
              path: 'lanes.repair.result.sliceTexts[1]',
              keys: SLICE_SPELLED_KEYS,
            },),).toEqual({
              sliceIndex: 1,
              incumbentKind: 'present',
              incumbentText: ARCHIVE_NAP,
              outcome: {
                kind: 'decided',
                acceptedText: ARCHIVE_NAP,
              },
            },);
          },
        },),
        it({
          name:
            'still REQUIRES the four, since they are what the ledger is checked against: a raw row missing '
              + 'one of them leaves the check with nothing to compare and would pass by having less',
          fn: async () => {
            /**
             What noIncumbentKind raised, read for its class as well as its wording.
             */
            const refusalOfNoIncumbentKind = caught(function noIncumbentKind() {
              parseEvidenceRow({
                value: {
                  sliceIndex: 1,
                  incumbentText: ARCHIVE_NAP,
                  outcome: { kind: 'unfilled', },
                },
                path: 'sliceTexts[1]',
                keys: SLICE_SPELLED_KEYS,
              },);
            },);

            expect(refusalOfNoIncumbentKind,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfNoIncumbentKind as Error).message,).toContain('sliceTexts[1].incumbentKind',);
          },
        },),
      ],
    },),
  ],
},);
