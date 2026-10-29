/**
 Per-package JSON file cache.
 
 One file per (npm name, version) pair under
 `~/.cache/monochromatic/deps-cube/<name>/<version>.json`. Each file holds a
 single object whose top-level keys are field names; values carry their own
 `fetchedAt` timestamps so different fields within the same file can have
 different TTLs (language data is immutable per version, last-commit data
 only fresh for 30 days).
 
 Writes are atomic: render to a sibling `.tmp` file, then rename; readers
 never see torn JSON even if the process is killed mid-write.
 
 @example
 ```ts
 import { createCache } from './cache.ts';
 const cache = createCache();
 const languages = await cache.read({
   name: 'preact',
   version: '10.26.0',
   field: 'languages',
   ttlMs: null,
 });
 if (languages === CACHE_MISS) {
   const fetched = await fetchLanguages();
   await cache.write({ name: 'preact', version: '10.26.0', field: 'languages', value: fetched });
 }
 ```
 */

import {
  mkdir,
  readFile,
  rename,
  writeFile,
} from 'node:fs/promises';
import { homedir, } from 'node:os';
import {
  dirname,
  join,
} from 'node:path';

import { caughtValueText as caughtErrorMessage, } from '@monochromatic-dev/caught-value-core/ts';

//region Types

/**
 Sentinel returned by {@link Cache.read} when a field is missing, expired, or
 unreadable. A distinct named value (not `undefined`) so a genuine cache miss
 is never confused with a stored value and never widens a slot to `T | undefined`.
 */
export const CACHE_MISS: unique symbol = Symbol('deps-cube cache field is missing or unreadable',);

/**
 One stored field's payload.
 */
type CacheValue<T,> = {
  /**
   The cached data.
   */
  value: T;
  /**
   Epoch milliseconds at which the value was written.
   */
  fetchedAt: number;
};

/**
 Top-level structure of a single cache file.
 */
type CacheFile = Record<string, CacheValue<unknown>>;

/**
 Address of a single field within the cache.
 */
type CacheKey = {
  /**
   Npm package name (may be scoped, e.g. `@anthropic-ai/sdk`).
   */
  readonly name: string;
  /**
   Concrete version string (e.g. `0.92.0`).
   */
  readonly version: string;
  /**
   Field name within the cache file (e.g. `languages`, `commits`).
   */
  readonly field: string;
};

/**
 Cache handle returned by {@link createCache}.
 */
export type Cache = {
  /**
   Reads a field's value if it exists and is not expired.
   
   @param key - Field address plus the field's TTL (omit `ttlMs` = never expire).
   
   @returns The cached value, or {@link CACHE_MISS} if missing/expired/unreadable.
   */
  readonly read: <T,>(
    {
      name,
      version,
      field,
      ttlMs,
    }: CacheKey & {
      readonly ttlMs?: number;
    },
  ) => Promise<T | typeof CACHE_MISS>;
  /**
   Writes a field's value, creating the cache file atomically.
   
   @param key - Field address.
   
   @param value - JSON-serialisable value to store.
   
   @returns Resolves once the file has been renamed into place.
   */
  readonly write: (
    {
      name,
      version,
      field,
      value,
    }: CacheKey & {
      readonly value: unknown;
    },
  ) => Promise<void>;
  /**
   Absolute path to the cache root, exposed for diagnostics and tests.
   */
  readonly rootDir: string;
};

//endregion Types

//region Implementation

/**
 Default cache root: `~/.cache/monochromatic/deps-cube/`. The XDG-compatible
 `$XDG_CACHE_HOME` is honoured when set.
 
 @returns Absolute path to the default cache root directory.
 */
function defaultRootDir(): string {
  /**
   XDG override; empty or unset falls through to the `$HOME/.cache` default.
   */
  const xdg = process.env
    .XDG_CACHE_HOME;
  /**
   Resolved cache home; honours XDG when present, falls back to the conventional location.
   */
  const cacheHome = ((xdg !== undefined) && (xdg !== '')) ? xdg : join(
    homedir(),
    '.cache',
  );
  return join(
    cacheHome,
    'monochromatic',
    'deps-cube',
  );
}

/**
 Resolves the absolute file path for one (name, version) cache entry.
 
 @param name - npm package name (may be scoped).
 
 @param version - Concrete version string.
 
 @param rootDir - Cache root directory.
 
 @returns Absolute path to the JSON file (which may not yet exist).
 */
