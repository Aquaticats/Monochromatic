//! What:
//! How a launcher finds the program to run and what it says when it finds none:
//! the program its record names,
//! otherwise the program of the same name under the tool's `current` link,
//! and, when neither exists,
//! one sentence naming the tool,
//! the locked version,
//! the version `current` names
//! and the command that makes the tree's links again.
//! Also the two reads both need:
//! the version `current` names and the version a lock pins.
//! Why:
//! A release archive of this repository unpacks into a folder whose name carries the version
//! (`monochromatic-cargo-shadow-0.1.4-x86_64-unknown-linux-gnu`),
//! so a record written when `current` named 0.1.4 holds a path through that folder.
//! Installing 0.1.5 in any tree deletes the 0.1.4 folder (older repository tools are never kept),
//! and every other tree's launcher then pointed at a file that no longer exists and exited 127,
//! although `current` named a working program all along.
//! Following `current` at run time makes a launcher outlive any number of version moves
//! without a new `install` in its tree.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // launcher-current-link.ts
//! export function locked_version(lockText: string, toolId: string): string | undefined
//! export function version_that_runs(real: string): string | undefined
//! export function program_to_run(real: string): string
//! export function missing_program_advice(facts: MissingProgram): string
//! ```

/// Imports `OsStr` from `std::ffi`.
use std::ffi::OsStr;
/// Imports `fs` from `std`.
use std::fs;
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};

/// Imports `Value` from `serde_json`.
use serde_json::Value;

/// Imports `CURRENT_LINK_NAME` from `super::cargo_installs`.
use super::cargo_installs::CURRENT_LINK_NAME;
/// Imports `find_bin_under` from `super::layout`.
use super::layout::find_bin_under;
/// Imports `ScopeKind` from `super::scope`.
use super::scope::ScopeKind;
/// Imports `file_name_text` from `super::util`.
use super::util::file_name_text;

/// The version the lock pins for the tool,
/// read from the lock's text.
pub fn locked_version(lock_text: &str, tool_id: &str) -> Option<String> {
    let lock: Value = serde_json::from_str(lock_text).ok()?;
    return lock.get("tools")?.get(tool_id)?.get("version")?.as_str().map(str::to_string);
}

/// Whether a path is a link named `current`,
/// the link of a repository-published tool
/// (an archive folder that happens to be called `current` is a directory and does not count).
fn is_current_link(candidate: &&Path) -> bool {
    return candidate.file_name() == Some(OsStr::new(CURRENT_LINK_NAME)) && candidate.is_symlink();
}

/// The `current` link above a program's path:
/// the nearest ancestor that is a link named `current`.
fn current_link_above(real: &Path) -> Option<&Path> {
    return real.ancestors().skip(1).find(is_current_link);
}

/// The version the tool's `current` link names,
/// found from the real program's path:
/// `<crate directory>/current/bin/<name>` for a crate,
/// `<tool directory>/current/<archive folder>/<name>` for a release archive,
/// so the link is the nearest ancestor of the program that is named `current`.
pub fn version_that_runs(real: &Path) -> Option<String> {
    let link: &Path = current_link_above(real)?;
    // The final directory's name, so a `current` that forwards to the other toolchain root's `current` still names the
    // version that runs; a link that does not resolve falls back to its own target.
    let target: PathBuf = match fs::canonicalize(link) {
        Ok(resolved) => resolved,
        Err(_) => fs::read_link(link).ok()?,
    };
    let name: String = file_name_text(&target);
    if name.is_empty() {
        return None;
    }
    return Some(name);
}

/// What:
/// The program a launcher runs:
/// `real` when it exists,
/// otherwise the program of the same file name under the `current` link above `real`,
/// otherwise `real` again (the launcher then reports that it could not start it).
/// Why:
/// The folder below `current` of a release archive carries the version,
/// so the path a record names stops existing when `current` moves,
/// while a program of the same name still sits under `current`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function programToRun(real: string): string
/// ```
pub fn program_to_run(real: &Path) -> PathBuf {
    if real.exists() {
        return real.to_path_buf();
    }
    let Some(link) = current_link_above(real) else {
        return real.to_path_buf();
    };
    let Some(name) = real.file_name() else {
        return real.to_path_buf();
    };
    return find_bin_under(link, &name.to_string_lossy()).unwrap_or_else(|| return real.to_path_buf());
}

/// What a launcher knows when it found no program to start.
#[derive(Clone, Copy, Debug)]
pub struct MissingProgram<'facts> {
    /// The tool's id in the lock.
    pub tool_id: &'facts str,
    /// Which scope made the launcher.
    pub scope: ScopeKind,
    /// The lock file the record names.
    pub lock_file: &'facts Path,
    /// The program the record names.
    pub real: &'facts Path,
    /// The lock file's text, when it could be read.
    pub lock_text: Option<&'facts str>,
}

/// What:
/// One sentence for a launcher that found no program:
/// the tool,
/// the version the lock pins,
/// the version `current` names (each only when it could be read),
/// that no program of that name exists,
/// and the one command that makes the links again.
/// Why:
/// `install` rebuilds the exposure directory and every launcher record from the lock,
/// so it is the one command that fixes a tree whose launcher cannot find its program;
/// the machine-global scope has no checkout to run it in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function missingProgramAdvice(facts: MissingProgram): string
/// ```
pub fn missing_program_advice(facts: MissingProgram<'_>) -> String {
    let name: String = file_name_text(facts.real);
    let locked_pin: Option<String> = facts.lock_text.and_then(|text| return locked_version(text, facts.tool_id));
    let current_version: Option<String> = version_that_runs(facts.real);
    let where_locked: &str = match facts.scope {
        ScopeKind::Repo => "this tree",
        ScopeKind::Global => "this machine's global lock",
    };
    let versions: String = match (locked_pin, current_version) {
        (Some(locked), Some(used)) => format!("{} is locked at {locked} in {where_locked} and its `current` link names {used}, but", facts.tool_id),
        (Some(locked), None) => format!("{} is locked at {locked} in {where_locked} and has no readable `current` link, so", facts.tool_id),
        (None, Some(used)) => format!("{}'s `current` link names {used}, but", facts.tool_id),
        (None, None) => format!("{} has no readable `current` link, so", facts.tool_id),
    };
    let command: String = match facts.scope {
        ScopeKind::Repo => {
            let tree: PathBuf = facts.lock_file.parent().map(Path::to_path_buf).unwrap_or_default();
            format!("run `mpm install` in {} to make this tree's links again", tree.display())
        }
        ScopeKind::Global => "run `mpm install -g` to make the machine-global links again".to_string(),
    };
    return format!("{versions} no program named {name} was found under `current` or at the recorded path; {command}");
}
