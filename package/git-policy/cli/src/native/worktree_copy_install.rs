//! What:
//!  Installation of one validated stage into a newly registered worktree:
//!  preflight of every existing destination entry, batched creation with its durable records,
//!  mode application and rollback of this installation's own paths when it fails.
//! Why:
//!  A destination that already holds an identical entry is accepted,
//!  and a differing entry is never overwritten, so the copy cannot destroy user state.
//!  Recording each batch before and after it runs bounds the work an interrupted run leaves unrecorded.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const copied = await installSnapshot({ snapshot, destinationRoot, journalState, priorCreations });
//! ```

use super::worktree_copy_compare::{entry_matches, lstat_or_absent};
use super::worktree_copy_error::{WorktreeCopyError, failure_with_cause};
use super::worktree_copy_install_entry::{
    InstalledPath, capture_installed_path, create_selected_entry, ensure_parents,
};
use super::worktree_copy_manifest::{EntryKind, WorktreeCopyEntry, apply_entry_modes};
use super::worktree_copy_rollback::rollback_created;
use super::worktree_copy_selection::filesystem_path;
use crate::json_record::quote;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// What:
///  The number of manifest entries claimed by one durable intent record and settled by one creation record.
/// Why:
///   Bounds both the number of journal appends and the work an interrupted owner can leave unrecorded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const INSTALL_BATCH_ENTRIES = 512;
/// ```
pub const INSTALL_BATCH_ENTRIES: usize = 512;

/// What:
///  The durable record of an installation in progress.
/// Why:
///   The installation reports every claim before it creates a path and every proven creation after,
///  so the journal slice can persist both without this module knowing the file format.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordIntents({ state, relativePaths }); recordCreations({ state, entries });
/// ```
pub trait InstallRecorder {
    /// Records the absent paths one batch is about to create, before any of them exists.
    fn record_intents(&mut self, relative_paths: &[String]) -> Result<(), WorktreeCopyError>;

    /// Records the identities of the paths one batch created.
    fn record_creations(&mut self, entries: &[InstalledPath]) -> Result<(), WorktreeCopyError>;
}

/// What:
///  A recorder that writes nothing, for installations whose caller keeps no durable state.
/// Why:
///   Such an installation is not crash-safe; it exists so the installer can be exercised before the journal lands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // no journal state
/// ```
#[derive(Debug, Default)]
pub struct NoRecord;

impl InstallRecorder for NoRecord {
    /// Accepts the intent without storing it.
    fn record_intents(&mut self, _relative_paths: &[String]) -> Result<(), WorktreeCopyError> {
        return Ok(());
    }

    /// Accepts the creations without storing them.
    fn record_creations(&mut self, _entries: &[InstalledPath]) -> Result<(), WorktreeCopyError> {
        return Ok(());
    }
}

/// What:
///  The selected paths absent from the destination, after every existing entry has been proven identical.
/// Why:
///   An existing entry that differs stops the installation before any mutation, and an identical one is accepted without being overwritten.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function preflightDestination({ snapshot, destinationRoot, recorded }): Promise<ReadonlySet<string>>;
/// ```
pub fn preflight_destination(
    stage_root: &Path,
    destination_root: &Path,
    entries: &[WorktreeCopyEntry],
    claimed: &BTreeSet<String>,
) -> Result<BTreeSet<String>, WorktreeCopyError> {
    let mut absent: BTreeSet<String> = BTreeSet::new();
    for entry in entries {
        let destination_path: PathBuf =
            filesystem_path(destination_root, entry.relative_path.as_str())?;
        // `match` on the existing entry: an absent path is recorded, a claimed directory is accepted, anything else must be identical.
        match lstat_or_absent(destination_path.as_path())? {
            None => {
                absent.insert(entry.relative_path.clone());
            }
            Some(metadata) => {
                let claimed_directory: bool = entry.kind == EntryKind::Directory
                    && metadata.is_dir()
                    && claimed.contains(&entry.relative_path);
                if claimed_directory {
                    continue;
                }
                if !entry_matches(stage_root, destination_root, entry)? {
                    let message: String = format!(
                        "cli-git: ignored-state copy would overwrite differing destination entry {} in {}.",
                        quote(entry.relative_path.as_str()),
                        quote(destination_root.to_string_lossy().as_ref())
                    );
                    return Err(failure_with_cause(message.as_str(), None));
                }
            }
        }
    }
    return Ok(absent);
}

