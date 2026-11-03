//! What:
//! The check a frozen install makes of every registry version a lockfile already holds, against the release-age and trust
//! policy.
//! Why:
//! pnpm 12.16.1 re-validates every entry of the lockfile on install, frozen or not, against the policies its resolvers were built
//! with, unless `trustLockfile` is true (`pnpm11/installing/deps-installer/src/install/mutateModules.ts:314-343`,
//! `startLockfileVerification`: "Re-validate every entry in the lockfile against the policies the resolver chain was built
//! with"; skipped when `opts.trustLockfile`). A passing result is cached by a hash of the lockfile
//! (`pnpm11/installing/deps-installer/src/install/verifyLockfileResolutionsCache.ts`). This is that check for `Cargo.lock`: the
//! default, no separate command. The release-age rule needs the sparse index's publish time per crate; the trust rule needs the
//! crate's history, which the database dump fills in bulk. A passing lockfile is remembered under
//! `.cache/cargo-shadow/verified/`, keyed by a SHA-256 of the lockfile text and the policy settings, so an unchanged lockfile costs
//! nothing the next time; a failing one is never remembered. There is no `trustLockfile` setting here: the catalog's settings
//! library has no such key, and the owner has not asked for one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // verify.ts
//! export async function verifyLockedEntries({ root, snapshots, settings, registry, now }): Promise<string[]>
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use monochromatic_dependency_catalog::Settings;
use sha2::{Digest, Sha256};

use crate::checks::{check_release_age, check_trust};
use crate::crates::LockSnapshot;
use crate::lockfile::{LockedVersion, registry_versions};
use crate::registry::Registry;

/// Where passing digests are remembered, relative to the repository root.
pub const VERIFIED_DIR: &str = ".cache/cargo-shadow/verified";

/// The lowercase hex of a SHA-256 digest, written per byte (the digest type has no `LowerHex` in sha2 0.11).
fn hex(digest: &[u8]) -> String {
    let mut text: String = String::new();
    for byte in digest {
        text.push_str(&format!("{byte:02x}"));
    }
    return text;
}

/// The cache key of one lockfile under one policy: a digest of the text and of every setting.
pub fn cache_key(snapshot: &LockSnapshot, settings: &Settings) -> String {
    let mut hasher: Sha256 = Sha256::new();
    hasher.update(snapshot.source.as_bytes());
    hasher.update(format!("{settings:?}").as_bytes());
    return hex(hasher.finalize().as_slice());
}

fn cache_file(root: &Path, snapshot: &LockSnapshot, settings: &Settings) -> PathBuf {
    return root.join(VERIFIED_DIR).join(format!("{}.{}.ok", snapshot.crate_dir.replace('/', "__"), cache_key(snapshot, settings)));
}

/// What:
/// `pub fn verify_locked_entries(...) -> Result<Vec<String>, String>` returns the findings against every registry version the
/// given lockfiles hold, skipping a lockfile whose digest passed before.
/// Why:
/// The findings are the release-age problems (strict) and the trust problems; the lockfiles that pass are remembered.
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
        if !cache_file(root, snapshot, settings).exists() {
            pending.push(snapshot);
        }
    }
    if pending.is_empty() {
        return Ok(Vec::new());
    }
    let sources: Vec<String> = pending.iter().map(|snapshot| { return snapshot.source.clone(); }).collect();
    let versions: Vec<LockedVersion> = registry_versions(&sources)?;
    eprintln!("cargo-shadow: verifying {} locked versions from {} lockfiles", versions.len(), pending.len());
    let mut problems: Vec<String> = check_release_age(&versions, settings, registry, now).map_err(|error| { return error.message; })?.problems;
    problems.extend(check_trust(&versions, settings, registry, now).map_err(|error| { return error.message; })?);
    if problems.is_empty() {
        for snapshot in pending {
            let path: PathBuf = cache_file(root, snapshot, settings);
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(path, "");
        }
    }
    return Ok(problems);
}

/// Stub for the red run.
pub fn verify_added_since_base(
    _snapshots: &[LockSnapshot],
    _read_base_lock: &dyn Fn(&str) -> String,
    _settings: &Settings,
    _registry: &dyn Registry,
    _now: i64,
) -> Result<Vec<String>, String> {
    return Ok(Vec::new());
}
