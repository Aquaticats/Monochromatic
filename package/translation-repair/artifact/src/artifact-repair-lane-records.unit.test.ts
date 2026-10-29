/**
 Tests for reading the repair lane's own records out of a settled artifact.

 WHAT THESE PIN is a PATH, not a shape. Version 1 wrote these records at the
 artifact root and version 2 writes them inside the repair lane, and every
 reader kept asking the root. Nothing caught it: raw JSON has no type to
 disagree with, so the retired key answered `undefined` and the reader that
 tolerated absence reported an empty corpus.

 So the first case puts DECOY records at the retired key. A reader that still
 asks the root passes every other case here and fails that one.

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
  ArtifactParseError,
  compareLanes,
} from '@monochromatic-dev/translation-repair-artifact-pool';
import { repairLaneRecordsOf, } from '../dist/final/node/index.mjs';
import {
  ARCHIVE_NAP,
  repairLedger,
  translateLedger,
  translateResult,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts/two-lane-ledger-fixtures.test-fixture.ts';

/**
 Identity a preparation gives itself, checked for SYNTAX only.
 */
const PREPARATION_IDENTITY = `sha256-preparation-v1:${'a7'.repeat(32,)}`;

/**
 One issue record as the repair lane writes them.
 */
const LANE_ISSUE = {
  sliceIndex: 0,
  repairDisposition: 'shipped',
  issue: { issueId: 'cat-1', },
};

/**
 One finding as the repair lane writes them.
 */
const LANE_FINDING = 'stage-quorum-unmet (critic 0/6)';

/**
 Records planted at the RETIRED root key, which nothing may read.

 Deliberately different from the lane's own, so a reader still asking the root
 returns these and is caught rather than agreeing by coincidence.
 */
const DECOY_ISSUE = {
  sliceIndex: 9,
  repairDisposition: 'shipped',
  issue: { issueId: 'decoy-must-not-be-read', },
};

/**
 Repair lane's raw result, with whatever this case changes.

 @param over - fields this case replaces, `issues` and `findings` above all

 @returns Raw result JSON

 @example
 ```ts
 const raw = repairResult({ issues: [], },);
 ```
 */
function repairResult(
  over: Record<string, unknown> = {},
): Record<string, unknown> {
  return {
    repairedText: `## Section one\n\n${ARCHIVE_NAP}`,
    status: 'unchanged',
    issues: [LANE_ISSUE,],
    findings: [LANE_FINDING,],
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
    ...over,
  };
}

/**
 One whole version 2 artifact, with whatever this case changes.

 @param repairRaw - repair lane's raw result

 @param rest - any top-level field this case adds, the retired keys included

 @returns Artifact as JSON

 @example
 ```ts
 const artifact = artifactWith({ repairRaw: repairResult({ issues: [], },), },);
 ```
 */
function artifactWith(
  {
    repairRaw = repairResult(),
    ...rest
  }: {
    readonly repairRaw?: Record<string, unknown>;
    readonly [field: string]: unknown;
  } = {},
): Record<string, unknown> {
  /**
   Both ledgers, named so the comparison is derived from the same rows the
   lanes carry rather than from a second copy of them.
   */
  const repairDelivery = repairLedger();

  /**
   Translate lane's rows, on the same footing.
   */
  const translateDelivery = translateLedger();
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
        result: translateResult(),
        delivery: translateDelivery,
      },
    },
    comparison: compareLanes({
      repair: repairDelivery,
      translate: translateDelivery,
    },),
    laneSelection: { kind: 'pending-human-decision', },
    ...rest,
  };
}

await describe({
  name: repairLaneRecordsOf.name,
  children: [
    it({
      name:
        'READS THE LANE, where version 2 keeps these records, rather than the '
          + 'artifact root where version 1 kept them. This is the whole point of '
          + 'the function: every reader of these records asked the root, got '
          + '`undefined` from raw JSON, and reported an empty corpus',
      fn: async () => {
        /**
         Records as this reader lifts them.
         */
        const records = repairLaneRecordsOf({
          value: artifactWith(),
          path: 'CatEntry1',
        },);

        expect(records.issues,).toStrictEqual([LANE_ISSUE,],);
        expect(records.findings,).toStrictEqual([LANE_FINDING,],);
      },
    },),

    it({
      name:
        'REFUSES AN ARTIFACT CARRYING THE RETIRED ROOT KEY at all, which is a '
          + 'stronger guarantee than preferring the lane over it. Version 2 names '
          + 'its top-level keys exactly, so the two spellings can never coexist '
          + 'for a reader to pick the wrong one between',
      fn: async () => {
        /**
         What retiredKeyPresent raised, read for its class as well as its wording.
         */
        const refusalOfRetiredKeyPresent = caught(function retiredKeyPresent() {
          repairLaneRecordsOf({
            value: artifactWith({ issues: [DECOY_ISSUE,], },),
            path: 'CatEntry1',
          },);
        },);

        expect(refusalOfRetiredKeyPresent,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfRetiredKeyPresent as Error).message,).toContain('issues',);
      },
    },),

    it({
      name:
        'ACCEPTS AN EMPTY ISSUE LIST, because an entry the critics filed nothing '
          + 'about still writes `issues: []`, and refusing it would turn an '
          + 'ordinary quiet entry into a parse failure',
      fn: async () => {
        /**
         Entry that settled without a single filed issue.
         */
        const records = repairLaneRecordsOf({
          value: artifactWith({ repairRaw: repairResult({ issues: [], },), },),
          path: 'CatEntry1',
        },);

        expect(records.issues,).toStrictEqual([],);
        expect(records.findings,).toStrictEqual([LANE_FINDING,],);
      },
    },),

    it({
      name:
        'REFUSES AN ABSENT ISSUE LIST rather than reading it as an entry that '
          + 'filed nothing. Absence means this reader and the writer disagree '
          + 'about where the records live, and answering that with an empty list '
          + 'is exactly what hid the moved path',
      fn: async () => {
        /**
         What noIssues raised, read for its class as well as its wording.
         */
        const refusalOfNoIssues = caught(function noIssues() {
          repairLaneRecordsOf({
            value: artifactWith({
              repairRaw: repairResult({ issues: undefined, },),
            },),
            path: 'CatEntry1',
          },);
        },);

        expect(refusalOfNoIssues,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfNoIssues as Error).message,).toContain('lanes.repair.result.issues',);
      },
    },),

    it({
      name:
        'REFUSES AN ABSENT FINDING LIST on the same reasoning, and this is the '
          + 'sharper half: findings exist to notice a stage going quiet, so a '
          + 'reader that answers their absence with silence reports the one '
          + 'condition it was built to detect',
      fn: async () => {
        /**
         What noFindings raised, read for its class as well as its wording.
         */
        const refusalOfNoFindings = caught(function noFindings() {
          repairLaneRecordsOf({
            value: artifactWith({
              repairRaw: repairResult({ findings: undefined, },),
            },),
            path: 'CatEntry1',
          },);
        },);

        expect(refusalOfNoFindings,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfNoFindings as Error).message,).toContain('lanes.repair.result.findings',);
      },
    },),
  ],
},);
