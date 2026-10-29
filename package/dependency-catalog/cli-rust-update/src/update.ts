/**
 Recursive no-save update of every first-party `Cargo.lock`, the Cargo
 counterpart of `pnpm -r --no-save update`.

 The update is one transaction across all lockfiles: every lockfile is
 snapshotted first, each crate is re-resolved under the release-age cutoff,
 a version the trust policy refuses is set aside for an older release where one fits
 (as pnpm's picker does), and the union of newly locked versions is checked once. Any refusal, error,
 or a dry run restores every lockfile, so a bad version never stays in some
 crates and not others. Catalog floors are left to `catalog:tighten`, and
 manifests are never written.

 @module
 */

import {
  readFile,
  writeFile,
} from 'node:fs/promises';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { readRustCatalog, } from '@monochromatic-dev/dependency-catalog-core-rust/ts';

import {
  excludedUpdateSpecs,
  publishTimeCutoff,
  resolveCrate,
  type RunCargo,
  updateExempt,
} from './cargo.ts';
import {
  checkReleaseAge,
  checkTrust,
} from './checks.ts';
import {
  lockedCrateDirs,
  type LockSnapshot,
  readLocks,
  restoreLocks,
} from './crates.ts';
import {
  addedVersions,
  type LockedVersion,
  registryVersions,
} from './lockfile-diff.ts';
import { l, } from './logger.ts';
import { pruneExcludes, } from './prune.ts';
import type { Registry, } from './registry.ts';
import { setAsideUntilClean, } from './set-aside.ts';

//region Types

/**
 Outcome of an update run.

 @example
 ```ts
 const report: UpdateReport = { crateDirs: [], added: [], problems: [], warnings: [], fellBack: [], written: false, pruned: [], };
 ```
 */
export type UpdateReport = {
  readonly crateDirs: readonly string[];
  /**
   Versions the update newly locked (kept or restored).
   */
  readonly added: readonly LockedVersion[];
  /**
   Findings that refused the update.
   */
  readonly problems: readonly string[];
  readonly warnings: readonly string[];
  /**
   Crates resolved without the release-age cutoff because `minimumReleaseAgeStrict` is false.
   */
  readonly fellBack: readonly string[];
  /**
   Whether the new lockfiles were kept.
   */
  readonly written: boolean;
  /**
   Exclude selectors removed from `rust-catalog.toml`.
   */
  readonly pruned: readonly string[];
};

//endregion Types

//region Resolve

/**
 Re-resolves every crate in turn; cargo serializes on its package cache lock anyway.

 @param root - repository root

 @param crateDirs - crates to resolve

 @param cutoff - `--publish-time` value, empty for none

 @param strict - `minimumReleaseAgeStrict`

 @param exclude - `minimumReleaseAgeExclude`

 @param runCargo - cargo runner

 @returns crates that fell back to an unconstrained resolution

 @throws CargoResolveError when a crate cannot resolve

 @example
 ```ts
 await resolveAll({ root, crateDirs, cutoff, strict: true, exclude: [], runCargo, });
 ```
 */
async function resolveAll(
  {
    root,
    crateDirs,
    cutoff,
    strict,
    exclude,
    runCargo,
  }: {
    readonly root: string;
    readonly crateDirs: readonly string[];
    readonly cutoff: string;
    readonly strict: boolean;
    readonly exclude: readonly string[];
    readonly runCargo: RunCargo;
  },
): Promise<readonly string[]> {
  /**
   Logger for progress.
   */
  const log = tagged({
    tag: resolveAll.name,
    l,
  },);
  /**
   Crates that fell back.
   */
  const fellBack: string[] = [];
  for (const crateDir of crateDirs) {
    log.info(`resolving ${crateDir}${cutoff === '' ? '' : ` with releases published before ${cutoff}`}`,);
    /**
     Whether this crate fell back to an unconstrained resolution.
     */
    // oxlint-disable-next-line no-await-in-loop -- crates share cargo's package cache lock, so parallel runs only queue.
    const { fellBack: crateFellBack, } = await resolveCrate({
      root,
      crateDir,
      cutoff,
      strict,
      runCargo,
    },);
    if (crateFellBack)
      fellBack.push(crateDir,);
    /**
     Lockfile this crate's resolution just wrote.
     */
    // oxlint-disable-next-line no-await-in-loop -- the exempt update reads the lockfile this crate's resolution just wrote.
    const [snapshot,] = await readLocks({
      root,
      crateDirs: [crateDir,],
    },);
    // oxlint-disable-next-line no-await-in-loop -- same crate, same cargo lock as the resolution.
    await updateExempt({
      root,
      crateDir,
      specs: excludedUpdateSpecs({
        lock: snapshot?.source ?? '',
        selectors: exclude,
      },),
      runCargo,
    },);
  }
  return fellBack;
}

