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
///  What the files the merge changes hold and where the person's own bytes are,
///  as the settle read it.
/// Why:
///   A lock can stop a merge before the clear wrote anything,
/// after the clear wrote the branch's content over the person's own bytes,
/// or after Git wrote the merge's own files.
///  The sentence that says where their bytes are is built from the reading,
/// so no ending promises a checkout the command did not leave.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BytesAfterAFailedMerge = { thePathsWereCleared: boolean; keptOn?: string };
/// ```
pub(crate) struct BytesAfterAFailedMerge<'a> {
    /// The clear put the paths the merge changes back to the branch's content.
    pub(crate) the_paths_were_cleared: bool,
    /// The ref the person's own bytes are kept on,
    /// or nothing when this merge captured none.
    pub(crate) kept_on: Option<&'a str>,
}

impl BytesAfterAFailedMerge<'_> {
    /// The sentence that says what the files hold and where the person's bytes are.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const where = (bytes) => bytes.thePathsWereCleared ? `the files hold the branch's content; your bytes are on ${kept}` : `every byte is on ${kept}`;
    /// ```
    fn where_your_bytes_are(&self) -> String {
        return match (self.the_paths_were_cleared, self.kept_on) {
            (true, Some(kept)) => {
                format!("The files this merge changes hold the branch's content where your own bytes were; those bytes are kept on {kept}.")
            }
            (true, None) => String::from("The files this merge changes hold the branch's content."),
            (false, Some(kept)) => format!("Every byte the files held before this command is kept on {kept}."),
            (false, None) => String::from("This merge wrote nothing to your files."),
        };
    }
}

/// What:
///  The sentence that says what a lock left in the way and how a command found it.
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
    /// Another program took the lock after this merge looked,
    /// and it stopped both Git's own update of the checkout and the putting back of the person's own bytes.
    TheLockStoppedTheSettle,
}

impl WhyTheMergeIsSaved {
    /// The clause that says what the lock did,
    /// for the line that names it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const blocked = (why) => ...;
    /// ```
    fn blocked(&self) -> String {
        return match self {
            WhyTheMergeIsSaved::TheWaitEnded(waited) => format!("did not let go within {} seconds", waited.as_secs()),
            WhyTheMergeIsSaved::NotTheCommandsToClear => String::from("is not one this command may clear"),
            // The merge is not made to wait for another program's command, which is the owner's rule, so the line must not read
            //  as one that waited.
            WhyTheMergeIsSaved::AnotherProgramHoldsIt(holder) => format!("is held by {holder} and this merge does not wait for it"),
            WhyTheMergeIsSaved::AnotherProgramTookIt => String::from("was taken by another program in the instant before this merge wrote"),
            WhyTheMergeIsSaved::TheLockStoppedTheSettle => {
                String::from("was taken by another program after this merge looked, and it stopped both Git's own update of the checkout and the putting back of your own bytes")
            }
        };
    }
}

/// What:
///  The line for a merge that is saved on a side ref because another program holds the index.
/// Why:
///   The command exits 0 and the branch has not moved,
/// so the person must be told where the merge is,
/// what finishes
///  it,
/// what the checkout holds meanwhile,
/// where their own bytes are when a lock stopped the putting back of them,
/// and how to clear a lock no program uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: git merge ${reference}: ...`
/// ```
pub(crate) fn a_deferred_save_finding(reference: &str, lock: &Path, side_ref: &str, why: &WhyTheMergeIsSaved, bytes: &BytesAfterAFailedMerge<'_>) -> String {
    return format!(
        "monochromatic-git-policy-cli: git merge {reference}: another program holds this repository's index ({lock}) and {blocked}, so the \
         merge is saved on {side_ref} and this command exits 0. The branch has not moved and nothing of the merge is in the checkout. \
         {where} The next git command in this worktree finishes the checkout once that lock is free. If no program is using this \
         repository, the lock was left by a program that was killed, and removing it lets a command run.\n",
        blocked = why.blocked(),
        where = bytes.where_your_bytes_are(),
        lock = lock.display()
    );
}

