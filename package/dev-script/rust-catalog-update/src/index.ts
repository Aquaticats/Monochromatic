/**
 Recursive no-save update of every first-party `Cargo.lock` under
 `rust-catalog.toml`'s release-age and trust policy, plus the CI trust check.

 @example
 ```ts
 import { cargoToolchain, createRegistry, createSpawnCargo, fetchTextWithUserAgent, updateRustLockfiles, } from '@monochromatic-dev/dev-script-rust-catalog-update';

 const registry = createRegistry({ root: process.cwd(), fetchText: fetchTextWithUserAgent, });
 const runCargo = createSpawnCargo({ toolchain: await cargoToolchain({ root: process.cwd(), explicit: process.env.RUSTUP_TOOLCHAIN ?? '', }), });
 await updateRustLockfiles({ root: process.cwd(), dryRun: true, now: Date.now(), runCargo, registry, });
 ```

 @packageDocumentation
 */

export {
  type CargoOutcome,
  CargoResolveError,
  createSpawnCargo,
  excludedUpdateSpecs,
  publishTimeCutoff,
  resolveCrate,
  type RunCargo,
  spawnCargo,
  updateExempt,
} from './cargo.ts';
export {
  checkReleaseAge,
  type CheckOutcome,
  checkTrust,
  groupByName,
} from './checks.ts';
export {
  main,
  parseMode,
  RustCatalogPolicyError,
  UsageError,
} from './cli.ts';
export {
  lockedCrateDirs,
  type LockSnapshot,
  readLocks,
  restoreLocks,
} from './crates.ts';
export {
  addedVersions,
  type LockedVersion,
  registryVersions,
  versionKey,
} from './lockfile-diff.ts';
export {
  releaseAgeProblems,
  trustLevel,
  trustProblems,
  type VersionRecord,
} from './policy.ts';
export { pruneExcludes, } from './prune.ts';
export {
  API_MIN_INTERVAL_MS,
  createRegistry,
  type FetchText,
  fetchTextWithUserAgent,
  HISTORY_CACHE_DIR,
  parseIndexPubtimes,
  parseVersionHistory,
  type Registry,
  RegistryError,
  sparseIndexPath,
  USER_AGENT,
} from './registry.ts';
export {
  isExcluded,
  matchesPattern,
  type ParsedSelector,
  parseSelector,
  pruneSelectors,
} from './selectors.ts';
export {
  cargoToolchain,
  rustToolchainFromLock,
} from './toolchain.ts';
export {
  type UpdateReport,
  updateRustLockfiles,
} from './update.ts';
export { verifyRustLockfiles, } from './verify.ts';
