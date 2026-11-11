import {
  crateNamePlaceholder,
  secondsPerMinute,
  versionPlaceholder,
} from './constants.ts';
import {
  isRecord,
  parseJson,
} from './json-value.ts';
import {
  foundValue,
  type Lookup,
  missingValue,
} from './lookup.ts';
import {
  describeRunFailures,
  describeRunOutcome,
  type RunOutcome,
  runPassed,
} from './run-outcome.ts';

/**
 Check that a listed workflow run was built from a commit.

 @param run - one entry of the run listing

 @param sha - lower-case commit hash or prefix of it

 @returns true when the entry has a numeric `databaseId` and a `headSha` that starts with `sha`

 @example
 ```ts
 isRunOfCommit({ run: { databaseId: 7, headSha: 'abc' }, sha: 'ab' });
 ```
 */
function isRunOfCommit({
  run,
  sha,
}: {
  readonly run: unknown;
  readonly sha: string;
}): boolean {
  return isRecord(run)
    && ((typeof run.headSha) === 'string')
    && ((typeof run.databaseId) === 'number')
    && (run.headSha as string).toLowerCase()
      .startsWith(sha);
}

/**
 Pick the workflow run that was built from a commit.

 @param listing - JSON text printed by `gh run list --json databaseId,headSha`, newest run first

 @param sha - lower-case commit hash or prefix of it

 @returns the run identifier of the first listed run whose hash starts with `sha`, or a miss when the listing is not
 JSON, not a list, or holds no such run

 @example
 ```ts
 selectRunId({ listing: '[{"databaseId":7,"headSha":"abc"}]', sha: 'abc' });
 ```
 */
export function selectRunId({
  listing,
  sha,
}: {
  readonly listing: string;
  readonly sha: string;
}): Lookup<string> {
  /**
   Parsed listing.
   */
  const runs = parseJson(listing);
  if (!runs.found)
    return runs;
  if (!Array.isArray(runs.value))
    return missingValue('The run listing is not a list.');
  /**
   The first run of the commit, when the listing holds one.
   */
  const run: unknown = (runs.value as readonly unknown[]).find(function isWanted(entry) {
    return isRunOfCommit({
      run: entry,
      sha
    });
  });
  return isRecord(run)
    ? foundValue(String(run.databaseId))
    : missingValue(`No listed run was built from ${sha}.`);
}

/**
 Read when crates.io created a version from its API answer.

 @param body - JSON text printed by the crates.io version endpoint

 @returns the `created_at` of the `version` object, or a miss when the answer is an error, not JSON, or lacks it

 @example
 ```ts
 parseCratesIoCreatedAt('{"version":{"created_at":"2026-10-29T12:00:00+00:00"}}');
 ```
 */
export function parseCratesIoCreatedAt(body: string): Lookup<string> {
  /**
   Parsed answer.
   */
  const answer = parseJson(body);
  if (!answer.found)
    return answer;
  if ((!isRecord(answer.value)) || (!isRecord(answer.value
    .version)))
    return missingValue('The answer has no version object.');
  /**
   Creation time.
   */
  const createdAt = answer.value
    .version
    .created_at;
  return (typeof createdAt) === 'string'
    ? foundValue(createdAt as string)
    : missingValue('The version has no created_at text.');
}

/**
 List the asset names of a GitHub release from its JSON.

 @param body - JSON text printed by `gh release view --json assets`

 @returns the asset names in order; empty when the text is not JSON or holds no assets

 @example
 ```ts
 parseAssetNames('{"assets":[{"name":"a.tar.gz"}]}');
 ```
 */
export function parseAssetNames(body: string): readonly string[] {
  /**
   Parsed answer.
   */
  const answer = parseJson(body);
  if ((!answer.found) || (!isRecord(answer.value))
    || (!Array.isArray(answer.value
      .assets)))
    return [];
  return (answer.value
    .assets as readonly unknown[])
    .flatMap(function nameOf(asset) {
      return isRecord(asset) && ((typeof asset.name) === 'string') ? [asset.name as string] : [];
    });
}

/**
 Fill the crate and version placeholders of a name template.

 @param template - text with `{crate}` and `{version}` placeholders

 @param crateName - crate name

 @param version - release version

 @returns the filled text; characters of the values that look like replacement patterns or placeholders stay literal

 @example
 ```ts
 expandTemplate({ template: '{crate}-v{version}', crateName: 'c', version: '1.0.0' });
 ```
 */
export function expandTemplate({
  template,
  crateName,
  version,
}: {
  readonly template: string;
  readonly crateName: string;
  readonly version: string;
}): string {
  // One pass over the template: a value is never searched for placeholders again, so it stays literal.
  return template.split(crateNamePlaceholder)
    .map(function fillVersion(part) {
      return part.split(versionPlaceholder)
        .join(version);
    })
    .join(crateName);
}

/**
 Find the expected assets a release lacks.

 @param expected - asset names the release must hold

 @param actual - asset names it holds

 @returns the expected names that are absent, in expected order; names compare whole

 @example
 ```ts
 findMissingAssets({ expected: ['a', 'b'], actual: ['a'] });
 ```
 */
export function findMissingAssets({
  expected,
  actual,
}: {
  readonly expected: readonly string[];
  readonly actual: readonly string[];
}): readonly string[] {
  /**
   Names the release holds, for constant-time lookup.
   */
  const held = new Set(actual);
  return expected.filter(function isAbsent(name) {
    return !held.has(name);
  });
}

