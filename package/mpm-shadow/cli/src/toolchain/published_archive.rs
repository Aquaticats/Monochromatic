//! What:
//! Where an archive-route tool that this repository publishes lives and how its version is chosen:
//! one directory per version under the tool's own directory,
//! `<toolchain root>/<tool id>/<version>`,
//! and a `current` link inside the tool's directory that names the newest finished version.
//! Why:
//! The owner's rule covers every repository-published tool,
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
/// Imports `BTreeMap`
/// and `BTreeSet` from `std::collections`.
use std::collections::{BTreeMap, BTreeSet};
/// Imports `fs` from `std`.
use std::fs;
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};

/// Imports `serde_json::Value`,
/// the shape a lock's text is read as when only a few fields matter.
use serde_json::Value;

/// Imports `OWN_CRATE_PREFIX` from `crate::constants`.
use crate::constants::OWN_CRATE_PREFIX;
/// Imports `compare_versions` from `crate::version`.
use crate::version::compare_versions;

/// Imports `CURRENT_LINK_NAME` from `super::cargo_installs`.
use super::cargo_installs::CURRENT_LINK_NAME;
/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports `atomic_symlink` from `super::install_support`.
use super::install_support::atomic_symlink;
/// Imports what this module uses from `super::layout`.
use super::layout::{find_bin_under, install_dir_name, tool_install_dir};
/// Imports what this module uses from `super::model`.
use super::model::{LockedTool, Route};
/// Imports `PresentVersion` from `super::prune_cargo`.
use super::prune_cargo::PresentVersion;
/// Imports `file_name_text` from `super::util`.
use super::util::file_name_text;

/// What:
/// Whether a tool is an archive-route tool published by this repository:
/// the `archive` route with an `options.tagPrefix` that starts with `monochromatic-`.
/// Why:
/// The tag prefix is what names one program's releases in this repository's many tags,
/// and this repository tags every program it publishes that way (`OWN_CRATE_PREFIX`,
/// the rule that already marks its crates),
/// so the plan needs no second list that could drift from the releases.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function isPublishedArchive(tool: LockedTool): boolean
/// ```
pub fn is_published_archive(tool: &LockedTool) -> bool {
    if tool.declaration.route != Route::Archive {
        return false;
    }
    return match &tool.declaration.options.tag_prefix {
        Some(prefix) => prefix.starts_with(OWN_CRATE_PREFIX),
        None => false,
    };
}

/// `<toolchain root>/<tool id>`,
/// the directory that holds every installed version of the tool.
pub fn tool_directory(toolchain_root: &Path, id: &str) -> PathBuf {
    return tool_install_dir(toolchain_root, id, "");
}

