/**
 crates.io data for the release-age and trust checks.

 Publish times come from the sparse index (`pubtime`), which crates.io serves
 without a rate limit. Version histories for the trust policy are cached per
 crate under `.cache/rust-catalog-update/crates-io/` because published
 versions never change. A cold cache is filled from the crates.io database
 dump (see `./dump.ts`); crates the dump cannot rate, and versions newer than
 the dump, come from `https://crates.io/api/v1/crates/<name>/versions`, which
 returns every version in one response and allows at most one request per
 second with an identifying user agent.
 See `doc/planning/rust-dependency-catalog.md`.

 @module
 */

import {
  access,
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { fillHistoriesFromDump, } from './dump.ts';
import {
  type FetchGunzipped,
  type FetchText,
  RegistryError,
} from './http.ts';
import { l, } from './logger.ts';
import type { VersionRecord, } from './policy.ts';

//region Constants

/**
 Sparse index root.
 */
const SPARSE_INDEX_URL = 'https://index.crates.io';

/**
 crates.io API root.
 */
const API_URL = 'https://crates.io/api/v1';

/**
 Spacing between crates.io API requests. The data-access policy allows "a
 maximum of 1 request per second"; requests exactly 1000 ms apart got HTTP 429
 after 657 requests (2026-10-29), so the client stays a margin under the limit.
 */
export const API_MIN_INTERVAL_MS = 1_100;

/**
 Attempts per API request when crates.io answers HTTP 429.
 */
const RATE_LIMIT_ATTEMPTS = 5;

/**
 Wait before retrying a 429 that names no `Retry-After`; it doubles per attempt.
 */
const RATE_LIMIT_BACKOFF_MS = 30_000;

/**
 HTTP status of a rate-limited request.
 */
const TOO_MANY_REQUESTS = 429;

/**
 Milliseconds per second, for `Retry-After`.
 */
const MS_PER_SECOND = 1_000;

/**
 crates.io database dump: every public table, rebuilt daily, served without a rate limit.
 */
export const DATABASE_DUMP_URL = 'https://static.crates.io/db-dump.tar.gz';

/**
 Fewest uncached crates for which reading the dump beats the API. The API
 costs {@link API_MIN_INTERVAL_MS} per crate; the dump read took 22 s for all
 locked crates (2026-10-29) plus a 2.1 GB download, so below about 100 crates
 the API is as fast and moves far fewer bytes.
 */
export const DUMP_BREAKEVEN_CRATES = 100;

/**
 Repo-relative cache directory for version histories.
 */
export const HISTORY_CACHE_DIR = '.cache/rust-catalog-update/crates-io';

/**
 Crate-name lengths with a dedicated sparse-index directory.
 */
const SHORT_NAME_LENGTHS: ReadonlySet<number> = new Set([
  1,
  2,
],);

/**
 Crate-name length whose directory also takes the first character.
 */
const THREE_CHARACTER_NAME = 3;

/**
 Characters per sparse-index directory level for longer names.
 */
const PREFIX_SEGMENT = 2;

//endregion Constants

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
    return `${String(THREE_CHARACTER_NAME,)}/${lower.slice(
      0,
      1,
    )}/${lower}`;
  return `${lower.slice(
    0,
    PREFIX_SEGMENT,
  )}/${lower.slice(
    PREFIX_SEGMENT,
    PREFIX_SEGMENT * 2,
  )}/${lower}`;
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
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 A string field of parsed JSON, or empty when it is missing or not a string.

 @param object - parsed JSON object

 @param key - field name

 @returns field text

 @example
 ```ts
 stringField({ object: { num: '1.0.0', }, key: 'num', }); // '1.0.0'
 ```
 */
function stringField({
  object,
  key,
}: {
  readonly object: Readonly<Record<string, unknown>>;
  readonly key: string
},): string {
  /**
   Field value.
   */
  const value = object[key];
  return (typeof value) === 'string' ? value : '';
}

/**
 A numeric or string id of parsed JSON as text, or empty when absent.

 @param object - parsed JSON object

 @param key - field name

 @returns id text

 @example
 ```ts
 idField({ object: { id: 3618, }, key: 'id', }); // '3618'
 ```
 */
function idField({
  object,
  key,
}: {
  readonly object: Readonly<Record<string, unknown>>;
  readonly key: string
},): string {
  /**
   Field value.
   */
  const value = object[key];
  if ((typeof value) === 'number')
    return String(value,);
  return (typeof value) === 'string' ? value : '';
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
    if (isObject(record,) && ((typeof record.vers) === 'string')) {
      times.set(
        stringField({
          object: record,
          key: 'vers',
        },),
        stringField({
          object: record,
          key: 'pubtime',
        },),
      );
    }
  }
  return times;
}

