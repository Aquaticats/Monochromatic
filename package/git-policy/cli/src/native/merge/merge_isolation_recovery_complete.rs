//! What:
//!  Finishing a merge from its record:
//! the install of the index entries and the files,
//! then the branch move,
//! with the checks the branch
//!  move needs.
//! Why:
//!  The decision that a merge must be finished is made in `merge_isolation_recovery.rs`,
//! and the writes it ends in are a
//!  file of their own,
//! which is also what keeps that decision's file under the package's line limit.
//!  The last look at `HEAD` sits beside the write because the two are one step's neighbours:
//! `git update-ref` verifies the branch
//!  and not the worktree,
//! so a worktree that switched branch is met by one read after the write and put
//!  back.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await complete({ place, root, record, intent, changes, judgements, installedBefore, stderr });
//! ```

use crate::diagnostic_log::debug;
use crate::merge_isolation_changes::Change;
use crate::merge_isolation_journal::MergeIntent;
use crate::merge_isolation_recovery_cleanup::{a_worktree_that_switched, an_installed_merge_behind_a_held_index, partly_installed};
use crate::merge_isolation_recovery_reads::symbolic_ref_awaiting;
use crate::merge_isolation_recovery_steps::{
    Installed, Judged, MergeRefusal, RecoveryPlace, install_entries_awaiting,
};
use crate::recovery_error::RecoveryError;
use crate::transaction_git::GitRequest;
use crate::transaction_git_awaiting::run_git_checked_awaiting;
use std::io::Write;
use std::path::Path;

/// What:
///  Bring the index entries and the worktree files of the changed paths to the merged state,
/// then move the branch.
/// Why:
///   The index entries are set first so a file written by `checkout-index` is recorded with the object it holds;
///  deletions go before writes so a file replaced by a directory of the same name can be created;
///  the branch moves last,
/// by compare-and-swap against the old commit,
/// so a branch that moved meanwhile is left alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function complete({ place, root, intent, changes, standings }): Promise<void>;
/// ```
pub async fn complete_awaiting(
    place: &RecoveryPlace,
    root: &Path,
    record: &Path,
    intent: &MergeIntent,
    changes: &[Change],
    judgements: &[Judged],
    installed_before: bool,
    out: &mut (dyn Write + Send),
) -> Result<Installed, MergeRefusal> {
    if install_entries_awaiting(place, root, changes, judgements, installed_before, out).await? == Installed::IndexHeld {
        // A merge that was already installed has nothing to defer: a command that went on would read it as another session's
        //  staged work, so it is stopped and told what is there.
        if installed_before {
            return Err(partly_installed(an_installed_merge_behind_a_held_index(
                intent.target_ref.as_deref().unwrap_or("HEAD"),
                root,
                place.git_dir.join("index.lock").as_path(),
                record,
            )));
        }
        return Ok(Installed::IndexHeld);
    }
    // The checkout was checked for its branch before anything was installed; this is the one more look before the branch moves,
    //  because another command can switch the worktree between the two while both branches name the same commit.
    let checked_out: Option<String> = symbolic_ref_awaiting(place, root).await.map_err(partly_installed)?;
    if checked_out != intent.target_ref {
        return Err(partly_installed(RecoveryError(format!(
            "A merge of {} in {} was installed here, and this worktree now has {} checked out: the branch was not moved and the \
             record was kept. Check out {} in this worktree and run any git command again.",
            intent.target_ref.as_deref().unwrap_or("a detached HEAD"),
            root.display(),
            checked_out.as_deref().unwrap_or("a detached HEAD"),
            intent.target_ref.as_deref().unwrap_or("a detached HEAD")
        ))));
    }
    // A control reaches this instant to switch the worktree as another command would, between the last look and the write.
    crate::service_fault::reach("before-the-branch-write");
    let target: &str = intent.target_ref.as_deref().unwrap_or("HEAD");
    let message: String = String::from("merge: Merge made by the 'ort' strategy. (completed by monochromatic-git-policy-cli recovery)");
    let mut arguments: Vec<&str> = vec!["update-ref", "-m", message.as_str()];
    if intent.target_ref.is_none() {
        arguments.push("--no-deref");
    }
    arguments.push(target);
    arguments.push(intent.new_oid.as_str());
    arguments.push(intent.old_oid.as_str());
    if let Err(error) = run_git_checked_awaiting(&place.git, &GitRequest::new(root, &arguments)).await {
        // The index and every file of the merge are in place and the branch did not move.
        return Err(MergeRefusal { error: RecoveryError(error.to_string()), began: true });
    }
    // The move is checked once more, because another command can switch the worktree in the instant between the last look and
    //  the write. A worktree that now belongs to another branch gets the branch put back and keeps the record; its index entries
    //  and files are still there, so the diagnostic says so rather than claim the checkout is clean.
    let after: Option<String> = symbolic_ref_awaiting(place, root).await.map_err(partly_installed)?;
    if after != intent.target_ref {
        let mut undo: Vec<&str> = vec!["update-ref"];
        if intent.target_ref.is_none() {
            undo.push("--no-deref");
        }
        undo.push(target);
        undo.push(intent.old_oid.as_str());
        undo.push(intent.new_oid.as_str());
        if let Err(error) = run_git_checked_awaiting(&place.git, &GitRequest::new(root, &undo)).await {
            debug(
                "settleFailedMerge",
                format!("the branch move of a merge whose worktree switched could not be put back: {error}").as_str(),
            );
        }
        return Err(partly_installed(a_worktree_that_switched(target, root, after.as_deref(), record)));
    }
    return Ok(Installed::Written);
}
