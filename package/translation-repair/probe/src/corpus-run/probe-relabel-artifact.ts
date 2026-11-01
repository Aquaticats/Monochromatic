import { join, } from 'node:path';

import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  type AdjudicatedIssue,
  isIssueCategory,
  isIssueSeverity,
  type IssueCategory,
  type IssueSeverity,
  readRunJson,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  artifactFileNameOf,
  ArtifactParseError,
  artifactsDirOf,
  requireArray,
  requireRecord,
  requireString,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  readSettledArtifact,
  repairLaneRecordsOf,
} from '@monochromatic-dev/translation-repair-artifact/ts';
import type { RepairRegion, } from '@monochromatic-dev/translation-repair-adjudication/ts';

//region Probe relabel artifact reading
// Reads a settled artifact back into the shapes a prober call needs.
//
// Kept apart from the case rebuild because these two answer different
// questions: this one is about what the run RECORDED, and the rebuild is about
// what the prompt CONTAINED.

/**
 One settled issue record, narrowed to what a rebuild reads.

 @example
 ```ts
 const [record,] = await readArtifactRecords({ entryId: 'Acheron', },);
 ```
 */
export type ArtifactRecord = {
  /**
   Accepted issue as the prompt renders it.
   */
  readonly issue: AdjudicatedIssue;

  /**
   Replacements the accuracy stage applied for this record.
   */
  readonly repairRegions: readonly RepairRegion[];

  /**
   Probe tallies keyed by envelope, already rendered for printing.
   */
  readonly recorded: Readonly<Record<string, string>>;
};

/**
 Counts a recorded probe tally prints, in reading order.
 */
const TALLY_FIELDS: readonly string[] = [
  'corroborated',
  'removalCorroborated',
  'noneFound',
  'uncertain',
];

/**
 Reads a severity, rejecting anything outside the taxonomy.

 @param value - severity as written in the artifact

 @param path - dotted path for the failure message

 @returns Severity as a taxonomy member

 @throws {@link ArtifactParseError} when the value names no known severity

 @example
 ```ts
 const severity = requireSeverity({ value: record.severity, path: 'issue.severity', },);
 ```
 */
function requireSeverity(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): IssueSeverity {
  if (!isIssueSeverity(value,)) {
    throw new ArtifactParseError({
      path,
      reason: 'one of the taxonomy severities',
    },);
  }
  return value;
}

/**
 Reads a claim category, rejecting anything outside the taxonomy.

 @param value - category as written in the artifact

 @param path - dotted path for the failure message

 @returns Category as a taxonomy member

 @throws {@link ArtifactParseError} when the value names no known category

 @example
 ```ts
 const category = requireCategory({ value: claim.category, path: 'claim.category', },);
 ```
 */
function requireCategory(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): IssueCategory {
  if (!isIssueCategory(value,)) {
    throw new ArtifactParseError({
      path,
      reason: 'one of the taxonomy categories',
    },);
  }
  return value;
}

/**
 Rebuilds the accepted issue as the prober sheet renders it.

 Only the three fields `renderPriorIssues` prints are reconstructed, and spans
 and tallies are left empty on purpose. Parsing evidence offsets here would
 carry a shape the prompt never shows, and inventing one would make the
 rebuild diverge from production in a way nothing would catch.

 @param value - one `issues[].issue` of an artifact

 @returns Issue carrying its id, severity, and claim summaries

 @throws {@link ArtifactParseError} when a rendered field is malformed

 @example
 ```ts
 const issue = readRenderedIssue({ value, },);
 ```
 */
function readRenderedIssue(
  { value, }: { readonly value: unknown; },
): AdjudicatedIssue {
  /**
   Issue as a record.
   */
  const record = requireRecord({
    value,
    path: 'issue',
  },);

  return {
    issueId: requireString({
      value: record.issueId,
      path: 'issue.issueId',
    },),
    status: 'accepted',
    severity: requireSeverity({
      value: record.severity,
      path: 'issue.severity',
    },),
    claims: requireArray({
      value: record.claims,
      path: 'issue.claims',
    },)
      .map(function toClaim(entry,) {
        /**
         Claim wrapper as a record.
         */
        const member = requireRecord({
          value: entry,
          path: 'issue.claims[]',
        },);

        /**
         Claim body carrying what the sheet prints.
         */
        const claim = requireRecord({
          value: member.claim,
          path: 'issue.claims[].claim',
        },);

        return {
          claimId: requireString({
            value: member.claimId,
            path: 'issue.claims[].claimId',
          },),
          claim: {
            category: requireCategory({
              value: claim.category,
              path: 'issue.claims[].claim.category',
            },),
            severity: requireSeverity({
              value: claim.severity,
              path: 'issue.claims[].claim.severity',
            },),
            summary: requireString({
              value: claim.summary,
              path: 'issue.claims[].claim.summary',
            },),
            spans: [],
          },
        };
      },),
    tallies: {},
  };
}

