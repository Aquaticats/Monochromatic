//! What:
//! The transaction around the manifests that cargo's own `update --breaking` rewrites:
//! the text of every workspace manifest before the run,
//! the text after it,
//! and the restore that puts back only what this run wrote.
//! Why:
//! `cargo update --breaking` edits version requirements in `Cargo.toml` as well as the lockfile,
//! so a refusal or a dry run that restored only the lockfiles would leave the new requirements behind.
//! The restore follows the lockfile rule:
//! a manifest is replaced only while it still holds what this run's cargo wrote,
//! through a sibling file and an atomic rename,
//! and one that another process changed is left alone with a warning.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // manifest-edit.ts
//! export type ManifestEdit = { path: string; before: string; produced?: string };
//! export function snapshotManifests(paths: string[]): ManifestEdit[]
//! export function recordManifestOutput(edits: ManifestEdit[]): void
//! export function restoreManifests(edits: ManifestEdit[]): string[]
//! ```

/// What:
/// `use std::cell::RefCell;` imports
/// `RefCell`
/// so this file can name them unqualified.
/// Why:
/// The list of manifests grows while the transaction holds a shared reference to it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS objects are mutable through any reference; no import is needed.
/// ```
use std::cell::RefCell;
/// What:
/// `use std::fs;` imports
/// `fs`
/// so this file can name them unqualified.
/// Why:
/// `fs` reads and compares the manifest files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { readFileSync } from 'node:fs';
/// ```
use std::fs;
/// What:
/// `use std::io;` imports
/// `io`
/// so this file can name them unqualified.
/// Why:
/// `io::Result` is the error type of reading and restoring a file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // errors are thrown in Node
/// ```
use std::io;
/// What:
/// `use std::path::PathBuf;` imports
/// `PathBuf`
/// so this file can name them unqualified.
/// Why:
/// A manifest's path is stored,
/// so it is the owned type (sibling:
/// `&Path`,
/// the borrowed one).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has one type for this: string
/// ```
use std::path::PathBuf;

/// What:
/// `use crate::journal::{ReplaceOutcome, no_hook, replace_if_holds};` imports
/// `ReplaceOutcome`,
/// `replace_if_holds`
/// so this file can name them unqualified.
/// Why:
/// A manifest goes back with the same atomic compare-and-replace the lockfiles use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ReplaceOutcome, replaceIfHolds } from 'crate/journal';
/// ```
use crate::journal::{ReplaceOutcome, no_hook, replace_if_holds};

/// One manifest the run may rewrite.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManifestEdit {
    /// Where the manifest is.
    pub path: PathBuf,
    /// Its text before the run.
    pub before: String,
    /// Its text right after this run's cargo call returned;
    /// `None` until then,
    /// and a manifest without it is never restored.
    pub produced: Option<String>,
}

/// What:
/// `pub fn snapshot_manifests(paths, edits) -> io::Result<()>` reads each manifest and adds it to `edits`.
/// Why:
/// The text must be taken before cargo runs;
/// a manifest that cannot be read stops the run before anything is mutated.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function snapshotManifests(paths: string[], edits: ManifestEdit[]): void  // throws when a file cannot be read
/// ```
pub fn snapshot_manifests(paths: &[PathBuf], edits: &RefCell<Vec<ManifestEdit>>) -> io::Result<()> {
    for path in paths {
        let before: String = fs::read_to_string(path)?;
        edits.borrow_mut().push(ManifestEdit { path: path.clone(), before, produced: None });
    }
    return Ok(());
}

/// What:
/// `pub fn record_manifest_output(edits)` reads each manifest again and stores the text as this run's output.
/// Why:
/// Called right after cargo returned,
/// success or failure,
/// so the restore can tell cargo's own edit from another writer's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function recordManifestOutput(edits: ManifestEdit[]): void
/// ```
pub fn record_manifest_output(edits: &RefCell<Vec<ManifestEdit>>) {
    for edit in edits.borrow_mut().iter_mut() {
        // A manifest that cannot be read now stays unrecorded and is therefore never restored.
        edit.produced = fs::read_to_string(&edit.path).ok();
    }
}

/// What:
/// `pub fn restore_manifests(edits) -> io::Result<Vec<String>>` puts each manifest back that still holds this run's output
/// and returns a warning for each one it left alone.
/// Why:
/// Unknown ownership never authorizes a restore:
/// a manifest with no recorded output,
/// or one that differs from it,
/// belongs to another process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function restoreManifests(edits: ManifestEdit[]): string[]  // throws when a write fails
/// ```
pub fn restore_manifests(edits: &RefCell<Vec<ManifestEdit>>) -> io::Result<Vec<String>> {
    let mut warnings: Vec<String> = Vec::new();
    for edit in edits.borrow().iter() {
        let live: String = fs::read_to_string(&edit.path).unwrap_or_default();
        if live == edit.before {
            continue;
        }
        match &edit.produced {
            Some(produced) if *produced == live => match replace_if_holds(&edit.path, produced, &edit.before, &no_hook)? {
                ReplaceOutcome::Replaced => {}
                ReplaceOutcome::LeftAlone => warnings.push(format!(
                    "{} was changed by another process while it was being restored; it was left as that process wrote it",
                    edit.path.display()
                )),
                ReplaceOutcome::Preserved(kept) => warnings.push(format!(
                    "{} was changed twice by other processes while it was being restored; the file displaced is kept as {}",
                    edit.path.display(),
                    kept.display()
                )),
            },
            _ => warnings.push(format!(
                "{} differs from its text before the run, but this run did not leave it that way, so another process did; it was left as it is, not restored",
                edit.path.display()
            )),
        }
    }
    return Ok(warnings);
}
