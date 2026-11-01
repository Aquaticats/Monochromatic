//! What:
//! How an archive install becomes visible:
//! the archive is unpacked into a hidden staging directory beside the version directory,
//! a completion marker is written there last,
//! and only then is the staging directory renamed to the version's name.
//! Why:
//! `tar --extract` creates a file with its final mode before it writes the content,
//! so an interrupted unpacking (a truncated archive,
//! a killed process,
//! a full disk) leaves an executable file of the right name with the wrong bytes.
//! If that sat at the version's name,
//! the install would count as finished,
//! the working older version would be deleted as superseded,
//! and `current` would point at the broken copy.
//! Staging keeps a failed unpacking away from the version's name and from any existing copy;
//! the marker is what tells a directory made by this scheme from one an earlier release unpacked in place
//! (which cannot be told from a broken one),
//! so such a directory is never the reason to delete anything and a reinstall replaces it.
//!
//! A run killed without cleanup leaves its hidden staging directory behind
//! (`.<version>.staging.<process id>` in the tool's directory);
//! nothing reads it,
//! and it can be deleted by hand.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // archive-staging.ts
//! export function stagingDirOf(versionDir: string): string
//! export function hasCompleteMarker(dir: string): boolean
//! export function writeCompleteMarker(dir: string, toolId: string, version: string): void
//! export function publishStaged(staging: string, finalDir: string): void
//! export function discardStaging(staging: string): void
//! ```

/// Imports `fs` from `std`.
use std::fs;
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};

/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports `is_finished` from `super::published_archive`.
use super::published_archive::is_finished;
/// Imports `file_name_text` from `super::util`.
use super::util::file_name_text;

/// The file whose presence shows an install finished;
/// it is the last file written before the version directory takes its name.
pub const COMPLETE_MARKER: &str = ".mpm-shadow-installed";

/// `<parent>/.<name>.<purpose>.<process id>`,
/// a hidden sibling of `version_dir` that no version scan lists.
fn hidden_sibling(version_dir: &Path, purpose: &str) -> PathBuf {
    let name: String = format!(".{}.{purpose}.{}", file_name_text(version_dir), std::process::id());
    return version_dir.with_file_name(name);
}

/// The staging directory for the install that will become `version_dir`.
pub fn staging_dir_of(version_dir: &Path) -> PathBuf {
    return hidden_sibling(version_dir, "staging");
}

/// Whether `dir` holds the completion marker as a regular file
/// (a link of that name does not count).
pub fn has_complete_marker(dir: &Path) -> bool {
    return match fs::symlink_metadata(dir.join(COMPLETE_MARKER)) {
        Ok(metadata) => metadata.is_file(),
        Err(_) => false,
    };
}

/// Writes the completion marker into `dir`;
/// call it after everything else in the install has been checked.
pub fn write_complete_marker(dir: &Path, tool_id: &str, version: &str) -> Result<(), ToolchainError> {
    let marker: PathBuf = dir.join(COMPLETE_MARKER);
    if let Err(error) = fs::write(&marker, format!("{tool_id} {version}\n")) {
        return Err(ToolchainError::failure(format!("cannot write {}: {error}", marker.display())));
    }
    return Ok(());
}

/// Removes a path the installer made and no longer needs,
/// whatever it turned out to be;
/// a failure is ignored because the caller is already on its way out,
/// and the leftover is hidden and harmless.
pub fn discard_path(path: &Path) {
    if fs::remove_dir_all(path).is_err() {
        // Best effort: the path may be a file or a link, or already gone.
        let _ = fs::remove_file(path);
    }
}

/// What:
/// Makes the staged install the version directory.
/// A copy already at the version's name that is finished
/// (it carries the marker and still holds `first_bin` as an executable file,
/// as `is_finished` checks)
/// is kept,
/// because another run installed the same verified release meanwhile,
/// and the staging directory is discarded;
/// any other copy is moved aside first,
/// so a failed rename puts it back,
/// and removed once the new one is in place.
/// Why:
/// The rename is atomic,
/// so a reader sees no version directory or a whole one,
/// and a copy that was working is never deleted before its replacement is in place.
/// A marker alone is not enough to keep the old copy:
/// a person who deleted or made non-executable the program in a marked directory
/// would otherwise see every later install discard the good copy and report success.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function publishStaged(staging: string, finalDir: string, firstBin: string): void  // throws
/// ```
pub fn publish_staged(staging: &Path, final_dir: &Path, first_bin: &str) -> Result<(), ToolchainError> {
    if is_finished(final_dir, first_bin) {
        discard_path(staging);
        return Ok(());
    }
    let had_copy: bool = fs::symlink_metadata(final_dir).is_ok();
    let aside: PathBuf = hidden_sibling(final_dir, "replaced");
    if had_copy {
        // A leftover of a killed run with the same process id would block the rename.
        discard_path(&aside);
        if let Err(error) = fs::rename(final_dir, &aside) {
            discard_path(staging);
            return Err(ToolchainError::failure(format!(
                "cannot move the copy at {} aside to install over it: {error}",
                final_dir.display()
            )));
        }
    }
    if let Err(error) = fs::rename(staging, final_dir) {
        if had_copy && let Err(restore_error) = fs::rename(&aside, final_dir) {
            discard_path(staging);
            return Err(ToolchainError::failure(format!(
                "cannot move the finished install to {}: {error}; the copy that was there is at {} and could not be put back: {restore_error}",
                final_dir.display(),
                aside.display()
            )));
        }
        discard_path(staging);
        return Err(ToolchainError::failure(format!("cannot move the finished install to {}: {error}", final_dir.display())));
    }
    if had_copy {
        discard_path(&aside);
    }
    return Ok(());
}
