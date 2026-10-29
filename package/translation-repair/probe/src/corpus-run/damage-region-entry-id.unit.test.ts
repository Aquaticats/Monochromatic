/**
 Tests that the damage census names an entry by ITS OWN file name.

 WHY THE FILE NAME. A settled artifact is written as `<entryId>.json`, so the
 name is the id and the census derives it there rather than trusting a field
 inside the file. Every row the census emits carries that id, the draw digests
 it, and the grading sheet shows it, so an id that is off by one character is
 an entry nobody can trace a graded item back to.

 WHAT WAS MEASURED. On 2026-08-25, starting the derived slice one character in
 failed no test in this package. Nothing throws on a wrong id and nothing is
 missing; every layer downstream carries it faithfully, which is the same
 shape as the forwarding blind spot this family already records.

 NO NETWORK AND NO REAL ARTIFACTS. One artifact is built by the version 2
 builder, written into a throwaway directory under the name production would
 give it, and read back through the reader the census uses.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { SliceDeliveryRecord, } from '@monochromatic-dev/translation-repair-translate-floor';
import type { PipelineDigest, } from '@monochromatic-dev/translation-repair-glossary';
import type { DocumentLanesResult, } from '@monochromatic-dev/translation-repair-translate-lane';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  type PreparationIdentity,
  preparationIdentity,
  type PreparedDocumentPair,
} from '@monochromatic-dev/translation-repair-corpus-source';
import { makeInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  buildSettledTwoLaneArtifact,
  NO_CONSISTENCY,
  NO_PAGE_ASSEMBLY,
} from '@monochromatic-dev/translation-repair-artifact';
import { collectTwoLaneShippedRegions, } from '../../dist/final/node/index.mjs';
import { SEAT_HYPER_OPENROUTER_VISION_EDITOR, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

//region Fixtures

/**
 Entry the artifact is about, and the name its file therefore carries.

 Deliberately begins with a letter that reads as a plausible id without it,
 so a census dropping the first character produces something that still looks
 like an entry rather than something obviously broken.
 */
const ENTRY_ID = 'MittensWindow1';

/**
 Archive wording of the first slice, which the archive does translate.
 */
const ARCHIVE_SILL = 'The kitten sleeps on the sill.';

/**
 Original of that slice.
 */
const SOURCE_SILL = '小猫在窗台上睡觉。';

/**
 Original of the second slice, which the archive never translated.
 */
const SOURCE_BOWL = '小猫有自己的碗。';

/**
 Wording the repair lane shipped over the first slice.
 */
const REPAIRED_SILL = 'The kitten is asleep on the windowsill.';

/**
 Wording the translate lane filled the untranslated slice with.
 */
const FILLED_BOWL = 'The kitten has a bowl of its own.';

/**
 Built pipeline these fixtures claim to have run under.
 */
const DIGEST = 'sha256-tree-v1:'.concat('d'.repeat(64,),) as unknown as PipelineDigest;

/**
 Two prepared slices: one the archive translates, one it never did.

 @returns Pairs shaped as preparation produces them

 @example
 ```ts
 const slices = kittenSlices();
 ```
 */
function kittenSlices(): readonly ChunkPair[] {
  return [
    {
      source: {
        sliceIndex: 0,
        nodes: [],
        startOffset: 0,
        endOffset: SOURCE_SILL.length,
        text: SOURCE_SILL,
      },
      target: {
        sliceIndex: 0,
        nodes: [],
        startOffset: 0,
        endOffset: ARCHIVE_SILL.length,
        text: ARCHIVE_SILL,
      },
    },
    {
      source: {
        sliceIndex: 1,
        nodes: [],
        startOffset: SOURCE_SILL.length,
        endOffset: SOURCE_SILL.length + SOURCE_BOWL.length,
        text: SOURCE_BOWL,
      },
      target: makeInsertionChunk({
        sliceIndex: 1,
        offset: ARCHIVE_SILL.length,
      },),
    },
  ];
}

