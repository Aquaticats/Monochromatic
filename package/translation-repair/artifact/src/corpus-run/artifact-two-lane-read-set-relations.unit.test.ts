/**
 Tests for what has to hold between a lane's index sets, counts, status and
 the ledger rows those describe.

 THESE ARE ALL REFUSAL BRANCHES, and a valid artifact reaches none of them.
 `parseSettledTwoLaneArtifact` calls every one of these on its way through a file,
 so the suite already ran them thousands of times, always down the arm where
 nothing was wrong. That is coverage of the caller and not of these: the
 question each answers is what it does with a lane whose two derivations
 disagree, and no valid fixture can ask it.

 WHY ORDER RATHER THAN MEMBERSHIP. Both contracts say the index lists are in
 document order, so an equal-length list in another order is a lane whose two
 derivations disagree, and a check that read them as sets would call that
 agreement. Two cases pin the ordering, and one pins a repeated index, which
 a set would also have swallowed.

 WHY BLOCKED COMPATIBILITY RUNS ONE WAY. A blocked run and an unblocked one
 produce the same ledger whenever no slice decided anything different from
 the archive, so the status is not derivable from the rows and a check that
 recomputed it would refuse valid artifacts. Three cases accept ledgers that
 a recomputation would have rejected.

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

import {
  type ArtifactDeliveryRow,
  ArtifactParseError,
  type ArtifactSliceDelivery,
  type ArtifactSliceOutcome,
} from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  type ArtifactEvidenceRow,
  type ArtifactRepairEvidence,
  type ArtifactTranslateEvidence,
  assertBlockedCompatible,
  assertIndexSetsMatchLedger,
  assertTranslateCountsAgree,
} from '../../dist/final/node/index.mjs';

/**
 Dotted path the cases hand in, standing for one lane's raw result.
 */
const LANE_PATH = 'lanes.repair.result';

/**
 Original of the slice every row here is about.
 */
const SOURCE_SILL = '猫猫在窗台上睡觉。';

/**
 Archive's own English for it.
 */
const ARCHIVE_SILL = 'The cat sleeps on the sill.';

/**
 Wording a lane decided on, where a row ships one.
 */
const DECIDED_SILL = 'The cat naps on the windowsill.';

/**
 Builds one ledger row, which is the only thing the index sets are derived
 from.

 The four fields beyond `sliceIndex` and `delivery` are held constant on
 purpose: nothing in this file reads them, and a case that varied them would
 suggest they mattered here.

 @param sliceIndex - global slice index this row is for

 @param delivery - how the document came to carry what it carries

 @returns Row a lane's ledger would hold

 @example
 ```ts
 const row = row({ sliceIndex: 0, delivery: { kind: 'replacement-shipped', }, },);
 ```
 */
function row(
  {
    sliceIndex,
    delivery,
  }: {
    readonly sliceIndex: number;
    readonly delivery: ArtifactSliceDelivery;
  },
): ArtifactDeliveryRow {
  return {
    sliceIndex,
    sourceText: SOURCE_SILL,
    incumbentKind: 'present',
    incumbentText: ARCHIVE_SILL,
    outcome: {
      kind: 'decided',
      acceptedText: DECIDED_SILL,
    },
    shippedText: (delivery.kind === 'replacement-shipped')
      ? DECIDED_SILL
      : ARCHIVE_SILL,
    delivery,
  };
}

/**
 Ledger four slices long, carrying one of each delivery the sets read.

 TWO SHIPPED SLICES RATHER THAN ONE, so a case can reorder the changed list
 without changing what is in it.

 @returns Rows in document order

 @example
 ```ts
 const ledger = mixedLedger();
 ```
 */
function mixedLedger(): readonly ArtifactDeliveryRow[] {
  return [
    row({
      sliceIndex: 0,
      delivery: { kind: 'replacement-shipped', },
    },),
    row({
      sliceIndex: 1,
      delivery: {
        kind: 'replacement-withdrawn',
        reason: 'assembly-integrity',
      },
    },),
    row({
      sliceIndex: 2,
      delivery: { kind: 'incumbent-retained', },
    },),
    row({
      sliceIndex: 3,
      delivery: { kind: 'replacement-shipped', },
    },),
  ];
}

