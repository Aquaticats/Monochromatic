//! What:
//! The cleanup step for `cargo-binstall` installs:
//! `mpm-shadow prune --toolchain` deletes the version directories under `<toolchain root>/cargo` that no known lock
//! names,
//! and for a crate this repository publishes keeps only the newest installed version;
//! the same holds for a release archive of this repository that a lock names (`published_archive.rs`).
//! Why:
//! Version-keyed installs let trees with different locks coexist,
//! so nothing removes a version when the last lock naming it moves on.
//! The known locks are this scope's own lock,
//! the lock of every worktree `git worktree list` reports for the repository,
//! and any file given with `--lock`;
//! the command refuses to delete anything when a listed worktree's lock cannot be read,
//! because an unreadable lock might name a version that is about to be deleted,
//! and a file given with `--lock` names versions or refuses the command:
//! it may not be absent, since a file that said what to keep cannot be read as keeping nothing.
//! A crate directory the scan cannot read refuses the command too:
//! reading it as empty would remove the whole directory, including a version a lock names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // prune-cargo.ts
//! export function planPrune(present: Map<string, Present[]>, named: Map<string, Set<string>>): Removal[]
//! export function namedVersions(lockText: string): Map<string, Set<string>>
//! export function pruneToolchain(scope: Scope, runner: Runner, extraLocks: string[], dryRun: boolean): string[]
//! ```

/// Imports the sorted collections the rule works on.
use std::collections::{BTreeMap, BTreeSet};
/// Imports `fs` from `std`.
use std::fs;
/// Imports `ErrorKind` from `std::io`.
use std::io::ErrorKind;
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};

/// Imports `ArgMatches` from `clap`.
use clap::ArgMatches;
/// Imports `Value` from `serde_json`.
use serde_json::Value;

/// Imports `OWN_CRATE_PREFIX` from `crate::constants`.
use crate::constants::OWN_CRATE_PREFIX;
/// Imports `compare_versions` from `crate::version`.
use crate::version::compare_versions;

/// Imports what this module uses from `super::cargo_installs`.
use super::across_roots::other_root_forwards_to;
/// Imports what this module uses from `super::cargo_installs`.
use super::cargo_installs::{CARGO_NAMESPACE, CURRENT_LINK_NAME, current_link, point_current_at, recorded_version, running_version};
/// Imports what this module uses from `super::published_archive`.
use super::published_archive::{
    NamedArchive, named_archives, plan_archive_prune, point_current_at_newest_archive, present_archive_versions,
};
/// Imports `tool_install_dir` from `super::layout`.
use super::layout::tool_install_dir;
/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports `worktree_lock_files` from `super::worktrees`.
use super::worktrees::worktree_lock_files;
/// Imports what this module uses from `super::runner`.
use super::runner::{ProcessRunner, Runner};
/// Imports what this module uses from `super::scope`.
use super::scope::{EnvMap, Scope, ScopeKind, resolve_scope};

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

/// The crate name a lock entry installs:
/// `packageId`,
/// else the origin of a `cargo:<crate>@<range>` request,
/// else the id without its `cargo:` prefix.
fn crate_of_entry(id: &str, entry: &Value) -> String {
    if let Some(package) = entry.get("packageId").and_then(Value::as_str) {
        return package.to_string();
    }
    if let Some(request) = entry.get("request").and_then(Value::as_str)
        && let Some(rest) = request.strip_prefix("cargo:")
    {
        return rest.split('@').next().unwrap_or(rest).to_string();
    }
    return id.strip_prefix("cargo:").unwrap_or(id).to_string();
}

