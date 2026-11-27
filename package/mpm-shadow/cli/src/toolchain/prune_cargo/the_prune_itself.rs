//! What:
//! The cleanup itself:
//! the scan of the versions installed now,
//! the deletion of every version no known lock names,
//! the link that names the newest version this run keeps,
//! and the same rule for a release archive of this repository.
//! Why:
//! Version-keyed installs let trees with different locks coexist,
//! so nothing removes a version when the last lock naming it moves on;
//! and a link is settled before anything is deleted,
//! so a run killed at any later point leaves a link that resolves or none at all.
//! A caller imports from `prune_cargo` and never from this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function pruneToolchain(scope: Scope, runner: Runner, extraLocks: string[], dryRun: boolean): string[];
//! function pruneArchives(scope: Scope, named: Map<string, NamedArchive>, dryRun: boolean, removed: string[]): void;
//! function removeCurrentLink(toolchainRoot: string, crateName: string, removals: [string, string, string][], unlinked: string[]): void;
//! function removeCrateDirectory(crateDir: string): void;
//! ```

/// Imports the sorted collection the plan of deletions is held in.
use std::collections::BTreeMap;
/// Imports `fs` from `std`.
use std::fs;
/// Imports `ErrorKind` from `std::io`.
use std::io::ErrorKind;
/// Imports what this file uses from `std::path`.
use std::path::{
    Path,
    PathBuf,
};

/// Imports the check that the other toolchain root runs the version about to be deleted.
use super::super::across_roots::other_root_forwards_to;
/// Imports the cargo namespace,
/// the link that names the version that runs,
/// and the writes to it.
use super::super::cargo_installs::{
    CARGO_NAMESPACE,
    current_link,
    point_current_at,
    running_version,
};
/// Imports `ToolchainError` from `super::super::error`.
use super::super::error::ToolchainError;
/// Imports the install directory of one tool version.
use super::super::layout::tool_install_dir;
/// Imports the release-archive tools a lock names,
/// their removal plan and their link.
use super::super::published_archive::{
    NamedArchive,
    plan_archive_prune,
    point_current_at_newest_archive,
    present_archive_versions,
};
/// Imports `Runner` from `super::super::runner`.
use super::super::runner::Runner;
/// Imports what this file uses from `super::super::scope`.
use super::super::scope::{
    Scope,
    ScopeKind,
};
/// Imports `worktree_lock_files` from `super::super::worktrees`.
use super::super::worktrees::worktree_lock_files;
/// Imports the prefix that marks a crate this repository publishes.
use crate::constants::OWN_CRATE_PREFIX;

/// Imports the rule that picks the versions to delete.
use super::the_rule_that_decides_what_goes::{
    PresentVersion,
    newest_finished,
    plan_prune,
    stays,
};
/// Imports every version directory the scan found.
use super::the_versions_on_disk::present_versions;
/// Imports the words a failed run leaves behind.
use super::what_a_failed_run_says_it_removed::{
    after_removing,
    partly_removed,
};
/// Imports what the known locks name.
use super::what_the_known_locks_name::named_by_all;
/// What:
/// Deletes (or,
/// with `dry_run`,
/// lists) the versions of repository-published release-archive tools that are older than the newest finished one,
/// and points each tool's `current` link at that version.
/// Why:
/// Older repository-published tools are never kept,
/// whichever route installed them.
/// Only tools some known lock names are judged:
/// an archive directory nothing names cannot be told from a third-party tool's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pruneArchives(scope: Scope, named: Map<string, NamedArchive>, dryRun: boolean): string[]
/// ```
fn prune_archives(
    scope: &Scope,
    named: &BTreeMap<String, NamedArchive>,
    dry_run: bool,
    removed: &mut Vec<PathBuf>,
) -> Result<(), ToolchainError> {
    let present = present_archive_versions(&scope.toolchain_root, named);
    if !dry_run {
        // What: every named archive tool's `current` is repointed before anything is deleted.
        // Why: it names the newest finished version, which is never one of the deletions below,
        // so moving the link first leaves a run killed at any later point with a link that resolves.
        for (id, archive) in named {
            point_current_at_newest_archive(&scope.toolchain_root, id, &archive.first_bin)
                .map_err(|error| return after_removing(error, removed))?;
        }
    }
    for (id, version) in plan_archive_prune(&present) {
        let directory: PathBuf = tool_install_dir(&scope.toolchain_root, &id, &version)
            .map_err(|error| return after_removing(error, removed))?;
        if !dry_run && let Err(error) = fs::remove_dir_all(&directory) {
            return Err(partly_removed(&directory, &error, removed));
        }
        removed.push(directory);
    }
    return Ok(());
}