/**
 Write a number of seconds in words, in whole minutes and seconds.

 @param totalSeconds - non-negative whole number of seconds

 @returns text such as `15 minutes`, `1 minute 30 seconds` or `45 seconds`

 @example
 ```ts
 describeDuration(900);
 ```
 */
export function describeDuration(totalSeconds: number): string {
  /**
   Whole minutes.
   */
  const minutes = Math.floor(totalSeconds / secondsPerMinute);
  /**
   Seconds left after the minutes.
   */
  const seconds = totalSeconds % secondsPerMinute;
  return [
    minutes === 0 ? '' : `${String(minutes)} ${minutes === 1 ? 'minute' : 'minutes'}`,
    seconds === 0 ? '' : `${String(seconds)} ${seconds === 1 ? 'second' : 'seconds'}`,
  ].filter(function isPresent(part) {
    return part !== '';
  })
    .join(' ');
}

/**
 Outcome of the publish check, ready to print.
 */
export type PublishReport = {
  readonly lines: readonly string[];
  /**
   True when the workflow run ended with success, crates.io has the version, and the release holds every expected
   asset.
   */
  readonly ok: boolean;
};

/**
 List why a workflow run is not a pass.

 @param workflow - workflow file of the run

 @param runId - numeric identifier of the run

 @param outcome - what the run says about itself, or a miss that says why it could not be followed to its end

 @returns one reason per way the run keeps the publish from being complete; empty when the run ended with success

 @example
 ```ts
 problemsOfRun({ workflow: 'cargo-publish.yml', runId: '42', outcome });
 ```
 */
function problemsOfRun({
  workflow,
  runId,
  outcome,
}: {
  readonly workflow: string;
  readonly runId: string;
  readonly outcome: Lookup<RunOutcome>;
}): readonly string[] {
  if (!outcome.found)
    return [`${workflow} run ${runId} did not finish: ${outcome.reason}`];
  if (runPassed(outcome.value))
    return [];
  /**
   Conclusion the run ended with.
   */
  const { conclusion } = outcome.value;
  return [`${workflow} run ${runId} concluded ${conclusion}`];
}

/**
 Format what was found after a publish and decide whether it is complete.

 The verdict on the workflow run comes from the run's own answer (see `RunOutcome`), never from the exit status of a
 watcher: a watcher fails for its own reasons too, and a run that passed must not be reported as failed.

 @param workflow - workflow file of the run that built the crate and its archives

 @param runId - numeric identifier of that run

 @param outcome - what the run says about itself, or a miss that says why it could not be followed to its end

 @param version - released version

 @param createdAt - crates.io creation time, or a miss when crates.io does not have the version

 @param skippedCratesIoReason - why crates.io was not asked, for a version that is never published there; when given,
 the report leaves crates.io out of the check and says so, and the result depends on the workflow run and the release
 only

 @param tag - release tag

 @param expectedAssets - asset names the release must hold

 @param release - asset names the release holds, or a miss that carries the `gh` error when the release could not be read

 @returns the lines to print and whether everything is in place

 @example
 ```ts
 formatPublishResult({ workflow: 'cargo-publish.yml', runId: '42', outcome, version: '1.0.0', createdAt: foundValue('x'), tag: 't', expectedAssets: [], release: foundValue([]) });
 ```
 */
export function formatPublishResult({
  workflow,
  runId,
  outcome,
  version,
  createdAt,
  skippedCratesIoReason,
  tag,
  expectedAssets,
  release,
}: {
  readonly workflow: string;
  readonly runId: string;
  readonly outcome: Lookup<RunOutcome>;
  readonly version: string;
  readonly createdAt: Lookup<string>;
  readonly skippedCratesIoReason?: string;
  readonly tag: string;
  readonly expectedAssets: readonly string[];
  readonly release: Lookup<readonly string[]>;
}): PublishReport {
  /**
   Expected assets the release lacks.
   */
  const missing = findMissingAssets({
    expected: expectedAssets,
    actual: release.found ? release.value : []
  });
  /**
   Reasons the check failed, in report order.
   */
  const problems = [
    ...problemsOfRun({
      workflow,
      runId,
      outcome
    }),
    ...((skippedCratesIoReason !== undefined) || createdAt.found ? [] : [`${version} is not on crates.io`]),
    ...(missing.length === 0 ? [] : [`missing release assets: ${missing.join(', ')}`]),
  ];
  /**
   What to show after `assets:`.
   */
  const assetText = release.found ? (release.value
    .length
    === 0 ? 'none' : release.value
      .join(', ')) : release.reason;
  return {
    lines: [
      outcome.found
        ? describeRunOutcome({
          workflow,
          runId,
          outcome: outcome.value
        })
        : `${workflow} run ${runId}: ${outcome.reason}`,
      ...(outcome.found ? describeRunFailures(outcome.value) : []),
      skippedCratesIoReason === undefined
        ? `crates.io ${version}: ${createdAt.found ? createdAt.value : 'not found'}`
        : `crates.io ${version}: skipped, ${skippedCratesIoReason}`,
      `release ${tag} assets: ${assetText}`,
      problems.length === 0 ? `PUBLISHED ${tag}` : `PUBLISH CHECK FAILED: ${problems.join('; ')}`,
    ],
    ok: problems.length === 0,
  };
}
