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
//! except a version a visible lock names or a running process of this user uses (`store_keeps.rs`),
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
/// Imports `ErrorKind` from `std::io`.
use std::io::ErrorKind;
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};

/// Imports `serde_json::Value`,
/// the shape a lock's text is read as when only a few fields matter.
use serde_json::Value;

/// Imports what this module uses from `crate::constants`.
use crate::constants::{OWN_CRATE_PREFIX, OWN_REPOSITORY_SOURCE};
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
use super::layout::{find_bin_under, install_dir_name, tool_directory, tool_downloads_dir, tool_install_dir};
/// Imports what this module uses from `super::model`.
use super::model::{LockedTool, Request, Route};
/// Imports `PresentVersion` from `super::prune_cargo`.
use super::prune_cargo::PresentVersion;
/// Imports the name checks from `super::segment`.
use super::segment::{check_bin_name, check_tool_id, check_version};
/// Imports the keeps every removal asks from `super::store_keeps`.
use super::store_keeps::InstallKeeps;
/// Imports `file_name_text` from `super::util`.
use super::util::file_name_text;

/// Whether a tag prefix starts the way every release tag of this repository does.
fn starts_with_own_crate_prefix(prefix: &str) -> bool {
    return prefix.starts_with(OWN_CRATE_PREFIX);
}

/// Whether a request source and a tag prefix name releases of this repository:
/// the source is `OWN_REPOSITORY_SOURCE` apart from letter case
/// (GitHub owner and repository names are case-insensitive,
/// and the names are ASCII),
/// and the prefix starts with `monochromatic-`.
fn names_own_releases(request_source: &str, tag_prefix: Option<&str>) -> bool {
    return request_source.eq_ignore_ascii_case(OWN_REPOSITORY_SOURCE) && tag_prefix.is_some_and(starts_with_own_crate_prefix);
}

/// What:
/// Whether a tool is an archive-route tool published by this repository:
/// the `archive` route,
/// a request for `github:Aquaticats/Monochromatic` (in any letter case),
/// and an `options.tagPrefix` that starts with `monochromatic-`.
/// Why:
/// The tag prefix is what names one program's releases in this repository's many tags,
/// and this repository tags every program it publishes that way (`OWN_CRATE_PREFIX`,
/// the rule that already marks its crates),
/// so the plan needs no second list that could drift from the releases.
/// The prefix alone would also match a tool of another repository that happens to start its tags the same way,
/// and the owner's rule ("no older repo-maintained tools") is about this repository's tools,
/// so the request has to name this repository too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function isPublishedArchive(tool: LockedTool): boolean
/// ```
pub fn is_published_archive(tool: &LockedTool) -> bool {
    if tool.declaration.route != Route::Archive {
        return false;
    }
    return names_own_releases(&tool.declaration.request.source, tool.declaration.options.tag_prefix.as_deref());
}

