/// Running Git for the settle,
/// on the runtime the caller runs on.
use crate::lazy_runtime::run_on_runtime;
/// The change one path of a merge makes,
/// kept so the controls of the folder reach the name they had.
#[cfg(test)]
use crate::merge_isolation_changes::Change;
use crate::merge_isolation_journal::MergeIntent;
/// The settle of a merge whose branch is not at the commit it was computed on.
use crate::merge_isolation_recovery_a_branch_that_moved::a_branch_that_moved_awaiting;
/// The settle of a merge whose branch this worktree does not have checked out.
use crate::merge_isolation_recovery_a_checkout_for_another_branch::a_checkout_for_another_branch_awaiting;
/// The settle of a merge whose commit the branch already names.
use crate::merge_isolation_recovery_a_merge_the_branch_already_names::a_merge_the_branch_already_names_awaiting;
/// The person's own work a merge carries,
/// in the file that carries it.
use crate::merge_isolation_recovery_carry::{
    carry_the_capture_awaiting,
    drop_the_records_refs_then_the_record_or_say,
    restore_the_capture_awaiting,
};
use crate::merge_isolation_recovery_cleanup::SideRef;
/// Settling a saved merge whose paths hold the person's own newer edits.
use crate::merge_isolation_recovery_cleanup::settle_a_saved_merge_over_newer_edits;
use crate::merge_isolation_recovery_cleanup::{
    edits_on_the_merges_own_paths,
    no_commit_names_the_merges_parent,
    staged_work_on_the_merges_paths,
};
/// The install and the branch move of a merge that is finished from its record.
use crate::merge_isolation_recovery_complete::complete_awaiting;
use crate::merge_isolation_recovery_notice::saved_merge_waits_for_the_index_lock;
pub use crate::merge_isolation_recovery_place::RecoveryPlace;
/// The ancestry question the moved-branch settling asks,
/// named here so its callers outside this file keep one import.
pub(crate) use crate::merge_isolation_recovery_reads::is_ancestor_awaiting;
/// The reads the decision rests on,
/// named here so a caller keeps one import.
pub use crate::merge_isolation_recovery_reads::{
    holds_the_merge,
    holds_the_merges_entry,
    paths_the_branch_does_not_hold_awaiting,
    resolve_commit_awaiting,
    symbolic_ref_awaiting,
    the_capture_holds_every_file_in_the_way,
    the_paths_hold_only_the_persons_own_bytes_awaiting,
};
/// The intent record a settle is given,
/// named here so a caller of the recovery keeps one import.
pub use crate::merge_isolation_recovery_record::MergeRecoveryOutcome;
pub use crate::merge_isolation_recovery_record::partly_installed;
pub use crate::merge_isolation_recovery_record::{
    IndexHeldAt,
    Installed,
    MergeRecoveryAction,
    MergeRefusal,
};
pub use crate::merge_isolation_recovery_steps::TakenOnDisk;
/// The synchronous faces this file keeps,
/// so the controls of the folder reach the names they had.
#[cfg(test)]
use crate::merge_isolation_recovery_steps::{
    finish_asides,
    merge_changes,
};
/// What this worktree already holds of the merge,
/// and the reading that says whether it began.
use crate::merge_isolation_recovery_the_evidence_of_an_install::{
    TheEvidence,
    the_evidence_of_an_install_awaiting,
};
pub use crate::merge_isolation_the_merges_paths_judged::{
    Judged,
    Standing,
};
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
        Err(refusal) => Err(MergeRefusal {
            error: RecoveryError(format!("{refusal}")),
            began: true,
        }),
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
        Err(error) => {
            return Err(MergeRefusal {
                error,
                began: true,
            });
        }
    };
    match current.as_deref() {
        Some(head) if head == intent.new_oid.as_str() => {
            return a_merge_the_branch_already_names_awaiting(place, record, intent, out).await;
        }
        Some(_moved) => {}
        None => {
            return Err(MergeRefusal {
                error: no_commit_names_the_merges_parent(target, root, intent, record),
                began: true,
            });
        }
    }
    // What this worktree already holds of the merge is read once, before anything can refuse: every refusal after this point
    //  reports from this one reading, so no path can say nothing is installed while the index holds the merge.
    let the_evidence: TheEvidence =
        the_evidence_of_an_install_awaiting(place, root, intent, current.as_deref(), out).await?;
    let TheEvidence {
        changes,
        index_standing,
        judgements,
        any_in_index,
        any_in_worktree,
        began,
    } = the_evidence;
    // The checkout belongs to the branch the merge was computed on: installing it into another branch's worktree would put
    //  this merge's files and index into a branch it is not for, and move a branch from a checkout that is not its own.
    let checked_out: Option<String> = match symbolic_ref_awaiting(place, root).await {
        Ok(found) => found,
        Err(error) => {
            return Err(MergeRefusal {
                error,
                began,
            });
        }
    };
    if checked_out != intent.target_ref {
        return a_checkout_for_another_branch_awaiting(
            place,
            root,
            record,
            intent,
            checked_out.as_deref(),
            current.as_deref(),
            any_in_index,
            any_in_worktree,
            began,
            out,
        )
        .await;
    }
    match current.as_deref() {
        Some(head) if head == intent.old_oid => {}
        Some(head) => {
            return a_branch_that_moved_awaiting(place, record, intent, head, began, out).await;
        }
        None => {
            return Err(MergeRefusal {
                error: no_commit_names_the_merges_parent(target, root, intent, record),
                began,
            });
        }
    }
    // A merge that was saved on a side ref is finished even though nothing of it is in the checkout yet: the record with a side
    //  ref is what says the merge should land, not the checkout (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
    //  finding 3).
    if intent.side_ref.is_none() && !any_in_index && !any_in_worktree {
        if !restore_the_capture_awaiting(place, root, intent, out).await? {
            return Ok(MergeRecoveryAction::IndexHeld(IndexHeldAt {
                the_merges_entries_in_the_checkout: false,
                landed: false,
                branch_moved: false,
            }));
        }
        drop_the_records_refs_then_the_record_or_say(
            place,
            root,
            record,
            intent,
            SideRef::Drop,
            began,
            out,
        )
        .await?;
        return Ok(MergeRecoveryAction::Discarded);
    }
    let mut foreign: Vec<String> = Vec::new();
    for (change, judged) in changes.iter().zip(judgements.iter()) {
        if judged.standing == Standing::Foreign {
            foreign.push(String::from_utf8_lossy(change.path.as_slice()).into_owned());
        }
    }
    // A merge saved behind a live index lock never began, and the person's own edits may stand in the paths it changes, where
    //  nothing of the merge is in the index or the checkout. The capture the record keeps holds their bytes when they were saved
    //  and no more, so the merge is installed over them only where the capture holds their bytes and their mode as they are now;
    //  otherwise the record settles with its ref named rather than write over them (a cross-vendor audit of alpha.10, its fourth
    //  finding; the owner's "never discard user data").
    let a_saved_merge_over_the_persons_own_bytes: bool = intent.side_ref.is_some()
        && !any_in_index
        && !any_in_worktree
        && the_paths_hold_only_the_persons_own_bytes_awaiting(
            place,
            root,
            changes.as_slice(),
            judgements.as_slice(),
        )
        .await;
    if !foreign.is_empty()
        && !(a_saved_merge_over_the_persons_own_bytes
            && the_capture_holds_every_file_in_the_way(
                intent,
                changes.as_slice(),
                judgements.as_slice(),
            ))
    {
        if a_saved_merge_over_the_persons_own_bytes {
            return settle_a_saved_merge_over_newer_edits(
                place,
                root,
                record,
                intent,
                foreign.as_slice(),
                out,
            )
            .await;
        }
        return Err(MergeRefusal {
            error: edits_on_the_merges_own_paths(root, foreign.as_slice(), record),
            began,
        });
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
        return Err(MergeRefusal {
            error: staged_work_on_the_merges_paths(target, root, staged.as_slice(), record),
            began,
        });
    }
    // A lock that outlasts the wait leaves everything as it was; the record stays for the next command, which finishes the
    //  same merge (the recovery is idempotent). A merge with nothing of itself in the checkout did not wait at all, so the
    //  command has to say where that merge is.
    if complete_awaiting(
        place,
        root,
        record,
        intent,
        &changes,
        &judgements,
        began,
        out,
    )
    .await?
        == Installed::IndexHeld
    {
        if intent.side_ref.is_some() {
            let line: String = saved_merge_waits_for_the_index_lock(
                target,
                intent.side_ref.as_deref().unwrap_or("its side ref"),
                place.git_dir.join("index.lock").as_path(),
            );
            let _noted = out.write_all(line.as_bytes());
        }
        return Ok(MergeRecoveryAction::IndexHeld(IndexHeldAt {
            the_merges_entries_in_the_checkout: any_in_index || any_in_worktree,
            landed: false,
            branch_moved: false,
        }));
    }
    // The checkout holds the merge, so the person's own work in its paths is merged with it before the record goes. The
    //  checkout was written by this command, so no earlier reading describes it: the reading of what it left is taken now, and a
    //  file written between this look and the carry's own take is caught by that take's check of the entry it removes.
    let mut written_paths: Vec<Vec<u8>> = Vec::with_capacity(changes.len());
    for change in &changes {
        written_paths.push(change.path.clone());
    }
    let written: Vec<crate::merge_isolation_carry_landed::LandedFile> =
        crate::merge_isolation_carry_landed::landed_of(root, written_paths.as_slice());
    if !carry_the_capture_awaiting(place, root, intent, written.as_slice(), out).await? {
        return Ok(MergeRecoveryAction::IndexHeld(IndexHeldAt {
            the_merges_entries_in_the_checkout: true,
            landed: true,
            branch_moved: false,
        }));
    }
    drop_the_records_refs_then_the_record_or_say(
        place,
        root,
        record,
        intent,
        SideRef::Drop,
        true,
        out,
    )
    .await?;
    return Ok(MergeRecoveryAction::Completed);
}

/// Recovery of records whose owner is dead,
/// alive or unreadable,
/// with its controls in the sibling files
/// `merge_isolation_recovery_tests.rs` declares and lists.
#[cfg(test)]
#[path = "merge_isolation_recovery_tests.rs"]
mod tests;
