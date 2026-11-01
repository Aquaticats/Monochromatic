//! What:
//!  One destination's journaled installation to an end, and the synchronization of every created worktree.
//!  The journal records the phase, the install log records each batch's claims and creations,
//!  and a finished or failed installation always removes its journal and stage.
//! Why:
//!  A failed installation must not stay pending, because recovery would replay the same failure
//!  before every later command; an interrupted one must resume from exactly what it recorded.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await completeJournal({ pending, snapshot });
//! ```

use super::transaction_git::GitContext;
use super::worktree_copy_error::{WorktreeCopyError, raw_failure};
use super::worktree_copy_install::{InstallRecorder, install_snapshot};
use super::worktree_copy_install_entry::InstalledPath;
use super::worktree_copy_install_log::{InstallLog, open_install_log};
use super::worktree_copy_claim::{Claim, Claiming, claim_destination};
use super::worktree_copy_journal::{
    Journal, PendingJournal, Phase, create_journal, ensure_journal_root, remove_pending_journal, write_journal,
};
use super::worktree_copy_manifest::{WorktreeCopyEntry, collect_entry_manifest};
use super::worktree_copy_recovery::{Registration, destination_registration, report_recovered, settle_claim};
use super::worktree_copy_registry::{CreatedWorktree, copied_marker_exists, mark_copied};
use super::worktree_copy_selection::CopySelection;
use super::worktree_copy_stage::{StagedSnapshot, new_stage_container, stage_ignored_snapshot};
use crate::json_record::quote;
use std::collections::BTreeSet;
use std::io::Write;
use std::path::{Path, PathBuf};

/// What:
///  The mutable durable state of one installation: the latest journal, the open log, and every claim and creation so far.
/// Why:
///   The installer reports claims and creations through this state, so each one reaches disk before the next step.
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
    pub intended: BTreeSet<String>,
    /// Every path whose creation has been recorded so far.
    pub created_paths: BTreeSet<String>,
}

impl InstallRecorder for TransactionState {
    /// Records the claims that no earlier record holds, then remembers them.
    fn record_intents(&mut self, relative_paths: &[String]) -> Result<(), WorktreeCopyError> {
        let unclaimed: Vec<String> = relative_paths
            .iter()
            .filter(|relative_path| return !self.intended.contains(relative_path.as_str()))
            .cloned()
            .collect();
        self.log.append_intents(unclaimed.as_slice())?;
        self.intended.extend(unclaimed);
        return Ok(());
    }

    /// Records the creations no earlier record holds, then remembers them.
    fn record_creations(&mut self, entries: &[InstalledPath]) -> Result<(), WorktreeCopyError> {
        let unrecorded: Vec<InstalledPath> = entries
            .iter()
            .filter(|entry| return !self.created_paths.contains(entry.relative_path.as_str()))
            .cloned()
            .collect();
        self.log.append_creations(unrecorded.as_slice())?;
        self.created_paths.extend(
            unrecorded
                .into_iter()
                .map(|entry| return entry.relative_path),
        );
        return Ok(());
    }
}

/// What:
///  Persists the installing phase before the destination is touched.
/// Why:
///   Recovery must know the installation had started, so it reads the log instead of discarding the stage.
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
///  The validated staged payload a journal describes, rebuilt from its stage.
/// Why:
///   Recovery installs from the stage that exists now, and refuses a stage that lost a claimed path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function snapshotFromJournal({ journal, intendedEntries }): Promise<StagedWorktreeSnapshot>;
/// ```
pub fn snapshot_from_journal(
    pending: &PendingJournal,
    intended: &BTreeSet<String>,
) -> Result<StagedSnapshot, WorktreeCopyError> {
    let stage_root: PathBuf = PathBuf::from(pending.journal.stage_root.as_str());
    let outcome: Result<Vec<WorktreeCopyEntry>, WorktreeCopyError> = (|| {
        let entries: Vec<WorktreeCopyEntry> = collect_entry_manifest(
            stage_root.as_path(),
            pending.journal.selected_roots.as_slice(),
            &[],
        )?;
        let present: BTreeSet<&str> = entries
            .iter()
            .map(|entry| return entry.relative_path.as_str())
            .collect();
        for claim in intended {
            if !present.contains(claim.as_str()) {
                let message: String = format!(
                    "cli-git: worktree-copy journal intent is absent from private stage {}.",
                    quote(pending.journal.stage_root.as_str())
                );
                return Err(super::worktree_copy_error::failure_with_cause(
                    message.as_str(),
                    None,
                ));
            }
        }
        return Ok(entries);
    })();
    // Every failure here reads as a failed recovery, with its own cause after it.
    let entries: Vec<WorktreeCopyEntry> = match outcome {
        Ok(entries) => entries,
        Err(error) => {
            let message: String = format!(
                "cli-git: could not recover staged ignored state at {}.",
                quote(pending.journal.stage_root.as_str())
            );
            return Err(super::worktree_copy_error::failure_with_cause(
                message.as_str(),
                Some(&error.0),
            ));
        }
    };
    return Ok(StagedSnapshot {
        entries,
        selected_roots: pending.journal.selected_roots.clone(),
        source_root: PathBuf::from(pending.journal.source_root.as_str()),
        stage_container: PathBuf::from(pending.journal.stage_container.as_str()),
        stage_root,
    });
}

