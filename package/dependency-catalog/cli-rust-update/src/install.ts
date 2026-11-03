/**
 Lockfile install, the counterpart of `pnpm install`: makes every first-party `Cargo.lock` satisfy its manifests.

 `mise run sync:files` renders catalog floors (and path-crate versions) into each crate's manifest, the way pnpm
 resolves `catalog:` references at install time, and it never touches a lockfile. A crate whose lockfile held an older
 release is then left with a lockfile its own manifest refuses (`cargo --locked` exits 101). pnpm repairs the same
 situation with `pnpm install`, so this is the matching command, and it follows `pnpm install`'s lockfile rules:

 - Outside CI, a stale lockfile is updated (`--no-frozen-lockfile`, pnpm's "Allows lockfile updates during installation").
 - In CI, or with `--frozen-lockfile`, a stale lockfile is a failure that names the roots and writes nothing
   (pnpm: "fails to install if the lockfile is out of sync with the manifest").
 - `--dry-run` reports what an install would update without writing anything.
 - Like pnpm in a workspace, a root that still satisfies its manifests keeps its lockfile byte for byte, and in a stale
   root the dependencies whose requirements still hold keep their locked versions.

 Per crate root, offline:

 1. `cargo update --workspace --locked --offline` asks whether the lockfile already satisfies the manifests.
    It reads the index only, never writes, and exits 0 for a consistent lockfile.
 2. A refusal means the lockfile is stale. `cargo update --workspace --offline` re-locks the workspace members and what
    their changed requirements force, and keeps every other locked release (the repair cargo applies on its own
    when `--locked` is absent).
 3. `cargo metadata --locked --offline` confirms the result.

 A non-frozen install is one transaction like the update: every lockfile is snapshotted first, any failure or refusal
 restores all of them, and a registry version that no lockfile held before goes through the same release-age and trust
 checks as an update.

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
 Outcome of an install run.

 @example
 ```ts
 const report: InstallReport = { crateDirs: [], stale: [], added: [], problems: [], warnings: [], written: false, };
 ```
 */
export type InstallReport = {
  /**
   Every crate root that has a `Cargo.lock`.
   */
  readonly crateDirs: readonly string[];
  /**
   Crate roots whose lockfile did not satisfy its manifests (updated, or only reported under `--dry-run`).
   */
  readonly stale: readonly string[];
  /**
   Registry versions the install newly locked (kept or restored).
   */
  readonly added: readonly LockedVersion[];
  /**
   Findings that refused the install.
   */
  readonly problems: readonly string[];
  readonly warnings: readonly string[];
  /**
   Whether updated lockfiles were kept.
   */
  readonly written: boolean;
};

//endregion Types

//region Errors

/**
 Thrown by a frozen install when a lockfile needs an update, as `pnpm install --frozen-lockfile` fails in CI.

 @example
 ```ts
 throw new StaleLockfilesError({ staleDirs: ['package/cli/a',], });
 ```
 */
export class StaleLockfilesError extends Error {
  /**
   Repo-relative crate roots whose lockfile is stale.
   */
  readonly staleDirs: readonly string[];

  /**
   Builds the error.

   @param staleDirs - crate roots whose lockfile is stale
   */
  constructor({ staleDirs, }: { readonly staleDirs: readonly string[]; },) {
    super([
      `Frozen install failed: ${String(staleDirs.length,)} Cargo.lock file(s) are present but need an update to satisfy their manifests.`,
      ...staleDirs.map(function bullet(dir: string,): string {
        return `- ${dir}`;
      },),
      'Nothing was written. Frozen is the default in CI, as in pnpm; it is also set by --frozen-lockfile.',
      'To update them, run `mise run rust:install -- --no-frozen-lockfile` outside CI and commit the Cargo.lock changes,',
      'or run `mise run rust:update`, which resolves online under the release-age and trust policy.',
    ].join('\n',),);
    this.name = 'StaleLockfilesError';
    this.staleDirs = staleDirs;
  }
}

/**
 Thrown when cargo cannot bring a crate's lockfile in line.

 @example
 ```ts
 throw new CargoInstallError({ crateDir: 'package/cli/a', command: 'cargo update --workspace --offline', output: '...', });
 ```
 */
export class CargoInstallError extends Error {
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
      'The install runs cargo offline, so it can pick only releases the local registry index and cache already hold.',
      'When cargo says an index entry or package is missing, fetch it once with network access (`cargo fetch` in the crate)',
      'and run the install again, or run `mise run rust:update`, which resolves online under the release-age and trust policy.',
      'When cargo reports conflicting requirements, fix the manifest or the rust-catalog.toml floor that it names.',
    ].join('\n',),);
    this.name = 'CargoInstallError';
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
const UPDATE_ARGS = [
  'update',
  '--workspace',
  '--offline',
] as const;

/**
 The owner's consistency check, run after the update.
 */
const VERIFY_ARGS = [
  'metadata',
  '--locked',
  '--offline',
  '--format-version',
  '1',
] as const;

/**
 Whether a crate root's lockfile no longer satisfies its manifests, without writing anything.

 @param root - repository root

 @param crateDir - repo-relative crate directory

 @param runCargo - cargo runner

 @returns true when `cargo update --workspace --locked --offline` refuses

 @example
 ```ts
 await isStale({ root, crateDir: 'package/cli/a', runCargo, });
 ```
 */
