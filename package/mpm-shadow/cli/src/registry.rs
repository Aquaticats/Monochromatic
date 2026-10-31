//! What:
//! Lists the cached archives and extracted sources of this repository's own crates in cargo's registry.
//! Why:
//! These are the copies a plain `cargo install` leaves behind for every version it ever fetched.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // registry.ts
//! export function listOwnEntries(cargoHome: string): RegistryEntry[] { ... }
//! ```

/// Imports the file-system module (lists directories).
use std::fs;

/// Imports the borrowed path type and the owned one.
use std::path::{Path, PathBuf};

/// Imports `CRATE_ARCHIVE_SUFFIX`,
/// `OWN_CRATE_PREFIX` from `crate::constants`.
use crate::constants::{CRATE_ARCHIVE_SUFFIX, OWN_CRATE_PREFIX};
/// Imports `split_entry_name` from `crate::version`.
use crate::version::split_entry_name;

/// What:
/// `pub struct RegistryEntry { ... }` declares a record of one cached copy:
/// the crate name,
/// the version and
///           the path of the file or directory to delete.
/// Why:
/// All three are owned (`String`,
/// `String`,
/// `PathBuf`;
/// siblings `&str`,
/// `&Path`) because the entries are
///           collected into a list that outlives the directory listing that produced them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export type RegistryEntry = { crateName: string; version: string; path: string };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryEntry {
    /// The crate's name.
    pub crate_name: String,
    /// The cached version.
    pub version: String,
    /// The archive file or extracted directory to delete.
    pub path: PathBuf,
}

/// What:
/// `fn child_names(directory: &Path) -> Vec<String>` declares a private function returning the names inside
///           a directory as an owned list.
/// Why:
/// A missing directory is normal (no registry yet) and lists nothing;
/// names are sorted so the output order
///           never depends on the file system.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function childNames(directory: string): string[] { try { return fs.readdirSync(directory).sort(); } catch { return []; } }
/// ```
fn child_names(directory: &Path) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();

    // What:     `fs::read_dir(directory)` returns a `Result` holding an iterator of directory entries, each itself a
    //           `Result`. `let Ok(entries) = ... else { return names };` leaves the function when listing fails.
    // Why:      An unreadable directory has no children as far as the prune is concerned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let entries; try { entries = fs.readdirSync(directory); } catch { return names; }
    // ```
    let Ok(entries) = fs::read_dir(directory) else {
        return names;
    };
    for entry in entries.flatten() {
        // `entry.file_name().to_str()` borrows the name as text when it is valid UTF-8 (an `Option`); `.to_string()`
        // copies it into an owned `String`. A name that is not valid UTF-8 cannot be one of our crates.
        if let Some(name) = entry.file_name().to_str() {
            names.push(name.to_string());
        }
    }
    names.sort();
    return names;
}

/// What:
/// `fn entry_from_name(name: &str, suffix: &str, directory: &Path) -> Option<RegistryEntry>` declares a
///           private function that turns one directory child into a registry entry,
/// or `None`.
/// Why:
/// Both kinds of child go through the same checks:
/// the right suffix,
/// a `<crate>-<version>` shape,
/// and the
///           repository's crate prefix.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entryFromName(name: string, suffix: string, directory: string): RegistryEntry | undefined
/// ```
fn entry_from_name(name: &str, suffix: &str, directory: &Path) -> Option<RegistryEntry> {
    // What:     `name.strip_suffix(suffix)` removes the suffix and returns `Some(rest)`, or `None` when the name does
    //           not end with it. `?` leaves the function with `None` when the value is `None`, and otherwise unwraps it.
    // Why:      An archive must end in `.crate`; an extracted directory has no suffix (the empty suffix always matches).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!name.endsWith(suffix)) return undefined;
    // const stem = name.slice(0, name.length - suffix.length);
    // ```
    let stem: &str = name.strip_suffix(suffix)?;
    let split = split_entry_name(stem)?;
    if !split.crate_name.starts_with(OWN_CRATE_PREFIX) {
        return None;
    }
    // What:     `Some(RegistryEntry { ... })` wraps the new record in the present case of `Option`.
    // Why:      This child is one of our crates' cached copies.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { crateName: split.crateName, version: split.version, path: path.join(directory, name) };
    // ```
    return Some(RegistryEntry {
        crate_name: split.crate_name,
        version: split.version,
        path: directory.join(name),
    });
}

/// What:
/// `pub fn list_own_entries(cargo_home: &Path) -> Vec<RegistryEntry>` declares a public function returning
///           every own-crate entry found under the registry's `cache` and `src` directories.
/// Why:
/// The prune decides what to delete from this full list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function listOwnEntries(cargoHome: string): RegistryEntry[]
/// ```
pub fn list_own_entries(cargo_home: &Path) -> Vec<RegistryEntry> {
    let mut found: Vec<RegistryEntry> = Vec::new();

    // What:     `[("cache", CRATE_ARCHIVE_SUFFIX), ("src", "")]` is a fixed-size array of pairs; the `for` takes each
    //           pair apart into `(kind, suffix)`.
    // Why:      Archives under `cache` end in `.crate`; extracted sources under `src` have no suffix.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const [kind, suffix] of [['cache', '.crate'], ['src', '']]) { ... }
    // ```
    for (kind, suffix) in [("cache", CRATE_ARCHIVE_SUFFIX), ("src", "")] {
        let kind_directory: PathBuf = cargo_home.join("registry").join(kind);

        // Under each kind there is one directory per registry index (`index.crates.io-<hash>`).
        for index in child_names(&kind_directory) {
            let index_directory: PathBuf = kind_directory.join(&index);
            for name in child_names(&index_directory) {
                if let Some(entry) = entry_from_name(&name, suffix, &index_directory) {
                    found.push(entry);
                }
            }
        }
    }
    return found;
}
