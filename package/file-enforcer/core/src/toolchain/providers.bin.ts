/**
 The `bin-sourced` route: release-tag resolution, `bin` configuration
 seeding, and `bin install` per declared primary binary.
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  existsSync,
  mkdirSync,
  readFileSync,
  writeFileSync,
} from 'node:fs';
import { stat, } from 'node:fs/promises';
import {
  basename,
  dirname,
} from 'node:path';
import {
  run,
  type RunResult,
} from './exec.ts';
import { homeDirFromEnv, } from './paths.ts';
import {
  artifactForHost,
  type ProviderContext,
} from './providers.context.ts';
import {
  binCliPath,
  resolveGithubToken,
} from './providers.environment.ts';
import { l, } from '../logger.ts';

//region Release tag

/**
 Address prefix of every GitHub release download.
 */
const GITHUB_PREFIX = 'https://github.com/';

/**
 Path segments `<owner>/<repo>/releases/download/<tag>/<asset>` splits into
 at least: owner, repo, `releases`, `download`, tag, and the asset (which
 may be empty when the URL ends at the tag's slash).
 */
const MIN_RELEASE_DOWNLOAD_SEGMENTS = 6;

/**
 Resolves the release-tag URL behind one artifact download URL
 (`https://github.com/<owner>/<repo>/releases/download/<tag>/<asset>` to
 `https://github.com/<owner>/<repo>/releases/tag/<tag>`).

 @param artifactUrl - Artifact download URL from `toolchain.lock.json`

 @returns Release-tag URL for `bin install`, or `null` when the URL is not a
 GitHub release download

 @example
 ```ts
 releaseTagUrl('https://github.com/example/demo/releases/download/v1.0.0/demo.tar.gz',);
 // 'https://github.com/example/demo/releases/tag/v1.0.0'
 ```
 */
export function releaseTagUrl(artifactUrl: string,): string | null {
  if (!artifactUrl.startsWith(GITHUB_PREFIX,))
    return null;
  /**
   Segments after the host: owner, repo, `releases`, `download`, tag, asset.
   */
  const segments = artifactUrl
    .slice(GITHUB_PREFIX.length,)
    .split('/',);
  /**
   Owner, repository, release area marker, download marker, and tag.
   */
  const [
    owner = '',
    repo = '',
    releases,
    download,
    tag = '',
  ] = segments;
  if ((segments.length < MIN_RELEASE_DOWNLOAD_SEGMENTS)
    || (releases !== 'releases')
    || (download !== 'download'))
    return null;
  if (owner === '')
    return null;
  if (repo === '')
    return null;
  if (tag === '')
    return null;
  return `${GITHUB_PREFIX}${owner}/${repo}/releases/tag/${decodeURIComponent(tag,)}`;
}

//endregion Release tag

//region bin configuration

/**
 Spaces per indentation level in the seeded `bin` configuration file.
 */
const BIN_CONFIG_INDENT_SPACES = 4;

/**
 Whether a parsed JSON value is a plain object.

 @param value - Parsed JSON value

 @returns True for an object that is neither `null` nor an array

 @example
 ```ts
 isRecord({ bins: {}, },); // true
 ```
 */
function isRecord(value: unknown,): value is Record<string, unknown> {
  if (value === null)
    return false;
  if ((typeof value) !== 'object')
    return false;
  return !Array.isArray(value,);
}

/**
 Reads the settings an earlier `bin` configuration file holds.

 @param configPath - Configuration file to read

 @returns The file's settings; empty when the file is absent or malformed

 @example
 ```ts
 const existing = readBinConfig('/tmp/bin/config.json',);
 ```
 */
function readBinConfig(configPath: string,): Readonly<Record<string, unknown>> {
  /**
   Logger tagged for this read.
   */
  const log = tagged({
    tag: readBinConfig.name,
    l,
  },);
  if (!existsSync(configPath,))
    return {};
  try {
    /**
     Parsed file content.
     */
    const parsed: unknown = JSON.parse(readFileSync(
      configPath,
      'utf8',
    ),);
    return isRecord(parsed,)
      ? parsed
      : {};
  }
  catch (error) {
    // Only a malformed file is replaced; any other failure is the caller's.
    if (!(error instanceof SyntaxError))
      throw error;
    log.info(`replacing the malformed bin configuration ${configPath}: ${caughtValueText(error,)}`,);
    return {};
  }
}

