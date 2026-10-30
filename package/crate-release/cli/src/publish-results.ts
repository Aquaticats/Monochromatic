import { caughtValueText } from '@monochromatic-dev/caught-value-core/ts';

import {
  crateNamePlaceholder,
  secondsPerMinute,
  versionPlaceholder,
} from './constants.ts';
import {
  foundValue,
  type Lookup,
  missingValue,
} from './lookup.ts';

/**
 Parse text as JSON without throwing.

 @param text - text under test

 @returns the parsed value, or a miss that carries the parser's complaint when the text is not JSON

 @example
 ```ts
 parseJson('{"a":1}');
 ```
 */
function parseJson(text: string): Lookup<unknown> {
  try {
    return foundValue(JSON.parse(text) as unknown);
  }
  catch (error) {
    return missingValue(`The text is not JSON: ${caughtValueText(error)}`);
  }
}

/**
 Check that a value is a plain object.

 @param value - value under test

 @returns true for objects other than null and arrays

 @example
 ```ts
 isRecord({});
 ```
 */
function isRecord(value: unknown): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value));
}

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
   True when the workflow succeeded, crates.io has the version, and the release holds every expected asset.
   */
  readonly ok: boolean;
};

/**
 Format what was found after a publish and decide whether it is complete.

 @param watchStatus - exit status of `gh run watch --exit-status`

 @param version - released version

 @param createdAt - crates.io creation time, or a miss when crates.io does not have the version

 @param tag - release tag

 @param expectedAssets - asset names the release must hold

 @param release - asset names the release holds, or a miss that carries the `gh` error when the release could not be read

 @returns the lines to print and whether everything is in place

 @example
 ```ts
 formatPublishResult({ watchStatus: 0, version: '1.0.0', createdAt: foundValue('x'), tag: 't', expectedAssets: [], release: foundValue([]) });
 ```
 */
export function formatPublishResult({
  watchStatus,
  version,
  createdAt,
  tag,
  expectedAssets,
  release,
}: {
  readonly watchStatus: number;
  readonly version: string;
  readonly createdAt: Lookup<string>;
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
    ...(watchStatus === 0 ? [] : [`workflow run exited ${String(watchStatus)}`]),
    ...(createdAt.found ? [] : [`${version} is not on crates.io`]),
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
      `watch exit ${String(watchStatus)}`,
      `crates.io ${version}: ${createdAt.found ? createdAt.value : 'not found'}`,
      `release ${tag} assets: ${assetText}`,
      problems.length === 0 ? 'PUBLISHED' : `PUBLISH CHECK FAILED: ${problems.join('; ')}`,
    ],
    ok: problems.length === 0,
  };
}
