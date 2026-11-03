//! What:
//! The catalog half of `pnpm update --latest`:
//! writes the new range of an updated dependency into `rust-catalog.toml`,
//! then runs
//! the sync that renders it into the manifests.
//! Why:
//! pnpm keeps `catalog:` in `package.json` and writes the updated range to the catalog entry in `pnpm-workspace.yaml`:
//! the
//! resolver collects `updatedCatalogs` for every wanted dependency with `updateSpec === true` and a catalog lookup
//! (`pnpm11/installing/deps-resolver/src/index.ts:280-322`,
//! `getUpdatedCatalogs`,
//! `getCatalogUpdatesOfProject`;
//! `pnpm11/installing/deps-resolver/src/wantedDepShouldUpdateCatalog.ts:3-5`:
//! true when `updateSpec` is set and `saveSpec` is not
//! false,
//! so `--no-save` turns it off).
//! `--latest` is the update that moves ranges,
//! so `cargo update --breaking` is the cargo
//! command that gets the same treatment:
//! every catalog entry whose highest locked version rose gets its `>=` floor raised to
//! that version,
//! and `file-enforcer write` renders the manifests.
//! Entries in named catalogs (bounded requirements by design) are
//! never touched.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // tighten.ts
//! export function raiseFloors({ source, locked, changed }): { source: string; changes: string[] }
//! ```

/// What:
/// `use std::collections::{BTreeMap, BTreeSet};` imports `BTreeMap`,
/// `BTreeSet` from the standard library so this file can name them unqualified.
/// Why:
/// `BTreeMap` is used by `highest_by_name`,
/// `raised_names`,
/// `raise_floors`;
/// `BTreeSet` is used by `raised_names`,
/// `raise_floors`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { BTreeMap, BTreeSet } from 'std/collections';
/// ```
use std::collections::{BTreeMap, BTreeSet};
/// What:
/// `use std::path::Path;` imports `Path` from the standard library so this file can name it unqualified.
/// Why:
/// `Path` is used by `sync`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Path } from 'std/path';
/// ```
use std::path::Path;
/// What:
/// `use std::process::Command;` imports `Command` from the standard library so this file can name it unqualified.
/// Why:
/// `Command` is used by `sync`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Command } from 'std/process';
/// ```
use std::process::Command;

/// What:
/// `use toml_edit::{DocumentMut, Item, Value};` imports `DocumentMut`,
/// `Item`,
/// `Value` from the external crate `toml_edit` so this file can name them unqualified.
/// Why:
/// `DocumentMut` is used by `raise_floors`;
/// `Item` is used by `raise_floors`;
/// `Value` is used by `raise_floors`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { DocumentMut, Item, Value } from 'toml_edit';
/// ```
use toml_edit::{DocumentMut, Item, Value};

/// What:
/// `use crate::lockfile::LockedVersion;` imports `LockedVersion` from another module of this crate so this file can name it unqualified.
/// Why:
/// `LockedVersion` is used by `highest_by_name`,
/// `raised_names`,
/// `raise_floors`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockedVersion } from 'crate/lockfile';
/// ```
use crate::lockfile::LockedVersion;
/// What:
/// `use crate::version_order::compare_texts;` imports `compare_texts` from another module of this crate so this file can name it unqualified.
/// Why:
/// `compare_texts` is used by `highest_by_name`,
/// `raised_names`,
/// `raise_floors`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { compare_texts } from 'crate/version_order';
/// ```
use crate::version_order::compare_texts;
/// What:
/// `use crate::adapt::text_of;` imports `text_of` from another module of this crate so this file can name it unqualified.
/// Why:
/// `text_of` is used by `raise_floors`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { text_of } from 'crate/adapt';
/// ```
use crate::adapt::text_of;

/// What:
/// `pub trait Syncer { fn sync(&self, root: &Path) -> Result<String, String>; }` renders the catalog into the manifests.
/// Why:
/// The real one runs `file-enforcer write`;
/// tests give a fake that records the call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface Syncer { sync(root: string): string }
/// ```
pub trait Syncer {
    /// Runs the sync from the repository root;
    /// the string is a one-line result for the report.
    fn sync(&self, root: &Path) -> Result<String, String>;
}

/// The real sync:
/// `file-enforcer write`,
/// the task `sync:files` runs.
pub struct FileEnforcer;

