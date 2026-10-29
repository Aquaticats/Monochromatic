/**
 Tests for checking a version 2 artifact's measurements against a
 preparation.

 WHAT THESE PIN is the boundary the standalone reader has to leave open. That
 reader checks the recorded preparation identity for SYNTAX and nothing more,
 because the inputs the identity hashes are not in the file; these cases run
 the same artifact against a preparation somebody rebuilt, and read its
 recorded measurements against it, as the rendering audit does. They tested an
 identity check nothing called until 2026-09-29 (ledger B30), and the
 measurements check the audit runs had no case of its own.

 The preparation here is a REAL one from `prepareDocumentPair` rather than a
 hand-built stand-in, so the identity, the slices and every measurement come
 from the same code a corpus pass runs.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { SliceDeliveryRecord, } from '@monochromatic-dev/translation-repair-translate-floor';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  preparationIdentity,
  type PreparedDocumentPair,
} from '@monochromatic-dev/translation-repair-corpus-source';
import type { PipelineDigest, } from '@monochromatic-dev/translation-repair-glossary';
import type { DocumentLanesResult, } from '@monochromatic-dev/translation-repair-translate-lane';
import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  buildSettledTwoLaneArtifact,
  NO_CONSISTENCY,
  NO_PAGE_ASSEMBLY,
  parseSettledTwoLaneArtifact,
  verifyArtifactMeasurements,
} from '../../dist/final/node/index.mjs';
import { toEvidence, } from '@monochromatic-dev/translation-repair-translate-floor/ts/corpus-run/lane-result-evidence.test-fixture.ts';

/**
 Original document, two sections a preparation slices apart.
 */
const SOURCE_DOC = '## 第一节\n\n猫猫在窗台上睡觉。\n\n## 第二节\n\n猫猫有自己的碗。\n';

/**
 Archive translation of it, structured the same way.
 */
const TARGET_DOC = '## Section one\n\nThe cat sleeps on the sill.\n\n## Section two\n\nThe cat has a bowl.\n';

/**
 A second pair, which no artifact here describes.
 */
const OTHER_SOURCE_DOC = '## 第一节\n\n猫猫在门口等着。\n\n## 第二节\n\n猫猫喜欢晒太阳。\n';

/**
 Its archive translation.
 */
const OTHER_TARGET_DOC = '## Section one\n\nThe cat waits by the door.\n\n## Section two\n\nThe cat likes the sun.\n';

/**
 Built pipeline these fixtures claim to have run under.
 */
const DIGEST = 'sha256-tree-v1:'.concat('c'.repeat(64,),) as unknown as PipelineDigest;

/**
 One lane ledger over a real preparation, where the lane examined every slice
 and kept what the archive already said.

 @param prepared - preparation to build rows from

 @returns One row per prepared slice, in document order

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
       Archive wording at this slice, which the lane decided to keep.
       */
      const incumbentText = slice.target
        .text;
      return {
        sliceIndex: slice.target
          .sliceIndex,
        sourceText: slice.source
          .text,
        incumbentKind: 'present',
        incumbentText,
        outcome: {
          kind: 'decided',
          acceptedText: incumbentText,
        },
        shippedText: incumbentText,
        delivery: { kind: 'incumbent-retained', },
      };
    },);
}

/**
 What one lane's raw result reports about those rows.

 @param rows - ledger the result describes

 @returns Raw result fields version 2 requires, shared by both lanes here

 @example
 ```ts
 const raw = rawResultFor({ rows, },);
 ```
 */
function rawResultFor(
  { rows, }: { readonly rows: readonly SliceDeliveryRecord[]; },
): Record<string, unknown> {
  return {
    sliceCount: rows.length,
    changedSliceIndices: [],
    withdrawnSliceIndices: [],

    // The two counts the translate lane reports beside its lists. Harmless on
    // the repair result, whose evidence core does not name them, and required
    // on the other side.
    changedSliceCount: 0,
    withdrawnSliceCount: 0,
    sliceTexts: rows.map(function evidenceOf(row,): Record<string, unknown> {
      return toEvidence(row,);
    },),
  };
}

/**
 Builds one artifact over a real preparation and reads it back.

 THROUGH JSON on the way, because that is what a reader holds: the writer's
 object and the file are two different things, and a check that skipped the
 serialization would not be reading an artifact at all.

 @param prepared - preparation both lanes ran over

 @returns Artifact as the version 2 reader returns it

 @example
 ```ts
 const artifact = writeAndRead({ prepared, },);
 ```
 */
