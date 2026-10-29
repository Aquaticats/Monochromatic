/**
 Test-only settled artifacts built over a real preparation, as the builder
 writes them: the rows keep every slice the archive has wording for and fill
 every insertion, which is enough for the builder's own checks. Moved out of
 `artifact-two-lane-rebuild.unit.test.ts` when `page-republish.unit.test.ts`
 needed the same artifacts.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import type { SliceDeliveryRecord, } from '@monochromatic-dev/translation-repair-translate-floor';
import {
  preparationIdentity,
  type PreparedDocumentPair,
} from '@monochromatic-dev/translation-repair-corpus-source';
import type { DocumentLanesResult, } from '@monochromatic-dev/translation-repair-translate-lane';
import {
  assertPipelineDigest,
  type PipelineDigest,
} from '@monochromatic-dev/translation-repair-glossary';
import {
  buildSettledTwoLaneArtifact,
  NO_CONSISTENCY,
  NO_PAGE_ASSEMBLY,
  type SettledArtifact,
} from '../../dist/final/node/index.mjs';

/**
 Hex digits in a tree digest.
 */
const DIGEST_HEX_LENGTH = 64;

/**
 Hex digits in a git commit id.
 */
const COMMIT_HEX_LENGTH = 40;

/**
 Digest every fixture artifact claims, before it is checked for shape.
 */
const DIGEST_TEXT = `sha256-tree-v1:${'c'.repeat(DIGEST_HEX_LENGTH,)}`;
assertPipelineDigest(DIGEST_TEXT,);

/**
 Digest every fixture artifact claims.
 */
const FIXTURE_DIGEST: PipelineDigest = DIGEST_TEXT;

/**
 Wording a lane writes where the archive holds none.
 */
const FRESH_LINE = 'The cat has been given a line.';

/**
 Rows that keep every slice the archive has wording for, and fill every
 insertion, which is enough for the builder's own checks.

 A PAIRED PREPARATION LEAVES INSERTIONS: a section or block the pairing did
 not claim is placed as an insertion slice whose archive wording is absent,
 and the builder refuses a row calling that wording present.

 @param prepared - preparation the rows describe

 @returns One row per slice

 @example
 ```ts
 const rows = keptEverything({ prepared, },);
 ```
 */
function keptEverything(
  { prepared, }: { readonly prepared: PreparedDocumentPair; },
): readonly SliceDeliveryRecord[] {
  return prepared.slices
    .map(function toRow(slice,): SliceDeliveryRecord {
      /**
       Original and archive sides of this slice.
       */
      const {
        source,
        target,
      } = slice;
      if (target.kind === 'insertion') {
        return {
          sliceIndex: target.sliceIndex,
          sourceText: source.text,
          incumbentKind: 'absent',
          incumbentText: target.text,
          outcome: {
            kind: 'decided',
            acceptedText: FRESH_LINE,
          },
          shippedText: FRESH_LINE,
          delivery: { kind: 'replacement-shipped', },
        };
      }
      return {
        sliceIndex: target.sliceIndex,
        sourceText: source.text,
        incumbentKind: 'present',
        incumbentText: target.text,
        outcome: {
          kind: 'decided',
          acceptedText: target.text,
        },
        shippedText: target.text,
        delivery: { kind: 'incumbent-retained', },
      };
    },);
}

/**
 Slices a lane shipped a replacement at.

 @param rows - lane's rows

 @returns Their indices

 @example
 ```ts
 const changed = changedBy({ rows, },);
 ```
 */
function changedBy(
  { rows, }: { readonly rows: readonly SliceDeliveryRecord[]; },
): readonly number[] {
  return rows
    .filter(function wasShipped({ delivery, },): boolean {
      return delivery.kind === 'replacement-shipped';
    },)
    .map(function indexOf({ sliceIndex, },): number {
      return sliceIndex;
    },);
}

/**
 What each slice was decided to carry, as a lane's raw result reports it.

 @param rows - lane's rows

 @returns One evidence row per slice

 @example
 ```ts
 const sliceTexts = evidenceOf({ rows, },);
 ```
 */
function evidenceOf(
  { rows, }: { readonly rows: readonly SliceDeliveryRecord[]; },
): DocumentLanesResult['repair']['sliceTexts'] {
  return rows.map(function toEvidence({
    sliceIndex,
    incumbentKind,
    incumbentText,
    outcome,
  },) {
    return {
      sliceIndex,
      incumbentKind,
      incumbentText,
      outcome,
    };
  },);
}

