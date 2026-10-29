/**
 Tests for the three assertions that decide whether a settled artifact is
 describing the run it claims to describe.

 THESE ARE THE GUARDS BETWEEN A RUN AND EVERY NUMBER READ OFF IT. The lane
 comparison already refuses two ledgers naming different preparations, and that
 refusal says nothing about whether either names the RIGHT one: two ledgers
 built over some other slicing agree with each other perfectly. These three run
 at the only boundary holding the preparation, both ledgers and both results at
 once, so a guard here that never fires means an artifact whose recorded
 identity names a slicing the lanes never ran over is filed as sound, and every
 later reading of it is confidently wrong.

 A PAST DEFECT IS WHY THEY ARE WORTH TESTING BY HAND. There the defect was the OPEN
 rather than the message: the guard's wording was right and the condition never
 fired, so every test that read the message passed while nothing was guarded.
 Each case in this file drives one refusal and changes exactly one thing from a
 fixture that passes, so a condition that stopped firing fails its own case
 rather than hiding behind a neighbour's.

 THE INSERTION SLICE IS LOAD-BEARING IN THE FIXTURE. `incumbentKind` is decided
 by `isInsertionChunk`, so a preparation of ordinary slices only can never
 exercise the `absent` side, and the case about it would pass against a guard
 that had been deleted. The fixture therefore carries one slice of each kind.

 Fixtures are cat-themed invention: Simplified Chinese against English.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { makeInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { IdentifiedDeliveryLedger, } from '@monochromatic-dev/translation-repair-lane-contest';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  assertPreparationIdentity,
  type PreparationIdentity,
  type PreparedDocumentPair,
} from '@monochromatic-dev/translation-repair-corpus-source';
import {
  ArtifactPreparationMismatchError,
  assertFindingsDescribePreparation,
  assertLedgerDescribesPreparation,
  assertResultCountsPreparation,
} from '../../dist/final/node/index.mjs';

//region Artifact two-lane verification tests

/**
 Brands a string as a preparation identity through the real guard.

 THROUGH THE GUARD RATHER THAN A CAST, so a fixture cannot quietly carry a
 shape the pipeline would refuse and pass cases that production would not.

 @param text - identity as a preparation stamps one

 @returns Same string, branded

 @throws {@link Error} when the fixture itself is not a valid identity

 @example
 ```ts
 const identity = identityOf('sha256-preparation-v1:...',);
 ```
 */
function identityOf(text: string,): PreparationIdentity {
  assertPreparationIdentity(text,);
  return text;
}

/**
 Name the preparation under test gives itself.
 */
const EXPECTED = identityOf(`sha256-preparation-v1:${'a7'.repeat(32,)}`,);

/**
 Name of some OTHER slicing, for the case about filing a ledger under the
 wrong one.
 */
const OTHER_IDENTITY = identityOf(`sha256-preparation-v1:${'b4'.repeat(32,)}`,);

/**
 Original of the slice the archive already rendered.
 */
const SOURCE_ONE = '猫坐在垫子上。';

/**
 Archive wording at that slice.
 */
const ARCHIVE_ONE = 'The cat sat on the mat.';

/**
 Original of the slice the archive never rendered.
 */
const SOURCE_TWO = '小猫在楼梯上看着。';

/**
 First finding the preparation observed, named so a case can keep it while
 changing the one beside it.
 */
const FIRST_FINDING = 'alignment structure-mismatch';

/**
 Alignment findings the preparation observed.
 */
const FINDINGS: readonly string[] = [
  FIRST_FINDING,
  'sections-merged 2',
];

/**
 Builds a content chunk, which is what an ordinary slice carries.

 @param sliceIndex - position within the document

 @param text - wording at that position

 @returns Chunk shaped as preparation produces one

 @example
 ```ts
 const chunk = contentChunk({ sliceIndex: 0, text: ARCHIVE_ONE, },);
 ```
 */
function contentChunk(
  {
    sliceIndex,
    text,
  }: {
    readonly sliceIndex: number;
    readonly text: string;
  },
): ChunkPair['source'] {
  return {
    sliceIndex,
    nodes: [],
    startOffset: 0,
    endOffset: text.length,
    text,
  };
}

/**
 Preparation every case measures a ledger against.

 Two slices: one the archive rendered, one it did not. The second is an
 insertion chunk, which is the only way `incumbentKind` can read `absent`.
 */
