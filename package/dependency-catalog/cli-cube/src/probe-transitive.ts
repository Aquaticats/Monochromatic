/**
 Depth-bounded transitive dep count via the npm registry.
 
 Walks `dependencies` of each package's latest manifest, counting distinct
 `name@version` pairs reachable within {@link TRANSITIVE_DEPTH_CAP}. Cycles
 are broken via the `visited` set. Per-(name, version) counts are cached
 with a 30-day TTL.
 
 Separated from `probe-fields.ts` to keep that file under the 300-line cap.
 
 @example
 ```ts
 const count = await probeTransitive({
   name: 'preact',
   version: '10.26.0',
   cache,
   visited: new Set(),
   depth: 0,
 });
 ```
 */

import { MS_PER_DAY, } from '@monochromatic-dev/const-core/ts';
import {
  type Cache,
  CACHE_MISS,
} from './cache.ts';
import { caughtValueText as caughtErrorMessage, } from '@monochromatic-dev/caught-value-core/ts';
import {
  type NpmPackage,
  probePackageManifest,
} from './probe-fields.ts';

//region Constants

/**
 Depth cap for transitive dep walks.
 */
const TRANSITIVE_DEPTH_CAP = 5;
/**
 Number of days the cached transitive count remains valid.
 */
const TTL_DAYS = 30;
/**
 TTL for cached transitive counts.
 */
const TTL_MS = TTL_DAYS * MS_PER_DAY;

//endregion Constants

//region Helpers

/**
 Absence marker for {@link readManifestSilent} meaning "registry fetch failed
 for this package"; never an {@link NpmPackage} manifest.
 */
const MANIFEST_FETCH_FAILED: unique symbol = Symbol('deps-cube manifest fetch failed during transitive walk',);

/**
 Wraps {@link probePackageManifest} so registry-fetch failures during the
 transitive walk return {@link MANIFEST_FETCH_FAILED} instead of throwing,
 logging the failure via {@link caughtErrorMessage}.
 
 @param npmName - npm package name.
 
 @param cache - File cache handle.
 
 @returns Manifest, or {@link MANIFEST_FETCH_FAILED} on any fetch error.
 */
async function readManifestSilent(
  {
    npmName,
    cache,
  }: {
    readonly npmName: string;
    readonly cache: Cache;
  },
): Promise<NpmPackage | typeof MANIFEST_FETCH_FAILED> {
  try {
    return await probePackageManifest({
      npmName,
      cache,
    },);
  }
  catch (error) {
    console.warn(
      `[deps-cube] transitive manifest probe failed for ${npmName}: ${caughtErrorMessage(error,)}`,
    );
    return MANIFEST_FETCH_FAILED;
  }
}

//endregion Helpers

//region Public API

/**
 Best-effort transitive dep count via a depth-bounded registry walk.
 
 @param name - npm package name.
 
 @param version - Concrete version.
 
 @param cache - File cache handle.
 
 @param visited - Set of already-visited `name@version` keys.
 
 @param depth - Current recursion depth (caller passes `0` initially).
 
 @returns Count of distinct packages reachable via `dependencies` within depth cap.
 
 @example
 ```ts
 const count = await probeTransitive({ name: 'preact', version: '10.26.0', cache, visited: new Set(), depth: 0 });
 ```
 */
export async function probeTransitive(
  {
    name,
    version,
    cache,
    visited,
    depth,
  }: {
    name: string;
    version: string;
    cache: Cache;
    visited: Set<string>;
    depth: number;
  },
): Promise<number> {
  /**
   `name@version` identity used both as the cycle-breaker key in `visited` and as the cache key.
   */
  const key = `${name}@${version}`;
  if (visited.has(key,))
    return 0;
  visited.add(key,);
  if (depth >= TRANSITIVE_DEPTH_CAP)
    return 0;

  /**
   Cached transitive count from a previous run, if still within {@link TTL_MS}.
   */
  const cached = await cache.read<number>({
    name,
    version,
    field: 'transitive',
    ttlMs: TTL_MS,
  },);
  if (cached !== CACHE_MISS)
    return cached;

  /**
   Full registry manifest for `name`; {@link MANIFEST_FETCH_FAILED} when the registry fetch failed.
   */
  const manifest = await readManifestSilent({
    npmName: name,
    cache,
  },);
  if (manifest === MANIFEST_FETCH_FAILED)
    return 0;
  /**
   Manifest entry for the exact requested version, falling back to any first version when the requested version is missing.
   */
  const versionManifest = manifest.versions?.[version]
    ?? Object
      .values(manifest.versions
        ?? {},)[0];
  /**
   Direct `dependencies` map from the version manifest; empty when none declared.
   */
  const deps = versionManifest?.dependencies
    ?? {};
  /**
   Direct dependency package names; each recurses into its own latest manifest.
   */
  const directNames = Object.keys(deps,);

  /**
   Per-direct-dep subtree counts (each direct dep contributes `1 + transitive_below`).
   */
  const subCounts = await Promise.all(directNames.map(
    async function recurseOne(depName,) {
      /**
       Latest-version manifest for the direct dep; {@link MANIFEST_FETCH_FAILED} when its registry fetch failed.
       */
      const depPkg = await readManifestSilent({
        npmName: depName,
        cache,
      },);
      if (depPkg === MANIFEST_FETCH_FAILED)
        return 0;
      /**
       Concrete version string for the direct dep, used as the cache key for the recursive call.
       */
      const depVersion = depPkg['dist-tags']
        ?.latest;
      if (depVersion === undefined)
        return 0;
      return 1 + await probeTransitive({
        name: depName,
        version: depVersion,
        cache,
        visited,
        depth: depth + 1,
      },);
    },
  ),);
  /**
   Sum across every direct dep's subtree; the final transitive count stored in the cache.
   */
  const total = subCounts.reduce(
    function add(
      a,
      b,
    ) {
      return a + b;
    },
    0,
  );
  await cache.write({
    name,
    version,
    field: 'transitive',
    value: total,
  },);
  return total;
}

//endregion Public API
