//! What:
//! Where `cargo-binstall` tools live and how their versions are chosen:
//! one directory per crate and version under the toolchain root,
//! `<toolchain root>/cargo/<crate>/<version>`,
//! each a cargo install root of its own (`bin/` plus cargo's own `.crates.toml` record).
//! Why:
//! The shared `$CARGO_HOME/bin` let a tree whose lock pinned an older wrapper overwrite the one binary every tree ran
//! (`doc/troubleshooting/toolchain-sync-older-lock-downgrades-shared-cargo-installs.md`).
//! Third-party crates coexist per version;
//! crates this repository publishes keep only the newest installed version,
//! reached through a `current` link inside the crate directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // cargo-installs.ts
//! export function isRepositoryPublished(tool: LockedTool): boolean
//! export function newestInstalled(root: string, crate: string): string | undefined
//! export function pointCurrentAtNewest(root: string, crate: string): string | undefined
//! export function olderLockWarnings(scope: Scope, tools: LockedTool[]): string[]
//! ```

/// Imports `Ordering` from `std::cmp`.
use std::cmp::Ordering;
/// Imports `BTreeMap`,
/// a map sorted by key,
/// from `std::collections`.
use std::collections::BTreeMap;
/// Imports `fs` from `std`.
use std::fs;
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};

/// Imports `OWN_CRATE_PREFIX` from `crate::constants`.
use crate::constants::OWN_CRATE_PREFIX;
/// Imports `installed_versions` from `crate::installed`.
use crate::installed::installed_versions;
/// Imports `compare_versions` from `crate::version`.
use crate::version::compare_versions;

/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports `atomic_symlink` from `super::install_support`.
use super::install_support::atomic_symlink;
/// Imports what this module uses from `super::layout`.
use super::layout::{applies_to_family, install_dir_name};
/// Imports what this module uses from `super::model`.
use super::model::{LockedTool, OsFamily, Route};
/// Imports `package_name` from `super::observe`.
use super::observe::package_name;
/// Imports `Scope` from `super::scope`.
use super::scope::Scope;
/// Imports `file_name_text` from `super::util`.
use super::util::file_name_text;

/// The directory below the toolchain root that holds every `cargo-binstall` install root.
pub const CARGO_NAMESPACE: &str = "cargo";

/// The link inside a repository-published crate's directory that names the version every tree runs.
pub const CURRENT_LINK_NAME: &str = "current";

/// What:
/// Whether a tool is published by this repository:
/// a `cargo-binstall` tool whose crate name starts with `monochromatic-`.
/// Why:
/// This repository names every crate it publishes that way (`OWN_CRATE_PREFIX`,
/// the same rule that limits the registry prune to its own crates),
/// so the plan needs no second list that could drift from the crates.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function isRepositoryPublished(tool: LockedTool): boolean
/// ```
pub fn is_repository_published(tool: &LockedTool) -> bool {
    return tool.declaration.route == Route::CargoBinstall && package_name(tool).starts_with(OWN_CRATE_PREFIX);
}

/// `<toolchain root>/cargo/<crate>`.
pub fn crate_directory(toolchain_root: &Path, crate_name: &str) -> PathBuf {
    return toolchain_root.join(CARGO_NAMESPACE).join(install_dir_name(crate_name));
}

/// `<toolchain root>/cargo/<crate>/<version>`,
/// the cargo install root of one version.
pub fn version_directory(toolchain_root: &Path, crate_name: &str, version: &str) -> PathBuf {
    return crate_directory(toolchain_root, crate_name).join(version);
}

/// `<toolchain root>/cargo/<crate>/current`.
pub fn current_link(toolchain_root: &Path, crate_name: &str) -> PathBuf {
    return crate_directory(toolchain_root, crate_name).join(CURRENT_LINK_NAME);
}

/// The version cargo's own record inside `version_dir` names for the crate,
/// or `None` when the install never finished.
pub fn recorded_version(version_dir: &Path, crate_name: &str) -> Option<String> {
    return installed_versions(version_dir).get(crate_name).cloned();
}

/// What:
/// Orders two version names newest first.
/// Why:
/// `sort_by` takes a function of two references;
/// it is generic over the text type so the same function serves `Vec<String>` without taking `&String` parameters.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const newestFirst = (left: string, right: string): number => compareVersions(right, left);
/// ```
fn newest_first<T: AsRef<str>>(left: &T, right: &T) -> Ordering {
    return compare_versions(right.as_ref(), left.as_ref());
}