/**
 Repair evidence agreeing with `mixedLedger` in every part these checks read.

 @param status - how the run ended, which only the compatibility check reads

 @param sliceCount - slices the preparation produced

 @param changedSliceIndices - slices the document carries a repair for

 @param withdrawnSliceIndices - slices the assembly guard took a repair back at

 @param sliceTexts - what the lane decided per slice, which neither check
 here reads and which is carried only so the fixture does not contradict its
 own slice count

 @returns Evidence a valid repair lane would carry

 @example
 ```ts
 const evidence = repairEvidence({ sliceCount: 3, },);
 ```
 */
function repairEvidence(
  {
    status = 'repaired',
    sliceCount = 4,
    changedSliceIndices = [
      0,
      3,
    ],
    withdrawnSliceIndices = [1,],
    sliceTexts = decidedRows(),
  }: {
    readonly status?: ArtifactRepairEvidence['status'];
    readonly sliceCount?: number;
    readonly changedSliceIndices?: readonly number[];
    readonly withdrawnSliceIndices?: readonly number[];
    readonly sliceTexts?: readonly ArtifactEvidenceRow[];
  } = {},
): ArtifactRepairEvidence {
  return {
    status,
    sliceCount,
    changedSliceIndices,
    withdrawnSliceIndices,
    sliceTexts,
  };
}

/**
 One evidence row, which only the translate status check reads.

 @param sliceIndex - global slice index this row is for

 @param outcome - what the lane did about it

 @returns Row a lane's raw result would carry

 @example
 ```ts
 const seen = evidenceRow({ sliceIndex: 0, outcome: { kind: 'unfilled', }, },);
 ```
 */
function evidenceRow(
  {
    sliceIndex,
    outcome,
  }: {
    readonly sliceIndex: number;
    readonly outcome: ArtifactSliceOutcome;
  },
): ArtifactEvidenceRow {
  return {
    sliceIndex,
    incumbentKind: 'present',
    incumbentText: ARCHIVE_SILL,
    outcome,
  };
}

/**
 Four evidence rows, none of them a slice the lane reached and could not
 fill, which is the only outcome the translate status reads.

 @returns Rows in document order

 @example
 ```ts
 const sliceTexts = decidedRows();
 ```
 */
function decidedRows(): readonly ArtifactEvidenceRow[] {
  return [
    evidenceRow({
      sliceIndex: 0,
      outcome: {
        kind: 'decided',
        acceptedText: DECIDED_SILL,
      },
    },),
    evidenceRow({
      sliceIndex: 1,
      outcome: {
        kind: 'decided',
        acceptedText: DECIDED_SILL,
      },
    },),
    evidenceRow({
      sliceIndex: 2,
      outcome: { kind: 'incumbent-fallback', },
    },),
    evidenceRow({
      sliceIndex: 3,
      outcome: {
        kind: 'decided',
        acceptedText: DECIDED_SILL,
      },
    },),
  ];
}

/**
 Translate evidence whose counts, lists and status all agree.

 @param status - whether the document is a whole translation

 @param sliceCount - slices the preparation produced

 @param changedSliceCount - second statement of how many slices shipped

 @param withdrawnSliceCount - second statement of how many were withdrawn

 @param changedSliceIndices - slices whose accepted text shipped

 @param withdrawnSliceIndices - slices the assembly guard took back

 @param sliceTexts - what the lane decided per slice, which the status is
 checked against

 @returns Evidence a valid translate lane would carry

 @example
 ```ts
 const evidence = translateEvidence({ status: 'unfilled', },);
 ```
 */
