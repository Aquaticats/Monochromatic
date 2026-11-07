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

/// Imports what this module uses from `super::across_roots`.
use super::across_roots::{winning_root, winning_version};
/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports `atomic_symlink` from `super::install_support`.
use super::install_support::atomic_symlink;
/// Imports what this module uses from `super::layout`.
use super::layout::{applies_to_family, install_dir_name, tool_install_dir};
/// Imports what this module uses from `super::model`.
use super::model::{LockedTool, OsFamily, Route};
/// Imports `package_name` from `super::observe`.
use super::observe::package_name;
/// Imports what this module uses from `super::published_archive`.
use super::published_archive::{
    first_bin_of_tool, is_finished, is_published_archive, newest_installed_archive, point_current_at_newest_archive,
    version_names as archive_version_names,
};
/// Imports what this module uses from `super::scope`.
use super::scope::{Scope, ScopeKind};
/// Imports `file_name_text` from `super::util`.
use super::util::file_name_text;

/// The directory below the toolchain root that holds every `cargo-binstall` install root.
pub const CARGO_NAMESPACE: &str = "cargo";

/// The link inside a repository-published crate's directory that names the version every tree runs.
pub const CURRENT_LINK_NAME: &str = "current";

/// What:
/// Whether a tool is published by this repository:
/// a `cargo-binstall` tool whose crate name starts with `monochromatic-`,
/// or an `archive` tool of the request `github:Aquaticats/Monochromatic` (in any letter case)
/// whose `options.tagPrefix` starts with it
/// (a release archive of this repository,
/// such as `mpm-shadow` itself).
/// Why:
/// This repository names every crate it publishes that way (`OWN_CRATE_PREFIX`,
/// the same rule that limits the registry prune to its own crates) and tags every release with the same start,
/// so the plan needs no second list that could drift from the crates.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function isRepositoryPublished(tool: LockedTool): boolean
/// ```
pub fn is_repository_published(tool: &LockedTool) -> bool {
    if is_published_archive(tool) {
        return true;
    }
    return tool.declaration.route == Route::CargoBinstall && is_own_crate_name(&package_name(tool));
}