async function isStale(
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
   Answer to whether the lockfile already satisfies the manifests.
   */
  const check = await runCargo({
    cwd: join(
      root,
      crateDir,
    ),
    args: CHECK_ARGS,
  },);
  return !check.ok;
}

/**
 Runs one cargo command in a crate root and throws when it fails.

 @param root - repository root

 @param crateDir - repo-relative crate directory

 @param args - cargo arguments

 @param runCargo - cargo runner

 @throws CargoInstallError when cargo exits non-zero

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
    throw new CargoInstallError({
      crateDir,
      command: `cargo ${args.join(' ',)}`,
      output: outcome.output,
    },);
}

//endregion Cargo

//region Install

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
 Crate roots whose lockfile is stale, found with read-only checks.

 @param root - repository root

 @param crateDirs - crate roots with a lockfile

 @param runCargo - cargo runner

 @returns stale roots in the order of `crateDirs`

 @example
 ```ts
 await staleRoots({ root, crateDirs, runCargo, });
 ```
 */
async function staleRoots(
  {
    root,
    crateDirs,
    runCargo,
  }: {
    readonly root: string;
    readonly crateDirs: readonly string[];
    readonly runCargo: RunCargo;
  },
): Promise<readonly string[]> {
  /**
   Stale roots found so far.
   */
  const stale: string[] = [];
  for (const crateDir of crateDirs) {
    // oxlint-disable-next-line no-await-in-loop -- crates share cargo's package cache lock, so parallel runs only queue.
    if (await isStale({
      root,
      crateDir,
      runCargo,
    },))
      stale.push(crateDir,);
  }
  return stale;
}

/**
 Makes every first-party lockfile satisfy its manifests, as `pnpm install` does, moving as little as the manifests force.

 @param root - repository root

 @param frozenLockfile - fail when a lockfile is stale instead of updating it (pnpm's `--frozen-lockfile`, the CI default)

 @param dryRun - report what would be updated, then restore every lockfile

 @param now - current time in milliseconds

 @param runCargo - cargo runner

 @param registry - registry lookups for versions that no lockfile held before

 @returns what was stale and whether the updated lockfiles were kept

 @throws StaleLockfilesError when `frozenLockfile` is true and a lockfile is stale (nothing is written)

 @throws RustCatalogError when `rust-catalog.toml` is invalid

 @throws CargoInstallError when a crate cannot be updated (every lockfile is restored first)

 @throws RegistryError when crates.io cannot be read for a newly locked version (every lockfile is restored first)

 @example
 ```ts
 await installRustLockfiles({ root, frozenLockfile: false, dryRun: false, now: Date.now(), runCargo: spawnCargo, registry, });
 ```
 */
export async function installRustLockfiles(
  {
    root,
    frozenLockfile,
    dryRun,
    now,
    runCargo,
    registry,
  }: {
    readonly root: string;
    readonly frozenLockfile: boolean;
    readonly dryRun: boolean;
    readonly now: number;
    readonly runCargo: RunCargo;
    readonly registry: Registry;
  },
): Promise<InstallReport> {
  /**
   Logger for the run.
   */
  const log = tagged({
    tag: installRustLockfiles.name,
    l,
  },);
  /**
   Crate roots with a lockfile.
   */
  const crateDirs = await lockedCrateDirs(root,);
  if (frozenLockfile) {
    /**
     Roots that would need an update.
     */
    const stale = await staleRoots({
      root,
      crateDirs,
      runCargo,
    },);
    if (stale.length > 0) {
      log.error(`frozen install: ${String(stale.length,)} stale lockfiles, nothing written`,);
      throw new StaleLockfilesError({ staleDirs: stale, },);
    }
    return {
      crateDirs,
      stale,
      added: [],
      problems: [],
      warnings: [],
      written: false,
    };
  }
  /**
   Release-age and trust settings.
   */
  const { settings, } = (await readRustCatalog({ rootDir: root, },)).document;
  /**
   Lockfiles before the install.
   */
  const before = await readLocks({
    root,
    crateDirs,
  },);
  log.info(`snapshotted ${String(before.length,)} lockfiles`,);
  try {
    /**
     Roots whose lockfile was stale and has been updated.
     */
    const stale: string[] = [];
    for (const crateDir of crateDirs) {
      // oxlint-disable-next-line no-await-in-loop -- crates share cargo's package cache lock, so parallel runs only queue.
      if (!await isStale({
        root,
        crateDir,
        runCargo,
      },)) {
        log.info(`${crateDir}: lockfile already satisfies its manifests`,);
        continue;
      }
      log.info(`${crateDir}: lockfile is stale, updating the workspace offline`,);
      // oxlint-disable-next-line no-await-in-loop -- same crate, same cargo lock as the check.
      await runOrThrow({
        root,
        crateDir,
        args: UPDATE_ARGS,
        runCargo,
      },);
      // oxlint-disable-next-line no-await-in-loop -- same crate, same cargo lock as the check.
      await runOrThrow({
        root,
        crateDir,
        args: VERIFY_ARGS,
        runCargo,
      },);
      stale.push(crateDir,);
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
     Whether to keep the updated lockfiles.
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
      stale,
      added,
      problems,
      warnings: age.warnings,
      written: keep,
    };
  } catch (error) {
    log.error(`install failed (${String(error,)}); restoring every lockfile`,);
    await restoreLocks({
      root,
      snapshots: before,
    },);
    throw error;
  }
}

//endregion Install