function translateEvidence(
  {
    status = 'complete',
    sliceCount = 4,
    changedSliceCount = 2,
    withdrawnSliceCount = 1,
    changedSliceIndices = [
      0,
      3,
    ],
    withdrawnSliceIndices = [1,],
    sliceTexts = decidedRows(),
  }: {
    readonly status?: ArtifactTranslateEvidence['status'];
    readonly sliceCount?: number;
    readonly changedSliceCount?: number;
    readonly withdrawnSliceCount?: number;
    readonly changedSliceIndices?: readonly number[];
    readonly withdrawnSliceIndices?: readonly number[];
    readonly sliceTexts?: readonly ArtifactEvidenceRow[];
  } = {},
): ArtifactTranslateEvidence {
  return {
    status,
    sliceCount,
    changedSliceCount,
    withdrawnSliceCount,
    changedSliceIndices,
    withdrawnSliceIndices,
    sliceTexts,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: assertIndexSetsMatchLedger.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS a lane whose two lists are exactly what its ledger rows '
            + 'produce, which is the control every refusal case here is a '
            + 'one-field departure from',
          fn: async () => {
            expect(function acceptAgreeing() {
              assertIndexSetsMatchLedger({
                evidence: repairEvidence(),
                ledger: mixedLedger(),
                path: LANE_PATH,
              },);
            },).not.toThrow();
          },
        },),

        it({
          name: 'REFUSES a changed list shorter than the shipped rows, naming how '
            + 'many the ledger holds rather than how many were recorded',
          fn: async () => {
            const refusalOfShortList = caught(function shortList() {
              assertIndexSetsMatchLedger({
                evidence: repairEvidence({ changedSliceIndices: [0,], },),
                ledger: mixedLedger(),
                path: LANE_PATH,
              },);
            },);

            expect(refusalOfShortList,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfShortList as Error).message,)
              .toContain('lanes.repair.result.changedSliceIndices',);
            expect((refusalOfShortList as Error).message,).toContain('2 slices',);
          },
        },),

        it({
          name: 'REFUSES a changed list holding the right indices in the wrong '
            + 'order, which is the case a membership check would have called '
            + 'agreement. Both contracts say document order, so a lane that '
            + 'recorded them another way derived them another way',
          fn: async () => {
            const refusalOfReordered = caught(function reordered() {
              assertIndexSetsMatchLedger({
                evidence: repairEvidence({
                  changedSliceIndices: [
                    3,
                    0,
                  ],
                },),
                ledger: mixedLedger(),
                path: LANE_PATH,
              },);
            },);

            expect(refusalOfReordered,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfReordered as Error).message,)
              .toContain('lanes.repair.result.changedSliceIndices[0]',);
            expect((refusalOfReordered as Error).message,).toContain('slice 0',);
          },
        },),

        it({
          name: 'REFUSES a changed list naming one slice twice, which a set would '
            + 'have collapsed into a list of the right length',
          fn: async () => {
            const refusalOfRepeat = caught(function repeat() {
              assertIndexSetsMatchLedger({
                evidence: repairEvidence({
                  changedSliceIndices: [
                    0,
                    0,
                  ],
                },),
                ledger: mixedLedger(),
                path: LANE_PATH,
              },);
            },);

            expect(refusalOfRepeat,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfRepeat as Error).message,)
              .toContain('lanes.repair.result.changedSliceIndices[1]',);
            expect((refusalOfRepeat as Error).message,).toContain('slice 3',);
          },
        },),

        it({
          name: 'ACCEPTS a withdrawn list that leaves out a whole-document '
            + 'refusal, since the withdrawn set names slices the assembly guard '
            + 'took back and a blocked run never assembled anything',
          fn: async () => {
            expect(function acceptOmittedRefusal() {
              assertIndexSetsMatchLedger({
                evidence: repairEvidence({
                  status: 'blocked-non-translation',
                  sliceCount: 2,
                  changedSliceIndices: [],
                  withdrawnSliceIndices: [],
                },),
                ledger: [
                  row({
                    sliceIndex: 0,
                    delivery: {
                      kind: 'replacement-withdrawn',
                      reason: 'blocked-non-translation',
                    },
                  },),
                  row({
                    sliceIndex: 1,
                    delivery: { kind: 'incumbent-retained', },
                  },),
                ],
                path: LANE_PATH,
              },);
            },).not.toThrow();
          },
        },),

        it({
          name: 'REFUSES a withdrawn list that counts a whole-document refusal as '
            + 'a guard withdrawal, which would make every blocked run look like a '
            + 'document the guard tore apart',
          fn: async () => {
            const refusalOfCountedRefusal = caught(function countedRefusal() {
              assertIndexSetsMatchLedger({
                evidence: repairEvidence({
                  status: 'blocked-non-translation',
                  sliceCount: 1,
                  changedSliceIndices: [],
                  withdrawnSliceIndices: [0,],
                },),
                ledger: [
                  row({
                    sliceIndex: 0,
                    delivery: {
                      kind: 'replacement-withdrawn',
                      reason: 'blocked-non-translation',
                    },
                  },),
                ],
                path: LANE_PATH,
              },);
            },);

            expect(refusalOfCountedRefusal,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfCountedRefusal as Error).message,)
              .toContain('lanes.repair.result.withdrawnSliceIndices',);
            expect((refusalOfCountedRefusal as Error).message,)
              .toContain('0 slices',);
          },
        },),

        it({
          name: 'REFUSES a slice count that disagrees with how many rows the '
            + 'ledger holds, which is the third statement of the same fact and '
            + 'the one every index in the lists is out of',
          fn: async () => {
            const refusalOfWrongCount = caught(function wrongCount() {
              assertIndexSetsMatchLedger({
                evidence: repairEvidence({ sliceCount: 5, },),
                ledger: mixedLedger(),
                path: LANE_PATH,
              },);
            },);

            expect(refusalOfWrongCount,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfWrongCount as Error).message,)
              .toContain('lanes.repair.result.sliceCount',);
            expect((refusalOfWrongCount as Error).message,).toContain('4 slices',);
          },
        },),
      ],
    },),

    describe({
      name: assertBlockedCompatible.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS a blocked lane whose every slice kept the archive, which '
            + 'is the ledger an unblocked run that changed nothing also produces. '
            + 'This is why the status is checked for compatibility rather than '
            + 'recomputed',
          fn: async () => {
            expect(function acceptQuietBlocked() {
              assertBlockedCompatible({
                evidence: repairEvidence({
                  status: 'blocked-non-translation',
                  sliceCount: 2,
                  changedSliceIndices: [],
                  withdrawnSliceIndices: [],
                },),
                ledger: [
                  row({
                    sliceIndex: 0,
                    delivery: { kind: 'incumbent-retained', },
                  },),
                  row({
                    sliceIndex: 1,
                    delivery: { kind: 'gap-remains', },
                  },),
                ],
                path: LANE_PATH,
              },);
            },).not.toThrow();
          },
        },),

        it({
          name: 'REFUSES a blocked lane whose ledger ships a replacement, since '
            + 'nothing was assembled by a blocked run and so nothing of it shipped',
          fn: async () => {
            const refusalOfBlockedShipping = caught(function blockedShipping() {
              assertBlockedCompatible({
                evidence: repairEvidence({ status: 'blocked-non-translation', },),
                ledger: mixedLedger(),
                path: LANE_PATH,
              },);
            },);

            expect(refusalOfBlockedShipping,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfBlockedShipping as Error).message,)
              .toContain('lanes.repair.result.status',);
            expect((refusalOfBlockedShipping as Error).message,)
              .toContain('slice 0 reports assembly having run',);
          },
        },),

        it({
          name: 'REFUSES a blocked lane whose ledger records a guard withdrawal, '
            + 'because the guard only ever ran on a document that was assembled',
          fn: async () => {
            const refusalOfBlockedGuard = caught(function blockedGuard() {
              assertBlockedCompatible({
                evidence: repairEvidence({
                  status: 'blocked-non-translation',
                  sliceCount: 1,
                  changedSliceIndices: [],
                  withdrawnSliceIndices: [7,],
                },),
                ledger: [
                  row({
                    sliceIndex: 7,
                    delivery: {
                      kind: 'replacement-withdrawn',
                      reason: 'assembly-integrity',
                    },
                  },),
                ],
                path: LANE_PATH,
              },);
            },);

            expect(refusalOfBlockedGuard,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfBlockedGuard as Error).message,)
              .toContain('slice 7 reports assembly having run',);
          },
        },),

        it({
          name: 'ACCEPTS a blocked lane whose ledger withdraws by whole-document '
            + 'refusal, which is the withdrawal a blocked run is supposed to leave',
          fn: async () => {
            expect(function acceptBlockedRefusal() {
              assertBlockedCompatible({
                evidence: repairEvidence({
                  status: 'blocked-non-translation',
                  sliceCount: 1,
                  changedSliceIndices: [],
                  withdrawnSliceIndices: [],
                },),
                ledger: [
                  row({
                    sliceIndex: 0,
                    delivery: {
                      kind: 'replacement-withdrawn',
                      reason: 'blocked-non-translation',
                    },
                  },),
                ],
                path: LANE_PATH,
              },);
            },).not.toThrow();
          },
        },),

        it({
          name: 'REFUSES an unblocked lane whose ledger withdraws by '
            + 'whole-document refusal, which only a blocked run produces',
          fn: async () => {
            const refusalOfStrayRefusal = caught(function strayRefusal() {
              assertBlockedCompatible({
                evidence: repairEvidence({
                  status: 'unchanged',
                  sliceCount: 1,
                  changedSliceIndices: [],
                  withdrawnSliceIndices: [],
                },),
                ledger: [
                  row({
                    sliceIndex: 0,
                    delivery: {
                      kind: 'replacement-withdrawn',
                      reason: 'blocked-non-translation',
                    },
                  },),
                ],
                path: LANE_PATH,
              },);
            },);

            expect(refusalOfStrayRefusal,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfStrayRefusal as Error).message,)
              .toContain('lanes.repair.result.status',);
            expect((refusalOfStrayRefusal as Error).message,)
              .toContain('slice 0 reports a withdrawal by whole-document refusal',);
          },
        },),

        it({
          name: 'ACCEPTS an unblocked lane carrying both a shipped replacement '
            + 'and a guard withdrawal, which is what an ordinary repaired '
            + 'document looks like',
          fn: async () => {
            expect(function acceptOrdinaryRepair() {
              assertBlockedCompatible({
                evidence: repairEvidence(),
                ledger: mixedLedger(),
                path: LANE_PATH,
              },);
            },).not.toThrow();
          },
        },),
      ],
    },),

    describe({
      name: assertTranslateCountsAgree.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS a lane whose two counts equal the lists beside them and '
            + 'whose status matches what its slices record',
          fn: async () => {
            expect(function acceptAgreeing() {
              assertTranslateCountsAgree({
                evidence: translateEvidence(),
                path: LANE_PATH,
              },);
            },).not.toThrow();
          },
        },),

        it({
          name: 'REFUSES a changed count that disagrees with the changed list, '
            + 'naming how many slices the list holds',
          fn: async () => {
            const refusalOfWrongChanged = caught(function wrongChanged() {
              assertTranslateCountsAgree({
                evidence: translateEvidence({ changedSliceCount: 1, },),
                path: LANE_PATH,
              },);
            },);

            expect(refusalOfWrongChanged,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfWrongChanged as Error).message,)
              .toContain('lanes.repair.result.changedSliceCount',);
            expect((refusalOfWrongChanged as Error).message,)
              .toContain('expected 2,',);
          },
        },),

        it({
          name: 'REFUSES a withdrawn count that disagrees with the withdrawn list',
          fn: async () => {
            const refusalOfWrongWithdrawn = caught(function wrongWithdrawn() {
              assertTranslateCountsAgree({
                evidence: translateEvidence({ withdrawnSliceCount: 0, },),
                path: LANE_PATH,
              },);
            },);

            expect(refusalOfWrongWithdrawn,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfWrongWithdrawn as Error).message,)
              .toContain('lanes.repair.result.withdrawnSliceCount',);
            expect((refusalOfWrongWithdrawn as Error).message,)
              .toContain('expected 1,',);
          },
        },),

        it({
          name: 'REFUSES a lane calling itself complete while one slice it '
            + 'reached went unfilled, which is a document with a hole in it '
            + 'reported as a whole translation',
          fn: async () => {
            const refusalOfCompleteHole = caught(function completeHole() {
              assertTranslateCountsAgree({
                evidence: translateEvidence({
                  sliceTexts: [
                    evidenceRow({
                      sliceIndex: 0,
                      outcome: { kind: 'unfilled', },
                    },),
                  ],
                },),
                path: LANE_PATH,
              },);
            },);

            expect(refusalOfCompleteHole,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfCompleteHole as Error).message,)
              .toContain('lanes.repair.result.status',);
            expect((refusalOfCompleteHole as Error).message,)
              .toContain('expected unfilled,',);
          },
        },),

        it({
          name: 'REFUSES a lane calling itself unfilled while every slice it '
            + 'reached was filled, which understates a document that is whole',
          fn: async () => {
            const refusalOfIdleUnfilled = caught(function idleUnfilled() {
              assertTranslateCountsAgree({
                evidence: translateEvidence({ status: 'unfilled', },),
                path: LANE_PATH,
              },);
            },);

            expect(refusalOfIdleUnfilled,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfIdleUnfilled as Error).message,)
              .toContain('lanes.repair.result.status',);
            expect((refusalOfIdleUnfilled as Error).message,)
              .toContain('expected complete,',);
          },
        },),

        it({
          name: 'ACCEPTS a complete lane holding a slice it never evaluated, '
            + 'since only a slice reached and left unfilled makes a hole and a '
            + 'slice left alone keeps the archive wording',
          fn: async () => {
            expect(function acceptNotEvaluated() {
              assertTranslateCountsAgree({
                evidence: translateEvidence({
                  sliceTexts: [
                    evidenceRow({
                      sliceIndex: 0,
                      outcome: { kind: 'not-evaluated', },
                    },),
                    evidenceRow({
                      sliceIndex: 1,
                      outcome: { kind: 'not-applicable', },
                    },),
                  ],
                },),
                path: LANE_PATH,
              },);
            },).not.toThrow();
          },
        },),
      ],
    },),
  ],
},);
