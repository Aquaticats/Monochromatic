/**
 crates.io data for the release-age and trust checks.

 Publish times come from the sparse index (`pubtime`), which crates.io serves
 without a rate limit. Version histories for the trust policy come from
 `https://crates.io/api/v1/crates/<name>/versions`, which returns every
 version in one response and allows at most one request per second with an
 identifying user agent; they are cached per crate under
 `.cache/rust-catalog-update/crates-io/` because published versions never change.
 See `doc/planning/rust-dependency-catalog.md`.

 @module
 */

import {
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { wait, } from '@monochromatic-dev/module-async-time/ts';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { l, } from './logger.ts';
import type { VersionRecord, } from './policy.ts';

//region Constants

/**
 User agent crates.io asks clients to send; names the repository, not a person.
 */
export const USER_AGENT = 'Monochromatic rust-catalog-update (https://github.com/Aquaticat/Monochromatic)';

/**
 Sparse index root.
 */
const SPARSE_INDEX_URL = 'https://index.crates.io';

/**
 crates.io API root.
 */
const API_URL = 'https://crates.io/api/v1';

/**
 crates.io's documented minimum spacing between API requests.
 */
export const API_MIN_INTERVAL_MS = 1000;

/**
 Repo-relative cache directory for version histories.
 */
export const HISTORY_CACHE_DIR = '.cache/rust-catalog-update/crates-io';

/**
 Crate-name lengths with a dedicated sparse-index directory.
 */
const SHORT_NAME_LENGTHS: ReadonlySet<number> = new Set([1, 2,],);

/**
 Crate-name length whose directory also takes the first character.
 */
const THREE_CHARACTER_NAME = 3;

/**
 Characters per sparse-index directory level for longer names.
 */
const PREFIX_SEGMENT = 2;

//endregion Constants

//region Errors

/**
 Thrown when crates.io or the sparse index cannot answer.

 @example
 ```ts
 throw new RegistryError({ url: 'https://index.crates.io/an/yh/anyhow', status: 503, });
 ```
 */
export class RegistryError extends Error {
  /**
   Requested URL.
   */
  readonly url: string;

  /**
   HTTP status, or 0 when the request never got a response.
   */
  readonly status: number;

  /**
   Builds the error.

   @param url - requested URL

   @param status - HTTP status, 0 without a response

   @param cause - underlying failure
   */
  constructor({ url, status, cause, }: { readonly url: string; readonly status: number; readonly cause?: unknown; },) {
    super(
      status === 0
        ? `Could not reach ${url}; check the network, then rerun.`
        : `${url} answered HTTP ${String(status,)}; rerun later, or check https://status.crates.io.`,
      { cause, },
    );
    this.name = 'RegistryError';
    this.url = url;
    this.status = status;
  }
}

//endregion Errors

//region Fetch

/**
 Fetches a URL's body as text.

 @example
 ```ts
 const fetchText: FetchText = async (url) => 'body';
 ```
 */
export type FetchText = (url: string,) => Promise<string>;

/**
 Sends one GET with {@link USER_AGENT}.

 @param url - URL to fetch

 @returns response of any status

 @throws RegistryError with status 0 when no response arrives

 @example
 ```ts
 await fetchOrThrow('https://index.crates.io/it/oa/itoa');
 ```
 */
async function fetchOrThrow(url: string,): Promise<Response> {
  try {
    return await fetch(url, { headers: { 'user-agent': USER_AGENT, }, },);
  } catch (error) {
    throw new RegistryError({ url, status: 0, cause: error, },);
  }
}

/**
 Default {@link FetchText} over the global `fetch`, sending {@link USER_AGENT}.

 @param url - URL to fetch

 @returns response body

 @throws RegistryError when the request fails or the status is not 2xx

 @example
 ```ts
 await fetchTextWithUserAgent('https://index.crates.io/it/oa/itoa');
 ```
 */
export async function fetchTextWithUserAgent(url: string,): Promise<string> {
  /**
   Response; a request that gets none becomes a RegistryError.
   */
  const response = await fetchOrThrow(url,);
  if (!response.ok)
    throw new RegistryError({ url, status: response.status, },);
  return await response.text();
}

//endregion Fetch

//region Parse

/**
 Sparse-index path of a crate, per the Cargo registry index layout.

 @param name - crate name

 @returns path below the index root

 @example
 ```ts
 sparseIndexPath('anyhow'); // 'an/yh/anyhow'
 sparseIndexPath('syn'); // '3/s/syn'
 ```
 */
export function sparseIndexPath(name: string,): string {
  /**
   Lowercased name; index paths are case-insensitive.
   */
  const lower = name.toLowerCase();
  if (SHORT_NAME_LENGTHS.has(lower.length,))
    return `${String(lower.length,)}/${lower}`;
  if (lower.length === THREE_CHARACTER_NAME)
    return `${String(THREE_CHARACTER_NAME,)}/${lower.slice(0, 1,)}/${lower}`;
  return `${lower.slice(0, PREFIX_SEGMENT,)}/${lower.slice(PREFIX_SEGMENT, PREFIX_SEGMENT * 2,)}/${lower}`;
}

/**
 Whether `value` is a plain object.

 @param value - parsed JSON

 @returns whether it is a non-array object

 @example
 ```ts
 isObject({}); // true
 ```
 */
function isObject(value: unknown,): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object') && (value !== null) && (!Array.isArray(value,));
}

