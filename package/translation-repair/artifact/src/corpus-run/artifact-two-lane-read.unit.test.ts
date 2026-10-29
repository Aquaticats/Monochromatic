/**
 Tests for reading one whole version 2 artifact.

 EVERY CASE BREAKS ONE VALID ARTIFACT IN EXACTLY ONE WAY, and the artifact's
 comparison is derived by version 2's own frozen rules rather than typed out,
 so a case that changes a ledger gets the comparison that follows from it
 without anyone hand-maintaining a second copy of the rules.

 WHAT THEY PIN, beyond shape: the checks that catch a file whose parts
 contradict each other. Every field here parses on its own in each of these
 cases; what fails is a relation between two of them, which is the whole
 reason a reader recomputes rather than believing what it is told.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ArtifactDeliveryRow,
  ArtifactParseError,
  compareLanes,
} from '@monochromatic-dev/translation-repair-artifact-pool';
import { parseSettledTwoLaneArtifact, } from '../../dist/final/node/index.mjs';
import {
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  ARCHIVE_NAP,
  repairLedger,
  translateLedger,
  translateResult,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts/two-lane-ledger-fixtures.test-fixture.ts';

/**
 Ballot backing the translate lane, carried by the contest cases.
 */
const CONTEST_BALLOT = {
  choice: 'translate',
  unsupported: [],
  unsupportedRaw: [],
  dropped: [],
  droppedRaw: [],
  reason: 'the archive`s wording drops the windowsill the original names',
};

/**
 Identity a preparation gives itself, which a standalone reader checks for
 SYNTAX only: the inputs it hashes are not in the file.
 */
const PREPARATION_IDENTITY = `sha256-preparation-v1:${'a7'.repeat(32,)}`;

/**
 Repair lane's raw result, carrying fields version 2 never described so every
 case runs against a record shaped like a real one.

 @returns Raw result JSON

 @example
 ```ts
 const raw = repairResult();
 ```
 */
function repairResult(): Record<string, unknown> {
  return {
    repairedText: `## Section one\n\n${ARCHIVE_NAP}`,
    status: 'unchanged',
    issues: [],
    findings: ['stage-quorum-unmet (critic 0/6)',],
    sliceCritics: [
      {
        sliceIndex: 0,
        heardCriticIds: [],
        claimAttributions: [],
      },
    ],
    sliceCount: 2,
    changedSliceIndices: [],
    withdrawnSliceIndices: [],
    sliceTexts: [
      {
        sliceIndex: 0,
        incumbentKind: 'present',
        incumbentText: ARCHIVE_NAP,
        outcome: {
          kind: 'decided',
          acceptedText: ARCHIVE_NAP,
        },
      },
      {
        sliceIndex: 1,
        incumbentKind: 'absent',
        incumbentText: '',
        outcome: { kind: 'not-applicable', },
      },
    ],
  };
}

/**
 One whole version 2 artifact, with whatever this case changes.

 OVERRIDES RATHER THAN MUTATION, so no case reaches into a nested structure
 and no case can leave one half-edited: a ledger handed in here is the ledger
 both the lane and the comparison are built from.

 @param repairDelivery - repair lane's ledger

 @param translateDelivery - translate lane's ledger

 @param repairRaw - repair lane's raw result

 @param translateRaw - translate lane's raw result

 @param comparison - comparison to record, which DEFAULTS to what version 2's
 own rules derive from the two ledgers, exactly as the writer does

 @param rest - any top-level field this case replaces

 @returns Artifact as JSON

 @example
 ```ts
 const artifact = artifactWith({ repairDelivery: rows, },);
 ```
 */
function artifactWith(
  {
    repairDelivery = repairLedger(),
    translateDelivery = translateLedger(),
    repairRaw = repairResult(),
    translateRaw = translateResult(),
    comparison = compareLanes({
      repair: repairDelivery,
      translate: translateDelivery,
    },),
    ...rest
  }: {
    readonly repairDelivery?: readonly ArtifactDeliveryRow[];
    readonly translateDelivery?: readonly ArtifactDeliveryRow[];
    readonly repairRaw?: Record<string, unknown>;
    readonly translateRaw?: Record<string, unknown>;
    readonly comparison?: readonly unknown[];
    readonly [field: string]: unknown;
  } = {},
): Record<string, unknown> {
  return {
    artifactSchemaVersion: 4,
    id: 'CatEntry1',
    tip: 'a'.repeat(40,),
    pipelineDigest: `sha256-tree-v1:${'c'.repeat(64,)}`,
    corpusSha: 'b'.repeat(40,),
    callConfig: {
      roster: [
        'Tabby',
        'Calico',
      ],
      retries: 2,
    },
    durationMs: 40,
    timestamp: '2026-08-16T21:00:00.000Z',
    preparation: {
      identity: PREPARATION_IDENTITY,
      sliceCount: 2,
      sourceChars: 40,
      targetChars: 60,
      sourceBytes: 90,
      alignmentPairCount: 2,
      alignmentFindings: [],
    },
    lanes: {
      repair: {
        result: repairRaw,
        delivery: repairDelivery,
      },
      translate: {
        result: translateRaw,
        delivery: translateDelivery,
      },
    },
    comparison,
    laneSelection: { kind: 'pending-human-decision', },
    ...rest,
  };
}

/**
 A ledger with one row replaced.

 @param rows - ledger to change

 @param at - position of the row to replace

 @param replace - what that row says instead, built from the row it replaces

 @returns New ledger, leaving the one passed in alone

 @example
 ```ts
 const rows = rowReplaced({ rows: repairLedger(), at: 1, replace: shipIt, },);
 ```
 */
function rowReplaced<const TRow,>(
  {
    rows,
    at,
    replace,
  }: {
    readonly rows: readonly TRow[];
    readonly at: number;
    readonly replace: (row: TRow,) => TRow;
  },
): readonly TRow[] {
  return rows.map(function replaceOne(
    row,
    position,
  ): TRow {
    return (position === at) ? replace(row,) : row;
  },);
}

/**
 Wording the repair lane decided in the cases where it decided one.
 */
const MENDED_NAP = 'The cat is asleep on the sill.';

/**
 Evidence rows saying the repair lane decided that wording at the first slice.

 @returns Raw slice rows for the repair result

 @example
 ```ts
 const rows = repairDecidedRows();
 ```
 */
function repairDecidedRows(): readonly Record<string, unknown>[] {
  return [
    {
      sliceIndex: 0,
      incumbentKind: 'present',
      incumbentText: ARCHIVE_NAP,
      outcome: {
        kind: 'decided',
        acceptedText: MENDED_NAP,
      },
    },
    {
      sliceIndex: 1,
      incumbentKind: 'absent',
      incumbentText: '',
      outcome: { kind: 'not-applicable', },
    },
  ];
}

