/**
 Settled artifacts carrying probe telemetry, for the `score-probe` cases.

 Each is a whole version 2 artifact the builder wrote over a small preparation,
 with its repair lane's issue records and findings replaced by the case's own,
 since the lane's result is the one part version 2 leaves to the lane.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { requireRecord, } from '@monochromatic-dev/translation-repair-artifact-pool';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import { settledArtifactText, } from './settled-artifact.test-fixture.ts';

/**
 Original with one section.
 */
const SOURCE_DOC = '## 第一节\n\n猫猫在窗台上睡觉。\n';

/**
 Archive English of the same shape.
 */
const TARGET_DOC = '## Section one\n\nThe cat sleeps on the sill.\n';

/**
 Probers every fixture reading was configured with, all of which answered.
 */
const PROBER_COUNT = 3;

/**
 One region tally, as an artifact carries it.

 @param envelopeId - envelope the region replaced

 @param issueIds - every issue the region serves

 @param corroborated - upheld claims of added damage, out of three probers

 @returns Tally for a fixture record

 @example
 ```ts
 const tally = probeRegion({ envelopeId: 'envelope/nap', issueIds: ['adjudicated/nap',], corroborated: 2, },);
 ```
 */
export function probeRegion(
  {
    envelopeId,
    issueIds,
    corroborated,
  }: {
    readonly envelopeId: string;
    readonly issueIds: readonly string[];
    readonly corroborated: number;
  },
): Readonly<Record<string, unknown>> {
  return {
    envelopeId,
    issueIds,
    corroborated,
    removalCorroborated: 0,
    contradicted: 0,
    unanchored: 0,
    preExisting: 0,
    noneFound: PROBER_COUNT - corroborated,
    uncertain: 0,
    claims: Array.from(
      { length: corroborated, },
      function toClaim(
        _unused,
        index,
      ): Readonly<Record<string, unknown>> {
        return {
          modelId: `hf:cat/Prober-${String(index,)}`,
          admissibility: 'corroborated',
          category: 'meaning',
          severity: 'major',
          evidence: 'the cat naps',
          omittedText: 'the cat had not napped',
          reason: 'new wording',
        };
      },
    ),
  };
}

/**
 One shipped repair record of an issue, probed by three probers.

 @param issueId - adjudicated issue the record is about

 @param regions - tallies of the regions the probe read

 @param refined - whether the naturalness lane rewrote the slice afterwards

 @returns Record for a fixture artifact

 @example
 ```ts
 const record = probedRecord({ issueId: 'adjudicated/nap', regions: [], refined: false, },);
 ```
 */
export function probedRecord(
  {
    issueId,
    regions,
    refined,
  }: {
    readonly issueId: string;
    readonly regions: readonly Readonly<Record<string, unknown>>[];
    readonly refined: boolean;
  },
): Readonly<Record<string, unknown>> {
  return {
    sliceIndex: 0,
    repairDisposition: 'shipped',
    resolved: true,
    refined,
    issue: { issueId, },
    introducedDefects: {
      heardProbers: PROBER_COUNT,
      configuredProbers: PROBER_COUNT,
      regions,
    },
  };
}

/**
 A settled artifact whose repair lane carries the given records.

 @param entryId - entry the artifact settles

 @param issues - repair lane's issue records

 @param findings - what the lane's stages reported

 @returns The artifact's JSON text

 @example
 ```ts
 const text = probeArtifactText({ entryId: 'Whiskers', issues: [], findings: [], },);
 ```
 */
export function probeArtifactText(
  {
    entryId,
    issues,
    findings,
  }: {
    readonly entryId: string;
    readonly issues: readonly unknown[];
    readonly findings: readonly string[];
  },
): string {
  /**
   The artifact as the builder writes it.
   */
  const built = requireRecord({
    value: JSON.parse(settledArtifactText({
      prepared: prepareDocumentPair({
        sourceText: SOURCE_DOC,
        targetText: TARGET_DOC,
        includeFrontMatter: true,
        sealArchiveOriginal: true,
      },),
      entryId,
    },),),
    path: 'artifact',
  },);

  /**
   Lane container every two-lane artifact carries.
   */
  const lanes = requireRecord({
    value: built.lanes,
    path: 'artifact.lanes',
  },);

  /**
   Repair lane, the one whose records the case replaces.
   */
  const repair = requireRecord({
    value: lanes.repair,
    path: 'artifact.lanes.repair',
  },);

  /**
   Repair lane's own result, which version 2 leaves to the lane.
   */
  const result = requireRecord({
    value: repair.result,
    path: 'artifact.lanes.repair.result',
  },);
  result.issues = issues;
  result.findings = findings;
  return JSON.stringify(built,);
}
