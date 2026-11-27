//! What:
//! Which version of a `cargo-binstall` tool runs,
//! whichever route installed it,
//! and how the `current` link inside the crate directory is settled on it:
//! the version the link names,
//! the newest finished one,
//! and the rule that keeps a link from moving onto a version that is going away.
//! Why:
//! Every tree's exposed link goes through `current`,
//! so two trees syncing at once must converge on one version,
//! and a version deleted underneath the link must never leave it dangling.
//! A repository-published tool answers through the archive layout when its route is `archive`,
//! and through the crate directory otherwise,
//! so the route switch lives here beside the link it settles.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function runningVersion(root: string, crate: string): string | undefined
//! export function newestInstalledOf(root: string, tool: LockedTool): string | undefined
//! export function pointCurrentAtNewestOf(root: string, tool: LockedTool): string | undefined
//! export function pointCurrentAtNewest(root: string, crate: string): string | undefined
//! export function pointCurrentAt(root: string, crate: string, version: string, deletedHere: string[]): boolean
//! ```

/// Imports `fs` from `std`.
use std::fs;
/// Imports what this module uses from `std::path`.
use std::path::{
    Path,
    PathBuf,
};

/// Imports `compare_versions` from `crate::version`.
use crate::version::compare_versions;

/// Imports `ToolchainError` from `super::super::error`.
use super::super::error::ToolchainError;
/// Imports `atomic_symlink` from `super::super::install_support`.
use super::super::install_support::atomic_symlink;
/// Imports `LockedTool` from `super::super::model`.
use super::super::model::LockedTool;
/// Imports `package_name` from `super::super::observe`.
use super::super::observe::package_name;
/// Imports what this module uses from `super::super::published_archive`.
use super::super::published_archive::{
    first_bin_of_tool,
    is_published_archive,
    newest_installed_archive,
    point_current_at_newest_archive,
};
/// Imports `file_name_text` from `super::super::util`.
use super::super::util::file_name_text;
/// Imports what this module uses from the parent.
use super::{
    current_link,
    newest_installed,
    recorded_version,
    version_directory,
};

/// What:
/// The newest finished version of a repository-published tool under the toolchain root,
/// whichever route installed it.
/// Why:
/// `install`,
/// `list` and the warnings all ask this one question.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function newestInstalledOf(root: string, tool: LockedTool): string | undefined
/// ```
pub fn newest_installed_of(
    toolchain_root: &Path,
    tool: &LockedTool,
) -> Option<String> {
    if is_published_archive(tool) {
        return newest_installed_archive(
            toolchain_root,
            &tool.declaration.id,
            first_bin_of_tool(tool),
        );
    }
    return newest_installed(toolchain_root, &package_name(tool));
}

/// What:
/// Points the `current` link of a repository-published tool at its newest finished version,
/// whichever route installed it,
/// and returns that version.
/// Why:
/// Every tree's exposed link goes through `current`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function pointCurrentAtNewestOf(root: string, tool: LockedTool): string | undefined
/// ```
pub fn point_current_at_newest_of(
    toolchain_root: &Path,
    tool: &LockedTool,
) -> Result<Option<String>, ToolchainError> {
    if is_published_archive(tool) {
        return point_current_at_newest_archive(
            toolchain_root,
            &tool.declaration.id,
            first_bin_of_tool(tool),
        );
    }
    return point_current_at_newest(toolchain_root, &package_name(tool));
}

/// The version the crate's `current` link names (the link's target is the version directory's name),
/// or `None` when there is no link.
pub fn running_version(
    toolchain_root: &Path,
    crate_name: &str,
) -> Option<String> {
    let target: PathBuf = fs::read_link(current_link(toolchain_root, crate_name)).ok()?;
    let name: String = file_name_text(&target);
    if name.is_empty() {
        return None;
    }
    return Some(name);
}

/// What:
/// Points `current` at the newest finished version of the crate and returns that version.
/// Why:
/// Every tree's exposed link goes through `current`,
/// so removing an older version never leaves a dangling link,
/// and two trees syncing at once converge on the newest.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function pointCurrentAtNewest(root: string, crate: string): string | undefined
/// ```
pub fn point_current_at_newest(
    toolchain_root: &Path,
    crate_name: &str,
) -> Result<Option<String>, ToolchainError> {
    let Some(newest) = newest_installed(toolchain_root, crate_name) else {
        return Ok(None);
    };
    // What: the newest finished version is written even when the link names an older one.
    // Why: this helper's whole job is to settle the link on that version, and `false` only reports that some other
    // link was left alone.
    let _written: bool = point_current_at(toolchain_root, crate_name, &newest, &[])?;
    return Ok(Some(newest));
}

/// What:
/// Points `current` at `version`,
/// unless it already names it,
/// and reports whether the link names `version` afterwards.
/// `deleted_here` names the versions the caller is about to delete.
/// Why:
/// Every tree's exposed link goes through `current`,
/// and a caller that knows which version survives (the prune,
/// for one) must be able to name it,
/// rather than have the newest installed version chosen for it.
/// `false` tells that caller the link was left naming something else,
/// so it can settle the link itself.
/// A link naming a version in `deleted_here` is never preserved:
/// that version is going away,
/// and the link has to end on a version that stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function pointCurrentAt(root: string, crate: string, version: string, deletedHere: string[]): boolean
/// ```
pub fn point_current_at(
    toolchain_root: &Path,
    crate_name: &str,
    version: &str,
    deleted_here: &[&str],
) -> Result<bool, ToolchainError> {
    let link: PathBuf = current_link(toolchain_root, crate_name);
    let running: Option<String> = running_version(toolchain_root, crate_name);
    if running.as_deref() == Some(version) {
        return Ok(true);
    }
    // What: a version that is not there at this moment is never what the link is pointed at.
    // Why: a caller's snapshot can name a version another command has already removed,
    // and a link written then dangles although a finished newer install is there.
    // The read is one `metadata`, so the window left is the instant between it and the link write.
    if !version_directory(toolchain_root, crate_name, version).is_dir() {
        return Ok(false);
    }
    // What: a link that already names a finished version newer than `version` stays as it is,
    // unless that newer version is one the caller deletes.
    // Why: the caller's snapshot is older than the install in front of it,
    // and an interrupted or concurrent install is not undone by an older plan;
    // a link left on a version the caller deletes would dangle once the deletions run.
    if let Some(named) = running.as_deref()
        && compare_versions(named, version).is_gt()
        && !deleted_here.contains(&named)
        && recorded_version(
            &version_directory(toolchain_root, crate_name, named),
            crate_name,
        )
        .is_some()
    {
        return Ok(false);
    }
    atomic_symlink(&link, Path::new(version))?;
    return Ok(true);
}