/**
 Repair row withdrawn by whole-document refusal, the shape only a blocked run
 produces: a decided outcome whose delivery is a refusal rather than a
 shipped or withheld replacement.

 @param row - delivery row to replace

 @returns Row recording the blocked-run withdrawal

 @example
 ```ts
 const refused = rowReplaced({ rows: repairLedger(), at: 0, replace: withdrawByRefusal, },);
 ```
 */
function withdrawByRefusal(row: ArtifactDeliveryRow,): ArtifactDeliveryRow {
  return {
    ...row,
    outcome: {
      kind: 'decided',
      acceptedText: MENDED_NAP,
    },
    delivery: {
      kind: 'replacement-withdrawn',
      reason: 'blocked-non-translation',
    },
  };
}

/**
 A ledger whose rows all claim the first slice.

 @param rows - ledger to collapse

 @returns Same rows, every one naming slice 0

 @example
 ```ts
 const rows = allNamingSliceZero({ rows: repairLedger(), },);
 ```
 */
function allNamingSliceZero(
  { rows, }: { readonly rows: readonly ArtifactDeliveryRow[]; },
): readonly ArtifactDeliveryRow[] {
  return rows.map(function nameZero(row,): ArtifactDeliveryRow {
    return {
      ...row,
      sliceIndex: 0,
    };
  },);
}

/**
 Raw slice rows collapsed the same way, so a lane still agrees with itself and
 only the repeat is left to catch.

 @param rows - raw slice rows to collapse

 @returns Same rows, every one naming slice 0

 @example
 ```ts
 const rows = evidenceNamingSliceZero({ rows: raw.sliceTexts, },);
 ```
 */
function evidenceNamingSliceZero(
  { rows, }: { readonly rows: readonly Record<string, unknown>[]; },
): readonly Record<string, unknown>[] {
  return rows.map(function nameZero(row,): Record<string, unknown> {
    return {
      ...row,
      sliceIndex: 0,
    };
  },);
}

/**
 Re-spells a generation 4 body as generation 3, which is what the pass wrote
 before the index rename reached the wire.

 ONE KEY, EVERYWHERE. A real generation 3 artifact spells the index
 `chunkIndex` in all twenty-odd places it appears: both ledgers, both raw
 results and every array inside them, the comparison, the lane selection and
 the consolidation. The change-set arrays already carried their current names
 in that generation, so nothing else moves.

 @param value - generation 4 body to re-spell

 @returns Same artifact as generation 3 wrote it

 @example
 ```ts
 const older = asGenerationThree(artifactWith(),);
 ```
 */
function asGenerationThree(value: unknown,): unknown {
  if (Array.isArray(value,))
    return value.map(asGenerationThree,);

  if ((value === null) || ((typeof value) !== 'object'))
    return value;

  return Object.fromEntries(
    Object.entries(value,)
      .map(function respell([
        key,
        held,
      ],): readonly [string, unknown] {
        if (key === 'sliceIndex')
          return [
            'chunkIndex',
            held,
          ];

        if (key === 'artifactSchemaVersion')
          return [
            key,
            3,
          ];

        return [
          key,
          asGenerationThree(held,),
        ];
      },),
  );
}

/**
 Drops each lane's raw result from a reading, leaving what was INTERPRETED.

 THE RAW RESULT IS THE FILE'S OWN RECORD, handed back unread so a caller
 wanting a field this version does not describe can still find it. It
 therefore still spells the index the way its own generation wrote it, and
 comparing it across generations would compare the two FILES rather than the
 two readings. Everything else here is parsed through the key vocabulary and
 must come out identical.

 @param artifact - reading to strip

 @returns Same reading with both raw records gone

 @example
 ```ts
 expect(interpretedOf({ artifact: older, },),).toStrictEqual(interpretedOf({ artifact: current, },),);
 ```
 */
function interpretedOf(
  { artifact, }: { readonly artifact: ReturnType<typeof parseSettledTwoLaneArtifact>; },
): Record<string, unknown> {
  /**
   Reading with its lanes held aside, since only those carry a raw record.
   */
  const {
    artifactSchemaVersion: _artifactSchemaVersion,
    lanes,
    ...rest
  } = artifact;

  return {
    ...rest,
    lanes: {
      repair: {
        evidence: lanes.repair.evidence,
        delivery: lanes.repair.delivery,
      },
      translate: {
        evidence: lanes.translate.evidence,
        delivery: lanes.translate.delivery,
      },
    },
  };
}

