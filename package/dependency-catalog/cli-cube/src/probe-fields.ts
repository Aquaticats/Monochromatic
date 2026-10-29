/**
 Field-level probes used by the probe orchestrator.
 
 Each field probe fetches a single attribute (registry manifest, downloads
 count, Linguist languages, last-commit date) and writes its result to the
 file cache with the appropriate TTL. Pure parser helpers live in
 {@link ./probe-field-parsers.ts}; the orchestrator imports them through this
 module's compatibility re-exports.
 
 @example
 ```ts
 const pkg = await probePackageManifest({ npmName: 'preact', cache });
 const dl = await probeDownloads({ npmName: 'preact', cache });
 ```
 */

import spawn from 'nano-spawn';

import {
  MS_PER_DAY,
  MS_PER_SECOND,
} from '@monochromatic-dev/const-core/ts';

import {
  type Cache,
  CACHE_MISS,
} from './cache.ts';
import { caughtValueText as caughtErrorMessage, } from '@monochromatic-dev/caught-value-core/ts';
import type { NpmPackage, } from './probe-field-types.ts';

export {
  classifyLicense,
  parseRepository,
  REPO_UNPARSEABLE,
  resolveVersion,
  VERSION_UNRESOLVED,
} from './probe-field-parsers.ts';
export type {
  LicenseClass,
  NpmPackage,
  NpmVersion,
  RepositoryInfo,
} from './probe-field-types.ts';

//region Constants

/**
 Days of validity for fields that can change with upstream activity.
 */
const TTL_DAYS = 30;
/**
 TTL in ms for fields that can change with upstream activity.
 */
const TTL_MS = TTL_DAYS * MS_PER_DAY;
/**
 HTTP timeout in seconds; lets `gh api` complete on slow links without hanging an audit.
 */
const HTTP_TIMEOUT_SECONDS = 30;
/**
 Per-HTTP-request timeout in ms.
 */
const HTTP_TIMEOUT_MS = HTTP_TIMEOUT_SECONDS * MS_PER_SECOND;

//endregion Constants

//region HTTP helpers

/**
 Fetches a URL with a hard timeout, returning the JSON response.
 
 @param url - Absolute URL to fetch.
 
 @returns Parsed JSON value.
 
 @throws When request times out, network errors, or HTTP status is not 2xx.
 */
async function fetchJson<T,>(url: string,): Promise<T> {
  /**
   HTTP response from `fetch`; aborted if it doesn't complete within {@link HTTP_TIMEOUT_MS}.
   */
  const response = await fetch(
    url,
    {
      signal: AbortSignal.timeout(HTTP_TIMEOUT_MS,),
    },
  );
  if (!response.ok)
    throw new Error(`HTTP ${response.status
      .toString()} on ${url}`,);
  // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- response.json() is `unknown`; caller asserts T.
  return await response.json() as T;
}

/**
 Runs a `gh api` invocation and parses the result as JSON.
 
 @param path - API path, for example `repos/{owner}/{repo}/languages`.
 
 @returns Parsed JSON response; throws on non-zero exit or parse failure.
 
 @throws When `gh` exits non-zero due to rate limit, auth failure, or not found.
 */
async function ghApi<T,>(path: string,): Promise<T> {
  /**
   `gh api` subprocess result; `stdout` holds the JSON payload, throws on non-zero exit.
   */
  const result = await spawn(
    'gh',
    [
      'api',
      path,
    ],
    {
      timeout: HTTP_TIMEOUT_MS,
    },
  );
  // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- JSON.parse returns `any`; caller asserts T.
  return JSON.parse(result.stdout,) as T;
}

//endregion HTTP helpers

//region Field probes

/**
 Fetches the package-level npm registry manifest, including all versions and time metadata.
 
 @param npmName - Real npm package name.
 
 @param cache - File cache handle.
 
 @returns Package-level npm manifest.
 
 @example
 ```ts
 const pkg = await probePackageManifest({ npmName: 'preact', cache });
 ```
 */