/**
 Publish times per version from a sparse-index file.

 @param text - index file, one JSON object per line

 @returns version to `pubtime`; empty string for versions the index records no time for

 @throws SyntaxError when a line is not JSON

 @example
 ```ts
 parseIndexPubtimes('{"vers":"1.0.18","pubtime":"2026-03-20T00:12:37Z"}\n').get('1.0.18'); // '2026-03-20T00:12:37Z'
 ```
 */
export function parseIndexPubtimes(text: string,): ReadonlyMap<string, string> {
  /**
   Times found.
   */
  const times = new Map<string, string>();
  for (const line of text.split('\n',)) {
    if (line.trim() === '')
      continue;
    /**
     One version record.
     */
    const record: unknown = JSON.parse(line,);
    if (isObject(record,) && ((typeof record.vers) === 'string'))
      times.set(String(record.vers,), (typeof record.pubtime) === 'string' ? String(record.pubtime,) : '',);
  }
  return times;
}

/**
 Trust-relevant fields of a crates.io versions response.

 @param text - `/api/v1/crates/<name>/versions` body

 @returns one record per version, in response order

 @throws SyntaxError when the body is not JSON

 @example
 ```ts
 parseVersionHistory('{"versions":[{"num":"1.0.0","created_at":"2020-01-01T00:00:00Z","published_by":{"id":1},"trustpub_data":null,"repository":"https://x"}]}');
 ```
 */
export function parseVersionHistory(text: string,): readonly VersionRecord[] {
  /**
   Parsed body.
   */
  const body: unknown = JSON.parse(text,);
  /**
   Raw version entries.
   */
  const versions = isObject(body,) && Array.isArray(body.versions,) ? body.versions : [];
  return versions.filter(isObject,).map(function toRecord(version: Readonly<Record<string, unknown>>,): VersionRecord {
    /**
     Publishing account, absent for Trusted Publishing.
     */
    const publisher = isObject(version.published_by,) ? version.published_by : {};
    /**
     Trusted Publishing provenance, absent for a token publish.
     */
    const trustpub = isObject(version.trustpub_data,) ? version.trustpub_data : {};
    return {
      num: (typeof version.num) === 'string' ? String(version.num,) : '',
      createdAt: (typeof version.created_at) === 'string' ? String(version.created_at,) : '',
      publisherId: (publisher.id === undefined) || (publisher.id === null) ? '' : String(publisher.id,),
      trustpubRepository: (typeof trustpub.repository) === 'string' ? String(trustpub.repository,) : '',
      repository: (typeof version.repository) === 'string' ? String(version.repository,) : '',
    };
  },);
}

//endregion Parse

//region Client

/**
 Registry lookups with caching and the API rate limit.

 @example
 ```ts
 const registry: Registry = createRegistry({ root, fetchText: fetchTextWithUserAgent, });
 ```
 */
