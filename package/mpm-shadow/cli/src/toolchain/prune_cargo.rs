//! What:
//! The cleanup step for `cargo-binstall` installs:
//! `mpm-shadow prune --toolchain` deletes the version directories under `<toolchain root>/cargo` that no known lock
//! names,
//! and for a crate this repository publishes keeps only the newest installed version.
//! Why:
//! Version-keyed installs let trees with different locks coexist,
//! so nothing removes a version when the last lock naming it moves on.
//! The known locks are this scope's own lock,
//! the lock of every worktree `git worktree list` reports for the repository,
//! and any file given with `--lock`;
//! the command refuses to delete anything when a listed worktree's lock cannot be read,
//! because an unreadable lock might name a version that is about to be deleted.
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
use super::cargo_installs::{CARGO_NAMESPACE, CURRENT_LINK_NAME, point_current_at_newest, recorded_version};
/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports what this module uses from `super::runner`.
use super::runner::{ProcessRunner, RunOutcome, RunRequest, Runner};
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
        if let Some(version) = entry.get("version").and_then(Value::as_str) {
            named.entry(crate_of_entry(id, entry)).or_default().insert(version.to_string());
        }
    }
    return Ok(named);
}

/// The worktree lock files `git worktree list --porcelain` reports for the repository at `root`.
/// A worktree whose directory is gone is skipped only when git marks it `prunable`;
/// otherwise (an unmounted disk,
/// say) the command refuses.
pub fn worktree_lock_files(runner: &dyn Runner, root: &Path) -> Result<Vec<PathBuf>, ToolchainError> {
    let request: RunRequest = RunRequest::new("git", &["-C", &root.to_string_lossy(), "worktree", "list", "--porcelain"]);
    let outcome: RunOutcome = runner.run(&request);
    if !outcome.succeeded() {
        return Err(ToolchainError::failure(format!(
            "cannot list the repository's worktrees to find the locks that name tool versions: {}",
            outcome.failure_text()
        )));
    }
    let mut files: Vec<PathBuf> = Vec::new();
    for block in outcome.stdout.split("\n\n") {
        let mut path: Option<&str> = None;
        let mut prunable: bool = false;
        for line in block.lines() {
            if let Some(found) = line.strip_prefix("worktree ") {
                path = Some(found);
            } else if line == "prunable" || line.starts_with("prunable ") {
                prunable = true;
            }
        }
        let Some(directory) = path else {
            continue;
        };
        if !Path::new(directory).is_dir() {
            if prunable {
                continue;
            }
            return Err(ToolchainError::failure(format!(
                "the worktree {directory} is listed by git but its directory is missing and not marked prunable, \
                 so its lock cannot be read; nothing was deleted. Mount it, run `git worktree prune` if it is gone, \
                 or name its lock with --lock"
            )));
        }
        files.push(Path::new(directory).join("toolchain.lock.json"));
    }
    return Ok(files);
}

/// The versions every lock in `lock_files` names;
/// a file that does not exist names none,
/// one that cannot be read or is not a lock refuses.
fn named_by_all(lock_files: &[PathBuf]) -> Result<BTreeMap<String, BTreeSet<String>>, ToolchainError> {
    let mut named: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for file in lock_files {
        let text: String = match fs::read_to_string(file) {
            Ok(text) => text,
            Err(error) if error.kind() == ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(ToolchainError::failure(format!("cannot read the lock {}: {error}; nothing was deleted", file.display())));
            }
        };
        for (crate_name, versions) in named_versions(&text, &file.display().to_string())? {
            named.entry(crate_name).or_default().extend(versions);
        }
    }
    return Ok(named);
}

/// Every crate and version directory under `<toolchain root>/cargo`.
fn present_versions(toolchain_root: &Path) -> BTreeMap<String, Vec<PresentVersion>> {
    let mut present: BTreeMap<String, Vec<PresentVersion>> = BTreeMap::new();
    let Ok(crates) = fs::read_dir(toolchain_root.join(CARGO_NAMESPACE)) else {
        return present;
    };
    for crate_entry in crates.flatten() {
        let crate_name: String = crate_entry.file_name().to_string_lossy().to_string();
        let Ok(versions) = fs::read_dir(crate_entry.path()) else {
            continue;
        };
        let mut found: Vec<PresentVersion> = Vec::new();
        for version_entry in versions.flatten() {
            let version: String = version_entry.file_name().to_string_lossy().to_string();
            let is_plain_directory: bool = match version_entry.file_type() {
                Ok(kind) => kind.is_dir(),
                Err(_) => false,
            };
            if is_plain_directory && !version.starts_with('.') && version != CURRENT_LINK_NAME {
                let finished: bool = recorded_version(&version_entry.path(), &crate_name).is_some();
                found.push(PresentVersion { version, finished });
            }
        }
        present.insert(crate_name, found);
    }
    return present;
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
    let mut lock_files: Vec<PathBuf> = vec![scope.lock_file.clone()];
    if scope.kind == ScopeKind::Repo {
        lock_files.extend(worktree_lock_files(runner, &scope.root)?);
    }
    lock_files.extend(extra_locks.iter().cloned());
    let named = named_by_all(&lock_files)?;
    let present = present_versions(&scope.toolchain_root);
    let mut removed: Vec<PathBuf> = Vec::new();
    for (crate_name, version) in plan_prune(&present, &named) {
        let directory: PathBuf = scope.toolchain_root.join(CARGO_NAMESPACE).join(&crate_name).join(&version);
        if !dry_run && let Err(error) = fs::remove_dir_all(&directory) {
            return Err(ToolchainError::failure(format!("cannot remove {}: {error}", directory.display())));
        }
        removed.push(directory);
    }
    if !dry_run {
        let after = present_versions(&scope.toolchain_root);
        for crate_name in present.keys() {
            let crate_dir: PathBuf = scope.toolchain_root.join(CARGO_NAMESPACE).join(crate_name);
            let remaining: bool = match after.get(crate_name) {
                Some(versions) => !versions.is_empty(),
                None => false,
            };
            if remaining {
                // Only a crate this repository publishes has a `current` link; a third-party crate is never given one.
                if crate_name.starts_with(OWN_CRATE_PREFIX) {
                    point_current_at_newest(&scope.toolchain_root, crate_name)?;
                }
            } else if let Err(error) = fs::remove_dir_all(&crate_dir) {
                return Err(ToolchainError::failure(format!("cannot remove {}: {error}", crate_dir.display())));
            }
        }
    }
    return Ok(removed);
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
                println!("no unnamed cargo version under {}", scope.toolchain_root.join(CARGO_NAMESPACE).display());
            }
            0
        }
        Err(error) => {
            eprintln!("mpm-shadow: {error}");
            error.exit_code()
        }
    };
}
