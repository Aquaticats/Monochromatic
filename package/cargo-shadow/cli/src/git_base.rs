//! What:
//! Reads the lockfiles of a base revision with `git`,
//! for the frozen install under `trustLockfile`.
//! Why:
//! The base is the pushed range's previous commit or a pull request's base (`doc/decision/rust-dependency-catalog.md`,
//! amended by the owner on 2026-10-29).
//! CI names it in the environment variable below;
//! without it the base is `HEAD`,
//! so a
//! developer's uncommitted lockfile changes are the ones checked.
//! The old TypeScript `rust:verify` took the same revision as
//! `--base`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // git-base.ts
//! export async function createGitBaseLockReader({ root, base }): Promise<(crateDir: string) => string>
//! ```

/// What:
/// `use std::path::Path;` imports `Path` from the standard library so this file can name it unqualified.
/// Why:
/// `Path` is used by `git_base_lock_reader`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Path } from 'std/path';
/// ```
use std::path::Path;
/// What:
/// `use std::process::Command;` imports `Command` from the standard library so this file can name it unqualified.
/// Why:
/// `Command` is used by `git_base_lock_reader`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Command } from 'std/process';
/// ```
use std::process::Command;

/// What:
/// `use crate::constants::LOCKFILE_NAME;` imports `LOCKFILE_NAME` from another module of this crate so this file can name it unqualified.
/// Why:
/// `LOCKFILE_NAME` is needed by the derive or macro expansions of this file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LOCKFILE_NAME } from 'crate/constants';
/// ```
use crate::constants::LOCKFILE_NAME;

/// The environment variable that names the base revision.
pub const BASE_VARIABLE: &str = "CARGO_SHADOW_TRUST_BASE";

/// The base used when the variable is absent or empty.
pub const DEFAULT_BASE: &str = "HEAD";

/// The upstream branch of the current branch, the first base a local run tries.
const UPSTREAM_BASE: &str = "@{upstream}";

/// The remote default branches a local run tries when the current branch has no upstream.
const DEFAULT_REMOTE_BASES: [&str; 2] = ["origin/main", "origin/master"];

/// What:
/// `fn git_start_failure(error: std::io::Error) -> String` words the failure to start `git`.
/// Why:
/// The base lock reader runs `git`,
/// and the user needs to know when it cannot even start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gitStartFailure = (error: Error) => `could not start git: ${error.message}`;
/// ```
fn git_start_failure(error: std::io::Error) -> String {
    return format!("could not start git: {error}");
}

/// How a trust base was chosen and what it resolved to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustBase {
    /// How the base is spelled (`CARGO_SHADOW_TRUST_BASE`, `@{upstream}`, `origin/main`, `HEAD`).
    pub spelling: String,
    /// The commit it names, or why it names none.
    pub commit: Result<String, String>,
}

/// What:
/// `fn resolve_commit(root, base) -> Result<String, String>` is the commit a revision names in the repository at `root`.
/// Why:
/// A remembered pass is keyed by the commit, not by how the base is spelled, so a base that moves is a different base.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveCommit(root: string, base: string): string  // git rev-parse --verify base^{commit}
/// ```
fn resolve_commit(root: &Path, base: &str) -> Result<String, String> {
    let resolved = Command::new("git")
        .args(["rev-parse", "--verify", "--quiet", "--end-of-options", &format!("{base}^{{commit}}")])
        .current_dir(root)
        .output()
        .map_err(git_start_failure)?;
    if !resolved.status.success() {
        return Err(format!(
            "The verification base {base:?} is not a commit in this repository.\nSet {BASE_VARIABLE} to a revision that exists locally (in CI, fetch it first), or leave it unset to compare with the upstream branch (HEAD in CI)."
        ));
    }
    return Ok(String::from_utf8_lossy(&resolved.stdout).trim().to_string());
}

/// What:
/// `pub fn resolve_trust_base(root, in_ci, explicit) -> TrustBase` chooses the revision whose lockfiles count as trusted.
/// Why:
/// An explicit `CARGO_SHADOW_TRUST_BASE` wins; otherwise CI compares with `HEAD` and a local run with `@{upstream}`,
/// else `origin/main`, else `origin/master`, else `HEAD`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function resolveTrustBase(root: string, inCi: boolean, explicit?: string): TrustBase
/// ```
pub fn resolve_trust_base(root: &Path, in_ci: bool, explicit: Option<&str>) -> TrustBase {
    // An explicit base wins everywhere.
    if let Some(found) = explicit
        && !found.trim().is_empty()
    {
        let spelling: String = found.trim().to_string();
        let commit: Result<String, String> = resolve_commit(root, &spelling);
        return TrustBase { spelling, commit };
    }
    // CI keeps its base.
    if in_ci {
        let commit: Result<String, String> = resolve_commit(root, DEFAULT_BASE);
        return TrustBase { spelling: DEFAULT_BASE.to_string(), commit };
    }
    // Delegated pick (2026-11-03): a local run compares with what is already on the upstream branch, so a lockfile that arrived
    // with a commit not pushed yet is new since the base; the first candidate that names a commit wins.
    for candidate in [UPSTREAM_BASE, DEFAULT_REMOTE_BASES[0], DEFAULT_REMOTE_BASES[1]] {
        if let Ok(commit) = resolve_commit(root, candidate) {
            return TrustBase { spelling: candidate.to_string(), commit: Ok(commit) };
        }
    }
    let commit: Result<String, String> = resolve_commit(root, DEFAULT_BASE);
    return TrustBase { spelling: DEFAULT_BASE.to_string(), commit };
}

/// What:
/// `pub fn git_lock_reader_at(root, commit) -> LockReader` reads one crate root's lockfile text at a commit
/// (empty when the lockfile did not exist there).
/// Why:
/// The base is resolved once, so every read names the same commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function gitLockReaderAt(root: string, commit: string): (crateDir: string) => string
/// ```
pub fn git_lock_reader_at(root: &Path, commit: &str) -> Box<dyn Fn(&str) -> String> {
    let root_path = root.to_path_buf();
    let commit_text: String = commit.to_string();
    return Box::new(move |crate_dir: &str| -> String {
        let object: String = format!("{commit_text}:{crate_dir}/{LOCKFILE_NAME}");
        return match Command::new("git").args(["show", &object]).current_dir(&root_path).output() {
            Ok(shown) if shown.status.success() => format!("{}\n", String::from_utf8_lossy(&shown.stdout)),
            _ => String::new(),
        };
    });
}

/// What:
/// `pub fn explicit_base_from_environment() -> Option<String>` is the base named by `CARGO_SHADOW_TRUST_BASE`, when it is set
/// and not blank.
/// Why:
/// CI sets it to the pushed range's previous commit or the pull request's base; it wins over every default.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const explicitBaseFromEnvironment = (): string | undefined => process.env.CARGO_SHADOW_TRUST_BASE?.trim() || undefined;
/// ```
pub fn explicit_base_from_environment() -> Option<String> {
    return match std::env::var(BASE_VARIABLE) {
        Ok(found) if !found.trim().is_empty() => Some(found.trim().to_string()),
        _ => None,
    };
}
