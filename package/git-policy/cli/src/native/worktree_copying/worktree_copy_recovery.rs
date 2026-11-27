//! What:
//!  Recovery of interrupted linked-worktree copies:
//! every pending journal is resumed,
//! completed,
//! or discarded
//!  with a notice;
//! a record that cannot be read is moved aside and the command goes on
//!  (`worktree_copy_unreadable_records.rs`),
//! while a path that is not this account's private plain file fails closed and stays;
//! the pending check that forwarded commands run first.
//! Whether a destination may be finished,
//! and whether it is still a registration of this repository,
//! is judged beside this file (`worktree_copy_recovery_destinations.rs`).
//! Why:
//!  A copy interrupted by a crash must not leave a worktree half-filled forever,
//! but recovery may only act on
//!  journals whose destination is still a registration of this repository and whose stage is still private.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const report = await recoverWorktreeCopyTransactions(commonDir);
//! await recoverPendingWorktreeCopies(commonDir);
//! ```

use crate::json_record::quote;
use crate::worktree_copy_ceiling::CopyCeiling;
use crate::worktree_copy_error::{
    WorktreeCopyError,
    failure_with_cause,
    raw_failure_of,
};
use crate::worktree_copy_journal::{
    Role,
    adopt_journal,
    assert_private_path,
    journal_owner_is_dead,
    read_pending_journals,
};
use crate::worktree_copy_journal_encoding_and_validation::{
    PendingJournal,
    Phase,
};
use crate::worktree_copy_journal_removal::remove_pending_journal;
use crate::worktree_copy_owner::current_copy_owner;
/// Which destinations one pass may finish a copy into,
/// and what a destination's own registration says:
/// re-exported so a caller that recovers a copy keeps this module's path.
pub use crate::worktree_copy_recovery_destinations::{
    RecoveryDestinations,
    Registration,
    destination_registration,
    finishes_destination,
};
use crate::worktree_copy_stage_binding::{
    stage_binding,
    verify_stage_binding,
};
use crate::worktree_copy_stage_claim_owner_verdicts::StageVerdicts;
use crate::worktree_copy_stage_claim_reaping::{
    judge_stage_owners,
    reap_judged_stages,
};
use crate::worktree_copy_transaction::{
    Outcome,
    SnapshotSource,
    complete_journal,
};
use std::io::Write;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Whether a journal's private stage and payload still exist.
/// Why:
///   Installation reads the payload,
/// so a missing payload ends recovery with a notice instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StagePresence = 'stage-intact' | 'stage-missing';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StagePresence {
    /// The container and payload exist and are private.
    Intact,
    /// The container or payload is gone.
    Missing,
}

/// What:
///  One pass of recovery:
/// how many transactions completed and one notice per transaction ended otherwise.
/// Why:
///   The caller prints the notices and the summary;
/// the counts come from this one value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecoveryReport = { recovered: number; notices: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryReport {
    /// Transactions that completed,
    /// including completed cleanups.
    pub recovered: usize,
    /// One line for each transaction that ended without completing.
    pub notices: Vec<String>,
}

/// What:
///  Checks that a journal's stage container and payload are private directories,
/// or that they are gone.
/// Why:
///   Installation copies from the payload,
/// so a planted or replaced stage must never be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function validateJournalStage(record): Promise<StagePresence>;
/// ```
pub fn validate_stage(pending: &PendingJournal) -> Result<StagePresence, WorktreeCopyError> {
    let container: &Path = pending.journal.stage_container.as_path();
    let payload: &Path = pending.journal.stage_root.as_path();
    // `match` on the no-follow lookup: an absent path is reported as missing, and any other failure names the destination.
    for path in [&container, &payload] {
        match std::fs::symlink_metadata(path) {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(StagePresence::Missing);
            }
            Err(error) => {
                let message: String = format!(
                    "monochromatic-git-policy-cli: could not validate worktree-copy recovery state for {}.",
                    quote(pending.journal.destination_root.to_string_lossy().as_ref())
                );
                return Err(failure_with_cause(message.as_str(), Some(&error)));
            }
        }
        assert_private_path(path, Role::PrivateStage)?;
    }
    return Ok(StagePresence::Intact);
}

