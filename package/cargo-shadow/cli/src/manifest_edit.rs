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
/// `use std::cell::RefCell;` imports `RefCell`, a list that can be changed through a shared reference.
/// Why:
/// The list of manifests grows while the transaction holds a shared reference to it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS objects are mutable through any reference; no import is needed.
/// ```
use std::cell::RefCell;
/// What:
/// `use std::io;` imports the file-system error module.
/// Why:
/// A manifest that cannot be read is an error, not an empty file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // errors are thrown in Node
/// ```
use std::io;
/// What:
/// `use std::path::PathBuf;` imports the owned path type.
/// Why:
/// A manifest's path is stored, so it is the owned type (sibling: `&Path`, the borrowed one).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has one type for this: string
/// ```
use std::path::PathBuf;

/// What:
/// `use crate::file_owner::{Claim, Window, claim_output, observe, timed};` imports the ownership evidence.
/// Why:
/// Each manifest is attributed to the cargo call of its own root with the same rule the lockfiles use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Claim, Window, claimOutput, observe, timed } from 'crate/file_owner';
/// ```
use crate::file_owner::{Claim, Window, claim_output, observe, timed};
/// What:
/// `use crate::journal::{KeepAs, ReplaceOutcome, replace_keeping};` imports the replacement protocol.
/// Why:
/// A manifest goes back with the same protocol the lockfiles use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { KeepAs, ReplaceOutcome, replaceKeeping } from 'crate/journal';
/// ```
use crate::journal::{KeepAs, ReplaceOutcome, replace_keeping};

/// One manifest the run may rewrite.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManifestEdit {
    /// The crate root whose cargo call may rewrite the manifest.
    pub crate_dir: String,
    /// Where the manifest is.
    pub path: PathBuf,
    /// Its text before the run.
    pub before: String,
    /// Its text as this run's cargo call left it;
    /// `None` until the evidence says the call wrote it, and a manifest without it is never restored.
    pub produced: Option<String>,
}

/// What:
/// `pub fn snapshot_manifests(crate_dir, paths, edits) -> io::Result<()>` reads each manifest and adds it to `edits`.
/// Why:
/// The text must be taken before cargo runs; a manifest that cannot be read (or is not UTF-8) stops the run before anything is mutated.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function snapshotManifests(crateDir: string, paths: string[], edits: ManifestEdit[]): void  // throws when a file cannot be read
/// ```
pub fn snapshot_manifests(crate_dir: &str, paths: &[PathBuf], edits: &RefCell<Vec<ManifestEdit>>) -> io::Result<()> {
    for path in paths {
        let before: String = match observe(path)? {
            Some(found) => found.text,
            None => return Err(io::Error::new(io::ErrorKind::NotFound, format!("{} does not exist", path.display()))),
        };
        edits.borrow_mut().push(ManifestEdit { crate_dir: crate_dir.to_string(), path: path.clone(), before, produced: None });
    }
    return Ok(());
}

/// What:
/// `pub fn run_recording_manifests<T>(crate_dir, edits, run) -> io::Result<T>` runs a child that may rewrite the manifests of one
/// crate root and records as its output each of those manifests that the evidence says it wrote.
/// Why:
/// Only the manifests of the root the child ran in are claimed, so an edit to another root's member is never taken for this run's output.
/// The claim is made whether the child succeeded or failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runRecordingManifests<T>(crateDir: string, edits: ManifestEdit[], run: () => T): T
/// ```
pub fn run_recording_manifests<T>(crate_dir: &str, edits: &RefCell<Vec<ManifestEdit>>, run: impl FnOnce() -> T) -> io::Result<T> {
    let mut priors: Vec<(usize, String)> = Vec::new();
    for (position, edit) in edits.borrow().iter().enumerate() {
        if edit.crate_dir != crate_dir {
            continue;
        }
        let prior: String = match &edit.produced {
            Some(known) => known.clone(),
            None => match observe(&edit.path)? {
                Some(found) => found.text,
                None => edit.before.clone(),
            },
        };
        priors.push((position, prior));
    }
    let (result, window): (T, Window) = timed(run);
    for (position, prior) in priors {
        let path: PathBuf = edits.borrow()[position].path.clone();
        if let Claim::Ours(text) = claim_output(&path, &prior, window)? {
            edits.borrow_mut()[position].produced = Some(text);
        }
    }
    return Ok(result);
}

/// What:
/// `pub fn restore_manifests(edits, keep) -> io::Result<Vec<String>>` puts each manifest back that still holds this run's output
/// and returns a warning for each one it left alone.
/// Why:
/// Unknown ownership never authorizes a restore: a manifest with no recorded output, or one that differs from it,
/// belongs to another process; a manifest that cannot be read is an error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function restoreManifests(edits: ManifestEdit[], keep: KeepAs): string[]  // throws when a read or write fails
/// ```
pub fn restore_manifests(edits: &RefCell<Vec<ManifestEdit>>, keep: &KeepAs) -> io::Result<Vec<String>> {
    let mut warnings: Vec<String> = Vec::new();
    for edit in edits.borrow().iter() {
        let live: String = match observe(&edit.path)? {
            Some(found) => found.text,
            None => {
                warnings.push(format!("{} is missing; it was not restored", edit.path.display()));
                continue;
            }
        };
        if live == edit.before {
            continue;
        }
        match &edit.produced {
            Some(produced) if *produced == live => match replace_keeping(&edit.path, produced, &edit.before, keep)? {
                ReplaceOutcome::Replaced(_) => {}
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