//endregion Resolve

//region Update

/**
 Updates every first-party lockfile under the catalog's release-age and trust policy.

 @param root - repository root

 @param dryRun - report what would change, then restore every lockfile

 @param now - current time in milliseconds

 @param runCargo - cargo runner

 @param registry - registry lookups

 @returns what changed and why it was kept or refused

 @throws RustCatalogError when `rust-catalog.toml` is invalid

 @throws CargoResolveError when a crate cannot resolve (every lockfile is restored first)

 @throws RegistryError when crates.io cannot be read (every lockfile is restored first)

 @example
 ```ts
 await updateRustLockfiles({ root, dryRun: true, now: Date.now(), runCargo: spawnCargo, registry, });
 ```
 */
export async function updateRustLockfiles(
  {
    root,
    dryRun,
    now,
    runCargo,
    registry,
  }: {
    readonly root: string;
    readonly dryRun: boolean;
    readonly now: number;
    readonly runCargo: RunCargo;
    readonly registry: Registry;
  },
): Promise<UpdateReport> {
  /**
   Logger for the run.
   */
  const log = tagged({
    tag: updateRustLockfiles.name,
    l,
  },);
  /**
   Catalog file and settings.
   */
  const catalog = await readRustCatalog({ rootDir: root, },);
  /**
   Release-age and trust settings.
   */
  const { settings, } = catalog.document;
  /**
   Crate roots with a lockfile.
   */
  const crateDirs = await lockedCrateDirs(root,);
  /**
   Lockfiles before the update.
   */
  const before: readonly LockSnapshot[] = await readLocks({
    root,
    crateDirs,
  },);
  log.info(`snapshotted ${String(before.length,)} lockfiles`,);
  try {
    /**
     Release-age cutoff as `--publish-time` takes it, empty for none.
     */
    const cutoff = settings.minimumReleaseAge === 0 ? '' : publishTimeCutoff({
      now,
      minutes: settings.minimumReleaseAge,
    },);
    /**
     Crates resolved without the cutoff.
     */
    const fellBack = await resolveAll({
      root,
      crateDirs,
      cutoff,
      strict: settings.minimumReleaseAgeStrict,
      exclude: settings.minimumReleaseAgeExclude,
      runCargo,
    },);
    /**
     Trust refusals moved to older releases, as pnpm's picker does.
     */
    const setAsideWarnings = await setAsideUntilClean({
      root,
      crateDirs,
      before: before.map(function toSource(snapshot,): string {
        return snapshot.source;
      },),
      registry,
      settings,
      now,
      cutoffMs: cutoff === '' ? 0 : Date.parse(cutoff,),
      runCargo,
    },);
    /**
     Lockfile texts after the update.
     */
    const after = (await readLocks({
      root,
      crateDirs,
    },)).map(function toSource(snapshot: LockSnapshot,): string {
      return snapshot.source;
    },);
    /**
     Newly locked versions across every crate.
     */
    const added = addedVersions({
      before: before.map(function toSource(snapshot,): string {
        return snapshot.source;
      },),
      after,
    },);
    /**
     Release-age findings.
     */
    const age = await checkReleaseAge({
      versions: added,
      settings,
      registry,
      now,
    },);
    /**
     Refusals.
     */
    const problems = [
      ...age.problems,
      ...await checkTrust({
        versions: added,
        settings,
        registry,
        now,
      },),
    ];
    /**
     Whether to keep the new lockfiles.
     */
    const keep = (problems.length === 0) && (!dryRun);
    if (!keep) {
      log.info(dryRun ? 'dry run: restoring every lockfile' : `refused by ${String(problems.length,)} findings: restoring every lockfile`,);
      await restoreLocks({
        root,
        snapshots: before,
      },);
      return {
        crateDirs,
        added,
        problems,
        warnings: [
          ...setAsideWarnings,
          ...age.warnings,
        ],
        fellBack,
        written: false,
        pruned: [],
      };
    }
    /**
     Catalog with stale exclude selectors pruned.
     */
    const pruned = pruneExcludes({
      source: await readFile(
        catalog.sourcePath,
        'utf8',
      ),
      settings,
      locked: registryVersions(after,),
    },);
    if (pruned.removed
      .length
      > 0)
      await writeFile(
        catalog.sourcePath,
        pruned.source,
      );
    return {
      crateDirs,
      added,
      problems,
      warnings: [
        ...setAsideWarnings,
        ...age.warnings,
      ],
      fellBack,
      written: true,
      pruned: pruned.removed,
    };
  } catch (error) {
    log.error(`update failed (${String(error,)}); restoring every lockfile`,);
    await restoreLocks({
      root,
      snapshots: before,
    },);
    throw error;
  }
}

//endregion Update