/**
 Both lanes' results for rows the caller states: nothing withdrawn, trimmed
 or refused.

 @param prepared - preparation the lanes ran over

 @param rows - rows both lanes report, which the builder's own checks must accept

 @returns Lane results the builder projects

 @example
 ```ts
 const lanes = lanesOver({ prepared, rows: keptEverything({ prepared, },), },);
 ```
 */
function lanesOver(
  {
    prepared,
    rows,
    translateRows = rows,
    unfilled = [],
  }: {
    readonly prepared: PreparedDocumentPair;
    readonly rows: readonly SliceDeliveryRecord[];
    readonly translateRows?: readonly SliceDeliveryRecord[];
    readonly unfilled?: DocumentLanesResult['translate']['unfilled'];
  },
): DocumentLanesResult {
  /**
   Identity both ledgers claim.
   */
  const identity = preparationIdentity({ prepared, },);

  /**
   Slices the repair lane shipped a replacement at, which are the insertions it filled.
   */
  const changedRepair = changedBy({ rows, },);
  /**
   Slices the translate lane shipped a replacement at.
   */
  const changedTranslate = changedBy({ rows: translateRows, },);

  return {
    alignmentFindings: [...prepared.alignmentFindings,],
    repair: {
      repairedText: prepared.targetText,
      status: 'unchanged',
      issues: [],
      findings: [],
      sliceCritics: [],
      sliceCount: rows.length,
      changedSliceIndices: changedRepair,
      withdrawnSliceIndices: [],
      trimmedReplacements: [],
      sliceTexts: evidenceOf({ rows, },),
      chunks: [],
    },
    translate: {
      translatedText: prepared.targetText,
      sliceCount: translateRows.length,
      changedSliceCount: changedTranslate.length,
      refusedSliceCount: 0,
      withdrawnSliceCount: 0,
      changedSliceIndices: changedTranslate,
      sliceSelections: [],
      withdrawnSliceIndices: [],
      trimmedReplacements: [],
      resumedSliceCount: 0,
      status: (unfilled.length === 0) ? 'complete' : 'unfilled',
      unfilled,
      slices: [],
      sliceTexts: evidenceOf({ rows: translateRows, },),
      findings: [],
    },
    repairDelivery: {
      preparationIdentity: identity,
      records: rows,
    },
    translateDelivery: {
      preparationIdentity: identity,
      records: translateRows,
    },
  };
}

/**
 A settled artifact over a preparation, as the builder returns it, carrying
 the contest and consolidation records the caller states.

 @param prepared - preparation the artifact records

 @param entryId - entry the artifact settles

 @param laneSelection - contest record, stated because the builder takes any
 the artifact schema allows while a pass writes only one

 @param consolidation - consolidation record, stated for the same reason

 @returns Artifact object, unserialized

 @example
 ```ts
 const artifact = settledArtifactOver({ prepared, entryId: 'CatEntry1', laneSelection: { kind: 'pending-human-decision', }, consolidation: { kind: 'not-run', }, },);
 ```
 */
export function settledArtifactOver(
  {
    prepared,
    entryId,
    laneSelection,
    consolidation,
  }: {
    readonly prepared: PreparedDocumentPair;
    readonly entryId: string;
    readonly laneSelection: Parameters<typeof buildSettledTwoLaneArtifact>[0]['laneSelection'];
    readonly consolidation: Parameters<typeof buildSettledTwoLaneArtifact>[0]['consolidation'];
  },
): SettledArtifact {
  return buildSettledTwoLaneArtifact({
    pageAssembly: NO_PAGE_ASSEMBLY,
    consistency: NO_CONSISTENCY,
    entryId,
    tip: 'a'.repeat(COMMIT_HEX_LENGTH,),
    pipelineDigest: FIXTURE_DIGEST,
    corpusSha: 'b'.repeat(COMMIT_HEX_LENGTH,),
    callConfig: { perCallTimeoutMs: 600_000, },
    durationMs: 1_234,
    prepared,
    lanes: lanesOver({
      prepared,
      rows: keptEverything({ prepared, },),
    },),
    laneSelection,
    consolidation,
  },);
}

/**
 A settled artifact over a preparation, as the bytes a file on disk carries.

 @param prepared - preparation the artifact records

 @param entryId - entry the artifact settles

 @returns Artifact's JSON text

 @example
 ```ts
 const value: unknown = JSON.parse(settledArtifactText({ prepared, entryId: 'CatEntry1', },),);
 ```
 */
