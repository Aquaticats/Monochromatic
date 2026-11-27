//! What:
//! The launcher file itself:
//! the hard link (or copy) of the newest installed `mpm-shadow` that a tree's exposed links point at,
//! placed with a rename,
//! and the record written beside it first.
//! Why:
//! The source may be a symbolic link:
//! a release archive's install leaves a link beside the folder it unpacked into,
//! and a hard link of a symbolic link is another symbolic link,
//! whose relative target then resolves beside the launcher instead of beside the version directory.
//! So the file the link resolves to is what a launcher must be,
//! and the record is written before the launcher exists,
//! so a launcher never exists without one.
//! A launcher folder under a repository whose link to the 860 drive dangles is never made:
//! the plain message of the repository link check comes before any write.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function isSameFile(first: string, second: string): boolean
//! function placeBinary(source: string, target: string): void  // throws
//! export function ensureLauncher(scope: Scope, toolId: string, name: string, real: string): string
//! ```

/// Imports `OsString` from `std::ffi`.
use std::ffi::OsString;
/// Imports `fs` from `std`.
use std::fs;
/// Imports what this module uses from `std::path`.
use std::path::{
    Path,
    PathBuf,
};

/// Imports `ToolchainError` from `super::super::error`.
use super::super::error::ToolchainError;
/// Imports what this module uses from `super::super::launcher_source`.
use super::super::launcher_source::{
    launcher_source,
    running_program,
};
/// Imports what this module uses from `super::super::repository_link_cause`.
use super::super::repository_link_cause::{
    CHECK_DEADLINE,
    LinkVerdict,
    repository_link_verdict,
};
/// Imports `Scope` from `super::super::scope`.
use super::super::scope::Scope;
/// Imports the one atomic write the launcher and the lock share.
use super::super::util::write_atomically;
/// Imports what this module uses from the parent.
use super::{
    LaunchRecord,
    record_path,
    render_record,
};

/// Whether both paths name the same file (the same device and inode),
/// which a rename between two links of one file would silently leave as two names.
#[cfg(unix)]
fn is_same_file(
    first: &Path,
    second: &Path,
) -> bool {
    /// Imports `MetadataExt` from `std::os::unix::fs`.
    use std::os::unix::fs::MetadataExt;
    return match (fs::metadata(first), fs::metadata(second)) {
        (Ok(left), Ok(right)) => left.dev() == right.dev() && left.ino() == right.ino(),
        _ => false,
    };
}

/// Without inodes every placement replaces the target.
#[cfg(not(unix))]
fn is_same_file(
    _first: &Path,
    _second: &Path,
) -> bool {
    return false;
}

/// What:
/// A hard link of `source` at `target` (a copy when the two are on different file systems),
/// placed with a rename.
/// Why:
/// `source` may be a symbolic link:
/// a release archive's install leaves a link beside the folder it unpacked into,
/// and the search for the newest installed `mpm-shadow` finds the program through `current`.
/// A hard link of a symbolic link is another symbolic link,
/// and its relative target then resolves beside the launcher instead of beside the version directory,
/// so the launcher dangled and the tool it was made for stopped starting.
/// The file the link resolves to is what a launcher must be.
fn place_binary(
    source: &Path,
    target: &Path,
) -> Result<(), ToolchainError> {
    let mut temporary_name: OsString = OsString::from(".");
    temporary_name.push(target.file_name().unwrap_or_default());
    temporary_name.push(format!(".{}.bin", std::process::id()));
    let temporary: PathBuf = target.with_file_name(temporary_name);
    // A link on the way to the program is followed: the launcher must be the program's own file.
    let file: PathBuf = match fs::canonicalize(source) {
        Ok(resolved) => resolved,
        Err(_) => source.to_path_buf(),
    };
    if is_same_file(&file, target) {
        return Ok(());
    }
    // Best effort: a leftover from a killed run is replaced; absence is the normal case.
    let _ = fs::remove_file(&temporary);
    if fs::hard_link(&file, &temporary).is_err()
        && let Err(error) = fs::copy(&file, &temporary)
    {
        return Err(ToolchainError::failure(format!(
            "cannot copy {} to {}: {error}",
            source.display(),
            temporary.display()
        )));
    }
    if let Err(error) = fs::rename(&temporary, target) {
        return Err(ToolchainError::failure(format!(
            "cannot place {}: {error}",
            target.display()
        )));
    }
    return Ok(());
}

/// What:
/// Makes the launcher for binary `name` of a repository-published tool in this tree and returns its path.
/// The record is written first,
/// so a launcher never exists without one.
/// Why:
/// The tree's exposed links and postinstall links point here instead of at the real binary,
/// and the launcher is a copy of the newest installed `mpm-shadow`
/// (`launcher_source`),
/// never of whichever copy happened to run `install`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function ensureLauncher(scope: Scope, tool: LockedTool, name: string, real: string): string
/// ```
pub fn ensure_launcher(
    scope: &Scope,
    tool_id: &str,
    name: &str,
    real: &Path,
) -> Result<PathBuf, ToolchainError> {
    // A launcher folder under a repository whose link to the 860 drive does not resolve is never made:
    // the plain message of the repository link check comes before any write.
    if let LinkVerdict::NotReady {
        message,
    } = repository_link_verdict(&scope.home, &scope.launcher_dir, CHECK_DEADLINE)
    {
        return Err(ToolchainError::failure(message));
    }
    if let Err(error) = fs::create_dir_all(&scope.launcher_dir) {
        return Err(ToolchainError::failure(format!(
            "cannot create {}: {error}",
            scope.launcher_dir.display()
        )));
    }
    let launcher: PathBuf = scope.launcher_dir.join(name);
    let record: LaunchRecord = LaunchRecord {
        tool_id: tool_id.to_string(),
        lock_file: scope.lock_file.clone(),
        real: real.to_path_buf(),
        scope: scope.kind,
    };
    write_atomically(&record_path(&launcher), render_record(&record).as_bytes())?;
    let source: PathBuf = launcher_source(scope, running_program()?, cfg!(target_os = "linux"))?;
    place_binary(&source, &launcher)?;
    return Ok(launcher);
}
