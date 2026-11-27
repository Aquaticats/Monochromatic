//! What:
//!  What a recovery does when this worktree has a branch checked out that the merge was not
//!  computed on:
//!  a merge nothing installed is discarded and its person's bytes are put back,
//!  and a merge with something in this checkout is refused with every path named.
//! Why:
//!   Installing a merge into another branch's worktree would put this merge's files and index
//! into a branch it is not for,
//!  and move a branch from a checkout that is not its own.
//!  A record that names no side ref and has nothing of the merge here has nothing to finish
//! and nothing to stop a command for,
//!  so it goes and the person runs the merge again on the branch it is for;
//! leaving it kept every command on another branch refused for a merge nothing installed.
//!  A merge kept on a side ref says where its commit is;
//! one that was only recorded has nothing to promise about it.
//!  This is a file of its own so the recovery that decides whether a merge began
//! stays inside the code-line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const action = await aCheckoutForAnotherBranch({ place, record, intent, checkedOut, current, anyInIndex, anyInWorktree, began, out });
//! ```

use crate::merge_isolation_journal::MergeIntent;
/// The person's own work a merge carries,
/// and the dropping of the refs that kept it.
use crate::merge_isolation_recovery_carry::{
    drop_the_records_refs_then_the_record_or_say,
    restore_the_capture_awaiting,
};
/// Which of a record's refs go when it is dropped.
use crate::merge_isolation_recovery_cleanup::SideRef;
/// Where recovery runs Git.
use crate::merge_isolation_recovery_place::RecoveryPlace;
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
///  Settle a merge whose branch this worktree does not have checked out.
/// Why:
///   Nothing of the merge may be finished into a branch it was not computed on,
/// and a merge nothing installed must not keep refusing every command here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function aCheckoutForAnotherBranch({ place, record, intent, checkedOut, current, anyInIndex, anyInWorktree, began, out }): Promise<MergeRecoveryAction>;
/// ```
pub(crate) async fn a_checkout_for_another_branch_awaiting<W: Write + ?Sized>(
    place: &RecoveryPlace,
    root: &Path,
    record: &Path,
    intent: &MergeIntent,
    checked_out: Option<&str>,
    current: Option<&str>,
    any_in_index: bool,
    any_in_worktree: bool,
    began: bool,
    out: &mut W,
) -> Result<MergeRecoveryAction, MergeRefusal> {
    let target: &str = intent.target_ref.as_deref().unwrap_or("HEAD");
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
            return Ok(MergeRecoveryAction::IndexHeld(IndexHeldAt {
                the_merges_entries_in_the_checkout: false,
                landed: false,
                branch_moved: current.as_deref() != Some(intent.old_oid.as_str()),
            }));
        }
        // The person's bytes are back, and the ref that kept them goes before the record that names it, so a deletion another
        //  command's lock blocked is retried by the next command instead of leaving a name nothing removes.
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
