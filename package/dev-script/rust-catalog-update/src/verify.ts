/**
 CI verification of the committed lockfiles against the trust policy.

 The catalog decision keeps pnpm's default `trustLockfile: false`: every
 locked registry version is re-checked, not only new ones. Release age is a
 resolution-time rule, as in pnpm, so committed lockfiles are not re-gated on it.

 @module
 */

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import { readRustCatalog, } from '@monochromatic-dev/module-rust-catalog/ts';

import { checkTrust, } from './checks.ts';
import {
  lockedCrateDirs,
  type LockSnapshot,
  readLocks,
} from './crates.ts';
import { registryVersions, } from './lockfile-diff.ts';
import { l, } from './logger.ts';
import type { Registry, } from './registry.ts';

//region Verify

/**
 Trust problems of every locked registry version.

 @param root - repository root

 @param now - current time in milliseconds

 @param registry - registry lookups

 @returns problems, empty when every version passes

 @throws RustCatalogError when `rust-catalog.toml` is invalid

 @throws RegistryError when crates.io cannot be read

 @example
 ```ts
 await verifyRustLockfiles({ root, now: Date.now(), registry, });
 ```
 */
export async function verifyRustLockfiles(
  { root, now, registry, }: { readonly root: string; readonly now: number; readonly registry: Registry; },
): Promise<readonly string[]> {
  /**
   Logger for the run.
   */
  const log = tagged({ tag: verifyRustLockfiles.name, l, },);
  /**
   Catalog settings.
   */
  const { document, } = await readRustCatalog({ rootDir: root, },);
  /**
   Every lockfile.
   */
  const locks = await readLocks({ root, crateDirs: await lockedCrateDirs(root,), },);
  /**
   Distinct locked registry versions.
   */
  const versions = registryVersions(locks.map(function toSource(snapshot: LockSnapshot,): string {
    return snapshot.source;
  },),);
  log.info(`verifying ${String(versions.length,)} locked versions from ${String(locks.length,)} lockfiles`,);
  return await checkTrust({ versions, settings: document.settings, registry, now, },);
}

//endregion Verify
