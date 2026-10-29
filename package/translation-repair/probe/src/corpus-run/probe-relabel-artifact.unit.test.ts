/**
 Tests for reading a settled run's repair-lane issue records back.

 THE ONE CASE THAT EARNS ITS KEEP IS WHERE THE RECORDS ARE READ FROM. This
 reader asked for `artifact.issues`, at the ROOT, which version 2 does not
 write; the issues live under `lanes.repair.result`. Every call therefore
 refused a perfectly well-formed artifact, and the relabel probe could not
 gather a single case. The module's own comment records it.

 A ROOT-READING VERSION AND A LANE-READING ONE ARE TOLD APART BY AN ARTIFACT
 WITH NO ROOT `issues` KEY AT ALL, which is what production writes. The lane
 reader answers with the lane's records; a root reader meets `requireArray` on
 an absent field and refuses. So the discriminating fixture is the ordinary
 one, and no malformed input is needed to make the point.

 THE RUNS DIRECTORY IS HANDED IN. Until 2026-10-10 the reader took it from
 the environment, whatever directory its caller was reading, so one case
 points the environment at another directory and shows it unread; that case
 writes `process.env`, which is process-wide, so every case here runs at
 `concurrency: 1` and the variable is put back however it ends.

 FIXTURES ARE INVENTED AND CAT-THEMED, written into a throwaway directory that
 removes itself. A real run directory holds unlicensed corpus wording.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ArtifactDeliveryRow,
  ArtifactParseError,
  compareLanes,
} from '@monochromatic-dev/translation-repair-artifact-pool';
import { readArtifactRecords, } from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { runsDirPointedAt, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/runs-dir-pointed.test-fixture.ts';

//region Probe relabel artifact reading tests

/**
 Entry these fixtures describe.
 */
const ENTRY_ID = 'whiskers';

/**
 Wording the archive carried for the one slice.
 */
const ARCHIVE_WORDING = 'The cat sat on the mat.';

/**
 Wording the translate lane produced instead.
 */
const FRESH_WORDING = 'The cat sat upon the mat.';

/**
 Source of that slice, in the Simplified Chinese the corpus is written in.
 */
const SOURCE_WORDING = '猫坐在垫子上。';

/**
 Region an edit replaced, named as the artifact names one.
 */
const ENVELOPE_ID = 'envelope/purr';

/**
 Issue that region served.
 */
const ISSUE_ID = 'adjudicated/purr';

/**
 Model the fixture's run records as its whole roster.
 */
const PROBER = 'cat-house/tabbyscribe-2';

/**
 Builds the ledger row both lanes carry for the one slice.

 @param shipped - wording this lane delivered

 @param delivery - how that wording got there

 @returns One-row ledger

 @example
 ```ts
 const rows = ledger({ shipped: ARCHIVE_WORDING, delivery: { kind: 'incumbent-retained', }, },);
 ```
 */
function ledger(
  {
    shipped,
    delivery,
  }: {
    readonly shipped: string;
    readonly delivery: ArtifactDeliveryRow['delivery'];
  },
): readonly ArtifactDeliveryRow[] {
  return [
    {
      sliceIndex: 0,
      sourceText: SOURCE_WORDING,
      incumbentKind: 'present',
      incumbentText: ARCHIVE_WORDING,
      outcome: {
        kind: 'decided',
        acceptedText: shipped,
      },
      shippedText: shipped,
      delivery,
    },
  ];
}

/**
 Builds a settled version 2 artifact carrying the given repair-lane issues.

 NO ROOT `issues` KEY, deliberately, because that is what production writes
 and what tells a lane reader from a root reader.

 @param issues - repair-lane issue records, in the shape the lane stores them

 @returns Whole artifact value

 @example
 ```ts
 const artifact = settledArtifact({ issues: [], },);
 ```
 */
