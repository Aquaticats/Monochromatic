//! What:
//!  The keeping of one held commit a record names:
//!  the question whether a name of the common Git directory already holds this commit,
//!  the create-only write of the batch's own kept reference,
//!  the write beside it when another commit holds that name or a lock stands on it,
//!  and the record of the worktree that write was made for.
//! Why:
//!   A held commit lives on the worktree's own reference and in its own Git directory,
//!  and a removal deletes both,
//!  so the commit reaches a reference of the common Git directory before Git runs.
//!  Every write is create-only,
//!  so a name another command wrote for another commit is never replaced,
//!  and the write that failed is read again before the name beside it is tried,
//!  so a reference another command had just written for this same commit is not written twice.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const kept = keepOne(git, commonDir, held); // nothing, one line for the person, or the refusal
//! ```

use crate::held_commit_record::Held;
use crate::service_land_side_reference_write::keep_on_a_side_ref;
use crate::service_scratch_ref_of_a_dropped_batch::{
    RefValue,
    kept_ref,
    rescue_ref,
    value_of,
};
use crate::transaction_git::GitContext;
use crate::worktree_removal_held_commits_words::{
    words_of_a_commit_kept,
    words_of_a_commit_kept_beside,
    words_of_a_commit_nothing_keeps,
    words_of_a_name_already_taken,
};
use std::path::Path;

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
pub(crate) enum Keeping {
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
pub(crate) fn names_this_commit(
    git: &GitContext,
    common_dir: &Path,
    name: &str,
    commit: &str,
) -> bool {
    return matches!(value_of(git, common_dir, name), RefValue::Names(named) if named == commit);
}

/// What:
///  Record the worktree a held commit was kept for,
/// beside the reference that keeps it.
/// Why:
///   The reference outlives the worktree,
/// and the pruning's line about a pruned commit names
///  the worktree and batch it came from:
///  this is the one moment both are known together
///  (`kept_reference_record.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordARemovalsKeptReference(commonDir, name, held.new, held.repository, Date.now());
/// ```
pub(crate) fn remember_the_worktree(
    common_dir: &Path,
    name: &str,
    held: &Held,
) {
    remember_the_commit_of_a_worktree(
        common_dir,
        name,
        held.new.as_str(),
        held.repository.as_path(),
    );
}

/// `remember_the_worktree` for a commit and a worktree a caller holds apart from the record.
pub(crate) fn remember_the_commit_of_a_worktree(
    common_dir: &Path,
    name: &str,
    commit: &str,
    worktree: &Path,
) {
    crate::kept_reference_record::record_a_removals_kept_reference(
        common_dir,
        name,
        commit,
        worktree,
        crate::kept_reference_pass_answers::now_ms(),
    );
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
pub(crate) fn taken_words(
    git: &GitContext,
    common_dir: &Path,
    held: &Held,
    name: &str,
    refused: &str,
) -> String {
    // `match` on what the reference names now: another commit is said as itself, anything else is Git's own words.
    return match value_of(git, common_dir, name) {
        RefValue::Names(named) if named != held.new => {
            words_of_a_name_already_taken(named.as_str())
        }
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
///  (`keep_on_a_side_ref`).
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
pub(crate) fn keep_one(
    git: &GitContext,
    held: &Held,
    common_dir: &Path,
) -> Keeping {
    let name: String = kept_ref(held.seq);
    let rescue: String = rescue_ref(held.seq);
    if names_this_commit(git, common_dir, name.as_str(), held.new.as_str()) {
        return Keeping::Nothing;
    }
    let Some(words) = keep_on_a_side_ref(git, common_dir, name.as_str(), held.new.as_str()) else {
        remember_the_worktree(common_dir, name.as_str(), held);
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
    let Some(also_refused) =
        keep_on_a_side_ref(git, common_dir, rescue.as_str(), held.new.as_str())
    else {
        remember_the_worktree(common_dir, rescue.as_str(), held);
        return Keeping::Kept(words_of_a_commit_kept_beside(
            held,
            name.as_str(),
            rescue.as_str(),
            taken.as_str(),
        ));
    };
    if names_this_commit(git, common_dir, rescue.as_str(), held.new.as_str()) {
        return Keeping::Nothing;
    }
    return Keeping::Refused(words_of_a_commit_nothing_keeps(
        held,
        name.as_str(),
        taken.as_str(),
        rescue.as_str(),
        also_refused.as_str(),
    ));
}