/// What:
/// Whether a crate name is in this repository's crate namespace:
/// it starts with `OWN_CRATE_PREFIX` once letter case and `_` versus `-` are ignored.
/// Why:
/// crates.io treats `Monochromatic_X` and `monochromatic-x` as one crate,
/// so a spelling variant of a repository crate must not be taken for a third-party one
/// (a third-party crate may fall back to compiling;
/// a repository crate never does).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function isOwnCrateName(name: string): boolean
/// ```
pub fn is_own_crate_name(name: &str) -> bool {
    let canonical: String = name.to_ascii_lowercase().replace('_', "-");
    return canonical.starts_with(OWN_CRATE_PREFIX);
}

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
pub fn newest_installed_of(toolchain_root: &Path, tool: &LockedTool) -> Option<String> {
    if is_published_archive(tool) {
        return newest_installed_archive(toolchain_root, &tool.declaration.id, first_bin_of_tool(tool));
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
pub fn point_current_at_newest_of(toolchain_root: &Path, tool: &LockedTool) -> Result<Option<String>, ToolchainError> {
    if is_published_archive(tool) {
        return point_current_at_newest_archive(toolchain_root, &tool.declaration.id, first_bin_of_tool(tool));
    }
    return point_current_at_newest(toolchain_root, &package_name(tool));
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
pub fn version_names(toolchain_root: &Path, crate_name: &str) -> Vec<String> {
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
pub fn finished_versions_of(toolchain_root: &Path, tool: &LockedTool) -> Vec<String> {
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
        if recorded_version(&version_directory(toolchain_root, &crate_name, &name), &crate_name).is_some() {
            finished.push(name);
        }
    }
    return finished;
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
pub fn observed_cargo_version(scope: &Scope, tool: &LockedTool) -> Option<String> {
    let crate_name: String = package_name(tool);
    if is_repository_published(tool) {
        return winning_version(scope, tool);
    }
    return recorded_version(&version_directory(&scope.toolchain_root, &crate_name, &tool.version), &crate_name);
}

/// Whether a character may appear in a word that needs no quoting.
fn is_plain_word_character(character: char) -> bool {
    return character.is_ascii_alphanumeric() || "@%+=:,./_-".contains(character);
}

/// What:
/// `shell_word(word)` gives `word` as one word that a POSIX shell reads back unchanged:
/// bare when every character is an ASCII letter,
/// a digit or one of `@%+=:,./_-`,
/// otherwise inside single quotes,
/// with each single quote of the word written as `'\''` (close the quotes,
/// an escaped quote,
/// reopen them).
/// An empty word becomes `''`.
/// Why:
/// The older-lock hints print a command for the person to paste into a shell;
/// a tool id holding a space,
/// a quote,
/// a `;` or a `$` must still name exactly that tool and run nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function shellWord(word: string): string {
///   return /^[A-Za-z0-9@%+=:,.\/_-]+$/.test(word) ? word : `'${word.replaceAll("'", "'\\''")}'`;
/// }
/// ```
pub fn shell_word(word: &str) -> String {
    // What: `word.chars()` walks the text one Unicode character at a time;
    //  `.all(|character| { ... })` is true only when the closure `|character| { ... }` (TS: `(character) => { ... }`)
    //  returns true for every character.
    //  `is_ascii_alphanumeric` is true for `A-Z`, `a-z` and `0-9` only.
    // Why: such a word needs no quoting, and leaving it bare keeps the common hint (`cargo:monochromatic-...`) readable.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const plain = word !== '' && [...word].every((character) => /[A-Za-z0-9]/.test(character) || '@%+=:,./_-'.includes(character));
    // ```
    let plain: bool = !word.is_empty()
        && word.chars().all(is_plain_word_character);
    if plain {
        // What: `.to_string()` copies the borrowed text `&str` into a new owned `String` (TS strings need no such step).
        // Why: the function hands back text the caller keeps after `word` is gone.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return word;
        // ```
        return word.to_string();
    }
    // What: `word.replace('\'', "'\\''")` swaps every single quote (the char literal `'\''`) for the four characters
    //  `'\''`; `format!("'{}'", ...)` wraps the result in one more pair of single quotes.
    // Why: inside single quotes a shell treats every character literally except the closing quote itself,
    //  so only the quotes need this close-escape-reopen form.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return `'${word.replaceAll("'", "'\\''")}'`;
    // ```
    return format!("'{}'", word.replace('\'', "'\\''"));
}

/// What:
/// The sentence that says a scope runs a newer repository-published tool than its lock pins,
/// with the remedy for that scope as mpm's own pnpm-style verb naming the tool:
/// `mpm update <tool id>` for a tree's lock,
/// `mpm update -g <tool id>` for the machine-global lock,
/// the id written as one shell word (`shell_word`).
/// Why:
/// One text for `install`,
/// `list`,
/// `update`,
/// `install --lockfile-only`,
/// `bin` and every run of the tool,
/// so a person searching for it finds all of them.
/// The remedy names the tool so it moves only that row of the lock,
/// and names `mpm` rather than a mise task because the repository is moving away from mise
/// and `mpm update -g` works from any directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function olderLockMessageFor(kind: 'repo' | 'global', toolId: string, locked: string, used: string): string
/// ```
pub fn older_lock_message_for(kind: ScopeKind, tool_id: &str, locked: &str, used: &str) -> String {
    // The tool id as one word the person can paste into a shell.
    let word: String = shell_word(tool_id);
    return match kind {
        ScopeKind::Repo => format!(
            "{tool_id} is locked at {locked} in this tree but {used} runs, because older repository tools are never kept \
             once a newer one is installed; upgrade this tree's lock with `mpm update {word}` or merge main"
        ),
        ScopeKind::Global => format!(
            "{tool_id} is locked at {locked} in this machine's global lock but {used} runs, because older repository tools are \
             never kept once a newer one is installed; upgrade the global lock with `mpm update -g {word}`"
        ),
    };
}

/// The older-lock sentence for a tree's lock (the repository scope).
pub fn older_lock_message(tool_id: &str, locked: &str, used: &str) -> String {
    return older_lock_message_for(ScopeKind::Repo, tool_id, locked, used);
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
        let Some(used) = winning_version(scope, tool) else {
            continue;
        };
        if is_newer_than_locked(&used, &tool.version) {
            warnings.push(older_lock_message_for(scope.kind, &tool.declaration.id, &tool.version, &used));
        }
    }
    return warnings;
}
