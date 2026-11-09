//! What:
//!  One destination's journaled installation to an end,
//! and the synchronization of every created worktree.
//!  The journal records the phase,
//! the install log records each batch's claims and creations,
//!  and a finished or failed installation always removes its journal and stage.
//! Why:
//!  A failed installation must not stay pending,
//! because recovery would replay the same failure
//!  before every later command;
//! an interrupted one must resume from exactly what it recorded.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await completeJournal({ pending, snapshot });
//! ```

use crate::diagnostic_log::debug;
use crate::json_record::quote;
use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_copy_install::{InstallRecorder, install_snapshot};
use crate::worktree_copy_install_entry::InstalledPath;
use crate::worktree_copy_install_log::{InstallLog, open_install_log};
use crate::worktree_copy_journal::write_journal;
use crate::worktree_copy_journal_encoding_and_validation::{Journal, PendingJournal, Phase};
use crate::worktree_copy_journal_removal::remove_pending_journal;
use crate::worktree_copy_manifest::{WorktreeCopyEntry, collect_entry_manifest};
use crate::worktree_copy_stage::StagedSnapshot;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// What:
///  The mutable durable state of one installation:
/// the latest journal,
/// the open log,
/// and every claim and creation so far.
/// Why:
///   The installer reports claims and creations through this state,
/// so each one reaches disk before the next step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type JournalState = { pending; log; intended: Set<string>; createdPaths: Set<string> };
/// ```
pub struct TransactionState {
    /// The latest durable journal.
    pub pending: PendingJournal,
    /// The open append-only log of this stage.
    pub log: InstallLog,
    /// Every selected path claimed so far.
    pub intended: BTreeSet<Vec<u8>>,
    /// Every path whose creation has been recorded so far.
    pub created_paths: BTreeSet<Vec<u8>>,
}

impl InstallRecorder for TransactionState {
    /// Records the claims that no earlier record holds,
    /// then remembers them.
    fn record_intents(&mut self, relative_paths: &[Vec<u8>]) -> Result<(), WorktreeCopyError> {
        let mut unclaimed: Vec<Vec<u8>> = Vec::new();
        for relative_path in relative_paths {
            if !self.intended.contains(relative_path.as_slice()) {
                unclaimed.push(relative_path.clone());
            }
        }
        self.log.append_intents(unclaimed.as_slice())?;
        self.intended.extend(unclaimed);
        return Ok(());
    }

    /// Records the creations no earlier record holds,
    /// then remembers them.
    fn record_creations(&mut self, entries: &[InstalledPath]) -> Result<(), WorktreeCopyError> {
        let mut unrecorded: Vec<InstalledPath> = Vec::new();
        for entry in entries {
            if !self.created_paths.contains(entry.relative_path.as_slice()) {
                unrecorded.push(entry.clone());
            }
        }
        self.log.append_creations(unrecorded.as_slice())?;
        for entry in unrecorded {
            self.created_paths.insert(entry.relative_path);
        }
        return Ok(());
    }
}

/// What:
///  Persists the installing phase before the destination is touched.
/// Why:
///   Recovery must know the installation had started,
/// so it reads the log instead of discarding the stage.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function beginInstalling(state): Promise<void>;
/// ```
pub fn begin_installing(state: &mut TransactionState) -> Result<(), WorktreeCopyError> {
    if state.pending.journal.phase == Phase::Installing {
        return Ok(());
    }
    let installing: Journal = Journal {
        phase: Phase::Installing,
        ..state.pending.journal.clone()
    };
    write_journal(state.pending.path.as_path(), &installing)?;
    state.pending.journal = installing;
    return Ok(());
}

