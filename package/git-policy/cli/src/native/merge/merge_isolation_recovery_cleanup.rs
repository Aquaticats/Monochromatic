//! What:
//!  What recovery writes to the records and side refs of a merge it settled,
//!  and the refusal that says whether anything of the merge is installed.
//! Why:
//!  A record and its side ref go together,
//! a refusal must carry whether the merge began,
//! and a saved merge whose branch
//!  advanced drops only its record;
//! the wording of all three lives beside the decision in a file the decision's own file does not
//!  carry.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await dropRecordAndSideRef({ place, root, record, intent }); return partlyInstalled(error);
//! ```

use crate::diagnostic_log::debug;
use crate::merge_isolation_journal::{MergeIntent, remove_merge_intent};
use crate::merge_isolation_recovery_steps::remove_record;
use crate::merge_isolation_recovery_steps::{MergeRefusal, RecoveryPlace};
use crate::recovery_error::RecoveryError;
use crate::transaction_git::GitRequest;
use crate::transaction_git_awaiting::run_git_checked_awaiting;
use std::path::Path;

/// What:
///  The refusal of a step that ran after the index was written,
///  so the merge is partly installed.
/// Why:
///   The branch write and the record's removal come after the index and the files,
/// and a failure there leaves a
///  merge a command must not read past.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const partlyInstalled = (error: RecoveryError): MergeRefusal => ({ error, began: true });
/// ```
pub fn partly_installed(error: RecoveryError) -> MergeRefusal {
    return MergeRefusal { error, began: true };
}

/// What:
///  A refusal about a merge that wrote nothing,
/// so a command that meets it may go on when the merge was saved.
/// Why:
///  Every refusal before the index write leaves the checkout as it was,
/// and the flag says so once for all of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const untouched = (error: RecoveryError): MergeRefusal => ({ error, began: false });
/// ```
pub fn untouched(error: RecoveryError) -> MergeRefusal {
    return MergeRefusal { error, began: false };
}

/// What:
///  Drop a record and,
/// when it named one,
/// the side ref that keeps its merge commit from being collected.
/// Why:
///   The side ref exists for the window between the save and the finish,
/// so the merge's landing is what removes it,
/// and only
///  while the ref still holds the merge's commit:
/// another command may have put its own commit on the name,
/// and that
///  commit may have no other.
/// A ref that is not deleted stays,
/// and the log says why,
/// because nothing of the merge is lost either
///  way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function dropRecordAndSideRef({ place, root, record, intent }): Promise<void>;
/// ```
pub async fn drop_record_and_side_ref_awaiting(place: &RecoveryPlace, root: &Path, record: &Path, intent: &MergeIntent) -> Result<(), RecoveryError> {
    remove_record(record)?;
    drop_side_ref_awaiting(place, root, intent).await;
    return Ok(());
}

/// What:
///  Delete the side ref a record names,
/// saying in the log when that fails.
/// Why:
///   A merge that lands must not leave a name pointing at its commit,
/// but a deletion that fails is not a reason to
///  keep the record or stop the command:
/// the commit stays,
/// and the name is what a person can delete.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function dropSideRef({ place, root, intent }): Promise<void>;
/// ```
pub async fn drop_side_ref_awaiting(place: &RecoveryPlace, root: &Path, intent: &MergeIntent) {
    let Some(name) = intent.side_ref.as_deref() else {
        return;
    };
    // The ref is deleted only while it still holds this merge's commit: another command may have put its own commit on the
    //  name, and that commit may have no other.
    let arguments: [&str; 5] = ["update-ref", "--no-deref", "-d", name, intent.new_oid.as_str()];
    if let Err(error) = run_git_checked_awaiting(&place.git, &GitRequest::new(root, &arguments)).await {
        debug(
            "settleFailedMerge",
            format!("the side ref {name} no longer holds the landed merge's commit, so it was left where it is: {error}").as_str(),
        );
    }
}

/// What:
///  Remove a record,
/// naming it when that fails.
/// Why:
///  A saved merge whose branch advanced keeps its side ref and drops only its record,
/// so no later command is
///  stopped for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(record, { force: true });
/// ```
pub fn drop_record(record: &Path) -> Result<(), RecoveryError> {
    return match remove_merge_intent(record) {
        Ok(()) => Ok(()),
        Err(error) => Err(RecoveryError(format!("removing {} failed: {error}", record.display()))),
    };
}

/// What:
///  The refusal for files the merge changes that now hold something else,
///  naming them and the record.
/// Why:
///  The text is long and says what the person can do,
/// and it belongs beside the other refusals this module keeps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// editsOnTheMergesOwnPaths(root, paths, record): RecoveryError;
/// ```
pub fn edits_on_the_merges_own_paths(root: &Path, paths: &[String], record: &Path) -> RecoveryError {
    return RecoveryError(format!(
        "A merge in {} was interrupted while Git updated these files, and they now hold neither their old nor their merged          content: {}. Nothing was changed and the record {} was kept. Save your edits to them elsewhere, restore them with \
         `git restore -- <path>`, then run any git command again so recovery can finish the merge; or remove the record to \
         abandon the merge and check `git status` yourself.",
        root.display(),
        paths.join(", "),
        record.display()
    ));
}

/// What:
///  The refusal for staged work on paths the merge replaces,
///  naming them and the record.
/// Why:
///   A staged version has no other name,
/// so it is never written over and the person is told which paths hold it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stagedWorkOnTheMergesPaths(target, root, paths, record): RecoveryError;
/// ```
pub fn staged_work_on_the_merges_paths(target: &str, root: &Path, paths: &[String], record: &Path) -> RecoveryError {
    return RecoveryError(format!(
        "A merge of {target} in {} is unfinished, and the index holds staged content for these paths that the merge would \
         replace: {}. Nothing was changed, the record {} was kept, and the staged content is still in the index. Commit or \
         unstage it, then run any git command again so recovery can finish the merge; or remove the record to abandon the \
         merge and check `git status` yourself.",
        root.display(),
        paths.join(", "),
        record.display()
    ));
}

