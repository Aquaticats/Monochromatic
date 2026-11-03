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

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;

use toml_edit::{DocumentMut, Item, Value};

use crate::lockfile::LockedVersion;
use crate::version_order::compare_texts;

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

impl Syncer for FileEnforcer {
    fn sync(&self, root: &Path) -> Result<String, String> {
        let output = Command::new("file-enforcer")
            .arg("write")
            .current_dir(root)
            .output()
            .map_err(|error| { return format!("could not start file-enforcer ({error}); run `mise run sync:files` to render the raised floors into the manifests"); })?;
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
        let was_higher_or_equal: bool = old.get(&name).map(|previous| { return compare_texts(&version, previous) != std::cmp::Ordering::Greater; }).unwrap_or(false);
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
    let mut document: DocumentMut = source.parse::<DocumentMut>().map_err(|error| { return error.to_string(); })?;
    let highest: BTreeMap<String, String> = highest_by_name(locked);
    let mut changes: Vec<String> = Vec::new();
    let catalog = match document.get_mut("catalog").and_then(|item| { return item.as_table_mut(); }) {
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
