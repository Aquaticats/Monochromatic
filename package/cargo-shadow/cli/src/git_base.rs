//! What:
//! Reads the lockfiles of a base revision with `git`, for the frozen install under `trustLockfile`.
//! Why:
//! The base is the pushed range's previous commit or a pull request's base (`doc/decision/rust-dependency-catalog.md`,
//! amended by the owner on 2026-10-29). CI names it in the environment variable below; without it the base is `HEAD`, so a
//! developer's uncommitted lockfile changes are the ones checked. The old TypeScript `rust:verify` took the same revision as
//! `--base`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // git-base.ts
//! export async function createGitBaseLockReader({ root, base }): Promise<(crateDir: string) => string>
//! ```

use std::path::Path;
use std::process::Command;

use crate::constants::LOCKFILE_NAME;

/// The environment variable that names the base revision.
pub const BASE_VARIABLE: &str = "CARGO_SHADOW_TRUST_BASE";

/// The base used when the variable is absent or empty.
pub const DEFAULT_BASE: &str = "HEAD";

/// The base revision from the environment.
pub fn base_from_environment() -> String {
    return match std::env::var(BASE_VARIABLE) {
        Ok(found) if !found.trim().is_empty() => found.trim().to_string(),
        _ => DEFAULT_BASE.to_string(),
    };
}

/// What:
/// `pub fn git_base_lock_reader(root: &Path, base: &str) -> Result<Box<dyn Fn(&str) -> String>, String>` resolves the base to a
/// commit once and returns a reader of `<commit>:<crate>/Cargo.lock`.
/// Why:
/// A base that is not a commit is an error naming the remedy (in CI, fetch it first); a lockfile absent at the base reads as the
/// empty string, so every version it holds counts as new.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function createGitBaseLockReader({ root, base }): Promise<ReadBaseLock>
/// ```
pub fn git_base_lock_reader(root: &Path, base: &str) -> Result<Box<dyn Fn(&str) -> String>, String> {
    let resolved = Command::new("git")
        .args(["rev-parse", "--verify", "--end-of-options", &format!("{base}^{{commit}}")])
        .current_dir(root)
        .output()
        .map_err(|error| { return format!("could not start git: {error}"); })?;
    if !resolved.status.success() {
        return Err(format!(
            "The verification base {base:?} is not a commit in this repository: {}\nSet {BASE_VARIABLE} to a revision that exists locally (in CI, fetch it first), or leave it unset to compare with HEAD.",
            String::from_utf8_lossy(&resolved.stderr).trim()
        ));
    }
    let commit: String = String::from_utf8_lossy(&resolved.stdout).trim().to_string();
    eprintln!("cargo-shadow: comparing lockfiles with {base} ({commit})");
    let root_path = root.to_path_buf();
    return Ok(Box::new(move |crate_dir: &str| -> String {
        let object: String = format!("{commit}:{crate_dir}/{LOCKFILE_NAME}");
        return match Command::new("git").args(["show", &object]).current_dir(&root_path).output() {
            Ok(shown) if shown.status.success() => format!("{}\n", String::from_utf8_lossy(&shown.stdout)),
            _ => String::new(),
        };
    }));
}