export async function probePackageManifest(
  {
    npmName,
    cache,
  }: {
    readonly npmName: string;
    readonly cache: Cache;
  },
): Promise<NpmPackage> {
  /**
   Cached manifest, valid for {@link TTL_MS}; {@link CACHE_MISS} on cache miss.
   */
  const cached = await cache.read<NpmPackage>({
    name: npmName,
    version: '_pkg',
    field: 'registry',
    ttlMs: TTL_MS,
  },);
  if (cached !== CACHE_MISS)
    return cached;
  /**
   Percent-encoded npm name preserving `@` for scoped packages.
   */
  const encodedName = encodeURIComponent(npmName,)
    .replaceAll(
    '%40',
    '@',
    );
  /**
   Fresh manifest from npm registry; written to cache below before return.
   */
  const fetched = await fetchJson<NpmPackage>(
    `https://registry.npmjs.org/${encodedName}`,
  );
  await cache.write({
    name: npmName,
    version: '_pkg',
    field: 'registry',
    value: fetched,
  },);
  return fetched;
}

/**
 Fetches last-week downloads. Returns `0` on error (logged via
 {@link caughtErrorMessage}) or for niche packages.
 
 @param npmName - Real npm package name.
 
 @param cache - File cache handle.
 
 @returns Weekly download count.
 
 @example
 ```ts
 const dl = await probeDownloads({ npmName: 'preact', cache });
 ```
 */
export async function probeDownloads(
  {
    npmName,
    cache,
  }: {
    readonly npmName: string;
    readonly cache: Cache;
  },
): Promise<number> {
  /**
   Cached downloads payload from a prior run, if still within TTL.
   */
  const cached = await cache.read<{ downloads: number; }>({
    name: npmName,
    version: '_pkg',
    field: 'downloads',
    ttlMs: TTL_MS,
  },);
  if (cached !== CACHE_MISS)
    return cached.downloads;
  try {
    /**
     Percent-encoded npm name preserving `@` for scoped packages.
     */
    const encodedName = encodeURIComponent(npmName,)
      .replaceAll(
      '%40',
      '@',
      );
    /**
     Fresh downloads payload from npm; outer try swallows transient failures.
     */
    const fetched = await fetchJson<{ downloads: number; }>(
      `https://api.npmjs.org/downloads/point/last-week/${encodedName}`,
    );
    await cache.write({
      name: npmName,
      version: '_pkg',
      field: 'downloads',
      value: fetched,
    },);
    return fetched.downloads;
  }
  catch (error) {
    console.warn(
      `[deps-cube] npm downloads probe failed for ${npmName}: ${caughtErrorMessage(error,)}`,
    );
    return 0;
  }
}

/**
 Absence marker for {@link probeLanguages} meaning "Linguist data is
 unavailable for this repo"; never a language byte-count record.
 
 @example
 ```ts
 const langs = await probeLanguages({ owner, repo, cache, },);
 if (langs === LANGUAGES_UNKNOWN)
   return;
 ```
 */
export const LANGUAGES_UNKNOWN: unique symbol = Symbol('deps-cube repository languages cannot be fetched',);

/**
 Calls Linguist for a GitHub repo, returning language byte counts.
 
 @param owner - GitHub owner.
 
 @param repo - GitHub repo name.
 
 @param cache - File cache handle.
 
 @returns Linguist byte-count record, or {@link LANGUAGES_UNKNOWN} (logged via
 {@link caughtErrorMessage}) when API errors due to private repo, 404, or rate limit.
 
 @example
 ```ts
 const langs = await probeLanguages({ owner: 'preactjs', repo: 'preact', cache });
 ```
 */