function writeAndRead(
  { prepared, }: { readonly prepared: PreparedDocumentPair; },
): ReturnType<typeof parseSettledTwoLaneArtifact> {
  /**
   Rows both lanes report, which are the same here: neither moved.
   */
  const rows = keptEverything({ prepared, },);

  /**
   Name this preparation gives itself, stamped on both ledgers by the driver.
   */
  const identity = preparationIdentity({ prepared, },);

  /**
   What the driver returned.
   */
  const lanes = {
    alignmentFindings: [...prepared.alignmentFindings,],
    repair: {
      ...rawResultFor({ rows, },),
      repairedText: prepared.targetText,
      status: 'unchanged',
    },
    translate: {
      ...rawResultFor({ rows, },),
      translatedText: prepared.targetText,
      status: 'complete',
    },
    repairDelivery: {
      preparationIdentity: identity,
      records: rows,
    },
    translateDelivery: {
      preparationIdentity: identity,
      records: rows,
    },
  } as unknown as DocumentLanesResult;
  /**
   What the writer assembled, still an object in memory.
   */
  const written = buildSettledTwoLaneArtifact({
    pageAssembly: NO_PAGE_ASSEMBLY,
    consistency: NO_CONSISTENCY,
    entryId: 'CatEntry1',
    tip: 'a'.repeat(40,),
    pipelineDigest: DIGEST,
    corpusSha: 'b'.repeat(40,),
    callConfig: { perCallTimeoutMs: 600_000, },
    durationMs: 1_234,
    prepared,
    lanes,
    laneSelection: { kind: 'pending-human-decision', },
    consolidation: { kind: 'not-run', },
  },);

  // THROUGH THE SERIALIZED FORM, deliberately, rather than a structured clone:
  // what a reader holds is the bytes a file carries, and a clone would preserve
  // things JSON drops.
  const serialized = JSON.stringify(written,);
  return parseSettledTwoLaneArtifact({ value: JSON.parse(serialized,), },);
}

await describe({
  name: verifyArtifactMeasurements.name,
  children: [
    it({
      name:
        'ACCEPTS an artifact against the preparation it was written over: every measurement it records '
          + 'of the two documents is what that preparation measures',
      fn: async () => {
        /**
         A real preparation of the cat pair.
         */
        const prepared = prepareDocumentPair({
          sourceText: SOURCE_DOC,
          targetText: TARGET_DOC,
        },);
        verifyArtifactMeasurements({
          artifact: writeAndRead({ prepared, },),
          prepared,
        },);
      },
    },),
    it({
      name:
        'REFUSES the same artifact against a preparation of DIFFERENT documents, naming the first '
          + 'measurement that differs: a standalone reader accepts any syntactically valid identity, and '
          + 'this is the check that tells one preparation from another',
      fn: async () => {
        /**
         Preparation the artifact describes.
         */
        const prepared = prepareDocumentPair({
          sourceText: SOURCE_DOC,
          targetText: TARGET_DOC,
        },);

        /**
         A preparation of another pair entirely.
         */
        const otherPair = prepareDocumentPair({
          sourceText: OTHER_SOURCE_DOC,
          targetText: OTHER_TARGET_DOC,
        },);

        /**
         Artifact written over the first.
         */
        const artifact = writeAndRead({ prepared, },);
        /**
         What differentDocuments raised, read for its class as well as its wording.
         */
        const refusalOfDifferentDocuments = caught(function differentDocuments() {
          verifyArtifactMeasurements({
            artifact,
            prepared: otherPair,
          },);
        },);

        expect(refusalOfDifferentDocuments,).toBeInstanceOf(ArtifactParseError,);
        // The two originals slice alike and differ in length, so the source's
        // character count is the first measurement the check reads that differs.
        expect(otherPair.slices.length,).toBe(prepared.slices.length,);
        expect((refusalOfDifferentDocuments as Error).message,).toContain('CatEntry1.preparation.sourceChars',);
      },
    },),
    it({
      name:
        'REFUSES the same DOCUMENTS sliced under a different budget into a different number of slices: '
          + 'the two preparations describe one pair of texts and pair different originals with different '
          + 'archive wordings, so an artifact read against the wrong one would report rows nobody produced',
      fn: async () => {
        /**
         Preparation the artifact describes.
         */
        const prepared = prepareDocumentPair({
          sourceText: SOURCE_DOC,
          targetText: TARGET_DOC,
        },);

        /**
         The same documents sliced far more finely.
         */
        const finer = prepareDocumentPair({
          sourceText: SOURCE_DOC,
          targetText: TARGET_DOC,
          sliceCharBudget: 8,
        },);

        // POSITIVE CONTROL for the case: unless the budget actually changed the
        // slice count, this would be checking an artifact against measurements
        // it shares and passing for the wrong reason.
        expect(finer.slices.length,).not
          .toBe(prepared.slices.length,);
        /**
         What differentSlicing raised, read for its class as well as its wording.
         */
        const refusalOfDifferentSlicing = caught(function differentSlicing() {
          verifyArtifactMeasurements({
            artifact: writeAndRead({ prepared, },),
            prepared: finer,
          },);
        },);

        expect(refusalOfDifferentSlicing,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfDifferentSlicing as Error).message,).toContain('CatEntry1.preparation.sliceCount',);
      },
    },),
  ],
},);
