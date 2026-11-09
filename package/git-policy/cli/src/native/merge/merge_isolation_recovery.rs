//! What:
//!  Recovery of an isolated merge whose process died:
//! discard it when it never touched the worktree or the index,
//!  complete it when Git had started to update them.
//! Why:
//!  The merge commit exists before the branch moves,
//! and Git updates the worktree and the index of the paths the
//!  merge changes before it moves the branch.
//!  A process killed in between leaves files and index entries of the merge beside a branch that still names the
//!  old commit,
//! which the next commit would then revert.
//!  Completing the merge from its record finishes exactly what Git had started,
//! for exactly the paths the merge
//!  changes,
//! and leaves every other staged entry and every unstaged change as it was.
//!  A path that holds neither its old nor its new content belongs to someone else,
//! so recovery stops and keeps the
//!  record instead of overwriting it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const record of await listMergeIntents(gitDir)) await recoverMerge({ record, place });
//! ```

/// Debug diagnostics.
use crate::diagnostic_log::debug;
/// The intent records and their files.
use crate::merge_isolation_journal::{
    MergeIntent, list_merge_intents, parse_merge_intent,
};
/// A merge whose branch moved while it was prepared.
use crate::merge_isolation_moved::settle_moved_branch;
/// The steps shared with the settling of a moved branch;
/// named here so the paths outside this file stay the same.
pub use crate::merge_isolation_recovery_cleanup::{drop_record_and_side_ref, partly_installed};
/// The reads the decision rests on,
/// named here so a caller keeps one import.
pub use crate::merge_isolation_recovery_reads::{
    holds_the_merge, holds_the_merges_entry, paths_the_branch_does_not_hold, resolve_commit, symbolic_ref,
};
pub use crate::merge_isolation_recovery_notice::recovery_notice;
pub use crate::merge_isolation_recovery_steps::{
    Installed, Judged, MergeRecoveryAction, MergeRefusal, RecoveryPlace, Standing, TakenOnDisk, finish_asides,
};
use crate::merge_isolation_recovery_cleanup::{
    drop_record, drop_side_ref, edits_on_the_merges_own_paths, no_commit_names_the_merges_parent, staged_work_on_the_merges_paths,
};
/// The person's own work a merge carries:
/// put back when the merge does not land,
/// merged with it when it does.
use crate::merge_isolation_carry::drop_capture_ref;
use crate::merge_isolation_carry_apply::{CarryFailure, LockWait, carry, restore};
use crate::merge_isolation_carry_notice::report_lines;
/// The install and the branch move of a merge that is finished from its record.
use crate::merge_isolation_recovery_complete::complete;
use crate::merge_isolation_recovery_steps::{
    index_standings, judge_changes, merge_changes, put_back_stranded_takes,
};
/// The raw differences and the worktree states.
use crate::merge_isolation_changes::Change;
/// The fail-closed recovery failure.
use crate::recovery_error::{RecoveryError, io_failure};
/// Owner liveness of a record.
use crate::transaction_owner::{OwnerLiveness, OwnerLivenessError, classify_transaction_owner_record};
/// Running real Git.
/// Writing the notices of what recovery did.
use std::io::Write;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// One record recovery visited.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MergeRecoveryOutcome = { record: string; action: MergeRecoveryAction; targetRef?: string; newOid: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MergeRecoveryOutcome {
    /// The record's file.
    pub record: PathBuf,
    /// What recovery did.
    pub action: MergeRecoveryAction,
    /// The ref the record names,
    /// or nothing when the merge ran on a detached `HEAD`.
    pub target_ref: Option<String>,
    /// The merge commit the record names.
    pub new_oid: String,
    /// The side ref that keeps the merge commit,
    /// for a merge that was saved because the index was locked.
    pub side_ref: Option<String>,
}