/// What:
///  The result of one installation attempt.
/// Why:
///   A failed installation is an outcome the caller reports, while a journal fault is an error that keeps the state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type InstallationOutcome = { kind: 'installed'; copiedEntries: number } | { kind: 'ended'; failure: WorktreeCopyError };
/// ```
#[derive(Debug)]
pub enum Outcome {
    /// Every selected entry exists; the count is the number this transaction created.
    Installed(usize),
    /// Installation failed after rollback; the journal and stage are removed.
    Ended(WorktreeCopyError),
}

/// What:
///  Runs one journaled installation: opens the log, seeds the claims, installs, and reports the latest journal.
/// Why:
///   Resumption and first attempts share this path, so an interrupted run's records are always honored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function installRecorded({ pending, snapshot }): Promise<{ latest; result }>;
/// ```
fn install_recorded(
    pending: PendingJournal,
    snapshot: &dyn Fn(&BTreeSet<String>) -> Result<StagedSnapshot, WorktreeCopyError>,
) -> Result<(PendingJournal, Outcome), WorktreeCopyError> {
    let stage: PathBuf = PathBuf::from(pending.journal.stage_container.as_str());
    let opened = open_install_log(stage.as_path())?;
    let mut intended: BTreeSet<String> = pending.journal.intended_entries.iter().cloned().collect();
    intended.extend(opened.recorded.intended.iter().cloned());
    let mut created_paths: BTreeSet<String> = pending
        .journal
        .created_entries
        .iter()
        .map(|entry| return entry.relative_path.clone())
        .collect();
    created_paths.extend(
        opened
            .recorded
            .created
            .iter()
            .map(|entry| return entry.relative_path.clone()),
    );
    let mut prior: Vec<InstalledPath> = pending.journal.created_entries.clone();
    prior.extend(opened.recorded.created.iter().cloned());
    let staged: StagedSnapshot = snapshot(&intended)?;
    let mut state: TransactionState = TransactionState {
        pending,
        log: opened.log,
        intended,
        created_paths,
    };
    begin_installing(&mut state)?;
    let claimed: BTreeSet<String> = state
        .intended
        .union(&state.created_paths)
        .cloned()
        .collect();
    let destination: PathBuf = PathBuf::from(state.pending.journal.destination_root.as_str());
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
///  Completes one journaled installation and removes its journal and stage, whatever the installation's outcome.
/// Why:
///   Only a journal that cannot be read or written stays behind, for inspection.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function completeJournal({ pending, snapshot }): Promise<InstallationOutcome>;
/// ```
pub fn complete_journal(
    pending: PendingJournal,
    snapshot: &dyn Fn(&BTreeSet<String>) -> Result<StagedSnapshot, WorktreeCopyError>,
) -> Result<Outcome, WorktreeCopyError> {
    let (latest, outcome) = install_recorded(pending, snapshot)?;
    remove_pending_journal(&latest)?;
    return Ok(outcome);
}

/// What:
///  How the synchronization of one created worktree ended.
/// Why:
///   Only a copy that ran counts toward the summary line; the others tell the user why nothing was copied here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Synchronized = { kind: 'copied'; entries: number } | { kind: 'already-copied' } | { kind: 'yielded'; reason: string } | { kind: 'unregistered' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Synchronized {
    /// This command ran the copy; the count is the number of entries it created.
    Copied(usize),
    /// A finished copy had already marked the worktree.
    AlreadyCopied,
    /// Another copy holds the worktree's claim, or its claim could not be settled; the text says why.
    Yielded(String),
    /// The worktree was removed before the copy installed anything, so nothing was created.
    Unregistered,
}

/// What:
///  Settles an abandoned claim found in the way of a new copy and prints its notices.
/// Why:
///   The new copy goes on after the old one is settled, and the user should read what happened to the old one first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// report(await settleClaim({ commonDir, claim }), stderr);
/// ```
fn settle_in_the_way(
    common_dir: &Path,
    claim: Claim,
    stderr: &mut dyn Write,
) -> Result<(), WorktreeCopyError> {
    return report_recovered(&settle_claim(common_dir, claim)?, stderr);
}