/// What:
/// The `cargo-binstall` crates and versions one lock names,
/// read from its JSON text without decoding the rest of the model.
/// Why:
/// Other worktrees may sit on a commit whose lock is in an older format;
/// only `tools.<id>.version` and `route` are needed,
/// and both are part of the lock's frozen surface.
/// A `cargo-binstall` row without a `version` text is not such a surface:
/// it is refused rather than read as naming nothing,
/// because naming nothing is what lets the prune delete the crate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function namedVersions(lockText: string): Map<string, Set<string>>  // throws when not a lock
/// ```
pub fn named_versions(lock_text: &str, origin: &str) -> Result<BTreeMap<String, BTreeSet<String>>, ToolchainError> {
    let lock: Value = match serde_json::from_str(lock_text) {
        Ok(value) => value,
        Err(error) => return Err(ToolchainError::failure(format!("{origin} is not JSON: {error}"))),
    };
    let Some(tools) = lock.get("tools").and_then(Value::as_object) else {
        return Err(ToolchainError::failure(format!("{origin} has no `tools` object")));
    };
    let mut named: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (id, entry) in tools {
        if entry.get("route").and_then(Value::as_str) != Some("cargo-binstall") {
            continue;
        }
        let Some(version) = entry.get("version").and_then(Value::as_str) else {
            // A row this command cannot read would protect nothing, and this is a deletion command: refusing keeps
            // the crate it installs instead of reading the lock as if the row were not there.
            return Err(ToolchainError::failure(format!(
                "{origin} names a cargo-binstall tool without a version, so nothing was deleted; tools.{id} has no `version` text"
            )));
        };
        named.entry(crate_of_entry(id, entry)).or_default().insert(version.to_string());
    }
    return Ok(named);
}

/// What:
/// The cargo versions and the release-archive tools every lock the two lists give names.
/// Why:
/// The two lists differ in what an absent file means.
/// A lock `git worktree list` reported may be gone by the time it is read (another run moved it on),
/// so a missing one names none.
/// A lock the caller named with `--lock` may not be missing:
/// reading it as naming nothing would let this deletion command delete the version it names,
/// so the refusal is raised where the file is read,
/// never by a check that runs before the read,
/// which a concurrent rename can outrun.
pub(super) fn named_by_all(
    discovered: &[PathBuf],
    explicit: &[PathBuf],
) -> Result<(BTreeMap<String, BTreeSet<String>>, BTreeMap<String, NamedArchive>), ToolchainError> {
    let mut named: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut archives: BTreeMap<String, NamedArchive> = BTreeMap::new();
    let mut locks_to_read: Vec<(&PathBuf, bool)> = discovered.iter().map(|file| return (file, false)).collect();
    locks_to_read.extend(explicit.iter().map(|file| return (file, true)));
    for (file, named_by_the_caller) in locks_to_read {
        let text: String = match fs::read_to_string(file) {
            Ok(text) => text,
            Err(error) if error.kind() == ErrorKind::NotFound && !named_by_the_caller => continue,
            Err(error) if named_by_the_caller => {
                return Err(ToolchainError::failure(format!(
                    "the --lock file {} cannot be read: {error}; nothing was deleted",
                    file.display()
                )));
            }
            Err(error) => {
                return Err(ToolchainError::failure(format!("cannot read the lock {}: {error}; nothing was deleted", file.display())));
            }
        };
        for (crate_name, versions) in named_versions(&text, &file.display().to_string())? {
            named.entry(crate_name).or_default().extend(versions);
        }
        for (id, archive) in named_archives(&text, &file.display().to_string())? {
            let slot: &mut NamedArchive = archives.entry(id).or_default();
            slot.versions.extend(archive.versions);
            slot.first_bin = archive.first_bin;
        }
    }
    return Ok((named, archives));
}

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
/// Every crate and version directory under `<toolchain root>/cargo`,
/// or an error naming the first directory whose contents could not be read.
/// Why:
/// The prune deletes the versions no lock names,
/// and the scan it runs after those deletions decides whether a crate directory still holds one.
/// A directory this walk cannot read must therefore be reported and not left out:
/// left out, it reads as "this crate holds nothing" and the whole crate directory goes,
/// including a version a lock names.
/// A crate directory that is gone holds nothing, so its absence is not a failed read.
pub(super) fn present_versions(toolchain_root: &Path) -> Result<BTreeMap<String, Vec<PresentVersion>>, ToolchainError> {
    let mut present: BTreeMap<String, Vec<PresentVersion>> = BTreeMap::new();
    let crates_dir: PathBuf = toolchain_root.join(CARGO_NAMESPACE);
    let crates = match fs::read_dir(&crates_dir) {
        Ok(entries) => entries,
        // No cargo directory yet: nothing is installed under it, so nothing can be deleted from it.
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(present),
        Err(error) => return Err(cannot_walk(&crates_dir, &error)),
    };
    for listed_crate in crates {
        let crate_entry: fs::DirEntry = listed_crate.map_err(|error| return cannot_walk(&crates_dir, &error))?;
        let crate_name: String = crate_entry.file_name().to_string_lossy().to_string();
        let versions = match fs::read_dir(crate_entry.path()) {
            Ok(entries) => entries,
            // The crate directory is gone: it holds no version that could be deleted.
            Err(error) if error.kind() == ErrorKind::NotFound => continue,
            Err(error) => return Err(cannot_walk(&crate_entry.path(), &error)),
        };
        let mut found: Vec<PresentVersion> = Vec::new();
        for listed_version in versions {
            let version_entry: fs::DirEntry = listed_version.map_err(|error| return cannot_walk(&crate_entry.path(), &error))?;
            let version: String = version_entry.file_name().to_string_lossy().to_string();
            let is_plain_directory: bool = version_entry
                .file_type()
                .map_err(|error| return cannot_walk(&version_entry.path(), &error))?
                .is_dir();
            if is_plain_directory && !version.starts_with('.') && version != CURRENT_LINK_NAME {
                let finished: bool = recorded_version(&version_entry.path(), &crate_name).is_some();
                found.push(PresentVersion { version, finished });
            }
        }
        present.insert(crate_name, found);
    }
    return Ok(present);
}

