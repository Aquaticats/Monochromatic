/**
 Lock-only reconciliation: makes every first-party `Cargo.lock` satisfy its manifests again after a manifest change.

 `mise run sync:files` renders catalog floors (and path-crate versions) into each crate's manifest,
 and nothing in that step touches a lockfile, so every crate whose lock held an older release is left with a lockfile
 that `cargo --locked` refuses. The recursive update cannot repair that: it runs before the floors move
 and re-resolves from scratch. This module is the missing step. It runs after the sync and moves only what the
 manifests force to move.

 Per crate root, offline:

 1. `cargo update --workspace --locked --offline` asks whether the lockfile already satisfies the manifests.
    It reads the index only, never writes, and exits 0 for a consistent lockfile, which is then left byte-for-byte alone.
 2. A refusal means the lockfile is stale. `cargo update --workspace --offline` re-locks the workspace members and what
    their changed requirements force, and keeps every other locked release (identical to the repair cargo applies
    on its own when `--locked` is absent).
 3. `cargo metadata --locked --offline` confirms the result, the check the owner measured the ten stale locks with.

 Like the update, it is one transaction: every lockfile is snapshotted first, any failure or refusal restores all of them,
 and a version that no lockfile held before goes through the same release-age and trust checks as an update.

 @module
 */

import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { readRustCatalog, } from '@monochromatic-dev/dependency-catalog-core-rust/ts';

import type {
  CargoOutcome,
  RunCargo,
} from './cargo.ts';
import {
  checkReleaseAge,
  type CheckOutcome,
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
} from './lockfile-diff.ts';
import { l, } from './logger.ts';
import type { Registry, } from './registry.ts';

//region Types

/**
 Outcome of a reconcile run.

 @example
 ```ts
 const report: ReconcileReport = { crateDirs: [], reconciled: [], added: [], problems: [], warnings: [], written: false, };
 ```
 */
export type ReconcileReport = {
  /**
   Every crate root that has a `Cargo.lock`.
   */
  readonly crateDirs: readonly string[];
  /**
   Crate roots whose lockfile did not satisfy its manifests and was regenerated.
   */
  readonly reconciled: readonly string[];
  /**
   Registry versions the reconcile newly locked (kept or restored).
   */
  readonly added: readonly LockedVersion[];
  /**
   Findings that refused the reconcile.
   */
  readonly problems: readonly string[];
  readonly warnings: readonly string[];
  /**
   Whether the regenerated lockfiles were kept.
   */
  readonly written: boolean;
};

//endregion Types

//region Errors

/**
 Thrown when cargo cannot reconcile a crate's lockfile.

 @example
 ```ts
 throw new CargoReconcileError({ crateDir: 'package/cli/a', command: 'cargo update --workspace --offline', output: '...', });
 ```
 */
export class CargoReconcileError extends Error {
  /**
   Repo-relative crate directory.
   */
  readonly crateDir: string;

  /**
   Builds the error.

   @param crateDir - crate directory

   @param command - command that failed

   @param output - cargo output
   */
  constructor({
    crateDir,
    command,
    output,
  }: {
    readonly crateDir: string;
    readonly command: string;
    readonly output: string
  },) {
    super([
      `\`${command}\` failed in ${crateDir}; every lockfile was restored.`,
      'Cargo output:',
      output.trim(),
      'The reconcile runs cargo offline, so it can pick only releases the local registry index and cache already hold.',
      'When cargo says an index entry or package is missing, fetch it once with network access (`cargo fetch` in the crate)',
      'and run the reconcile again, or run `mise run rust:update`, which resolves online under the release-age and trust policy.',
      'When cargo reports conflicting requirements, fix the manifest or the rust-catalog.toml floor that it names.',
    ].join('\n',),);
    this.name = 'CargoReconcileError';
    this.crateDir = crateDir;
  }
}

//endregion Errors

//region Cargo

/**
 Asks whether the lockfile satisfies the manifests: exits 0 without writing when it does.
 */
const CHECK_ARGS = [
  'update',
  '--workspace',
  '--locked',
  '--offline',
] as const;

/**
 Re-locks the workspace members and whatever their changed requirements force, keeping every other locked release.
 */
const RECONCILE_ARGS = [
  'update',
  '--workspace',
  '--offline',
] as const;

/**
 The owner's consistency check, run after the reconcile.
 */
const VERIFY_ARGS = [
  'metadata',
  '--locked',
  '--offline',
  '--format-version',
  '1',
] as const;

/**
 Runs one cargo command in a crate root and throws when it fails.

 @param root - repository root

 @param crateDir - repo-relative crate directory

 @param args - cargo arguments

 @param runCargo - cargo runner

 @throws CargoReconcileError when cargo exits non-zero

 @example
 ```ts
 await runOrThrow({ root, crateDir, args: ['update', '--workspace', '--offline',], runCargo, });
 ```
 */
async function runOrThrow(
  {
    root,
    crateDir,
    args,
    runCargo,
  }: {
    readonly root: string;
    readonly crateDir: string;
    readonly args: readonly string[];
    readonly runCargo: RunCargo;
  },
): Promise<void> {
  /**
   Outcome of the command.
   */
  const outcome: CargoOutcome = await runCargo({
    cwd: join(
      root,
      crateDir,
    ),
    args,
  },);
  if (!outcome.ok)
    throw new CargoReconcileError({
      crateDir,
      command: `cargo ${args.join(' ',)}`,
      output: outcome.output,
    },);
}

