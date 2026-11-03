//! What:
//! The two checks a frozen install makes of the registry versions in the lockfiles:
//! every entry (`trustLockfile = false`,
//! pnpm's
//! default,
//! below) or only the versions new since a base revision (`trustLockfile = true`,
//! `verify_added_since_base`).
//! Why:
//! pnpm 12.16.1 re-validates every entry of the lockfile on install,
//! frozen or not,
//! against the policies its resolvers were built
//! with,
//! unless `trustLockfile` is true (`pnpm11/installing/deps-installer/src/install/mutateModules.ts:314-343`,
//! `startLockfileVerification`:
//! "Re-validate every entry in the lockfile against the policies the resolver chain was built
//! with";
//! skipped when `opts.trustLockfile`).
//! A passing result is cached by a hash of the lockfile
//! (`pnpm11/installing/deps-installer/src/install/verifyLockfileResolutionsCache.ts`).
//! This is that check for `Cargo.lock`:
//! the
//! default,
//! no separate command.
//! The release-age rule needs the sparse index's publish time per crate;
//! the trust rule needs the
//! crate's history,
//! which the database dump fills in bulk.
//! A passing lockfile is remembered under
//! `.cache/cargo-shadow/verified/`,
//! keyed by a SHA-256 of the lockfile text and the policy settings,
//! so an unchanged lockfile costs
//! nothing the next time;
//! a failing one is never remembered.
//! Which of the two checks runs is the catalog setting `trustLockfile` (default false, as in pnpm;
//! the repository's `rust-catalog.toml` sets it true by the owner's decision of 2026-10-29),
//! read by the caller (`install.rs`), not here.
//! A remembered pass is keyed by a scope string as well
//! (the install mode, and for `trustLockfile = true` the base spelling),
//! so a pass under one mode, policy or base is never taken for a pass under another.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // verify.ts
//! export async function verifyLockedEntries({ root, snapshots, settings, registry, now }): Promise<string[]>
//! ```

/// What:
/// `use std::fs;` imports `fs` from the standard library so this file can name it unqualified.
/// Why:
/// `fs` is used by `verify_locked_entries`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { fs } from 'std';
/// ```
use std::fs;
/// What:
/// `use std::path::{Path, PathBuf};` imports `Path`,
/// `PathBuf` from the standard library so this file can name them unqualified.
/// Why:
/// `Path` is used by `cache_file`,
/// `verify_locked_entries`;
/// `PathBuf` is used by `cache_file`,
/// `verify_locked_entries`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Path, PathBuf } from 'std/path';
/// ```
use std::path::{Path, PathBuf};

/// What:
/// `use monochromatic_dependency_catalog::Settings;` imports `Settings` from the external crate `monochromatic_dependency_catalog` so this file can name it unqualified.
/// Why:
/// `Settings` is used by `cache_key`,
/// `cache_file`,
/// `verify_locked_entries`,
/// `verify_added_since_base`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Settings } from 'monochromatic_dependency_catalog';
/// ```
use monochromatic_dependency_catalog::Settings;
/// What:
/// `use sha2::{Digest, Sha256};` imports `Digest`,
/// `Sha256` from the external crate `sha2` so this file can name them unqualified.
/// Why:
/// `Digest` must be in scope for its methods:
/// `update` and `finalize` on the hasher;
/// `Sha256` is used by `cache_key`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Digest, Sha256 } from 'sha2';
/// ```
use sha2::{Digest, Sha256};

