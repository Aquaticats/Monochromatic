/**
 Discovers crate manifests, syncs them with `rust-catalog.toml`, and writes the results.

 Reads go through the file-enforcer read cache so this step sees what earlier
 steps (the Cargo manifest engine) produced in the same run; writes go through
 {@link overwrite}, so `--check` reports drift instead of writing.

 @module
 */

import { glob, } from 'node:fs/promises';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { RUST_CATALOG_FILENAME, } from '@monochromatic-dev/dependency-catalog-core-rust/ts';

import {
  ABSENT_FILE_CONTENT,
  overwrite,
  readExisting,
} from '../../io/write.ts';
import { l, } from '../../logger.ts';
import { addWatchedPaths, } from '../../tracker.ts';
import { syncRustCatalog, } from './sync.ts';
import type { ManifestFile, } from './types.ts';

//region Discovery

/**
 Path fragments marking a manifest as a build-artifact or dependency copy.
 */
const EXCLUDED_FRAGMENTS = [
  '/target/',
  '/node_modules/',
] as const;

/**
 Reads every first-party manifest matched by the globs, deduplicated and sorted.

 @param manifestGlobs - bounded-depth discovery globs

 @returns manifest paths and texts

 @example
 ```ts
 await readManifests(['package/<category>/<crate>/Cargo.toml',]);
 ```
 */
async function readManifests(manifestGlobs: readonly string[],): Promise<readonly ManifestFile[]> {
  /**
   Matched paths across every glob.
   */
  const paths = (await Promise.all(manifestGlobs.map(async function collect(pattern,): Promise<readonly string[]> {
    return await Array.fromAsync(glob(pattern,),);
  },),)).flat()
    .filter(function isFirstParty(path,): boolean {
      return EXCLUDED_FRAGMENTS.every(function excludes(fragment,): boolean {
        return !path.includes(fragment,);
      },);
    },);
  /**
   Unique sorted paths.
   */
  const unique = [...new Set(paths,),].toSorted();
  /**
   Manifest texts; a concurrently removed file is skipped.
   */
  const files = await Promise.all(unique.map(async function read(path,): Promise<readonly ManifestFile[]> {
    /**
     Current text.
     */
    const source = await readExisting(path,);
    return source === ABSENT_FILE_CONTENT ? [] : [{
      path,
      source,
    },];
  },),);
  return files.flat();
}

//endregion Discovery

//region Manage

/**
 Syncs every crate manifest with `rust-catalog.toml` in `rootDir`.

 A repository without the catalog file is left alone.

 @param rootDir - directory holding `rust-catalog.toml`

 @param manifestGlobs - bounded-depth manifest discovery globs, relative to the working directory

 @param manifestPath - optional staleness manifest path override forwarded to every write

 @throws RustCatalogError when the catalog is invalid

 @throws RustCatalogSyncError when a manifest needs a human decision; nothing is written

 @example
 ```ts
 await manageRustCatalog({ rootDir: '.', manifestGlobs: ['package/<category>/<crate>/Cargo.toml',], });
 ```
 */
export async function manageRustCatalog(
  {
    rootDir,
    manifestGlobs,
    manifestPath,
  }: {
    readonly rootDir: string;
    readonly manifestGlobs: readonly string[];
    readonly manifestPath?: string;
  },
): Promise<void> {
  /**
   Logger tagged for this step.
   */
  const log = tagged({
    tag: manageRustCatalog.name,
    l,
  },);
  /**
   Catalog path.
   */
  const catalogPath = join(
    rootDir,
    RUST_CATALOG_FILENAME,
  );
  /**
   Catalog text.
   */
  const catalogSource = await readExisting(catalogPath,);
  if (catalogSource === ABSENT_FILE_CONTENT) {
    log.info(`no ${catalogPath}; skipping the Rust dependency catalog sync`,);
    return;
  }
  /**
   Every first-party manifest.
   */
  const manifests = await readManifests(manifestGlobs,);
  addWatchedPaths([
    catalogPath,
    ...manifests.map(function toPath(manifest,): string {
      return manifest.path;
    },),
  ],);
  log.debug(`syncing ${String(manifests.length,)} manifests with ${catalogPath}`,);

  /**
   New texts and reports.
   */
  const result = syncRustCatalog({
    catalogPath,
    catalogSource,
    manifests,
  },);
  for (const warning of result.warnings)
    log.warn(warning,);
  if (result.affectedCrates
    .length
    > 0)
    log.info(`affected crates (generated [workspace.dependencies] changed): ${result.affectedCrates
      .join(', ',)}`,);

  /**
   Manifest override spread into each write; empty so production callers keep the default manifest.
   */
  const manifestPathOption = manifestPath === undefined
    ? {}
    : { manifestPath, };
  await overwrite({
    dest: catalogPath,
    content: result.catalog,
    ...manifestPathOption,
  },);
  await Promise.all([...result.manifests
    .keys(),].map(async function write(path: string,): Promise<void> {
      await overwrite({
        dest: path,
        content: result.manifests
          .get(path,)
          ?? '',
        ...manifestPathOption,
      },);
    },),);
}

//endregion Manage
