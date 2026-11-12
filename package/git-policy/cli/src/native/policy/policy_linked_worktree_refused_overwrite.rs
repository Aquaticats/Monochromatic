//! What:
//!  Refusing every discarding form that would write over or delete a worktree entry
//!  that exists nowhere else,
//!  before Git runs.
//! Why:
//!  A forced tree write replaces an untracked file,
//!  turns a file into a directory,
//!  deletes the work inside a nested repository at a path it writes,
//!  and writes over an ignored file the target tree tracks,
//!  and none of that is uncommitted work of the outer repository,
//!  so the status the guard reads never names it.
//!  The owner's answer of 2026-11-12T10:35:07 is "Refuse with opt-out"
//!  (`doc/handover/cli-git-rust-implementation.md`, the section of about 10:33):
//!  a command that could destroy a person's data is refused and names what it would destroy,
//!  and the guard's `--no-enforce-worktree` runs it as real Git would,
//!  with no copy.
//!  A refusal is also the only answer that leaves the worktree exactly as it was when it
//!  prints, so the entries the person wanted to keep are still where they left them.
//!  The question is asked only for a form that writes a tree or a path of one,
//!  because a clean's own answer is Git's dry run
//!  and every other form reaches only paths its index already holds.
//!  A worktree that holds nothing at those paths costs two Git listings and no refusal,
//!  so a clean worktree that merely holds untracked files elsewhere runs as it always did.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const found = entriesAFormWouldWriteOver({ facts, command, region });
//! // found === ['b.txt', 'nested/']
//! ```

/// The builder of the written paths,
///  which decides whether this form writes anything at all.
use crate::command_written_paths::{WrittenPaths, written_paths};
/// The facts interface every check asks.
use crate::repository_facts::RepositoryFacts;
/// The guarded form being judged.
use crate::rule_linked_worktree::GuardedCommand;
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  Everything one question is asked with:
///  the facts,
///  the form and the tokens after it.
///  A `struct` is a record with named fields.
/// Why:
///   The three travel together at both places a form is let through,
///  and one record keeps the caller from passing them in a different order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WrittenOverRequest<'a> = { facts; command; region; shellDirectory };
/// ```
pub(crate) struct WrittenOverRequest<'a> {
    /// The facts every question is asked.
    pub(crate) facts: &'a mut dyn RepositoryFacts,
    /// The guarded form being judged.
    pub(crate) command: GuardedCommand,
    /// The tokens after the subcommand,
    ///  as the guard decided about them.
    pub(crate) region: &'a [OsString],
    /// The directory the shell of a `!` alias runs in,
    ///  when the form is the command that shell runs;
    ///  nothing for a direct command and an ordinary alias.
    pub(crate) shell_directory: Option<&'a Path>,
}

/// What:
///  Every worktree entry the form would write over or delete
///  and no repository holds,
///  each as Git printed it,
///  or why the question could not be answered.
/// Why:
///  An answer with no entry lets the command run exactly as it did before,
///  and a reading that failed must never be read as "nothing is in the way":
///  what the command would overwrite was not seen,
///  so the caller refuses instead of guessing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entriesAFormWouldWriteOver(request): string[]; // throws
/// ```
pub(crate) fn entries_a_form_would_write_over(request: WrittenOverRequest<'_>) -> Result<Vec<String>, String> {
    // A clean deletes what its own dry run names, and the nested repositories it deletes keep their own
    //  recorded answer.
    if matches!(
        request.command,
        GuardedCommand::Clean | GuardedCommand::CleanIgnored | GuardedCommand::CleanIgnoredOnly
    ) {
        return Ok(Vec::<String>::new());
    }
    let Some(written): Option<WrittenPaths> = written_paths(request.command, request.region, request.shell_directory) else {
        return Ok(Vec::<String>::new());
    };
    return request.facts.paths_in_the_way(&written);
}

/// The entries a form would overwrite, read through `written_paths`, and the places a rejection comes from.
#[cfg(test)]
#[path = "policy_linked_worktree_refused_overwrite_tests.rs"]
mod tests;
