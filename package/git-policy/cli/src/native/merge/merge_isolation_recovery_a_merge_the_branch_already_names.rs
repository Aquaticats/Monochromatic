//! What:
//!  The settle of a merge the branch already names:
//!  the person's own work in its paths is merged with the merge,
//!  and the record that names the merge goes only once every byte of theirs is back.
//! Why:
//!   The record is what still names the worktree the merge's install went into,
//! and a checkout another branch owns may hold its index entries and files,
//!  so the branch is read before the record goes
//! (the owner's "never discard user data").
//!  A record whose carry already finished needs nothing from the capture's objects,
//! which may be collected by then:
//! a kill between the drop of a ref and the removal of the record leaves exactly that,
//!  and the read that failed there refused every command.
//!  This is a file of its own so the recovery that decides whether a merge began
//! stays inside the code-line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const action = await aMergeTheBranchAlreadyNames({ place, record, intent, out }); // AlreadyLanded, or IndexHeld
//! ```

/// The raw differences of a merge,
/// and the worktree path one of its changes writes.
use crate::merge_isolation_changes::Change;
use crate::merge_isolation_journal::MergeIntent;
/// The person's own work a merge carries,
/// the dropping of the refs that kept it,
/// and the removal of the record.
use crate::merge_isolation_recovery_carry::{
    carry_the_capture_awaiting,
    drop_the_records_refs_then_the_record_or_say,
};
/// Which of a record's refs go when it is dropped.
use crate::merge_isolation_recovery_cleanup::SideRef;
/// A landed merge found in another branch's worktree.
use crate::merge_isolation_recovery_cleanup::a_landed_merge_in_another_branchs_worktree;
/// Where recovery runs Git.
use crate::merge_isolation_recovery_place::RecoveryPlace;
/// Whether a ref is checked out here.
use crate::merge_isolation_recovery_reads::symbolic_ref_awaiting;
/// What one recovery came to,
/// and the refusal that carries whether the merge began.
use crate::merge_isolation_recovery_record::{
    IndexHeldAt,
    MergeRecoveryAction,
    MergeRefusal,
    partly_installed,
};
/// The paths a merge changes,
/// and the putting back of what a killed take left beside them.
use crate::merge_isolation_recovery_steps::{
    merge_changes_awaiting,
    put_back_stranded_takes_awaiting,
};
/// Writing the notices of what recovery did.
use std::io::Write;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Settle a merge whose commit the branch already names,
///  so what is left is the person's own work in the merge's paths and the record.
/// Why:
///   A merge that already landed is settled from the branch alone,
/// without reading a worktree that may have become unreadable since,
///  and the record goes only once the carry has written every byte of theirs back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function aMergeTheBranchAlreadyNames({ place, record, intent, out }): Promise<MergeRecoveryAction>;
/// ```
pub(crate) async fn a_merge_the_branch_already_names_awaiting<W: Write + ?Sized>(
    place: &RecoveryPlace,
    record: &Path,
    intent: &MergeIntent,
    out: &mut W,
) -> Result<MergeRecoveryAction, MergeRefusal> {
    let root: &Path = intent.repository_root.as_path();
    let target: &str = intent.target_ref.as_deref().unwrap_or("HEAD");
    // The branch already names the merge, and the record is what still names the worktree its install went into: a
    //  checkout another branch owns may hold the merge's index entries and files, so the branch is read before the
    //  record goes (the owner's "never discard user data"; a trial reviewer's finding).
    let checked_out: Option<String> = symbolic_ref_awaiting(place, root)
        .await
        .map_err(partly_installed)?;
    if checked_out != intent.target_ref {
        return Err(partly_installed(
            a_landed_merge_in_another_branchs_worktree(
                target,
                root,
                checked_out.as_deref(),
                record,
            ),
        ));
    }
    // The merge is in the branch, so the person's own work in its paths is merged with it now, and the record goes only
    //  once every byte of theirs is back in the checkout.
    // What the command that moved the checkout read is what this carry judges the paths against, so a save that
    //  arrived after a kill is left alone instead of being replaced by the person's older layer.
    let mut landed: Vec<crate::merge_isolation_carry_landed::LandedFile> =
        crate::merge_isolation_carry_landed::read_landed_record(
            place.git_dir.as_path(),
            intent.owner.transaction_id.as_str(),
        );
    // A file a command that stopped left under its private name is put back before the carry judges the path: the take
    //  that moved it aside registers nothing until its placement returns, so a kill in that window leaves a copy no
    //  record names, and the path is empty while it waits beside it. The carry would read the empty path as somebody
    //  else's write, leave it, and the record would then go with the person's file still absent.
    let changes: Vec<Change> = match merge_changes_awaiting(place, root, intent).await {
        Ok(found) => found,
        // The branch already names this merge, so a read that fails cannot prove the checkout untouched.
        Err(error) => {
            return Err(MergeRefusal {
                error,
                began: true,
            });
        }
    };
    let put_back: Vec<PathBuf> = put_back_stranded_takes_awaiting(
        &place.git,
        root,
        place.git_dir.as_path(),
        changes.as_slice(),
        intent.carried.as_ref(),
        intent.owner.transaction_id.as_str(),
        out,
    )
    .await;
    // A file this command put back is the entry the merge judged at that path: the rename left its identity as it was and
    //  changed only its change time, so the reading is taken again there and the carry writes over the file it put back.
    for location in &put_back {
        for entry in landed.iter_mut() {
            if crate::merge_isolation_changes::worktree_path(root, entry.path.as_slice())
                == *location
            {
                let path: Vec<u8> = entry.path.clone();
                *entry = crate::merge_isolation_carry_landed::landed_file_at(
                    location.as_path(),
                    path.as_slice(),
                );
            }
        }
    }
    // A record whose carry already finished needs nothing from the capture's objects, which may be collected by then: a
    //  kill between the drop of a ref and the removal of the record leaves exactly that, and the read that failed there
    //  refused every command (`merge_isolation_journal.rs`, `carry_finished`).
    let already_carried: bool = crate::merge_isolation_journal::carry_finished(record);
    if !already_carried
        && !carry_the_capture_awaiting(place, root, intent, landed.as_slice(), out).await?
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
        true,
        out,
    )
    .await?;
    return Ok(MergeRecoveryAction::AlreadyLanded);
}
