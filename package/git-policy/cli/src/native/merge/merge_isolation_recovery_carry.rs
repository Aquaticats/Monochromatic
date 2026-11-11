//! What:
//!  The person's own work a merge carries,
//! across the two moves the recovery of a merge can make:
//!  the carry into a merge that landed,
//!  the carry onto a moved head,
//!  and the putting back for a merge that did not land.
//! Why:
//!  A merge replaces exactly the paths this work holds,
//! so every byte of it is in the record's capture before a path changes,
//!  and putting it back is what makes the record removable;
//! the carries have a file of their own because each is one awaited
//!  sequence of reads and writes over the same capture
//! (`merge_isolation_recovery.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const carried = await carryTheCapture({ place, root, intent, stderr });
//! ```

use crate::diagnostic_log::debug;
use crate::merge_isolation_carry::drop_capture_ref_awaiting;
use crate::merge_isolation_carry_apply::{CarryFailure, LockWait, carry_awaiting, changes_between_awaiting, restore_awaiting};
use crate::merge_isolation_carry_notice::report_lines;
use crate::merge_isolation_changes::Change;
use crate::merge_isolation_journal::MergeIntent;
use crate::merge_isolation_recovery_cleanup::partly_installed;
use crate::merge_isolation_recovery_steps::{MergeRefusal, RecoveryPlace, merge_changes_awaiting};
use crate::recovery_error::RecoveryError;
use std::path::Path;

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
pub(crate) async fn carry_the_capture_awaiting<W: std::io::Write + ?Sized>(
    place: &RecoveryPlace,
    root: &Path,
    intent: &MergeIntent,
    out: &mut W,
) -> Result<bool, MergeRefusal> {
    let Some(capture) = intent.carried.as_ref() else { return Ok(true) };
    let changes: Vec<Change> = merge_changes_awaiting(place, root, intent).await.map_err(partly_installed)?;
    // The reading the apply made before it died, which is the one thing that tells the file Git wrote from a save whose bytes
    //  accept the whole merge: a record written before that reading names no entry, and then the bytes are all the evidence
    //  there is, as before (`merge_isolation_carry_landed.rs`).
    match carry_awaiting(&place.git, root, place.git_dir.as_path(), changes.as_slice(), capture, intent.landed.as_slice(), intent.owner.transaction_id.as_str(), LockWait::WhileFinishing, out).await {
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
///  Merge the person's own work with the tree of one commit,
/// which is the moved head when the branch was built on the merge.
/// Why:
///   A carry onto the old merge's own commit would leave the moved head's later changes out of the files,
/// and a
///  carry onto the moved head keeps them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function carryOnto({ place, root, intent, end, out }): Promise<boolean>;
/// ```
pub(crate) async fn carry_onto_awaiting<W: std::io::Write + ?Sized>(place: &RecoveryPlace, root: &Path, intent: &MergeIntent, end: &str, out: &mut W) -> Result<bool, MergeRefusal> {
    let Some(capture) = intent.carried.as_ref() else { return Ok(true) };
    let mut changes: Vec<Change> =
        changes_between_awaiting(&place.git, root, intent.old_oid.as_str(), end).await.map_err(the_changes_could_not_be_read)?;
    // A captured path the moved head left alone is not in that difference, so the merge's own side of it is what the person's
    //  work merges with.
    let merge: Vec<Change> = merge_changes_awaiting(place, root, intent).await.map_err(partly_installed)?;
    for entry in &capture.paths {
        if the_change_of(changes.as_slice(), entry.path.as_slice()).is_some() {
            continue;
        }
        if let Some(change) = the_change_of(merge.as_slice(), entry.path.as_slice()) {
            changes.push(change.clone());
        }
    }
    match carry_awaiting(&place.git, root, place.git_dir.as_path(), changes.as_slice(), capture, intent.landed.as_slice(), intent.owner.transaction_id.as_str(), LockWait::WhileFinishing, out).await {
        Ok(report) => {
            for line in report_lines(intent.target_ref.as_deref().unwrap_or("HEAD"), &report) {
                let _noted = out.write_all(format!("{line}\n").as_bytes());
            }
            return Ok(true);
        }
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
pub(crate) async fn restore_the_capture_awaiting<W: std::io::Write + ?Sized>(
    place: &RecoveryPlace,
    root: &Path,
    intent: &MergeIntent,
    out: &mut W,
) -> Result<bool, MergeRefusal> {
    let Some(capture) = intent.carried.as_ref() else { return Ok(true) };
    let changes: Vec<Change> = merge_changes_awaiting(place, root, intent).await.map_err(partly_installed)?;
    match restore_awaiting(&place.git, root, place.git_dir.as_path(), changes.as_slice(), capture, intent.owner.transaction_id.as_str(), LockWait::WhileFinishing, out).await {
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
pub(crate) fn the_work_could_not_be_carried(failure: &CarryFailure) -> String {
    let detail: String = match failure {
        CarryFailure::IndexHeld(text) | CarryFailure::Failed(text) => String::from(text.as_str()),
        CarryFailure::Foreign(path) | CarryFailure::Changed(path) => {
            format!("{} changed after the merge captured it", crate::path_bytes::path_message_text(path.as_slice()))
        }
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
pub(crate) async fn drop_capture_awaiting(place: &RecoveryPlace, root: &Path, intent: &MergeIntent) {
    if let Some(capture) = intent.carried.as_ref() {
        drop_capture_ref_awaiting(&place.git, root, capture).await;
    }
}

/// The one change of a list that names this path,
/// or nothing when the list has none.
/// Why:
///   A captured path asks both the moved head's difference and the merge's own for its entry,
/// and a named function
///  answers it without a closure that a reader has to follow across the loops.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theChangeOf = (changes: Change[], path: Uint8Array): Change | undefined => changes.find((change) => sameBytes(change.path, path));
/// ```
fn the_change_of<'a>(changes: &'a [Change], path: &[u8]) -> Option<&'a Change> {
    for change in changes {
        crate::collected_elements_looked_at::note_looked_at(1);
        if change.path == path {
            return Some(change);
        }
    }
    return None;
}

/// The refusal of a difference a settling could not read,
/// which is reported as a merge that may be half installed.
/// Why:
///   The bytes of the moved head decide which paths a settling writes,
/// so a read that failed cannot be told from an install that began.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theChangesCouldNotBeRead = (error: string): MergeRefusal => partlyInstalled(new RecoveryError(error));
/// ```
fn the_changes_could_not_be_read(error: String) -> MergeRefusal {
    return partly_installed(RecoveryError(error));
}
