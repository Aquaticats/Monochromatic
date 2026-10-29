/**
 Canonical `Cargo.toml` enforcement across the workspace-free crate fleet.
 
 Discovers every crate manifest, filters build-artifact and dependency copies,
 and rewrites each in place to match a {@link CargoManifestSpec}. There is no
 Cargo workspace here (deliberately), so this generator is the single source of
 truth for the properties the spec owns. See
 `doc/planning/cargo-toml-file-enforcer.md`.
 
 @module
 */

import { glob, } from 'node:fs/promises';

import {
  ABSENT_FILE_CONTENT,
  overwrite,
  readExisting,
} from '../io/write.ts';
import { addWatchedPaths, } from '../tracker.ts';
import { applyCargoPlan, } from './apply-plan.ts';
import type { CargoManifestSpec, } from './types.ts';

/**
 Path fragments whose presence marks a discovered manifest as a build-artifact
 or dependency copy rather than a first-party crate.
 */
const CARGO_MANIFEST_EXCLUDED_FRAGMENTS = [
  '/target/',
  '/node_modules/',
] as const;

/**
 Whether a discovered manifest path names a first-party crate manifest.
 
 @param manifestPath - Path returned by the discovery glob
 
 @returns Whether path is outside every excluded fragment
 
 @example
 ```ts
 isFirstPartyManifest('package/linter/monochromatic-lint/Cargo.toml'); // true
 ```
 */
function isFirstPartyManifest(manifestPath: string,): boolean {
  return CARGO_MANIFEST_EXCLUDED_FRAGMENTS
    .every(function excludes(fragment,): boolean {
      return !manifestPath.includes(fragment,);
    },);
}

/**
 Enforces one manifest against the spec, skipping the write when the plan
 leaves the content unchanged (splice mode makes that the common case).
 
 @param manifestPath - First-party crate manifest path
 
 @param spec - Plan builder for the fleet
 
 @example
 ```ts
 await enforceManifest({ manifestPath: 'package/linter/monochromatic-lint/Cargo.toml', spec, });
 ```
 */
async function enforceManifest(
  {
    manifestPath,
    spec,
  }: {
    readonly manifestPath: string;
    readonly spec: CargoManifestSpec;
  },
): Promise<void> {
  /**
   Current manifest text; a concurrently-removed file is left alone.
   */
  const content = await readExisting(manifestPath,);
  if (content === ABSENT_FILE_CONTENT)
    return;

  /**
   Text after the plan applies; equal to content when nothing was owned-drift.
   */
  const next = applyCargoPlan({
    content,
    plan: spec({ manifestPath, },),
  },);
  if (next === content)
    return;

  await overwrite({
    dest: manifestPath,
    content: next,
  },);
}

/**
 Discovers and enforces every first-party `Cargo.toml` under a glob.
 
 Manifests are registered for watch-mode protection (they are their own
 source and destination; the content-skip in {@link overwrite} prevents a
 write/watch loop once converged).
 
 Discovery uses explicit-depth globs rather than a recursive `**` so traversal
 never descends into the deep gitignored `target/` trees (which vendor many
 dependency manifests); the fragment filter stays as a safety net.
 
 @param manifestGlobs - Bounded-depth discovery globs, for example `package/​*​/​*​/Cargo.toml`
 
 @param spec - Plan builder describing every owned edit
 
 @example
 ```ts
 await manageCargoManifests({
   manifestGlobs: ['package/​*​/​*​/Cargo.toml', 'package/​*​/​*​/​*​/Cargo.toml'],
   spec,
 });
 ```
 */
export async function manageCargoManifests(
  {
    manifestGlobs,
    spec,
  }: {
    readonly manifestGlobs: readonly string[];
    readonly spec: CargoManifestSpec;
  },
): Promise<void> {
  /**
   First-party manifest paths gathered concurrently across every discovery glob.
   */
  const manifestPaths = (await Promise.all(manifestGlobs.map(
    async function collectGlob(manifestGlob,): Promise<readonly string[]> {
      return (await Array.fromAsync(glob(manifestGlob,),))
        .filter(isFirstPartyManifest,);
    },
  ),)).flat();
  /**
   Unique paths, sorted for deterministic logging and safe from overlapping globs.
   */
  const uniqueManifestPaths = [...new Set(manifestPaths,),].toSorted();
  addWatchedPaths(uniqueManifestPaths,);

  await Promise.all(uniqueManifestPaths.map(async function enforceOne(manifestPath,): Promise<void> {
    await enforceManifest({
      manifestPath,
      spec,
    },);
  },),);
}
