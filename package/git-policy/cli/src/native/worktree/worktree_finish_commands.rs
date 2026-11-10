//! What:
//!  The two commands a message prints for a worktree an interrupted `git worktree add` left:
//!  the one that finishes
//!  its checkout and the one that removes it.
//! Why:
//!  Both the settlement of an interrupted add and the placement ladder that could not complete a
//!  checkout print them,
//!  and one owner keeps the two messages saying the same thing.
//!  `git reset --hard` alone does not run there:
//!  a
//!  worktree whose checkout never ran reads as a worktree holding uncommitted changes,
//!  so
//!  `linked-worktree-only` refuses a hard reset in it and the person who pasted the printed line was
//!  refused again.
//!  The finish command therefore names that policy's hatch,
//!  and the sentence says what the
//!  reset discards.
//!  The path is quoted the way a shell reads it back,
//!  because the person pastes the line.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // finishAndRemove('/tmp/half') => ['git -C /tmp/half reset --no-enforce-worktree --hard',
//! //   'git worktree remove --force /tmp/half']
//! ```

use crate::escape_hatch::WORKTREE_ENFORCEMENT_ESCAPE_HATCH;
/// `as_bytes` on an `OsStr`, which the shell quoting reads.
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

/// What:
///  The command that completes the checkout,
///  and the one that removes the worktree,
///  each already quoted for the shell.
/// Why:
///   A control runs the printed text,
///  so the words and the messages come from one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function finishAndRemove(destination: string): [string, string];
/// ```
pub fn finish_and_remove(destination: &Path) -> (String, String) {
    // The path is read by the person's shell, so it is quoted the way a shell reads a word back, not the way JSON
    // spells a string: `$bucket` and a backquote inside double quotes would be expanded by the shell that runs `rm`.
    let shown: String = crate::shell_quote::shell_quote(destination.as_os_str().as_bytes());
    let finish: String = format!("git -C {shown} reset {WORKTREE_ENFORCEMENT_ESCAPE_HATCH} --hard");
    let remove: String = format!("git worktree remove --force {shown}");
    return (finish, remove);
}

/// What:
///  The sentence both messages print after their own words.
/// Why:
///   It says what finishes the checkout,
///  what that discards,
///  why the guard has to be told,
///  and what removes
///  the worktree instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// finishOrRemoveSentence('/tmp/half');
/// // '`git -C /tmp/half reset --no-enforce-worktree --hard` completes its checkout, ...'
/// ```
pub fn finish_or_remove_sentence(destination: &Path) -> String {
    let (finish, remove): (String, String) = finish_and_remove(destination);
    return format!(
        "`{finish}` completes its checkout, discarding whatever that worktree holds (the wrapper's worktree guard \
         refuses a hard reset in a worktree that holds uncommitted changes, which a checkout that never ran reads as, \
         unless {WORKTREE_ENFORCEMENT_ESCAPE_HATCH} is given), and `{remove}` removes it."
    );
}

/// The words and the quoting of both commands, and what the sentence says they do.
#[cfg(test)]
#[path = "worktree_finish_commands_tests.rs"]
mod tests;