const PREPARED: PreparedDocumentPair = {
  sourceText: `${SOURCE_ONE}\n\n${SOURCE_TWO}`,
  targetText: ARCHIVE_ONE,
  slices: [
    {
      source: contentChunk({
        sliceIndex: 0,
        text: SOURCE_ONE,
      },),
      target: contentChunk({
        sliceIndex: 0,
        text: ARCHIVE_ONE,
      },),
    },
    {
      source: contentChunk({
        sliceIndex: 1,
        text: SOURCE_TWO,
      },),
      target: makeInsertionChunk({
        sliceIndex: 1,
        offset: ARCHIVE_ONE.length,
      },),
    },
  ],
  lineStructuredSliceIndices: new Set<number>(),
  declaredNames: [],
  alignmentFindings: FINDINGS,
  unclaimedTargetBlocks: [],
  alignmentPairCount: 2,
};

/**
 One row of a ledger, with every field the assertion joins on.
 */
type FixtureRow = {
  readonly sliceIndex: number;
  readonly sourceText: string;
  readonly incumbentKind: 'present' | 'absent';
  readonly incumbentText: string;
};

/**
 Row for the slice the archive rendered.

 NAMED rather than indexed out of the list, because the cases in this file rebuild the
 list with one field changed and indexing it back out would need `!`.
 */
const ROW_ONE: FixtureRow = {
  sliceIndex: 0,
  sourceText: SOURCE_ONE,
  incumbentKind: 'present',
  incumbentText: ARCHIVE_ONE,
};

/**
 Row for the slice the archive never rendered.
 */
const ROW_TWO: FixtureRow = {
  sliceIndex: 1,
  sourceText: SOURCE_TWO,
  incumbentKind: 'absent',
  incumbentText: '',
};

/**
 Rows a ledger built over {@link PREPARED} carries.
 */
const MATCHING_ROWS: readonly FixtureRow[] = [
  ROW_ONE,
  ROW_TWO,
];

/**
 Builds a ledger from rows, filling the fields the assertion does not read.

 @param rows - per-slice facts, which is all these assertions join on

 @param identity - slicing the ledger claims to have been built over

 @returns Ledger shaped as a lane driver returns one

 @example
 ```ts
 const ledger = ledgerOf({ rows: MATCHING_ROWS, identity: EXPECTED, },);
 ```
 */
function ledgerOf(
  {
    rows,
    identity,
  }: {
    readonly rows: readonly FixtureRow[];
    readonly identity: PreparationIdentity;
  },
): IdentifiedDeliveryLedger {
  return {
    preparationIdentity: identity,
    records: rows.map(function toRecord(row,) {
      return {
        ...row,
        outcome: {
          kind: 'decided' as const,
          acceptedText: row.incumbentText,
        },
        shippedText: row.incumbentText,
        delivery: { kind: 'incumbent-retained' as const, },
      };
    },),
  };
}

/**
 Runs the ledger assertion over rows that differ from the matching set in
 exactly one way.

 @param rows - rows to check, matching unless a case changed one

 @param identity - name the ledger claims, the expected one unless changed

 @throws {@link ArtifactPreparationMismatchError} when anything disagrees

 @example
 ```ts
 checking({ rows: MATCHING_ROWS, },);
 ```
 */
