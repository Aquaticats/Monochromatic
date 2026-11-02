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
    InstalledPath, capture_installed_path, create_selected_entry, ensure_parents_remembering,
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
/// const INSTALL_BATCH_ENTRIES = 4096;
/// ```
pub const INSTALL_BATCH_ENTRIES: usize = 4096;


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
    known: &mut std::collections::HashSet<String>,
) -> Result<(), WorktreeCopyError> {
    let claims: Vec<String> = batch
        .iter()
        .map(|entry| return entry.relative_path.clone())
        .filter(|relative_path| return absent.contains(relative_path))
        .collect();
    recorder.record_intents(claims.as_slice())?;
    let batch_start: usize = created.len();
    // Directories and the scaffolding above every entry first, in manifest order, so each leaf finds its parent.
    let mut leaves: Vec<&WorktreeCopyEntry> = Vec::new();
    for entry in batch {
        ensure_parents_remembering(destination_root, entry, created, known)?;
        if entry.kind != EntryKind::Directory {
            leaves.push(entry);
            continue;
        }
        let destination_path: PathBuf =
            filesystem_path(destination_root, entry.relative_path.as_str())?;
        // A path that appeared since preflight is never created over; the next settle step decides what it is.
        if lstat_or_absent(destination_path.as_path())?.is_none() {
            create_selected_entry(stage_root, destination_root, entry)?;
            created.push(capture_installed_path(
                destination_path.as_path(),
                entry.relative_path.as_str(),
                true,
            )?);
        }
        known.insert(entry.relative_path.clone());
    }
    install_leaves(stage_root, destination_root, leaves.as_slice(), created)?;
    recorder.record_creations(&created[batch_start..])?;
    return Ok(());
}

/// What:
///  The leaves per worker below which a batch's files and links are created on one thread.
/// Why:
///   A large batch spends its time in independent link calls; a small one is faster without the thread start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PARALLEL_INSTALL_MIN_LEAVES = 1024;
/// ```
const PARALLEL_INSTALL_MIN_LEAVES: usize = 1024;

/// What:
///  Creates one slice of files and links, stopping at the first failure, and returns what it created with that failure.
/// Why:
///   Rollback must know every path that exists, including those created before a failure, so the slice never discards
///  its record when it stops.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function installSlice({ stageRoot, destinationRoot, slice }): Promise<{ created: InstalledPath[]; failure?: WorktreeCopyError }>;
/// ```
fn install_slice(
    stage_root: &Path,
    destination_root: &Path,
    slice: &[&WorktreeCopyEntry],
) -> (Vec<InstalledPath>, Option<WorktreeCopyError>) {
    let mut made: Vec<InstalledPath> = Vec::new();
    for entry in slice {
        let outcome: Result<Option<InstalledPath>, WorktreeCopyError> = (|| {
            let destination_path: PathBuf =
                filesystem_path(destination_root, entry.relative_path.as_str())?;
            // A path that appeared since preflight is never created over; the next settle step decides what it is.
            if lstat_or_absent(destination_path.as_path())?.is_some() {
                return Ok(None);
            }
            create_selected_entry(stage_root, destination_root, entry)?;
            return capture_installed_path(destination_path.as_path(), entry.relative_path.as_str(), true).map(Some);
        })();
        match outcome {
            Ok(Some(installed)) => made.push(installed),
            Ok(None) => {}
            Err(error) => return (made, Some(error)),
        }
    }
    return (made, None);
}

/// What:
///  Creates the files and links of one batch, in parallel slices when the batch is large.
/// Why:
///   Their parents exist already, so the creations are independent; every created path is recorded for rollback
///  before the first failure is returned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function installLeaves({ stageRoot, destinationRoot, leaves, created }): Promise<void>;
/// ```
fn install_leaves(
    stage_root: &Path,
    destination_root: &Path,
    leaves: &[&WorktreeCopyEntry],
    created: &mut Vec<InstalledPath>,
) -> Result<(), WorktreeCopyError> {
    let workers: usize = if leaves.len() < PARALLEL_INSTALL_MIN_LEAVES {
        1
    } else {
        std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get).min(8)
    };
    let slice_length: usize = leaves.len().div_ceil(workers).max(1);
    let results: Vec<(Vec<InstalledPath>, Option<WorktreeCopyError>)> = std::thread::scope(|scope| {
        let handles: Vec<_> = leaves
            .chunks(slice_length)
            .map(|slice| return scope.spawn(move || return install_slice(stage_root, destination_root, slice)))
            .collect();
        return handles
            .into_iter()
            .map(|handle| {
                return handle.join().unwrap_or_else(|_panic| {
                    return (Vec::new(), Some(failure_with_cause("cli-git: an install worker panicked.", None)));
                });
            })
            .collect();
    });
    let mut failure: Option<WorktreeCopyError> = None;
    for (made, stopped) in results {
        created.extend(made);
        failure = failure.or(stopped);
    }
    return match failure {
        Some(error) => Err(error),
        None => Ok(()),
    };
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
    let started: std::time::Instant = std::time::Instant::now();
    let absent: BTreeSet<String> =
        preflight_destination(stage_root, destination_root, entries, claimed)?;
    let preflighted: std::time::Duration = started.elapsed();
    let mut known: std::collections::HashSet<String> = std::collections::HashSet::new();
    for batch in entries.chunks(INSTALL_BATCH_ENTRIES) {
        install_batch(
            stage_root,
            destination_root,
            batch,
            &absent,
            created,
            recorder,
            &mut known,
        )?;
    }
    let installed: std::time::Duration = started.elapsed();
    apply_entry_modes(destination_root, entries)?;
    super::diagnostic_log::debug(
        "worktreeCopyInstall",
        format!(
            "installed {} entries: preflight {} ms, create and record {} ms, modes {} ms",
            entries.len(),
            preflighted.as_millis(),
            (installed - preflighted).as_millis(),
            (started.elapsed() - installed).as_millis()
        )
        .as_str(),
    );
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