/// What:
///  Whether the worktree's Git directory holds an intent record that recovery must look at:
/// one whose owner is
///  dead,
/// or one that cannot be read.
/// Why:
///   The common case needs one failed directory read and no lock.
///  A record of a merge that still runs is not recovered,
/// so a nested command does not touch its ancestor's merge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await listMergeIntents(gitDir)).some((record) => !isLive(record))
/// ```
pub fn merge_intents_to_recover(git_dir: &Path) -> Result<bool, RecoveryError> {
    for record in list_merge_intents(git_dir)? {
        let bytes: Vec<u8> = match std::fs::read(record.as_path()) {
            Ok(found) => found,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(io_failure("reading", record.as_path(), &error)),
        };
        let Some(intent) = parse_merge_intent(bytes.as_slice()) else {
            return Ok(true);
        };
        if owner_liveness(&intent, record.as_path())? == OwnerLiveness::Dead {
            return Ok(true);
        }
    }
    return Ok(false);
}

/// What:
///  Recover every intent record whose owner is dead.
/// Why:
///   A record whose owner still runs is left alone,
/// so recovery does not race a live merge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function recoverInterruptedMerges({ place, gitDir }): Promise<MergeRecoveryOutcome[]>;
/// ```
pub fn recover_interrupted_merges(
    place: &RecoveryPlace,
    git_dir: &Path,
    out: &mut dyn Write,
) -> Result<Vec<MergeRecoveryOutcome>, RecoveryError> {
    let mut outcomes: Vec<MergeRecoveryOutcome> = Vec::new();
    // Every record is read and its owner judged before any is settled, so a record of another namespace older than the patience
    //  stops the command with nothing changed (owner, 2026-11-04, "Refuse and name it (Recommended)"; sol-1 round 3, P1 C).
    let mut planned: Vec<(PathBuf, MergeIntent, OwnerLiveness)> = Vec::new();
    for record in list_merge_intents(git_dir)? {
        let bytes: Vec<u8> = match std::fs::read(record.as_path()) {
            Ok(found) => found,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(io_failure("reading", record.as_path(), &error)),
        };
        let Some(intent) = parse_merge_intent(bytes.as_slice()) else {
            return Err(RecoveryError(format!(
                "The merge record {} is malformed. It was left in place; if no merge is running in this worktree, \
                 remove it, and check the branch and `git status` before merging again.",
                record.display()
            )));
        };
        let liveness: OwnerLiveness = owner_liveness(&intent, record.as_path())?;
        planned.push((record, intent, liveness));
    }
    for (record, intent, liveness) in planned {
        let action: MergeRecoveryAction = if liveness == OwnerLiveness::Alive {
            debug("recoverInterruptedMerges", format!("the owner of {} still runs", record.display()).as_str());
            MergeRecoveryAction::OwnerActive
        } else {
            match settle_failed_merge(place, record.as_path(), &intent, out) {
                Ok(settled) => settled,
                // A merge that was saved on a side ref has nothing of itself in the checkout, so a command that cannot finish it
                //  has nothing to be stopped for: it says where the merge is and what blocked it, and goes on. A begun merge's
                //  refusal still stops the command, because the checkout may hold its files.
                Err(refusal) if !refusal.began && intent.side_ref.is_some() => {
                    let side_ref: &str = intent.side_ref.as_deref().unwrap_or("its side ref");
                    let _noted = out.write_all(
                        format!(
                            "monochromatic-git-policy-cli: the merge saved on {side_ref} was not finished, and nothing of it is in the \
                             checkout; this command goes on. {}\n",
                            refusal.error.0
                        )
                        .as_bytes(),
                    );
                    MergeRecoveryAction::DeferredRefused
                }
                Err(refusal) => {
                    // Part of the merge is in the index or the worktree, so the command must not go on reading the checkout.
                    let side_ref: &str = intent.side_ref.as_deref().unwrap_or("its record");
                    let target: &str = intent.target_ref.as_deref().unwrap_or("HEAD");
                    return Err(RecoveryError(format!(
                        "A merge saved on {side_ref} for {target} is partly installed in this worktree: {}. Nothing more was changed \
                         and the record {} was kept; check `git status` and `git stash list` before you commit anything.",
                        refusal.error.0,
                        record.display()
                    )));
                }
            }
        };
        let outcome: MergeRecoveryOutcome = MergeRecoveryOutcome {
            record,
            action,
            target_ref: intent.target_ref.clone(),
            new_oid: intent.new_oid.clone(),
            side_ref: intent.side_ref.clone(),
        };
        if let Some(line) = recovery_notice(&outcome) {
            // A closed terminal is not a reason to stop the command the person asked for.
            let _noted = out.write_all(line.as_bytes());
        }
        outcomes.push(outcome);
    }
    return Ok(outcomes);
}