/// `<toolchain root>/<tool id>/current`;
/// refuses an id that is not one plain directory name.
pub fn current_link_of(toolchain_root: &Path, id: &str) -> Result<PathBuf, ToolchainError> {
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
pub fn is_finished(version_dir: &Path, first_bin: &str) -> bool {
    if !has_complete_marker(version_dir) {
        return false;
    }
    return first_bin.is_empty() || find_bin_under(version_dir, first_bin).is_some();
}

/// Orders two version names newest first.
fn newest_first<T: AsRef<str>>(left: &T, right: &T) -> Ordering {
    return compare_versions(right.as_ref(), left.as_ref());
}

/// Every version directory of the tool (a plain directory that is not `current`,
/// not hidden),
/// newest first.
pub fn version_names(toolchain_root: &Path, id: &str) -> Vec<String> {
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
pub fn newest_installed_archive(toolchain_root: &Path, id: &str, first_bin: &str) -> Option<String> {
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
pub fn running_version_archive(toolchain_root: &Path, id: &str) -> Option<String> {
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
pub fn point_current_at_newest_archive(toolchain_root: &Path, id: &str, first_bin: &str) -> Result<Option<String>, ToolchainError> {
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
pub fn remove_downloads(toolchain_root: &Path, id: &str, version: &str) -> Result<(), ToolchainError> {
    let directory: PathBuf = tool_downloads_dir(toolchain_root, id, version)?;
    match fs::remove_dir_all(&directory) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => {
            return Err(ToolchainError::failure(format!("cannot remove the superseded downloads {}: {error}", directory.display())));
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
/// except the ones `keeps` protects,
/// and returns the version directories deleted.
/// Why:
/// A repository-published tool is never kept in an older version once a newer one is installed,
/// but the newest-only rule does not outrank a version that is in use:
/// a worktree's lock may pin it,
/// a service may be running it,
/// and taking it away leaves that tree to install it again
/// and that service unable to start until someone does.
/// The version this install wrote is kept whatever the newest finished one is,
/// because a concurrent install of a newer version must not turn this run's own install into an older one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function removeOlderArchives(root: string, id: string, firstBin: string, keeps: InstallKeeps, installedVersion: string): string[]
/// ```
pub fn remove_older_archives(
    toolchain_root: &Path,
    id: &str,
    first_bin: &str,
    keeps: &InstallKeeps,
    installed_version: &str,
) -> Result<Vec<PathBuf>, ToolchainError> {
    let Some(newest) = newest_installed_archive(toolchain_root, id, first_bin) else {
        return Ok(Vec::new());
    };
    // What: the directory name a lock row for this tool becomes, which is what `keeps` was read by.
    // Why: the keep rule names a version by the tool's store directory and the version, never by a full path.
    let tool_store_name: String = install_dir_name(id);
    let mut removed: Vec<PathBuf> = Vec::new();
    for name in version_names(toolchain_root, id) {
        if compare_versions(&name, &newest) != Ordering::Less {
            continue;
        }
        if name == installed_version {
            // This install's own version stays, even when another run installed a newer one meanwhile.
            continue;
        }
        let directory: PathBuf = tool_install_dir(toolchain_root, id, &name)?;
        if keeps.keep_reason(&tool_store_name, &name, &directory).is_some() {
            // A lock names it or a process of this user runs it: the newest-only rule stops here.
            continue;
        }
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
/// A tool whose id,
/// version or first bin is not one plain directory name refuses the whole read,
/// naming `origin` (the lock's path) and the entry.
/// Why:
/// Other worktrees may sit on a commit whose lock is in an older format;
/// such a lock has no `options.tagPrefix` and names no archive tool,
/// which is the safe reading for a cleanup.
/// A lock that names a repository release under an id like `..` is not an older format but a crafted one,
/// and the id would be joined onto the toolchain root and listed and deleted from,
/// so the cleanup stops instead of judging it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function namedArchives(lockText: string, origin: string): Map<string, NamedArchive>  // throws
/// ```
pub fn named_archives(lock_text: &str, origin: &str) -> Result<BTreeMap<String, NamedArchive>, ToolchainError> {
    let mut named: BTreeMap<String, NamedArchive> = BTreeMap::new();
    let Ok(lock) = serde_json::from_str::<Value>(lock_text) else {
        return Ok(named);
    };
    let Some(tools) = lock.get("tools").and_then(Value::as_object) else {
        return Ok(named);
    };
    for (id, entry) in tools {
        if entry.get("route").and_then(Value::as_str) != Some("archive") {
            continue;
        }
        let source: String = match entry.get("request").and_then(Value::as_str).map(Request::parse) {
            Some(Ok(request)) => request.source,
            _ => String::new(),
        };
        let own: bool = names_own_releases(&source, tag_prefix_of(entry));
        let Some(version) = entry.get("version").and_then(Value::as_str) else {
            continue;
        };
        if !own {
            continue;
        }
        // A tool that declares no bin finishes on its marker alone; its first bin is the empty name.
        let first_bin: &str = first_bin_of(entry).unwrap_or("");
        let bin_check: Result<(), String> = if first_bin.is_empty() { Ok(()) } else { check_bin_name(first_bin) };
        for checked in [check_tool_id(id), check_version(version), bin_check] {
            if let Err(message) = checked {
                return Err(ToolchainError::failure(format!(
                    "{origin} names a release archive that cannot be judged safely, so nothing was deleted; tools.{id}: {message}"
                )));
            }
        }
        let slot: &mut NamedArchive = named.entry(id.clone()).or_default();
        slot.versions.insert(version.to_string());
        slot.first_bin = first_bin.to_string();
    }
    return Ok(named);
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
            let Ok(directory) = tool_install_dir(toolchain_root, id, &version) else {
                continue;
            };
            let finished: bool = is_finished(&directory, &archive.first_bin);
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
