//! What:
//! Finds the repository that uses the Rust dependency catalog and the Cargo workspace that holds a directory.
//! Why:
//! The install step runs for exactly one Cargo workspace, the one cargo itself would use for the current directory, and only
//! inside a repository that has a `rust-catalog.toml`; anywhere else the shadow is silent.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // workspace.ts
//! export function findCatalogRoot(start: string): string | undefined
//! export function findCargoRoot(start: string): string | undefined
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use toml_edit::DocumentMut;

use crate::constants::{CATALOG_MARKER, MANIFEST_NAME};

/// The nearest ancestor (the directory itself included) that holds `rust-catalog.toml`.
pub fn find_catalog_root(start: &Path) -> Option<PathBuf> {
    for directory in start.ancestors() {
        if directory.join(CATALOG_MARKER).is_file() {
            return Some(directory.to_path_buf());
        }
    }
    return None;
}

/// Whether manifest text declares a `[workspace]` table.
fn declares_workspace(text: &str) -> bool {
    return match text.parse::<DocumentMut>() {
        Ok(document) => document.contains_key("workspace"),
        Err(_) => false,
    };
}

/// What:
/// `pub fn find_cargo_root(start: &Path) -> Option<PathBuf>` walks up from `start` to the nearest manifest that declares
/// `[workspace]`; with none, it is the nearest manifest of any kind.
/// Why:
/// Cargo's own rule: a package belongs to the closest ancestor workspace. In this repository every crate is its own
/// workspace root, so the nearest manifest is nearly always the answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function findCargoRoot(start: string): string | undefined
/// ```
pub fn find_cargo_root(start: &Path) -> Option<PathBuf> {
    let mut nearest: Option<PathBuf> = None;
    for directory in start.ancestors() {
        let manifest: PathBuf = directory.join(MANIFEST_NAME);
        let text: String = match fs::read_to_string(&manifest) {
            Ok(found) => found,
            Err(_) => continue,
        };
        if nearest.is_none() {
            nearest = Some(directory.to_path_buf());
        }
        if declares_workspace(&text) {
            return Some(directory.to_path_buf());
        }
    }
    return nearest;
}