/// What:
/// Deletes (or,
/// with `dry_run`,
/// lists) the version directories no known lock names.
/// Why:
/// This is the cleanup the version-keyed layout needs.
/// It runs only when asked:
/// `install` never deletes another tree's version.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function pruneToolchain(scope: Scope, runner: Runner, extraLocks: string[], dryRun: boolean): string[]
/// ```
pub fn prune_toolchain(
    scope: &Scope,
    runner: &dyn Runner,
    extra_locks: &[PathBuf],
    dry_run: bool,
) -> Result<Vec<PathBuf>, ToolchainError> {
    // What: the versions installed now are listed before any lock is read.
    // Why: a version that finishes installing after this point is not in this list, so this run cannot delete it.
    // Read after the locks, such a version would look like one no lock names, which is exactly what gets deleted.
    let present = present_versions(&scope.toolchain_root)
        .map_err(|error| return after_removing(error, &[]))?;
    let mut lock_files: Vec<PathBuf> = vec![scope.lock_file.clone()];
    if scope.kind == ScopeKind::Repo {
        lock_files.extend(worktree_lock_files(runner, &scope.root)?);
    }
    let (named, named_archive_tools) = named_by_all(&lock_files, extra_locks)?;
    // What: the deletions this run will make, with the versions the other toolchain root forwards to left out.
    // Why: the link decision below has to know which versions really stay, and that forward is one way one stays.
    let mut planned: Vec<(String, String, PathBuf)> = Vec::new();
    for (crate_name, version) in plan_prune(&present, &named) {
        let directory: PathBuf = scope
            .toolchain_root
            .join(CARGO_NAMESPACE)
            .join(&crate_name)
            .join(&version);
        // A version the other toolchain root's `current` forwards to is the one that root runs: never pruned from here.
        if other_root_forwards_to(
            &current_link(&scope.other_toolchain_root, &crate_name),
            &directory,
        ) {
            continue;
        }
        planned.push((crate_name, version, directory));
    }
    if !dry_run {
        // What: every repository-published crate's `current` is settled before anything is deleted.
        // Why: a crate that keeps a version has the newest finished one, which is never one of the deletions
        // below, so the link is moved to it; a crate that keeps none loses the link,
        // because every version the link could name is about to go.
        // Either way a run killed at any later point leaves a link that resolves or none at all.
        // What: the links this pass removed, so a failure on a later crate says what already went.
        // Why: no version directory has been removed yet, and the failure must not claim that nothing was.
        let mut unlinked: Vec<PathBuf> = Vec::new();
        for crate_name in present.keys() {
            if !crate_name.starts_with(OWN_CRATE_PREFIX) || present[crate_name].is_empty() {
                // A crate with no version here holds only a link forwarding to the other toolchain root's newer
                // copy (see `managed_losers.rs`): nothing is pruned from it, so its link is not this run's to move.
                continue;
            }
            // What: the candidates are the versions of this crate the plan keeps, not every installed version.
            // Why: the newest installed one may be one this run deletes, because the other toolchain root forwards
            // to an older one instead (see `across_roots.rs`), and a prune that succeeds must not leave `current`
            // naming a directory it removed.
            let kept: Vec<&PresentVersion> = present[crate_name]
                .iter()
                .filter(|candidate| return stays(crate_name, candidate, &planned))
                .collect();
            // What: the versions of this crate the plan deletes, which the link must never be left on.
            // Why: a link naming one of them would dangle once the deletions run, so the guard that leaves a newer
            // link alone does not apply to them.
            let deleted_here: Vec<&str> = planned
                .iter()
                .filter(|(crate_of_removal, _version, _directory)| {
                    return crate_of_removal == crate_name;
                })
                .map(|(_crate, version, _directory)| return version.as_str())
                .collect();
            match newest_finished(&kept) {
                Some(newest) => {
                    // What: a repoint that fails is restated with the links this pass already removed.
                    // Why: the links are gone whether or not the new one lands, and the report is the only place a
                    // person learns it.
                    let pointed: bool =
                        point_current_at(&scope.toolchain_root, crate_name, &newest, &deleted_here)
                            .map_err(|error| return after_removing(error, &unlinked))?;
                    if !pointed {
                        // The version this run keeps is not there any more, so the link was left alone:
                        // it must then not go on naming a version this run deletes.
                        remove_current_link(
                            &scope.toolchain_root,
                            crate_name,
                            &planned,
                            &mut unlinked,
                        )?;
                    }
                }
                // What: with no finished version that stays, the link is kept only when what it names stays too.
                // Why: an unfinished install is not something `current` may name,
                // and a link left naming a version this run deletes dangles once the deletions run.
                None => {
                    remove_current_link(&scope.toolchain_root, crate_name, &planned, &mut unlinked)?
                }
            }
        }
    }
    let mut removed: Vec<PathBuf> = Vec::new();
    for (_crate_name, _version, directory) in planned {
        if !dry_run && let Err(error) = fs::remove_dir_all(&directory) {
            return Err(partly_removed(&directory, &error, &removed));
        }
        removed.push(directory);
    }
    if !dry_run {
        let after = present_versions(&scope.toolchain_root)
            .map_err(|error| return after_removing(error, &removed))?;
        for crate_name in present.keys() {
            let crate_dir: PathBuf = scope.toolchain_root.join(CARGO_NAMESPACE).join(crate_name);
            let remaining: bool = match after.get(crate_name) {
                Some(versions) => !versions.is_empty(),
                None => false,
            };
            let had_versions: bool = match present.get(crate_name) {
                Some(versions) => !versions.is_empty(),
                None => false,
            };
            // Only a crate that held versions and has none left is removed with its directory:
            // a crate that keeps one already has its `current` link repointed above,
            // and a directory that held no version before holds only a `current` link forwarding to the other
            // toolchain root's newer copy (see `managed_losers.rs`), which this run never pruned.
            // `remove_dir` is what keeps a version installed after the scan above: an empty directory goes,
            // and a directory that is no longer empty stays with whatever another run put there.
            if !remaining && had_versions {
                remove_crate_directory(&crate_dir)
                    .map_err(|error| return after_removing(error, &removed))?;
            }
        }
    }
    prune_archives(scope, &named_archive_tools, dry_run, &mut removed)?;
    return Ok(removed);
}