/**
 Pre-seeds the `bin` CLI configuration so no interactive prompt appears:
 `bin` asks for its download directory until `default_path` is configured.
 The seeded file is the scope's own (`BIN_CONFIG` target,
 `paths.binConfigFile`), so two scopes never share one `default_path` and one
 install database.

 @param downloadDir - Download directory to record as `default_path`

 @param configPath - The scope's configuration file path

 @returns The configuration path the scope runs `bin` with

 @example
 ```ts
 seedBinConfig({ downloadDir: '/tmp/toolchain/bin', configPath: '/tmp/toolchain/bin.json', },);
 ```
 */
export function seedBinConfig(
  {
    downloadDir,
    configPath,
  }: {
    readonly downloadDir: string;
    readonly configPath: string;
  },
): string {
  /**
   Existing settings kept, with `default_path` converged to the toolchain.
   */
  const existing = readBinConfig(configPath,);
  mkdirSync(
    dirname(configPath,),
    { recursive: true },
  );
  writeFileSync(
    configPath,
    `${
      JSON.stringify(
        {
          ...existing,
          default_path: downloadDir,
          bins: existing.bins ?? {},
        },
        null,
        BIN_CONFIG_INDENT_SPACES,
      )
    }\n`,
    'utf8',
  );
  return configPath;
}

//endregion bin configuration

//region bin install

/**
 Result reported when a tool declares no primary binary to install.
 */
const NOTHING_INSTALLED: RunResult = {
  stdout: '',
  stderr: '',
  code: 0,
};

/**
 Error code of a path that does not exist.
 */
const MISSING_ENTRY_CODE = 'ENOENT';

/**
 Whether a directory exists at a path.

 @param path - Path to inspect

 @returns False when nothing exists there or it is not a directory

 @example
 ```ts
 await isDirectory('/tmp',); // true
 ```
 */
async function isDirectory(path: string,): Promise<boolean> {
  try {
    /**
     Kind of the entry at the path.
     */
    const kind = await stat(path,);
    return kind.isDirectory();
  }
  catch (error) {
    if (!Error.isError(error,))
      throw error;
    if (('code' in error) && (error.code === MISSING_ENTRY_CODE))
      return false;
    throw error;
  }
}

/**
 Installs one primary binary through the ordered `-n` patterns: nested
 archive file, root-level archive file, then the raw asset with a destination
 rename; the first attempt that succeeds ends the sequence.

 @param cli - Absolute `bin` CLI path

 @param tagUrl - Release-tag URL `bin install` resolves

 @param asset - Asset file name inside the release

 @param name - Primary binary name to install

 @param env - Environment `bin` runs under

 @returns The first successful run, else the last failed one

 @throws Error when no attempt could be run

 @example
 ```ts
 const outcome = await installPrimary({ cli, tagUrl, asset: 'demo.tar.gz', name: 'demo', env, },);
 ```
 */
async function installPrimary(
  {
    cli,
    tagUrl,
    asset,
    name,
    env,
  }: {
    readonly cli: string;
    readonly tagUrl: string;
    readonly asset: string;
    readonly name: string;
    readonly env: Readonly<Record<string, string>>;
  },
): Promise<RunResult> {
  /**
   Candidate `-n` patterns, in the order they are tried.
   */
  const attempts = [
    [
      '--name',
      `${asset}/**/${name}`,
    ],
    [
      '--name',
      `${asset}/${name}`,
    ],
    [
      '--name',
      asset,
      name,
    ],
  ];
  /**
   Runs made so far, in attempt order.
   */
  const outcomes: RunResult[] = [];
  for (const attempt of attempts) {
    /**
     Run of this attempt.
     */
    // oxlint-disable-next-line no-await-in-loop -- the patterns are ordered fallbacks; the next one runs only after the previous failed
    const outcome = await run({
      command: cli,
      args: [
        'install',
        '--force',
        tagUrl,
        ...attempt,
      ],
      env,
    },);
    if (outcome.code === 0)
      return outcome;
    outcomes.push(outcome,);
  }
  /**
   Last failed run, which names the final fallback's error.
   */
  const last = outcomes.at(-1,);
  if (last === undefined)
    throw new Error(`no bin install attempt ran for ${name} from ${tagUrl}`,);
  return last;
}

