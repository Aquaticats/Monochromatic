import { join, } from 'node:path';

import {
  type ArtifactFileName,
  ArtifactParseError,
  entryIdOfArtifact,
  keepEligible,
  listArtifactFiles,
  type PoolPolicy,
  resolvePool,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  isJsonArray,
  isJsonRecord,
  refusalText,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { readRunJson, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { decodeSliceCritics, } from './attribution-decode.ts';
import {
  ARTIFACT_SCHEMA_VERSION_V1,
  type ArtifactSchemaReading,
  CHUNK_SPELLED_KEYS,
  keyVocabularyOf,
  readArtifactSchemaVersion,
} from '@monochromatic-dev/translation-repair-artifact/ts';
import type {
  AcceptedIssueView,
  AttributionEntry,
} from './attribution-report.ts';

//region Attribution read
// Parses settled artifacts into the shape the attribution report needs.
//
// ONE KEY IS TOLERANT OF ABSENCE, and only one: the critic record
// (`sliceCritics`/`chunkCritics`), because an artifact settled before
// attribution existed carries no such key at all, and that absence is DATA
// rather than a fault. The report counts those entries separately instead of
// reading them as critics that raised nothing.
//
// EVERY OTHER FIELD HERE IS REQUIRED, because every generation that has ever
// settled an artifact writes it: `id`, `issues`, an issue record, its `issue`,
// its `status`, its `claims`, a claim's `claimId`, and, from version 2 on,
// `lanes`, `lanes.repair` and `lanes.repair.result`. For these, absence is not
// a legacy reading, it is the same corruption a wrong shape is, and both are
// refused the same way: `ArtifactParseError` naming the path.

/**
 Reads the claim ids one adjudicated issue represents.

 @param issue - adjudicated issue block

 @param path - dotted path of `issue` itself, which each refusal of a field
 inside it extends

 @returns Deterministic claim ids this issue represents

 @throws {@link ArtifactParseError} when `claims` is absent or not an array, a
 member is not a record, or its `claimId` is absent or not a string

 @example
 ```ts
 const claimIds = readClaimIds({ issue, path: `${entryId}.issues[0].issue`, },);
 ```
 */
function readClaimIds(
  {
    issue,
    path,
  }: {
    readonly issue: Readonly<Record<string, unknown>>;
    readonly path: string;
  },
): readonly string[] {
  /**
   Member claims of the issue.
   */
  const { claims, } = issue;
  if (!isJsonArray(claims,))
    throw new ArtifactParseError({
      path: `${path}.claims`,
      reason: 'an array',
    },);

  return claims.map(function toId(
    member,
    index,
  ): string {
    /**
     Path of this claim member.
     */
    const here = `${path}.claims[${String(index,)}]`;
    if (!isJsonRecord(member,))
      throw new ArtifactParseError({
        path: here,
        reason: 'a record',
      },);

    /**
     Deterministic identity of this claim.
     */
    const { claimId, } = member;
    if ((typeof claimId) !== 'string')
      throw new ArtifactParseError({
        path: `${here}.claimId`,
        reason: 'a string',
      },);

    return claimId;
  },);
}

/**
 Record carrying this artifact's own attribution and issue records.

 TWO PATHS, AND EXACTLY ONE PER ARTIFACT, CHOSEN BY THE ARTIFACT'S OWN
 GENERATION rather than guessed from what keys happen to be there. Version 1
 wrote the critic record and `issues` at the artifact root and carried no
 `lanes` key at all; version 2 onward writes them inside the repair lane at
 `lanes.repair.result`, root-level decoys included, and a reader that still
 asked the root would silently agree with one.

 Measured, before this held, over the settled artifacts: 0 of 47 carried
 `chunkCritics` at the root and 47 of 47 carried it in the repair lane, so the
 whole population was filed as pre-feature and the 2479 attributions it held
 reached no consumer. The issue read failed the same way, 0 records against
 1546.

 The version 1 path stays rather than being replaced, because artifacts
 outlive the pipelines that wrote them and a settled file must keep answering
 for itself. A `lanes` key on a version 1 or unversioned artifact is refused
 rather than read: that generation never wrote one, so its presence is
 corruption rather than a two-lane artifact caught early.

 @param parsed - parsed artifact

 @param entryId - artifact identity, which starts the path of each refusal here

 @param reading - this artifact's own generation, already read by the caller

 @returns Root record on a generation that keeps its records there, else the
 repair lane's own result

 @throws {@link ArtifactParseError} when a version 1 or unversioned artifact
 carries a `lanes` key, or when a version 2 or later artifact's `lanes`,
 `lanes.repair` or `lanes.repair.result` is absent or not a record

 @example
 ```ts
 const records = recordsHolderOf({ parsed, entryId, reading, },);
 ```
 */
function recordsHolderOf(
  {
    parsed,
    entryId,
    reading,
  }: {
    readonly parsed: Readonly<Record<string, unknown>>;
    readonly entryId: string;
    readonly reading: ArtifactSchemaReading;
  },
): Readonly<Record<string, unknown>> {
  /**
   Whether this generation keeps its records at the artifact root: every
   unversioned artifact, and version 1, which is the version that introduced
   the field without yet moving the records it names.
   */
  const keepsRecordsAtRoot = (reading.kind === 'unversioned')
    || (reading.version === ARTIFACT_SCHEMA_VERSION_V1);

  if (keepsRecordsAtRoot) {
    if (Object.hasOwn(
      parsed,
      'lanes',
    ))
      throw new ArtifactParseError({
        path: `${entryId}.lanes`,
        reason: 'no lanes, since this generation keeps its records at the root',
      },);
    return parsed;
  }

  /**
   Lane container every two-lane generation writes.
   */
  const { lanes, } = parsed;
  if (!isJsonRecord(lanes,))
    throw new ArtifactParseError({
      path: `${entryId}.lanes`,
      reason: 'a record',
    },);

  /**
   Repair lane, the only one that files issues or hears critics.
   */
  const { repair, } = lanes;
  if (!isJsonRecord(repair,))
    throw new ArtifactParseError({
      path: `${entryId}.lanes.repair`,
      reason: 'a record',
    },);

  /**
   Lane's own result, spelled `result` on disk.
   */
  const { result, } = repair;
  if (!isJsonRecord(result,))
    throw new ArtifactParseError({
      path: `${entryId}.lanes.repair.result`,
      reason: 'a record',
    },);

  return result;
}

/**
 Reads one artifact's accepted-issue views.

 @param raw - record holding this artifact's own `issues`

 @param entryId - artifact identity, which starts the path of each refusal here

 @returns Issue views this artifact's records hold

 @throws {@link ArtifactParseError} when `issues` is absent or not an array, a
 record is not a record, its `issue` is absent or not a record, or its
 `status` is absent or not a string

 @example
 ```ts
 const issues = readIssueViews({ raw, entryId, },);
 ```
 */
function readIssueViews(
  {
    raw,
    entryId,
  }: {
    readonly raw: Readonly<Record<string, unknown>>;
    readonly entryId: string;
  },
): readonly AcceptedIssueView[] {
  /**
   Issue records of this artifact.
   */
  const { issues, } = raw;
  if (!isJsonArray(issues,))
    throw new ArtifactParseError({
      path: `${entryId}.issues`,
      reason: 'an array',
    },);

  return issues.map(function toView(
    record,
    index,
  ): AcceptedIssueView {
    /**
     Path of this issue record.
     */
    const here = `${entryId}.issues[${String(index,)}]`;
    if (!isJsonRecord(record,))
      throw new ArtifactParseError({
        path: here,
        reason: 'a record',
      },);

    /**
     Adjudicated issue inside the record.
     */
    const { issue, } = record;
    if (!isJsonRecord(issue,))
      throw new ArtifactParseError({
        path: `${here}.issue`,
        reason: 'a record',
      },);

    /**
     Adjudication status of the issue.
     */
    const { status, } = issue;
    if ((typeof status) !== 'string')
      throw new ArtifactParseError({
        path: `${here}.issue.status`,
        reason: 'a string',
      },);

    return {
      status,
      claimIds: readClaimIds({
        issue,
        path: `${here}.issue`,
      },),
    };
  },);
}

/**
 Reads one artifact into the shape the report needs.

 EXPORTED FOR ITS OWN CASES. The gather admits only what the pool placed, and
 placement already refuses a file that is not a record or whose `id` is not its
 file name, so those two refusals here are reached only when the file changes
 between the pool's read and this one. They stay because this read is the one
 the report trusts, and a case can reach them only by calling this directly.

 @param name - artifact file name, which names the failures that precede
 reading `id` and is the identity the pool admitted the artifact under

 @param parsed - parsed artifact

 @returns Entry view

 @throws {@link ArtifactParseError} when `parsed` is not a record, `id` is
 not the entry id its file name keys, or any lane, issue record, status, claim
 or claim id it holds is absent or malformed

 @example
 ```ts
 const entry = attributionEntryOf({ name: 'Whiskers.json', parsed, },);
 ```
 */
export function attributionEntryOf(
  {
    name,
    parsed,
  }: {
    readonly name: ArtifactFileName;
    readonly parsed: unknown;
  },
): AttributionEntry {
  if (!isJsonRecord(parsed,))
    throw new ArtifactParseError({
      path: name,
      reason: 'a record',
    },);

  /**
   Entry id the pool admitted this artifact under, which is its file name.
   */
  const keyedId = entryIdOfArtifact({ name, },);

  // NAMED BY THE FILE, not by itself: an artifact whose `id` is not its file
  // name has no identity a refusal could trust, so this is the one check here
  // that starts its path with the file's base name. The same agreement the
  // pool's placement requires, held again because this is a second read.
  if ((keyedId === '') || (parsed.id !== keyedId))
    throw new ArtifactParseError({
      path: `${name}.id`,
      reason: 'the entry id the file is named for, since the pool admitted the artifact under that name',
    },);

  /**
   Identity that starts the path of each refusal of a field inside this
   artifact, equal to the `id` it records.
   */
  const entryId = keyedId;

  /**
   Generation this artifact records, or a named absence for one settled
   before the field existed.
   */
  const reading = readArtifactSchemaVersion({
    artifact: parsed,
    path: entryId,
  },);

  /**
   Where this artifact keeps its records, which is not the artifact itself on
   anything version 2 wrote.
   */
  const records = recordsHolderOf({
    parsed,
    entryId,
    reading,
  },);

  /**
   Spelling this artifact's own generation gave the critic record. An
   artifact with no version field predates version 1 and so predates every
   rename, which is the same spelling version 1 used.
   */
  const keys = (reading.kind === 'unversioned')
    ? CHUNK_SPELLED_KEYS
    : keyVocabularyOf({ version: reading.version, },);

  return {
    id: entryId,
    // ABSENT versus MALFORMED, and the difference decides the population. An
    // artifact settled before attribution existed has no such key, and the key
    // being OMITTED here is what makes the entry ineligible. A key that is
    // present but not an array is corruption, and letting it fall through to
    // the same omission would move a broken artifact into the pre-feature
    // population on the strength of its own breakage.
    //
    // ASKED OF THE HOLDER, NOT OF THE ARTIFACT. Asking the artifact root put
    // every version 2 artifact into the pre-feature population, which is the
    // one answer here that looks like an ordinary reading of an older corpus.
    ...(Object.hasOwn(
      records,
      keys.sliceCritics,
    )
      ? {
        sliceCritics: decodeSliceCritics({
          value: records[keys.sliceCritics],
          entryId,
          keys,
        },),
      }
      : {}),
    issues: readIssueViews({
      raw: records,
      entryId,
    },),
  };
}

/**
 One artifact that could not be read at all.

 @example
 ```ts
 const failure: MalformedArtifact = { name: 'Kitten.json', reason: 'Unexpected end of JSON input', };
 ```
 */
export type MalformedArtifact = {
  /**
   File that failed, so a reader can go look at it.
   */
  readonly name: string;

  /**
   Why it failed, named rather than summarized.
   */
  readonly reason: string;
};

/**
 Everything a run directory yielded, including what it could not.

 @example
 ```ts
 const { entries, malformed, } = await gatherAttributionEntries({ artifactsDir, policy, repository, },);
 ```
 */
export type AttributionGather = {
  /**
   Entries that parsed.
   */
  readonly entries: readonly AttributionEntry[];

  /**
   Artifacts that did not, held apart from the eligible and ineligible
   populations rather than folded into either.
   */
  readonly malformed: readonly MalformedArtifact[];
};

/**
 Reads every settled artifact into the shape the report needs.

 ISOLATED PER ARTIFACT, which is the difference between a loud failure and a
 useless one. The decoding (`attribution-decode.ts`) throws by design, and a bare
 `Promise.all` over the directory would let ONE bad file reject the whole
 gather: a single truncated artifact would mean no calibration at all for
 every other entry in the run. That is the same disproportion the writer
 avoids by not throwing on a telemetry invariant.

 Half-written artifacts are a real case rather than a hypothetical one. A pass
 killed at its hard cap can leave one, which is why `openSliceCache` already
 treats a half-written slice as absent, and `JSON.parse` on it raises a
 `SyntaxError` that has nothing to do with attribution.

 @param artifactsDir - directory the pass writes entries into

 @param policy - generation policy the entry read from its environment, which
 decides which generations the pool admits (`readPoolPolicy`)

 @param repository - checkout a required commit is resolved in and every
 recorded commit is placed against, which the entry names: the package's own
 in a run, a throwaway one in a case

 @returns Entries that parsed, and the artifacts that did not

 @example
 ```ts
 const { entries, malformed, } = await gatherAttributionEntries({ artifactsDir, policy, repository, },);
 ```
 */
export async function gatherAttributionEntries(
  {
    artifactsDir,
    policy,
    repository,
  }: {
    readonly artifactsDir: string;
    readonly policy: PoolPolicy;
    readonly repository: string;
  },
): Promise<AttributionGather> {
  /**
   One directory listing, shared with the census.

   Taken once and threaded through, because the accumulation writes into this
   directory continuously: a second listing inside the census would classify a
   different set of files from the one this reader goes on to read.
   */
  const listed = await listArtifactFiles({ artifactsDir, },);

  /**
   Artifact file names.
   */
  const names = keepEligible({
    names: listed,
    eligible: await resolvePool({
      artifactsDir,
      names: listed,
      policy,
      repository,
    },),
  },);

  /**
   One outcome per artifact: the entry it yielded, or why it yielded none.
   */
  const outcomes = await Promise.all(names.map(async function readOne(name,): Promise<
    { readonly entry: AttributionEntry; } | { readonly failure: MalformedArtifact; }
    > {
    try {
      return {
        entry: attributionEntryOf({
          name,
          // THE READ IS INSIDE THE GUARD TOO, not only the parse. Opening was a
          // bare `readFile` until 2026-08-25, so a file that would not open
          // arrived here as an ordinary `Error` whose message quotes the whole
          // path, and the `catch` around it could only answer `refused by Error`. This
          // names the filesystem code instead, and names the file by base name.
          parsed: await readRunJson({
            path: join(
              artifactsDir,
              name,
            ),
          },),
        },),
      };
    }
    catch (error) {
      // Recorded rather than rethrown, and never swallowed: the reason travels
      // to the caller, which reports it beside the population it is missing
      // from.
      return {
        failure: {
          name,
          reason: refusalText({ error, },),
        },
      };
    }
  },),);

  return {
    entries: outcomes.flatMap(function toEntries(outcome,): readonly AttributionEntry[] {
      return ('entry' in outcome) ? [outcome.entry,] : [];
    },),
    malformed: outcomes.flatMap(function toFailures(outcome,): readonly MalformedArtifact[] {
      return ('failure' in outcome) ? [outcome.failure,] : [];
    },),
  };
}

//endregion Attribution read