function checking(
  {
    rows = MATCHING_ROWS,
    identity = EXPECTED,
  }: {
    readonly rows?: readonly FixtureRow[];
    readonly identity?: PreparationIdentity;
  },
): void {
  assertLedgerDescribesPreparation({
    prepared: PREPARED,
    expected: EXPECTED,
    ledger: ledgerOf({
      rows,
      identity,
    },),
    lane: 'repair',
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: assertLedgerDescribesPreparation.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS a ledger whose rows describe this preparation, so the refusal cases in this file are '
            + 'about what they change and not about the fixture',
          fn: async () => {
            expect(() => {
              checking({},);
            },).not.toThrow();
          },
        },),
        it({
          name: 'REFUSES a ledger built over a DIFFERENT slicing, which is the check the lane '
            + 'comparison cannot make: two ledgers over the wrong preparation agree perfectly',
          fn: async () => {
            expect(() => {
              checking({ identity: OTHER_IDENTITY, },);
            },).toThrow(ArtifactPreparationMismatchError,);
          },
        },),
        it({
          name: 'REFUSES a ledger with fewer rows than the preparation has slices',
          fn: async () => {
            expect(() => {
              checking({ rows: MATCHING_ROWS.slice(0, 1,), },);
            },).toThrow('has 1 row for a preparation of',);
          },
        },),
        it({
          name: 'REFUSES a row naming a slice the preparation does not have at that position, '
            + 'which is the hash collision this exists to turn into a refusal',
          fn: async () => {
            expect(() => {
              checking({
                rows: [
                  {
                    ...ROW_ONE,
                    sliceIndex: 7,
                  },
                  ROW_TWO,
                ],
              },);
            },).toThrow('names slice 7 at position 0',);
          },
        },),
        it({
          name: 'REFUSES a row rendering an ORIGINAL the preparation does not carry there',
          fn: async () => {
            expect(() => {
              checking({
                rows: [
                  {
                    ...ROW_ONE,
                    sourceText: '狗在门口叫。',
                  },
                  ROW_TWO,
                ],
              },);
            },).toThrow('renders an original the preparation does not carry there',);
          },
        },),
        it({
          name: 'REFUSES a row calling the archive PRESENT where the preparation calls it absent, '
            + 'which only an insertion slice in the fixture can reach',
          fn: async () => {
            // Slice 1 is an insertion: the archive never rendered it. A ledger
            // claiming wording stands there would have the lane repairing text
            // nobody wrote.
            expect(() => {
              checking({
                rows: [
                  ROW_ONE,
                  {
                    ...ROW_TWO,
                    incumbentKind: 'present',
                  },
                ],
              },);
            },).toThrow('calls the archive wording present where the preparation calls it absent',);
          },
        },),
        it({
          name: 'REFUSES a row carrying archive WORDING the preparation does not have there',
          fn: async () => {
            expect(() => {
              checking({
                rows: [
                  {
                    ...ROW_ONE,
                    incumbentText: 'The cat sat on the rug.',
                  },
                  ROW_TWO,
                ],
              },);
            },).toThrow('carries archive wording the preparation does not have there',);
          },
        },),
        it({
          name: 'NAMES THE LANE at fault, so an operator is not left checking both sides',
          fn: async () => {
            expect(() => {
              assertLedgerDescribesPreparation({
                prepared: PREPARED,
                expected: EXPECTED,
                ledger: ledgerOf({
                  rows: MATCHING_ROWS,
                  identity: OTHER_IDENTITY,
                },),
                lane: 'translate',
              },);
            },).toThrow('translate ledger was built over',);
          },
        },),
      ],
    },),

    describe({
      name: assertResultCountsPreparation.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS a result counting the slices the preparation produced',
          fn: async () => {
            expect(() => {
              assertResultCountsPreparation({
                prepared: PREPARED,
                sliceCount: PREPARED.slices.length,
                lane: 'repair',
              },);
            },).not.toThrow();
          },
        },),
        it({
          name: 'REFUSES a result counting slices the preparation does not have, which is how a '
            + 'result from another entry paired with this ledger is caught',
          fn: async () => {
            // The ledgers are checked row by row and the raw results beside them are
            // not, so this one cheap field is what stands between a grossly
            // mismatched pairing and an artifact that reads as sound.
            expect(() => {
              assertResultCountsPreparation({
                prepared: PREPARED,
                sliceCount: PREPARED.slices.length + 1,
                lane: 'translate',
              },);
            },).toThrow('translate result counts 3 slices where the preparation has 2',);
          },
        },),
      ],
    },),

    describe({
      name: assertFindingsDescribePreparation.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS findings identical to the preparation\'s own',
          fn: async () => {
            expect(() => {
              assertFindingsDescribePreparation({
                prepared: PREPARED,
                reported: FINDINGS,
              },);
            },).not.toThrow();
          },
        },),
        it({
          name: 'REFUSES a run reporting a different NUMBER of alignment findings',
          fn: async () => {
            expect(() => {
              assertFindingsDescribePreparation({
                prepared: PREPARED,
                reported: FINDINGS.slice(0, 1,),
              },);
            },).toThrow('reports 1 alignment finding for a preparation with 2',);
          },
        },),
        it({
          name: 'REFUSES a run reporting the same COUNT of findings with different wording, which '
            + 'a length check alone would pass',
          fn: async () => {
            // Two derivations of one fact reach this boundary. Recording either
            // silently is picking one, and the count matching is exactly the case
            // where picking wrong is invisible.
            expect(() => {
              assertFindingsDescribePreparation({
                prepared: PREPARED,
                reported: [
                  FIRST_FINDING,
                  'sections-merged 9',
                ],
              },);
            },).toThrow('alignment finding 1 is not what the preparation observed there',);
          },
        },),
      ],
    },),
  ],
},);

//endregion Artifact two-lane verification tests