function settledArtifact(
  { issues, }: { readonly issues: readonly unknown[]; },
): Record<string, unknown> {
  /**
   Repair lane's ledger, which kept the archive's wording.
   */
  const repairDelivery = ledger({
    shipped: ARCHIVE_WORDING,
    delivery: { kind: 'incumbent-retained', },
  },);

  /**
   Translate lane's ledger, which shipped its own.
   */
  const translateDelivery = ledger({
    shipped: FRESH_WORDING,
    delivery: { kind: 'replacement-shipped', },
  },);

  return {
    artifactSchemaVersion: 4,
    id: ENTRY_ID,
    tip: 'a'.repeat(40,),
    pipelineDigest: `sha256-tree-v1:${'c'.repeat(64,)}`,
    corpusSha: 'b'.repeat(40,),
    callConfig: {
      roster: [PROBER,],
      retries: 2,
    },
    durationMs: 40,
    timestamp: '2026-08-25T12:00:00.000Z',
    preparation: {
      identity: `sha256-preparation-v1:${'a7'.repeat(32,)}`,
      sliceCount: 1,
      sourceChars: 40,
      targetChars: 60,
      sourceBytes: 90,
      alignmentPairCount: 1,
      alignmentFindings: [],
    },
    lanes: {
      repair: {
        result: {
          repairedText: ARCHIVE_WORDING,
          status: 'unchanged',
          issues,
          findings: [],
          sliceCritics: [
            {
              sliceIndex: 0,
              heardCriticIds: [],
              claimAttributions: [],
            },
          ],
          sliceCount: 1,
          changedSliceIndices: [],
          withdrawnSliceIndices: [],
          sliceTexts: [
            {
              sliceIndex: 0,
              incumbentKind: 'present',
              incumbentText: ARCHIVE_WORDING,
              outcome: {
                kind: 'decided',
                acceptedText: ARCHIVE_WORDING,
              },
            },
          ],
        },
        delivery: repairDelivery,
      },
      translate: {
        result: {
          translatedText: FRESH_WORDING,
          sliceCount: 1,
          changedSliceCount: 1,
          refusedSliceCount: 0,
          withdrawnSliceCount: 0,
          changedSliceIndices: [0,],
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
              incumbentText: ARCHIVE_WORDING,
              outcome: {
                kind: 'decided',
                acceptedText: FRESH_WORDING,
              },
            },
          ],
        },
        delivery: translateDelivery,
      },
    },
    comparison: compareLanes({
      repair: repairDelivery,
      translate: translateDelivery,
    },),
    laneSelection: { kind: 'pending-human-decision', },
  };
}

/**
 Builds one repair-lane issue record.

 @param withRegions - whether the record carries a replaced region, since an
 older run records none and the reader defaults it

 @returns Record as the lane stores one

 @example
 ```ts
 const record = issueRecord({ withRegions: true, },);
 ```
 */
function issueRecord(
  { withRegions, }: { readonly withRegions: boolean; },
): unknown {
  return {
    sliceIndex: 0,
    resolved: false,
    issue: {
      issueId: ISSUE_ID,
      status: 'accepted',
      severity: 'minor',
      claims: [
        {
          claimId: 'claim/whisker',
          claim: {
            category: 'accuracy/omission',
            severity: 'minor',
            summary: 'A purr is dropped from the greeting.',
            spans: [
              {
                side: 'source',
                nodeId: 'block/0',
                quotedText: SOURCE_WORDING,
              },
              {
                side: 'target',
                nodeId: 'block/0',
                quotedText: '',
              },
            ],
          },
        },
      ],
      tallies: {},
    },
    ...(withRegions
      ? {
        repairRegions: [
          {
            envelopeId: ENVELOPE_ID,
            issueIds: [ISSUE_ID,],
            before: ARCHIVE_WORDING,
            editorAfter: FRESH_WORDING,
          },
        ],
      }
      : {}),
  };
}

/**
 Builds an artifact of the generation before schema versions, the shape the
 round-three draw consists of: no version field and the issue records at the
 root.

 @param issues - issue records to place at the root

 @returns Artifact as its JSON would parse

 @example
 ```ts
 const artifact = legacyArtifact({ issues: [], },);
 ```
 */
function legacyArtifact(
  { issues, }: { readonly issues: readonly unknown[]; },
): Record<string, unknown> {
  return {
    id: ENTRY_ID,
    status: 'settled',
    // The legacy generation recorded a repair beside every issue, and its
    // parser requires the disposition and the refined flag once any repair
    // field is present.
    issues: issues.map(function withRepairFields(record,): unknown {
      return {
        ...(record as Record<string, unknown>),
        refined: false,
        repairDisposition: 'unchanged',
      };
    },),
  };
}

/**
 Writes one artifact into a throwaway run and reads its records back.

 @param artifact - whole artifact value

 @returns Records the reader made of it

 @throws Whatever the reader refuses with

 @example
 ```ts
 const records = await recordsOf({ artifact: settledArtifact({ issues: [], },), },);
 ```
 */
