//! What:
//! The rule the cargo cleanup runs:
//! which `(crate, version)` pairs to delete,
//! and which version of one crate the link that names no version may keep.
//! Why:
//! The rule is pure over plain maps,
//! so its tests feed it allowed and rejected cases without a file system:
//! a crate this repository publishes keeps its newest finished version when any lock names the crate,
//! and picks nothing otherwise;
//! any other crate keeps exactly the versions some lock names.
//! A caller imports from `prune_cargo` and never from this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export type PresentVersion = { version: string; finished: boolean };
//! export function planPrune(present: Map<string, PresentVersion[]>, named: Map<string, Set<string>>): [string, string][];
//! function stays(crateName: string, candidate: PresentVersion, removals: [string, string, string][]): boolean;
//! function newestFinished(kept: PresentVersion[]): string | undefined;
//! ```

/// Imports the sorted collections the rule works on.
use std::collections::{
    BTreeMap,
    BTreeSet,
};
/// Imports the directory a removal names.
use std::path::PathBuf;

/// Imports the prefix that marks a crate this repository publishes.
use crate::constants::OWN_CRATE_PREFIX;
/// Imports the version order every newest-wins rule here uses.
use crate::version::compare_versions;
/// One version directory found on disk.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresentVersion {
    /// The version (the directory's name).
    pub version: String,
    /// Whether cargo's record inside the directory names the crate,
    /// so the install finished.
    pub finished: bool,
}

/// What:
/// The rule:
/// which `(crate, version)` pairs to delete.
/// A crate this repository publishes keeps its newest finished version when any lock names the crate,
/// and nothing otherwise;
/// any other crate keeps exactly the versions some lock names.
/// Why:
/// Pure over plain maps,
/// so tests feed it allowed and rejected cases without a file system.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function planPrune(present: Map<string, PresentVersion[]>, named: Map<string, Set<string>>): [string, string][]
/// ```
pub fn plan_prune(
    present: &BTreeMap<String, Vec<PresentVersion>>,
    named: &BTreeMap<String, BTreeSet<String>>,
) -> Vec<(String, String)> {
    let mut removals: Vec<(String, String)> = Vec::new();
    for (crate_name, versions) in present {
        let named_here: Option<&BTreeSet<String>> = named.get(crate_name);
        if crate_name.starts_with(OWN_CRATE_PREFIX) {
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
            for candidate in versions {
                let keep: bool = match (named_here, newest) {
                    (Some(_), Some(kept)) => !compare_versions(&candidate.version, kept).is_lt(),
                    (Some(_), None) => true,
                    (None, _) => false,
                };
                if !keep {
                    removals.push((crate_name.clone(), candidate.version.clone()));
                }
            }
            continue;
        }
        for candidate in versions {
            let is_named: bool = match named_here {
                Some(set) => set.contains(&candidate.version),
                None => false,
            };
            if !is_named {
                removals.push((crate_name.clone(), candidate.version.clone()));
            }
        }
    }
    return removals;
}

/// What:
/// Whether this run keeps `candidate`,
/// a version of `crate_name`,
/// from `removals`.
/// Why:
/// The link decision counts only the versions that stay,
/// and a removal carries the crate it belongs to.
pub(super) fn stays(
    crate_name: &str,
    candidate: &PresentVersion,
    removals: &[(String, String, PathBuf)],
) -> bool {
    return !removals
        .iter()
        .any(|(crate_of_removal, version, _directory)| {
            return crate_of_removal == crate_name && version == &candidate.version;
        });
}

/// What:
/// The newest finished version of `kept`,
/// or `None` when none of them finished.
/// Why:
/// Only a finished version can be what `current` names:
/// an unfinished one is still being written.
pub(super) fn newest_finished(kept: &[&PresentVersion]) -> Option<String> {
    let mut newest: Option<&str> = None;
    for candidate in kept {
        if !candidate.finished {
            continue;
        }
        let is_newer: bool = match newest {
            Some(held) => compare_versions(&candidate.version, held).is_gt(),
            None => true,
        };
        if is_newer {
            newest = Some(&candidate.version);
        }
    }
    return newest.map(|version| return version.to_string());
}
