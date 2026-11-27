//! What:
//! The archive-route tools one lock names,
//! read from its JSON text without decoding the rest of the model,
//! the version directories present for each of them,
//! and the `(tool id, version)` pairs a prune would delete.
//! Why:
//! Only tools some known lock names as repository-published are ever judged:
//! an archive directory nothing names is never judged,
//! because its owner cannot be told from a third-party tool's.
//! Other worktrees may sit on a commit whose lock is in an older format,
//! and such a lock has no `options.tagPrefix` and names no archive tool,
//! which is the safe reading for a cleanup.
//! A lock that names a repository release under an id like `..` is not an older format but a crafted one,
//! so the read stops instead of joining that id onto the toolchain root.
//! The plan itself is pure over plain maps,
//! so its tests feed allowed and rejected cases without a file system.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function namedArchives(lockText: string, origin: string): Map<string, NamedArchive>  // throws
//! export function presentArchiveVersions(root: string, named: Map<string, NamedArchive>): Map<string, PresentVersion[]>
//! export function planArchivePrune(present: Map<string, PresentVersion[]>): [string, string][]
//! ```

/// Imports `BTreeMap` and `BTreeSet` from `std::collections`.
use std::collections::{
    BTreeMap,
    BTreeSet,
};
/// Imports `Path` from `std::path`.
use std::path::Path;

/// Imports `serde_json::Value`,
/// the shape a lock's text is read as when only a few fields matter.
use serde_json::Value;

/// Imports `compare_versions` from `crate::version`.
use crate::version::compare_versions;

/// Imports `ToolchainError` from `super::super::error`.
use super::super::error::ToolchainError;
/// Imports what this module uses from `super::super::layout`.
use super::super::layout::tool_install_dir;
/// Imports what this module uses from `super::super::model`.
use super::super::model::Request;
/// Imports `PresentVersion` from `super::super::prune_cargo`.
use super::super::prune_cargo::PresentVersion;
/// Imports the name checks from `super::super::segment`.
use super::super::segment::{
    check_bin_name,
    check_tool_id,
    check_version,
};
/// Imports `names_own_releases` from `super::what_counts_as_a_release_of_this_repository`.
use super::what_counts_as_a_release_of_this_repository::names_own_releases;
/// Imports what this module uses from the parent.
use super::{
    is_finished,
    version_names,
};

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
pub fn named_archives(
    lock_text: &str,
    origin: &str,
) -> Result<BTreeMap<String, NamedArchive>, ToolchainError> {
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
        let source: String = match entry
            .get("request")
            .and_then(Value::as_str)
            .map(Request::parse)
        {
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
        let bin_check: Result<(), String> = if first_bin.is_empty() {
            Ok(())
        } else {
            check_bin_name(first_bin)
        };
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
/// Every archive-route tool a lock names,
/// whichever repository publishes it,
/// with the versions and first bin each row records.
/// Why:
/// A hidden sibling may belong to a third-party archive tool as well as to one of this repository's,
/// and a finished copy of a version no known lock names is what prune removes.
/// The rows are only compared with names found on disk,
/// never joined onto a path,
/// so a crafted name cannot make this read reach outside the toolchain root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function everyArchiveNamed(lockText: string): Map<string, NamedArchive>
/// ```
pub(crate) fn every_archive_named(lock_text: &str) -> BTreeMap<String, NamedArchive> {
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
        let Some(version) = entry.get("version").and_then(Value::as_str) else {
            continue;
        };
        // What: a first bin the plan could not use is no name to look for, so the marker alone finishes such a copy.
        // Why: a crafted bin name must not reach a path below the hidden directory.
        let first_bin: &str = match first_bin_of(entry) {
            Some(name) if check_bin_name(name).is_ok() => name,
            _ => "",
        };
        let slot: &mut NamedArchive = named.entry(id.clone()).or_default();
        slot.versions.insert(version.to_string());
        slot.first_bin = first_bin.to_string();
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
pub fn present_archive_versions(
    toolchain_root: &Path,
    named: &BTreeMap<String, NamedArchive>,
) -> BTreeMap<String, Vec<PresentVersion>> {
    let mut present: BTreeMap<String, Vec<PresentVersion>> = BTreeMap::new();
    for (id, archive) in named {
        let mut found: Vec<PresentVersion> = Vec::new();
        for version in version_names(toolchain_root, id) {
            let Ok(directory) = tool_install_dir(toolchain_root, id, &version) else {
                continue;
            };
            let finished: bool = is_finished(&directory, &archive.first_bin);
            found.push(PresentVersion {
                version,
                finished,
            });
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
pub fn plan_archive_prune(
    present: &BTreeMap<String, Vec<PresentVersion>>
) -> Vec<(String, String)> {
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