export async function probeLanguages(
  {
    owner,
    repo,
    cache,
  }: {
    readonly owner: string;
    readonly repo: string;
    readonly cache: Cache;
  },
): Promise<Record<string, number> | typeof LANGUAGES_UNKNOWN> {
  /**
   Cache key for languages probe; `<owner>/<repo>` prevents repo collisions.
   */
  const key = `${owner}/${repo}`;
  // Language data is immutable per published version, so `ttlMs` is omitted (never expires).
  /**
   Cached Linguist response if previously fetched.
   */
  const cached = await cache.read<Record<string, number>>({
    name: key,
    version: '_repo',
    field: 'languages',
  },);
  if (cached !== CACHE_MISS)
    return cached;
  try {
    /**
     Fresh Linguist response; on error, catch returns {@link LANGUAGES_UNKNOWN} to signal unknown upstream.
     */
    const fetched = await ghApi<Record<string, number>>(
      `repos/${owner}/${repo}/languages`,
    );
    await cache.write({
      name: key,
      version: '_repo',
      field: 'languages',
      value: fetched,
    },);
    return fetched;
  }
  catch (error) {
    console.warn(
      `[deps-cube] GitHub languages probe failed for ${owner}/${repo}: ${caughtErrorMessage(error,)}`,
    );
    return LANGUAGES_UNKNOWN;
  }
}

/**
 Absence marker for {@link probeLastCommit} meaning "last-commit date is
 unavailable for this repo"; never an ISO timestamp string.
 
 @example
 ```ts
 const date = await probeLastCommit({ owner, repo, directory, cache, },);
 if (date === LAST_COMMIT_UNKNOWN)
   return;
 ```
 */
export const LAST_COMMIT_UNKNOWN: unique symbol = Symbol('deps-cube last commit date cannot be fetched',);

/**
 Fetches whole-repo `pushed_at`, or most recent commit date in `directory`.
 
 @param owner - GitHub owner.
 
 @param repo - GitHub repo name.
 
 @param directory - Subdirectory for monorepo-housed packages; whole repo if omitted.
 
 @param cache - File cache handle.
 
 @returns ISO timestamp string, or {@link LAST_COMMIT_UNKNOWN} (logged via
 {@link caughtErrorMessage}) when API errors out.
 
 @example
 ```ts
 const date = await probeLastCommit({ owner: 'lezer-parser', repo: 'common', directory: undefined, cache });
 ```
 */
export async function probeLastCommit(
  {
    owner,
    repo,
    directory,
    cache,
  }: {
    readonly owner: string;
    readonly repo: string;
    readonly directory?: string;
    readonly cache: Cache;
  },
): Promise<string | typeof LAST_COMMIT_UNKNOWN> {
  /**
   Cache key shared by whole-repo and path-scoped variants; `field` discriminates.
   */
  const key = `${owner}/${repo}`;
  /**
   Cache field tag distinguishing whole-repo `pushed_at` from per-directory commits.
   */
  const field = directory === undefined ? 'pushed_at' : `commits:${directory}`;
  /**
   Cached ISO date string from a prior probe of same `key`/`field`.
   */
  const cached = await cache.read<string>({
    name: key,
    version: '_repo',
    field,
    ttlMs: TTL_MS,
  },);
  if (cached !== CACHE_MISS)
    return cached;

  try {
    if (directory === undefined) {
      /**
       Whole-repo metadata; `pushed_at` is cheap proxy for any commit anywhere.
       */
      const repoMeta = await ghApi<{ pushed_at: string; }>(`repos/${owner}/${repo}`,);
      await cache.write({
        name: key,
        version: '_repo',
        field,
        value: repoMeta.pushed_at,
      },);
      return repoMeta.pushed_at;
    }
    /**
     Path-scoped commit list, most-recent first; only first author's date is consumed.
     */
    const commits = await ghApi<
      readonly { commit?: { author?: { date?: string; }; }; }[]
      >(
      `repos/${owner}/${repo}/commits?path=${encodeURIComponent(directory,)}&per_page=1`,
      );
    /**
     Author date of most-recent path-scoped commit; `undefined` means unexpected shape.
     */
    const date = commits[0]
      ?.commit
      ?.author
      ?.date;
    if (date === undefined)
      return LAST_COMMIT_UNKNOWN;
    await cache.write({
      name: key,
      version: '_repo',
      field,
      value: date,
    },);
    return date;
  }
  catch (error) {
    console.warn(
      `[deps-cube] GitHub last-commit probe failed for ${owner}/${repo}: ${caughtErrorMessage(error,)}`,
    );
    return LAST_COMMIT_UNKNOWN;
  }
}

//endregion Field probes
