//! What:
//! Finds the first-party crate roots that carry a `Cargo.lock`, and snapshots and restores their lockfiles.
//! Why:
//! The catalog commands treat all lockfiles as one transaction: any failure or refusal puts every lockfile back as it was.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // crates.ts
//! export async function lockedCrateDirs(root: string): Promise<string[]>
//! export async function readLocks({ root, crateDirs }): Promise<LockSnapshot[]>
//! export async function restoreLocks({ root, snapshots }): Promise<void>
//! ```

use std::fs;
use std::io;
use std::path::Path;

use crate::constants::{LOCKFILE_NAME, SKIPPED_DIRECTORIES};

/// How many directory levels under `package/` hold crate roots: `package/<category>/<name>` and
/// `package/<category>/<name>/<sub>`, the same two globs file-enforcer's Rust catalog sync uses.
const CRATE_ROOT_DEPTHS: [usize; 2] = [2, 3];

/// The top directory of the first-party packages.
const PACKAGE_DIRECTORY: &str = "package";

/// One crate's lockfile text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LockSnapshot {
    /// Repo-relative crate directory, forward slashes.
    pub crate_dir: String,
    /// The lockfile's text.
    pub source: String,
}

/// Collects the repo-relative directories exactly `depth` levels below `base` that hold a lockfile.
fn collect_at_depth(root: &Path, relative: &str, depth: usize, found: &mut Vec<String>) -> io::Result<()> {
    if depth == 0 {
        if root.join(relative).join(LOCKFILE_NAME).is_file() {
            found.push(relative.to_string());
        }
        return Ok(());
    }
    let entries = match fs::read_dir(root.join(relative)) {
        Ok(listing) => listing,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    for listed in entries {
        let entry = listed?;
        let name: String = entry.file_name().to_string_lossy().to_string();
        if SKIPPED_DIRECTORIES.contains(&name.as_str()) || !entry.file_type()?.is_dir() {
            continue;
        }
        collect_at_depth(root, &format!("{relative}/{name}"), depth - 1, found)?;
    }
    return Ok(());
}

/// What:
/// `pub fn locked_crate_dirs(root: &Path) -> io::Result<Vec<String>>` lists the repo-relative crate directories with a
/// `Cargo.lock`, sorted.
/// Why:
/// Same discovery as the update tool's and file-enforcer's globs; `target` and `node_modules` are never entered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function lockedCrateDirs(root: string): Promise<string[]>
/// ```
pub fn locked_crate_dirs(root: &Path) -> io::Result<Vec<String>> {
    let mut found: Vec<String> = Vec::new();
    for depth in CRATE_ROOT_DEPTHS {
        collect_at_depth(root, PACKAGE_DIRECTORY, depth, &mut found)?;
    }
    found.sort();
    found.dedup();
    return Ok(found);
}

/// Reads every listed crate's lockfile, in order.
pub fn read_locks(root: &Path, crate_dirs: &[String]) -> io::Result<Vec<LockSnapshot>> {
    let mut snapshots: Vec<LockSnapshot> = Vec::new();
    for crate_dir in crate_dirs {
        let source: String = fs::read_to_string(root.join(crate_dir).join(LOCKFILE_NAME))?;
        snapshots.push(LockSnapshot { crate_dir: crate_dir.clone(), source });
    }
    return Ok(snapshots);
}

/// Writes the snapshots back, so a refused or dry-run install leaves every lockfile as it was.
pub fn restore_locks(root: &Path, snapshots: &[LockSnapshot]) -> io::Result<()> {
    for snapshot in snapshots {
        fs::write(root.join(&snapshot.crate_dir).join(LOCKFILE_NAME), &snapshot.source)?;
    }
    return Ok(());
}
