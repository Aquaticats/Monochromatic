/**
 Build-time incremental cache for the SSG pipeline.
 
 Stores content hashes and rendered HTML from the unified pipeline to skip
 re-processing unchanged MDX files. Cache is persisted to `.cache/build-manifest.json`.
 
 Invalidation:
 - Content hash change: re-process that file only
 - Pipeline hash change: invalidate all entries (config/plugin change)
 - File deletion: stale entries cleaned after build
 */
import {
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { dirname, } from 'node:path';

import * as v from 'valibot';

import {
  type PostFrontmatter,
  postFrontmatterSchema,
} from './content.ts';
import type { Logger, } from './types.ts';

// File justification: 164 lines; schema definitions, I/O, and lookup form a
// cohesive cache API; splitting into 3+ sub-40-line files adds indirection
// without improving navigability.
export {
  computePipelineFingerprint,
  sha256,
} from './cache-hash.ts';

//region Schema and types

/**
 Per-file cache entry with content hash and pre-rendered HTML.
 
 The frontmatter record carries each date's git author offset, so a warm-cache
 build compares calendar days in the author's own zone exactly as a fresh
 build does, rather than falling back to the machine's zone.
 */
export type CacheEntry = {
  readonly contentHash: string;
  readonly html: string;
  readonly frontmatter: PostFrontmatter;
};

/**
 Valibot schema for a single cache entry.
 */
const cacheEntrySchema = v.object({
  contentHash: v.string(),
  html: v.string(),
  frontmatter: postFrontmatterSchema,
},);

/**
 On-disk cache structure at `.cache/build-manifest.json`.
 */
export type BuildManifest = {
  pipelineHash: string;
  /**
   HEAD commit SHA captured when this manifest was written.
   Used to validate cached git-derived publication/update dates:
   when the current HEAD matches, cached dates are reusable without
   re-probing git. When HEAD has moved, dates are re-derived.
   */
  headSha: string;
  content: Readonly<Record<string, CacheEntry>>;
};

/**
 Valibot schema for the on-disk build manifest.
 */
const buildManifestSchema = v.object({
  pipelineHash: v.string(),
  headSha: v.string(),
  content: v.record(
    v.string(),
    cacheEntrySchema,
  ),
},);

//endregion Schema and types

//region Sentinels

/**
 Sentinel returned by {@link readCache} when no manifest file exists on disk.
 A genuine `Symbol` rather than `null`/`undefined`, which the
 `no-nullish-union` rule rejects as non-sentinels.
 */
export const NO_CACHE: unique symbol = Symbol('build manifest file missing',);

/**
 Sentinel returned by {@link getCachedEntry} when the manifest has no matching,
 content-hash-current entry for a file. A genuine `Symbol` rather than
 `null`/`undefined`.
 */
export const CACHE_MISS: unique symbol = Symbol('build cache entry missing',);

//endregion Sentinels

/**
 Default path for the cache manifest file.
 */
const CACHE_PATH = '.cache/build-manifest.json';

//region Cache I/O

/**
 Reads the build manifest from disk.
 
 Returns {@link NO_CACHE} when the cache file does not exist.
 Logs and discards corrupted or invalid manifests rather than
 crashing the build, since a missing cache just triggers a full rebuild.
 
 @param l - logger for cache read errors
 
 @returns parsed and validated manifest, or {@link NO_CACHE} on any failure
 
 @example
 ```ts
 const manifest = await readCache({ l: logger });
 ```
 */
export async function readCache(
  { l, }: { readonly l: Logger; },
): Promise<BuildManifest | typeof NO_CACHE> {
  try {
    /**
     Raw JSON text read before schema validation so parse errors and validation errors share the same catch.
     */
    const raw = await readFile(
      CACHE_PATH,
      'utf8',
    );
    return v.parse(
      buildManifestSchema,
      JSON.parse(raw,),
    );
  }
  catch (error) {
    // ENOENT is expected on first build; everything else is worth logging
    /**
     Distinguishes the benign first-build case from genuine failures so logs stay quiet on the happy path.
     */
    const isFileNotFound = (Error.isError(error,))
      && ('code' in error)
      && (error.code
        === 'ENOENT');

    if (!isFileNotFound) {
      l.error(
        `Failed to read or validate build cache, starting fresh: ${String(error,)}`,
      );
    }
    return NO_CACHE;
  }
}

/**
 Writes the build manifest to disk, creating the `.cache/` directory if needed.
 
 @param manifest - build manifest to persist
 
 @mutates manifest - `JSON.stringify` may invoke `toJSON`, getters, or proxy traps.
 
 @example
 ```ts
 await writeCache(manifest);
 ```
 */
export async function writeCache(manifest: BuildManifest,): Promise<void> {
  await mkdir(
    dirname(CACHE_PATH,),
    { recursive: true, },
  );
  await writeFile(
    CACHE_PATH,
    JSON.stringify(
      manifest,
      undefined,
      2,
    ),
    'utf8',
  );
}

//endregion Cache I/O

//region Cache lookup

/**
 Looks up a cached entry for a given file path and content hash.
 
 @param manifest - current build manifest
 
 @param filePath - relative path to the MDX file
 
 @param contentHash - SHA-256 of the current file contents
 
 @returns cached entry if the content hash matches, otherwise {@link CACHE_MISS}
 
 @example
 ```ts
 const entry = getCachedEntry({ manifest, filePath: 'en/hello.mdx', contentHash: hash });
 ```
 */
export function getCachedEntry(
  {
    manifest,
    filePath,
    contentHash,
  }: {
    readonly manifest: Readonly<BuildManifest>;
    readonly filePath: string;
    readonly contentHash: string;
  },
): CacheEntry | typeof CACHE_MISS {
  /**
   Lookup separated from the hash check so the missing-key and stale-hash branches both early-return.
   */
  const entry = manifest.content[filePath];
  if (entry === undefined)
    return CACHE_MISS;

  if (entry.contentHash
    !== contentHash)
    return CACHE_MISS;

  return entry;
}

/**
 Creates a new cache entry for a processed MDX file.
 
 @param contentHash - SHA-256 of the raw file contents
 
 @param html - rendered HTML from the unified pipeline
 
 @param frontmatter - validated frontmatter data
 
 @returns cache entry ready for insertion into the manifest
 
 @example
 ```ts
 const entry = createCacheEntry({ contentHash: hash, html: '<p>Hello</p>', frontmatter: data });
 ```
 */
export function createCacheEntry(
  {
    contentHash,
    html,
    frontmatter,
  }: {
    readonly contentHash: string;
    readonly html: string;
    readonly frontmatter: CacheEntry['frontmatter'];
  },
): CacheEntry {
  return {
    contentHash,
    html,
    frontmatter,
  };
}

/**
 Builds a new manifest from processed entries, cleaning up stale paths.
 
 @param pipelineHash - current pipeline configuration hash
 
 @param headSha - HEAD commit SHA to persist for git-date reuse
 
 @param entries - record of file paths to cache entries
 
 @returns new build manifest
 
 @example
 ```ts
 const manifest = buildManifest({ pipelineHash, headSha, entries });
 ```
 */
export function buildManifest(
  {
    pipelineHash,
    headSha,
    entries,
  }: {
    readonly pipelineHash: string;
    readonly headSha: string;
    readonly entries: Readonly<Record<string, CacheEntry>>;
  },
): BuildManifest {
  return {
    pipelineHash,
    headSha,
    content: entries,
  };
}

//endregion Cache lookup