/// The refusal a scan raises for one directory whose contents it could not read.
fn cannot_walk(directory: &Path, error: &std::io::Error) -> ToolchainError {
    return ToolchainError::failure(format!("cannot read {}: {error}", directory.display()));
}

/// What:
/// `error`, restated with what this run has already removed from the tree.
/// Why:
/// The report is the only place a person learns what a failed run deleted,
/// so a failure after a removal must name what is gone and never say that nothing was deleted.
pub(super) fn after_removing(error: ToolchainError, removed: &[PathBuf]) -> ToolchainError {
    if removed.is_empty() {
        return ToolchainError { kind: error.kind, message: format!("{}; nothing was deleted", error.message) };
    }
    let message: String = format!("{}; already removed: {}", error.message, paths_named(removed));
    return ToolchainError { kind: error.kind, message };
}

/// What:
/// The failure of a recursive removal, which may already have unlinked part of the tree,
/// with what else this run removed.
/// Why:
/// `remove_dir_all` unlinks what it can before it stops,
/// and the directory never reaches the list of removed directories,
/// so the report must name it and say that part of it may be gone,
/// whatever that list holds.
fn partly_removed(directory: &Path, error: &std::io::Error, removed: &[PathBuf]) -> ToolchainError {
    let mut message: String = format!("cannot remove {}: {error}; part of it may be gone", directory.display());
    if !removed.is_empty() {
        message.push_str(&format!(", and already removed: {}", paths_named(removed)));
    }
    return ToolchainError::failure(message);
}

/// The removed directories as one list of paths, for a report.
fn paths_named(removed: &[PathBuf]) -> String {
    return removed.iter().map(|path| return path.display().to_string()).collect::<Vec<String>>().join(", ");
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
    let present = present_versions(&scope.toolchain_root).map_err(|error| return after_removing(error, &[]))?;
    let mut lock_files: Vec<PathBuf> = vec![scope.lock_file.clone()];
    if scope.kind == ScopeKind::Repo {
        lock_files.extend(worktree_lock_files(runner, &scope.root)?);
    }
    let (named, named_archive_tools) = named_by_all(&lock_files, extra_locks)?;
    // What: the deletions this run will make, with the versions the other toolchain root forwards to left out.
    // Why: the link decision below has to know which versions really stay, and that forward is one way one stays.
    let mut planned: Vec<(String, String, PathBuf)> = Vec::new();
    for (crate_name, version) in plan_prune(&present, &named) {
        let directory: PathBuf = scope.toolchain_root.join(CARGO_NAMESPACE).join(&crate_name).join(&version);
        // A version the other toolchain root's `current` forwards to is the one that root runs: never pruned from here.
        if other_root_forwards_to(&current_link(&scope.other_toolchain_root, &crate_name), &directory) {
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
            let kept: Vec<&PresentVersion> =
                present[crate_name].iter().filter(|candidate| return stays(crate_name, candidate, &planned)).collect();
            match newest_finished(&kept) {
                Some(newest) => point_current_at(&scope.toolchain_root, crate_name, &newest)?,
                // What: with no finished version that stays, the link is kept only when what it names stays too.
                // Why: an unfinished install is not something `current` may name,
                // and a link left naming a version this run deletes dangles once the deletions run.
                None => remove_current_link(&scope.toolchain_root, crate_name, &planned, &mut unlinked)?,
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
        let after = present_versions(&scope.toolchain_root).map_err(|error| return after_removing(error, &removed))?;
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
                remove_crate_directory(&crate_dir).map_err(|error| return after_removing(error, &removed))?;
            }
        }
    }
    prune_archives(scope, &named_archive_tools, dry_run, &mut removed)?;
    return Ok(removed);
}