export function settledArtifactText(
  {
    prepared,
    entryId,
  }: {
    readonly prepared: PreparedDocumentPair;
    readonly entryId: string;
  },
): string {
  /**
   The artifact as the builder writes it.
   */
  const built = settledArtifactOver({
    prepared,
    entryId,
    laneSelection: { kind: 'pending-human-decision', },
    consolidation: { kind: 'not-run', },
  },);
  return JSON.stringify(built,);
}

/**
 A settled artifact over a preparation and the rows the caller states, as the
 builder returns it, undecided and unconsolidated, pinned to the corpus commit
 the caller names.

 @param prepared - preparation the artifact records

 @param entryId - entry the artifact settles

 @param corpusSha - commit the artifact claims its pair was read at

 @param rows - rows both lanes report

 @returns Artifact object, unserialized

 @example
 ```ts
 const artifact = settledArtifactOverRows({ prepared, entryId: 'CatEntry1', corpusSha, rows, },);
 ```
 */
export function settledArtifactOverRows(
  {
    prepared,
    entryId,
    corpusSha,
    rows,
  }: {
    readonly prepared: PreparedDocumentPair;
    readonly entryId: string;
    readonly corpusSha: string;
    readonly rows: readonly SliceDeliveryRecord[];
  },
): SettledArtifact {
  return buildSettledTwoLaneArtifact({
    pageAssembly: NO_PAGE_ASSEMBLY,
    consistency: NO_CONSISTENCY,
    entryId,
    tip: 'a'.repeat(COMMIT_HEX_LENGTH,),
    pipelineDigest: FIXTURE_DIGEST,
    corpusSha,
    callConfig: { perCallTimeoutMs: 600_000, },
    durationMs: 1_234,
    prepared,
    lanes: lanesOver({
      prepared,
      rows,
    },),
    laneSelection: { kind: 'pending-human-decision', },
    consolidation: { kind: 'not-run', },
  },);
}

/**
 A settled artifact over a preparation whose translate lane left every insertion unfilled, as the builder returns it:
 the repair lane had nothing to do there, and the translate lane's outcome carries the floor mark or not as the caller
 states.

 @param prepared - preparation the artifact records, holding at least one insertion

 @param entryId - entry the artifact settles

 @param benchBelowFloor - whether the translate lane never bought a candidate for the gaps

 @returns Artifact object, unserialized

 @example
 ```ts
 const artifact = settledArtifactWithGap({ prepared, entryId: 'CatEntry1', benchBelowFloor: true, },);
 ```
 */
export function settledArtifactWithGap(
  {
    prepared,
    entryId,
    benchBelowFloor,
  }: {
    readonly prepared: PreparedDocumentPair;
    readonly entryId: string;
    readonly benchBelowFloor: boolean;
  },
): SettledArtifact {
  /**
   Rows the repair lane reports: the archive retained, and nothing to do at an insertion.
   */
  const repairRows = keptEverything({ prepared, },)
    .map(function leaveGap(row,): SliceDeliveryRecord {
      return (row.incumbentKind === 'absent')
        ? {
          ...row,
          outcome: { kind: 'not-applicable', },
          shippedText: row.incumbentText,
          delivery: { kind: 'gap-remains', },
        }
        : row;
    },);
  /**
   Rows the translate lane reports: the same, with the gap unfilled.
   */
  const translateRows = repairRows.map(function leaveUnfilled(row,): SliceDeliveryRecord {
    return (row.incumbentKind === 'absent')
      ? {
        ...row,
        outcome: benchBelowFloor
          ? {
            kind: 'unfilled',
            benchBelowFloor: true,
          }
          : { kind: 'unfilled', },
      }
      : row;
  },);
  return buildSettledTwoLaneArtifact({
    pageAssembly: NO_PAGE_ASSEMBLY,
    consistency: NO_CONSISTENCY,
    entryId,
    tip: 'a'.repeat(COMMIT_HEX_LENGTH,),
    pipelineDigest: FIXTURE_DIGEST,
    corpusSha: 'b'.repeat(COMMIT_HEX_LENGTH,),
    callConfig: { perCallTimeoutMs: 600_000, },
    durationMs: 1_234,
    prepared,
    lanes: lanesOver({
      prepared,
      rows: repairRows,
      translateRows,
      unfilled: translateRows
        .filter(function isGap({ outcome, },): boolean {
          return outcome.kind === 'unfilled';
        },)
        .map(function toUnfilled({ sliceIndex, },) {
          return {
            sliceIndex,
            reason: 'no-candidate-backed' as const,
            findings: [],
            ...(benchBelowFloor ? { benchBelowFloor: true as const, } : {}),
          };
        },),
    },),
    laneSelection: { kind: 'pending-human-decision', },
    consolidation: { kind: 'not-run', },
  },);
}