/**
 Reconciles one crate root's lockfile.

 @param root - repository root

 @param crateDir - repo-relative crate directory

 @param runCargo - cargo runner

 @returns whether the lockfile was stale and has been regenerated

 @throws CargoReconcileError when the update fails or its result still fails `cargo metadata --locked --offline`

 @example
 ```ts
 await reconcileCrate({ root, crateDir: 'package/cli/a', runCargo, }); // false when nothing was stale
 ```
 */
async function reconcileCrate(
  {
    root,
    crateDir,
    runCargo,
  }: {
    readonly root: string;
    readonly crateDir: string;
    readonly runCargo: RunCargo;
  },
): Promise<boolean> {
  /**
   Logger for this crate.
   */
  const log = tagged({
    tag: reconcileCrate.name,
    l,
  },);
  /**
   Answer to whether the lockfile already satisfies the manifests.
   */
  const check = await runCargo({
    cwd: join(
      root,
      crateDir,
    ),
    args: CHECK_ARGS,
  },);
  if (check.ok) {
    log.info(`${crateDir}: lockfile already satisfies its manifests`,);
    return false;
  }
  log.info(`${crateDir}: lockfile is stale, re-locking the workspace offline`,);
  await runOrThrow({
    root,
    crateDir,
    args: RECONCILE_ARGS,
    runCargo,
  },);
  await runOrThrow({
    root,
    crateDir,
    args: VERIFY_ARGS,
    runCargo,
  },);
  return true;
}

//endregion Cargo

//region Reconcile

/**
 Outcome of a check pass over no versions.
 */
const NO_FINDINGS: CheckOutcome = {
  problems: [],
  warnings: [],
};

/**
 Lockfile text of a snapshot.

 @param snapshot - one crate's lockfile

 @returns its text

 @example
 ```ts
 [snapshot,].map(sourceOf);
 ```
 */
function sourceOf(snapshot: LockSnapshot,): string {
  return snapshot.source;
}

/**
 Makes every first-party lockfile satisfy its manifests, moving as little as the manifests force.

 @param root - repository root

 @param dryRun - report what would be regenerated, then restore every lockfile

 @param now - current time in milliseconds

 @param runCargo - cargo runner

 @param registry - registry lookups for versions that no lockfile held before

 @returns what was regenerated and whether it was kept

 @throws RustCatalogError when `rust-catalog.toml` is invalid

 @throws CargoReconcileError when a crate cannot be reconciled (every lockfile is restored first)

 @throws RegistryError when crates.io cannot be read for a newly locked version (every lockfile is restored first)

 @example
 ```ts
 await reconcileRustLockfiles({ root, dryRun: false, now: Date.now(), runCargo: spawnCargo, registry, });
 ```
 */
export async function reconcileRustLockfiles(
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
): Promise<ReconcileReport> {
  /**
   Logger for the run.
   */
  const log = tagged({
    tag: reconcileRustLockfiles.name,
    l,
  },);
  /**
   Release-age and trust settings.
   */
  const { settings, } = (await readRustCatalog({ rootDir: root, },)).document;
  /**
   Crate roots with a lockfile.
   */
  const crateDirs = await lockedCrateDirs(root,);
  /**
   Lockfiles before the reconcile.
   */
  const before = await readLocks({
    root,
    crateDirs,
  },);
  log.info(`snapshotted ${String(before.length,)} lockfiles`,);
  try {
    /**
     Crate roots regenerated so far.
     */
    const reconciled: string[] = [];
    for (const crateDir of crateDirs) {
      // oxlint-disable-next-line no-await-in-loop -- crates share cargo's package cache lock, so parallel runs only queue.
      if (await reconcileCrate({
        root,
        crateDir,
        runCargo,
      },))
        reconciled.push(crateDir,);
    }
    /**
     Newly locked versions across every crate.
     */
    const added = addedVersions({
      before: before.map(sourceOf,),
      after: (await readLocks({
        root,
        crateDirs,
      },)).map(sourceOf,),
    },);
    /**
     Release-age findings for versions no lockfile held before.
     */
    const age = added.length === 0 ? NO_FINDINGS : await checkReleaseAge({
      versions: added,
      settings,
      registry,
      now,
    },);
    /**
     Trust findings for versions no lockfile held before.
     */
    const trust = added.length === 0 ? [] : await checkTrust({
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
      ...trust,
    ];
    /**
     Whether to keep the regenerated lockfiles.
     */
    const keep = (problems.length === 0) && (!dryRun);
    if (!keep) {
      log.info(dryRun ? 'dry run: restoring every lockfile' : `refused by ${String(problems.length,)} findings: restoring every lockfile`,);
      await restoreLocks({
        root,
        snapshots: before,
      },);
    }
    return {
      crateDirs,
      reconciled,
      added,
      problems,
      warnings: age.warnings,
      written: keep,
    };
  } catch (error) {
    log.error(`reconcile failed (${String(error,)}); restoring every lockfile`,);
    await restoreLocks({
      root,
      snapshots: before,
    },);
    throw error;
  }
}

//endregion Reconcile
