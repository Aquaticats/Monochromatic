//! What:
//! The prune rule:
//! of every cached copy of this repository's own crates,
//! keep one version per crate and delete
//!           the rest.
//! Why:
//! mpm 8.2.0 removes no previous version on install or upgrade and has no cleanup for the `cargo` manager,
//! so
//!           the copies a plain `cargo install` leaves behind pile up in cargo's registry.
//!
//! The kept version is the installed one when an install record names the crate (that is the version just
//! provisioned):
//! the newest of cargo's `<cargo home>/.crates.toml` and the version-keyed records under
//! `<toolchain root>/cargo/<crate>/` that `mpm-shadow install` writes;
//! otherwise the newest cached version (a library crate such as `monochromatic-jsonc-edit`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // prune.ts
//! export function findStale(entries: RegistryEntry[], installed: Map<string, string>): RegistryEntry[] { ... }
//! export function pruneStaleOwnCrates(cargoHome: string): string[] { ... }
//! ```

/// Imports the sorted map (see `installed.rs`).
use std::collections::BTreeMap;

/// Imports the comparison result type (see `version.rs`).
use std::cmp::Ordering;

/// Imports the file-system module (deletes files and directories).
use std::fs;

/// Imports the owned and borrowed path types.
use std::path::{Path, PathBuf};

/// Imports `installed_versions` from `crate::installed`.
use crate::installed::installed_versions;
/// Imports `RegistryEntry`,
/// `list_own_entries` from `crate::registry`.
use crate::registry::{RegistryEntry, list_own_entries};
/// Imports `repository_versions_installed` from `crate::toolchain::cargo_installs`.
use crate::toolchain::cargo_installs::repository_versions_installed;
/// Imports `compare_versions` from `crate::version`.
use crate::version::compare_versions;

/// What:
/// `fn newest_versions(entries: &[RegistryEntry]) -> BTreeMap<String, String>` declares a private function.
///           `&[RegistryEntry]` is a borrowed view of a list (siblings:
/// `Vec<RegistryEntry>` owns the list,
///           `[RegistryEntry; N]` is a fixed-size array).
/// Why:
/// It finds,
/// for each crate,
/// the newest version present in the cache.
/// `&[...]` and not `Vec` because the
///           function only reads the list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function newestVersions(entries: readonly RegistryEntry[]): Map<string, string>
/// ```
fn newest_versions(entries: &[RegistryEntry]) -> BTreeMap<String, String> {
    let mut newest: BTreeMap<String, String> = BTreeMap::new();
    for entry in entries {
        // What:     `newest.get(&entry.crate_name)` looks the crate up and returns `Option<&String>`: a borrowed
        //           value or `None`. `&entry.crate_name` lends the key. The `match` picks the branch by whether the
        //           crate has been seen.
        // Why:      Replace the stored version only when this entry's version is newer.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const current = newest.get(entry.crateName);
        // if (current === undefined || compareVersions(entry.version, current) > 0) newest.set(entry.crateName, entry.version);
        // ```
        let is_newer: bool = match newest.get(&entry.crate_name) {
            None => true,
            Some(current) => compare_versions(&entry.version, current) == Ordering::Greater,
        };
        if is_newer {
            newest.insert(entry.crate_name.clone(), entry.version.clone());
        }
    }
    return newest;
}

/// What:
/// `pub fn find_stale(entries: Vec<RegistryEntry>, installed: &BTreeMap<String, String>) -> Vec<RegistryEntry>`
///           declares a public function.
/// It takes the entry list by value (it moves the list in) and the installed
///           map by borrow,
/// and returns the entries to delete.
/// Why:
/// This is the whole rule,
/// kept free of file-system access so the tests can feed it plain lists.
/// `Vec` and
///           not a borrow for `entries` because the stale ones are moved into the result.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function findStale(entries: RegistryEntry[], installed: Map<string, string>): RegistryEntry[]
/// ```
pub fn find_stale(entries: Vec<RegistryEntry>, installed: &BTreeMap<String, String>) -> Vec<RegistryEntry> {
    let newest: BTreeMap<String, String> = newest_versions(&entries);
    let mut stale: Vec<RegistryEntry> = Vec::new();
    for entry in entries {
        // What:     `installed.get(&entry.crate_name).or_else(|| newest.get(&entry.crate_name))` takes the installed
        //           version when there is one, otherwise the newest cached one. `or_else` runs its closure only when
        //           the first `Option` is `None`. `if let Some(kept)` unwraps the result.
        // Why:      This is the version to keep for the entry's crate.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const kept = installed.get(entry.crateName) ?? newest.get(entry.crateName);
        // ```
        let kept: Option<&String> = match installed.get(&entry.crate_name) {
            Some(version) => Some(version),
            None => newest.get(&entry.crate_name),
        };
        if let Some(kept_version) = kept
            && *kept_version != entry.version
        {
            stale.push(entry);
        }
    }
    return stale;
}