await describe({
  name: parseSettledTwoLaneArtifact.name,
  children: [
    it({
      name:
        'reports an artifact written before archiveText existed as UNRECORDED rather than as empty, '
          + 'since a file that never carried the field cannot say the entry had no English',
      fn: async () => {
        const parsed = parseSettledTwoLaneArtifact({ value: artifactWith(), },);
        expect(parsed.preparation
          .archiveText
          .kind,).toBe('unrecorded',);
      },
    },),
    it({
      name: 'ROUND-TRIPS the archive text an artifact does carry',
      fn: async () => {
        /**
         Invented archive English, cat-themed, standing in for an entry's whole
         translation.
         */
        const held = '## A cat\'s day\n\nThe kitten dozes on the windowsill.\n';
        /**
         The fixture's own preparation, extended through the helper's field
         escape hatch rather than by spreading its `unknown` value.
         */
        const preparation = artifactWith()
          .preparation as Record<string, unknown>;
        const parsed = parseSettledTwoLaneArtifact({
          value: artifactWith({
            preparation: {
              ...preparation,
              archiveText: held,
            },
          },),
        },);
        expect(parsed.preparation
          .archiveText
          .kind,).toBe('stored',);
        expect((parsed.preparation
          .archiveText
          .kind === 'stored')
          ? parsed.preparation
            .archiveText
            .text
          : '',).toBe(held,);
      },
    },),
    it({
      name: 'REFUSES an archiveText that is not a string, rather than coercing it',
      fn: async () => {
        /**
         {@inheritDoc preparation}
         */
        const preparation = artifactWith()
          .preparation as Record<string, unknown>;
        /**
         What readWrong raised, read for its class as well as its wording.
         */
        const refusalOfReadWrong = caught(function readWrong() {
          parseSettledTwoLaneArtifact({
            value: artifactWith({
              preparation: {
                ...preparation,
                archiveText: 42,
              },
            },),
          },);
        },);

        expect(refusalOfReadWrong,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfReadWrong as Error).message,).toBe(
          'artifact parse failed at CatEntry1.preparation.archiveText: expected a string.',
        );
      },
    },),
    it({
      name: 'ROUND-TRIPS THE SPANS AN ARCHIVE NOTE SEALED, and refuses an empty list and a span ending before its '
        + 'start, each by the path of the field (ledger T8)',
      fn: async () => {
        /**
         {@inheritDoc preparation}
         */
        const preparation = artifactWith()
          .preparation as Record<string, unknown>;
        /**
         One sealed span, as a generation-twelve preparation records it.
         */
        const sealed = {
          startOffset: 4,
          endOffset: 30,
          note: 'Original English by the cat herself',
        };
        const parsed = parseSettledTwoLaneArtifact({
          value: artifactWith({
            preparation: {
              ...preparation,
              archiveOriginalSpans: [sealed,],
            },
          },),
        },);
        expect(parsed.preparation
          .archiveOriginalSpans,).toStrictEqual([sealed,],);

        /**
         Refusals of a list recorded empty and of a span recorded backwards.
         */
        const refusals = [
          [],
          [{
            ...sealed,
            endOffset: sealed.startOffset - 1,
          },],
        ].map(function readSpans(archiveOriginalSpans,): unknown {
          return caught(function readBroken() {
            parseSettledTwoLaneArtifact({
              value: artifactWith({
                preparation: {
                  ...preparation,
                  archiveOriginalSpans,
                },
              },),
            },);
          },);
        },);
        expect(refusals[0],).toBeInstanceOf(ArtifactParseError,);
        expect((refusals[0] as Error).message,).toBe(
          'artifact parse failed at CatEntry1.preparation.archiveOriginalSpans: expected a non-empty '
            + 'list, since nothing sealed is recorded as absence.',
        );
        expect(refusals[1],).toBeInstanceOf(ArtifactParseError,);
        expect((refusals[1] as Error).message,).toBe(
          'artifact parse failed at CatEntry1.preparation.archiveOriginalSpans[0].endOffset: expected '
            + 'an end at or after its start.',
        );
      },
    },),
    it({
      name: 'REFUSES A DIGEST THAT IS NOT ONE and a front matter authority other than the archive\'s, each by the '
        + 'path of the field (ledger T8)',
      fn: async () => {
        /**
         {@inheritDoc preparation}
         */
        const preparation = artifactWith()
          .preparation as Record<string, unknown>;
        /**
         A digest naming no built output.
         */
        const digestRefusal = caught(function readDigest() {
          parseSettledTwoLaneArtifact({ value: artifactWith({ pipelineDigest: 'whiskers', },), },);
        },);
        /**
         A front matter authority no preparation records.
         */
        const authorityRefusal = caught(function readAuthority() {
          parseSettledTwoLaneArtifact({
            value: artifactWith({
              preparation: {
                ...preparation,
                frontMatterAuthority: 'lanes',
              },
            },),
          },);
        },);
        expect(digestRefusal,).toBeInstanceOf(ArtifactParseError,);
        // THE SHAPE, and never the recorded value: forwarding the assertion's
        // message once quoted `whiskers` into this refusal (ledger B34).
        expect((digestRefusal as Error).message,).toBe(
          'artifact parse failed at CatEntry1.pipelineDigest: expected a pipeline digest, "sha256-tree-v1:" '
            + 'followed by 64 lowercase hex characters.',
        );
        expect(authorityRefusal,).toBeInstanceOf(ArtifactParseError,);
        expect((authorityRefusal as Error).message,).toBe(
          'artifact parse failed at CatEntry1.preparation.frontMatterAuthority: expected \'archive\', the '
            + 'one authority a preparation records.',
        );
      },
    },),
    it({
      name:
        'reads a whole artifact, RECOMPUTES its comparison, and hands back each raw lane result unread: '
          + 'a reader wanting a field this version does not check gets it from the artifact rather than '
          + 'from a later generation of this parser',
      fn: async () => {
        /**
         One valid artifact, read back.
         */
        const parsed = parseSettledTwoLaneArtifact({ value: artifactWith(), },);
        expect(parsed.id,).toBe('CatEntry1',);
        expect(parsed.preparation
          .identity,).toBe(PREPARATION_IDENTITY,);
        expect(parsed.lanes
          .repair
          .evidence
          .status,).toBe('unchanged',);
        expect(parsed.lanes
          .translate
          .evidence
          .status,).toBe('unfilled',);
        expect(parsed.comparison
          .map(function toVerdict(row,): string {
            return row.laneRelation;
          },),).toEqual([
            'translate-only',
            'gap-remains',
          ],);

        // The whole raw record, including fields version 2 never described.
        expect(parsed.lanes
          .repair
          .raw
          .repairedText,).toBe(`## Section one\n\n${ARCHIVE_NAP}`,);
        expect(parsed.laneSelection,).toEqual({ kind: 'pending-human-decision', },);
      },
    },),
    it({
      name:
        'reads a SETTLED CONTEST against the comparison it derived for itself, rather than against one '
          + 'a test typed out. The gap slice here leaves both lanes carrying nothing, so a filter reading '
          + 'verdict names rather than lane wording would offer it to the roster as a contest between two '
          + 'empty candidates',
      fn: async () => {
        /**
         Artifact whose contest answers the one slice the two lanes worded
         differently, which is slice 0: slice 1 is a gap neither lane filled.
         */
        const parsed = parseSettledTwoLaneArtifact({
          value: artifactWith({
            laneSelection: {
              kind: 'contested',
              slices: [
                {
                  sliceIndex: 0,
                  verdict: {
                    kind: 'lane-won',
                    lane: 'translate',
                  },
                  ballots: [
                    CONTEST_BALLOT,
                    CONTEST_BALLOT,
                  ],
                  usable: 2,
                },
              ],
            },
          },),
        },);
        if (parsed.laneSelection
          .kind !== 'contested')
          throw new Error('reader returned a pending selection for a contested one',);
        expect(parsed.laneSelection
          .slices
          .map(function nameSlice(slice,): number {
            return slice.sliceIndex;
          },),).toEqual([0,],);
      },
    },),
    it({
      name:
        'REFUSES a contest answering the GAP slice, where neither lane put down wording and there is '
          + 'therefore nothing for a roster to choose between',
      fn: async () => {
        /**
         What contestsAGap raised, read for its class as well as its wording.
         */
        const refusalOfContestsAGap = caught(function contestsAGap() {
          parseSettledTwoLaneArtifact({
            value: artifactWith({
              laneSelection: {
                kind: 'contested',
                slices: [
                  {
                    sliceIndex: 1,
                    verdict: { kind: 'quorum-not-met', },
                    ballots: [],
                    usable: 0,
                  },
                ],
              },
            },),
          },);
        },);

        expect(refusalOfContestsAGap,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfContestsAGap as Error).message,).toBe(
          'artifact parse failed at CatEntry1.laneSelection.slices: expected slices [0], which are the '
            + 'ones where the two lanes differ, rather than [1].',
        );
      },
    },),
    it({
      name:
        'REFUSES a key on a lane ENVELOPE and ACCEPTS the same key inside that lane`s RESULT, which is '
          + 'the schema-ownership rule in one pair: version 2 says a lane is a result beside a ledger, and '
          + 'says nothing at all about what a result holds',
      fn: async () => {
        /**
         What extraOnEnvelope raised, read for its class as well as its wording.
         */
        const refusalOfExtraOnEnvelope = caught(function extraOnEnvelope() {
          parseSettledTwoLaneArtifact({
            value: artifactWith({
              lanes: {
                repair: {
                  result: repairResult(),
                  delivery: repairLedger(),
                  whiskers: 3,
                },
                translate: {
                  result: translateResult(),
                  delivery: translateLedger(),
                },
              },
            },),
          },);
        },);

        expect(refusalOfExtraOnEnvelope,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfExtraOnEnvelope as Error).message,).toBe(
          'artifact parse failed at CatEntry1.lanes.repair.whiskers: expected no key here beyond '
            + 'result, delivery.',
        );
        expect(parseSettledTwoLaneArtifact({
          value: artifactWith({
            repairRaw: {
              ...repairResult(),
              whiskers: 3,
            },
          },),
        },).lanes
          .repair
          .raw
          .whiskers,).toBe(3,);
      },
    },),
    it({
      name:
        'REFUSES A THIRD LANE on the lanes record itself, which the "REFUSES a key on a lane ENVELOPE" case '
          + 'cannot reach: '
          + 'that one pins the keys INSIDE a lane, and this shape names exactly two lanes, so a file '
          + 'recording a third lane`s deliveries would otherwise be read as an ordinary two-lane '
          + 'artifact with one lane`s work silently dropped',
      fn: async () => {
        /**
         What thirdLane raised, read for its class as well as its wording.
         */
        const refusalOfThirdLane = caught(function thirdLane() {
          parseSettledTwoLaneArtifact({
            value: artifactWith({
              lanes: {
                repair: {
                  result: repairResult(),
                  delivery: repairLedger(),
                },
                translate: {
                  result: translateResult(),
                  delivery: translateLedger(),
                },
                consolidate: {
                  result: repairResult(),
                  delivery: repairLedger(),
                },
              },
            },),
          },);
        },);

        expect(refusalOfThirdLane,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfThirdLane as Error).message,).toBe(
          'artifact parse failed at CatEntry1.lanes.consolidate: expected no key here beyond repair, translate.',
        );
      },
    },),
    it({
      name:
        'REFUSES a nested `null` in the call configuration and ACCEPTS one in an unknown raw addition: '
          + 'the writer controls every byte of the first and leaves an unset key out, and controls none of '
          + 'the second',
      fn: async () => {
        /**
         What nullInConfig raised, read for its class as well as its wording.
         */
        const refusalOfNullInConfig = caught(function nullInConfig() {
          parseSettledTwoLaneArtifact({ value: artifactWith({ callConfig: { budget: { slice: null, }, }, },), },);
        },);

        expect(refusalOfNullInConfig,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfNullInConfig as Error).message,).toBe(
          'artifact parse failed at CatEntry1.callConfig.budget.slice: expected a boolean, number, '
            + 'string, array, or object, and never null.',
        );
        expect(parseSettledTwoLaneArtifact({
          value: artifactWith({
            translateRaw: {
              ...translateResult(),
              confidence: { perSlice: null, },
            },
          },),
        },).lanes
          .translate
          .raw
          .confidence,).toEqual({ perSlice: null, },);
      },
    },),
    it({
      name:
        'REFUSES a raw result whose slices are in the WRONG ORDER, even though every row is right: the '
          + 'ledger and the result are joined by position because both are stated in document order, and '
          + 'joining by slice index instead would report that everything matched',
      fn: async () => {
        /**
         The repair result with its two correct rows swapped.
         */
        const raw = repairResult();
        /**
         What rowsOutOfOrder raised, read for its class as well as its wording.
         */
        const refusalOfRowsOutOfOrder = caught(function rowsOutOfOrder() {
          parseSettledTwoLaneArtifact({
            value: artifactWith({
              repairRaw: {
                ...raw,
                sliceTexts: (raw.sliceTexts as readonly unknown[]).toReversed(),
              },
            },),
          },);
        },);

        expect(refusalOfRowsOutOfOrder,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfRowsOutOfOrder as Error).message,).toBe(
          'artifact parse failed at CatEntry1.lanes.repair.delivery[0].sliceIndex: expected slice 1, '
            + 'which the raw result names at this position, rather than slice 0.',
        );
      },
    },),
    it({
      name: 'REFUSES A LEDGER holding other than one row per prepared slice, naming both counts',
      fn: async () => {
        /**
         The read of an artifact whose preparation names a third slice neither ledger holds.
         */
        const read = () =>
          parseSettledTwoLaneArtifact({
            value: artifactWith({
              preparation: {
                identity: PREPARATION_IDENTITY,
                sliceCount: 3,
                sourceChars: 40,
                targetChars: 60,
                sourceBytes: 90,
                alignmentPairCount: 2,
                alignmentFindings: [],
              },
            },),
          },);
        expect(read,).toThrow(ArtifactParseError,);
        expect(read,).toThrow('lanes.repair.delivery: expected one row per prepared slice, which is 3 here, rather than 2.',);
      },
    },),
    it({
      name:
        'REFUSES two ledgers of equal length covering DIFFERENT slices, which a length check alone reads '
          + 'as a matching pair and which no later join could recover from',
      fn: async () => {
        /**
         Translate ledger renumbered so it covers slices 1 and 2.
         */
        const shifted = translateLedger().map(function renumber(row,): ArtifactDeliveryRow {
          return {
            ...row,
            sliceIndex: row.sliceIndex + 1,
          };
        },);

        /**
         Its raw result renumbered the same way, so the lane agrees with
         itself and only the two LANES disagree.
         */
        const raw = translateResult();
        /**
         What coverageDiffers raised, read for its class as well as its wording.
         */
        const refusalOfCoverageDiffers = caught(function coverageDiffers() {
          parseSettledTwoLaneArtifact({
            value: artifactWith({
              translateDelivery: shifted,
              translateRaw: {
                ...raw,
                changedSliceIndices: [1,],
                sliceTexts: (raw.sliceTexts as readonly Record<string, unknown>[])
                  .map(function renumber(row,): Record<string, unknown> {
                    return {
                      ...row,
                      sliceIndex: (row.sliceIndex as number) + 1,
                    };
                  },),
              },
              comparison: [],
            },),
          },);
        },);

        expect(refusalOfCoverageDiffers,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfCoverageDiffers as Error).message,).toBe(
          'artifact parse failed at CatEntry1.comparison: expected two ledgers this version can compare '
            + '(position 0 names slice 0 in the repair ledger and slice 1 in the translate ledger).',
        );
      },
    },),
    it({
      name:
        'REFUSES a ledger that names one slice TWICE, which every other relation reads as a matching '
          + 'pair: the evidence joins by POSITION and agrees, the two lanes join by position and agree, '
          + 'and the row count still equals the prepared slice count',
      fn: async () => {
        /**
         Repair ledger whose two rows both claim slice 0.
         */
        const collapsedRepair = allNamingSliceZero({ rows: repairLedger(), },);

        /**
         Translate ledger collapsed the same way.
         */
        const collapsedTranslate = allNamingSliceZero({ rows: translateLedger(), },);

        // POSITIVE CONTROL for what this case is about: the frozen comparison
        // reads these two ledgers as a matching pair, so a reader that ran only
        // the cross-lane rules would have accepted the repeat.
        expect(
          compareLanes({
            repair: collapsedRepair,
            translate: collapsedTranslate,
          },).length,
        ).toBe(2,);

        /**
         Repair raw result whose slice rows are collapsed to match.
         */
        const repairRaw = repairResult();

        /**
         Translate raw result on the same footing, its shipped set still
         naming the slice its first row shipped.
         */
        const translateRaw = translateResult();
        /**
         What slicesRepeat raised, read for its class as well as its wording.
         */
        const refusalOfSlicesRepeat = caught(function slicesRepeat() {
          parseSettledTwoLaneArtifact({
            value: artifactWith({
              repairDelivery: collapsedRepair,
              translateDelivery: collapsedTranslate,
              repairRaw: {
                ...repairRaw,
                sliceTexts: evidenceNamingSliceZero({
                  rows: repairRaw.sliceTexts as readonly Record<string, unknown>[],
                },),
              },
              translateRaw: {
                ...translateRaw,
                sliceTexts: evidenceNamingSliceZero({
                  rows: translateRaw.sliceTexts as readonly Record<string, unknown>[],
                },),
              },
            },),
          },);
        },);

        expect(refusalOfSlicesRepeat,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfSlicesRepeat as Error).message,).toBe(
          'artifact parse failed at CatEntry1.lanes.repair.delivery[1].sliceIndex: expected a slice '
            + 'after 0, which the row before this one names, since a ledger is stated in document order and '
            + 'every check here joins by position; this row names 0, so the rows are a repeat.',
        );
      },
    },),
    it({
      name:
        'REFUSES a ledger whose rows are PERMUTED rather than repeated, which the first version of this '
          + 'check let through: distinctness was the rule, and a ledger naming slice 1 then slice 0 is '
          + 'perfectly distinct while every positional join in this reader reads each row against the '
          + 'wrong slice of the preparation',
      fn: async () => {
        // Both ledgers reversed, so each lane still agrees with itself and with
        // the other, and only the anchor to document order is broken.
        /**
         Repair ledger in the wrong order.
         */
        const flippedRepair = repairLedger()
          .toReversed();

        /**
         Translate ledger flipped the same way.
         */
        const flippedTranslate = translateLedger()
          .toReversed();

        // POSITIVE CONTROL: the frozen comparison reads the two flipped ledgers
        // as a matching pair, so nothing outside this check was going to notice.
        expect(
          compareLanes({
            repair: flippedRepair,
            translate: flippedTranslate,
          },).length,
        ).toBe(2,);

        /**
         Repair raw result with its slice rows flipped to match.
         */
        const repairRaw = repairResult();

        /**
         Translate raw result on the same footing.
         */
        const translateRaw = translateResult();
        /**
         What slicesPermuted raised, read for its class as well as its wording.
         */
        const refusalOfSlicesPermuted = caught(function slicesPermuted() {
          parseSettledTwoLaneArtifact({
            value: artifactWith({
              repairDelivery: flippedRepair,
              translateDelivery: flippedTranslate,
              repairRaw: {
                ...repairRaw,
                sliceTexts: (repairRaw.sliceTexts as readonly Record<string, unknown>[]).toReversed(),
              },
              translateRaw: {
                ...translateRaw,
                sliceTexts: (translateRaw.sliceTexts as readonly Record<string, unknown>[]).toReversed(),
              },
            },),
          },);
        },);

        expect(refusalOfSlicesPermuted,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfSlicesPermuted as Error).message,).toBe(
          'artifact parse failed at CatEntry1.lanes.repair.delivery[1].sliceIndex: expected a slice '
            + 'after 1, which the row before this one names, since a ledger is stated in document order and '
            + 'every check here joins by position; this row names 0, so the rows are out of order.',
        );
      },
    },),
    it({
      name:
        'ACCEPTS a BLOCKED repair run that withdrew nothing by assembly, which is the case a reader that '
          + 'recomputed the status would refuse: a blocked run whose slices all agreed with the archive '
          + 'produces no withdrawal at all, and is an ordinary artifact',
      fn: async () => {
        expect(parseSettledTwoLaneArtifact({
          value: artifactWith({
            repairRaw: {
              ...repairResult(),
              status: 'blocked-non-translation',
            },
          },),
        },).lanes
          .repair
          .evidence
          .status,).toBe('blocked-non-translation',);
      },
    },),
    it({
      name:
        'REFUSES a run whose status and deliveries could not have happened together, in both directions: '
          + 'an unblocked run carrying a withdrawal by whole-document refusal, and a blocked one carrying a '
          + 'shipped replacement, since a blocked run never assembles anything',
      fn: async () => {
        /**
         An unblocked run naming the refusal that only a blocked one produces.
         */
        const refusedWhileRunning = rowReplaced({
          rows: repairLedger(),
          at: 0,
          replace: withdrawByRefusal,
        },);
        /**
         What unblockedRefusal raised, read for its class as well as its wording.
         */
        const refusalOfUnblockedRefusal = caught(function unblockedRefusal() {
          parseSettledTwoLaneArtifact({
            value: artifactWith({
              repairDelivery: refusedWhileRunning,
              repairRaw: {
                ...repairResult(),
                sliceTexts: repairDecidedRows(),
              },
            },),
          },);
        },);

        expect(refusalOfUnblockedRefusal,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfUnblockedRefusal as Error).message,).toBe(
          'artifact parse failed at CatEntry1.lanes.repair.result.status: expected a status this ledger '
            + 'could hold: slice 0 reports a withdrawal by whole-document refusal, which only a blocked run '
            + 'produces.',
        );

        /**
         A blocked run naming a slice its document carries.
         */
        const shippedWhileBlocked = rowReplaced({
          rows: repairLedger(),
          at: 0,
          replace: function shipIt(row,): ArtifactDeliveryRow {
            return {
              ...row,
              outcome: {
                kind: 'decided',
                acceptedText: MENDED_NAP,
              },
              shippedText: MENDED_NAP,
              delivery: { kind: 'replacement-shipped', },
            };
          },
        },);
        /**
         What blockedShipping raised, read for its class as well as its wording.
         */
        const refusalOfBlockedShipping = caught(function blockedShipping() {
          parseSettledTwoLaneArtifact({
            value: artifactWith({
              repairDelivery: shippedWhileBlocked,
              repairRaw: {
                ...repairResult(),
                status: 'blocked-non-translation',
                changedSliceIndices: [0,],
                sliceTexts: repairDecidedRows(),
              },
            },),
          },);
        },);

        expect(refusalOfBlockedShipping,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfBlockedShipping as Error).message,).toBe(
          'artifact parse failed at CatEntry1.lanes.repair.result.status: expected a status this ledger '
            + 'could hold: slice 0 reports assembly having run, which a blocked run never reaches.',
        );
      },
    },),
    it({
      name:
        'REFUSES an index set the ledger rows do not produce, and reads the WITHDRAWN set as the '
          + 'assembly guard`s alone: a run refused as a whole never assembled anything, so counting its '
          + 'withdrawals would make every blocked document look like one the guard tore apart',
      fn: async () => {
        /**
         What shippedDisagrees raised, read for its class as well as its wording.
         */
        const refusalOfShippedDisagrees = caught(function shippedDisagrees() {
          parseSettledTwoLaneArtifact({
            value: artifactWith({
              repairRaw: {
                ...repairResult(),
                changedSliceIndices: [0,],
              },
            },),
          },);
        },);

        expect(refusalOfShippedDisagrees,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfShippedDisagrees as Error).message,).toBe(
          'artifact parse failed at CatEntry1.lanes.repair.result.changedSliceIndices: expected 0 '
            + 'slices, which is what this lane\'s ledger rows say, rather than 1.',
        );

        /**
         A blocked run whose blocked withdrawal stays OUT of the withdrawn
         list, which is what the writer produces.
         */
        const withdrawnByRefusal = rowReplaced({
          rows: repairLedger(),
          at: 0,
          replace: withdrawByRefusal,
        },);
        expect(parseSettledTwoLaneArtifact({
          value: artifactWith({
            repairDelivery: withdrawnByRefusal,
            repairRaw: {
              ...repairResult(),
              status: 'blocked-non-translation',
              sliceTexts: repairDecidedRows(),
            },
          },),
        },).lanes
          .repair
          .evidence
          .withdrawnSliceIndices,).toEqual([],);
      },
    },),
    it({
      name:
        'REFUSES a row whose two axes cannot both be true, which every field parsing on its own cannot '
          + 'catch: a document carrying a shipped replacement at a slice its lane says it had nothing to '
          + 'do at is a row this reader would otherwise hand on as fact',
      fn: async () => {
        /**
         What axesDisagree raised, read for its class as well as its wording.
         */
        const refusalOfAxesDisagree = caught(function axesDisagree() {
          parseSettledTwoLaneArtifact({
            value: artifactWith({
              repairDelivery: rowReplaced({
                rows: repairLedger(),
                at: 1,
                replace: function shipAtGap(row,): ArtifactDeliveryRow {
                  return {
                    ...row,
                    delivery: { kind: 'replacement-shipped', },
                  };
                },
              },),
            },),
          },);
        },);

        expect(refusalOfAxesDisagree,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfAxesDisagree as Error).message,).toBe(
          'artifact parse failed at CatEntry1.lanes.repair.delivery[1]: expected a row whose outcome '
            + 'and delivery can both be true: slice 1 reports a replacement and an outcome of '
            + 'not-applicable, so there is no decision for the delivery to describe.',
        );
      },
    },),
    it({
      name:
        'REFUSES a recorded comparison the ledgers do not derive, since a stored comparison is a claim '
          + 'about two ledgers stored beside it and nothing has to trust it',
      fn: async () => {
        /**
         The derived comparison with one lane relation changed to something
         the rows do not produce.
         */
        const retitled = compareLanes({
          repair: repairLedger(),
          translate: translateLedger(),
        },)
          .map(function retitleFirst(
            row,
            position,
          ): Record<string, unknown> {
            return (position === 0)
              ? {
                ...row,
                laneRelation: 'both-agree',
              }
              : row;
          },);
        /**
         What comparisonDisagrees raised, read for its class as well as its wording.
         */
        const refusalOfComparisonDisagrees = caught(function comparisonDisagrees() {
          parseSettledTwoLaneArtifact({ value: artifactWith({ comparison: retitled, },), },);
        },);

        expect(refusalOfComparisonDisagrees,).toBeInstanceOf(ArtifactParseError,);
        // The differing field is named; the stored row, which carries the archive
        // text and both lanes' output, is not quoted into a marked message that
        // every CLI prints.
        expect((refusalOfComparisonDisagrees as Error).message,).toBe(
          'artifact parse failed at CatEntry1.comparison[0]: expected what this version\'s rules derive for slice 0 '
            + 'from the ledgers stored beside it; the stored row differs on laneRelation.',
        );
      },
    },),
    it({
      name:
        'REFUSES a recorded comparison carrying MORE rows than the two ledgers derive, which the '
          + 'row-by-row check beside it cannot reach: that walk visits the DERIVED rows, so a row recorded '
          + 'past the end of them is never read, and the file`s claim about a slice its own ledgers do not '
          + 'cover would be dropped without a word',
      fn: async () => {
        /**
         What version 2's rules derive from the two ledgers, which is two rows
         because the preparation carries two slices.
         */
        const derived = compareLanes({
          repair: repairLedger(),
          translate: translateLedger(),
        },);

        /**
         The same comparison with its last row recorded twice, so the file
         claims three slices where the ledgers cover two.
         */
        const overlong = [
          ...derived,
          ...derived.slice(-1,),
        ];

        /**
         What comparisonOverruns raised, read for its class as well as its wording.
         */
        const refusalOfComparisonOverruns = caught(function comparisonOverruns() {
          parseSettledTwoLaneArtifact({ value: artifactWith({ comparison: overlong, },), },);
        },);

        expect(refusalOfComparisonOverruns,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfComparisonOverruns as Error).message,).toBe(
          'artifact parse failed at CatEntry1.comparison: expected one row per slice the two ledgers cover, '
            + 'which is 2 here, rather than 3.',
        );
      },
    },),
    it({
      name:
        'ACCEPTS a recorded comparison whose rows carry the same values in a different KEY ORDER, which '
          + 'is what a file read off disk holds: key order is not part of what a row says, and refusing it '
          + 'would fail artifacts over a difference no reader can see',
      fn: async () => {
        /**
         The same comparison with every key written in the opposite order.
         */
        const reordered = compareLanes({
          repair: repairLedger(),
          translate: translateLedger(),
        },)
          .map(function reverseKeys(row,): Record<string, unknown> {
            return Object.fromEntries(Object.entries(row,)
              .toReversed(),);
          },);
        expect(parseSettledTwoLaneArtifact({ value: artifactWith({ comparison: reordered, },), },).comparison
          .length,).toBe(2,);
      },
    },),
    it({
      name:
        'REFUSES text that differs only by Unicode COMPOSITION, since the contract compares character '
          + 'for character with no normalization: a reader that quietly normalized would report two '
          + 'derivations as agreeing when the bytes on disk do not',
      fn: async () => {
        /**
         Archive wording written composed in the ledger.
         */
        const composed = 'The cat naps in the café.';

        // BOTH DOCUMENTS carry the composed wording, so the two lanes agree
        // with each other and the only disagreement left is INSIDE the repair
        // lane, between its raw result and its own ledger. Leaving the translate
        // side alone would make the lanes disagree first, and this case would
        // pass on a refusal it is not about.
        /**
         What compositionDiffers raised, read for its class as well as its wording.
         */
        const refusalOfCompositionDiffers = caught(function compositionDiffers() {
          parseSettledTwoLaneArtifact({
            value: artifactWith({
              translateDelivery: rowReplaced({
                rows: translateLedger(),
                at: 0,
                replace: function composeIncumbent(row,): ArtifactDeliveryRow {
                  return {
                    ...row,
                    incumbentText: composed,
                  };
                },
              },),
              translateRaw: {
                ...translateResult(),
                sliceTexts: rowReplaced({
                  rows: translateResult().sliceTexts as readonly Record<string, unknown>[],
                  at: 0,
                  replace: function composeIncumbent(row,): Record<string, unknown> {
                    return {
                      ...row,
                      incumbentText: composed,
                    };
                  },
                },),
              },
              repairDelivery: rowReplaced({
                rows: repairLedger(),
                at: 0,
                replace: function composeThroughout(row,): ArtifactDeliveryRow {
                  return {
                    ...row,
                    incumbentText: composed,
                    shippedText: composed,
                    outcome: {
                      kind: 'decided',
                      acceptedText: composed,
                    },
                  };
                },
              },),
              repairRaw: {
                ...repairResult(),
                sliceTexts: rowReplaced({
                  rows: repairResult().sliceTexts as readonly Record<string, unknown>[],
                  at: 0,
                  replace: function decomposeIncumbent(row,): Record<string, unknown> {
                    return {
                      ...row,
                      incumbentText: composed.normalize('NFD',),
                      outcome: {
                        kind: 'decided',
                        acceptedText: composed.normalize('NFD',),
                      },
                    };
                  },
                },),
              },
            },),
          },);
        },);

        expect(refusalOfCompositionDiffers,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfCompositionDiffers as Error).message,).toBe(
          'artifact parse failed at CatEntry1.lanes.repair.delivery[0].incumbentText: expected the '
            + 'archive wording the raw result records for slice 0.',
        );
      },
    },),
    it({
      name: 'ACCEPTS GENERATION SIX with explicit consolidation state',
      fn: async () => {
        /**
         Generation-six fixture after full parsing.
         */
        const parsed = parseSettledTwoLaneArtifact({
          value: artifactWith({
            artifactSchemaVersion: 6,
            consolidation: { kind: 'not-run', },
          },),
        },);
        expect(parsed.artifactSchemaVersion,).toBe(6,);
        expect(parsed.consolidation,).toEqual({ kind: 'not-run', },);
      },
    },),
    it({
      name:
        'REFUSES every generation but the five that wrote this shape, including a MISSING one and a '
          + 'version spelled as text: dispatch has already chosen this reader by the time it is called, '
          + 'so a file arriving here under another version is a caller reading the wrong file',
      fn: async () => {
        expect([
          undefined,
          null,
          '3',
          1,
          7,
        ].map(function refuses(version,): string {
          try {
            parseSettledTwoLaneArtifact({ value: artifactWith({ artifactSchemaVersion: version, },), },);
            return 'accepted';
          } catch (error) {
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
    it({
      name:
        'ACCEPTS an identity that is syntactically an identity and describes some other preparation, '
          + 'which pins the SYNTAX-ONLY boundary: the inputs the identity hashes are not in the file, so a '
          + 'standalone reader cannot tell one from another and must not pretend otherwise',
      fn: async () => {
        /**
         A well formed identity of a preparation nobody here ran.
         */
        const foreign = `sha256-preparation-v1:${'bd'.repeat(32,)}`;
        expect(parseSettledTwoLaneArtifact({
          value: artifactWith({
            preparation: {
              identity: foreign,
              sliceCount: 2,
              sourceChars: 40,
              targetChars: 60,
              sourceBytes: 90,
              alignmentPairCount: 2,
              alignmentFindings: [],
            },
          },),
        },).preparation
          .identity,).toBe(foreign,);
        /**
         What badIdentity raised, read for its class as well as its wording.
         */
        const refusalOfBadIdentity = caught(function badIdentity() {
          parseSettledTwoLaneArtifact({
            value: artifactWith({
              preparation: {
                identity: 'sha256-preparation-v1:cafe',
                sliceCount: 2,
                sourceChars: 40,
                targetChars: 60,
                sourceBytes: 90,
                alignmentPairCount: 2,
                alignmentFindings: [],
              },
            },),
          },);
        },);

        expect(refusalOfBadIdentity,).toBeInstanceOf(ArtifactParseError,);
        // THE SHAPE, and never the recorded value: forwarding the assertion's
        // message once quoted the stored identity into this refusal (ledger B34).
        expect((refusalOfBadIdentity as Error).message,).toBe(
          'artifact parse failed at CatEntry1.preparation.identity: expected a preparation identity, '
            + '"sha256-preparation-v1:" or "sha256-preparation-v2:" followed by 64 lowercase hex characters.',
        );
      },
    },),
    it({
      name:
        'REFUSES a translate lane whose counts or status disagree with what it recorded per slice, which '
          + 'is the lane`s own arithmetic checked against its own rows',
      fn: async () => {
        /**
         What countDisagrees raised, read for its class as well as its wording.
         */
        const refusalOfCountDisagrees = caught(function countDisagrees() {
          parseSettledTwoLaneArtifact({
            value: artifactWith({
              translateRaw: {
                ...translateResult(),
                changedSliceCount: 2,
              },
            },),
          },);
        },);

        expect(refusalOfCountDisagrees,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfCountDisagrees as Error).message,).toBe(
          'artifact parse failed at CatEntry1.lanes.translate.result.changedSliceCount: expected 1, '
            + 'which is how many slices this lane names as shipped.',
        );
        /**
         What statusDisagrees raised, read for its class as well as its wording.
         */
        const refusalOfStatusDisagrees = caught(function statusDisagrees() {
          parseSettledTwoLaneArtifact({
            value: artifactWith({
              translateRaw: {
                ...translateResult(),
                status: 'complete',
              },
            },),
          },);
        },);

        expect(refusalOfStatusDisagrees,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfStatusDisagrees as Error).message,).toBe(
          'artifact parse failed at CatEntry1.lanes.translate.result.status: expected unfilled, since '
            + 'this lane records a slice it reached and could not fill.',
        );
      },
    },),
    it({
      name:
        'REFUSES each lane`s own status ON THE OTHER LANE, in both directions. The two allowed sets sit '
          + 'a few lines apart in one file, so a reader that offered either set to both lanes would take '
          + 'a repair result claiming it was complete and a translate result claiming it was unchanged, '
          + 'and every relation the reader checks after it reads a status it was never asked to doubt',
      fn: async () => {
        /**
         What repairBorrowsTranslate raised, read for its class as well as its wording.
         */
        const refusalOfRepairBorrowsTranslate = caught(function repairBorrowsTranslate() {
          parseSettledTwoLaneArtifact({
            value: artifactWith({
              repairRaw: {
                ...repairResult(),
                status: 'complete',
              },
            },),
          },);
        },);

        expect(refusalOfRepairBorrowsTranslate,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfRepairBorrowsTranslate as Error).message,).toBe(
          'artifact parse failed at CatEntry1.lanes.repair.result.status: expected one of repaired, '
            + 'unchanged, blocked-non-translation.',
        );

        /**
         What translateBorrowsRepair raised, read the same way.
         */
        const refusalOfTranslateBorrowsRepair = caught(function translateBorrowsRepair() {
          parseSettledTwoLaneArtifact({
            value: artifactWith({
              translateRaw: {
                ...translateResult(),
                status: 'unchanged',
              },
            },),
          },);
        },);

        expect(refusalOfTranslateBorrowsRepair,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfTranslateBorrowsRepair as Error).message,).toBe(
          'artifact parse failed at CatEntry1.lanes.translate.result.status: expected one of complete, '
            + 'unfilled.',
        );
      },
    },),
    it({
      name:
        'CARRIES THE REPAIR LANE`S BALLOTS THROUGH, verbatim reasons included, which is the whole '
          + 'reason the rounds are recorded: this lane can delete a declared name from a translation, '
          + 'and until the rounds landed in the artifact the only way to see it happen was to run a '
          + 'live probe against the real judging sheet',
      fn: async () => {
        /**
         One judged round of the shape the editor stage now records.
         */
        const round = {
          kind: 'selected',
          stage: 'envelope',
          envelopeId: 'envelope/nap',
          slate: [
            {
              index: 1,
              rendered: 'Mittens the Cat naps on the sill.',
              hash: 'hash-with-alias',
              producer: {
                kind: 'model',
                modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
              },
            },
            {
              index: 2,
              rendered: 'The cat naps on the sill.',
              hash: 'hash-without-alias',
              producer: {
                kind: 'model',
                modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              },
            },
          ],
          ballots: [
            {
              modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
              best: 2,
              reason: 'the alias has no basis in the original',
              weight: 1,
              selfVote: false,
            },
          ],
          tally: {
            judgesAvailable: 1,
            ballots: 1,
            abstentions: 0,
            selfVotes: 0,
          },
          perCandidate: [],
          selectedIndex: 2,
          voteWeight: 1,
        };

        /**
         Artifact as the bytes a settled file holds.

         SERIALIZED AND PARSED RATHER THAN CLONED, which is not the same test:
         a clone would preserve values JSON cannot carry, and what this case
         asks is whether the round survives the trip to disk and back.
         */
        const serialized = JSON.stringify(artifactWith({
          repairRaw: {
            ...repairResult(),
            chunks: [
              {
                sliceIndex: 0,
                rounds: [round,],
                droppedDeclaredNames: ['Mittens the Cat',],
              },
            ],
          },
        },),);

        /**
         Same artifact as a reader receives it.
         */
        const written = JSON.parse(serialized,) as unknown;

        /**
         Repair lane as a reader gets it back.
         */
        const { raw, } = parseSettledTwoLaneArtifact({ value: written, },)
          .lanes
          .repair;
        expect(raw.chunks,).toEqual([
          {
            sliceIndex: 0,
            rounds: [round,],
            droppedDeclaredNames: ['Mittens the Cat',],
          },
        ],);
      },
    },),
    it({
      name:
        'REFUSES a lane selection naming a lane directly, since a decided contest is recorded as a '
          + 'contest with its ballots and a bare lane name would be a verdict with nothing behind it',
      fn: async () => {
        /**
         What unknownSelection raised, read for its class as well as its wording.
         */
        const refusalOfUnknownSelection = caught(function unknownSelection() {
          parseSettledTwoLaneArtifact({ value: artifactWith({ laneSelection: { kind: 'translate', }, },), },);
        },);

        expect(refusalOfUnknownSelection,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfUnknownSelection as Error).message,).toBe(
          'artifact parse failed at CatEntry1.laneSelection.kind: expected one of '
            + 'pending-human-decision, contested.',
        );
      },
    },),
    it({
      name:
        'POSITIVE CONTROL for the "READS A GENERATION 3 BODY TO EXACTLY WHAT GENERATION 4 READS TO" case: '
          + 're-spelling the body actually moves the '
          + 'index everywhere it appears, so a re-speller that changed nothing would make the equality '
          + 'that follows say nothing at all',
      fn: async () => {
        /**
         Generation 4 body as JSON text.
         */
        const current = JSON.stringify(artifactWith(),);

        /**
         Same body re-spelled, as JSON text.
         */
        const older = JSON.stringify(
          asGenerationThree(artifactWith(),),
        );

        expect(current.includes('"sliceIndex"',),).toBe(true,);
        expect(current.includes('"chunkIndex"',),).toBe(false,);
        expect(older.includes('"sliceIndex"',),).toBe(false,);
        expect(older.includes('"chunkIndex"',),).toBe(true,);
      },
    },),

    it({
      name:
        'READS A GENERATION 3 BODY TO EXACTLY WHAT GENERATION 4 READS TO, which is what the key '
          + 'vocabulary is for: the two files differ only in how the index is spelled, and a reader that '
          + 'holds one spelling reports the older file`s ledger as naming no slices at all',
      fn: async () => {
        /**
         Older body's reading.
         */
        const older = parseSettledTwoLaneArtifact({ value: asGenerationThree(artifactWith(),), },);

        /**
         Current body's reading.
         */
        const current = parseSettledTwoLaneArtifact({ value: artifactWith(), },);

        expect(interpretedOf({ artifact: older, },),)
          .toStrictEqual(interpretedOf({ artifact: current, },),);

        // AND THE RAW RECORDS DIFFER, which is what makes the exclusion above a
        // statement rather than a hiding place: each lane's result is the
        // file's own bytes handed back unread, so it still carries the spelling
        // its generation wrote.
        expect(older.lanes.repair.raw,).not.toStrictEqual(current.lanes.repair.raw,);
      },
    },),

    it({
      name:
        'REFUSES a generation 3 body that declares generation 4, rather than reading its ledger rows '
          + 'as carrying no index: a relabelled file is the one failure the vocabulary cannot see, since '
          + 'the label is the only thing that picks the spelling',
      fn: async () => {
        /**
         What the reader did with an older body under the current label.
         */
        const refusalOfRelabelled = caught(function readsRelabelled() {
          parseSettledTwoLaneArtifact({
            value: {
              ...asGenerationThree(artifactWith(),) as Record<string, unknown>,
              artifactSchemaVersion: 4,
            },
          },);
        },);

        expect(refusalOfRelabelled,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfRelabelled as Error).message,).toBe(
          'artifact parse failed at CatEntry1.lanes.repair.delivery[0].chunkIndex: expected no key here '
            + 'beyond sliceIndex, sourceText, incumbentKind, incumbentText, outcome, shippedText, delivery.',
        );
      },
    },),
  ],
},);