/**
 Yanked versions from a sparse-index file.

 @param text - index file, one JSON object per line

 @returns versions whose record has `"yanked": true`

 @throws SyntaxError when a line is not JSON

 @example
 ```ts
 parseIndexYanked('{"vers":"1.0.1","yanked":true}\n').has('1.0.1'); // true
 ```
 */
export function parseIndexYanked(text: string,): ReadonlySet<string> {
  /**
   Yanked versions found.
   */
  const yanked = new Set<string>();
  for (const line of text.split('\n',)) {
    if (line.trim() === '')
      continue;
    /**
     One version record.
     */
    const record: unknown = JSON.parse(line,);
    if (isObject(record,) && ((typeof record.vers) === 'string')
      && (record.yanked === true))
      yanked.add(record.vers,);
  }
  return yanked;
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
  return versions.filter(function isEntry(entry: unknown,): entry is Readonly<Record<string, unknown>> {
    return isObject(entry,);
  },)
    .map(function toRecord(version: Readonly<Record<string, unknown>>,): VersionRecord {
      /**
       Publishing account, absent for Trusted Publishing.
       */
      const publisher = isObject(version.published_by,) ? version.published_by : {};
      /**
       Trusted Publishing provenance, absent for a token publish.
       */
      const trustpub = isObject(version.trustpub_data,) ? version.trustpub_data : {};
      return {
        num: stringField({
          object: version,
          key: 'num',
        },),
        createdAt: stringField({
          object: version,
          key: 'created_at',
        },),
        publisherId: idField({
          object: publisher,
          key: 'id',
        },),
        trustpubRepository: stringField({
          object: trustpub,
          key: 'repository',
        },),
        repository: stringField({
          object: version,
          key: 'repository',
        },),
      };
    },);
}

/**
 Account ids of a crate's listed individual owners from a crates.io owners response.

 Team owners (`kind: "team"`) are left out: the API does not list their members.

 @param text - `/api/v1/crates/<name>/owners` body

 @returns user ids as text

 @throws SyntaxError when the body is not JSON

 @example
 ```ts
 parseOwnerIds('{"users":[{"id":3618,"kind":"user"},{"id":9,"kind":"team"}]}'); // Set { '3618' }
 ```
 */
