/**
 CI verification of newly locked registry versions against the trust policy.

 The check covers only versions that no lockfile held at a base revision,
 the counterpart of pnpm's `trustLockfile: true`:
 the owner never accepts pull requests that change a lockfile,
 so every committed lockfile comes from the owner's own sessions and is part of the trusted base
 (doc/decision/rust-dependency-catalog.md).
 Release age is a resolution-time rule here, so locked versions are not re-gated on it.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { readRustCatalog, } from '@monochromatic-dev/dependency-catalog-core-rust/ts';

import { checkTrust, } from './checks.ts';
import {
  lockedCrateDirs,
  type LockSnapshot,
  readLocks,
} from './crates.ts';
import { addedVersions, } from './lockfile-diff.ts';
import { l, } from './logger.ts';
import type { Registry, } from './registry.ts';

//region Verify

/**
 Reads one crate's lockfile text at the base revision.

 Returns an empty string when the lockfile did not exist there, so every version it holds counts as new.
 */
export type ReadBaseLock = (crateDir: string,) => Promise<string>;

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
 Trust problems of the registry versions that no lockfile held at the base revision.

 @param root - repository root

 @param now - current time in milliseconds

 @param registry - registry lookups

 @param readBaseLock - base-revision lockfile reader

 @returns problems, empty when every new version passes

 @throws RustCatalogError when `rust-catalog.toml` is invalid

 @throws RegistryError when crates.io cannot be read

 @example
 ```ts
 await verifyRustLockfiles({ root, now: Date.now(), registry, readBaseLock, });
 ```
 */
export async function verifyRustLockfiles(
  {
    root,
    now,
    registry,
    readBaseLock,
  }: {
    readonly root: string;
    readonly now: number;
    readonly registry: Registry;
    readonly readBaseLock: ReadBaseLock
  },
): Promise<readonly string[]> {
  /**
   Logger for the run.
   */
  const log = tagged({
    tag: verifyRustLockfiles.name,
    l,
  },);
  /**
   Catalog settings.
   */
  const { document, } = await readRustCatalog({ rootDir: root, },);
  /**
   Every current lockfile.
   */
  const locks = await readLocks({
    root,
    crateDirs: await lockedCrateDirs(root,),
  },);
  /**
   The same lockfiles at the base revision.
   */
  const baseSources = await Promise.all(locks.map(async function readBase(snapshot: LockSnapshot,): Promise<string> {
    return await readBaseLock(snapshot.crateDir,);
  },),);
  /**
   Registry versions no lockfile held at the base revision.
   */
  const versions = addedVersions({
    before: baseSources,
    after: locks.map(sourceOf,),
  },);
  log.info(`verifying ${String(versions.length,)} newly locked versions from ${String(locks.length,)} lockfiles`,);
  if (versions.length === 0)
    return [];
  return await checkTrust({
    versions,
    settings: document.settings,
    registry,
    now,
  },);
}

//endregion Verify