/// `<toolchain root>/<tool id>/current`.
pub fn current_link_of(toolchain_root: &Path, id: &str) -> PathBuf {
    return tool_directory(toolchain_root, id).join(CURRENT_LINK_NAME);
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

/// Whether the install in `version_dir` finished:
/// the tool's first declared bin is found in it.
/// An interrupted unpacking leaves a directory without the program.
fn is_finished(version_dir: &Path, first_bin: &str) -> bool {
    if first_bin.is_empty() {
        return false;
    }
    return find_bin_under(version_dir, first_bin).is_some();
}

/// Orders two version names newest first.
fn newest_first<T: AsRef<str>>(left: &T, right: &T) -> Ordering {
    return compare_versions(right.as_ref(), left.as_ref());
}

/// Every version directory of the tool (a plain directory that is not `current`,
/// not hidden),
/// newest first.
fn version_names(toolchain_root: &Path, id: &str) -> Vec<String> {
    let Ok(entries) = fs::read_dir(tool_directory(toolchain_root, id)) else {
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
pub fn newest_installed_archive(toolchain_root: &Path, id: &str, first_bin: &str) -> Option<String> {
    for name in version_names(toolchain_root, id) {
        if is_finished(&tool_install_dir(toolchain_root, id, &name), first_bin) {
            return Some(name);
        }
    }
    return None;
}

/// The version the tool's `current` link names (the link's target is the version directory's name),
/// or `None` when there is no link.
pub fn running_version_archive(toolchain_root: &Path, id: &str) -> Option<String> {
    let target: PathBuf = fs::read_link(current_link_of(toolchain_root, id)).ok()?;
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
pub fn point_current_at_newest_archive(toolchain_root: &Path, id: &str, first_bin: &str) -> Result<Option<String>, ToolchainError> {
    let Some(newest) = newest_installed_archive(toolchain_root, id, first_bin) else {
        return Ok(None);
    };
    if running_version_archive(toolchain_root, id).as_deref() != Some(newest.as_str()) {
        atomic_symlink(&current_link_of(toolchain_root, id), Path::new(&newest))?;
    }
    return Ok(Some(newest));
}

/// Removes the downloaded archives of one version of the tool,
/// which `install` keeps under `<toolchain root>/downloads` as `<id>-<version>-<file>`.
fn remove_downloads(toolchain_root: &Path, id: &str, version: &str) -> Result<(), ToolchainError> {
    let stem: String = format!("{}-{}-", install_dir_name(id), install_dir_name(version));
    let Ok(entries) = fs::read_dir(toolchain_root.join("downloads")) else {
        return Ok(());
    };
    for entry in entries.flatten() {
        if !entry.file_name().to_string_lossy().starts_with(&stem) {
            continue;
        }
        if let Err(error) = fs::remove_file(entry.path()) {
            return Err(ToolchainError::failure(format!("cannot remove the superseded download {}: {error}", entry.path().display())));
        }
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
pub fn remove_older_archives(toolchain_root: &Path, id: &str, first_bin: &str) -> Result<Vec<PathBuf>, ToolchainError> {
    let Some(newest) = newest_installed_archive(toolchain_root, id, first_bin) else {
        return Ok(Vec::new());
    };
    let mut removed: Vec<PathBuf> = Vec::new();
    for name in version_names(toolchain_root, id) {
        if compare_versions(&name, &newest) != Ordering::Less {
            continue;
        }
        let directory: PathBuf = tool_install_dir(toolchain_root, id, &name);
        if let Err(error) = fs::remove_dir_all(&directory) {
            return Err(ToolchainError::failure(format!("cannot remove the superseded {}: {error}", directory.display())));
        }
        remove_downloads(toolchain_root, id, &name)?;
        removed.push(directory);
    }
    return Ok(removed);
}

/// One archive-route repository-published tool a lock names:
/// the versions it pins and the program that shows an install finished.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NamedArchive {
    /// Versions some known lock pins for the tool.
    pub versions: BTreeSet<String>,
    /// The tool's first declared bin.
    pub first_bin: String,
}

/// The `options.tagPrefix` of a lock entry.
fn tag_prefix_of(entry: &Value) -> Option<&str> {
    let options: &Value = entry.get("options")?;
    return options.get("tagPrefix")?.as_str();
}

/// The first declared bin of a lock entry.
fn first_bin_of(entry: &Value) -> Option<&str> {
    let bins: &Vec<Value> = entry.get("bins")?.as_array()?;
    return bins.first()?.as_str();
}

/// What:
/// The archive-route repository-published tools one lock names,
/// by tool id,
/// read from its JSON text without decoding the rest of the model.
/// Why:
/// Other worktrees may sit on a commit whose lock is in an older format;
/// such a lock has no `options.tagPrefix` and names no archive tool,
/// which is the safe reading for a cleanup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function namedArchives(lockText: string): Map<string, NamedArchive>
/// ```
pub fn named_archives(lock_text: &str) -> BTreeMap<String, NamedArchive> {
    let mut named: BTreeMap<String, NamedArchive> = BTreeMap::new();
    let Ok(lock) = serde_json::from_str::<Value>(lock_text) else {
        return named;
    };
    let Some(tools) = lock.get("tools").and_then(Value::as_object) else {
        return named;
    };
    for (id, entry) in tools {
        if entry.get("route").and_then(Value::as_str) != Some("archive") {
            continue;
        }
        let own: bool = match tag_prefix_of(entry) {
            Some(prefix) => prefix.starts_with(OWN_CRATE_PREFIX),
            None => false,
        };
        let (Some(first_bin), Some(version)) = (first_bin_of(entry), entry.get("version").and_then(Value::as_str)) else {
            continue;
        };
        if own {
            let slot: &mut NamedArchive = named.entry(id.clone()).or_default();
            slot.versions.insert(version.to_string());
            slot.first_bin = first_bin.to_string();
        }
    }
    return named;
}

/// What:
/// The version directories under each named tool's directory,
/// with whether each finished.
/// Why:
/// Only tools some known lock names as repository-published are looked at:
/// an archive directory nothing names is never judged,
/// because its owner cannot be told from a third-party tool's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function presentArchiveVersions(root: string, named: Map<string, NamedArchive>): Map<string, PresentVersion[]>
/// ```
pub fn present_archive_versions(toolchain_root: &Path, named: &BTreeMap<String, NamedArchive>) -> BTreeMap<String, Vec<PresentVersion>> {
    let mut present: BTreeMap<String, Vec<PresentVersion>> = BTreeMap::new();
    for (id, archive) in named {
        let mut found: Vec<PresentVersion> = Vec::new();
        for version in version_names(toolchain_root, id) {
            let directory: PathBuf = tool_install_dir(toolchain_root, id, &version);
            let finished: bool = find_bin_under(&directory, &archive.first_bin).is_some();
            found.push(PresentVersion { version, finished });
        }
        if !found.is_empty() {
            present.insert(id.clone(), found);
        }
    }
    return present;
}

/// What:
/// The `(tool id, version)` pairs of repository-published archive tools to delete:
/// everything older than the newest finished version;
/// when no version finished,
/// nothing.
/// Why:
/// Pure over plain maps,
/// so tests feed it allowed and rejected cases without a file system.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function planArchivePrune(present: Map<string, PresentVersion[]>): [string, string][]
/// ```
pub fn plan_archive_prune(present: &BTreeMap<String, Vec<PresentVersion>>) -> Vec<(String, String)> {
    let mut removals: Vec<(String, String)> = Vec::new();
    for (id, versions) in present {
        let mut newest: Option<&str> = None;
        for candidate in versions {
            if !candidate.finished {
                continue;
            }
            let is_newer: bool = match newest {
                Some(kept) => compare_versions(&candidate.version, kept).is_gt(),
                None => true,
            };
            if is_newer {
                newest = Some(&candidate.version);
            }
        }
        let Some(kept) = newest else {
            continue;
        };
        for candidate in versions {
            if compare_versions(&candidate.version, kept).is_lt() {
                removals.push((id.clone(), candidate.version.clone()));
            }
        }
    }
    return removals;
}
