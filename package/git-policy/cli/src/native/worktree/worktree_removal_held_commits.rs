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
//!  Every write is a create-only one,
//! so a name another command wrote is never replaced.
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
//! for (const held of linkedHeldCommits(commonDir)) { const kept = await keepOne(git, commonDir, held); if (kept.refused) return stopped(kept.refused); console.error(words(kept)); }
//! ```

use crate::held_commit_record::{Held, list};
use crate::held_commit_words::{locks_named, removal};
use crate::service_land_git_steps_and_import_stream::keep_on_a_side_ref_only_if_free;
use crate::service_scratch_ref_of_a_dropped_batch::{RefValue, kept_ref, rescue_ref, value_of};
use crate::transaction_git::GitContext;
use std::path::{Path, PathBuf};

/// The branch as a person reads it,
/// without the `refs/heads/` every branch carries.
fn branch_of(held: &Held) -> String {
    return held.reference.short().spoken();
}

/// The conflict sentence:
/// the batch's kept reference already names another commit,
/// so it is left as it is and the removal keeps nothing of this commit there.
fn words_of_a_kept_name_taken(held: &Held, name: &str, named: &str) -> String {
    return format!(
        "the commit {} for {} is held in {}, and {} already names {}, so it is left there and this removal keeps nothing of it \
         on a reference of this repository.",
        held.new,
        branch_of(held),
        held.repository.display(),
        name,
        named
    );
}

/// The sentence for a commit kept at the batch's own kept reference.
fn words_of_a_commit_kept(held: &Held, name: &str) -> String {
    return format!(
        "the commit {} for {} is held in {} behind {}, and a command that removes that worktree leaves no record there that could move \
         the branch: it is kept at `{}`, {} stays where it is, and no command moves it by itself.",
        held.new,
        branch_of(held),
        held.repository.display(),
        locks_named(held.locks.as_slice()),
        name,
        branch_of(held)
    );
}

/// The sentence for a commit kept beside the batch's own kept reference,
/// which could not be written.
fn words_of_a_commit_kept_beside(held: &Held, name: &str, rescue: &str, words: &str) -> String {
    return format!(
        "the commit {} for {} is held in {} behind {}, and `{}` could not be written ({}), so it is kept at `{}` instead: a command \
         that removes that worktree leaves no record there that could move the branch, {} stays where it is, and no command moves it by itself.",
        held.new,
        branch_of(held),
        held.repository.display(),
        locks_named(held.locks.as_slice()),
        name,
        words,
        rescue,
        branch_of(held)
    );
}

/// The paragraph for a commit no reference of the repository could hold,
/// which stops the removal.
fn words_of_a_commit_nothing_keeps(held: &Held, name: &str, words: &str, rescue: &str, rescue_words: &str) -> String {
    return format!(
        "the commit {} for {} is held in {} behind {}, and it could not be kept on a reference of this repository: `{}` was not \
         written ({}), and neither was `{}` ({}), so a removal of that worktree would leave nothing that names the commit.",
        held.new,
        branch_of(held),
        held.repository.display(),
        locks_named(held.locks.as_slice()),
        name,
        words,
        rescue,
        rescue_words
    );
}

/// What:
///  What keeping one held commit came to.
/// Why:
///   The three answers are what the person is told and what the command does next,
///  and the refusal is not a failure of the question but a state of the repository:
///  no reference can be made to hold the commit,
/// so no command may remove the worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Keeping = { kind: 'nothing' } | { kind: 'kept'; line: string } | { kind: 'refused'; line: string };
/// ```
enum Keeping {
    /// The batch's own kept reference already names its commit,
    /// or another command wrote it while this one was keeping it,
    /// so nothing was written.
    Nothing,
    /// The commit is kept where a removal cannot reach it,
    /// and this is the line the person reads.
    Kept(String),
    /// No reference could be made to hold the commit,
    /// and this is the paragraph that says so.
    Refused(String),
}

