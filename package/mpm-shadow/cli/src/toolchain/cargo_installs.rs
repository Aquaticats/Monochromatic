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
//! Three answers are files of their own,
//! named for what they hold:
//! which tools count as this repository's own (`a_tool_this_repository_publishes`),
//! the sentence about a lock that pins an older version (`the_older_lock_sentence`),
//! and the version that runs,
//! with the `current` link that names it (`which_version_runs`).
//! A caller imports every name from this file and never from a part.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // cargo-installs.ts
//! export function newestInstalled(root: string, crate: string): string | undefined
//! export * from './cargo_installs/a_tool_this_repository_publishes.ts';
//! export * from './cargo_installs/the_older_lock_sentence.ts';
//! export * from './cargo_installs/which_version_runs.ts';
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
use std::path::{
    Path,
    PathBuf,
};

/// Imports `OWN_CRATE_PREFIX` from `crate::constants`.
use crate::constants::OWN_CRATE_PREFIX;
/// Imports `installed_versions` from `crate::installed`.
use crate::installed::installed_versions;
/// Imports `compare_versions` from `crate::version`.
use crate::version::compare_versions;

/// Imports what this module uses from `super::across_roots`.
use super::across_roots::{
    winning_root,
    winning_version,
};
/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports what this module uses from `super::layout`.
use super::layout::{
    install_dir_name,
    tool_install_dir,
};
/// Imports `LockedTool` from `super::model`.
use super::model::LockedTool;
/// Imports `package_name` from `super::observe`.
use super::observe::package_name;
/// Imports what this module uses from `super::published_archive`.
use super::published_archive::{
    first_bin_of_tool,
    is_finished,
    is_published_archive,
    version_names as archive_version_names,
};
/// Imports `Scope` from `super::scope`.
use super::scope::Scope;

/// Which tools count as this repository's own.
mod a_tool_this_repository_publishes;
/// The sentence about a lock that pins an older version,
/// and the shell quoting of a tool id.
mod the_older_lock_sentence;
/// The version that runs,
/// and the `current` link that names it.
mod which_version_runs;

/// Re-exports the two predicates that answer whether a tool is this repository's own.
pub use self::a_tool_this_repository_publishes::{
    is_own_crate_name,
    is_repository_published,
};
/// Re-exports the older-lock sentence,
/// the shell word it carries and the warning lines built from both.
pub use self::the_older_lock_sentence::{
    is_newer_than_locked,
    older_lock_message,
    older_lock_message_for,
    older_lock_warnings,
    shell_word,
};
/// Re-exports the version that runs,
/// the newest finished version of a tool and the settling of `current`.
pub use self::which_version_runs::{
    newest_installed_of,
    point_current_at,
    point_current_at_newest,
    point_current_at_newest_of,
    running_version,
};

/// The directory below the toolchain root that holds every `cargo-binstall` install root.
pub const CARGO_NAMESPACE: &str = "cargo";

/// The link inside a repository-published crate's directory that names the version every tree runs.
pub const CURRENT_LINK_NAME: &str = "current";

/// `<toolchain root>/cargo/<crate>`.
pub fn crate_directory(
    toolchain_root: &Path,
    crate_name: &str,
) -> PathBuf {
    return toolchain_root
        .join(CARGO_NAMESPACE)
        .join(install_dir_name(crate_name));
}

/// `<toolchain root>/cargo/<crate>/<version>`,
/// the cargo install root of one version.
pub fn version_directory(
    toolchain_root: &Path,
    crate_name: &str,
    version: &str,
) -> PathBuf {
    return crate_directory(toolchain_root, crate_name).join(version);
}

/// `<toolchain root>/cargo/<crate>/current`.
pub fn current_link(
    toolchain_root: &Path,
    crate_name: &str,
) -> PathBuf {
    return crate_directory(toolchain_root, crate_name).join(CURRENT_LINK_NAME);
}

/// The version cargo's own record inside `version_dir` names for the crate,
/// or `None` when the install never finished.
pub fn recorded_version(
    version_dir: &Path,
    crate_name: &str,
) -> Option<String> {
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
fn newest_first<T: AsRef<str>>(
    left: &T,
    right: &T,
) -> Ordering {
    return compare_versions(right.as_ref(), left.as_ref());
}

/// Every version directory of the crate (a plain directory that is not `current`,
/// not hidden),
/// newest first.
pub fn version_names(
    toolchain_root: &Path,
    crate_name: &str,
) -> Vec<String> {
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
pub fn newest_installed(
    toolchain_root: &Path,
    crate_name: &str,
) -> Option<String> {
    for name in version_names(toolchain_root, crate_name) {
        if recorded_version(
            &version_directory(toolchain_root, crate_name, &name),
            crate_name,
        )
        .is_some()
        {
            return Some(name);
        }
    }
    return None;
}

/// What:
/// Every finished version of a repository-published tool under one toolchain root,
/// newest first,
/// whichever route installed it.
/// Why:
/// The cleanup of older managed copies in the other root lists these.
/// A directory cargo never recorded,
/// or an archive without its completion marker,
/// is unfinished and not listed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function finishedVersionsOf(root: string, tool: LockedTool): string[]
/// ```
pub fn finished_versions_of(
    toolchain_root: &Path,
    tool: &LockedTool,
) -> Vec<String> {
    let mut finished: Vec<String> = Vec::new();
    if is_published_archive(tool) {
        let id: &str = &tool.declaration.id;
        for name in archive_version_names(toolchain_root, id) {
            if let Ok(directory) = tool_install_dir(toolchain_root, id, &name)
                && is_finished(&directory, first_bin_of_tool(tool))
            {
                finished.push(name);
            }
        }
        return finished;
    }
    let crate_name: String = package_name(tool);
    for name in version_names(toolchain_root, &crate_name) {
        if recorded_version(
            &version_directory(toolchain_root, &crate_name, &name),
            &crate_name,
        )
        .is_some()
        {
            finished.push(name);
        }
    }
    return finished;
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
pub fn remove_older_than_newest(
    toolchain_root: &Path,
    crate_name: &str,
) -> Result<Vec<PathBuf>, ToolchainError> {
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
            return Err(ToolchainError::failure(format!(
                "cannot remove the superseded {}: {error}",
                directory.display()
            )));
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
pub fn cargo_bin_dir_of(
    tool: &LockedTool,
    scope: &Scope,
) -> PathBuf {
    let crate_name: String = package_name(tool);
    if is_repository_published(tool) {
        return current_link(&winning_root(scope, tool), &crate_name).join("bin");
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
pub fn observed_cargo_version(
    scope: &Scope,
    tool: &LockedTool,
) -> Option<String> {
    let crate_name: String = package_name(tool);
    if is_repository_published(tool) {
        return winning_version(scope, tool);
    }
    return recorded_version(
        &version_directory(&scope.toolchain_root, &crate_name, &tool.version),
        &crate_name,
    );
}
