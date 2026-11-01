/**
 pnpm virtual-store probe for catalog-tighten's MISS classification.
 
 When {@link readInstalledVersion} finds no direct-dependency symlink for a
 catalog entry, the entry is not necessarily absent: pnpm's isolated linker
 only creates a top-level `node_modules/<name>` symlink for a **direct**
 dependency, so a package installed purely as a transitive dependency lives
 only in the flattened store at
 `<modulesDir>/.pnpm/<mangled>@<version>/node_modules/<name>/package.json`.
 This probe reads that store so the caller can distinguish "present but
 undeclared" (a dependency-hygiene signal: a live package uses it without
 declaring it, or the catalog entry is dead) from "absent" (no live consumer
 at all). It classifies only; it never feeds the tightening path, because a
 transitive copy's version is whatever some dependent happened to pull, not
 the version the catalog entry itself resolves to.
 */

import {
  readdir,
} from 'node:fs/promises';
import {
  join,
} from 'node:path';

import {
  NO_MANIFEST_VERSION,
  readVersionFromPackageJson,
} from './version-read.ts';

//region Store probe

/**
 Sentinel returned by {@link readStoreVersions} when the pnpm virtual store
 holds no copy of the requested name. A `unique symbol`; callers narrow with
 `=== NOT_IN_STORE`.
 */
export const NOT_IN_STORE: unique symbol = Symbol('catalog-tighten/not-in-store',);

/**
 Encodes an npm name into the prefix pnpm uses for its flattened store
 directories, replacing the single scope separator `/` with `+`. An npm name
 carries at most one `/` (the scope boundary), so a literal global replace is
 exact.
 
 @param npmName - npm package name, e.g. `\@types/mdx` or `openai`
 
 @returns store-directory prefix, e.g. `\@types+mdx` or `openai`
 
 @example
 ```ts
 toStorePrefix("\@types/mdx") // "\@types+mdx"
 toStorePrefix("openai") // "openai"
 ```
 */
function toStorePrefix(npmName: string,): string {
  return npmName.replaceAll(
    '/',
    '+',
  );
}

/**
 Reads every distinct version of `npmName` present in the pnpm virtual store.
 
 Enumerates `<monorepoRoot>/<modulesDir>/.pnpm`, keeps directories whose name
 begins with `<mangled>@` (the trailing `@` pins the package boundary, so
 `micromark@` never matches `micromark-core@…`), and reads the `version` from
 each candidate's inner `node_modules/<name>/package.json`. A spuriously
 matched directory yields {@link NO_MANIFEST_VERSION} on the inner read and is
 dropped, so the boundary check and the manifest read are two independent
 guards. Returns the distinct versions sorted, or {@link NOT_IN_STORE} when
 the store holds no copy (including when there is no `.pnpm` directory, as
 under a PnP or hoisted layout).
 
 @param npmName - npm package name to look up
 
 @param monorepoRoot - absolute path to the monorepo root
 
 @param modulesDir - per-importer modules directory name (usually `node_modules`); `.pnpm` sits inside it
 
 @returns sorted distinct store versions, or {@link NOT_IN_STORE}
 
 @example
 ```ts
 await readStoreVersions({ npmName: "\@types/mdx", monorepoRoot: "/repo", modulesDir: "node_modules" }) // ["2.0.14"]
 await readStoreVersions({ npmName: "preact", monorepoRoot: "/repo", modulesDir: "node_modules" }) // NOT_IN_STORE
 ```
 */
