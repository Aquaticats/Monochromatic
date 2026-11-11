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
    return held.reference.trim_start_matches("refs/heads/").to_string();
}

/// The clause saying why a kept reference was not written:
/// it names another commit,
/// so it is left as it is and this commit is kept beside it.
fn words_of_a_name_already_taken(named: &str) -> String {
    return format!("it already names {named}, which is another commit, so it is left there");
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
    /// A reference of this repository now names the commit,
    /// and this is the line the person reads.
    Kept(String),
    /// No reference could be made to hold the commit,
    /// and this is the paragraph that says so.
    Refused(String),
}

/// Whether the reference `name` of `common_dir` names this very commit.
/// Why:
///  Every answer of keeping a commit rests on which commit a reference names now,
///  so the question is asked once here for a name that may be absent,
///  unreadable,
///  or already written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// namesThisCommit(git, commonDir, 'refs/cli-git/kept/7', held.new)
/// ```
fn names_this_commit(git: &GitContext, common_dir: &Path, name: &str, commit: &str) -> bool {
    return matches!(value_of(git, common_dir, name), RefValue::Names(named) if named == commit);
}

/// What:
///  Why a kept reference was not written,
///  as the person reads it.
/// Why:
///  A locked name and a name another batch's commit is kept under are two different answers,
///  and only after the write failed can the read tell them apart:
///  the person needs to know that a commit is kept beside the name,
///  not that a lock stood there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// takenWords(git, commonDir, name, refused) // the collision clause, or Git's own words
/// ```
fn taken_words(git: &GitContext, common_dir: &Path, held: &Held, name: &str, refused: &str) -> String {
    // `match` on what the reference names now: another commit is said as itself, anything else is Git's own words.
    return match value_of(git, common_dir, name) {
        RefValue::Names(named) if named != held.new => words_of_a_name_already_taken(named.as_str()),
        _ => refused.to_owned(),
    };
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
///  A name that holds another batch's commit keeps nothing of this one,
///  so the write beside it is what makes the commit kept;
///  only when that write fails too is the commit held by no reference of the repository,
///  and that alone is a refusal.
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
    let rescue: String = rescue_ref(held.seq);
    if names_this_commit(git, common_dir, name.as_str(), held.new.as_str()) {
        return Keeping::Nothing;
    }
    let Some(words) = keep_on_a_side_ref_only_if_free(git, common_dir, name.as_str(), held.new.as_str()) else {
        return Keeping::Kept(words_of_a_commit_kept(held, name.as_str()));
    };
    // The write failed, but another command may have written this very commit between the question and it.
    if names_this_commit(git, common_dir, name.as_str(), held.new.as_str()) {
        return Keeping::Nothing;
    }
    // The name belongs to another batch's commit, or a lock stands on it; either way this commit needs the name beside it.
    let taken: String = taken_words(git, common_dir, held, name.as_str(), words.as_str());
    if names_this_commit(git, common_dir, rescue.as_str(), held.new.as_str()) {
        return Keeping::Nothing;
    }
    let Some(also_refused) = keep_on_a_side_ref_only_if_free(git, common_dir, rescue.as_str(), held.new.as_str()) else {
        return Keeping::Kept(words_of_a_commit_kept_beside(held, name.as_str(), rescue.as_str(), taken.as_str()));
    };
    if names_this_commit(git, common_dir, rescue.as_str(), held.new.as_str()) {
        return Keeping::Nothing;
    }
    return Keeping::Refused(words_of_a_commit_nothing_keeps(held, name.as_str(), taken.as_str(), rescue.as_str(), also_refused.as_str()));
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
    // The worktrees whose commit no reference could hold, which is what the closing paragraph names the removal of.
    let mut unkept_worktrees: Vec<PathBuf> = Vec::new();
    for git_dir in directories {
        let (records, _damaged) = list(git_dir.as_path());
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
    }
    if !kept.refused.is_empty() {
        kept.refused.push(nothing_was_removed(common_dir, tried.as_slice(), unkept_worktrees.as_slice()));
    }
    return kept;
}

/// What:
///  The command that finishes a removal this refusal stopped,
///  one per worktree that holds a commit no reference could keep.
/// Why:
///   The refusal used to name the command the person had just run,
///  and after a merge that command is up to date:
///  it looks at no worktree and writes no reference,
///  so following it finished nothing and said nothing
///  (a cross-view review of the landed sweep,
///  finding 2).
///  The removal of the worktree itself is the command that keeps the commit:
///  it runs this keeper before Git removes anything,
///  and every removal route of this program does.
///  A worktree whose checkout is already gone is finished by `git worktree prune`,
///  which routes the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removalCommands(holders) // 'run `git worktree remove ...`'
/// ```
fn removal_commands(holders: &[PathBuf]) -> String {
    let mut commands: Vec<String> = Vec::new();
    let mut prune: bool = false;
    for holder in holders {
        if holder.is_dir() {
            commands.push(format!("`git worktree remove {}`", crate::shell_quote::shell_quote(holder.as_os_str().as_encoded_bytes())));
        } else {
            prune = true;
        }
    }
    if prune {
        commands.push(String::from("`git worktree prune`"));
    }
    // `match` on how many commands there are: one reads as one sentence, none is the plain `git worktree remove` shape.
    return match commands.as_slice() {
        [] => String::from("run `git worktree remove <the path of each worktree above>`"),
        [only] => format!("run {only}"),
        many => format!("run {}", many.join(" or ")),
    };
}

/// What:
///  The paragraph that stops a removal,
///  saying that nothing was removed,
///  what still holds the commits,
///  and what makes the command able to run.
/// Why:
///   A command that must stop has to say what it kept and where,
///  and the person's next step is one of two things:
///  make the directory writable and remove the worktree,
///  or remove a lock that nothing holds.
///  The removal of the worktree is what keeps the commit,
///  so the paragraph names it for each worktree that holds one,
///  and it never names a command that is up to date and removes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `no reference of this repository could be written for the held commits above, so nothing was removed ...`
/// ```
fn nothing_was_removed(common_dir: &Path, names: &[String], holders: &[PathBuf]) -> String {
    let directory: PathBuf = common_dir.join("refs/cli-git/kept");
    let mut paragraph: String = format!(
        "no reference of this repository could be written for the held commits above, so nothing was removed and no removal ran: \
         the worktrees, their records of the hold and the references that name those commits are as they were. When `{}` can be \
         written again, {}: the commit of each worktree above is kept on a reference of this repository before Git removes \
         anything. Until then those commits are held only by the worktrees' own references, and a `git gc --prune=now` run from \
         another checkout of this repository takes them, because Git there does not count another worktree's references.",
        directory.display(),
        removal_commands(holders)
    );
    let standing: Vec<PathBuf> = standing_locks(common_dir, names);
    if !standing.is_empty() {
        paragraph.push_str(
            format!(
                " That removal needs {} gone first; if no Git process is using this repository, remove {} with `{}`.",
                locks_named(standing.as_slice()),
                if standing.len() == 1 { "it" } else { "them" },
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