/**
 Preparation both lanes claim to have run over.

 @returns Preparation shaped as `prepareDocumentPair` returns one

 @example
 ```ts
 const prepared = kittenPreparation();
 ```
 */
function kittenPreparation(): PreparedDocumentPair {
  return {
    sourceText: SOURCE_SILL + SOURCE_BOWL,
    targetText: ARCHIVE_SILL,
    slices: kittenSlices(),
    lineStructuredSliceIndices: new Set<number>(),
    declaredNames: [],
    alignmentFindings: [],
    alignmentPairCount: 2,
  } as unknown as PreparedDocumentPair;
}

/**
 Name that preparation gives itself, stamped on both ledgers.

 @returns Identity of the prepared slicing

 @example
 ```ts
 const identity = kittenIdentity();
 ```
 */
function kittenIdentity(): PreparationIdentity {
  return preparationIdentity({ prepared: kittenPreparation(), },);
}

/**
 Repair lane ledger: it mended the first slice and had no work at the anchor.

 @returns Two rows, one per prepared slice

 @example
 ```ts
 const rows = repairLedger();
 ```
 */
function repairLedger(): readonly SliceDeliveryRecord[] {
  return [
    {
      sliceIndex: 0,
      sourceText: SOURCE_SILL,
      incumbentKind: 'present',
      incumbentText: ARCHIVE_SILL,
      outcome: {
        kind: 'decided',
        acceptedText: REPAIRED_SILL,
      },
      shippedText: REPAIRED_SILL,
      delivery: { kind: 'replacement-shipped', },
    },
    {
      sliceIndex: 1,
      sourceText: SOURCE_BOWL,
      incumbentKind: 'absent',
      incumbentText: '',
      outcome: { kind: 'not-applicable', },
      shippedText: '',
      delivery: { kind: 'gap-remains', },
    },
  ];
}

/**
 Translate lane ledger: it kept the archive's first slice and filled the gap.

 @returns Two rows, one per prepared slice

 @example
 ```ts
 const rows = translateLedger();
 ```
 */
function translateLedger(): readonly SliceDeliveryRecord[] {
  return [
    {
      sliceIndex: 0,
      sourceText: SOURCE_SILL,
      incumbentKind: 'present',
      incumbentText: ARCHIVE_SILL,
      outcome: {
        kind: 'decided',
        acceptedText: ARCHIVE_SILL,
      },
      shippedText: ARCHIVE_SILL,
      delivery: { kind: 'incumbent-retained', },
    },
    {
      sliceIndex: 1,
      sourceText: SOURCE_BOWL,
      incumbentKind: 'absent',
      incumbentText: '',
      outcome: {
        kind: 'decided',
        acceptedText: FILLED_BOWL,
      },
      shippedText: FILLED_BOWL,
      delivery: { kind: 'replacement-shipped', },
    },
  ];
}

/**
 What both lanes returned over that preparation.

 @returns Driver result shaped as `runDocumentLanes` returns one

 @example
 ```ts
 const lanes = kittenLanes();
 ```
 */