export async function readStoreVersions(
  {
    npmName,
    monorepoRoot,
    modulesDir,
  }: {
    readonly npmName: string;
    readonly monorepoRoot: string;
    readonly modulesDir: string;
  },
): Promise<readonly string[] | typeof NOT_IN_STORE> {
  /**
   Flattened pnpm store directory; each child is `<mangled>@<version>[(peers)]`.
   */
  const storeDir = join(
    monorepoRoot,
    modulesDir,
    '.pnpm',
  );
  /**
   Direct children of `.pnpm`, or the not-in-store sentinel when the directory is absent (PnP/hoisted layout).
   */
  const entries = await readStoreEntries(storeDir,);
  if (entries === NOT_IN_STORE)
    return NOT_IN_STORE;

  /**
   Store-directory prefix that a copy of `npmName` must start with, boundary-terminated by `@`.
   */
  const prefix = `${toStorePrefix(npmName,)}@`;
  /**
   Store directories whose name marks them as a copy of exactly `npmName`.
   */
  const matching = entries.filter(function startsWithPrefix(entry,): boolean {
    return entry.startsWith(prefix,);
  },);
  /**
   Version read from each candidate's inner manifest; `NO_MANIFEST_VERSION` when the inner read fails.
   */
  const versions = await Promise.all(matching.map(async function readStoreVersion(
    entry,
  ): Promise<string | typeof NO_MANIFEST_VERSION> {
    return await readVersionFromPackageJson(join(
      storeDir,
      entry,
      'node_modules',
      npmName,
      'package.json',
    ),);
  },),);
  /**
   Distinct real versions found across all store copies, sorted for deterministic reporting.
   */
  const distinct = [
    ...new Set(versions.filter(function isVersion(version,): version is string {
      return version !== NO_MANIFEST_VERSION;
    },),),
  ]
    .toSorted();
  if (distinct.length === 0)
    return NOT_IN_STORE;
  return distinct;
}

/**
 Lists the child directory names of the pnpm store, converting an absent store
 directory into {@link NOT_IN_STORE} rather than surfacing a raw `ENOENT`.
 
 @param storeDir - absolute path to `<modulesDir>/.pnpm`
 
 @returns entry names, or {@link NOT_IN_STORE} when the directory cannot be read
 
 @example
 ```ts
 await readStoreEntries("/repo/node_modules/.pnpm") // ["\@types+mdx\@2.0.14", "openai\@6.26.0", ...]
 ```
 */
async function readStoreEntries(storeDir: string,): Promise<string[] | typeof NOT_IN_STORE> {
  try {
    return await readdir(storeDir,);
  }
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    return NOT_IN_STORE;
  }
}

/**
 Returns the store versions of the first candidate npm name that has any store
 copy, or {@link NOT_IN_STORE} when none of the candidates is in the store.
 Mirrors the candidate ordering of the symlink resolver so an aliased entry is
 classified by its installed key first.
 
 @param npmNames - ordered candidate npm names (catalog key first, alias target next)
 
 @param monorepoRoot - absolute path to the monorepo root
 
 @param modulesDir - per-importer modules directory name; `.pnpm` sits inside it
 
 @returns store versions of the first candidate found, or {@link NOT_IN_STORE}
 
 @example
 ```ts
 await firstStoreHit({ npmNames: ["\@types/mdx"], monorepoRoot: "/repo", modulesDir: "node_modules" }) // ["2.0.14"]
 ```
 */
export async function firstStoreHit(
  {
    npmNames,
    monorepoRoot: root,
    modulesDir: dir,
  }: {
    readonly npmNames: readonly string[];
    readonly monorepoRoot: string;
    readonly modulesDir: string;
  },
): Promise<readonly string[] | typeof NOT_IN_STORE> {
  /**
   Store lookup result for every candidate name, preserving candidate order.
   */
  const perName = await Promise.all(npmNames.map(async function probeStore(
    candidate,
  ): Promise<readonly string[] | typeof NOT_IN_STORE> {
    return await readStoreVersions({
      npmName: candidate,
      monorepoRoot: root,
      modulesDir: dir,
    },);
  },),);
  /**
   First candidate with a store copy; the symlink resolver already failed, so any hit is transitive-only.
   */
  const hit = perName.find(function isHit(
    result,
  ): result is readonly string[] {
    return result !== NOT_IN_STORE;
  },);
  return hit ?? NOT_IN_STORE;
}

//endregion Store probe