/**
 Reads one region out of an artifact's raw JSON.

 @param value - one element of a record's `repairRegions`

 @returns Region with its texts

 @throws {@link ArtifactParseError} when a field is malformed

 @example
 ```ts
 const region = readRegion({ value, },);
 ```
 */
function readRegion({ value, }: { readonly value: unknown; },): RepairRegion {
  /**
   Region as a record.
   */
  const record = requireRecord({
    value,
    path: 'repairRegion',
  },);

  return {
    envelopeId: requireString({
      value: record.envelopeId,
      path: 'repairRegion.envelopeId',
    },),
    issueIds: requireArray({
      value: record.issueIds,
      path: 'repairRegion.issueIds',
    },)
      .map(function toId(entry,) {
        return requireString({
          value: entry,
          path: 'repairRegion.issueIds[]',
        },);
      },),
    before: requireString({
      value: record.before,
      path: 'repairRegion.before',
    },),
    editorAfter: requireString({
      value: record.editorAfter,
      path: 'repairRegion.editorAfter',
    },),
  };
}

/**
 Renders one region's recorded probe tally as a printable line.

 @param value - one element of `introducedDefects.regions`

 @returns Envelope id paired with its counts

 @throws {@link ArtifactParseError} when the envelope id is malformed

 @example
 ```ts
 const [id, line,] = readRecordedTally({ value, },);
 ```
 */
function readRecordedTally({ value, }: { readonly value: unknown; },): readonly [
  string,
  string,
] {
  /**
   Tally as a record.
   */
  const record = requireRecord({
    value,
    path: 'introducedDefects.region',
  },);

  /**
   Counts rendered through JSON so a field of an unexpected type prints as
   itself rather than as a default stringification.
   */
  const counts = TALLY_FIELDS
    .map(function toCount(field,) {
      return `${field}=${JSON.stringify(record[field] ?? 0,)}`;
    },)
    .join(' ',);

  return [
    requireString({
      value: record.envelopeId,
      path: 'introducedDefects.region.envelopeId',
    },),
    counts,
  ];
}

/**
 Reads the recorded probe tallies of one settled record.

 @param record - one `issues[]` element as a record

 @returns Rendered tally per envelope, empty when the record was never probed

 @throws {@link ArtifactParseError} when a region is malformed or repeats an
 envelope id an earlier region of the record carries

 @example
 ```ts
 const recorded = readRecordedTallies({ record, },);
 ```
 */
function readRecordedTallies(
  { record, }: { readonly record: Readonly<Record<string, unknown>>; },
): Readonly<Record<string, string>> {
  /**
   Probe block, absent on a record the stage never probed.
   */
  const probe = record.introducedDefects;
  if (!isJsonRecord(probe,))
    return {};
  if (!('regions' in probe))
    return {};

  /**
   Regions the probe block records.
   */
  const regions = requireArray({
    value: probe.regions,
    path: 'introducedDefects.regions',
  },);

  /**
   Rendered tally per envelope id, first met first; a map until handed back,
   as every record filled by a key is (ledger B77).
   */
  const tallies = new Map<string, string>();
  for (const [at, entry,] of regions.entries()) {
    /**
     Envelope id and rendered counts of this region.
     */
    const [envelopeId, line,] = readRecordedTally({ value: entry, },);
    // NAMES THE POSITION, NEVER THE ID: the record is a stored file, and a
    // refusal of this class is printed whole.
    if (tallies.has(envelopeId,)) {
      throw new ArtifactParseError({
        path: `introducedDefects.regions[${String(at,)}].envelopeId`,
        reason: 'an envelope id no earlier region of this record carries',
      },);
    }
    tallies.set(
      envelopeId,
      line,
    );
  }
  return Object.fromEntries(tallies,);
}