/// What:
/// `fn file_enforcer_start_failure(error: std::io::Error) -> String` words the failure to start `file-enforcer`,
/// with the command that renders the floors by hand.
/// Why:
/// Raised floors only reach the manifests through `file-enforcer write`,
/// so the user is told how to do that step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fileEnforcerStartFailure = (error: Error) => `could not start file-enforcer (${error.message}); run the sync task`;
/// ```
fn file_enforcer_start_failure(error: std::io::Error) -> String {
    return format!("could not start file-enforcer ({error}); run `mise run sync:files` to render the raised floors into the manifests");
}

/// What:
/// `impl Syncer for FileEnforcer` makes the `file-enforcer` program usable as the sync step.
/// Why:
/// `cargo update --breaking` raises catalog floors and then renders them into the manifests;
/// tests use a fake that only records the call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class FileEnforcer implements Syncer { sync(root) { ... } }
/// ```
impl Syncer for FileEnforcer {
    /// What:
    /// `fn sync(&self, root)` runs `file-enforcer write` in the repository root and returns a one-line result.
    /// Why:
    /// That command is the `sync:files` task,
    /// the one place manifests are rendered from the catalog;
    /// a missing program becomes a message that names the manual step.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// sync(root: string): string  // throws with the remedy
    /// ```
    fn sync(&self, root: &Path) -> Result<String, String> {
        let output = Command::new("file-enforcer")
            .arg("write")
            .current_dir(root)
            .output()
            .map_err(file_enforcer_start_failure)?;
        if !output.status.success() {
            return Err(format!("file-enforcer write failed: {}", String::from_utf8_lossy(&output.stderr).trim()));
        }
        return Ok("rendered the catalog into the manifests with file-enforcer write".to_string());
    }
}

/// The highest locked version of each crate in the given versions.
fn highest_by_name(locked: &[LockedVersion]) -> BTreeMap<String, String> {
    let mut highest: BTreeMap<String, String> = BTreeMap::new();
    for version in locked {
        let replace: bool = match highest.get(&version.name) {
            Some(current) => compare_texts(&version.version, current) == std::cmp::Ordering::Greater,
            None => true,
        };
        if replace {
            highest.insert(version.name.clone(), version.version.clone());
        }
    }
    return highest;
}

/// The names whose highest locked version is higher in `after` than in `before`.
pub fn raised_names(before: &[LockedVersion], after: &[LockedVersion]) -> BTreeSet<String> {
    let old: BTreeMap<String, String> = highest_by_name(before);
    let mut raised: BTreeSet<String> = BTreeSet::new();
    for (name, version) in highest_by_name(after) {
        let was_higher_or_equal: bool = match old.get(&name) {
            Some(previous) => compare_texts(&version, previous) != std::cmp::Ordering::Greater,
            None => false,
        };
        if !was_higher_or_equal {
            raised.insert(name);
        }
    }
    return raised;
}

/// What:
/// `pub fn raise_floors(source: &str, locked: &[LockedVersion], changed: &BTreeSet<String>) -> Result<(String, Vec<String>), String>`
/// raises the `>=` floor of each default-catalog entry in `changed` to the highest locked version when that is higher.
/// Why:
/// Only `>=` floors are raised,
/// the file's other text and each value's comments are kept,
/// and an entry whose floor is already at
/// or above the locked version is left alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function raiseFloors({ source, locked, changed }): { source: string; changes: string[] }
/// ```
pub fn raise_floors(source: &str, locked: &[LockedVersion], changed: &BTreeSet<String>) -> Result<(String, Vec<String>), String> {
    let mut document: DocumentMut = source.parse::<DocumentMut>().map_err(text_of)?;
    let highest: BTreeMap<String, String> = highest_by_name(locked);
    let mut changes: Vec<String> = Vec::new();
    let catalog = match document.get_mut("catalog").and_then(Item::as_table_mut) {
        Some(found) => found,
        None => return Ok((source.to_string(), changes)),
    };
    for name in changed {
        let new_version: &String = match highest.get(name) {
            Some(found) => found,
            None => continue,
        };
        let item: &mut Item = match catalog.get_mut(name) {
            Some(found) => found,
            None => continue,
        };
        let current: String = match item.as_str() {
            Some(text) => text.to_string(),
            None => continue,
        };
        let floor: &str = match current.strip_prefix(">=") {
            Some(found) => found.trim(),
            None => continue,
        };
        if compare_texts(new_version, floor) != std::cmp::Ordering::Greater {
            continue;
        }
        let mut replacement: Value = Value::from(format!(">={new_version}"));
        if let Some(old) = item.as_value() {
            *replacement.decor_mut() = old.decor().clone();
        }
        *item = Item::Value(replacement);
        changes.push(format!("{name}: >={floor} to >={new_version}"));
    }
    return Ok((document.to_string(), changes));
}
