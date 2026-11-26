//! What:
//!  The commit a reference lock holds in a linked worktree,
//!  before a command that may remove that worktree:
//!  kept where a removal cannot reach it,
//!  said to the person,
//!  and,
//! when no reference can hold it,
//! the command stopped
//!  with what is still held and where.
//! Why:
//!   A held commit is an object no branch names,
//!  held by the worktree's own Git directory and by the per-worktree reference
//!  `refs/worktree/cli-git-service/<batch>`,
//!  and `git worktree remove` deletes both.
//!  The record of the hold goes with them,
//!  so no command of any worktree settles that batch afterwards,
//!  and no reference names the commit:
//!  `git gc --prune=now` then takes it,
//!  which loses a commit the wrapper said it kept
//!  (the owner's rule "never discard user data").
//!  The commit is kept on `refs/cli-git/kept/<batch>`,
//!  a reference of the common Git directory that a removal leaves alone,
//!  before Git runs,
//!  because the wrapper hands the removal to Git and cannot look afterwards
//!  (`worktree_removal_rescue.rs`).
//!  The questions and the write are asked in the common Git directory,
//!  which is there whenever a registration is there to remove:
//!  `git worktree prune` exists for the state a person leaves by deleting a checkout,
//!  and a question asked in that deleted checkout is one Git cannot answer,
//!  so a commit went unkept in the one case the command is for
//!  (a review of `ee545033ad`,
//! finding 1).
//!  `refs/cli-git/kept/<batch>` can be locked by another command,
//!  or lie in a directory that refuses writes,
//!  so the commit is kept on `refs/cli-git/kept/<batch>-rescue` then,
//!  and only when no reference at all can be written does the command stop before Git runs
//!  (the same review,
//! finding 2).
//!  A name already written for another batch's commit is left where it is,
//!  because replacing it would forget the commit it was made for,
//!  and this batch's commit goes beside it like one a lock refused:
//!  reading that collision as a keep kept nothing,
//!  and the removal that followed took the last reference to the commit
//!  (a review of `a735f75caa`,
//! finding 1).
//!  A record this build cannot read names a batch whose commit is held by the per-worktree reference alone,
//!  so that commit is read from the reference and kept the same way
//!  (a review of the landed sweep,
//!  round 210).
//!  Nothing here removes a lock or a record.
//! This command is split across siblings because this file is over the line limit:
//! this one serves every linked worktree and answers the sweep,
//! `worktree_removal_held_commits_words.rs` holds the words a person reads,
//! `worktree_removal_held_commits_keeping.rs` keeps one readable record's commit,
//! and `worktree_removal_held_commits_unreadable_record.rs` keeps the commit behind a record this build cannot read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const held of linkedHeldCommits(commonDir)) { const kept = await keepOne(git, commonDir, held); if (kept.refused) return stopped(kept.refused); console.error(words(kept)); }
//! ```

use crate::held_commit_record::list;
use crate::service_scratch_ref_of_a_dropped_batch::{
    kept_ref,
    rescue_ref,
};
use crate::transaction_git::GitContext;
use crate::worktree_removal_held_commits_keeping::{
    Keeping,
    keep_one,
};
use crate::worktree_removal_held_commits_unreadable_record::{
    UnreadableKeeping,
    keep_the_commit_of_an_unreadable_record,
    worktree_of,
};
use crate::worktree_removal_held_commits_words::nothing_was_removed;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The lines the person reads about the commits that are kept,
/// and the paragraphs that stop the command for the ones no reference could hold.
/// Why:
///   What a command does next depends on whether any commit stayed unkept,
///  and a caller that read only lines would run the removal that deletes the last holder of a commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type KeptCommits = { said: string[]; refused: string[] };
/// ```
pub(crate) struct KeptCommits {
    /// What was kept and where,
    /// in the order the batches were served.
    pub said: Vec<String>,
    /// One paragraph for each held commit no reference could keep,
    /// then the paragraph that says nothing was removed and what to do next;
    /// empty when every commit was kept.
    pub refused: Vec<String>,
}

/// What:
///  Keeps the commit of every held batch of every linked worktree of `common_dir`
///  on the reference a removal of that worktree cannot reach,
///  and answers the lines the person reads,
///  and the refusal that stops the command when one such commit cannot be kept at all.
/// Why:
///   Which worktrees this command removes is Git's own decision,
///  and the wrapper hands the process to Git before it can see it,
///  so every linked worktree is served and each keeps its commits.
///  A worktree with nothing held costs one directory read.
///  Every held commit is served even after one cannot be kept,
///  because the command does not run:
/// the worktrees stay,
///  and the lines say what each of their commits came to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const gitDir of linkedGitDirs(commonDir)) for (const held of heldRecords(gitDir)) said.push(keepOne(git, commonDir, held));
/// ```
pub(crate) fn keep_held_commits(
    common_dir: &Path,
    git: &GitContext,
) -> KeptCommits {
    let mut kept: KeptCommits = KeptCommits {
        said: Vec::new(),
        refused: Vec::new(),
    };
    let Ok(listing) = std::fs::read_dir(common_dir.join("worktrees")) else {
        return kept;
    };
    let mut directories: Vec<PathBuf> = Vec::new();
    for entry in listing.flatten() {
        let git_dir: PathBuf = entry.path();
        if git_dir.is_dir() {
            directories.push(git_dir);
        }
    }
    directories.sort();
    let mut tried: Vec<String> = Vec::new();
    // The worktrees whose commit no reference could hold, which is what the closing paragraph names the removal of.
    let mut unkept_worktrees: Vec<PathBuf> = Vec::new();
    for git_dir in directories {
        let (records, damaged) = list(git_dir.as_path());
        for held in records {
            match keep_one(git, &held, common_dir) {
                Keeping::Nothing => {}
                Keeping::Kept(line) => kept.said.push(line),
                Keeping::Refused(line) => {
                    kept.refused.push(line);
                    unkept_worktrees.push(held.repository.clone());
                }
            }
            tried.push(kept_ref(held.seq));
            tried.push(rescue_ref(held.seq));
        }
        // A record this build cannot read names a batch whose commit the per-worktree reference holds alone, and a removal
        // deletes that reference with the worktree's own Git directory. The commit is read from it and kept where a removal
        // cannot reach it; a batch nothing of this repository can name keeps the worktree
        // (a review of the landed sweep, round 210).
        for record in damaged {
            match keep_the_commit_of_an_unreadable_record(
                git,
                common_dir,
                git_dir.as_path(),
                record.as_path(),
            ) {
                UnreadableKeeping::Kept(line) => kept.said.push(line),
                UnreadableKeeping::Refused(line) => {
                    kept.refused.push(line);
                    let holder: PathBuf = match worktree_of(git_dir.as_path()) {
                        Some(found) => found,
                        None => git_dir.clone(),
                    };
                    unkept_worktrees.push(holder);
                }
            }
        }
    }
    if !kept.refused.is_empty() {
        kept.refused.push(nothing_was_removed(
            common_dir,
            tried.as_slice(),
            unkept_worktrees.as_slice(),
        ));
    }
    return kept;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_held_commits_tests.rs"]
mod tests;
