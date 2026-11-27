//! What:
//! Where an archive-route tool that this repository publishes lives and how its version is chosen:
//! one directory per version under the tool's own directory,
//! `<toolchain root>/<tool id>/<version>`,
//! and a `current` link inside the tool's directory that names the newest finished version.
//! Why:
//! The owner's rule covers every repository-published tool
//! (a release archive of `github:Aquaticats/Monochromatic` with a `monochromatic-` tag prefix;
//! a tool of any other repository is never judged by it),
//! whatever its route:
//! "No older repo-maintained tools should ever exist when newer is present."
//! `mpm-shadow` itself is such a tool (its plan entry is a release archive of this repository,
//! so that the lock records
//! the SHA-256 a fresh machine checks before it runs the first `mpm-shadow`),
//! and it gets the same treatment as a `monochromatic-` crate on the `cargo-binstall` route:
//! the newest version only,
//! a `current` link,
//! a launcher that warns on every run,
//! and warnings from every command.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // published-archive.ts
//! export function isPublishedArchive(tool: LockedTool): boolean
//! export function newestInstalledArchive(root: string, id: string, firstBin: string): string | undefined
//! export function pointCurrentAtNewestArchive(root: string, id: string, firstBin: string): string | undefined
//! export function removeOlderArchives(root: string, id: string, firstBin: string): string[]
//! ```

/// Imports `Ordering` from `std::cmp`.
use std::cmp::Ordering;
/// Imports `fs` from `std`.
use std::fs;
/// Imports `ErrorKind` from `std::io`.
use std::io::ErrorKind;
/// Imports what this module uses from `std::path`.
use std::path::{
    Path,
    PathBuf,
};

/// Imports `compare_versions` from `crate::version`.
use crate::version::compare_versions;

/// Imports `has_complete_marker` from `super::archive_staging`.
use super::archive_staging::has_complete_marker;
/// Imports `CURRENT_LINK_NAME` from `super::cargo_installs`.
use super::cargo_installs::CURRENT_LINK_NAME;
/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports `atomic_symlink` from `super::install_support`.
use super::install_support::atomic_symlink;
/// Imports what this module uses from `super::layout`.
use super::layout::{
    find_bin_under,
    tool_directory,
    tool_downloads_dir,
    tool_install_dir,
};
/// Imports `LockedTool` from `super::model`.
use super::model::LockedTool;
/// Imports `file_name_text` from `super::util`.
use super::util::file_name_text;

/// The archive tools one lock names,
/// their present versions and the prune's plan.
mod what_a_lock_names;
/// What counts as a release archive of this repository.
mod what_counts_as_a_release_of_this_repository;

/// Re-exports the read of every archive a lock names,
/// which the cleanup of hidden installs uses.
pub(crate) use self::what_a_lock_names::every_archive_named;
/// Re-exports the lock reading,
/// the present versions and the prune's plan.
pub use self::what_a_lock_names::{
    NamedArchive,
    named_archives,
    plan_archive_prune,
    present_archive_versions,
};
/// Re-exports the check for a release archive of this repository.
pub use self::what_counts_as_a_release_of_this_repository::is_published_archive;

/// `<toolchain root>/<tool id>/current`;
/// refuses an id that is not one plain directory name.
pub fn current_link_of(
    toolchain_root: &Path,
    id: &str,
) -> Result<PathBuf, ToolchainError> {
    return Ok(tool_directory(toolchain_root, id)?.join(CURRENT_LINK_NAME));
}

/// The program that shows an install of the tool finished:
/// its first declared bin,
/// or an empty name (which no install satisfies) when it declares none.
pub fn first_bin_of_tool(tool: &LockedTool) -> &str {
    return match tool.declaration.bins.first() {
        Some(name) => name.as_str(),
        None => "",
    };
}

/// What:
/// The program an install of this tool must hold to count as finished:
/// the first declared bin of a tool this repository publishes,
/// and nothing for any other tool,
/// which finishes on the marker alone.
/// Why:
/// Every reader of "is this version finished" asks the same question of the same tool,
/// and one answer here keeps the install,
/// the pruning and the repair of a killed install from disagreeing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function programThatFinishesTheInstall(tool: LockedTool): string
/// ```
pub fn program_that_finishes_the_install(tool: &LockedTool) -> &str {
    if !is_published_archive(tool) {
        return "";
    }
    return first_bin_of_tool(tool);
}

/// What:
/// Whether the install in `version_dir` finished:
/// it carries the completion marker (written last,
/// after the unpacking succeeded and the program was found),
/// and the tool's first declared bin is still found in it.
/// A tool that declares no bin finishes on the marker alone.
/// Why:
/// The program alone proves nothing:
/// `tar` creates a file with its final mode before it writes the content,
/// so an interrupted unpacking leaves an executable file of the right name and the wrong bytes.
/// A directory without the marker (one an earlier release unpacked in place,
/// or one cut short) is therefore never finished:
/// it is never the newest,
/// so it is never the reason an older working version is deleted or `current` moves,
/// and the next install of that version replaces it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function isFinished(versionDir: string, firstBin: string): boolean
/// ```
pub fn is_finished(
    version_dir: &Path,
    first_bin: &str,
) -> bool {
    if !has_complete_marker(version_dir) {
        return false;
    }
    return first_bin.is_empty() || find_bin_under(version_dir, first_bin).is_some();
}