/// What:
/// Removes a crate's `current` link when it names a version this run deletes or one that is already gone,
/// and records the link in `unlinked`.
/// Why:
/// When no version of the crate remains,
/// the link would name a directory this run deletes;
/// removing it first leaves a run killed at any later point with no link rather than a dangling one.
/// A link that names a version which still exists and is not one of the deletions belongs to another
/// command's install,
/// so it is left alone.
/// The check is one `read_link` and one `metadata`:
/// no lock,
/// no wait,
/// and the instant between the check and the unlink is the one window left.
/// This pass runs before any version directory is removed,
/// so `unlinked` is everything the run has deleted
/// when a later crate's link cannot be removed.
fn remove_current_link(
    toolchain_root: &Path,
    crate_name: &str,
    removals: &[(String, String, PathBuf)],
    unlinked: &mut Vec<PathBuf>,
) -> Result<(), ToolchainError> {
    let link: PathBuf = current_link(toolchain_root, crate_name);
    let named: String = match running_version(toolchain_root, crate_name) {
        Some(named) => named,
        // No link to read, or one whose name this run cannot take: nothing here is this run's to remove.
        None => return Ok(()),
    };
    let going: bool = removals
        .iter()
        .any(|(crate_of_removal, version, _directory)| {
            return crate_of_removal == crate_name && version == &named;
        });
    if !going && fs::metadata(&link).is_ok() {
        return Ok(());
    }
    return match fs::remove_file(&link) {
        Ok(()) => {
            unlinked.push(link);
            Ok(())
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(after_removing(
            ToolchainError::failure(format!("cannot remove {}: {error}", link.display())),
            unlinked,
        )),
    };
}

/// What:
/// Removes a crate directory when it is empty.
/// Why:
/// `remove_dir_all` would carry away a version another run installed after the scan that decided this crate
/// holds none,
/// and that version may be one a lock names.
/// A directory that is no longer empty is another run's work,
/// so it stays and this is not a failure.
fn remove_crate_directory(crate_dir: &Path) -> Result<(), ToolchainError> {
    return match fs::remove_dir(crate_dir) {
        Ok(()) => Ok(()),
        // `ENOTEMPTY`: something arrived after the scan, and the directory is not this run's to remove.
        Err(error) if error.kind() == ErrorKind::DirectoryNotEmpty => Ok(()),
        // Already gone, so there is nothing left to remove.
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(ToolchainError::failure(format!(
            "cannot remove {}: {error}",
            crate_dir.display()
        ))),
    };
}