export function parseOwnerIds(text: string,): ReadonlySet<string> {
  /**
   Parsed body.
   */
  const body: unknown = JSON.parse(text,);
  /**
   Raw owner entries.
   */
  const users = isObject(body,) && Array.isArray(body.users,) ? body.users : [];
  return new Set(users.filter(function isUser(entry: unknown,): entry is Readonly<Record<string, unknown>> {
    return isObject(entry,) && (entry.kind === 'user');
  },)
    .map(function toId(entry: Readonly<Record<string, unknown>>,): string {
      return idField({
        object: entry,
        key: 'id',
      },);
    },)
    .filter(function isPresent(id: string,): boolean {
      return id !== '';
    },),);
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
   Yanked versions of a crate, read fresh from the sparse index because yanking changes.
   */
  readonly yanked: (name: string,) => Promise<ReadonlySet<string>>;
  /**
   Version history of a crate, refetched when a needed version is missing from the cache.
   */
  readonly history: (request: {
    readonly name: string;
    readonly versions: readonly string[]
  },) => Promise<readonly VersionRecord[]>;
  /**
   Account ids of a crate's listed individual owners, fetched fresh because ownership changes.
   */
  readonly owners: (name: string,) => Promise<ReadonlySet<string>>;
  /**
   Fills the history cache from the database dump when many of these crates have no cache file yet.
   */
  readonly prefetch: (names: ReadonlySet<string>,) => Promise<void>;
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
  const log = tagged({
    tag: readCachedHistory.name,
    l,
  },);
  try {
    /**
     Cache file body.
     */
    const parsed: unknown = JSON.parse(await readFile(
      path,
      'utf8',
    ),);
    if (!Array.isArray(parsed,))
      return [];
    return parsed.filter(function isEntry(entry: unknown,): entry is Readonly<Record<string, unknown>> {
      return isObject(entry,);
    },)
      .map(function toRecord(entry: Readonly<Record<string, unknown>>,): VersionRecord {
        return {
          num: stringField({
            object: entry,
            key: 'num',
          },),
          createdAt: stringField({
            object: entry,
            key: 'createdAt',
          },),
          publisherId: stringField({
            object: entry,
            key: 'publisherId',
          },),
          trustpubRepository: stringField({
            object: entry,
            key: 'trustpubRepository',
          },),
          repository: stringField({
            object: entry,
            key: 'repository',
          },),
        };
      },);
  } catch (error) {
    log.debug(`no usable cache at ${path} (${String(error,)}); fetching from crates.io`,);
    return [];
  }
}

/**
 Builds a {@link Registry}.

 @param root - repository root; the history cache lives below it

 @param fetchText - body fetcher

 @param fetchDump - decompressed stream of the database dump

 @param minIntervalMs - spacing between crates.io API requests

 @param rateLimitBackoffMs - first wait after a 429 without `Retry-After`

 @param dumpBreakevenCrates - fewest uncached crates that make the dump worth reading

 @returns registry lookups

 @example
 ```ts
 const registry = createRegistry({ root: process.cwd(), fetchText: fetchTextWithUserAgent, fetchDump: fetchGunzippedWithUserAgent, });
 ```
 */