/// Orders two version names newest first.
fn newest_first<T: AsRef<str>>(
    left: &T,
    right: &T,
) -> Ordering {
    return compare_versions(right.as_ref(), left.as_ref());
}

/// Every version directory of the tool (a plain directory that is not `current`,
/// not hidden),
/// newest first.
pub fn version_names(
    toolchain_root: &Path,
    id: &str,
) -> Vec<String> {
    // An id that is not one plain directory name is refused where the plan and the lock are decoded; for any other
    // value there is no directory to list, and listing none is the safe answer.
    let Ok(directory) = tool_directory(toolchain_root, id) else {
        return Vec::new();
    };
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut names: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        let name: String = entry.file_name().to_string_lossy().to_string();
        let is_plain_directory: bool = match entry.file_type() {
            Ok(kind) => kind.is_dir(),
            Err(_) => false,
        };
        if is_plain_directory && !name.starts_with('.') && name != CURRENT_LINK_NAME {
            names.push(name);
        }
    }
    names.sort_by(newest_first);
    return names;
}

/// The newest version of the tool whose install finished.
pub fn newest_installed_archive(
    toolchain_root: &Path,
    id: &str,
    first_bin: &str,
) -> Option<String> {
    for name in version_names(toolchain_root, id) {
        // A listed name is a plain directory name, so the path is always built.
        if let Ok(directory) = tool_install_dir(toolchain_root, id, &name)
            && is_finished(&directory, first_bin)
        {
            return Some(name);
        }
    }
    return None;
}

/// The version the tool's `current` link names (the link's target is the version directory's name),
/// or `None` when there is no link.
pub fn running_version_archive(
    toolchain_root: &Path,
    id: &str,
) -> Option<String> {
    let target: PathBuf = fs::read_link(current_link_of(toolchain_root, id).ok()?).ok()?;
    let name: String = file_name_text(&target);
    if name.is_empty() {
        return None;
    }
    return Some(name);
}

/// What:
/// Points `current` at the newest finished version of the tool and returns that version.
/// Why:
/// Every tree's exposed link goes through `current`,
/// so removing an older version never leaves a dangling link,
/// and two trees installing at once converge on the newest.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function pointCurrentAtNewestArchive(root: string, id: string, firstBin: string): string | undefined
/// ```
pub fn point_current_at_newest_archive(
    toolchain_root: &Path,
    id: &str,
    first_bin: &str,
) -> Result<Option<String>, ToolchainError> {
    let Some(newest) = newest_installed_archive(toolchain_root, id, first_bin) else {
        return Ok(None);
    };
    if running_version_archive(toolchain_root, id).as_deref() != Some(newest.as_str()) {
        atomic_symlink(&current_link_of(toolchain_root, id)?, Path::new(&newest))?;
    }
    return Ok(Some(newest));
}

/// What:
/// Removes the downloaded archives of one version of the tool,
/// which `install` keeps under `<toolchain root>/downloads/<tool>/<version>/`.
/// Why:
/// One directory per tool and version:
/// the earlier flat names (`<id>-<version>-<file>`) were matched by prefix,
/// and the ids `a` and `a-1` can make the same prefix.
/// Archives an earlier release downloaded under the flat names stay where they are;
/// nothing reads them and they can be deleted by hand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function removeDownloads(root: string, id: string, version: string): void
/// ```
pub fn remove_downloads(
    toolchain_root: &Path,
    id: &str,
    version: &str,
) -> Result<(), ToolchainError> {
    let directory: PathBuf = tool_downloads_dir(toolchain_root, id, version)?;
    match fs::remove_dir_all(&directory) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => {
            return Err(ToolchainError::failure(format!(
                "cannot remove the superseded downloads {}: {error}",
                directory.display()
            )));
        }
    }
    if let Some(tool_downloads) = directory.parent() {
        // Best effort: the tool's downloads directory goes once its last version's downloads are gone, and stays when
        // another version's downloads are still in it.
        let _ = fs::remove_dir(tool_downloads);
    }
    return Ok(());
}

/// What:
/// Deletes every version directory of the tool that is older than the newest finished one,
/// with the archives downloaded for them,
/// and returns the version directories deleted.
/// Why:
/// A repository-published tool is never kept in an older version once a newer one is installed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function removeOlderArchives(root: string, id: string, firstBin: string): string[]
/// ```
pub fn remove_older_archives(
    toolchain_root: &Path,
    id: &str,
    first_bin: &str,
) -> Result<Vec<PathBuf>, ToolchainError> {
    let Some(newest) = newest_installed_archive(toolchain_root, id, first_bin) else {
        return Ok(Vec::new());
    };
    let mut removed: Vec<PathBuf> = Vec::new();
    for name in version_names(toolchain_root, id) {
        if compare_versions(&name, &newest) != Ordering::Less {
            continue;
        }
        let directory: PathBuf = tool_install_dir(toolchain_root, id, &name)?;
        if let Err(error) = fs::remove_dir_all(&directory) {
            return Err(ToolchainError::failure(format!(
                "cannot remove the superseded {}: {error}",
                directory.display()
            )));
        }
        remove_downloads(toolchain_root, id, &name)?;
        removed.push(directory);
    }
    return Ok(removed);
}
