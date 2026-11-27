//! What:
//!  What follows the branch move of a landing and of a restart:
//! the conclusion state and the index update,
//! and the decision
//!  whether a batch a killed or failed landing left open may be replayed.
//! Why:
//!  A replay runs after the lock the landing held was given up or lost,
//! so another program may have staged into the index since.
//!  A replay is made only over the very index file the landing saw under its lock (`service_land_start.rs`),
//! under the lock the
//!  restart takes,
//! and the head is checked again under that lock;
//! every other case leaves the index as its writers left it,
//!  says so in the service's log,
//! and finishes what does not touch the index.
//! The live landing and the restart share the one
//!  routine that finishes a batch,
//! so an alternate index and the conclusion state are settled the same way by both
//!  (`doc/planning/cli-git-main-commit-within-native-plus-five.md`,
//! "A replay only over the index the landing saw").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const done = await finishAfterMove(git, intent, lock, journal, headCheck) // true: every obligation of the batch is settled
//! ```

//! The record that finishes a batch and the gate a replay is judged by are declared beside this file:
//! `service_land_finish_after_the_move.rs` and `service_land_finish_replay_gate.rs`.

use crate::service_index_lock::remove_stale;
use crate::service_instance::instance_of;
use crate::service_journal::{
    INDEXED_EXTENSION,
    Intent,
    OPEN_EXTENSION,
    Recovery,
    list_intents,
    list_intents_read_only,
    mark_done,
    remove_start,
};
use crate::service_journal_intent_text_and_parse::parse_intent;
use crate::service_land_context::LandContext;
pub use crate::service_land_finish_after_the_move::{
    finish_after_move,
    follow_up,
};
pub(crate) use crate::service_land_finish_replay_gate::{
    Settled,
    settle,
};
use crate::service_scratch_ref_marker_of_a_batch::clear_recorded_scratch_refs;
use crate::service_scratch_ref_of_a_dropped_batch::clear_after_dropped_batch;
use crate::transaction_git::GitContext;
use std::path::PathBuf;
/// What a restart did with a batch it decided on.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Handled {
    /// The batch landed and every obligation of it is settled;
    /// it is marked done.
    Finished,
    /// The batch landed and something is left (the index lock is held,
    /// a record cannot be read);
    /// it stays open.
    StillOpen,
    /// The batch never landed;
    /// its commit is written to `refs/cli-git/kept/<number>` when Git writes that ref,
    /// and its intent and records are removed.
    Dropped,
    /// The batch waits for a reference lock that nothing proves gone,
    /// so its intent and its commit are left as they are
    /// (`held_commit_record.rs`).
    Held,
    /// The batch looks as if it never landed,
    /// and its journal is another service's:
    /// the intent and records stay on disk.
    Left,
}

/// One thing a restart did,
/// for the service's log.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryReport {
    /// The batch.
    pub seq: u64,
    /// The repository whose batch it is,
    /// which the line about it names:
    /// a journal read across a machine cannot say it otherwise.
    pub repository: PathBuf,
    /// What was decided.
    pub decision: Recovery,
    /// What was done about it.
    pub handled: Handled,
}

/// Whose journal a recovery works on,
/// which decides whether it may remove the intent of a batch that looks unlanded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum JournalOwner {
    /// This service's own:
    /// no landing of another service is in progress in it,
    /// so an unlanded intent is dropped.
    ThisService,
    /// An earlier helper's,
    /// which may be alive:
    /// its landing writes the intent before it moves the branch,
    /// so "unlanded" may be
    ///  "about to land",
    /// and the intent stays for that helper's own start-up recovery.
    EarlierHelper,
}

/// What:
///  Finishes or drops every batch the journal holds open.
/// Why:
///   Runs before each batch of a service that serves one worktree (`settle_open_batches`),
/// and before each batch of a development run that serves none,
/// so an index update
///  that gave up while another program held Git's index lock is finished by the next commit and not only by a restart.
/// An earlier release's shared journal is settled at a service's start instead
/// (`service_main.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recover(context)
/// ```
pub fn recover(context: &LandContext) -> Vec<RecoveryReport> {
    return recover_selected(context, None, JournalOwner::ThisService);
}

/// What:
///  Finishes or drops the open batches of the worktree one instance serves,
/// found in a journal that other worktrees' batches
///  may share.
/// Why:
///   The single service of the releases before one service per worktree kept one journal for every worktree,
/// in the directory
///  above each instance's own,
/// and an instance takes over the batches of its worktree there (found by the Git directory the
///  intent records,
/// which an index outside it,
/// as `GIT_INDEX_FILE` names one,
/// cannot change;
/// an intent that records none is
///  resolved from its repository).
/// A development run may also give several instances one journal.
/// Whatever belongs to
///  another worktree is left to its instance,
/// and a record that cannot be read is not any instance's to remove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recoverInstance(context, instance)
/// ```
pub fn recover_instance(
    context: &LandContext,
    instance: &str,
) -> Vec<RecoveryReport> {
    return recover_selected(context, Some(instance), JournalOwner::ThisService);
}

