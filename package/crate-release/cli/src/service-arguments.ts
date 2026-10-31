import {
  decimalDigits,
  usesOnly,
} from '@monochromatic-dev/shell-argument-core/ts/character-set.ts';

import { cratesIoApiBase } from './constants.ts';
import { CrateReleaseError } from './error.ts';
import { parseCrateName } from './validate.ts';
import { assertReleaseVersion } from './version.ts';

/**
 The `--repo` argument of a `gh` call, when a repository is named.

 @param repo - `owner/name`; absent to let `gh` read it from the working directory

 @returns zero or one argument

 @example
 ```ts
 repoArguments('owner/name');
 ```
 */
function repoArguments(repo?: string): readonly string[] {
  return repo === undefined ? [] : [`--repo=${repo}`];
}

/**
 Arguments of `gh run list` for the latest runs of a workflow, as JSON.

 The list is filtered by the caller in code, never by a query program, so no commit hash is interpolated into a query.

 @param workflow - workflow file name

 @param limit - how many recent runs to list

 @param repo - `owner/name`; absent to use the repository of the working directory

 @returns the arguments after `gh`; each option carries its value after `=`, so a value that starts with a hyphen
 stays inside its option

 @example
 ```ts
 buildRunListArguments({ workflow: 'cargo-publish.yml', limit: 10 });
 ```
 */
export function buildRunListArguments({
  workflow,
  limit,
  repo,
}: {
  readonly workflow: string;
  readonly limit: number;
  readonly repo?: string;
}): readonly string[] {
  return [
    'run',
    'list',
    `--workflow=${workflow}`,
    `--limit=${String(limit)}`,
    '--json=databaseId,headSha',
    ...repoArguments(repo),
  ];
}

/**
 Arguments of `gh run watch` that wait for a run and exit with its result.

 @param runId - numeric run identifier

 @param repo - `owner/name`; absent to use the repository of the working directory

 @returns the arguments after `gh`

 @throws CrateReleaseError when the run identifier is not all digits

 @example
 ```ts
 buildRunWatchArguments({ runId: '123' });
 ```
 */
export function buildRunWatchArguments({
  runId,
  repo,
}: {
  readonly runId: string;
  readonly repo?: string;
}): readonly string[] {
  if (!usesOnly({
    text: runId,
    allowed: decimalDigits
  }))
    throw new CrateReleaseError(`A run identifier must be all digits, got ${JSON.stringify(runId)}.`);
  return [
    'run',
    'watch',
    runId,
    '--exit-status',
    ...repoArguments(repo),
  ];
}

/**
 Arguments of `gh release view` that list a release's assets as JSON.

 @param tag - release tag

 @param repo - `owner/name`; absent to use the repository of the working directory

 @returns the arguments after `gh`

 @throws CrateReleaseError when the tag is empty, holds NUL, or starts with a hyphen that `gh` would read as an option

 @example
 ```ts
 buildReleaseViewArguments({ tag: 'c-v1.0.0' });
 ```
 */
export function buildReleaseViewArguments({
  tag,
  repo,
}: {
  readonly tag: string;
  readonly repo?: string;
}): readonly string[] {
  if ((tag === '') || tag.includes('\0')
    || tag.startsWith('-'))
    throw new CrateReleaseError(
      `A release tag must be non-empty, free of NUL and not start with a hyphen, got ${JSON.stringify(tag)}.`,
    );
  return [
    'release',
    'view',
    tag,
    '--json=assets',
    ...repoArguments(repo),
  ];
}

/**
 Address of one version of a crate in the crates.io API.

 @param crateName - crate name

 @param version - release version

 @returns the URL, with each part percent-encoded

 @throws CrateReleaseError when the crate name or version has characters that could change the path

 @example
 ```ts
 cratesIoUrl({ crateName: 'c', version: '1.0.0' });
 ```
 */
export function cratesIoUrl({
  crateName,
  version,
}: {
  readonly crateName: string;
  readonly version: string;
}): string {
  parseCrateName({
    option: 'the crate name',
    value: crateName
  });
  assertReleaseVersion({
    option: 'the version',
    value: version
  });
  return `${cratesIoApiBase}/${encodeURIComponent(crateName)}/${encodeURIComponent(version)}`;
}

/**
 Arguments of the `curl` call that asks crates.io about one version.

 @param crateName - crate name

 @param version - release version

 @param userAgent - caller name, which crates.io requires

 @param maxTimeSeconds - seconds before `curl` gives up

 @returns the arguments after `curl`

 @throws CrateReleaseError when the crate name or version has characters that could change the path

 @example
 ```ts
 buildCratesIoArguments({ crateName: 'c', version: '1.0.0', userAgent: 'check', maxTimeSeconds: 30 });
 ```
 */
export function buildCratesIoArguments({
  crateName,
  version,
  userAgent,
  maxTimeSeconds,
}: {
  readonly crateName: string;
  readonly version: string;
  readonly userAgent: string;
  readonly maxTimeSeconds: number;
}): readonly string[] {
  return [
    '--silent',
    '--show-error',
    '--max-time',
    String(maxTimeSeconds),
    '--user-agent',
    userAgent,
    cratesIoUrl({
      crateName,
      version
    }),
  ];
}