/// What:
///  The commit each held batch of a linked worktree is kept on,
/// and the line the person reads about it;
///  `Nothing` when the batch's commit is already kept there.
/// Why:
///   The kept reference is the one reference a removal leaves,
///  and it must not be replaced when it already names another commit:
///  that would forget the commit it was made for.
///  A create-only write answers the same question for a name another command writes at the same moment
///  (`keep_on_a_side_ref_only_if_free`).
///  The write that failed is read again before the name beside it is tried,
///  so a reference another command had just written for this same commit is not written twice.
///  The question is asked in `common_dir`,
///  which is the common Git directory of the repository whose registrations are being served,
///  rather than in the worktree:
///  `git worktree prune` runs after a checkout was deleted,
///  and Git cannot answer a question asked in a directory that is not there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const kept = await valueOf(git, commonDir, keptRef(seq)); kept.kind === 'absent' ? (await keepOnlyIfFree(...), words(held)) : undefined
/// ```
fn keep_one(git: &GitContext, held: &Held, common_dir: &Path) -> Keeping {
    let name: String = kept_ref(held.seq);
    if let RefValue::Names(named) = value_of(git, common_dir, name.as_str()) {
        return if named == held.new { Keeping::Nothing } else { Keeping::Kept(words_of_a_kept_name_taken(held, name.as_str(), named.as_str())) };
    }
    let rescue: String = rescue_ref(held.seq);
    let Some(words) = keep_on_a_side_ref_only_if_free(git, common_dir, name.as_str(), held.new.as_str()) else {
        return Keeping::Kept(words_of_a_commit_kept(held, name.as_str()));
    };
    // The write failed, but another command may have written this very commit between the question and it.
    if let RefValue::Names(named) = value_of(git, common_dir, name.as_str()) {
        return if named == held.new { Keeping::Nothing } else { Keeping::Kept(words_of_a_kept_name_taken(held, name.as_str(), named.as_str())) };
    }
    let Some(also_refused) = keep_on_a_side_ref_only_if_free(git, common_dir, rescue.as_str(), held.new.as_str()) else {
        return Keeping::Kept(words_of_a_commit_kept_beside(held, name.as_str(), rescue.as_str(), words.as_str()));
    };
    return Keeping::Refused(words_of_a_commit_nothing_keeps(held, name.as_str(), words.as_str(), rescue.as_str(), also_refused.as_str()));
}

/// The lock files that would refuse these writes and stand now:
/// the lock of each name that was tried,
/// and the packed references' own.
/// Why:
///  A refusal is not actionable unless it names what stands in the way,
/// and a lock nothing holds is the one such thing the person removes.
fn standing_locks(common_dir: &Path, names: &[String]) -> Vec<PathBuf> {
    let mut standing: Vec<PathBuf> = Vec::new();
    for name in names {
        let lock: PathBuf = common_dir.join(format!("{name}.lock"));
        if lock.symlink_metadata().is_ok() {
            standing.push(lock);
        }
    }
    let packed: PathBuf = common_dir.join("packed-refs.lock");
    if packed.symlink_metadata().is_ok() {
        standing.push(packed);
    }
    return standing;
}

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
pub(crate) fn keep_held_commits(common_dir: &Path, git: &GitContext) -> KeptCommits {
    let mut kept: KeptCommits = KeptCommits { said: Vec::new(), refused: Vec::new() };
    let Ok(listing) = std::fs::read_dir(common_dir.join("worktrees")) else { return kept };
    let mut directories: Vec<PathBuf> = Vec::new();
    for entry in listing.flatten() {
        let git_dir: PathBuf = entry.path();
        if git_dir.is_dir() {
            directories.push(git_dir);
        }
    }
    directories.sort();
    let mut tried: Vec<String> = Vec::new();
    for git_dir in directories {
        let (records, _damaged) = list(git_dir.as_path());
        for held in records {
            match keep_one(git, &held, common_dir) {
                Keeping::Nothing => {}
                Keeping::Kept(line) => kept.said.push(line),
                Keeping::Refused(line) => kept.refused.push(line),
            }
            tried.push(kept_ref(held.seq));
            tried.push(rescue_ref(held.seq));
        }
    }
    if !kept.refused.is_empty() {
        kept.refused.push(nothing_was_removed(common_dir, tried.as_slice()));
    }
    return kept;
}

/// What:
///  The paragraph that stops a removal,
///  saying that nothing was removed,
///  what still holds the commits,
///  and what makes the command able to run.
/// Why:
///   A command that must stop has to say what it kept and where,
///  and the person's next step is one of two things:
///  make the directory writable,
///  or remove a lock that nothing holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `no reference of this repository could be written for the held commits above, so nothing was removed and no Git command ran: ...`
/// ```
fn nothing_was_removed(common_dir: &Path, names: &[String]) -> String {
    let directory: PathBuf = common_dir.join("refs/cli-git/kept");
    let mut paragraph: String = format!(
        "no reference of this repository could be written for the held commits above, so nothing was removed and no Git command ran: \
         the worktrees, their records of the hold and the references that name those commits are as they were. When `{}` can be \
         written again, run the same command again, and it keeps each of those commits before Git removes anything. Until then those \
         commits are held only by the worktrees' own references, and a `git gc --prune=now` run from another checkout of this \
         repository takes them, because Git there does not count another worktree's references.",
        directory.display()
    );
    let standing: Vec<PathBuf> = standing_locks(common_dir, names);
    if !standing.is_empty() {
        paragraph.push_str(
            format!(
                " The {} stands in the way; if no Git process is using this repository, remove it with `{}`.",
                locks_named(standing.as_slice()),
                removal(standing.as_slice())
            )
            .as_str(),
        );
    }
    return paragraph;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_held_commits_tests.rs"]
mod tests;
