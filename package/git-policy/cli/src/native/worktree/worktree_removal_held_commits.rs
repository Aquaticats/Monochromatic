//! What:
//!  The commit a reference lock holds in a linked worktree,
//!  before a command that may remove that worktree:
//!  kept where a removal cannot reach it,
//!  and said to the person.
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
//!  The person is told the branch does not move and where the commit is kept,
//!  before the removal,
//!  because no command can move that branch from a worktree that is gone.
//!  The sentence is printed whether Git then removes the worktree or refuses it:
//!  the commit is kept either way,
//!  and a refused removal leaves a reference that names a commit the batch still holds.
//!  Nothing here removes a lock or a record.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const held of linkedHeldCommits(commonDir)) { await keepOnASideRef(git, held.repository, held.seq, held.new); console.error(words(held)); }
//! ```

use crate::held_commit_record::{Held, list};
use crate::held_commit_words::locks_named;
use crate::service_land_git_steps_and_import_stream::keep_on_a_side_ref;
use crate::service_scratch_ref_of_a_dropped_batch::{RefValue, kept_ref, value_of};
use crate::transaction_git::GitContext;
use std::path::{Path, PathBuf};

/// The branch as a person reads it,
/// without the `refs/heads/` every branch carries.
fn branch_of(held: &Held) -> String {
    return held.reference.trim_start_matches("refs/heads/").to_string();
}

/// What:
///  The commit each held batch of a linked worktree is kept on,
/// and the line the person reads about it;
///  `None` when the batch's commit is already kept there.
/// Why:
///   The kept reference is the one reference a removal leaves,
///  and it must not be replaced when it already names another commit:
///  that would forget a commit an earlier batch was kept under.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const kept = await valueOf(git, held.repository, keptRef(held.seq)); kept.kind === 'absent' ? (await keepOnASideRef(...), words(held)) : undefined
/// ```
fn keep_one(git: &GitContext, held: &Held) -> Option<String> {
    let name: String = kept_ref(held.seq);
    match value_of(git, held.repository.as_path(), name.as_str()) {
        RefValue::Absent => {}
        // The batch's own commit, kept there by an earlier command: nothing to write.
        RefValue::Names(named) if named == held.new => return None,
        RefValue::Names(named) => {
            return Some(format!(
                "the commit {} for {} is held in {}, and {} already names {}, so it is left there and this removal keeps nothing of it \
                 on a reference of this repository.",
                held.new,
                branch_of(held),
                held.repository.display(),
                name,
                named
            ));
        }
        RefValue::Unasked => {
            return Some(format!(
                "the commit {} for {} is held in {}, and whether {} already names a commit cannot be read, so nothing was written and Git's own \
                 answer decides.",
                held.new,
                branch_of(held),
                held.repository.display(),
                name
            ));
        }
    }
    return match keep_on_a_side_ref(git, held.repository.as_path(), held.seq, held.new.as_str()) {
        None => Some(format!(
            "the commit {} for {} is held in {} behind {}, and a command that removes that worktree leaves no record there that could move \
             the branch: it is kept at `{}`, {} stays where it is, and no command moves it by itself.",
            held.new,
            branch_of(held),
            held.repository.display(),
            locks_named(held.locks.as_slice()),
            name,
            branch_of(held)
        )),
        Some(words) => Some(format!(
            "the commit {} for {} is held in {}, and keeping it at `{}` against this removal failed ({words}); a removal of that worktree \
             leaves no record there that could move the branch.",
            held.new,
            branch_of(held),
            held.repository.display(),
            name
        )),
    };
}

/// What:
///  Keeps the commit of every held batch of every linked worktree of `common_dir`
///  on the reference a removal of that worktree cannot reach,
///  and answers the lines the person reads.
/// Why:
///   Which worktrees this command removes is Git's own decision,
///  and the wrapper hands the process to Git before it can see it,
///  so every linked worktree is served and each keeps its commits.
///  A worktree with nothing held costs one directory read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const gitDir of linkedGitDirs(commonDir)) for (const held of heldRecords(gitDir)) said.push(keepOne(git, held));
/// ```
pub(crate) fn keep_held_commits(common_dir: &Path, git: &GitContext) -> Vec<String> {
    let mut said: Vec<String> = Vec::new();
    let Ok(listing) = std::fs::read_dir(common_dir.join("worktrees")) else { return said };
    let mut directories: Vec<PathBuf> = Vec::new();
    for entry in listing.flatten() {
        let git_dir: PathBuf = entry.path();
        if git_dir.is_dir() {
            directories.push(git_dir);
        }
    }
    directories.sort();
    for git_dir in directories {
        let (records, _damaged) = list(git_dir.as_path());
        for held in records {
            if let Some(line) = keep_one(git, &held) {
                said.push(line);
            }
        }
    }
    return said;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_held_commits_tests.rs"]
mod tests;
