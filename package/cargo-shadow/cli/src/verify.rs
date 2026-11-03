//! What:
//! Verification of newly locked registry versions against the trust policy: the check covers only versions that no lockfile
//! held at a base revision.
//! Why:
//! This is the counterpart of pnpm's `trustLockfile: true` (https://pnpm.io/settings/store, read 2026-11-03): the owner never
//! accepts pull requests that change a lockfile, so every committed lockfile comes from the owner's own sessions and is part
//! of the trusted base. Release age is a resolution-time rule here, so locked versions are not re-gated on it. A crate whose
//! lockfile did not exist at the base counts as new in full.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // verify.ts
//! export async function verifyRustLockfiles({ root, now, registry, readBaseLock }): Promise<string[]>
//! ```

use std::path::Path;
use std::process::Command;

use monochromatic_dependency_catalog::Settings;

use crate::checks::check_trust;
use crate::constants::LOCKFILE_NAME;
use crate::crates::{LockSnapshot, locked_crate_dirs, read_locks};
use crate::lockfile::{LockedVersion, added_versions};
use crate::registry::Registry;

/// What:
/// `pub type ReadBaseLock = dyn Fn(&str) -> String` reads one crate's lockfile text at the base revision, or the empty string
/// when the lockfile did not exist there.
/// Why:
/// Tests give it canned texts; the command gives it `git show`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReadBaseLock = (crateDir: string) => Promise<string>;
/// ```
pub type ReadBaseLock<'a> = &'a dyn Fn(&str) -> String;

/// The trust problems of the registry versions that no lockfile held at the base revision.
pub fn verify_rust_lockfiles(
    root: &Path,
    now: i64,
    settings: &Settings,
    registry: &dyn Registry,
    read_base_lock: ReadBaseLock,
) -> Result<Vec<String>, String> {
    let crate_dirs: Vec<String> = locked_crate_dirs(root).map_err(|error| { return error.to_string(); })?;
    let locks: Vec<LockSnapshot> = read_locks(root, &crate_dirs).map_err(|error| { return error.to_string(); })?;
    let base_sources: Vec<String> = locks.iter().map(|snapshot| { return read_base_lock(&snapshot.crate_dir); }).collect();
    let current: Vec<String> = locks.iter().map(|snapshot| { return snapshot.source.clone(); }).collect();
    let versions: Vec<LockedVersion> = added_versions(&base_sources, &current)?;
    eprintln!("cargo-shadow: verifying {} newly locked versions from {} lockfiles", versions.len(), locks.len());
    if versions.is_empty() {
        return Ok(Vec::new());
    }
    return check_trust(&versions, settings, registry, now).map_err(|error| { return error.message; });
}

/// What:
/// `pub fn git_base_lock_reader(root: &Path, base: &str) -> Result<impl Fn(&str) -> String, String>` resolves the base to a
/// commit once and returns a reader of `<commit>:<crate>/Cargo.lock`.
/// Why:
/// A base that is not a commit is an error naming the remedy (in CI, fetch it first); a lockfile absent at the base reads as
/// the empty string, so every version it holds counts as new.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function createGitBaseLockReader({ root, base }): Promise<ReadBaseLock>
/// ```
pub fn git_base_lock_reader(root: &Path, base: &str) -> Result<impl Fn(&str) -> String, String> {
    let resolved = Command::new("git")
        .args(["rev-parse", "--verify", "--end-of-options", &format!("{base}^{{commit}}")])
        .current_dir(root)
        .output()
        .map_err(|error| { return format!("could not start git: {error}"); })?;
    if !resolved.status.success() {
        return Err(format!(
            "The verification base {base:?} is not a commit in this repository: {}\nPass --base with a revision that exists locally (in CI, fetch it first), or omit --base to compare with HEAD.",
            String::from_utf8_lossy(&resolved.stderr).trim()
        ));
    }
    let commit: String = String::from_utf8_lossy(&resolved.stdout).trim().to_string();
    eprintln!("cargo-shadow: comparing lockfiles with {base} ({commit})");
    let root_path = root.to_path_buf();
    return Ok(move |crate_dir: &str| -> String {
        let object: String = format!("{commit}:{crate_dir}/{LOCKFILE_NAME}");
        return match Command::new("git").args(["show", &object]).current_dir(&root_path).output() {
            Ok(shown) if shown.status.success() => format!("{}\n", String::from_utf8_lossy(&shown.stdout)),
            _ => String::new(),
        };
    });
}
