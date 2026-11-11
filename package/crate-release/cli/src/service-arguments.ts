import {
  decimalDigits,
  usesOnly,
} from '@monochromatic-dev/shell-argument-core/ts/character-set.ts';

import {
  cratesIoApiBase,
  releaseWorkflow,
  releaseWorkflowInputs,
} from './constants.ts';
import { CrateReleaseError } from './error.ts';
import {
  parseCrateName,
  parseManifestPath,
  parseRepository,
  parseSha,
} from './validate.ts';
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
 Arguments of `gh workflow run` that start the release workflow for a crate.

 Each input travels as one `--raw-field=<name>=<value>` argument, so a note that starts with a hyphen or holds shell
 syntax stays inside its value, and the workflow receives it as text.

 @param crate - crate name, which is also the workflow's `crate` choice

 @param from - version the crate holds now

 @param to - version to release

 @param note - one-line summary that ends the commit subject

 @param dryRun - true to add the `dry-run` input, which stops the workflow before it pushes or publishes; false leaves
 the input out, so the workflow's own default applies

 @param repo - `owner/name`; absent to use the repository of the working directory

 @returns the arguments after `gh`; the branch is left to `gh`, which uses the default branch

 @example
 ```ts
 buildReleaseDispatchArguments({ crate: 'c', from: '0.1.0', to: '0.1.1', note: 'n', dryRun: false });
 ```
 */
export function buildReleaseDispatchArguments({
  crate,
  from,
  to,
  note,
  dryRun,
  repo,
}: {
  readonly crate: string;
  readonly from: string;
  readonly to: string;
  readonly note: string;
  readonly dryRun: boolean;
  readonly repo?: string;
}): readonly string[] {
  return [
    'workflow',
    'run',
    releaseWorkflow,
    `--raw-field=${releaseWorkflowInputs.crate}=${crate}`,
    `--raw-field=${releaseWorkflowInputs.from}=${from}`,
    `--raw-field=${releaseWorkflowInputs.to}=${to}`,
    `--raw-field=${releaseWorkflowInputs.note}=${note}`,
    ...(dryRun ? [`--raw-field=${releaseWorkflowInputs.dryRun}=true`] : []),
    ...repoArguments(repo),
  ];
}

/**
 Arguments of `gh api graphql` that send a request body kept in a file.

 @param requestFile - absolute path of the file that holds the JSON request

 @returns the arguments after `gh`

 @throws CrateReleaseError when the path is not absolute or starts with a hyphen that `gh` would read as an option

 @example
 ```ts
 buildGraphqlInputArguments('/runner/temp/request.json');
 ```
 */
export function buildGraphqlInputArguments(requestFile: string): readonly string[] {
  if ((!requestFile.startsWith('/')) || requestFile.includes('\0'))
    throw new CrateReleaseError(
      `The request file must be an absolute path without NUL, got ${JSON.stringify(requestFile)}.`,
    );
  return [
    'api',
    'graphql',
    `--input=${requestFile}`,
  ];
}

/**
 Arguments of `gh run view` that read when a run was created, how it stands and what each of its jobs concluded, as
 JSON.

The answer of this call is the only verdict a wait accepts about a run: `gh run watch --exit-status` exits 1 both for a
run that concluded otherwise than success and for a failure of the watcher itself, so its status cannot be read as the
run's.

 @param runId - numeric run identifier

 @param repo - `owner/name`; absent to use the repository of the working directory

 @returns the arguments after `gh`

 @throws CrateReleaseError when the run identifier is not all digits

 @example
 ```ts
 buildRunViewArguments({ runId: '123' });
 ```
 */
export function buildRunViewArguments({
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
    'view',
    runId,
    '--json=createdAt,status,conclusion,jobs',
    ...repoArguments(repo),
  ];
}

/**
 Arguments of `gh run list` for the latest manually started runs of a workflow, as JSON with their creation times.

 The list is filtered by the caller in code, so no time or hash is interpolated into a query.

 @param workflow - workflow file name

 @param limit - how many recent runs to list

 @param repo - `owner/name`; absent to use the repository of the working directory

 @returns the arguments after `gh`

 @example
 ```ts
 buildDispatchedRunListArguments({ workflow: 'cargo-publish.yml', limit: 10 });
 ```
 */
export function buildDispatchedRunListArguments({
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
    '--event=workflow_dispatch',
    `--limit=${String(limit)}`,
    '--json=databaseId,headSha,createdAt',
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

/**
 Placeholder pair that `gh api` replaces with the owner and name of the repository of its working directory.
 */
const workingDirectoryRepository = '{owner}/{repo}';

/**
 Media type that makes the contents endpoint of the GitHub API answer with the file's bytes instead of JSON.
 */
const rawContentsMediaType = 'application/vnd.github.raw+json';

/**
 Arguments of `gh api` that read one file as it was at one commit, answering with the file's text.

 `gh api` has no repository option, so a named repository is written into the endpoint, and an unnamed one leaves the
 `{owner}/{repo}` placeholders for `gh` to fill from its working directory. The path is percent-encoded segment by
 segment and the commit must be hexadecimal, so neither can add a query parameter or leave the endpoint.

 @param manifestPath - repository-relative path of the file, which must be a `Cargo.toml`

 @param sha - commit to read the file at, as a hexadecimal hash

 @param repo - `owner/name`; absent to use the repository of the working directory

 @returns the arguments after `gh`

 @throws CrateReleaseError when the path is not a manifest path inside the repository, the commit is not a hexadecimal
 hash, or the repository is not of the form `owner/name`

 @example
 ```ts
 buildManifestContentsArguments({ manifestPath: 'package/a/Cargo.toml', sha: 'abc1234' });
 ```
 */
export function buildManifestContentsArguments({
  manifestPath,
  sha,
  repo,
}: {
  readonly manifestPath: string;
  readonly sha: string;
  readonly repo?: string;
}): readonly string[] {
  parseManifestPath({
    option: 'the manifest path',
    value: manifestPath
  });
  return [
    'api',
    `repos/${
      repo === undefined
        ? workingDirectoryRepository
        : parseRepository({
          option: 'the repository',
          value: repo
        })
    }/contents/${
      manifestPath.split('/')
        .map(function encodeSegment(segment) {
          return encodeURIComponent(segment);
        })
        .join('/')
    }?ref=${
      parseSha({
        option: 'the commit',
        value: sha
      })
    }`,
    `--header=Accept: ${rawContentsMediaType}`,
  ];
}