/// What:
///  The line for a merge that is not saved because its commit could not be kept on a side ref.
/// Why:
///   Without that name a collection can drop the commit,
/// so nothing is promised for a later finish;
/// the branch has not moved,
/// nothing of the merge is in the checkout,
/// the person's own bytes are kept on the capture's ref,
/// and the next command puts them back,
/// which is what the line says instead of claiming nothing changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: git merge ${reference}: ...`
/// ```
pub(crate) fn a_merge_that_could_not_be_kept(reference: &str, lock: &Path, bytes: &BytesAfterAFailedMerge<'_>) -> String {
    return format!(
        "monochromatic-git-policy-cli: git merge {reference}: another program holds this repository's index ({lock}), and this merge is \
         not saved for a later command because its commit could not be kept on a side reference. The branch has not moved and nothing \
         of the merge is in the checkout. {where} The next git command in this worktree puts your own bytes back once that lock is \
         free, and the merge is yours to run again then: `git merge {reference}`. If no program is using this repository, removing that \
         lock lets a command run.\n",
        where = bytes.where_your_bytes_are(),
        lock = lock.display()
    );
}

/// What:
///  The line for a merge whose save could not be recorded,
/// with its commit already kept on a side ref.
/// Why:
///   The ref names the commit and nothing removes it,
/// so the person is told how to land the merge with it or to drop it,
/// and the next command puts their own bytes back from the record this command kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: git merge ${reference}: ...`
/// ```
pub(crate) fn a_merge_whose_save_could_not_be_recorded(
    reference: &str,
    lock: &Path,
    side_ref: &str,
    merge_oid: &str,
    bytes: &BytesAfterAFailedMerge<'_>,
) -> String {
    return format!(
        "monochromatic-git-policy-cli: git merge {reference}: another program holds this repository's index ({lock}), and the record that \
         would finish this merge later could not be written. The branch has not moved and nothing of the merge is in the checkout. \
         {where} Its commit is kept on {side_ref}, which nothing removes: `git merge {merge_oid}` lands it once that lock is free, or \
         `git update-ref -d {side_ref}` drops it once you have decided. The next git command in this worktree puts your own bytes back. \
         If no program is using this repository, removing that lock lets a command run.\n",
        where = bytes.where_your_bytes_are(),
        lock = lock.display()
    );
}

/// What:
///  The line for a merge whose branch moved while it was prepared and whose own bytes the lock kept in the capture.
/// Why:
///   That merge cannot be applied as it was saved,
/// so nothing is kept on a side ref;
/// the branch moved,
/// nothing of the merge is in the checkout,
/// and the next command puts the person's bytes back and leaves the merge to them,
/// which the line says instead of claiming the branch has not moved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: git merge ${reference}: ...`
/// ```
pub(crate) fn a_merge_the_branch_moved_under(reference: &str, lock: &Path, bytes: &BytesAfterAFailedMerge<'_>) -> String {
    return format!(
        "monochromatic-git-policy-cli: git merge {reference}: another program holds this repository's index ({lock}), and the branch moved \
         while this merge was prepared, so this merge does not land. Nothing of the merge is in the checkout. {where} The next git \
         command in this worktree puts your own bytes back once that lock is free, and the merge is yours to run again on the branch as \
         it is now: `git merge {reference}`. If no program is using this repository, removing that lock lets a command run.\n",
        where = bytes.where_your_bytes_are(),
        lock = lock.display()
    );
}

/// What:
///  The line for a merge that is in the branch while a lock keeps the person's own bytes out of the files.
/// Why:
///   The merge landed,
/// so nothing is left to apply;
/// only the carry waits,
/// and the line says where their bytes are and that the next command carries them in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: git merge ${reference}: the merge is in the branch ...`
/// ```
pub(crate) fn a_merge_in_the_branch_whose_carry_waits(reference: &str, lock: &Path, bytes: &BytesAfterAFailedMerge<'_>) -> String {
    return format!(
        "monochromatic-git-policy-cli: git merge {reference}: the merge is in the branch, and another program holds this repository's \
         index ({lock}), so your own changes could not be carried into the files yet. {where} The next git command in this worktree \
         carries them into the files once that lock is free. Nothing of yours was removed.\n",
        where = bytes.where_your_bytes_are(),
        lock = lock.display()
    );
}

/// What:
///  The line for a merge whose entries are partly in the checkout while a lock keeps the rest out.
/// Why:
///   Part of the merge is installed and only this record can finish it,
/// so the command exits 0 and says what stands,
/// rather than refusing a checkout it changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: git merge ${reference}: part of the merge is in the checkout ...`
/// ```
pub(crate) fn a_merge_partly_installed_behind_the_lock(reference: &str, lock: &Path, bytes: &BytesAfterAFailedMerge<'_>) -> String {
    return format!(
        "monochromatic-git-policy-cli: git merge {reference}: part of the merge is in the index and the worktree, and another program \
         holds this repository's index ({lock}) so the rest waits. {where} The next git command in this worktree finishes it once that \
         lock is free. Nothing of yours was lost.\n",
        where = bytes.where_your_bytes_are(),
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