/// What:
///  Whether the record's owner still runs,
/// treating a probe the account may not make as alive.
/// Why:
///   Recovery must never touch a merge that may be live.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { return await classifyTransactionOwnerRecord(owner, path); } catch (e) { if (e.code === 'EPERM') return 'alive'; throw e; }
/// ```
fn owner_liveness(intent: &MergeIntent, record: &Path) -> Result<OwnerLiveness, RecoveryError> {
    match classify_transaction_owner_record(&intent.owner, record) {
        Ok(found) => return Ok(found),
        Err(OwnerLivenessError::Probe(error)) if error.raw_os_error() == Some(libc::EPERM) => {
            return Ok(OwnerLiveness::Alive);
        }
        Err(OwnerLivenessError::Probe(error)) => {
            return Err(RecoveryError(format!(
                "probing the owner {} of the merge record {} failed: {error}",
                intent.owner.owner_pid,
                record.display()
            )));
        }
        // Owner, 2026-11-04, "Refuse and name it (Recommended)": the command stops, and nothing has been changed.
        Err(OwnerLivenessError::ForeignRecordRefused(text)) => return Err(RecoveryError(text)),
        Err(OwnerLivenessError::Identity(error)) => {
            return Err(RecoveryError(format!(
                "reading the identity of the owner {} of the merge record {} failed: {error}",
                intent.owner.owner_pid,
                record.display()
            )));
        }
    }
}

