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
//! The kept version is the installed one when cargo's install record names the crate (that is the version just
//! provisioned),
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
/// `pub fn prune_stale_own_crates(cargo_home: &Path) -> Vec<PathBuf>` declares a public function that deletes
///           the stale entries under a cargo home and returns the paths it deleted.
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
pub fn prune_stale_own_crates(cargo_home: &Path) -> Vec<PathBuf> {
    let installed: BTreeMap<String, String> = installed_versions(cargo_home);
    let stale: Vec<RegistryEntry> = find_stale(list_own_entries(cargo_home), &installed);
    let mut deleted: Vec<PathBuf> = Vec::new();
    for entry in stale {
        // An archive is a file and an extracted source is a directory, and they need different delete calls.
        let result: std::io::Result<()> = if entry.path.is_dir() {
            fs::remove_dir_all(&entry.path)
        } else {
            fs::remove_file(&entry.path)
        };

        // What:     `match result { Ok(()) => ..., Err(error) => ... }` picks the branch for success or failure.
        //           `Ok(())` is success carrying no value (`()` is the empty tuple, Rust's `void`).
        // Why:      Record what was deleted; report what could not be.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // try { rmSync(entry.path, { recursive: true }); deleted.push(entry.path); } catch (error) { console.error(...); }
        // ```
        match result {
            Ok(()) => deleted.push(entry.path),
            Err(error) => eprintln!("mpm-shadow: could not delete {}: {error}", entry.path.display()),
        }
    }
    return deleted;
}