/// Every version directory of the crate (a plain directory that is not `current`,
/// not hidden),
/// newest first.
fn version_names(toolchain_root: &Path, crate_name: &str) -> Vec<String> {
    let Ok(entries) = fs::read_dir(crate_directory(toolchain_root, crate_name)) else {
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

/// What:
/// The newest finished version of every repository-published crate installed under one toolchain root,
/// by crate name.
/// Why:
/// The registry prune keeps the version that is installed;
/// for these crates that is read from the version-keyed records under `<toolchain root>/cargo/<crate>/`,
/// not from `$CARGO_HOME/.crates.toml`,
/// which `install` no longer writes.
/// A directory without a record for its crate is an unfinished install and is ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function repositoryVersionsInstalled(toolchainRoot: string): Map<string, string>
/// ```
pub fn repository_versions_installed(toolchain_root: &Path) -> BTreeMap<String, String> {
    let mut versions: BTreeMap<String, String> = BTreeMap::new();
    let Ok(entries) = fs::read_dir(toolchain_root.join(CARGO_NAMESPACE)) else {
        return versions;
    };
    for entry in entries.flatten() {
        let name: String = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with(OWN_CRATE_PREFIX) {
            continue;
        }
        if let Some(newest) = newest_installed(toolchain_root, &name) {
            versions.insert(name, newest);
        }
    }
    return versions;
}

/// The newest version of the crate whose install finished (cargo's record names it).
pub fn newest_installed(toolchain_root: &Path, crate_name: &str) -> Option<String> {
    for name in version_names(toolchain_root, crate_name) {
        if recorded_version(&version_directory(toolchain_root, crate_name, &name), crate_name).is_some() {
            return Some(name);
        }
    }
    return None;
}

/// The version the crate's `current` link names (the link's target is the version directory's name),
/// or `None` when there is no link.
pub fn running_version(toolchain_root: &Path, crate_name: &str) -> Option<String> {
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
pub fn point_current_at_newest(toolchain_root: &Path, crate_name: &str) -> Result<Option<String>, ToolchainError> {
    let Some(newest) = newest_installed(toolchain_root, crate_name) else {
        return Ok(None);
    };
    if running_version(toolchain_root, crate_name).as_deref() != Some(newest.as_str()) {
        atomic_symlink(&current_link(toolchain_root, crate_name), Path::new(&newest))?;
    }
    return Ok(Some(newest));
}

/// What:
/// Deletes every version directory of the crate that is older than the newest finished one,
/// and returns the paths deleted.
/// Why:
/// A repository-published tool is never kept in an older version once a newer one is installed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function removeOlderThanNewest(root: string, crate: string): string[]
/// ```
pub fn remove_older_than_newest(toolchain_root: &Path, crate_name: &str) -> Result<Vec<PathBuf>, ToolchainError> {
    let Some(newest) = newest_installed(toolchain_root, crate_name) else {
        return Ok(Vec::new());
    };
    let mut removed: Vec<PathBuf> = Vec::new();
    for name in version_names(toolchain_root, crate_name) {
        if compare_versions(&name, &newest) != Ordering::Less {
            continue;
        }
        let directory: PathBuf = version_directory(toolchain_root, crate_name, &name);
        if let Err(error) = fs::remove_dir_all(&directory) {
            return Err(ToolchainError::failure(format!("cannot remove the superseded {}: {error}", directory.display())));
        }
        removed.push(directory);
    }
    return Ok(removed);
}

/// What:
/// The directory a tool's binaries are looked for in:
/// `current/bin` for a repository-published tool (the newest installed),
/// otherwise the `bin` of the locked version's own directory.
/// Why:
/// Observers,
/// the exposure directory and postinstall links all ask here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function cargoBinDir(tool: LockedTool, scope: Scope): string
/// ```
pub fn cargo_bin_dir_of(tool: &LockedTool, scope: &Scope) -> PathBuf {
    let crate_name: String = package_name(tool);
    if is_repository_published(tool) {
        return current_link(&scope.toolchain_root, &crate_name).join("bin");
    }
    return version_directory(&scope.toolchain_root, &crate_name, &tool.version).join("bin");
}

/// What:
/// The installed version of a `cargo-binstall` tool as `list` and `install` read it:
/// the newest finished version for a repository-published tool,
/// the locked version when its own directory holds a finished install for any other.
/// Why:
/// The legacy `$CARGO_HOME/.crates.toml` is deliberately not read:
/// it belongs to the shared install this layout replaces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function observedCargoVersion(scope: Scope, tool: LockedTool): string | undefined
/// ```
pub fn observed_cargo_version(scope: &Scope, tool: &LockedTool) -> Option<String> {
    let crate_name: String = package_name(tool);
    if is_repository_published(tool) {
        return newest_installed(&scope.toolchain_root, &crate_name);
    }
    return recorded_version(&version_directory(&scope.toolchain_root, &crate_name, &tool.version), &crate_name);
}

/// What:
/// The sentence that says a tree runs a newer repository-published tool than its lock pins,
/// with the remedy.
/// Why:
/// One text for `install`,
/// `list`,
/// `update`,
/// `install --lockfile-only`,
/// `bin` and every run of the tool,
/// so a person searching for it finds all of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function olderLockMessage(toolId: string, locked: string, used: string): string
/// ```
pub fn older_lock_message(tool_id: &str, locked: &str, used: &str) -> String {
    return format!(
        "{tool_id} is locked at {locked} in this tree but {used} runs, because older repository tools are never kept \
         once a newer one is installed; upgrade this tree's lock with `mise run toolchain:update` or merge main"
    );
}

/// Whether `used` is newer than `locked`.
pub fn is_newer_than_locked(used: &str, locked: &str) -> bool {
    return compare_versions(used, locked) == Ordering::Greater;
}

/// What:
/// A warning for every repository-published tool of this host whose installed version is newer than the lock's.
/// Why:
/// Commands that resolve tools for a tree report what will run in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function olderLockWarnings(scope: Scope, tools: LockedTool[], family?: OsFamily): string[]
/// ```
pub fn older_lock_warnings(scope: &Scope, tools: &[LockedTool], family: Option<OsFamily>) -> Vec<String> {
    let mut warnings: Vec<String> = Vec::new();
    for tool in tools {
        if !is_repository_published(tool) || !applies_to_family(&tool.declaration.os, family) {
            continue;
        }
        let Some(used) = newest_installed(&scope.toolchain_root, &package_name(tool)) else {
            continue;
        };
        if is_newer_than_locked(&used, &tool.version) {
            warnings.push(older_lock_message(&tool.declaration.id, &tool.version, &used));
        }
    }
    return warnings;
}