/// What:
/// Whether this run keeps `candidate`, a version of `crate_name`, from `removals`.
/// Why:
/// The link decision counts only the versions that stay,
/// and a removal carries the crate it belongs to.
fn stays(crate_name: &str, candidate: &PresentVersion, removals: &[(String, String, PathBuf)]) -> bool {
    return !removals.iter().any(|(crate_of_removal, version, _directory)| {
        return crate_of_removal == crate_name && version == &candidate.version;
    });
}

/// What:
/// The newest finished version of `kept`, or `None` when none of them finished.
/// Why:
/// Only a finished version can be what `current` names: an unfinished one is still being written.
fn newest_finished(kept: &[&PresentVersion]) -> Option<String> {
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

/// What:
/// Removes a crate's `current` link when it names a version this run deletes or one that is already gone,
/// and records the link in `unlinked`.
/// Why:
/// When no version of the crate remains, the link would name a directory this run deletes;
/// removing it first leaves a run killed at any later point with no link rather than a dangling one.
/// A link that names a version which still exists and is not one of the deletions belongs to another
/// command's install, so it is left alone. The check is one `read_link` and one `metadata`: no lock, no wait,
/// and the instant between the check and the unlink is the one window left.
/// This pass runs before any version directory is removed, so `unlinked` is everything the run has deleted
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
    let going: bool = removals.iter().any(|(crate_of_removal, version, _directory)| {
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
/// holds none, and that version may be one a lock names.
/// A directory that is no longer empty is another run's work, so it stays and this is not a failure.
fn remove_crate_directory(crate_dir: &Path) -> Result<(), ToolchainError> {
    return match fs::remove_dir(crate_dir) {
        Ok(()) => Ok(()),
        // `ENOTEMPTY`: something arrived after the scan, and the directory is not this run's to remove.
        Err(error) if error.kind() == ErrorKind::DirectoryNotEmpty => Ok(()),
        // Already gone, so there is nothing left to remove.
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(ToolchainError::failure(format!("cannot remove {}: {error}", crate_dir.display()))),
    };
}

/// What:
/// `mpm-shadow prune --toolchain [--global] [--dry-run] [--lock <file>]...`:
/// runs `prune_toolchain` for the scope the working directory (or `--global`) selects and prints the result.
/// Why:
/// `cli.rs` hands the parsed arguments here,
/// so the command's wiring sits beside the rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runPruneCommand(matches: ParsedArgs): number
/// ```
pub fn run_prune_command(matches: &ArgMatches) -> i32 {
    let env: EnvMap = EnvMap::from_process();
    let start: PathBuf = match std::env::current_dir() {
        Ok(directory) => directory,
        Err(_) => PathBuf::from("."),
    };
    let scope: Scope = match resolve_scope(matches.get_flag("global"), &start, &env) {
        Ok(found) => found,
        Err(error) => {
            eprintln!("mpm-shadow: {error}");
            return error.exit_code();
        }
    };
    let mut extras: Vec<PathBuf> = Vec::new();
    if let Some(values) = matches.get_many::<String>("lock") {
        for value in values {
            extras.push(PathBuf::from(value));
        }
    }
    let dry_run: bool = matches.get_flag("dry-run");
    return match prune_toolchain(&scope, &ProcessRunner, &extras, dry_run) {
        Ok(removed) => {
            let verb: &str = if dry_run { "would delete" } else { "deleted" };
            for path in &removed {
                println!("{verb} {}", path.display());
            }
            if removed.is_empty() {
                println!(
                    "nothing to delete under {}: every cargo version there is named by a known lock, \
                     and no release archive of this repository is older than its newest finished version",
                    scope.toolchain_root.display()
                );
            }
            0
        }
        Err(error) => {
            eprintln!("mpm-shadow: {error}");
            error.exit_code()
        }
    };
}
