/// The intent record a settle is given,
/// and the walk over the records a command finds,
/// named here so a caller keeps one import.
pub use crate::merge_isolation_recovery_records::{
    MergeRecoveryOutcome, merge_intents_to_recover, recover_interrupted_merges, recover_interrupted_merges_awaiting,
};
use crate::merge_isolation_journal::MergeIntent;
/// The steps shared with the settling of a moved branch;
/// named here so the paths outside this file stay the same.
pub use crate::merge_isolation_recovery_cleanup::{drop_record_and_side_ref_awaiting, partly_installed};
/// The reads the decision rests on,
/// named here so a caller keeps one import.
pub use crate::merge_isolation_recovery_reads::{
    holds_the_merge, holds_the_merges_entry, paths_the_branch_does_not_hold_awaiting, resolve_commit_awaiting, symbolic_ref_awaiting,
};
pub use crate::merge_isolation_recovery_notice::recovery_notice;
use crate::merge_isolation_recovery_notice::saved_merge_waits_for_the_index_lock;
pub use crate::merge_isolation_recovery_steps::{
    Installed, Judged, MergeRecoveryAction, MergeRefusal, RecoveryPlace, Standing, TakenOnDisk,
};
/// The synchronous faces this file keeps,
/// so the controls of the folder reach the names they had.
#[cfg(test)]
use crate::merge_isolation_recovery_steps::{finish_asides, merge_changes};
use crate::merge_isolation_recovery_cleanup::{
    a_landed_merge_in_another_branchs_worktree, drop_record, drop_side_ref_awaiting, edits_on_the_merges_own_paths,
    no_commit_names_the_merges_parent, staged_work_on_the_merges_paths,
};
/// The person's own work a merge carries,
/// in the file that carries it.
use crate::merge_isolation_recovery_carry::{
    carry_the_capture_awaiting, carry_onto_awaiting, drop_capture_awaiting, restore_the_capture_awaiting,
};
/// Running Git for the ancestry question,
/// on the runtime the caller runs on.
use crate::lazy_runtime::run_on_runtime;
use crate::transaction_git::GitRequest;
use crate::transaction_git_awaiting::run_git_checked_awaiting;
/// The install and the branch move of a merge that is finished from its record.
use crate::merge_isolation_recovery_complete::complete_awaiting;
use crate::merge_isolation_recovery_steps::{
    index_standings_awaiting, judge_changes_awaiting, merge_changes_awaiting, put_back_stranded_takes_awaiting,
};
/// The awaited face of the settle of a moved branch.
use crate::merge_isolation_moved::settle_moved_branch_awaiting;
/// The raw differences and the worktree states.
use crate::merge_isolation_changes::Change;
/// The fail-closed recovery failure.
use crate::recovery_error::RecoveryError;
/// Writing the notices of what recovery did.
use std::io::Write;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

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
    return match run_on_runtime(settle_failed_merge_awaiting(place, record, intent, out)) {
        Ok(found) => found,
        Err(refusal) => Err(MergeRefusal { error: RecoveryError(format!("{refusal}")), began: true }),
    };
}