/**
 Issue records of a version 2 artifact, which keeps them in the repair lane.

 @param artifactValue - artifact as its JSON parsed

 @param entryId - entry the artifact settles, for the refusal path

 @returns Records as written

 @example
 ```ts
 const issues = laneIssues({ artifactValue, entryId, },);
 ```
 */
function laneIssues(
  {
    artifactValue,
    entryId,
  }: {
    readonly artifactValue: unknown;
    readonly entryId: string;
  },
): readonly unknown[] {
  /**
   Lane records, parsed by the version 2 reader so the walk is type-checked.
   */
  const { issues, } = repairLaneRecordsOf({
    value: artifactValue,
    path: `artifact ${entryId}`,
  },);
  return issues;
}

/**
 Issue records of a legacy or version 1 artifact, which keeps them at the root.

 @param artifactValue - artifact as its JSON parsed

 @param entryId - entry the artifact settles, for the refusal path

 @returns Records as written

 @example
 ```ts
 const issues = rootIssues({ artifactValue, entryId, },);
 ```
 */
function rootIssues(
  {
    artifactValue,
    entryId,
  }: {
    readonly artifactValue: unknown;
    readonly entryId: string;
  },
): readonly unknown[] {
  /**
   Artifact as a record, which the dispatching reader already proved it is.
   */
  const artifact = requireRecord({
    value: artifactValue,
    path: `artifact ${entryId}`,
  },);
  return requireArray({
    value: artifact.issues,
    path: `artifact ${entryId}.issues`,
  },);
}

/**
 Reads one artifact into the records a rebuild needs.

 @param runsDir - runs directory the artifact is read from, the one its
 caller reads the manifest from; until 2026-10-10 this read the environment
 itself, whatever directory the caller was reading

 @param entryId - corpus entry id

 @returns Settled records carrying issues, regions, and recorded tallies

 @throws {@link ArtifactParseError} when the artifact is malformed

 @example
 ```ts
 const records = await readArtifactRecords({ runsDir, entryId: 'Acheron', },);
 ```
 */
export async function readArtifactRecords(
  {
    runsDir,
    entryId,
  }: {
    readonly runsDir: string;
    readonly entryId: string;
  },
): Promise<readonly ArtifactRecord[]> {
  /**
   Whole artifact as written, read once and dispatched by generation.
   */
  const artifactValue = await readRunJson({
    path: join(
      artifactsDirOf({ runsDir, },),
      artifactFileNameOf({ entryId, },),
    ),
  },);

  /**
   Which generation wrote it, which decides where its issue records live.
   */
  const reading = readSettledArtifact({ value: artifactValue, },);

  /**
   Whether the issue records sit in the repair lane, which is where version 2
   keeps them, or at the root, which is where every earlier generation did.
   */
  const inLane = reading.kind === 'version-2';

  /**
   Issue records where this generation keeps them.

   ROOT FOR THE LEGACY GENERATIONS, LANE FOR VERSION 2. This once read
   `artifact.issues` only, which version 2 does not write, so every call
   refused a well-formed two-lane artifact; the move to the lane then refused
   every legacy artifact instead, which is what the round-three draw consists
   of. The dispatching reader already knows which is which.
   */
  const issues = inLane
    ? laneIssues({
      artifactValue,
      entryId,
    },)
    : rootIssues({
      artifactValue,
      entryId,
    },);

  /**
   Path a refusal names for one record.
   */
  const recordPath = inLane
    ? `artifact ${entryId}.lanes.repair.result.issues[]`
    : `artifact ${entryId}.issues[]`;

  return issues
    .map(function toRecord(value,): ArtifactRecord {
      /**
       Issue record as a record.
       */
      const record = requireRecord({
        value,
        path: recordPath,
      },);

      return {
        issue: readRenderedIssue({ value: record.issue, },),
        repairRegions: requireArray({
          value: record.repairRegions ?? [],
          path: 'repairRegions',
        },)
          .map(function toRegion(entry,) {
            return readRegion({ value: entry, },);
          },),
        recorded: readRecordedTallies({ record, },),
      };
    },);
}

//endregion Probe relabel artifact reading
