//! What:
//!  The words a person reads about a held commit:
//!  the sentence for a commit kept at the batch's own reference,
//!  the sentence for one kept beside a name another commit holds,
//!  the paragraph for a commit no reference could hold,
//!  the sentences about a record this build cannot read,
//!  and the paragraph that stops a removal and names the command that finishes it.
//! Why:
//!   A command that keeps a commit says where it is before Git removes the worktree,
//!  because no command can move that branch afterwards,
//!  and a command that must stop says what still holds the commit and what makes the
//!  removal able to run.
//!  The paragraph names the locks that stand in the way,
//!  which is the one thing there a person removes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! console.error(wordsOfACommitKept(held, name));
//! ```

use crate::held_commit_record::Held;
use crate::held_commit_words::{
    locks_named,
    removal,
};
use std::path::{
    Path,
    PathBuf,
};

/// The branch as a person reads it,
/// without the `refs/heads/` every branch carries.
pub(crate) fn branch_of(held: &Held) -> String {
    return held.reference.short().spoken();
}

/// The clause saying why a kept reference was not written:
/// it names another commit,
/// so it is left as it is and this commit is kept beside it.
pub(crate) fn words_of_a_name_already_taken(named: &str) -> String {
    return format!("it already names {named}, which is another commit, so it is left there");
}

/// The sentence for a commit kept at the batch's own kept reference.
pub(crate) fn words_of_a_commit_kept(
    held: &Held,
    name: &str,
) -> String {
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
pub(crate) fn words_of_a_commit_kept_beside(
    held: &Held,
    name: &str,
    rescue: &str,
    words: &str,
) -> String {
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
pub(crate) fn words_of_a_commit_nothing_keeps(
    held: &Held,
    name: &str,
    words: &str,
    rescue: &str,
    rescue_words: &str,
) -> String {
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
///  The sentence for a commit kept from the worktree's own reference of a batch whose record cannot be read.
/// Why:
///   The person has to know which record was unreadable,
///  which reference was read instead and where the commit is now,
///  because the record they may have to clear is the only other account of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `the record ${record} cannot be read ... the commit ${commit} is kept at ${name}`
/// ```
pub(crate) fn words_of_a_commit_kept_from_an_unreadable_record(
    record: &Path,
    scratch: &str,
    commit: &str,
    name: &str,
) -> String {
    return format!(
        "the record {} cannot be read by this build, so the commit {} that its batch's own reference `{}` names is kept at `{}`: \
         a command that removes that worktree leaves no record there that could move the branch, and that reference is the \
         commit's only other holder. The record itself is left where it is.",
        record.display(),
        commit,
        scratch,
        name
    );
}

/// What:
///  The paragraph that keeps a worktree whose held record cannot be read and whose commit nothing a removal spares names.
/// Why:
///   The worktree's own Git directory holds the record and the per-worktree reference,
///  and a removal deletes both,
///  so the removal is what must not run,
///  and the person reads which record it was and what stood in the way of keeping the commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `the record ${record} cannot be read ... so the worktree ${worktree} stays`
/// ```
pub(crate) fn words_of_a_commit_nothing_keeps_behind_an_unreadable_record(
    record: &Path,
    worktree: &Path,
    why: &str,
) -> String {
    return format!(
        "the record {} cannot be read by this build, and {}: so no reference of this repository names the commit behind it, and \
         the worktree {} stays, because its own Git directory and that reference are the only holders of whatever the record \
         names. Keep the commit first, or clear the record yourself once you know what it held.",
        record.display(),
        why,
        worktree.display()
    );
}

/// The lock files that would refuse these writes and stand now:
/// the lock of each name that was tried,
/// and the packed references' own.
/// Why:
///  A refusal is not actionable unless it names what stands in the way,
/// and a lock nothing holds is the one such thing the person removes.
pub(crate) fn standing_locks(
    common_dir: &Path,
    names: &[String],
) -> Vec<PathBuf> {
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
pub(crate) fn removal_commands(holders: &[PathBuf]) -> String {
    let mut commands: Vec<String> = Vec::new();
    let mut prune: bool = false;
    for holder in holders {
        if holder.is_dir() {
            commands.push(format!(
                "`git worktree remove {}`",
                crate::shell_quote::shell_quote(holder.as_os_str().as_encoded_bytes())
            ));
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
pub(crate) fn nothing_was_removed(
    common_dir: &Path,
    names: &[String],
    holders: &[PathBuf],
) -> String {
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