/// What:
///  Read the stage's manifest and check that it holds every path the journal says was claimed.
/// Why:
///   A claim the stage lacks means the stage is not the one the journal describes,
/// and nothing may be installed from it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const entries = collectEntryManifest(stageRoot, selectedRoots); for (const claim of intended) if (!has(entries, claim)) throw ...; return entries;
/// ```
fn stage_entries_holding_intents(pending: &PendingJournal, stage_root: &Path, intended: &BTreeSet<Vec<u8>>) -> Result<Vec<WorktreeCopyEntry>, WorktreeCopyError> {
    let entries: Vec<WorktreeCopyEntry> = collect_entry_manifest(stage_root, pending.journal.selected_roots.as_slice(), &[])?;
    let mut present: BTreeSet<&[u8]> = BTreeSet::new();
    for entry in &entries {
        present.insert(entry.relative_path.as_slice());
    }
    for claim in intended {
        if !present.contains(claim.as_slice()) {
            let message: String = format!(
                "monochromatic-git-policy-cli: worktree-copy journal intent is absent from private stage {}.",
                quote(pending.journal.stage_root.to_string_lossy().as_ref())
            );
            return Err(crate::worktree_copy_error::failure_with_cause(message.as_str(), None));
        }
    }
    return Ok(entries);
}

/// What:
///  The validated staged payload a journal describes,
/// rebuilt from its stage.
/// Why:
///   Recovery installs from the stage that exists now,
/// and refuses a stage that lost a claimed path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function snapshotFromJournal({ journal, intendedEntries }): Promise<StagedWorktreeSnapshot>;
/// ```
pub fn snapshot_from_journal(
    pending: &PendingJournal,
    intended: &BTreeSet<Vec<u8>>,
) -> Result<StagedSnapshot, WorktreeCopyError> {
    let stage_root: &Path = pending.journal.stage_root.as_path();
    let outcome: Result<Vec<WorktreeCopyEntry>, WorktreeCopyError> = stage_entries_holding_intents(pending, stage_root, intended);
    // Every failure here reads as a failed recovery, with its own cause after it.
    let entries: Vec<WorktreeCopyEntry> = match outcome {
        Ok(entries) => entries,
        Err(error) => {
            let message: String = format!(
                "monochromatic-git-policy-cli: could not recover staged ignored state at {}.",
                quote(pending.journal.stage_root.to_string_lossy().as_ref())
            );
            return Err(crate::worktree_copy_error::failure_with_cause(
                message.as_str(),
                Some(&error.0),
            ));
        }
    };
    return Ok(StagedSnapshot {
        entries,
        selected_roots: pending.journal.selected_roots.clone(),
        source_root: pending.journal.source_root.clone(),
        stage_container: pending.journal.stage_container.clone(),
        stage_root: stage_root.to_path_buf(),
        unreadable: Vec::new(),
        unsettled: Vec::new(),
    });
}

/// What:
///  The result of one installation attempt.
/// Why:
///   A failed installation is an outcome the caller reports,
/// while a journal fault is an error that keeps the state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type InstallationOutcome = { kind: 'installed'; copiedEntries: number } | { kind: 'ended'; failure: WorktreeCopyError };
/// ```
#[derive(Debug)]
pub enum Outcome {
    /// Every selected entry exists;
    /// the count is the number this transaction created.
    Installed(usize),
    /// Installation failed after rollback;
    /// the journal and stage are removed.
    Ended(WorktreeCopyError),
}

/// What:
///  Where the installation gets the snapshot of the stage it installs from:
/// the one just made,
/// or the one rebuilt from the journal.
/// Why:
///   A resumed installation has no snapshot in memory,
/// so it rebuilds one from the stage and checks it holds every path the journal claimed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SnapshotSource = { staged: StagedSnapshot } | { fromJournal: PendingJournal }
/// ```
pub enum SnapshotSource<'a> {
    /// The snapshot the staging step just made.
    Staged(&'a StagedSnapshot),
    /// The snapshot rebuilt from the stage that a journal names.
    FromJournal(&'a PendingJournal),
}

impl SnapshotSource<'_> {
    /// The snapshot,
    /// given the paths the journal and the install log say were claimed.
    fn staged_snapshot(&self, intended: &BTreeSet<Vec<u8>>) -> Result<StagedSnapshot, WorktreeCopyError> {
        return match self {
            SnapshotSource::Staged(staged) => Ok((*staged).clone()),
            SnapshotSource::FromJournal(pending) => snapshot_from_journal(pending, intended),
        };
    }
}

