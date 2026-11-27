//! What:
//!  The words a recovery refuses a merge with:
//!  each names the paths,
//!  the record and what the person can do.
//! Why:
//!  The text of a refusal is long and says what a person can do about it,
//!  and the settle of a saved merge,
//!  its records and its side refs is a decision of its own;
//!  the two live in separate files,
//!  and every caller of the recovery reaches these words through the decision's file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refusal = editsOnTheMergesOwnPaths(root, paths, record); // one RecoveryError per shape
//! ```

/// The intent record a refusal names.
use crate::merge_isolation_journal::MergeIntent;
/// The fail-closed recovery failure a refusal is spelled in.
use crate::recovery_error::RecoveryError;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

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
pub fn edits_on_the_merges_own_paths(
    root: &Path,
    paths: &[String],
    record: &Path,
) -> RecoveryError {
    return RecoveryError(format!(
        "A merge in {} was interrupted while Git updated these files, and they now hold neither their old nor their merged          content: {}. Nothing was changed and the record {} was kept. Save your edits to them elsewhere, restore them with \
         `git restore --no-enforce-worktree -- <path>`, then run any git command again so recovery can finish the merge; or remove the record to \
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
pub fn staged_work_on_the_merges_paths(
    target: &str,
    root: &Path,
    paths: &[String],
    record: &Path,
) -> RecoveryError {
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
pub fn no_commit_names_the_merges_parent(
    target: &str,
    root: &Path,
    intent: &MergeIntent,
    record: &Path,
) -> RecoveryError {
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
pub fn a_worktree_that_switched(
    target: &str,
    root: &Path,
    checked_out: Option<&str>,
    record: &Path,
) -> RecoveryError {
    let landed: &str = checked_out.unwrap_or("a detached HEAD");
    return RecoveryError(format!(
        "A merge of {target} in {} was installed here and {target} was moved onto it, but this worktree now has {landed} \
         checked out: another command switched it in the instant between the last look and the branch move. The branch was put \
         back where it was and the record {} was kept, but the merge's index entries and files are still in this worktree, which \
         now belongs to {landed}, so its staged content is the merge's until you deal with it. Check `git status` and `git diff \
         --cached` here: finish the merge by checking out {target} and running any git command again, or undo what is staged \
         with `git restore --no-enforce-worktree --staged --worktree -- <path>`, and remove the record {} when the checkout is as you want it.",
        root.display(),
        record.display(),
        record.display()
    ));
}

/// What:
///  The refusal for a merge whose worktree switched and whose branch could not be put back,
///  with the branch still naming the merge.
/// Why:
///   A person told the branch was restored would leave a branch that names the merge;
///  this says what the undo's failure left,
///  and what moving the branch back failed on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// aBranchThatCouldNotBePutBack(target, root, checkedOut, record, cause): RecoveryError;
/// ```
pub fn a_branch_that_could_not_be_put_back(
    target: &str,
    root: &Path,
    checked_out: Option<&str>,
    record: &Path,
    cause: &str,
) -> RecoveryError {
    let landed: &str = checked_out.unwrap_or("a detached HEAD");
    return RecoveryError(format!(
        "A merge of {target} in {} was installed here and {target} was moved onto it, but this worktree now has {landed} \
         checked out: another command switched it in the instant between the last look and the branch move. Moving {target} \
         back failed, so {target} still names the merge: {cause}. The merge's index entries and files are still in this \
         worktree, which now belongs to {landed}, so its staged content is the merge's until you deal with it. Check `git \
         status` and `git diff --cached` here: check out {target} and run any git command again once whatever holds its ref \
         lock is gone, so recovery can settle the branch; or undo what is staged with `git restore --no-enforce-worktree --staged --worktree -- \
         <path>` and remove the record {} when the checkout is as you want it.",
        root.display(),
        record.display()
    ));
}

/// What:
///  The refusal for a merge already in its branch while this worktree belongs to another branch,
///  with the record kept.
/// Why:
///   A checkout another branch owns may hold the merge's index entries and files;
///  the record is the only thing that still names them.
///  Dropping it would leave them with nothing that ever settles them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// aLandedMergeInAnotherBranchsWorktree(target, root, checkedOut, record): RecoveryError;
/// ```
pub fn a_landed_merge_in_another_branchs_worktree(
    target: &str,
    root: &Path,
    checked_out: Option<&str>,
    record: &Path,
) -> RecoveryError {
    let landed: &str = checked_out.unwrap_or("a detached HEAD");
    return RecoveryError(format!(
        "A merge of {target} in {} is already in {target}, and this worktree has {landed} checked out: the merge's index \
         entries and files may still be in this worktree, which belongs to {landed}. Nothing was changed and the record {} \
         was kept. Check `git status` and `git diff --cached` here; check out {target} and run any git command again so \
         recovery can settle the merge, or undo what is staged with `git restore --no-enforce-worktree --staged --worktree -- <path>`, and remove \
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
pub fn an_installed_merge_behind_a_held_index(
    target: &str,
    root: &Path,
    lock: &Path,
    record: &Path,
) -> RecoveryError {
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