/// What:
///  Brings one pending transaction to an end.
/// Why:
///   Every journal ends as completed,
/// resumed-and-completed,
/// or discarded with its notice;
/// only unsafe state stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function recoverTransaction({ commonDir, journal }): Promise<RecoveryStep>;
/// ```
fn recover_transaction(
    common_dir: &Path,
    pending: PendingJournal,
    ceiling: &CopyCeiling,
) -> Result<(bool, Option<String>), WorktreeCopyError> {
    let destination: String = quote(pending.journal.destination_root.to_string_lossy().as_ref());
    let registration: Registration =
        destination_registration(common_dir, pending.journal.destination_root.as_path());
    let stage: StagePresence = validate_stage(&pending)?;
    if registration == Registration::Unregistered {
        remove_pending_journal(&pending)?;
        return Ok((
            false,
            Some(format!(
                "monochromatic-git-policy-cli: discarded an interrupted ignored-state copy for {destination}, which is no longer a registered worktree."
            )),
        ));
    }
    if pending.journal.phase == Phase::Complete {
        remove_pending_journal(&pending)?;
        return Ok((true, None));
    }
    if stage == StagePresence::Missing {
        remove_pending_journal(&pending)?;
        return Ok((
            false,
            Some(format!(
                "monochromatic-git-policy-cli: discarded an interrupted ignored-state copy into {destination} because its private stage is missing; ignored files there may be incomplete."
            )),
        ));
    }
    // The repository still registers a worktree at that path, and its pointer does not name that registration:
    // a rewrite can leave the file missing, cut short or naming another directory, so the copy stays and the person is told.
    if registration == Registration::Unconfirmed {
        return Ok((
            false,
            Some(format!(
                "monochromatic-git-policy-cli: kept an interrupted ignored-state copy for {destination}: this repository still registers that worktree, but its `.git` file does not name that registration, so nothing was changed. `git worktree list` shows what is registered there, and once a command has left that file naming the registration again the next command resumes the copy."
            )),
        ));
    }
    // A registration or a destination that could not be read is not an absence of one: the copy stays,
    // and the person is told which files were kept rather than which answer was assumed.
    if registration == Registration::Unproven {
        return Ok((
            false,
            Some(format!(
                "monochromatic-git-policy-cli: kept an interrupted ignored-state copy for {destination} because this repository's registrations, or that destination, could not be examined: whether a registration names that worktree cannot be told, so nothing was changed. Its journal and private stage stay where they are; `git worktree list` shows what is registered there, and the next command resumes the copy once they can be read."
            )),
        ));
    }
    let journal_for_snapshot: PendingJournal = pending.clone();
    // The stage records the registration the copy was made for; a destination that is another registration now is not
    // this copy's to fill, and nothing in it is written.
    if let Some(bound) = stage_binding(pending.journal.stage_container.as_path())?
        && let Err(reason) = verify_stage_binding(common_dir, &bound)
    {
        remove_pending_journal(&pending)?;
        return Ok((
            false,
            Some(format!(
                "monochromatic-git-policy-cli: discarded an interrupted ignored-state copy into {destination}: {reason}. Nothing was written into that worktree; `git worktree list` shows what is registered there now, and a `git worktree add` into the same path copies the ignored state afresh."
            )),
        ));
    }
    let stage_container: PathBuf = journal_for_snapshot.journal.stage_container.clone();
    // `match` on the outcome: a completed installation is recovered; a failed one becomes a notice.
    let outcome: Outcome = complete_journal(
        pending,
        &SnapshotSource::FromJournal(&journal_for_snapshot),
        ceiling,
    )?;
    // The add's claim names the stage this journal filled, and a kill between the journal's creation and the claim's
    // release leaves both; the journal's own end settles the claim, so no copy finished here leaves a claim naming a
    // stage that is gone.
    if let Some(claim) =
        crate::worktree_copy_stage_claim::claim_naming_stage(common_dir, stage_container.as_path())
    {
        match outcome {
            // The copy is made, so the add is complete and its records go.
            Outcome::Installed(_) => claim.settle()?,
            // The installation ended, and the record of the unfilled worktree now names what is missing there, so the
            // claim is released rather than marked complete.
            Outcome::Ended(_) => claim.release()?,
        }
    }
    match outcome {
        Outcome::Installed(_) => return Ok((true, None)),
        Outcome::Ended(failure) => {
            return Ok((
                false,
                Some(format!(
                    "monochromatic-git-policy-cli: ended an interrupted ignored-state copy into {destination} without finishing it. {} That worktree stays named as one whose ignored files are missing, by every command in this repository, until it is removed or replaced.",
                    failure.0
                )),
            ));
        }
    }
}