/// What:
///  Stages, journals and installs the ignored state of one newly created worktree, if this command wins its claim.
/// Why:
///   The claim is held from before the first staged byte until after the journal and stage are gone, and it names the
///  stage, so no interruption leaves a stage that recovery cannot find and no other copy works on the destination
///  meanwhile; a copy that loses the claim yields at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function synchronizeCreatedWorktree({ commonDir, sourceRoot, destination, registeredRoots, gitPath }): Promise<Synchronized>;
/// ```
pub fn synchronize_created_worktree(
    context: &GitContext,
    common_dir: &Path,
    source_root: &Path,
    destination: &CreatedWorktree,
    registered_roots: &[PathBuf],
    selection: &CopySelection,
    stderr: &mut dyn Write,
) -> Result<Synchronized, WorktreeCopyError> {
    return synchronize_with_checkpoints(
        context,
        common_dir,
        source_root,
        destination,
        registered_roots,
        selection,
        stderr,
        &mut |_checkpoint| {},
    );
}

/// What:
///  The points of a synchronization a test can stop at: the claim is held, and the stage is journaled.
/// Why:
///   A worktree removal or a competing copy that lands at one of these points is the race the claim exists for, and
///  a test cannot otherwise place it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Checkpoint = 'claimed' | 'staged';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Checkpoint {
    /// The claim is held and nothing is staged yet.
    Claimed,
    /// The stage is complete and its journal is written; nothing is installed yet.
    Staged,
}

/// What:
///  `synchronize_created_worktree` with a function called at each checkpoint.
/// Why:
///   The production entry passes a function that does nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await synchronizeCreatedWorktree({ ...options, checkpoint });
/// ```
pub fn synchronize_with_checkpoints(
    context: &GitContext,
    common_dir: &Path,
    source_root: &Path,
    destination: &CreatedWorktree,
    registered_roots: &[PathBuf],
    selection: &CopySelection,
    stderr: &mut dyn Write,
    checkpoint: &mut dyn FnMut(Checkpoint),
) -> Result<Synchronized, WorktreeCopyError> {
    let admin_root: PathBuf = common_dir.join("worktrees");
    if copied_marker_exists(admin_root.as_path(), destination.admin_id.as_str())? {
        return Ok(Synchronized::AlreadyCopied);
    }
    let journals: PathBuf = ensure_journal_root(common_dir)?;
    let container: PathBuf = new_stage_container(destination.root.as_path())?;
    let claiming: Claiming = claim_destination(
        journals.as_path(),
        destination.admin_id.as_str(),
        destination.root.as_path(),
        container.as_path(),
        &mut |abandoned| return settle_in_the_way(common_dir, abandoned, stderr),
    )?;
    let claim: Claim = match claiming {
        Claiming::Claimed(claim) => claim,
        Claiming::Held(reason) => return Ok(Synchronized::Yielded(reason)),
    };
    checkpoint(Checkpoint::Claimed);
    // A copy that finished between the first look and the claim leaves its marker; the claim is released unused.
    if copied_marker_exists(admin_root.as_path(), destination.admin_id.as_str())? {
        claim.release()?;
        return Ok(Synchronized::AlreadyCopied);
    }
    if destination_registration(common_dir, destination.root.as_path())? == Registration::Unregistered {
        claim.release()?;
        return Ok(Synchronized::Unregistered);
    }
    // `match` on staging: a failure has already removed its container, so only the claim is left to release.
    let snapshot: StagedSnapshot = match stage_ignored_snapshot(
        context,
        source_root,
        claim.container().as_path(),
        registered_roots,
        selection,
    ) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            claim.release()?;
            return Err(error);
        }
    };
    let pending: PendingJournal = match create_journal(common_dir, destination.root.as_path(), &snapshot) {
        Ok(pending) => pending,
        Err(error) => {
            // A failed removal replaces the journal failure, as the incumbent's cleanup throws its own error there.
            std::fs::remove_dir_all(snapshot.stage_container.as_path())
                .map_err(|removal| return raw_failure(&removal))?;
            claim.release()?;
            return Err(error);
        }
    };
    checkpoint(Checkpoint::Staged);
    // Staging takes minutes, so the worktree may have been removed meanwhile; nothing is installed into a removed one.
    if destination_registration(common_dir, destination.root.as_path())? == Registration::Unregistered {
        remove_pending_journal(&pending)?;
        claim.release()?;
        return Ok(Synchronized::Unregistered);
    }
    let staged: StagedSnapshot = snapshot.clone();
    let outcome: Outcome = complete_journal(pending, &|_intended| return Ok(staged.clone()))?;
    // The copy has ended, installed or failed, so a later arrival must not start it again.
    mark_copied(admin_root.as_path(), destination.admin_id.as_str())?;
    claim.release()?;
    return match outcome {
        Outcome::Installed(copied) => Ok(Synchronized::Copied(copied)),
        Outcome::Ended(error) => Err(error),
    };
}

/// Transaction controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_transaction_tests.rs"]
mod tests;

/// The races a settlement lock used to prevent stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_race_tests.rs"]
mod race_tests;