/// What:
///  Creates one batch's absent entries in manifest order, recording the claim before and the identities after.
/// Why:
///   The claim is durable before the first path exists, so an interruption leaves a record of every path that might be partial.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function installBatch({ snapshot, destinationRoot, batch, absent, created, journalState }): Promise<void>;
/// ```
fn install_batch(
    stage_root: &Path,
    destination_root: &Path,
    batch: &[WorktreeCopyEntry],
    absent: &BTreeSet<String>,
    created: &mut Vec<InstalledPath>,
    recorder: &mut dyn InstallRecorder,
) -> Result<(), WorktreeCopyError> {
    let claims: Vec<String> = batch
        .iter()
        .map(|entry| return entry.relative_path.clone())
        .filter(|relative_path| return absent.contains(relative_path))
        .collect();
    recorder.record_intents(claims.as_slice())?;
    let batch_start: usize = created.len();
    for entry in batch {
        ensure_parents(destination_root, entry, created)?;
        let destination_path: PathBuf =
            filesystem_path(destination_root, entry.relative_path.as_str())?;
        // A path that appeared since preflight is never created over; the next settle step decides what it is.
        if lstat_or_absent(destination_path.as_path())?.is_some() {
            continue;
        }
        create_selected_entry(stage_root, destination_root, entry)?;
        created.push(capture_installed_path(
            destination_path.as_path(),
            entry.relative_path.as_str(),
            true,
        )?);
    }
    recorder.record_creations(&created[batch_start..])?;
    return Ok(());
}

/// What:
///  Installs a whole stage into the destination, or rolls back its own paths and reports what it kept.
/// Why:
///   A failed installation ends with the destination as it was before, apart from paths the rollback could not prove were its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function installSnapshot({ snapshot, destinationRoot, journalState, priorCreations }): Promise<number>;
/// ```
pub fn install_snapshot(
    stage_root: &Path,
    destination_root: &Path,
    entries: &[WorktreeCopyEntry],
    claimed: &BTreeSet<String>,
    prior: &[InstalledPath],
    recorder: &mut dyn InstallRecorder,
) -> Result<usize, WorktreeCopyError> {
    let mut created: Vec<InstalledPath> = prior.to_vec();
    match install_all(
        stage_root,
        destination_root,
        entries,
        claimed,
        &mut created,
        recorder,
    ) {
        Ok(copied) => return Ok(copied),
        Err(error) => {
            let retained: Vec<String> =
                rollback_created(stage_root, destination_root, entries, created.as_slice())?;
            let suffix: String = if retained.is_empty() {
                String::new()
            } else {
                let names: Vec<String> = retained
                    .iter()
                    .map(|path| return quote(path.as_str()))
                    .collect();
                format!(" Rollback retained: {}.", names.join(", "))
            };
            let message: String = format!(
                "cli-git: ignored-state installation failed for {}.{suffix}",
                quote(destination_root.to_string_lossy().as_ref())
            );
            return Err(failure_with_cause(message.as_str(), Some(&error.0)));
        }
    }
}

/// What:
///  The installation steps that run before rollback can apply: preflight, batches, and modes.
/// Why:
///   Separating them lets one rollback cover every failure after the first mutation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const absent = await preflightDestination(...); for (const batch of installBatches(...)) await installBatch(...); await applyEntryModes(...);
/// ```
fn install_all(
    stage_root: &Path,
    destination_root: &Path,
    entries: &[WorktreeCopyEntry],
    claimed: &BTreeSet<String>,
    created: &mut Vec<InstalledPath>,
    recorder: &mut dyn InstallRecorder,
) -> Result<usize, WorktreeCopyError> {
    let absent: BTreeSet<String> =
        preflight_destination(stage_root, destination_root, entries, claimed)?;
    for batch in entries.chunks(INSTALL_BATCH_ENTRIES) {
        install_batch(
            stage_root,
            destination_root,
            batch,
            &absent,
            created,
            recorder,
        )?;
    }
    apply_entry_modes(destination_root, entries)?;
    let selected: usize = created
        .iter()
        .filter(|installed| return installed.selected)
        .count();
    return Ok(selected);
}

/// Installation controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_install_tests.rs"]
mod tests;