function filePath(
  {
    name,
    version,
    rootDir,
  }: {
    readonly name: string;
    readonly version: string;
    readonly rootDir: string;
  },
): string {
  return join(
    rootDir,
    name,
    `${version}.json`,
  );
}

/**
 Loads and parses a cache file, returning an empty record on any I/O or
 parse error (reported via {@link caughtErrorMessage}) so callers can treat
 missing-and-corrupt as equivalent cache-miss conditions.
 
 @param path - Absolute path to the JSON file.
 
 @returns Parsed cache-file contents, or `{}` if the file is missing or unparseable.
 */
async function readFileOrEmpty(path: string,): Promise<CacheFile> {
  try {
    /**
     UTF-8 file contents fed to `JSON.parse`; any failure here is caught below as a miss.
     */
    const raw = await readFile(
      path,
      'utf8',
    );
    // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- JSON.parse returns `any`; cache files are written only by us.
    return JSON.parse(raw,) as CacheFile;
  }
  catch (error) {
    console.warn(
      `[deps-cube] cache file read failed for ${path}: ${caughtErrorMessage(error,)}`,
    );
    return {};
  }
}

/**
 Creates a cache handle bound to `rootDir`.
 
 @param rootDir - Optional cache root override. Defaults to {@link defaultRootDir}.
 
 @returns A {@link Cache} handle providing `read` and `write` methods.
 
 @example
 ```ts
 const cache = createCache();
 // or with a custom root for tests:
 const cache = createCache({ rootDir: '/tmp/cache-test-1234' });
 ```
 */
export function createCache(
  { rootDir, }: { readonly rootDir?: string; } = {},
): Cache {
  /**
   Cache root used by every closure on this handle; pinned at construction so later env mutations are ignored.
   */
  const resolvedRoot = rootDir ?? defaultRootDir();

  /**
   Reads a single field, applying TTL.
   
   @returns Parsed value, or {@link CACHE_MISS} if missing or stale.
   */
  async function read<T,>(
    {
      name,
      version,
      field,
      ttlMs,
    }: CacheKey & {
      readonly ttlMs?: number;
    },
  ): Promise<T | typeof CACHE_MISS> {
    /**
     Absolute on-disk path for this (name, version) pair.
     */
    const path = filePath({
      name,
      version,
      rootDir: resolvedRoot,
    },);
    /**
     Parsed cache file; treats a missing or corrupt file as an empty record.
     */
    const file = await readFileOrEmpty(path,);
    /**
     Stored entry for the requested field, or `undefined` for a miss.
     */
    const entry = file[field];
    if (entry === undefined)
      return CACHE_MISS;
    if ((ttlMs !== undefined) && ((Date.now()
      - entry
        .fetchedAt) > ttlMs))
      return CACHE_MISS;
    // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- caller asserts T; cache file shape is opaque at this layer.
    return entry.value as T;
  }

  /**
   Writes a single field, updating any existing fields in the same file.
   Uses atomic tmp+rename so torn reads are impossible.
   */
  async function write(
    {
      name,
      version,
      field,
      value,
    }: CacheKey & {
      readonly value: unknown;
    },
  ): Promise<void> {
    /**
     Destination cache file path.
     */
    const path = filePath({
      name,
      version,
      rootDir: resolvedRoot,
    },);
    /**
     Current file contents preserved across the write so sibling fields aren't clobbered.
     */
    const existing = await readFileOrEmpty(path,);
    /**
     Merged file body to be serialised; new value plus a fresh `fetchedAt` overwrites the matching field.
     */
    const next: CacheFile = {
      ...existing,
      [field]: {
        value,
        fetchedAt: Date.now(),
      },
    };
    await mkdir(
      dirname(path,),
      { recursive: true, },
    );
    /**
     Sibling temp path used for the atomic write; pid and timestamp avoid collisions between concurrent writes.
     */
    const tmpPath = `${path}.${process.pid}.${Date.now()}.tmp`;
    await writeFile(
      tmpPath,
      JSON.stringify(
        next,
        null,
        2,
      ),
      'utf8',
    );
    await rename(
      tmpPath,
      path,
    );
  }

  return {
    read,
    write,
    rootDir: resolvedRoot,
  };
}

//endregion Implementation