function kittenLanes(): DocumentLanesResult {
  return {
    alignmentFindings: [],
    repair: {
      repairedText: REPAIRED_SILL,
      status: 'repaired',
      issues: [],
      findings: [],
      sliceCritics: [
        {
          sliceIndex: 0,
          heardCriticIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
          claimAttributions: [],
        },
      ],
      sliceCount: 2,
      changedSliceIndices: [0,],
      withdrawnSliceIndices: [],
      sliceTexts: [
        {
          sliceIndex: 0,
          incumbentKind: 'present',
          incumbentText: ARCHIVE_SILL,
          outcome: {
            kind: 'decided',
            acceptedText: REPAIRED_SILL,
          },
        },
        {
          sliceIndex: 1,
          incumbentKind: 'absent',
          incumbentText: '',
          outcome: { kind: 'not-applicable', },
        },
      ],
    },
    translate: {
      translatedText: `${ARCHIVE_SILL}\n\n${FILLED_BOWL}`,
      sliceCount: 2,
      changedSliceCount: 1,
      refusedSliceCount: 0,
      withdrawnSliceCount: 0,
      changedSliceIndices: [1,],
      withdrawnSliceIndices: [],
      resumedSliceCount: 0,
      status: 'complete',
      unfilled: [],
      slices: [],
      sliceSelections: [],
      findings: [],
      sliceTexts: [
        {
          sliceIndex: 0,
          incumbentKind: 'present',
          incumbentText: ARCHIVE_SILL,
          outcome: {
            kind: 'decided',
            acceptedText: ARCHIVE_SILL,
          },
        },
        {
          sliceIndex: 1,
          incumbentKind: 'absent',
          incumbentText: '',
          outcome: {
            kind: 'decided',
            acceptedText: FILLED_BOWL,
          },
        },
      ],
    },
    repairDelivery: { preparationIdentity: kittenIdentity(), records: repairLedger(), },
    translateDelivery: { preparationIdentity: kittenIdentity(), records: translateLedger(), },
  } as unknown as DocumentLanesResult;
}

/**
 Writes one settled artifact under the name production gives it and reads the
 shipped regions back out of the directory holding it.

 @returns Census the collector produced, and the directory it read

 @example
 ```ts
 const census = await censusOverOneEntry();
 ```
 */
async function censusOverOneEntry(): Promise<
  Awaited<ReturnType<typeof collectTwoLaneShippedRegions>>
  > {
  /**
   Throwaway pool holding exactly one artifact.
   */
  await using scratch = await scratchDir({ prefix: 'translation-repair-regions-', },);
  const artifactsDir = scratch.path;

  await writeFile(
    join(
      artifactsDir,
      `${ENTRY_ID}.json`,
    ),
    JSON.stringify(buildSettledTwoLaneArtifact({
      pageAssembly: NO_PAGE_ASSEMBLY,
      consistency: NO_CONSISTENCY,
      entryId: ENTRY_ID,
      tip: 'a'.repeat(40,),
      pipelineDigest: DIGEST,
      corpusSha: 'b'.repeat(40,),
      callConfig: { perCallTimeoutMs: 600_000, },
      durationMs: 1_234,
      prepared: kittenPreparation(),
      lanes: kittenLanes(),
      laneSelection: { kind: 'pending-human-decision', },
      consolidation: { kind: 'not-run', },
    },),),
    'utf8',
  );

  /**
   Every region the damage question can be asked about here.
   */
  const census = await collectTwoLaneShippedRegions({
    artifactsDir,
    files: [`${ENTRY_ID}.json`,],
  },);

  return census;
}

//endregion Fixtures

await describe({
  name: collectTwoLaneShippedRegions.name,
  children: [
    it({
      name: 'NAMES each region by the whole file name it came from, since the draw digests that id and '
        + 'the grading sheet shows it, so one off by a character is an entry nobody can trace a graded '
        + 'item back to',
      fn: async () => {
        /**
         Regions read back out of a pool holding one artifact.
         */
        const census = await censusOverOneEntry();

        expect(census.regions
          .map(function toEntryId(region,): string {
            return region.entryId;
          },),).toStrictEqual([ENTRY_ID,],);
      },
    },),

    it({
      name: 'READS the repair lane replacement as a region and COUNTS the translate lane fill apart '
        + 'from it, since a slice the archive never had carries no wording an edit could have damaged',
      fn: async () => {
        /**
         Regions read back out of a pool holding one artifact.
         */
        const census = await censusOverOneEntry();

        expect(census.regions
          .map(function toLane(region,): string {
            return region.lane;
          },),).toStrictEqual(['repair',],);
        expect(census.regions[0]?.sliceIndex,).toBe(0,);
        expect(census.regions[0]?.incumbentText,).toBe(ARCHIVE_SILL,);
        expect(census.filledWithoutIncumbent,).toBe(1,);
      },
    },),
  ],
},);