/// What:
///  The refusal for a merge whose expected parent no commit names,
///  naming the branch,
///  the record and the commits.
/// Why:
///  The text says what to look at and what to remove,
/// and it belongs beside the other refusals this module keeps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// noCommitNamesTheMergesParent(target, root, intent, record): RecoveryError;
/// ```
pub fn no_commit_names_the_merges_parent(target: &str, root: &Path, intent: &MergeIntent, record: &Path) -> RecoveryError {
    return RecoveryError(format!(
        "{target} names no commit although the interrupted merge in {} expected {}. Nothing was changed and the record {} was \
         kept. Check `git status` and `git log {target}`; remove the record when {target} is as you want it.",
        root.display(),
        intent.old_oid,
        record.display()
    ));
}

/// What:
///  The refusal for a merge whose worktree switched branch in the instant before its branch moved,
///  with the branch already put back.
/// Why:
///   The person needs to know the branch was moved and then restored,
/// which branch the worktree has,
/// and that the record
///  waits for them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// aWorktreeThatSwitched(target, root, checkedOut, record): RecoveryError;
/// ```
pub fn a_worktree_that_switched(target: &str, root: &Path, checked_out: Option<&str>, record: &Path) -> RecoveryError {
    let landed: &str = checked_out.unwrap_or("a detached HEAD");
    return RecoveryError(format!(
        "A merge of {target} in {} was installed here and {target} was moved onto it, but this worktree now has {landed} \
         checked out: another command switched it in the instant between the last look and the branch move. The branch was put \
         back where it was and the record {} was kept, but the merge's index entries and files are still in this worktree, which \
         now belongs to {landed}, so its staged content is the merge's until you deal with it. Check `git status` and `git diff \
         --cached` here: finish the merge by checking out {target} and running any git command again, or undo what is staged \
         with `git restore --staged --worktree -- <path>`, and remove the record {} when the checkout is as you want it.",
        root.display(),
        record.display(),
        record.display()
    ));
}

/// What:
///  The refusal for a merge whose worktree switched and whose branch could not be put back,
///  with the branch still naming the merge.
/// Why:
///   A person told the branch was restored would leave a branch that names the merge,
/// so this says what the undo's
///  failure left and what moving the branch back failed on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// aBranchThatCouldNotBePutBack(target, root, checkedOut, record, cause): RecoveryError;
/// ```
pub fn a_branch_that_could_not_be_put_back(target: &str, root: &Path, checked_out: Option<&str>, record: &Path, cause: &str) -> RecoveryError {
    let landed: &str = checked_out.unwrap_or("a detached HEAD");
    return RecoveryError(format!(
        "A merge of {target} in {} was installed here and {target} was moved onto it, but this worktree now has {landed} \
         checked out: another command switched it in the instant between the last look and the branch move. Moving {target} \
         back failed, so {target} still names the merge: {cause}. The merge's index entries and files are still in this \
         worktree, which now belongs to {landed}, so its staged content is the merge's until you deal with it. Check `git \
         status` and `git diff --cached` here: check out {target} and run any git command again once whatever holds its ref \
         lock is gone, so recovery can settle the branch; or undo what is staged with `git restore --staged --worktree -- \
         <path>` and remove the record {} when the checkout is as you want it.",
        root.display(),
        record.display()
    ));
}

/// What:
///  The refusal for a merge already in its branch while this worktree belongs to another branch,
///  with the record kept.
/// Why:
///   A checkout another branch owns may hold the merge's index entries and files,
/// and the record is the only thing that
///  still names them: dropping it leaves them with nothing that ever settles them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// aLandedMergeInAnotherBranchsWorktree(target, root, checkedOut, record): RecoveryError;
/// ```
pub fn a_landed_merge_in_another_branchs_worktree(target: &str, root: &Path, checked_out: Option<&str>, record: &Path) -> RecoveryError {
    let landed: &str = checked_out.unwrap_or("a detached HEAD");
    return RecoveryError(format!(
        "A merge of {target} in {} is already in {target}, and this worktree has {landed} checked out: the merge's index \
         entries and files may still be in this worktree, which belongs to {landed}. Nothing was changed and the record {} \
         was kept. Check `git status` and `git diff --cached` here; check out {target} and run any git command again so \
         recovery can settle the merge, or undo what is staged with `git restore --staged --worktree -- <path>`, and remove \
         the record {} when the checkout is as you want it.",
        root.display(),
        record.display(),
        record.display()
    ));
}

/// What:
///  The refusal for a merge whose index entries are already installed and whose index stayed held past the wait.
/// Why:
///   A command that went on would read the half-installed merge as another session's staged work,
/// so it is stopped
///  and told what is there,
/// where the lock is,
/// and which record waits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// anInstalledMergeBehindAHeldIndex(target, root, lock, record): RecoveryError;
/// ```
pub fn an_installed_merge_behind_a_held_index(target: &str, root: &Path, lock: &Path, record: &Path) -> RecoveryError {
    return RecoveryError(format!(
        "A merge of {target} in {} is unfinished, and this merge's index entries are already installed here, but {} stayed \
         held by another program, so nothing more was written and the record {} was kept. A command is stopped rather than \
         read the checkout as another session's staged work. Check `git status` and `git diff --cached`, finish or undo what \
         is staged, then remove the record {} when the checkout is as you want it.",
        root.display(),
        lock.display(),
        record.display(),
        record.display()
    ));
}
