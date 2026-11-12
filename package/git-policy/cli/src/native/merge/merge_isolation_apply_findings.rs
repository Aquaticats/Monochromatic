//! What:
//!  The words an isolated merge prints when it could not be finished now:
//!  a merge saved on a side ref because another program holds the index,
//!  one that could not be saved at all,
//!  and the carry back that stopped.
//! Why:
//!   Each ending keeps the merge made and the branch where it is,
//!  so the person reads where their work is and what finishes it;
//!  the merge that prints them is `merge_isolation_apply.rs`,
//!  which is at its code-line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! stderr.write(aDeferredSaveFinding(reference, lock, sideRef, heldFor));
//! ```

use crate::merge_isolation_carry_apply::CarryFailure;
use crate::merge_isolation_carry_parts::Capture;
use std::path::Path;
use std::time::Duration;

/// What:
///  The line for a merge that is saved on a side ref because another program holds the index.
/// Why:
///   The command exits 0 and the branch has not moved,
/// so the person must be told where the merge is,
/// what finishes
///  it,
/// what happens to their own edits meanwhile,
/// and how to clear a lock no program uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: git merge ${reference}: ...`
/// ```
pub(crate) fn a_deferred_save_finding(reference: &str, lock: &Path, side_ref: &str, why: WhyTheMergeIsSaved) -> String {
    let blocked: String = match why {
        WhyTheMergeIsSaved::TheWaitEnded(waited) => format!("did not let go within {} seconds", waited.as_secs()),
        WhyTheMergeIsSaved::NotTheCommandsToClear => String::from("is not one this command may clear"),
        // The merge is not made to wait for another program's command, which is the owner's rule, so the line must not read as
        //  one that waited.
        WhyTheMergeIsSaved::AnotherProgramHoldsIt(holder) => format!("is held by {holder} and this merge does not wait for it"),
        WhyTheMergeIsSaved::AnotherProgramTookIt => String::from("was taken by another program in the instant before this merge wrote"),
    };
    return format!(
        "monochromatic-git-policy-cli: git merge {reference}: another program holds this repository's index ({lock}) and {blocked}, so the \
         merge is saved on {side_ref} and this command exits 0. Nothing of the merge is in the checkout yet and the branch has not moved: \
         the next git command in this worktree finishes the checkout once that lock is free, and the edits you make meanwhile to files the \
         merge does not change are kept. If no program is using this repository, the lock was left by a program that was killed, and \
         removing it lets a command run.\n",
        lock = lock.display()
    );
}

/// What:
///  Why a merge saved itself instead of applying,
/// as the line that says so words it.
/// Why:
///   A merge that gave up waiting and one that never waited read differently to a person,
/// and only the first may say it
///  waited.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WhyTheMergeIsSaved = { kind: 'the-wait-ended'; waited: Duration } | { kind: 'not-the-commands-to-clear' } | { kind: 'another-program-holds-it'; holder: string } | { kind: 'another-program-took-it' };
/// ```
#[derive(Clone, Debug)]
pub(crate) enum WhyTheMergeIsSaved {
    /// The wait reached its ceiling with the lock still held.
    TheWaitEnded(Duration),
    /// The lock stayed where it was through this command's removal,
    /// which only a lock this command may not clear does.
    NotTheCommandsToClear,
    /// Another program's command holds the lock,
    /// and the merge did not wait for it.
    AnotherProgramHoldsIt(String),
    /// Another program took the lock after this merge looked,
    /// so nothing of the clear was written.
    AnotherProgramTookIt,
}

/// What:
///  The line for a merge whose commit could not be kept on a side ref.
/// Why:
///   Without that name the commit can be collected,
/// so the merge is refused with nothing changed rather than
///  promised for a later finish it may not reach.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: git merge ${reference}: ...`
/// ```
pub(crate) fn a_merge_that_could_not_be_saved(reference: &str, lock: &Path) -> String {
    return format!(
        "monochromatic-git-policy-cli: git merge {reference}: another program holds this repository's index ({lock}), and the merge could \
         not be kept on a side ref for a later finish. Nothing was changed. Run the merge again once that program is done, and if no \
         program is using this repository, removing that lock lets a command run.\n",
        lock = lock.display()
    );
}

/// What:
///  The line a merge prints when the carry back could not be written now.
/// Why:
///   The merge is made and the branch moved,
/// so the command exits 0;
/// the person's bytes are in the objects the capture
///  names,
/// and the record is kept,
/// so the next command in the worktree carries them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: git merge ${reference}: the merge is made, and your changes could not be carried back (${detail}) ...`
/// ```
pub(crate) fn a_carry_that_stopped(reference: &str, failure: &CarryFailure, capture: &Capture) -> String {
    let detail: String = match failure {
        CarryFailure::IndexHeld(lock) => format!("another program holds this repository's index ({lock})"),
        CarryFailure::Failed(text) => String::from(text.as_str()),
        CarryFailure::Foreign(path) | CarryFailure::Changed(path) => {
            format!("{} holds something else now", String::from_utf8_lossy(path.as_slice()))
        }
    };
    let kept: &str = capture.reference.as_deref().unwrap_or("the record this merge wrote");
    return format!(
        "monochromatic-git-policy-cli: git merge {reference}: the merge is made, and your own changes could not be carried back yet because \
         {detail}. They are kept in {kept}, and the next git command in this worktree carries them into the files. Nothing of yours was \
         removed.\n"
    );
}

/// What:
///  The text of a failure a carry or a clear reports.
/// Why:
///   Recovery's refusal takes text,
/// and a failure a person reads names what the step was doing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const failureText = (failure: CarryFailure) => failure.detail;
/// ```
pub(crate) fn failure_text(failure: &CarryFailure) -> String {
    return match failure {
        CarryFailure::IndexHeld(lock) => format!("another program held {lock} until this merge had waited its longest"),
        CarryFailure::Failed(detail) => String::from(detail.as_str()),
        CarryFailure::Foreign(path) | CarryFailure::Changed(path) => format!(
            "{} was written after this merge captured it, so the merge was not written over it",
            String::from_utf8_lossy(path.as_slice())
        ),
    };
}

/// What:
///  The line a merge prints when it is finished and a ref its record keeps could not be removed.
/// Why:
///   The merge is made and the person's work is in the checkout,
/// so the command exits `0`;
/// the record stays because it is the only
///  thing that still names that ref,
/// and the next command's recovery removes both,
/// which is what the line promises
/// (`a cross-vendor audit of alpha.12`,
/// "Capture-ref cleanup is never retried after a transient failure").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = (record) => `... the record ${record} was kept and the next git command here tries again ...`;
/// ```
pub(crate) fn a_refused_cleanup(record: &Path) -> String {
    return format!(
        "monochromatic-git-policy-cli: the merge is made, and a ref its record keeps could not be removed yet, so the record {} was \
         kept and the next git command in this worktree tries again. Nothing of yours was removed or left in an object nothing names.\n",
        record.display()
    );
}