/// What:
///  Settle a merge that is known not to be running:
/// discard it if it never began,
/// otherwise finish it.
/// Why:
///   The merge's own process calls this when Git reported a failure,
/// and recovery calls it for a dead owner;
/// both
///  need the same decision from the same evidence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function settleFailedMerge({ place, record, intent }): Promise<MergeRecoveryAction>;
/// ```
pub fn settle_failed_merge(
    place: &RecoveryPlace,
    record: &Path,
    intent: &MergeIntent,
    out: &mut dyn Write,
) -> Result<MergeRecoveryAction, MergeRefusal> {
    let root: &Path = intent.repository_root.as_path();
    let target: &str = intent.target_ref.as_deref().unwrap_or("HEAD");
    // The branch is resolved first, so a merge that already landed is settled from that alone: its leftover record is removed
    //  without reading a worktree that may have become unreadable since
    //  (a cross-vendor review of the eighth round, its seventh finding).
    let current: Option<String> = match resolve_commit(place, root, target) {
        Ok(found) => found,
        // Nothing was read, so a refusal here cannot prove the checkout untouched.
        Err(error) => return Err(MergeRefusal { error, began: true }),
    };
    match current.as_deref() {
        Some(head) if head == intent.new_oid.as_str() => {
            // The merge is in the branch, so the person's own work in its paths is merged with it now, and the record goes only
            //  once every byte of theirs is back in the checkout.
            if !carry_the_capture(place, root, intent, out)? {
                return Ok(MergeRecoveryAction::IndexHeld);
            }
            if let Err(error) = drop_record_and_side_ref(place, root, record, intent) {
                // The branch already names this merge, so a refusal here reports it as installed.
                return Err(MergeRefusal { error, began: true });
            }
            return Ok(MergeRecoveryAction::AlreadyLanded);
        }
        Some(_moved) => {}
        None => return Err(MergeRefusal { error: no_commit_names_the_merges_parent(target, root, intent, record), began: true }),
    }
    // A merge whose own commits cannot be read may still be half installed, so a command is not let past this: the refusal
    //  that follows says the merge is installed even where nothing was read, which is the safe answer.
    let changes: Vec<Change> = match merge_changes(place, root, intent) {
        Ok(found) => found,
        Err(error) => return Err(MergeRefusal { error, began: true }),
    };
    // A file a command that stopped left under its private name is put back before the paths are judged: the path is empty
    //  while the file waits beside it, and would otherwise be read as someone else's.
    put_back_stranded_takes(root, changes.as_slice(), out);
    // What this worktree already holds of the merge is read once, the index first and the worktree second, before anything can
    //  refuse; every refusal after this point reports from this one reading, so no path can say nothing is installed while the
    //  index holds the merge.
    let index_standing: Vec<Standing> = match index_standings(place, root, intent.old_oid.as_str(), changes.as_slice()) {
        Ok(found) => found,
        // An index that cannot be read may hold the merge, so a command must not read the checkout past this.
        Err(error) => return Err(MergeRefusal { error, began: true }),
    };
    // Whether anything of the merge is in the index at all, which is what keeps a record from being only discarded; and whether
    //  every path the merge changes holds its side in the index, which is what an install leaves because Git writes the entries
    //  in one step.
    let any_in_index: bool = index_standing.iter().any(holds_the_merges_entry);
    let every_entry_is_the_merges: bool = !index_standing.is_empty() && index_standing.iter().all(holds_the_merges_entry);
    let judgements: Vec<Judged> = match judge_changes(place, root, changes.as_slice()) {
        Ok(found) => found,
        // The worktree cannot be read; the index is the evidence of the install that is still there.
        Err(error) => return Err(MergeRefusal { error, began: every_entry_is_the_merges }),
    };
    let any_in_worktree: bool = judgements.iter().any(holds_the_merge);
    let every_file_is_the_merges: bool = !judgements.is_empty() && judgements.iter().all(holds_the_merge);
    // Every path holding the merge is what an install leaves.
    let every_path_is_the_merges: bool = every_entry_is_the_merges || every_file_is_the_merges;
    // Evidence of an install is a path the branch does not already hold the merge's content for and the checkout does:
    //  an independent commit of the same content gives the branch that content, while an install leaves the branch as it was,
    //  whatever another command does to the checkout afterwards.
    let branch_lacks: Vec<bool> = match current.as_deref() {
        Some(tip) => match paths_the_branch_does_not_hold(place, root, tip, intent.new_oid.as_str(), changes.as_slice()) {
            Ok(found) => found,
            Err(error) => return Err(MergeRefusal { error, began: every_path_is_the_merges }),
        },
        None => Vec::new(),
    };
    let mut evidence: bool = false;
    for (index, lacks) in branch_lacks.iter().enumerate() {
        let held: bool = index_standing.get(index) == Some(&Standing::New) || judgements.get(index).is_some_and(holds_the_merge);
        if *lacks && held {
            evidence = true;
            break;
        }
    }
    let began: bool = evidence;
    // The checkout belongs to the branch the merge was computed on: installing it into another branch's worktree would put
    //  this merge's files and index into a branch it is not for, and move a branch from a checkout that is not its own.
    let checked_out: Option<String> = match symbolic_ref(place, root) {
        Ok(found) => found,
        Err(error) => return Err(MergeRefusal { error, began }),
    };
    if checked_out != intent.target_ref {
        // A record that names no side ref and has nothing of the merge in this checkout has nothing to finish and nothing to stop
        //  a command for: the branch it was computed on is not the one here, and no index entry or file of it is either. Nothing
        //  was saved of it, so the person runs the merge again on the branch it is for; leaving the record kept every command on
        //  another branch refused for a merge nothing installed
        //  (a cross-vendor review of the ninth round,
        // its fifth finding).
        // A merge kept on a side ref says where the commit is; one that was only recorded has nothing to promise about it.
        let kept: &str = match intent.side_ref.is_some() {
            true => " and the merge's commit is still where it was saved",
            false => "",
        };
        if intent.side_ref.is_none() && !any_in_index && !any_in_worktree {
            if !restore_the_capture(place, root, intent, out)? {
                return Ok(MergeRecoveryAction::IndexHeld);
            }
            if let Err(error) = drop_record(record) {
                return Err(MergeRefusal { error, began });
            }
            drop_capture(place, root, intent);
            return Ok(MergeRecoveryAction::Discarded);
        }
        return Err(MergeRefusal {
            error: RecoveryError(format!(
            "A merge saved for {target} in {} was not finished: this worktree has {} checked out, and finishing the merge would \
             put it into that branch's checkout instead. Nothing was changed and the record {} was kept{kept}. Check out {target} \
             in this worktree and run any git command again.",
            root.display(),
            checked_out.as_deref().unwrap_or("a detached HEAD"),
            record.display()
            )),
            began,
        });
    }
    match current.as_deref() {
        Some(head) if head == intent.old_oid => {}
        Some(head) => {
            if intent.side_ref.is_some() {
                // A merge that is partly installed here has files and index entries nobody else can settle: its record stays,
                //  and the command is told, because dropping it would leave them with nothing that ever recovers them.
                if began {
                    return Err(MergeRefusal {
                        error: RecoveryError(format!(
                            "A merge saved for {target} in {} is partly installed in this worktree, and {target} has moved to {} \
                             since. Its index entries and any file it had set aside are still here, so the record {} was kept and \
                             nothing was removed; the merge's commit is still where it was saved. Check `git status`, finish or \
                             undo what is staged, then remove the record {} when the checkout is as you want it.",
                            root.display(),
                            head,
                            record.display(),
                            record.display()
                        )),
                        began,
                    });
                }
                // A merge that a command already reported as saved must not vanish: the record goes, so nothing stops a later
                //  command, and the side ref keeps its commit and is named.
                if !restore_the_capture(place, root, intent, out)? {
                    return Ok(MergeRecoveryAction::IndexHeld);
                }
                if let Err(error) = drop_record(record) {
                    return Err(MergeRefusal { error, began });
                }
                drop_capture(place, root, intent);
                return Ok(MergeRecoveryAction::SavedBranchMoved);
            }
            let action: MergeRecoveryAction = settle_moved_branch(place, record, intent, head, out)?;
            if action == MergeRecoveryAction::BranchMoved {
                drop_side_ref(place, root, intent);
                if restore_the_capture(place, root, intent, out)? {
                    drop_capture(place, root, intent);
                }
            }
            return Ok(action);
        }
        None => return Err(MergeRefusal { error: no_commit_names_the_merges_parent(target, root, intent, record), began }),
    }
    // A merge that was saved on a side ref is finished even though nothing of it is in the checkout yet: the record with a side
    //  ref is what says the merge should land, not the checkout (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
    //  finding 3).
    if intent.side_ref.is_none() && !any_in_index && !any_in_worktree {
        if !restore_the_capture(place, root, intent, out)? {
            return Ok(MergeRecoveryAction::IndexHeld);
        }
        if let Err(error) = drop_record_and_side_ref(place, root, record, intent) {
            return Err(MergeRefusal { error, began });
        }
        return Ok(MergeRecoveryAction::Discarded);
    }
    let mut foreign: Vec<String> = Vec::new();
    for (change, judged) in changes.iter().zip(judgements.iter()) {
        if judged.standing == Standing::Foreign {
            foreign.push(String::from_utf8_lossy(change.path.as_slice()).into_owned());
        }
    }
    if !foreign.is_empty() {
        return Err(MergeRefusal { error: edits_on_the_merges_own_paths(root, foreign.as_slice(), record), began });
    }
    // A staged version of a path the merge replaces has no other name, so it is never written over: the finish stops and names
    //  the paths, and the staged content stays in the index.
    let mut staged: Vec<String> = Vec::new();
    for (change, standing) in changes.iter().zip(index_standing.iter()) {
        if *standing == Standing::Foreign {
            staged.push(String::from_utf8_lossy(change.path.as_slice()).into_owned());
        }
    }
    if !staged.is_empty() {
        return Err(MergeRefusal { error: staged_work_on_the_merges_paths(target, root, staged.as_slice(), record), began });
    }
    // A lock that outlasts the wait leaves everything as it was; the record stays for the next command, which finishes the
    //  same merge (the recovery is idempotent).
    if complete(place, root, record, intent, &changes, &judgements, began, out)? == Installed::IndexHeld {
        return Ok(MergeRecoveryAction::IndexHeld);
    }
    // The checkout holds the merge, so the person's own work in its paths is merged with it before the record goes.
    if !carry_the_capture(place, root, intent, out)? {
        return Ok(MergeRecoveryAction::IndexHeld);
    }
    drop_record_and_side_ref(place, root, record, intent).map_err(partly_installed)?;
    return Ok(MergeRecoveryAction::Completed);
}

