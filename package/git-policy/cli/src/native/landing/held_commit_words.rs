//! What:
//!  What the wrapper says about a commit that a reference lock keeps off its branch:
//!  the answer the held commit ends with,
//!  and the lines every later command prints while it waits.
//! Why:
//!  The person must know three things at once,
//!  and each sentence says one of them:
//!  the branch has not moved yet,
//!  the lock is what the commit waits for and it is named,
//!  and what happens next
//!  (`SPEC.md`,
//!  "A commit held behind a reference lock").
//!  Held commits must never collect without anyone noticing:
//!  the count of the ones waiting in the worktree is part of what is printed.
//!  Nothing here claims a holder has ended,
//!  because no judgement can say so,
//!  so the words say what is known and give the one command that is safe to paste.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! console.error(heldAnswer(held, commit, waiting)); for (const line of heldNotices(gitDir)) console.error(line);
//! ```

use crate::held_commit_record::Held;
use crate::stale_ref_lock_message::shell_quote;
use std::path::PathBuf;

/// What:
///  The word that agrees with a count of held commits.
/// Why:
///   The count is read on every command while anything is held,
///  so it reads as a sentence and not as `1 commit(s)`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// heldCount(1) // '1 held commit waits in this worktree.'
/// ```
pub fn counted(count: usize) -> String {
    return if count == 1 { format!("{count} held commit waits in this worktree.") } else { format!("{count} held commits wait in this worktree.") };
}

/// The branch as a person reads it,
/// without the `refs/heads/` every branch carries.
fn branch_of(reference: &crate::ref_name::RefName) -> String {
    // The words name the branch as a shell word, so a name that is not text is shown in the form a person can type.
    return reference.short().spoken();
}

/// What:
///  The locks as one phrase,
///  each in backticks.
/// Why:
///   A person pastes these paths into a shell,
///  so every byte of them is shown and nothing is abbreviated.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// locksNamed(locks) // 'the lock `a`' or 'the locks `a` and `b`'
/// ```
pub fn locks_named(locks: &[PathBuf]) -> String {
    let mut names: Vec<String> = Vec::new();
    for lock in locks {
        names.push(format!("`{}`", lock.display()));
    }
    return match names.as_slice() {
        [only] => format!("the lock {only}"),
        [first, second] => format!("the locks {first} and {second}"),
        many => format!("the locks {}", many.join(", ")),
    };
}

/// What:
///  The one safe command for a lock no Git holds:
///  `rm --` with every path quoted.
/// Why:
///   The wrapper never removes a reference lock itself
///  (`SPEC.md`,
///  "The rule:
/// a lock is removed only on a proof about its holder"),
///  so the person is given the command that removes it,
///  quoted so a shell reads exactly that path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `rm -- ${locks.map(shellQuote).join(' ')}`
/// ```
pub fn removal(locks: &[PathBuf]) -> String {
    let mut words: Vec<String> = Vec::new();
    for lock in locks {
        words.push(shell_quote(lock.as_path()));
    }
    return format!("rm -- {}", words.join(" "));
}

/// What:
///  The words a commit ends with when a lock nothing proves gone kept its branch write from happening,
///  on standard error,
///  with exit `0`.
/// Why:
///   The owner's rule is that commits never fail,
///  so the command says what became of the commit instead of failing:
///  the commit is written and kept,
///  the branch has not moved yet,
///  the lock is named,
///  and the next command moves the branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `the commit ${commit} is held: ${branch} has not moved yet, because ${locks} stands in the way ...`
/// ```
pub fn answer(held: &Held, commit: &str, waiting: usize) -> String {
    return format!(
        "the commit {commit} is held: {} has not moved yet, because {} stands in the way and nothing proves that a Git process still holds \
it. The next command that finds the lock gone moves the branch. If no Git process is using this repository, remove it with `{}`; the commit \
is kept at `{}` either way. {}",
        branch_of(&held.reference),
        locks_named(held.locks.as_slice()),
        removal(held.locks.as_slice()),
        crate::service_scratch_ref_of_a_dropped_batch::scratch_ref(held.seq),
        counted(waiting)
    );
}

/// What:
///  The heading a command prints above the lines of the commits it finds held.
/// Why:
///  The count is what tells a person that held commits are collecting,
///  and the heading is printed before the lock of any of them is read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${count} held commit(s) wait in this worktree, behind a reference lock:`
/// ```
pub fn heading(count: usize) -> String {
    return if count == 1 { format!("{count} held commit waits in this worktree, behind a reference lock:") } else { format!("{count} held commits wait in this worktree, behind a reference lock:") };
}