/// What:
///  Recovers every pending journal whose owner is dead,
/// each by taking it over first,
/// and reaps the stages dead owners left.
/// Why:
///   A journal whose owner still runs is a copy in progress and is left whole;
/// a dead owner's journal is taken over by one
///  atomic rename,
/// so of several commands that find it exactly one recovers it and the others move on without waiting.
/// Every owner is judged once,
/// before anything is changed,
/// so the sweep acts on one answer and an item of another namespace older than the patience
///  stops the command with nothing changed.
/// The copy an interrupted add owes is finished by [`recover_pending_worktree_copies`],
/// which holds the real Git that
///  staging needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const report = await recoverWorktreeCopyTransactions(commonDir);
/// ```
pub fn recover_worktree_copy_transactions(
    common_dir: &Path
) -> Result<RecoveryReport, WorktreeCopyError> {
    let mut verdicts: StageVerdicts = judge_stage_owners(common_dir)?;
    let mut settled: Vec<String> =
        crate::worktree_copy_unreadable_records::settle_unreadable_records(
            common_dir,
            &mut verdicts,
        )?;
    let mut report: RecoveryReport = recover_judged_transactions(
        common_dir,
        &verdicts,
        &RecoveryDestinations::EveryDestination,
        &CopyCeiling::unbounded(),
    )?;
    settled.append(&mut report.notices);
    report.notices = settled;
    return Ok(report);
}

/// What:
///  The part of the recovery that follows the judgement of every owner:
/// the journals,
/// then the stages and records a dead owner left.
/// Why:
///   The judgement must be taken before the first change,
/// so the caller that also finishes an owed copy judges once and passes the
///  same verdicts here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const report = await recoverJudgedTransactions(commonDir, verdicts);
/// ```
fn recover_judged_transactions(
    common_dir: &Path,
    verdicts: &StageVerdicts,
    destinations: &RecoveryDestinations<'_>,
    ceiling: &CopyCeiling,
) -> Result<RecoveryReport, WorktreeCopyError> {
    let pending: Vec<PendingJournal> = read_pending_journals(common_dir)?;
    let owner = current_copy_owner()?;
    let mut recovered: usize = 0;
    let mut notices: Vec<String> = Vec::new();
    let mut dead: Vec<PendingJournal> = Vec::new();
    for journal in pending {
        if journal_owner_is_dead(&journal)? {
            dead.push(journal);
        }
    }
    for journal in dead {
        // A copy into another worktree is that worktree's own work: staging or installing the source's ignored state
        // costs what the tree costs, and a command in this worktree gains nothing from it. The journal is left whole,
        // with no takeover, so a long-running command elsewhere never holds a registration this worktree needs, and the
        // first command run there finishes it. A destination no registration names has no command left to run in it, so
        // its transaction is settled here like any other.
        if !finishes_destination(destinations, journal.journal.destination_root.as_path())
            && destination_registration(common_dir, journal.journal.destination_root.as_path())
                != Registration::Unregistered
        {
            notices.push(
                crate::worktree_copy_owed_copy::owed_in_another_worktree_notice(
                    journal.journal.destination_root.as_path(),
                ),
            );
            continue;
        }
        // `match` on the take-over: another command already took this journal, and it recovers it.
        let Some(mine) = adopt_journal(&journal, &owner)? else {
            continue;
        };
        let (completed, notice) = recover_transaction(common_dir, mine, ceiling)?;
        if completed {
            recovered += 1;
        }
        if let Some(line) = notice {
            notices.push(line);
        }
    }
    notices.extend(reap_judged_stages(common_dir, verdicts, destinations)?);
    // A worktree a copy gave up on stays named until it is gone or replaced, so every command says what is missing
    // there and what settles it; a copy still running wrote its record first and is left without a word.
    notices.extend(crate::worktree_copy_unfilled_record::report_unfilled_records(common_dir)?);
    return Ok(RecoveryReport {
        recovered,
        notices,
    });
}

