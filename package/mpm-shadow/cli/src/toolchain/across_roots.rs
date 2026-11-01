//! What:
//! Which copy of a repository-published tool runs when both toolchain roots (the repository scope's and the machine-global one)
//! can hold one:
//! the newest finished version across both,
//! wherever it is.
//! Why:
//! The owner's decision for this repository's own tools is one set across both roots,
//! where the newest copy wins wherever it is,
//! and a scope whose lock pins an older version runs the newer copy and prints the older-lock warning.
//! Every link,
//! launcher and observation of such a tool asks here instead of looking in its own root only.
//! A tie (the same version in both roots) goes to the root of the scope that asks,
//! so a scope's own links stay inside its own root when nothing newer exists elsewhere.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // across-roots.ts
//! export function winningCopy(scope: Scope, tool: LockedTool): { root: string; version: string } | undefined
//! export function winningRoot(scope: Scope, tool: LockedTool): string
//! export function winningVersion(scope: Scope, tool: LockedTool): string | undefined
//! ```

/// Imports `Ordering` from `std::cmp`.
use std::cmp::Ordering;
/// Imports `PathBuf` from `std::path`.
use std::fs;
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};

/// Imports `compare_versions` from `crate::version`.
use crate::version::compare_versions;

/// Imports `newest_installed_of` from `super::cargo_installs`.
use super::cargo_installs::newest_installed_of;
/// Imports `LockedTool` from `super::model`.
use super::model::LockedTool;
/// Imports `Scope` from `super::scope`.
use super::scope::Scope;

/// The copy that runs:
/// the toolchain root that holds it and its version.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WinningCopy {
    /// The toolchain root that holds the newest finished version.
    pub root: PathBuf,
    /// That version.
    pub version: String,
}

/// What:
/// The newest finished version of the tool across the scope's root and the other scope's root,
/// with the root that holds it,
/// or `None` when neither holds a finished one.
/// Why:
/// A tie goes to the scope's own root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function winningCopy(scope: Scope, tool: LockedTool): { root: string; version: string } | undefined
/// ```
pub fn winning_copy(scope: &Scope, tool: &LockedTool) -> Option<WinningCopy> {
    let own: Option<String> = newest_installed_of(&scope.toolchain_root, tool);
    let other: Option<String> = newest_installed_of(&scope.other_toolchain_root, tool);
    return match (own, other) {
        (Some(mine), Some(theirs)) => {
            if compare_versions(&theirs, &mine) == Ordering::Greater {
                Some(WinningCopy { root: scope.other_toolchain_root.clone(), version: theirs })
            } else {
                Some(WinningCopy { root: scope.toolchain_root.clone(), version: mine })
            }
        }
        (Some(mine), None) => Some(WinningCopy { root: scope.toolchain_root.clone(), version: mine }),
        (None, Some(theirs)) => Some(WinningCopy { root: scope.other_toolchain_root.clone(), version: theirs }),
        (None, None) => None,
    };
}

/// The root whose `current` link the tool runs through:
/// the winning copy's root,
/// or the scope's own when nothing is installed yet.
pub fn winning_root(scope: &Scope, tool: &LockedTool) -> PathBuf {
    return match winning_copy(scope, tool) {
        Some(winner) => winner.root,
        None => scope.toolchain_root.clone(),
    };
}

/// The version that runs.
pub fn winning_version(scope: &Scope, tool: &LockedTool) -> Option<String> {
    return match winning_copy(scope, tool) {
        Some(winner) => Some(winner.version),
        None => None,
    };
}

/// What:
/// Whether the other toolchain root's `current` link of a tool resolves into `directory`,
/// a version directory of this root:
/// the other root forwards to it (see `managed_losers.rs`),
/// so deleting it would leave that root's launchers and links dangling.
/// Why:
/// `prune --toolchain` reads only this scope's locks;
/// the other scope's lock may be the one that needs the version,
/// and the forward is the evidence that the other scope runs it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function otherRootForwardsTo(otherCurrent: string, directory: string): boolean
/// ```
pub fn other_root_forwards_to(other_current: &Path, directory: &Path) -> bool {
    let (Ok(resolved), Ok(target)) = (fs::canonicalize(other_current), fs::canonicalize(directory)) else {
        return false;
    };
    return resolved.starts_with(target);
}