/// What:
/// `use crate::checks::{check_release_age, check_trust};` imports `check_release_age`,
/// `check_trust` from another module of this crate so this file can name them unqualified.
/// Why:
/// `check_release_age` is used by `verify_locked_entries`;
/// `check_trust` is used by `verify_locked_entries`,
/// `verify_added_since_base`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { check_release_age, check_trust } from 'crate/checks';
/// ```
use crate::checks::{check_release_age, check_trust};
/// What:
/// `use crate::crates::LockSnapshot;` imports `LockSnapshot` from another module of this crate so this file can name it unqualified.
/// Why:
/// `LockSnapshot` is used by `cache_key`,
/// `cache_file`,
/// `verify_locked_entries`,
/// `verify_added_since_base`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockSnapshot } from 'crate/crates';
/// ```
use crate::crates::LockSnapshot;
/// What:
/// `use crate::lockfile::{LockedVersion, added_versions, registry_versions};` imports `LockedVersion`,
/// `added_versions`,
/// `registry_versions` from another module of this crate so this file can name them unqualified.
/// Why:
/// `LockedVersion` is used by `verify_locked_entries`,
/// `verify_added_since_base`;
/// `added_versions` is used by `verify_added_since_base`;
/// `registry_versions` is used by `verify_locked_entries`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockedVersion, added_versions, registry_versions } from 'crate/lockfile';
/// ```
use crate::lockfile::{LockedVersion, added_versions, registry_versions};
/// What:
/// `use crate::registry::Registry;` imports `Registry` from another module of this crate so this file can name it unqualified.
/// Why:
/// `Registry` is used by `verify_locked_entries`,
/// `verify_added_since_base`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Registry } from 'crate/registry';
/// ```
use crate::registry::Registry;
/// What:
/// `use crate::adapt::message_of;` imports `message_of` from another module of this crate so this file can name it unqualified.
/// Why:
/// `message_of` is used by `verify_locked_entries`,
/// `verify_added_since_base`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { message_of } from 'crate/adapt';
/// ```
use crate::adapt::message_of;

/// Where passing digests are remembered,
/// relative to the repository root.
pub const VERIFIED_DIR: &str = ".cache/cargo-shadow/verified";

/// The lowercase hex of a SHA-256 digest,
/// written per byte (the digest type has no `LowerHex` in sha2 0.11).
fn hex(digest: &[u8]) -> String {
    let mut text: String = String::new();
    for byte in digest {
        text.push_str(&format!("{byte:02x}"));
    }
    return text;
}

/// The scope of the remembered passes of the frozen check of every locked entry.
pub const ALL_ENTRIES_SCOPE: &str = "frozen-all-entries";

/// What:
/// `pub fn cache_key(snapshot, settings, scope) -> String` is the cache key of one lockfile under one policy and one check:
/// a digest of the text, of every setting and of the scope.
/// Why:
/// A lockfile that passed one check (for example every entry, or only what is new since one base) says nothing about another
/// check, another base or another policy, so each of those gets its own key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const cacheKey = (snapshot: LockSnapshot, settings: Settings, scope: string): string => sha256(snapshot.source + JSON.stringify(settings) + scope);
/// ```
pub fn cache_key(snapshot: &LockSnapshot, settings: &Settings, scope: &str) -> String {
    let mut hasher: Sha256 = Sha256::new();
    hasher.update(snapshot.source.as_bytes());
    hasher.update(format!("{settings:?}").as_bytes());
    // A separator keeps the lockfile text and the scope from running into each other.
    hasher.update([0u8]);
    hasher.update(scope.as_bytes());
    return hex(hasher.finalize().as_slice());
}

/// What:
/// `fn cache_file(root, snapshot, settings, scope) -> PathBuf` is the marker file for one lockfile under one policy and check.
/// Why:
/// The lookup before a check and the write after a pass must agree on the name,
/// which holds the crate directory and the digest of the lockfile text, settings and scope.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// cacheFile(root: string, snapshot: LockSnapshot, settings: Settings, scope: string): string
/// ```
fn cache_file(root: &Path, snapshot: &LockSnapshot, settings: &Settings, scope: &str) -> PathBuf {
    return root.join(VERIFIED_DIR).join(format!("{}.{}.ok", snapshot.crate_dir.replace('/', "__"), cache_key(snapshot, settings, scope)));
}

/// What:
/// `pub fn is_verified(root, snapshot, settings, scope) -> bool` says whether this lockfile text passed this check under this
/// policy before.
/// Why:
/// An unchanged, already verified lockfile costs one file lookup instead of registry requests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const isVerified = (root, snapshot, settings, scope): boolean => existsSync(cacheFile(root, snapshot, settings, scope));
/// ```
pub fn is_verified(root: &Path, snapshot: &LockSnapshot, settings: &Settings, scope: &str) -> bool {
    return cache_file(root, snapshot, settings, scope).exists();
}