/// What:
///  Writes each notice on its own line,
/// then the summary line when any transaction recovered.
/// Why:
///   The user sees why each discarded copy was discarded,
/// and how many resumed copies completed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// reportRecoveredWorktreeCopies(report);
/// ```
pub fn report_recovered(
    report: &RecoveryReport,
    stderr: &mut dyn Write,
) -> Result<(), WorktreeCopyError> {
    for notice in &report.notices {
        writeln!(stderr, "{notice}").map_err(raw_failure_of)?;
    }
    if report.recovered > 0 {
        let noun: &str = if report.recovered == 1 {
            "transaction"
        } else {
            "transactions"
        };
        writeln!(
            stderr,
            "monochromatic-git-policy-cli: recovered ignored-state copies for {} worktree {noun}.",
            report.recovered
        )
        .map_err(raw_failure_of)?;
    }
    return Ok(());
}

/// What:
///  The pending check every forwarded command in a linked worktree or a bare repository runs first:
/// recovers what dead
///  owners left,
/// and does nothing when nothing is pending.
/// Why:
///   It never waits and takes no lock:
/// items of live owners are skipped,
/// and one atomic rename picks the single recoverer.
/// A copy is finished only into the worktree the command runs in,
/// and under the bound one command holds for work it did not start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recoverPendingWorktreeCopies(commonDir, destinations, stderr);
/// ```
pub fn recover_pending_worktree_copies(
    common_dir: &Path,
    destinations: RecoveryDestinations<'_>,
    context: &crate::transaction_git::GitContext,
    stderr: &mut dyn Write,
) -> Result<(), WorktreeCopyError> {
    // Any item at all is work: a journal to resume, a claim or a record to settle, a stage to reap, or a temporary a
    // killed write never renamed. An empty root costs this one listing and nothing else.
    if !crate::worktree_copy_journal::journal_items_exist(common_dir)? {
        return Ok(());
    }
    // One judgement of every owner, before anything is changed: the copy an interrupted add owes is finished first,
    // with the same verdicts the sweep below settles its claims on, into the worktree this command runs in.
    let mut verdicts: StageVerdicts = judge_stage_owners(common_dir)?;
    // The records nothing can read are settled before anything reads them, so one record a crash published before its
    // bytes never stops a command of this repository, in this worktree or any other.
    let mut settled: Vec<String> =
        crate::worktree_copy_unreadable_records::settle_unreadable_records(
            common_dir,
            &mut verdicts,
        )?;
    let ceiling: CopyCeiling =
        CopyCeiling::lasting(crate::worktree_copy_ceiling::OWED_COPY_CEILING);
    let mut finished: Vec<String> = crate::worktree_copy_owed_copy::finish_owed_worktree_copies(
        common_dir,
        &verdicts,
        &destinations,
        &ceiling,
        context,
        stderr,
    )?;
    let mut report: RecoveryReport =
        recover_judged_transactions(common_dir, &verdicts, &destinations, &ceiling)?;
    settled.append(&mut finished);
    settled.append(&mut report.notices);
    report.notices = settled;
    return report_recovered(&report, stderr);
}

/// Recovery controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_recovery_tests.rs"]
mod tests;