/// What:
///  `recover_instance` over a journal an earlier helper may still be writing:
/// a batch that landed is finished as before,
/// and a
///  batch that looks as if it never landed is left on disk,
/// never dropped.
/// Why:
///   A live landing writes its intent,
/// then waits for the index lock,
/// then moves the branch.
/// A drop in that window removes the
///  record of a commit that lands a moment later,
/// and the person's index shows it reverted if the helper dies before it
///  installs the index.
/// A finish is safe against a live landing:
/// it never moves the branch,
/// its index replay needs the lock and
///  the records the landing writes,
/// and two finishes end in one `.done`.
/// The earlier helper drops what is still unlanded when it
///  starts again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recoverInstanceWithoutDrops(context, instance)
/// ```
pub fn recover_instance_without_drops(
    context: &LandContext,
    instance: &str,
) -> Vec<RecoveryReport> {
    return recover_selected(context, Some(instance), JournalOwner::EarlierHelper);
}

/// The branch questions a restart decides an open batch from,
/// and their awaited faces,
/// kept in a file of their own to stay under the file-size limit.
#[path = "service_land_finish_branch_questions.rs"]
mod branch_questions;
use self::branch_questions::decision_for;

/// What:
///  The intent one batch's open record holds now:
///  nothing when the record is gone,
///  nothing inside the outer value when it cannot be read,
///  and the intent when it can.
/// Why:
///  A settlement rewrites a batch's intent before it moves the branch and removes the record once nothing is owed,
/// so
///  a decision about the batch has to read the record again rather than reuse a copy parsed before the settlement;
///  the
///  three answers are told apart because "gone" means the batch is finished and "unreadable" means nothing may be removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function theIntentAsItReadsNow(path): Promise<Intent | 'gone' | 'unreadable'>
/// ```
fn the_intent_as_it_reads_now(path: &std::path::Path) -> Option<Option<Intent>> {
    let text: String = std::fs::read_to_string(path).ok()?;
    return Some(parse_intent(text.as_str()));
}