/// What:
///  The line every command prints while a held commit waits for its lock.
/// Why:
///   The record is the only place the state exists,
///  so the line cannot disagree with it,
///  and a held commit that nobody removes the lock for keeps being said.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `the commit ${commit} for ${branch} is held: the branch has not moved yet, and it waits for ${locks}`
/// ```
pub fn waiting(held: &Held, commit: &str) -> String {
    return format!(
        "the commit {commit} for {} is held: the branch has not moved yet, and it waits for {}. The next command that finds the lock gone \
moves the branch; if no Git process is using this repository, remove it with `{}`.",
        branch_of(&held.reference),
        locks_named(held.locks.as_slice()),
        removal(held.locks.as_slice())
    );
}

/// What:
///  The line for a held commit whose change cannot be combined with the branch as it is now.
/// Why:
///  A conflict is the one case where nothing decides for the person:
///  the commit is kept,
///  and the paths that conflict are named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `the commit ${commit} is held: ${branch} moved, and the change does not combine with it on ${paths}`
/// ```
pub fn conflicts(held: &Held, commit: &str, paths: &[Vec<u8>]) -> String {
    let mut named: Vec<String> = Vec::new();
    for path in paths {
        named.push(String::from_utf8_lossy(path).into_owned());
    }
    return format!(
        "the commit {commit} is held: {} moved while the lock stood, and the change does not combine with the branch as it is now on {}. \
The commit is kept at `{}`; nothing was changed. Bring the change onto the branch yourself, then remove the record at `{}`.",
        branch_of(&held.reference),
        named.join(", "),
        crate::service_scratch_ref_of_a_dropped_batch::scratch_ref(held.seq),
        crate::held_commit_record::path_of(held.git_dir.as_path(), held.seq).display()
    );
}

/// What:
///  The line for a held commit whose branch move waits for something this command will not do.
/// Why:
///  A record that cannot be read,
///  a branch value this release cannot carry the commit onto and a commit made again that would need a program to sign it
///  are each a state the person has to know about,
///  and none of them is a reason to lose the commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `the commit ${commit} is held: ${branch} has not moved yet, and this command cannot move it, because ${because}`
/// ```
pub fn cannot(held: &Held, commit: &str, because: &str) -> String {
    return format!(
        "the commit {commit} is held: {} has not moved yet, and this command cannot move it, because {because}. The commit is kept at \
`{}`; nothing was changed.",
        branch_of(&held.reference),
        crate::service_scratch_ref_of_a_dropped_batch::scratch_ref(held.seq)
    );
}

/// What:
///  The line for a held commit that has just landed.
/// Why:
///  The branch moved,
///  so the record is done;
///  the person is told which commit it is,
///  and that no gate released its push,
///  because the command that made the commit had already ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `the held commit ${commit} landed on ${branch}: ...`
/// ```
pub fn landed(held: &Held, commit: &str, the_index_waits: bool) -> String {
    let branch: String = branch_of(&held.reference);
    let index: &str = if the_index_waits { "Another program wrote the index after the landing saw it, so its entries are left as that program made them and the next commit in this worktree sets them; until then `git status` shows the commit's paths. " } else { "" };
    return format!(
        "the held commit {commit} landed on {branch}: the lock that held its branch write is gone, so the branch has moved. {index}Its push \
was not released, because the command that made the commit had already ended and ran no post-commit gate; the next commit or push from \
{branch} carries it."
    );
}

/// What:
///  The line for a record whose commit the branch already shows.
/// Why:
///  A record can outlive its batch when the branch moved past the commit or the settlement finished and a kill kept the
///  record,
///  and saying so keeps a person from looking for the commit twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `the held commit ${commit} for ${branch} is already on the branch`
/// ```
pub fn already_there(held: &Held, commit: &str) -> String {
    return format!(
        "the held commit {commit} for {} is already on the branch, so nothing waits any more; its record at `{}` is removed.",
        branch_of(&held.reference),
        crate::held_commit_record::path_of(held.git_dir.as_path(), held.seq).display()
    );
}

/// What:
///  The line for the records of a worktree that cannot be read.
/// Why:
///   A record names a commit kept on a side reference,
///  so a record this build cannot read is said and never removed:
///  removing it would leave a commit nothing points at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `these held-commit records cannot be read ...`
/// ```
pub fn unreadable(paths: &[PathBuf]) -> String {
    let mut names: Vec<String> = Vec::new();
    for path in paths {
        names.push(format!("`{}`", path.display()));
    }
    return format!("these held-commit records cannot be read, so nothing was done about the commits they name and they are kept: {}", names.join(", "));
}