/// What:
/// `pub fn installed_everywhere(cargo_home: &Path, toolchain_roots: &[PathBuf]) -> BTreeMap<String, String>`
///           declares a public function returning,
/// for each repository crate,
/// the newest version recorded as installed in any of
///           the places `install` or a plain `cargo install` can put it:
/// the version-keyed records under
///           `<toolchain root>/cargo/<crate>/` of every given root,
/// and cargo's own `<cargo home>/.crates.toml`.
/// Why:
/// After the cutover `install` writes only the version-keyed records,
/// so the `.crates.toml` row goes stale and,
/// read alone,
/// would keep an old cached version and delete a newer one.
/// Trees still on the TypeScript toolchain keep installing into `$CARGO_HOME`,
/// so that file is still read.
/// The newest wins because older repository tools are never kept once a newer one is installed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function installedEverywhere(cargoHome: string, toolchainRoots: string[]): Map<string, string>
/// ```
pub fn installed_everywhere(cargo_home: &Path, toolchain_roots: &[PathBuf]) -> BTreeMap<String, String> {
    let mut merged: BTreeMap<String, String> = installed_versions(cargo_home);
    for root in toolchain_roots {
        for (crate_name, version) in repository_versions_installed(root) {
            let is_newer: bool = match merged.get(&crate_name) {
                None => true,
                Some(current) => compare_versions(&version, current) == Ordering::Greater,
            };
            if is_newer {
                merged.insert(crate_name, version);
            }
        }
    }
    return merged;
}

/// What:
/// `pub fn prune_stale_own_crates(cargo_home: &Path, toolchain_roots: &[PathBuf]) -> Vec<PathBuf>` declares a
///           public function that deletes the stale entries under a cargo home and returns the paths it deleted.
///           The kept version of a crate comes from `installed_everywhere`.
/// Why:
/// A deletion that fails is reported on standard error and skipped:
/// a leftover cache entry never affects the
///           mpm run that just finished,
/// so it must not change its exit code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function pruneStaleOwnCrates(cargoHome: string): string[]
/// ```
pub fn prune_stale_own_crates(cargo_home: &Path, toolchain_roots: &[PathBuf]) -> Vec<PathBuf> {
    let report: PruneReport = prune_stale_own_crates_reporting(cargo_home, toolchain_roots);
    for (path, reason) in &report.failed {
        eprintln!("mpm-shadow: could not delete {}: {reason}", path.display());
    }
    return report.deleted;
}

/// What a prune did:
/// the paths removed,
/// their total size,
/// and the paths it could not remove with the system's reason.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PruneReport {
    /// Paths removed.
    pub deleted: Vec<PathBuf>,
    /// Total size in bytes of what was removed.
    pub bytes: u64,
    /// Paths that could not be removed,
    /// each with why.
    pub failed: Vec<(PathBuf, String)>,
}

/// What:
/// The size in bytes of a file,
/// or of a directory with everything inside it.
/// Why:
/// The cleanup summary reports how much was removed.
/// A work stack walks the tree,
/// so a deep source directory cannot overflow the call stack.
/// Anything that cannot be read counts as zero:
/// the size is a courtesy,
/// not a decision.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sizeOf(path: string): number
/// ```
fn size_of(path: &Path) -> u64 {
    let mut total: u64 = 0;
    let mut pending: Vec<PathBuf> = vec![path.to_path_buf()];
    while let Some(current) = pending.pop() {
        let Ok(metadata) = fs::symlink_metadata(&current) else {
            continue;
        };
        if metadata.is_dir() {
            if let Ok(children) = fs::read_dir(&current) {
                for child in children.flatten() {
                    pending.push(child.path());
                }
            }
        } else {
            total += metadata.len();
        }
    }
    return total;
}

/// What:
/// Deletes the stale entries under a cargo home like `prune_stale_own_crates`,
/// but prints nothing and returns what happened:
/// the paths removed,
/// their total size,
/// and each path that could not be removed with the system's reason.
/// Why:
/// The caller decides how to word the result (the shim prints one summary line).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function pruneStaleOwnCratesReporting(cargoHome: string, toolchainRoots: string[]): PruneReport
/// ```
pub fn prune_stale_own_crates_reporting(cargo_home: &Path, toolchain_roots: &[PathBuf]) -> PruneReport {
    let installed: BTreeMap<String, String> = installed_everywhere(cargo_home, toolchain_roots);
    let stale: Vec<RegistryEntry> = find_stale(list_own_entries(cargo_home), &installed);
    let mut report: PruneReport = PruneReport::default();
    for entry in stale {
        let size: u64 = size_of(&entry.path);
        // An archive is a file and an extracted source is a directory, and they need different delete calls.
        let result: std::io::Result<()> = if entry.path.is_dir() {
            fs::remove_dir_all(&entry.path)
        } else {
            fs::remove_file(&entry.path)
        };
        match result {
            Ok(()) => {
                report.bytes += size;
                report.deleted.push(entry.path);
            }
            // What: a copy another run of the shadow removed first is a finished cleanup.
            // Why: every tree's shadow prunes the same cargo home, so two of them reach this entry at once, and the
            // space the other one gave back is not this run's to report.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => report.failed.push((entry.path, error.to_string())),
        }
    }
    return report;
}