/// What:
///  Merge the person's own work in the merge's paths with the merge,
/// and say whether it is back.
/// Why:
///   A merge that landed holds the merge's content at those paths,
/// so their bytes are in the capture and nowhere else;
/// the
///  carry is idempotent,
/// so a kill between two paths is finished by the next command with the same answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function carryTheCapture({ place, root, intent, out }): Promise<boolean>;
/// ```
fn carry_the_capture(
    place: &RecoveryPlace,
    root: &Path,
    intent: &MergeIntent,
    out: &mut dyn Write,
) -> Result<bool, MergeRefusal> {
    let Some(capture) = intent.carried.as_ref() else { return Ok(true) };
    let changes: Vec<Change> = merge_changes(place, root, intent).map_err(partly_installed)?;
    match carry(&place.git, root, place.git_dir.as_path(), changes.as_slice(), capture, LockWait::WhileFinishing, out) {
        Ok(report) => {
            for line in report_lines(intent.target_ref.as_deref().unwrap_or("HEAD"), &report) {
                let _noted = out.write_all(format!("{line}\n").as_bytes());
            }
            return Ok(true);
        }
        // A lock that stays held leaves everything to the next command, which carries the same bytes.
        Err(CarryFailure::IndexHeld(lock)) => {
            debug("recoverInterruptedMerges", format!("the work a merge carries waits for {lock}").as_str());
            return Ok(false);
        }
        Err(other) => return Err(MergeRefusal { error: RecoveryError(the_work_could_not_be_carried(&other)), began: false }),
    }
}

