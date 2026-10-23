/**
 Tests that a whole settled artifact tells a slice the select bench never bought a candidate for from a slice a panel
 declined, by the artifact alone, and that the mark belongs to generation twenty-one.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool';
import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source';
import { makeInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract';
import { parseSettledTwoLaneArtifact, } from '../../dist/final/node/index.mjs';
import { settledArtifactWithGap, } from './settled-artifact.test-fixture.ts';

/**
 Original of the passage the archive rendered.
 */
const SOURCE_ONE = '猫猫在窗台上睡觉。';

/**
 Original of the passage the archive never translated.
 */
const SOURCE_TWO = '小猫在楼梯上看着。';

/**
 Archive's own English for the first passage.
 */
const ARCHIVE_ONE = 'The cat sleeps on the sill.';

/**
 Preparation of two slices: one the archive rendered, one it did not, which is an insertion.
 */
const PREPARED: PreparedDocumentPair = {
  sourceText: `${SOURCE_ONE}\n\n${SOURCE_TWO}`,
  targetText: ARCHIVE_ONE,
  slices: [
    {
      source: {
        sliceIndex: 0,
        nodes: [],
        startOffset: 0,
        endOffset: SOURCE_ONE.length,
        text: SOURCE_ONE,
      },
      target: {
        sliceIndex: 0,
        nodes: [],
        startOffset: 0,
        endOffset: ARCHIVE_ONE.length,
        text: ARCHIVE_ONE,
      },
    },
    {
      source: {
        sliceIndex: 1,
        nodes: [],
        startOffset: 0,
        endOffset: SOURCE_TWO.length,
        text: SOURCE_TWO,
      },
      target: makeInsertionChunk({
        sliceIndex: 1,
        offset: ARCHIVE_ONE.length,
      },),
    },
  ],
  lineStructuredSliceIndices: new Set<number>(),
  declaredNames: [],
  alignmentFindings: [],
  unclaimedTargetBlocks: [],
  alignmentPairCount: 2,
};

/**
 Artifact as a file on disk carries it, with the gap marked or not.

 @param benchBelowFloor - whether the translate lane never bought a candidate for the gap

 @returns The artifact read back from its bytes

 @example
 ```ts
 const written = writtenArtifact({ benchBelowFloor: true, },);
 ```
 */
function writtenArtifact(
  { benchBelowFloor, }: { readonly benchBelowFloor: boolean; },
): Record<string, unknown> {
  /**
   The artifact as the builder returns it.
   */
  const built = settledArtifactWithGap({
    prepared: PREPARED,
    entryId: 'CatEntry1',
    benchBelowFloor,
  },);
  /**
   The bytes a file on disk carries.
   */
  const bytes = JSON.stringify(built,);
  return JSON.parse(bytes,) as Record<string, unknown>;
}

/**
 Outcomes the parsed translate lane records, in slice order.

 @param artifact - artifact as a file carries it

 @returns One outcome per slice

 @example
 ```ts
 const outcomes = translateOutcomesOf({ artifact, },);
 ```
 */
function translateOutcomesOf(
  { artifact, }: { readonly artifact: Record<string, unknown>; },
): readonly unknown[] {
  return parseSettledTwoLaneArtifact({ value: artifact, },)
    .lanes
    .translate
    .delivery
    .map(function outcomeOf(row,): unknown {
      return row.outcome;
    },);
}

await describe({
  name: parseSettledTwoLaneArtifact.name,
  concurrency: 1,
  children: [
    it({
      name: 'TELLS A GAP THE SELECT BENCH NEVER BOUGHT A CANDIDATE FOR FROM A GAP A PANEL DECLINED, by the artifact '
        + 'alone: the written generation-twenty-one file carries the mark on the one and not on the other',
      fn: async () => {
        /**
         Outcomes of the gap slice in each file.
         */
        const gapOf = (benchBelowFloor: boolean,): unknown => translateOutcomesOf({
          artifact: writtenArtifact({ benchBelowFloor, }),
        },)
          .find(function isGap(outcome,): boolean {
            return (outcome as { readonly kind: string; }).kind === 'unfilled';
          },);
        expect({
          skipped: gapOf(true,),
          declined: gapOf(false,),
        },).toEqual({
          skipped: {
            kind: 'unfilled',
            benchBelowFloor: true,
          },
          declined: { kind: 'unfilled', },
        },);
      },
    },),
    it({
      name: 'REFUSES THE MARK IN A FILE THAT RECORDS GENERATION TWENTY, which did not record it, rather than reading a '
        + 'key the generation never wrote',
      fn: async () => {
        /**
         The marked file relabelled as the generation before.
         */
        const older = {
          ...writtenArtifact({ benchBelowFloor: true, }),
          artifactSchemaVersion: 20,
        };
        const refusal = caught(function readOlder() {
          return parseSettledTwoLaneArtifact({ value: older, },);
        },);
        expect(refusal,).toBeInstanceOf(ArtifactParseError,);
        expect(String(refusal,),).toContain('.outcome.benchBelowFloor: expected no key here beyond kind.',);
      },
    },),
  ],
},);
