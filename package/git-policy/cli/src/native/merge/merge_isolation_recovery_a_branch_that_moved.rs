//! What:
//!  What a recovery does when the branch is not at the commit the merge was computed on:
//!  the person's work is carried onto the head the branch moved to,
//!  a saved merge is finished or kept with its commit named,
//!  and a partly installed merge is refused.
//! Why:
//!   A merge that landed and was then built on by another session is still the person's merge:
//! the checkout is put back to the moved head and their work is merged onto it,
//!  rather than written back as it was before the merge,
//! which would show the merge's lines undone.
//!  A merge kept on a side ref must not vanish,
//! so its record goes and the side ref that keeps its commit stays and is named;
//! one that was only recorded has nothing to promise about its commit.
//!  The record stays until every byte of the person's is in the files,
//!  so a deletion another command's lock blocked is retried by the next command.
//!  This is a file of its own so the recovery that decides whether a merge began
//! stays inside the code-line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const action = await aBranchThatMoved({ place, record, intent, head, began, out });
//! ```

use crate::merge_isolation_journal::MergeIntent;
/// The settling of a merge whose branch moved while it was prepared or applied.
use crate::merge_isolation_moved::settle_moved_branch_awaiting;
/// The person's own work a merge carries,
/// the keeping of the ref that holds it,
/// and the dropping of both once it is back.
use crate::merge_isolation_recovery_carry::{
    carry_onto_awaiting,
    drop_the_records_refs_then_the_record_or_say,
    restore_the_capture_awaiting,
};
/// Which of a record's refs go when it is dropped.
use crate::merge_isolation_recovery_cleanup::SideRef;
/// Where recovery runs Git.
use crate::merge_isolation_recovery_place::RecoveryPlace;
/// Whether the merge's commit is behind the head the branch moved to.
use crate::merge_isolation_recovery_reads::is_ancestor_awaiting;
/// What one recovery came to,
/// and the refusal that carries whether the merge began.
use crate::merge_isolation_recovery_record::{
    IndexHeldAt,
    MergeRecoveryAction,
    MergeRefusal,
};
/// The fail-closed recovery failure.
use crate::recovery_error::RecoveryError;
/// Writing the notices of what recovery did.
use std::io::Write;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  Settle a merge whose branch is not at the commit the merge was computed on.
/// Why:
///   What a moved branch leaves behind is one of three things,
/// and each is settled from the same evidence:
///  a merge that landed and was built on,
/// a merge saved on a side ref,
///  or a merge that was only recorded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function aBranchThatMoved({ place, record, intent, head, began, out }): Promise<MergeRecoveryAction>;
/// ```
pub(crate) async fn a_branch_that_moved_awaiting<W: Write + ?Sized>(
    place: &RecoveryPlace,
    record: &Path,
    intent: &MergeIntent,
    head: &str,
    began: bool,
    out: &mut W,
) -> Result<MergeRecoveryAction, MergeRefusal> {
    let root: &Path = intent.repository_root.as_path();
    let target: &str = intent.target_ref.as_deref().unwrap_or("HEAD");
    // A merge that landed and was then built on by another session is still the person's merge: the checkout is put back
    //  to the moved head, and their own work is merged onto it rather than written back as it was before the merge, which
    //  would show the merge's lines undone. The record stays until their bytes are in the files.
    if intent.carried.is_some()
        && is_ancestor_awaiting(place, root, intent.new_oid.as_str(), head).await?
    {
        let action: MergeRecoveryAction =
            settle_moving_the_record_aside_awaiting(place, record, intent, head, out).await?;
        if action != MergeRecoveryAction::BranchMoved {
            return Ok(action);
        }
        if !carry_onto_awaiting(
            place,
            root,
            intent,
            head,
            intent.target_ref.as_deref().unwrap_or("HEAD"),
            out,
        )
        .await?
        {
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
            began,
            out,
        )
        .await?;
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
            return Ok(MergeRecoveryAction::IndexHeld(IndexHeldAt {
                the_merges_entries_in_the_checkout: false,
                landed: false,
                branch_moved: true,
            }));
        }
        // The side ref stays: it is the only name that still keeps the merge's commit reachable, and the line says so.
        drop_the_records_refs_then_the_record_or_say(
            place,
            root,
            record,
            intent,
            SideRef::Keep,
            began,
            out,
        )
        .await?;
        return Ok(MergeRecoveryAction::SavedBranchMoved);
    }
    let action: MergeRecoveryAction =
        settle_moved_branch_awaiting(place, record, intent, head, true, out).await?;
    if action == MergeRecoveryAction::BranchMoved {
        // The person's bytes come back before the ref that keeps them goes, and the record that names both goes last, so a
        //  deletion another command's lock blocked is retried by the next command.
        if !restore_the_capture_awaiting(place, root, intent, out).await? {
            return Ok(MergeRecoveryAction::IndexHeld(IndexHeldAt {
                the_merges_entries_in_the_checkout: false,
                landed: false,
                branch_moved: true,
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
    }
    return Ok(action);
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