fn recover_selected(
    context: &LandContext,
    instance: Option<&str>,
    owner: JournalOwner,
) -> Vec<RecoveryReport> {
    let mut reports: Vec<RecoveryReport> = Vec::new();
    let listed: Vec<(std::path::PathBuf, Option<Intent>)> = match owner {
        JournalOwner::ThisService => list_intents(context.journal.as_path(), OPEN_EXTENSION),
        JournalOwner::EarlierHelper => {
            list_intents_read_only(context.journal.as_path(), OPEN_EXTENSION)
        }
    };
    for (path, parsed) in listed {
        let Some(mut intent) = parsed else {
            // A record this build cannot read names a batch's branch values, paths, index and cleanup, and is the only thing that
            //  does: removing it would forget a landed commit's owed index update and leave the person's next commit recording
            //  the commit's own paths against the index (`doc/planning/cli-git-commit-and-landing-route-findings-on-main.md`,
            //  finding 1). It is kept and named instead, and what it owed is settled from its own record in the worktree
            //  (`owed_index_update.rs`) or by the person.
            let named: String = crate::owed_index_update::batch_number_of_path(path.as_path());
            crate::service_error!(
                "batch {named}: its open record {} cannot be read, so it is kept and nothing of it is settled; its owed index update is \
settled from the worktree's own record of it, and the file may be removed once that record is gone",
                path.display()
            );
            continue;
        };
        // With an instance named, only the batches of that instance's worktree are recovered.
        let selected: bool = match instance {
            Some(named) => instance_of(intent.git_dir.as_path()) == named,
            None => true,
        };
        if !selected {
            continue;
        }
        // The selection follows whatever this service already ran its Git with, so an absolute path it names wins over a
        //  relative one an earlier option wrote.
        let mut prefix: Vec<std::ffi::OsString> = context.git.global_prefix.clone();
        prefix.extend(crate::repository_selection::of_paths(
            intent.git_dir.as_path(),
            intent.repository.as_path(),
        ));
        let git: GitContext = GitContext {
            common_dir: Some(intent.common_dir.clone()),
            global_prefix: prefix,
            ..context.git.clone()
        };
        // A batch a reference lock keeps off its branch is not one a restart decides from the branch: the settlement tries it first,
        //  and one still waiting is left where it is for the next command (`held_commit_settle.rs`).
        let was_held: bool =
            crate::held_commit_settle::record_of(&intent, context.journal.as_path()).is_some();
        if let Some(report) = crate::held_commit_settle::settle_a_recorded_hold(
            &git,
            context.journal.as_path(),
            &intent,
        ) {
            reports.push(report);
            continue;
        }
        // A settlement that finished the hold replays the commit onto the branch as it moved and rewrites the intent to name
        //  the commit it put there, so deciding from the copy parsed earlier asks the branch about a commit it does not name:
        //  that reads as a batch which never landed, drops it, and deletes the record of the index update the landed commit
        //  still owes (`owed_index_update.rs`). A record the settlement marked done is gone, and nothing is left to decide.
        if was_held {
            match the_intent_as_it_reads_now(path.as_path()) {
                // The settlement finished every obligation of the batch, and said so by removing its record: the hold's record went
                //  with it and its intent was renamed to `.done`. A restart that says nothing about a batch it just landed and
                //  finished tells the person who asked what became of it nothing at all, and a settle request builds its whole
                //  answer from these reports: the batch would be in neither the settled nor the unanswered list, and
                //  `service settle-earlier-batches` would print no line for it and exit 0 (`earlier_service_journal_adopted.rs`,
                //  `settle_on_request`). So it is reported as what it is.
                None => {
                    reports.push(RecoveryReport {
                        seq: intent.seq,
                        repository: intent.repository.clone(),
                        decision: Recovery::Finish,
                        handled: Handled::Finished,
                    });
                    continue;
                }
                Some(Some(fresh)) => intent = fresh,
                // A record this build cannot read is kept and named rather than dropped: removing it would forget a landed commit's
                //  owed index update (`owed_index_update.rs`).
                Some(None) => {
                    crate::held_commit_settle::say_settled_batch_record_cannot_be_read(
                        &intent,
                        path.as_path(),
                    );
                    continue;
                }
            }
        }
        let Some(decision) = decision_for(&git, &intent) else {
            crate::service_error!(
                "batch {} stays open: Git gave no answer about {} {}, so its intent is kept",
                intent.seq,
                intent.reference,
                crate::service_timing::in_the_repository_at(intent.repository.as_path())
            );
            continue;
        };
        let handled: Handled = match (decision, owner) {
            // The branch holds no batch of this one's kind: a held batch's own record decided above, so nothing here reaches Hold.
            (Recovery::Hold, _) => Handled::Held,
            (Recovery::Drop, JournalOwner::EarlierHelper) => Handled::Left,
            (Recovery::Drop, JournalOwner::ThisService) => {
                crate::service_land_kept_commit::keep_what_the_branch_does_not_show(&git, &intent);
                remove_stale(intent.index.as_path(), intent.seq);
                // The batch will never land, so the scratch reference its killed landing left is removed here; the commit it
                //  holds is kept first, on the batch's side reference (`service_scratch_ref_of_a_dropped_batch.rs`).
                clear_after_dropped_batch(&git, intent.repository.as_path(), intent.seq);
                crate::service_server_land_group::forget_the_pushes_of(&intent);
                // A batch that will never land owes no index update, so any record of one is forgotten with it, and so is the copy
                //  of the conclusion state it would have removed.
                crate::owed_index_update::remove(intent.git_dir.as_path(), intent.seq);
                crate::conclusion_state_copy::remove(context.journal.as_path(), intent.seq);
                let _removed = std::fs::remove_file(&path);
                let _record_removed = std::fs::remove_file(crate::service_journal::batch_path(
                    context.journal.as_path(),
                    intent.seq,
                    INDEXED_EXTENSION,
                ));
                remove_start(context.journal.as_path(), intent.seq);
                Handled::Dropped
            }
            (Recovery::Finish, _) => {
                // The batch landed, so its pushes are owed: a service killed before it held them leaves recovery to do it.
                crate::service_server_land_group::hold_the_pushes_of(&intent);
                remove_stale(intent.index.as_path(), intent.seq);
                if matches!(
                    settle(&git, &intent, context.journal.as_path()),
                    Settled::Done
                ) {
                    let _marked = mark_done(context.journal.as_path(), intent.seq);
                    Handled::Finished
                } else {
                    Handled::StillOpen
                }
            }
        };
        reports.push(RecoveryReport {
            seq: intent.seq,
            repository: intent.repository.clone(),
            decision,
            handled,
        });
    }
    // A landing killed between its import and its intent leaves its batch's scratch reference with no intent to settle it, and
    //  the record it wrote is the only thing that names that reference's worktree: the sweep keeps the commit it holds and then
    //  removes it (`service_scratch_ref_marker_of_a_batch.rs`). An earlier helper's journal is its own to settle.
    if owner == JournalOwner::ThisService {
        clear_recorded_scratch_refs(context, instance);
    }
    return reports;
}

/// Controls for the awaited Git questions of the restart stay out of the release executable.
#[cfg(test)]
#[path = "service_land_finish_awaiting_tests.rs"]
mod awaiting_tests;
