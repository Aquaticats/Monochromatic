//! What:
//! Every crate and version directory under one toolchain root,
//! read from the file system,
//! and the refusal a directory the walk cannot read raises.
//! Why:
//! The prune deletes the versions no lock names,
//! and the scan it runs after those deletions decides whether a crate directory still holds one,
//! so a directory this walk cannot read must be reported and not left out:
//! left out,
//! it reads as "this crate holds nothing" and the whole crate directory goes,
//! including a version a lock names.
//! A caller imports from `prune_cargo` and never from this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function presentVersions(toolchainRoot: string): Map<string, PresentVersion[]>;
//! function cannotWalk(directory: string, error: Error): Error;
//! ```

/// Imports the sorted collection the scan answers with.
use std::collections::BTreeMap;
/// Imports `fs` from `std`.
use std::fs;
/// Imports `ErrorKind` from `std::io`.
use std::io::ErrorKind;
/// Imports the path types the scan names its directories with.
use std::path::{
    Path,
    PathBuf,
};

/// Imports the namespace and the link name a version directory is told apart by,
/// and the check that an install finished.
use super::super::cargo_installs::{
    CARGO_NAMESPACE,
    CURRENT_LINK_NAME,
    recorded_version,
};
/// Imports `ToolchainError` from `super::super::error`.
use super::super::error::ToolchainError;

/// Imports one version directory the scan found.
use super::the_rule_that_decides_what_goes::PresentVersion;
/// What:
/// Every crate and version directory under `<toolchain root>/cargo`,
/// or an error naming the first directory whose contents could not be read.
/// Why:
/// The prune deletes the versions no lock names,
/// and the scan it runs after those deletions decides whether a crate directory still holds one.
/// A directory this walk cannot read must therefore be reported and not left out:
/// left out,
/// it reads as "this crate holds nothing" and the whole crate directory goes,
/// including a version a lock names.
/// A crate directory that is gone holds nothing,
/// so its absence is not a failed read.
pub(crate) fn present_versions(
    toolchain_root: &Path
) -> Result<BTreeMap<String, Vec<PresentVersion>>, ToolchainError> {
    let mut present: BTreeMap<String, Vec<PresentVersion>> = BTreeMap::new();
    let crates_dir: PathBuf = toolchain_root.join(CARGO_NAMESPACE);
    let crates = match fs::read_dir(&crates_dir) {
        Ok(entries) => entries,
        // No cargo directory yet: nothing is installed under it, so nothing can be deleted from it.
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(present),
        Err(error) => return Err(cannot_walk(&crates_dir, &error)),
    };
    for listed_crate in crates {
        let crate_entry: fs::DirEntry =
            listed_crate.map_err(|error| return cannot_walk(&crates_dir, &error))?;
        let crate_name: String = crate_entry.file_name().to_string_lossy().to_string();
        let versions = match fs::read_dir(crate_entry.path()) {
            Ok(entries) => entries,
            // The crate directory is gone: it holds no version that could be deleted.
            Err(error) if error.kind() == ErrorKind::NotFound => continue,
            Err(error) => return Err(cannot_walk(&crate_entry.path(), &error)),
        };
        let mut found: Vec<PresentVersion> = Vec::new();
        for listed_version in versions {
            let version_entry: fs::DirEntry =
                listed_version.map_err(|error| return cannot_walk(&crate_entry.path(), &error))?;
            let version: String = version_entry.file_name().to_string_lossy().to_string();
            let is_plain_directory: bool = version_entry
                .file_type()
                .map_err(|error| return cannot_walk(&version_entry.path(), &error))?
                .is_dir();
            if is_plain_directory && !version.starts_with('.') && version != CURRENT_LINK_NAME {
                let finished: bool = recorded_version(&version_entry.path(), &crate_name).is_some();
                found.push(PresentVersion {
                    version,
                    finished,
                });
            }
        }
        present.insert(crate_name, found);
    }
    return Ok(present);
}

/// The refusal a scan raises for one directory whose contents it could not read.
fn cannot_walk(
    directory: &Path,
    error: &std::io::Error,
) -> ToolchainError {
    return ToolchainError::failure(format!("cannot read {}: {error}", directory.display()));
}