export function createRegistry(
  {
    root,
    fetchText,
    fetchDump,
    minIntervalMs = API_MIN_INTERVAL_MS,
    rateLimitBackoffMs = RATE_LIMIT_BACKOFF_MS,
    dumpBreakevenCrates = DUMP_BREAKEVEN_CRATES,
  }: {
    readonly root: string;
    readonly fetchText: FetchText;
    readonly fetchDump: FetchGunzipped;
    readonly minIntervalMs?: number;
    readonly rateLimitBackoffMs?: number;
    readonly dumpBreakevenCrates?: number;
  },
): Registry {
  /**
   Logger for requests.
   */
  const log = tagged({
    tag: createRegistry.name,
    l,
  },);
  /**
   Time of the last API request, for spacing; callers request histories one at a time.
   */
  const throttle = { lastRequestAt: Number.NEGATIVE_INFINITY, };
  /**
   History cache directory.
   */
  const cacheDir = join(
    root,
    HISTORY_CACHE_DIR,
  );
  /**
   Fetches a crates.io API URL no sooner than `minIntervalMs` after the previous one,
   retrying an HTTP 429 after its `Retry-After` (or a doubling backoff).

   @param url - API URL

   @returns response body

   @throws RegistryError when the request fails, or still answers 429 after every attempt

   @example
   ```ts
   await spacedApiGet('https://crates.io/api/v1/crates/anyhow/owners');
   ```
   */
  async function spacedApiGet(url: string,): Promise<string> {
    for (let attempt = 1; ; attempt += 1) {
      /**
       Time left before the next request is allowed.
       */
      const delay = (throttle.lastRequestAt + minIntervalMs) - Date.now();
      if (delay > 0)
        // oxlint-disable-next-line no-await-in-loop -- requests are spaced one at a time by design.
        await wait(delay,);
      throttle.lastRequestAt = Date.now();
      log.info(`fetching ${url}`,);
      try {
        // oxlint-disable-next-line no-await-in-loop -- one request at a time keeps under the rate limit.
        return await fetchText(url,);
      } catch (error) {
        if ((!(error instanceof RegistryError)) || (error.status !== TOO_MANY_REQUESTS)
          || (attempt >= RATE_LIMIT_ATTEMPTS))
          throw error;
        /**
         Wait the server asked for, else the doubling backoff.
         */
        const pause = error.retryAfterSeconds > 0 ? error.retryAfterSeconds * MS_PER_SECOND : rateLimitBackoffMs * (2 ** (attempt - 1));
        log.warn(`${url} answered HTTP 429 (attempt ${String(attempt,)} of ${String(RATE_LIMIT_ATTEMPTS,)}); retrying in ${String(pause,)} ms`,);
        // oxlint-disable-next-line no-await-in-loop -- the retry must wait for the server's window.
        await wait(pause,);
      }
    }
  }

  /**
   Whether a crate already has a cache file.

   @param name - crate name

   @returns whether its history file exists

   @example
   ```ts
   await isCached('anyhow');
   ```
   */
  async function isCached(name: string,): Promise<boolean> {
    try {
      await access(join(
        cacheDir,
        `${name}.json`,
      ),);
      return true;
    } catch (error) {
      if (Error.isError(error,) && ('code' in error)
        && (error.code === 'ENOENT'))
        return false;
      throw error;
    }
  }
  return {
    prefetch: async function prefetch(names: ReadonlySet<string>,): Promise<void> {
      /**
       Crates without a cache file.
       */
      const uncached = new Set((await Promise.all([...names,].map(async function check(name: string,): Promise<readonly string[]> {
        return (await isCached(name,)) ? [] : [name,];
      },),)).flat(),);
      if (uncached.size < dumpBreakevenCrates) {
        log.info(`${String(uncached.size,)} crates lack a cached history (below ${String(dumpBreakevenCrates,)}); using the crates.io API for them`,);
        return;
      }
      log.info(`${String(uncached.size,)} crates lack a cached history; reading ${DATABASE_DUMP_URL}`,);
      /**
       Fill outcome.
       */
      const result = await fillHistoriesFromDump({
        tarChunks: await fetchDump(DATABASE_DUMP_URL,),
        names: uncached,
        cacheDir,
      },);
      log.info(`dump: ${String(result.found,)} found, ${String(result.written,)} written, ${String(result.kept,)} already cached, ${String(result.leftToApi
        .length,)} left to the API (possible Trusted Publishing releases)`,);
    },
    owners: async function owners(name: string,): Promise<ReadonlySet<string>> {
      return parseOwnerIds(await spacedApiGet(`${API_URL}/crates/${encodeURIComponent(name,)}/owners`,),);
    },
    pubtimes: async function pubtimes(name: string,): Promise<ReadonlyMap<string, string>> {
      /**
       Index file URL.
       */
      const url = `${SPARSE_INDEX_URL}/${sparseIndexPath(name,)}`;
      log.debug(`fetching ${url}`,);
      return parseIndexPubtimes(await fetchText(url,),);
    },
    yanked: async function yanked(name: string,): Promise<ReadonlySet<string>> {
      /**
       Index file URL.
       */
      const url = `${SPARSE_INDEX_URL}/${sparseIndexPath(name,)}`;
      log.debug(`fetching ${url} for yanked versions`,);
      return parseIndexYanked(await fetchText(url,),);
    },
    history: async function history({
      name,
      versions,
    },): Promise<readonly VersionRecord[]> {
      /**
       Cache file for this crate.
       */
      const path = join(
        cacheDir,
        `${name}.json`,
      );
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
       Fresh records.
       */
      const fresh = parseVersionHistory(await spacedApiGet(`${API_URL}/crates/${encodeURIComponent(name,)}/versions`,),);
      await mkdir(
        cacheDir,
        { recursive: true, },
      );
      await writeFile(
        path,
        `${JSON.stringify(fresh,)}\n`,
      );
      return fresh;
    },
  };
}

//endregion Client
