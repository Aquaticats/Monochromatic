//! What:
//!  The line that tells the person what recovery did to a merge whose owner is gone.
//! Why:
//!  A command a person ran to look at the worktree can move the branch onto a merge commit that a killed merge
//!  started,
//! and it can finish a merge that was saved on a side ref;
//! both are state changes the person did not ask for,
//! so they are
//!  said on standard error,
//! and the wording lives in a file of its own so the file that decides what recovery does stays
//!  readable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function recoveryNotice(outcome: MergeRecoveryOutcome): string | undefined;
//! ```

use crate::merge_isolation_recovery::MergeRecoveryOutcome;
use crate::merge_isolation_recovery_steps::MergeRecoveryAction;
/// A borrowed filesystem path.
use std::path::Path;

/// What:
///  The line for a merge that is saved on its side ref and waits for another program to let go of the index lock.
/// Why:
///  The command goes on at once instead of waiting out the ceiling,
/// so it has to say where the merge is and what
///  finishes it;
/// otherwise a person who ran `git rev-parse HEAD` learns that their branch has not moved and nothing
///  more.
///  The line names where the merge is saved and the lock,
/// never the wrapper's own record files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: the merge saved on ${sideRef} waits for ${lock}; ...`
/// ```
pub fn saved_merge_waits_for_the_index_lock(target: &str, side_ref: &str, lock: &Path) -> String {
    return format!(
        "monochromatic-git-policy-cli: the merge saved for {target} on {side_ref} waits for the index lock ({lock}), and nothing of it \
         is in the checkout yet, so this command goes on without waiting for it. A command run once that lock is free finishes the \
         merge; the edits you make meanwhile to files the merge does not change are kept.\n",
        lock = lock.display()
    );
}

/// What:
///  The line that tells the person what recovery did to a merge whose owner is gone,
/// for the outcomes that changed the checkout,
/// or nothing for the outcomes that changed nothing.
/// Why:
///   A command a person ran to look at the worktree can move the branch onto a merge commit that a killed merge
///  started;
/// that is a state change the person did not ask for,
/// and the only record of it was a debug line
///  (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
/// finding 1).
///  A discarded record,
/// a live owner's record and one whose branch already names the merge changed
///  nothing of the branch,
/// so they say nothing.
/// The line names the ref and the commit,
/// never the wrapper's own
///  record files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recoveryNotice(outcome: MergeRecoveryOutcome): string | undefined;
/// ```
pub fn recovery_notice(outcome: &MergeRecoveryOutcome) -> Option<String> {
    let reference: &str = outcome.target_ref.as_deref().unwrap_or("HEAD (detached)");
    return match outcome.action {
        MergeRecoveryAction::Completed => Some(format!(
            "monochromatic-git-policy-cli: finished an interrupted merge: {reference} now names {}, the merge commit. A command that \
             started this merge ended before it moved the branch, so this command moved it; the merge's files were already in the \
             checkout.\n",
            outcome.new_oid
        )),
        MergeRecoveryAction::BranchMoved => Some(format!(
            "monochromatic-git-policy-cli: {reference} moved while a merge was being prepared here, so that merge did not land; its \
             files were put back and the checkout matches the branch.\n"
        )),
        // The merge was reported saved, so it is not dropped: the record goes, and the side ref keeps its commit.
        MergeRecoveryAction::SavedBranchMoved => Some(format!(
            "monochromatic-git-policy-cli: the merge saved on {} does not apply: {reference} moved after it was saved, so it cannot \
             land as it was saved. Nothing was changed. The merge commit {} is kept on that ref, which nothing removes: land it with \
             `git merge {}`, or delete the ref once you have decided.\n",
            outcome.side_ref.as_deref().unwrap_or("its side ref"),
            outcome.new_oid,
            outcome.new_oid
        )),
        MergeRecoveryAction::OwnerActive
        | MergeRecoveryAction::Discarded
        | MergeRecoveryAction::AlreadyLanded
        | MergeRecoveryAction::IndexHeld
        // A merge that is partly installed is named by the caller that decides whether the command may go on
        //  (`startup_gate.rs`), because the words say what a command that writes must not do and what a reading one
        //  reads instead.
        | MergeRecoveryAction::PartlyInstalled
        | MergeRecoveryAction::DeferredRefused => None,
    };
}