/// What:
///  Put the person's own work back exactly as it was captured,
/// for a merge that does not land.
/// Why:
///   The clear may have run before the merge did,
/// so the checkout can hold the branch's content where their bytes
///  were;
/// a merge that is abandoned gives them back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function restoreTheCapture({ place, root, intent, out }): Promise<boolean>;
/// ```
fn restore_the_capture(
    place: &RecoveryPlace,
    root: &Path,
    intent: &MergeIntent,
    out: &mut dyn Write,
) -> Result<bool, MergeRefusal> {
    let Some(capture) = intent.carried.as_ref() else { return Ok(true) };
    let changes: Vec<Change> = merge_changes(place, root, intent).map_err(partly_installed)?;
    match restore(&place.git, root, place.git_dir.as_path(), changes.as_slice(), capture, LockWait::WhileFinishing, out) {
        Ok(()) => return Ok(true),
        // A lock that stays held leaves everything to the next command.
        Err(CarryFailure::IndexHeld(lock)) => {
            debug("recoverInterruptedMerges", format!("the work a merge puts back waits for {lock}").as_str());
            return Ok(false);
        }
        Err(other) => return Err(MergeRefusal { error: RecoveryError(the_work_could_not_be_carried(&other)), began: false }),
    }
}

/// The refusal for work of the person's that could not be put back,
/// naming what stopped it and where the bytes are.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theWorkCouldNotBeCarried = (failure: CarryFailure) => `... ${failure.detail} ...`;
/// ```
fn the_work_could_not_be_carried(failure: &CarryFailure) -> String {
    let detail: String = match failure {
        CarryFailure::IndexHeld(text) | CarryFailure::Failed(text) => String::from(text.as_str()),
        CarryFailure::Foreign(path) => format!(
            "{} changed after the merge captured it",
            crate::path_bytes::path_message_text(path.as_slice())
        ),
    };
    return format!(
        "A merge in this worktree keeps the work its paths held, and that work could not be carried into them yet: {detail}. The bytes are \
         in the objects the merge's record names, and the record was kept; the next git command in this worktree tries again. Nothing of \
         the person's was removed."
    );
}

/// Drop the ref that kept a capture,
/// once its bytes are back in the checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function dropCapture({ place, root, intent }): void;
/// ```
fn drop_capture(place: &RecoveryPlace, root: &Path, intent: &MergeIntent) {
    if let Some(capture) = intent.carried.as_ref() {
        drop_capture_ref(&place.git, root, capture);
    }
}

/// Recovery of records whose owner is dead,
/// alive or unreadable.
#[cfg(test)]
#[path = "merge_isolation_recovery_tests.rs"]
mod tests;