/// What:
///  The same settling of a merge that is known not to be running,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The decision reads Git,
/// and a synchronous start registers the step's pipes by entering the runtime,
///  which is refused from inside it;
/// so a task can only reach this face,
/// which makes the same decision from the same evidence,
/// in the same order,
/// with every notice going to the sink the caller gives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function settleFailedMerge({ place, record, intent, stderr }): Promise<MergeRecoveryAction>;
/// ```
pub async fn settle_failed_merge_awaiting<W: std::io::Write + ?Sized>(
    place: &RecoveryPlace,
    record: &Path,
    intent: &MergeIntent,
    out: &mut W,
) -> Result<MergeRecoveryAction, MergeRefusal> {
    let root: &Path = intent.repository_root.as_path();
    let target: &str = intent.target_ref.as_deref().unwrap_or("HEAD");
    // The branch is resolved first, so a merge that already landed is settled from that alone: its leftover record is removed
    //  without reading a worktree that may have become unreadable since
    //  (a cross-vendor review of the eighth round, its seventh finding).
    let current: Option<String> = match resolve_commit_awaiting(place, root, target).await {
        Ok(found) => found,
        // Nothing was read, so a refusal here cannot prove the checkout untouched.
        Err(error) => return Err(MergeRefusal { error, began: true }),
    };
    match current.as_deref() {
        Some(head) if head == intent.new_oid.as_str() => {
            // The branch already names the merge, and the record is what still names the worktree its install went into: a
            //  checkout another branch owns may hold the merge's index entries and files, so the branch is read before the
            //  record goes (the owner's "never discard user data"; a trial reviewer's finding).
            let checked_out: Option<String> = symbolic_ref_awaiting(place, root).await.map_err(partly_installed)?;
            if checked_out != intent.target_ref {
                return Err(partly_installed(a_landed_merge_in_another_branchs_worktree(
                    target,
                    root,
                    checked_out.as_deref(),
                    record,
                )));
            }
            // The merge is in the branch, so the person's own work in its paths is merged with it now, and the record goes only
            //  once every byte of theirs is back in the checkout.
            // What the command that moved the checkout read is what this carry judges the paths against, so a save that
            //  arrived after a kill is left alone instead of being replaced by the person's older layer.
            let landed: Vec<crate::merge_isolation_carry_landed::LandedFile> =
                crate::merge_isolation_carry_landed::read_landed_record(place.git_dir.as_path(), intent.owner.transaction_id.as_str());
            if !carry_the_capture_awaiting(place, root, intent, landed.as_slice(), out).await? {
                return Ok(MergeRecoveryAction::IndexHeld);
            }
            if let Err(error) = drop_record_and_side_ref_awaiting(place, root, record, intent).await {
                // The branch already names this merge, so a refusal here reports it as installed.
                return Err(MergeRefusal { error, began: true });
            }
            drop_capture_awaiting(place, root, intent).await;
            return Ok(MergeRecoveryAction::AlreadyLanded);
        }
        Some(_moved) => {}
        None => return Err(MergeRefusal { error: no_commit_names_the_merges_parent(target, root, intent, record), began: true }),
    }
    // A merge whose own commits cannot be read may still be half installed, so a command is not let past this: the refusal
    //  that follows says the merge is installed even where nothing was read, which is the safe answer.
    let changes: Vec<Change> = match merge_changes_awaiting(place, root, intent).await {
        Ok(found) => found,
        Err(error) => return Err(MergeRefusal { error, began: true }),
    };
    // A file a command that stopped left under its private name is put back before the paths are judged: the path is empty
    //  while the file waits beside it, and would otherwise be read as someone else's.
    put_back_stranded_takes_awaiting(&place.git, root, place.git_dir.as_path(), changes.as_slice(), intent.carried.as_ref(), out).await;
    // What this worktree already holds of the merge is read once, the index first and the worktree second, before anything can
    //  refuse; every refusal after this point reports from this one reading, so no path can say nothing is installed while the
    //  index holds the merge.
    let index_standing: Vec<Standing> = match index_standings_awaiting(place, root, intent.old_oid.as_str(), changes.as_slice()).await {
        Ok(found) => found,
        // An index that cannot be read may hold the merge, so a command must not read the checkout past this.
        Err(error) => return Err(MergeRefusal { error, began: true }),
    };
    // Whether anything of the merge is in the index at all, which is what keeps a record from being only discarded; and whether
    //  every path the merge changes holds its side in the index, which is what an install leaves because Git writes the entries
    //  in one step.
    let any_in_index: bool = index_standing.iter().any(holds_the_merges_entry);
    let every_entry_is_the_merges: bool = !index_standing.is_empty() && index_standing.iter().all(holds_the_merges_entry);
    let judgements: Vec<Judged> = match judge_changes_awaiting(place, root, changes.as_slice()).await {
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
        Some(tip) => match paths_the_branch_does_not_hold_awaiting(place, root, tip, intent.new_oid.as_str(), changes.as_slice()).await {
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
    let checked_out: Option<String> = match symbolic_ref_awaiting(place, root).await {
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
            if !restore_the_capture_awaiting(place, root, intent, out).await? {
                return Ok(MergeRecoveryAction::IndexHeld);
            }
            if let Err(error) = drop_record(record) {
                return Err(MergeRefusal { error, began });
            }
            drop_capture_awaiting(place, root, intent).await;
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
            // A merge that landed and was then built on by another session is still the person's merge: the checkout is put back
            //  to the moved head, and their own work is merged onto it rather than written back as it was before the merge, which
            //  would show the merge's lines undone. The record stays until their bytes are in the files.
            if intent.carried.is_some() && is_ancestor_awaiting(place, root, intent.new_oid.as_str(), head).await? {
                let action: MergeRecoveryAction = settle_moving_the_record_aside_awaiting(place, record, intent, head, out).await?;
                if action != MergeRecoveryAction::BranchMoved {
                    return Ok(action);
                }
                if !carry_onto_awaiting(place, root, intent, head, out).await? {
                    return Ok(MergeRecoveryAction::IndexHeld);
                }
                if let Err(error) = drop_record(record) {
                    return Err(MergeRefusal { error, began });
                }
                drop_side_ref_awaiting(place, root, intent).await;
                drop_capture_awaiting(place, root, intent).await;
                return Ok(MergeRecoveryAction::AlreadyLanded);
            }
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
                if !restore_the_capture_awaiting(place, root, intent, out).await? {
                    return Ok(MergeRecoveryAction::IndexHeld);
                }
                if let Err(error) = drop_record(record) {
                    return Err(MergeRefusal { error, began });
                }
                drop_capture_awaiting(place, root, intent).await;
                return Ok(MergeRecoveryAction::SavedBranchMoved);
            }
            let action: MergeRecoveryAction = settle_moved_branch_awaiting(place, record, intent, head, false, out).await?;
            if action == MergeRecoveryAction::BranchMoved {
                drop_side_ref_awaiting(place, root, intent).await;
                if restore_the_capture_awaiting(place, root, intent, out).await? {
                    drop_capture_awaiting(place, root, intent).await;
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
        if !restore_the_capture_awaiting(place, root, intent, out).await? {
            return Ok(MergeRecoveryAction::IndexHeld);
        }
        if let Err(error) = drop_record_and_side_ref_awaiting(place, root, record, intent).await {
            return Err(MergeRefusal { error, began });
        }
        drop_capture_awaiting(place, root, intent).await;
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
    //  same merge (the recovery is idempotent). A merge with nothing of itself in the checkout did not wait at all, so the
    //  command has to say where that merge is.
    if complete_awaiting(place, root, record, intent, &changes, &judgements, began, out).await? == Installed::IndexHeld {
        if intent.side_ref.is_some() {
            let line: String = saved_merge_waits_for_the_index_lock(target, intent.side_ref.as_deref().unwrap_or("its side ref"), place.git_dir.join("index.lock").as_path());
            let _noted = out.write_all(line.as_bytes());
        }
        return Ok(MergeRecoveryAction::IndexHeld);
    }
    // The checkout holds the merge, so the person's own work in its paths is merged with it before the record goes. The
    //  checkout was written by this command, so no earlier reading describes it.
    if !carry_the_capture_awaiting(place, root, intent, &[], out).await? {
        return Ok(MergeRecoveryAction::IndexHeld);
    }
    drop_record_and_side_ref_awaiting(place, root, record, intent).await.map_err(partly_installed)?;
    drop_capture_awaiting(place, root, intent).await;
    return Ok(MergeRecoveryAction::Completed);
}


/// What:
///  Whether one commit is an ancestor of another,
/// which is what says a merge landed and a branch was built on it.
/// Why:
///   The two cases of a moved branch are unlike:
/// a branch built on the merge holds its content,
/// and one that was reset away does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isAncestor(place, root, ancestor, commit): boolean; // git merge-base --is-ancestor
/// ```
async fn is_ancestor_awaiting(place: &RecoveryPlace, root: &Path, ancestor: &str, commit: &str) -> Result<bool, MergeRefusal> {
    let arguments: [&str; 4] = ["merge-base", "--is-ancestor", ancestor, commit];
    return match run_git_checked_awaiting(&place.git, &GitRequest::new(root, &arguments)).await {
        Ok(_) => Ok(true),
        // `git merge-base --is-ancestor` answers 1 for "no" and anything else is a failure to read.
        Err(error) if error.to_string().contains("exit") => Ok(false),
        Err(error) => Err(MergeRefusal { error: RecoveryError(error.to_string()), began: true }),
    };
}

/// What:
///  Put the merge's paths at the moved head while the record stays,
/// so the person's work can be carried onto it.
/// Why:
///  The record is the only thing that names the capture,
/// and the carry needs it until the bytes are in the files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const action = await settleMovedBranch({ place, record, intent, movedTo, keepRecord: true });
/// ```
async fn settle_moving_the_record_aside_awaiting<W: std::io::Write + ?Sized>(
    place: &RecoveryPlace,
    record: &Path,
    intent: &MergeIntent,
    moved_to: &str,
    out: &mut W,
) -> Result<MergeRecoveryAction, MergeRefusal> {
    return settle_moved_branch_awaiting(place, record, intent, moved_to, true, out).await;
}






/// Recovery of records whose owner is dead,
/// alive or unreadable.
#[cfg(test)]
#[path = "merge_isolation_recovery_tests.rs"]
mod tests;
