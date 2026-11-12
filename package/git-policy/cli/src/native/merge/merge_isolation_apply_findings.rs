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
pub(crate) fn a_deferred_save_finding(reference: &str, lock: &Path, side_ref: &str, held_for: Option<Duration>) -> String {
    let blocked: String = match held_for {
        Some(waited) => format!("did not let go within {} seconds", waited.as_secs()),
        None => String::from("is not one this command may clear"),
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
///  The line for a merge that is saved on a side ref because another merge is installing in the same worktree.
/// Why:
///   The command exits 0 and the branch has not moved,
/// so the person must be told where the merge is,
/// what finishes it,
///  and that nothing of theirs was touched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: git merge ${reference}: another merge is installing ...`
/// ```
pub(crate) fn a_deferred_save_behind_a_merge(reference: &str, holder_pid: i64, side_ref: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: git merge {reference}: another merge is installing its result in this worktree (process {holder_pid}), and \
         one merge installs in a worktree at a time, so this one is saved on {side_ref} and this command exits 0. Nothing of it is in the \
         checkout and the branch has not moved: the next git command in this worktree finishes it once that merge is done, and the edits you \
         make meanwhile to files it does not change are kept.\n"
    );
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