/**
 Installs one `bin-sourced` tool:
 `bin install <release-tag-url> -n <asset> glob <bin>` per declared primary
 binary, with fallbacks for root-level archive files and raw release assets.

 @param context - Provider context

 @returns `bin list` style descriptions of the installed files

 @throws Error when the artifact is not a GitHub release download, a binary
 fails to install, or the install replaces the toolchain bin directory

 @example
 ```ts
 const outcome = await installBinSourced({ entry, paths, env, extraDirs: [], },);
 ```
 */
export async function installBinSourced(
  context: ProviderContext,
): Promise<RunResult> {
  /**
   Plan entry, path set, environment and scope plan this install reads.
   */
  const {
    entry,
    paths,
    env,
    tools,
  } = context;
  /**
   Verified artifact for the host platform, whose URL carries the tag.
   */
  const artifact = artifactForHost(entry,);
  /**
   Release-tag URL `bin install` resolves to the locked release.
   */
  const tagUrl = releaseTagUrl(artifact.url,);
  if (tagUrl === null)
    throw new Error(`cannot derive a release-tag URL from ${artifact.url}`,);
  /**
   Asset file name inside the release.
   */
  const asset = basename(artifact.url,);
  /**
   Alias map: extra exposure names sharing one installed file.
   */
  const aliases = entry.options
    .binAliases
    ?? {};
  /**
   Primary binary names `bin install` copies (aliases link to these).
   */
  const primaries = entry.bins
    .filter(
    function isPrimary(name,): boolean {
      return aliases[name] === undefined;
    },
    );
  seedBinConfig({
    downloadDir: paths.toolchainBinDir,
    configPath: paths.binConfigFile,
  },);
  /**
   Absolute `bin` CLI at the bootstrap's install directory.
   */
  const cli = binCliPath({
    paths,
    env,
    tools,
  },);
  /**
   GitHub token so release lookups do not hit the unauthenticated API quota.
   */
  const token = await resolveGithubToken({ env, },);
  /**
   `bin` environment: the seeded home, the scope's own `bin` database and
   `default_path`, and the token when one exists.
   */
  const binEnv = {
    ...process.env,
    HOME: homeDirFromEnv({ env, }),
    BIN_CONFIG: paths.binConfigFile,
    ...(token === null
      ? {}
      : { GITHUB_TOKEN: token, }),
  };
  /**
   Last observed run result.
   */
  const results: RunResult[] = [];
  for (const name of primaries) {
    /**
     Run that installed this binary, or the last failed attempt.
     */
    // oxlint-disable-next-line no-await-in-loop -- one `bin` install database serves every binary, so installs must not overlap
    const result = await installPrimary({
      cli,
      tagUrl,
      asset,
      name,
      env: binEnv,
    },);
    if (result.code !== 0) {
      throw new Error(
        `bin install failed for ${
          entry.id
        } (${
          name
        }): ${
          result.stderr || result.stdout
        }`,
      );
    }
    results.push(result,);
  }
  // A payload whose extracted name collides with a directory in the download
  // directory makes `bin install --force` rename that directory aside (the
  // tree-sitter `.gz` incident, 2026-10-28): fail loudly instead of leaving
  // the toolchain without its bin directory.
  if (!(await isDirectory(paths.toolchainBinDir,))) {
    throw new Error(
      `bin install for ${
        entry.id
      } replaced the toolchain bin directory ${
        paths.toolchainBinDir
      } with a file; restore it from its '.bin.old' sibling`,
    );
  }
  return results.at(-1,) ?? NOTHING_INSTALLED;
}

//endregion bin install
