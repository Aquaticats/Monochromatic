//! What:
//!  The linked-worktree policy's carve-out for the abort and skip forms
//!  (`git merge --abort`,
//!  `git rebase --abort`,
//!  `git cherry-pick --abort`,
//!  `git revert --abort`,
//!  `git am --abort` and the `--skip` forms of the same operations)
//!  while the operation they undo waits for its conflicts.
//! Why:
//!  The guard rejects a form that can discard uncommitted work while the worktree holds any,
//!  and an operation that stopped with conflicts makes the worktree hold its own state,
//!  so every abort form was refused for exactly as long as it was the command a person needs.
//!  The rule that protects the person is the owner's:
//!  a command that could destroy work which exists nowhere else is refused,
//!  and the person who means it runs the command with the guard's opt-out
//!  (`doc/handover/cli-git-rust-implementation.md`,
//!  the answers of 2026-11-12T10:35:07, "Refuse with opt-out").
//!  So this carve-out gives three answers and no copies:
//!  a form whose own operation is absent is Git's to refuse and discards nothing,
//!  a form whose only state is the operation's own runs as Git runs it,
//!  and a form that would discard the person's own work is refused,
//!  naming each path, what stands there, the quit form that keeps the work and the opt-out.
//!  The state is read per command,
//!  because an abort form can end another operation's state:
//!  a `git rebase --rebase-merges` merge step,
//!  or `git am --abort` during a rebase that applies patches.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdict = judgeAbortDiscard(facts, GuardedCommand.AbortMerge);
//! // 'left-to-git' | 'allowed' | { kind: 'refused', text }
//! ```

use crate::repository_facts::RepositoryFacts;
use crate::repository_facts_abort_discard::AbortDiscard;
use crate::rule_abort_messages::{discard_refusal_text, ignored_file_refusal_text, unreadable_refusal_text, untracked_file_text};
use crate::rule_abort_operation::{AbortOperation, OperationEnd, ended_operation};
use crate::rule_linked_worktree::GuardedCommand;
use crate::rule_linked_worktree_messages::ignored_file_message;

/// What the policy does with an abort or a skip.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   Three different answers need three different effects:
///  the usual rules,
///  Git's own refusal,
///  the command as Git runs it,
///  and a refusal that names what it would discard.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AbortVerdict {
    /// Not an abort or skip form,
    /// or the answer could not be read in a way this carve-out can act on:
    /// the usual rules decide.
    NotApplicable,
    /// No operation this command ends waits:
    /// Git refuses it and discards nothing,
    /// so nothing is measured.
    LeftToGit,
    /// The operation waits and its own state is all there is:
    /// the command runs exactly as Git runs it,
    /// and no line is added.
    Allowed,
    /// The command would discard work that stands nowhere else:
    /// it is refused with this text,
    /// and nothing was changed.
    Refused(String),
}

/// What:
///  The refusal of a command whose reading failed.
///  `operation` is the operation the command's word ends,
///  which is the one the sentence can name without a measurement.
/// Why:
///   A reading that failed cannot say the command is safe,
///  and the refusal has to name the reason rather than fall back to a sentence about a dirty worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = unreadableRefusalText(operation, ends, operation, message);
/// ```
fn unreadable(operation: AbortOperation, ends: OperationEnd, message: &str) -> AbortVerdict {
    return AbortVerdict::Refused(unreadable_refusal_text(operation, ends, operation, message));
}

/// What:
///  Judge an abort or a skip against the operation that waits.
///  `&mut dyn RepositoryFacts` lends any facts provider for writing.
/// Why:
///   The order is the owner's:
///  the command must be an abort or skip form,
///  the operation it ends must be the one waiting,
///  the person's own work must be found,
///  and only then is the command refused or let through.
///  An answer that cannot be read is a refusal,
///  which loses nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function judgeAbortDiscard(facts, command): AbortVerdict;
/// ```
pub fn judge_abort_discard(facts: &mut dyn RepositoryFacts, command: GuardedCommand) -> AbortVerdict {
    // Every other guarded form deletes or overwrites by itself,
    //  and this carve-out is only about the forms that end an operation.
    let Some((operation, ends)): Option<(AbortOperation, OperationEnd)> = ended_operation(command) else {
        return AbortVerdict::NotApplicable;
    };
    let discard: AbortDiscard = match facts.abort_discard(command) {
        Ok(answer) => answer,
        Err(message) => return unreadable(operation, ends, message.as_str()),
    };
    let AbortDiscard::Waiting { in_progress, ends: measured_ends, restored, baseline, worktree, index, staged } = discard else {
        return AbortVerdict::LeftToGit;
    };
    debug_assert!(measured_ends == ends, "the reading of what this form ends is the same one the guard is judging");
    // An ignored file at a path the restored tree tracks is where a person keeps their environment and their local
    //  secrets, and the command writes that tree back: it is refused naming the paths.
    let revision: Option<&std::ffi::OsStr> = Some(restored.as_os_str());
    match facts.ignored_files_the_tree_tracks(revision) {
        Ok(shared) if !shared.is_empty() => {
            return AbortVerdict::Refused(ignored_file_refusal_text(ignored_file_message(command, revision, shared.as_slice()), in_progress));
        }
        Ok(_) => {}
        Err(message) => return unreadable(operation, ends, message.as_str()),
    }
    // An abort's reset refuses an untracked file at a path the restored tree tracks
    //  (measured for every abort form with Git 2.56.0): nothing is lost, the command does not run,
    //  and the person is told that it is Git that refuses and which file stands in the way.
    //  A skip's reset writes that same tree over the worktree without a word (measured), so there the file is work
    //  the command would lose, and the wrapper is what refuses.
    match facts.untracked_files_the_tree_tracks(restored.as_os_str()) {
        Ok(untracked) if !untracked.is_empty() => {
            return AbortVerdict::Refused(untracked_file_text(operation, ends, in_progress, restored.to_string_lossy().as_ref(), untracked.as_slice()));
        }
        Ok(_) => {}
        Err(message) => return unreadable(operation, ends, message.as_str()),
    }
    if worktree.is_empty() && index.is_empty() {
        // Only the operation's own state is there, which is what the command is for.
        return AbortVerdict::Allowed;
    }
    return AbortVerdict::Refused(discard_refusal_text(operation, ends, in_progress, baseline, worktree.as_slice(), index.as_slice(), staged.as_slice()));
}

/// The verdict and its readings.
#[cfg(test)]
#[path = "abort_discard_guard_tests.rs"]
mod tests;