/// What:
///  Runs one journaled installation:
/// opens the log,
/// seeds the claims,
/// installs,
/// and reports the latest journal.
/// Why:
///   Resumption and first attempts share this path,
/// so an interrupted run's records are always honored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function installRecorded({ pending, snapshot }): Promise<{ latest; result }>;
/// ```
fn install_recorded(
    pending: PendingJournal,
    snapshot: &SnapshotSource<'_>,
) -> Result<(PendingJournal, Outcome), WorktreeCopyError> {
    let opened = open_install_log(pending.journal.stage_container.as_path())?;
    let mut intended: BTreeSet<Vec<u8>> = pending.journal.intended_entries.iter().cloned().collect();
    intended.extend(opened.recorded.intended.iter().cloned());
    let mut created_paths: BTreeSet<Vec<u8>> = BTreeSet::new();
    for entry in &pending.journal.created_entries {
        created_paths.insert(entry.relative_path.clone());
    }
    for entry in &opened.recorded.created {
        created_paths.insert(entry.relative_path.clone());
    }
    let mut prior: Vec<InstalledPath> = pending.journal.created_entries.clone();
    prior.extend(opened.recorded.created.iter().cloned());
    let staged: StagedSnapshot = snapshot.staged_snapshot(&intended)?;
    let mut state: TransactionState = TransactionState {
        pending,
        log: opened.log,
        intended,
        created_paths,
    };
    begin_installing(&mut state)?;
    let claimed: BTreeSet<Vec<u8>> = state
        .intended
        .union(&state.created_paths)
        .cloned()
        .collect();
    let destination: PathBuf = state.pending.journal.destination_root.clone();
    // The last moment before the first entry: every entry is written through this path, and the registration the stage
    // was made for is the only destination this copy may fill. A check earlier in the command cannot cover a destination
    // that another command moved and made again since.
    if let Some(bound) = crate::worktree_copy_stage_binding::stage_binding(state.pending.journal.stage_container.as_path())?
        && crate::worktree_add_binding::path_identity(bound.destination.as_path()) != Some(bound.destination_id)
    {
        let message: String = format!(
            "monochromatic-git-policy-cli: the ignored state for {} was not installed: the worktree is no longer the registration the copy was made for ({}). Nothing in it was changed; `git worktree list` shows what is registered there now.",
            quote(destination.to_string_lossy().as_ref()),
            bound.admin.display()
        );
        return Err(crate::worktree_copy_error::failure_with_cause(message.as_str(), None));
    }
    let result: Result<usize, WorktreeCopyError> = install_snapshot(
        staged.stage_root.as_path(),
        destination.as_path(),
        staged.entries.as_slice(),
        &claimed,
        prior.as_slice(),
        &mut state,
    );
    let TransactionState {
        pending: latest, ..
    } = state;
    // `match` turns an installation failure into an outcome, and leaves journal faults as errors.
    match result {
        Ok(copied) => return Ok((latest, Outcome::Installed(copied))),
        Err(error) => return Ok((latest, Outcome::Ended(error))),
    }
}

/// What:
///  Completes one journaled installation and removes its journal and stage,
/// whatever the installation's outcome.
/// Why:
///   Only a journal that cannot be read or written stays behind,
/// for inspection.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function completeJournal({ pending, snapshot }): Promise<InstallationOutcome>;
/// ```
pub fn complete_journal(
    pending: PendingJournal,
    snapshot: &SnapshotSource<'_>,
) -> Result<Outcome, WorktreeCopyError> {
    let started: std::time::Instant = std::time::Instant::now();
    let (latest, outcome) = install_recorded(pending, snapshot)?;
    let recorded: std::time::Duration = started.elapsed();
    remove_pending_journal(&latest)?;
    debug(
        "worktreeCopyTransaction",
        format!(
            "completed: install {} ms, stage and journal removal {} ms",
            recorded.as_millis(),
            (started.elapsed() - recorded).as_millis()
        )
        .as_str(),
    );
    return Ok(outcome);
}

/// Transaction controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_transaction_tests.rs"]
mod tests;
