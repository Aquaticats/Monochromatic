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

use crate::diagnostic_log::debug;
use crate::transaction_git::GitContext;
use crate::worktree_copy_error::{WorktreeCopyError, raw_failure_of};
use crate::worktree_copy_install::{InstallRecorder, install_snapshot};
use crate::worktree_copy_install_entry::InstalledPath;
use crate::worktree_copy_install_log::{InstallLog, open_install_log};
use crate::worktree_copy_journal::{
    Journal, PendingJournal, Phase, create_journal, remove_pending_journal, write_journal,
};
use crate::worktree_copy_manifest::{WorktreeCopyEntry, collect_entry_manifest};
use crate::worktree_copy_selection::CopySelection;
use crate::worktree_copy_stage::{StagedSnapshot, stage_ignored_snapshot};
use crate::worktree_copy_stage_claim::{StageClaim, claim_stage};
use crate::json_record::quote;
use std::collections::BTreeSet;
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
                return Err(crate::worktree_copy_error::failure_with_cause(
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
            return Err(crate::worktree_copy_error::failure_with_cause(
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
        unreadable: Vec::new(),
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

/// What:
///  Stages, journals and installs the ignored state of one newly created worktree.
/// Why:
///   The stage is removed if the journal cannot be created, and a process that dies while staging leaves a claim,
///  so no stage survives without a record that recovery can act on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function synchronizeCreatedWorktree({ commonDir, sourceRoot, destinationRoot, registeredRoots, gitPath }): Promise<number>;
/// ```
pub fn synchronize_created_worktree(
    context: &GitContext,
    common_dir: &Path,
    source_root: &Path,
    destination_root: &Path,
    registered_roots: &[PathBuf],
    selection: &CopySelection,
) -> Result<usize, WorktreeCopyError> {
    return synchronize_created_worktree_noting(
        context,
        common_dir,
        source_root,
        destination_root,
        registered_roots,
        selection,
        &mut std::io::sink(),
    );
}

/// How many unreadable paths the notice names before it says how many more there are.
///  Exempt from a citation under `LQM` (class: display cap): it limits how much text a message, log field or record
///  names, and no command result or duration depends on it. Classed once in `doc/planning/cli-git-shipped-limits-
///  audit.md`, "Exempt limits"; the coordinator's pick under the owner's delegation, open to the owner's veto.
const NOTICE_PATHS: usize = 3;

/// What:
///  The one notice line for ignored entries the user could not read and the copy therefore left out.
/// Why:
///   The new worktree lacks them, so the person sees how many and which first ones, without a flood for a large set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `cli-git: left out ${count} unreadable ignored entries from ${source}: ${first.join(', ')}${more}.\n`
/// ```
fn unreadable_notice(source_root: &Path, unreadable: &[String]) -> String {
    let named: Vec<String> = unreadable.iter().take(NOTICE_PATHS).map(|path| return quote(path.as_str())).collect();
    let rest: usize = unreadable.len().saturating_sub(NOTICE_PATHS);
    let more: String = if rest == 0 { String::new() } else { format!(" and {rest} more") };
    let noun: &str = if unreadable.len() == 1 { "entry" } else { "entries" };
    return format!(
        "cli-git: left out {} unreadable ignored {noun} from {}: {}{more}.\n",
        unreadable.len(),
        quote(source_root.to_string_lossy().as_ref()),
        named.join(", ")
    );
}

/// What:
///  Stages one source's ignored state, installs it into one created worktree, and writes a notice naming the
///  entries the user could not read.
/// Why:
///   The notice goes to the writer the lifecycle holds for the person; the count of copied entries is the result.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function synchronizeCreatedWorktreeNoting({ commonDir, sourceRoot, destinationRoot, registeredRoots, selection, notice }): Promise<number>;
/// ```
pub fn synchronize_created_worktree_noting(
    context: &GitContext,
    common_dir: &Path,
    source_root: &Path,
    destination_root: &Path,
    registered_roots: &[PathBuf],
    selection: &CopySelection,
    notice: &mut dyn std::io::Write,
) -> Result<usize, WorktreeCopyError> {
    let started: std::time::Instant = std::time::Instant::now();
    let claim: StageClaim = claim_stage(common_dir, destination_root)?;
    // `match` on staging: a failure has already removed its container, so only the claim is left to release.
    let snapshot: StagedSnapshot = match stage_ignored_snapshot(
        context,
        source_root,
        claim.container(),
        registered_roots,
        selection,
    ) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            claim.release()?;
            return Err(error);
        }
    };
    let staged_done: std::time::Duration = started.elapsed();
    let journaling: std::time::Instant = std::time::Instant::now();
    let pending: PendingJournal = match create_journal(common_dir, destination_root, &snapshot) {
        Ok(pending) => pending,
        Err(error) => {
            // A failed removal replaces the journal failure, as the incumbent's cleanup throws its own error there.
            std::fs::remove_dir_all(snapshot.stage_container.as_path())
                .map_err(raw_failure_of)?;
            claim.release()?;
            return Err(error);
        }
    };
    debug(
        "worktreeCopyTransaction",
        format!("journal created in {} ms after the stage at {} ms", journaling.elapsed().as_millis(), staged_done.as_millis()).as_str(),
    );
    // The journal names the stage from here on, so the claim has done its work.
    claim.release()?;
    let staged_after: std::time::Duration = started.elapsed();
    let staged: StagedSnapshot = snapshot.clone();
    let outcome: Outcome = complete_journal(pending, &|_intended| return Ok(staged.clone()))?;
    debug(
        "worktreeCopyTransaction",
        format!(
            "settled {} entries in {} ms: stage and journal {} ms, install and completion {} ms",
            snapshot.entries.len(),
            started.elapsed().as_millis(),
            staged_after.as_millis(),
            (started.elapsed() - staged_after).as_millis()
        )
        .as_str(),
    );
    return match outcome {
        Outcome::Installed(copied) => {
            if !snapshot.unreadable.is_empty() {
                // A closed standard error leaves nowhere to print; the log line of the stage still names the entries.
                let _written = notice.write_all(unreadable_notice(source_root, snapshot.unreadable.as_slice()).as_bytes());
            }
            Ok(copied)
        }
        Outcome::Ended(error) => Err(error),
    };
}

/// Transaction controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_transaction_tests.rs"]
mod tests;