/// What:
/// `pub fn mark_verified(root, snapshot, settings, scope)` remembers that this lockfile text passed this check.
/// Why:
/// Only a pass is ever written; a failing lockfile is checked again each time.
/// A write that fails only costs the next check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function markVerified(root, snapshot, settings, scope): void  // best effort
/// ```
pub fn mark_verified(root: &Path, snapshot: &LockSnapshot, settings: &Settings, scope: &str) {
    let path: PathBuf = cache_file(root, snapshot, settings, scope);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(path, "");
}

/// What:
/// `pub fn verify_locked_entries(...) -> Result<Vec<String>, String>` returns the findings against every registry version the
/// given lockfiles hold,
/// skipping a lockfile whose digest passed before.
/// Why:
/// The findings are the release-age problems (strict) and the trust problems;
/// the lockfiles that pass are remembered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function verifyLockedEntries(args): Promise<string[]>
/// ```
pub fn verify_locked_entries(
    root: &Path,
    snapshots: &[LockSnapshot],
    settings: &Settings,
    registry: &dyn Registry,
    now: i64,
) -> Result<Vec<String>, String> {
    let mut pending: Vec<&LockSnapshot> = Vec::new();
    for snapshot in snapshots {
        if !is_verified(root, snapshot, settings, ALL_ENTRIES_SCOPE) {
            pending.push(snapshot);
        }
    }
    if pending.is_empty() {
        return Ok(Vec::new());
    }
    let sources: Vec<String> = pending.iter().copied().map(LockSnapshot::source_text).collect();
    let versions: Vec<LockedVersion> = registry_versions(&sources)?;
    eprintln!("cargo-shadow: verifying {} locked versions from {} lockfiles", versions.len(), pending.len());
    let mut problems: Vec<String> = check_release_age(&versions, settings, registry, now).map_err(message_of)?.problems;
    problems.extend(check_trust(&versions, settings, registry, now).map_err(message_of)?);
    if problems.is_empty() {
        for snapshot in pending {
            mark_verified(root, snapshot, settings, ALL_ENTRIES_SCOPE);
        }
    }
    return Ok(problems);
}

/// What:
/// `pub fn verify_added_since_base(...) -> Result<Vec<String>, String>` returns the trust findings for the registry versions
/// that the given lockfiles hold and that the lockfiles at the base revision did not.
/// Why:
/// This is the frozen install under `trustLockfile = true`,
/// the owner's decision of 2026-10-29
/// (`doc/decision/rust-dependency-catalog.md`):
/// committed lockfiles are part of the trusted base because the owner never
/// accepts pull requests that change a lockfile,
/// so only what changed since the base is checked,
/// and only for trust;
/// release
/// age is a resolution-time rule,
/// so a version that was locked has already passed it.
/// A lockfile that did not exist at the base
/// reads as the empty text,
/// so every version in it counts as new.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function verifyRustLockfiles({ root, now, registry, readBaseLock }): Promise<string[]>
/// ```
pub fn verify_added_since_base(
    snapshots: &[LockSnapshot],
    read_base_lock: &dyn Fn(&str) -> String,
    settings: &Settings,
    registry: &dyn Registry,
    now: i64,
) -> Result<Vec<String>, String> {
    let mut base_sources: Vec<String> = Vec::new();
    for snapshot in snapshots {
        base_sources.push(read_base_lock(&snapshot.crate_dir));
    }
    let current: Vec<String> = snapshots.iter().map(LockSnapshot::source_text).collect();
    let versions: Vec<LockedVersion> = added_versions(&base_sources, &current)?;
    eprintln!("cargo-shadow: verifying {} versions newly locked since the base from {} lockfiles", versions.len(), snapshots.len());
    if versions.is_empty() {
        return Ok(Vec::new());
    }
    // Delegated pick (2026-11-03): the release age applies to the versions new since the base as well as the trust policy,
    // on this frozen path as on the local one.
    let mut problems: Vec<String> = check_release_age(&versions, settings, registry, now).map_err(message_of)?.problems;
    problems.extend(check_trust(&versions, settings, registry, now).map_err(message_of)?);
    return Ok(problems);
}