async function recordsOf(
  { artifact, }: { readonly artifact: Record<string, unknown>; },
): Promise<readonly {
  readonly issue: { readonly issueId: string; };
  readonly repairRegions: readonly { readonly envelopeId: string; }[];
  readonly recorded: Readonly<Record<string, string>>;
}[]> {
  await using runs = await scratchDir({ prefix: 'whiskers-relabel-artifact-', },);
  await mkdir(
    join(
      runs.path,
      'artifacts',
    ),
    { recursive: true, },
  );
  await writeFile(
    join(
      runs.path,
      'artifacts',
      `${ENTRY_ID}.json`,
    ),
    JSON.stringify(artifact,),
    'utf8',
  );
  return await readArtifactRecords({
    runsDir: runs.path,
    entryId: ENTRY_ID,
  },);
}

await describe({
  name: readArtifactRecords.name,
  children: [
    it({
      name: 'READS the issues from the repair lane, where version 2 keeps them',
      fn: async () => {
        // A reader looking at `artifact.issues` meets an absent field on this
        // ordinary artifact and refuses. That was the defect: every call
        // refused a well-formed run, and the probe gathered nothing.
        expect(await recordsOf({ artifact: settledArtifact({ issues: [], },), },),).toEqual([],);
      },
    },),
    it({
      name: 'READS THE RUNS DIRECTORY IT IS HANDED, not the one the environment names, so a caller reading one run '
        + 'reads that run\'s artifacts',
      fn: async () => {
        await using elsewhere = await scratchDir({ prefix: 'whiskers-relabel-elsewhere-', },);
        using pointed = runsDirPointedAt({ path: elsewhere.path, },);
        expect(await recordsOf({ artifact: settledArtifact({ issues: [], },), },),).toEqual([],);
      },
    },),
    it({
      name: 'RETURNS one record per issue the lane adjudicated',
      fn: async () => {
        /**
         Records read back from a lane carrying one issue.
         */
        const records = await recordsOf({
          artifact: settledArtifact({ issues: [issueRecord({ withRegions: true, },),], },),
        },);

        expect(records.length,).toBe(1,);
        expect(records[0]?.issue.issueId,).toBe(ISSUE_ID,);
      },
    },),
    it({
      name: 'CARRIES the replaced regions, which the relabelling is about',
      fn: async () => {
        /**
         Records read back from a lane carrying one replaced region.
         */
        const records = await recordsOf({
          artifact: settledArtifact({ issues: [issueRecord({ withRegions: true, },),], },),
        },);

        expect(records[0]?.repairRegions.length,).toBe(1,);
        expect(records[0]?.repairRegions[0]?.envelopeId,).toBe(ENVELOPE_ID,);
      },
    },),
    it({
      name: 'READS the issues from the ROOT of a legacy artifact, which is what the round-three draw '
        + 'consists of',
      fn: async () => {
        // The reader once looked only at the lane, so every artifact written
        // before schema versions refused at its first field, and the two
        // round-three instruments could not run against their own draw.
        const records = await recordsOf({
          artifact: legacyArtifact({ issues: [issueRecord({ withRegions: true, },),], },),
        },);
        expect(records.length,).toBe(1,);
        expect(records[0]?.issue.issueId,).toBe(ISSUE_ID,);
        expect(records[0]?.repairRegions[0]?.envelopeId,).toBe(ENVELOPE_ID,);
      },
    },),
    it({
      name: 'READS a legacy artifact with no issues as no records, not as a refusal',
      fn: async () => {
        expect(await recordsOf({ artifact: legacyArtifact({ issues: [], },), },),).toEqual([],);
      },
    },),
    it({
      name: 'ACCEPTS a record from before regions were stored, defaulting them to none',
      fn: async () => {
        // An issue with no region is an issue the editor never acted on, which
        // is an ordinary outcome rather than a malformed file. Refusing it
        // would make one unrepaired issue lose a whole entry's cases.
        /**
         Records read back from a lane whose record stores no regions.
         */
        const records = await recordsOf({
          artifact: settledArtifact({ issues: [issueRecord({ withRegions: false, },),], },),
        },);

        expect(records.length,).toBe(1,);
        expect(records[0]?.repairRegions,).toEqual([],);
      },
    },),
    it({
      name: 'RENDERS each recorded probe tally under its envelope, the counts in reading order, a count never '
        + 'written as 0 and a count of another type as itself (ledger T8)',
      fn: async () => {
        /**
         Records read back from a lane whose record carries one probed region.
         */
        const records = await recordsOf({
          artifact: settledArtifact({
            issues: [{
              ...(issueRecord({ withRegions: true, },) as Record<string, unknown>),
              introducedDefects: {
                regions: [{
                  envelopeId: ENVELOPE_ID,
                  corroborated: 2,
                  noneFound: 'several',
                },],
              },
            },],
          },),
        },);
        expect(records[0]?.recorded,).toEqual({
          [ENVELOPE_ID]: 'corroborated=2 removalCorroborated=0 noneFound="several" uncertain=0',
        },);
      },
    },),
    it({
      name: 'READS a probe block written as null, or holding no regions, as no tallies (ledger T8)',
      fn: async () => {
        /**
         Tallies read back from a lane whose record carries one probe block.

         @param introducedDefects - probe block as written

         @returns Record's rendered tallies
         */
        async function recordedOf(
          introducedDefects: unknown,
        ): Promise<Readonly<Record<string, string>> | 'no record'> {
          /**
           Records read back.
           */
          const records = await recordsOf({
            artifact: settledArtifact({
              issues: [{
                ...(issueRecord({ withRegions: true, },) as Record<string, unknown>),
                introducedDefects,
              },],
            },),
          },);
          return records[0]?.recorded ?? 'no record';
        }
        expect(await recordedOf(null,),).toEqual({},);
        expect(await recordedOf({},),).toEqual({},);
      },
    },),
    it({
      name: 'REFUSES an issue severity or a claim category outside the taxonomy, naming where each sits '
        + '(ledger T8)',
      fn: async () => {
        /**
         The one fixture record, as a record to vary.
         */
        const record = issueRecord({ withRegions: true, },) as {
          readonly issue: {
            readonly claims: readonly { readonly claim: Record<string, unknown>; }[];
          } & Record<string, unknown>;
        } & Record<string, unknown>;
        /**
         Reading of an issue whose severity no taxonomy names.
         */
        const severity = recordsOf({
          artifact: legacyArtifact({
            issues: [{
              ...record,
              issue: {
                ...record.issue,
                severity: 'purring',
              },
            },],
          },),
        },);
        await expect(severity,).rejects.toBeInstanceOf(ArtifactParseError,);
        await expect(severity,).rejects.toThrow('issue.severity',);
        /**
         Reading of a claim whose category no taxonomy names.
         */
        const category = recordsOf({
          artifact: legacyArtifact({
            issues: [{
              ...record,
              issue: {
                ...record.issue,
                claims: record.issue.claims.map(function purring(member,) {
                  return {
                    ...member,
                    claim: {
                      ...member.claim,
                      category: 'purr/unknown',
                    },
                  };
                },),
              },
            },],
          },),
        },);
        await expect(category,).rejects.toBeInstanceOf(ArtifactParseError,);
        await expect(category,).rejects.toThrow('issue.claims[].claim.category',);
      },
    },),
    it({
      name: 'REFUSES a recorded tally with no envelope id, naming where it sits (ledger T8)',
      fn: async () => {
        /**
         Reading of a lane whose probed region names no envelope.
         */
        const refusal = recordsOf({
          artifact: settledArtifact({
            issues: [{
              ...(issueRecord({ withRegions: true, },) as Record<string, unknown>),
              introducedDefects: { regions: [{ corroborated: 1, },], },
            },],
          },),
        },);
        await expect(refusal,).rejects.toBeInstanceOf(ArtifactParseError,);
        await expect(refusal,).rejects.toThrow('introducedDefects.region.envelopeId',);
      },
    },),
    it({
      name: 'REFUSES A SECOND PROBED REGION UNDER ONE ENVELOPE ID, naming the position of the second and '
        + 'quoting no stored value',
      fn: async () => {
        /**
         Reading of a lane whose probe block tallies one envelope twice, with different counts.
         */
        const refusal = recordsOf({
          artifact: settledArtifact({
            issues: [{
              ...(issueRecord({ withRegions: true, },) as Record<string, unknown>),
              introducedDefects: {
                regions: [
                  { envelopeId: ENVELOPE_ID, corroborated: 2, },
                  { envelopeId: 'envelope/mew', corroborated: 1, },
                  { envelopeId: ENVELOPE_ID, noneFound: 3, },
                ],
              },
            },],
          },),
        },);
        await expect(refusal,).rejects.toBeInstanceOf(ArtifactParseError,);
        await expect(refusal,).rejects.toThrow(
          'artifact parse failed at introducedDefects.regions[2].envelopeId: expected '
            + 'an envelope id no earlier region of this record carries.',
        );
      },
    },),
  ],
  concurrency: 1,
},);

//endregion Probe relabel artifact reading tests