export type Registry = {
  /**
   Publish times of every version of a crate.
   */
  readonly pubtimes: (name: string,) => Promise<ReadonlyMap<string, string>>;
  /**
   Version history of a crate, refetched when a needed version is missing from the cache.
   */
  readonly history: (request: { readonly name: string; readonly versions: readonly string[]; },) => Promise<readonly VersionRecord[]>;
};

/**
 Reads a cached history, or nothing when there is no usable cache file.

 @param path - cache file

 @returns cached records, or an empty list

 @example
 ```ts
 await readCachedHistory('.cache/rust-catalog-update/crates-io/anyhow.json');
 ```
 */
async function readCachedHistory(path: string,): Promise<readonly VersionRecord[]> {
  /**
   Logger for cache misses.
   */
  const log = tagged({ tag: readCachedHistory.name, l, },);
  try {
    /**
     Cache file body.
     */
    const parsed: unknown = JSON.parse(await readFile(path, 'utf8',),);
    return Array.isArray(parsed,) ? parsed.filter(isObject,).map(function toRecord(entry,): VersionRecord {
      return {
        num: String(entry.num ?? '',),
        createdAt: String(entry.createdAt ?? '',),
        publisherId: String(entry.publisherId ?? '',),
        trustpubRepository: String(entry.trustpubRepository ?? '',),
        repository: String(entry.repository ?? '',),
      };
    },) : [];
  } catch (error) {
    log.debug(`no usable cache at ${path} (${String(error,)}); fetching from crates.io`,);
    return [];
  }
}

/**
 Builds a {@link Registry}.

 @param root - repository root; the history cache lives below it

 @param fetchText - body fetcher

 @param minIntervalMs - spacing between crates.io API requests

 @returns registry lookups

 @example
 ```ts
 const registry = createRegistry({ root: process.cwd(), fetchText: fetchTextWithUserAgent, });
 ```
 */
export function createRegistry(
  { root, fetchText, minIntervalMs = API_MIN_INTERVAL_MS, }: { readonly root: string; readonly fetchText: FetchText; readonly minIntervalMs?: number; },
): Registry {
  /**
   Logger for requests.
   */
  const log = tagged({ tag: createRegistry.name, l, },);
  /**
   Time of the last API request, for spacing; callers request histories one at a time.
   */
  const throttle = { lastRequestAt: Number.NEGATIVE_INFINITY, };
  /**
   History cache directory.
   */
  const cacheDir = join(root, HISTORY_CACHE_DIR,);
  return {
    pubtimes: async function pubtimes(name: string,): Promise<ReadonlyMap<string, string>> {
      /**
       Index file URL.
       */
      const url = `${SPARSE_INDEX_URL}/${sparseIndexPath(name,)}`;
      log.debug(`fetching ${url}`,);
      return parseIndexPubtimes(await fetchText(url,),);
    },
    history: async function history({ name, versions, },): Promise<readonly VersionRecord[]> {
      /**
       Cache file for this crate.
       */
      const path = join(cacheDir, `${name}.json`,);
      /**
       Cached records.
       */
      const cached = await readCachedHistory(path,);
      /**
       Versions the cache already holds.
       */
      const known = new Set(cached.map(function toNum(record: VersionRecord,): string {
        return record.num;
      },),);
      if ((cached.length > 0) && versions.every(function isKnown(version: string,): boolean {
        return known.has(version,);
      },))
        return cached;
      /**
       Time left before the next request is allowed.
       */
      const delay = throttle.lastRequestAt + minIntervalMs - Date.now();
      if (delay > 0)
        await wait(delay,);
      throttle.lastRequestAt = Date.now();
      /**
       API URL for this crate.
       */
      const url = `${API_URL}/crates/${encodeURIComponent(name,)}/versions`;
      log.info(`fetching ${url}`,);
      /**
       Fresh records.
       */
      const fresh = parseVersionHistory(await fetchText(url,),);
      await mkdir(cacheDir, { recursive: true, },);
      await writeFile(path, `${JSON.stringify(fresh,)}\n`,);
      return fresh;
    },
  };
}

//endregion Client
