//! What:
//!  The linked-worktree policy's carve-out for the abort forms
//!  (`git merge --abort`,
//!  `git rebase --abort`,
//!  `git cherry-pick --abort`,
//!  `git revert --abort`,
//!  `git am --abort`)
//!  while the operation they undo waits for its conflicts.
//! Why:
//!  The guard rejects a form that can discard uncommitted work while the worktree holds any,
//!  and an operation that stopped with conflicts makes the worktree hold its own state,
//!  so every abort form was refused for exactly as long as it was the command a person needs.
//!  The rule that protects the person is the owner's "save,
//!  then allow" (2026-11-02),
//!  which the conflict-side carve-out already follows:
//!  the operation's own state is what the abort exists to let go,
//!  the person's own work is copied beside the audit log first,
//!  and the notice names where each copy is.
//!  Where this command's own state is absent,
//!  Git refuses the command itself and discards nothing,
//!  so it is left to Git;
//!  the state is read per command,
//!  because an abort form can end another operation's state
//!  (a `git rebase --rebase-merges` merge step, or `git am --abort` during a rebase that applies patches).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdict = judgeAbortDiscard(facts, GuardedCommand.AbortMerge, identity, now(), limits);
//! // 'left-to-git' | { kind: 'allowed', notice, copies } | { kind: 'copy-failed', text }
//! ```

use crate::conflict_side_guard::prune_after_save;
use crate::conflict_sides::{SaveFailure, SavedCopy, save_copies};
use crate::discarded_prune::Limits;
use crate::repository_facts::RepositoryFacts;
use crate::repository_facts_abort_discard::AbortDiscard;
use crate::rule_abort_messages::{copy_failed_text, saved_before_abort_notice, staged_content_text};
use crate::rule_abort_operation::{AbortOperation, aborted_operation, operation_name};
use crate::rule_linked_worktree::GuardedCommand;
use crate::rule_linked_worktree_messages::ignored_file_message;
use crate::worktree_identity::WorktreeIdentity;

/// What the policy does with an abort form.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   Four different answers need four different effects:
///  the usual rules,
///  Git's own refusal,
///  a save and then the command,
///  and a rejection because the save failed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AbortVerdict {
    /// Not an abort form,
    /// no operation of its kind waits,
    /// or the answer could not be read:
    /// the usual rules decide.
    NotApplicable,
    /// No operation this command aborts waits:
    /// Git refuses it and discards nothing,
    /// so nothing is measured and nothing is saved.
    LeftToGit,
    /// The operation waits and the person's own work is on disk:
    /// the command runs,
    /// and the notice names the copies.
    Allowed {
        /// The notice to show the person before Git runs.
        notice: String,
        /// What was saved,
        /// for the audit record.
        copies: Vec<SavedCopy>,
    },
    /// A copy could not be written:
    /// the command is rejected with this text,
    /// because it would discard unsaved work.
    CopyFailed(String),
}

/// What:
///  The text of a rejected abort whose copy could not be written.
///  `&SaveFailure` borrows what the save step reported.
/// Why:
///   The rejection names the command,
///  the operation in progress,
///  the path or directory that failed and why,
///  and keeps the hatch as the way through for a person who means it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function copyFailed(operation: AbortOperation, inProgress: AbortOperation, failure: SaveFailure): string;
/// ```
fn copy_failed(operation: AbortOperation, in_progress: AbortOperation, failure: &SaveFailure) -> String {
    return copy_failed_text(operation, in_progress, failure.subject.as_str(), failure.reason.as_str());
}

/// What:
///  The worktree root,
///  the common directory and the identity of the repository the command runs in,
///  or nothing when the identity has no worktree (which the guard rejects before this point).
/// Why:
///   A copy is written below the common Git directory and read from the worktree,
///  so both are needed and both come from one identity.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function places(identity: WorktreeIdentity): { root: string; common: string } | undefined;
/// ```
fn places(identity: &WorktreeIdentity) -> Option<(&std::path::Path, &std::path::Path)> {
    return match identity {
        WorktreeIdentity::MainWorktree { common_dir, worktree_root, .. }
        | WorktreeIdentity::LinkedWorktree { common_dir, worktree_root, .. } => Some((worktree_root.as_path(), common_dir.as_path())),
        WorktreeIdentity::BareRepository { .. } | WorktreeIdentity::OutsideWorktree => None,
    };
}

/// What:
///  Judge an abort form against the operation that waits,
///  saving the person's own work when the command may run.
/// Why:
///   The order is the owner's:
///  the command must be an abort form,
///  the operation it aborts must be the one waiting,
///  the person's own work must be on disk,
///  and only then does the command run.
///  An answer that cannot be read keeps the usual rejection,
///  which loses nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function judgeAbortDiscard(facts, command, identity, milliseconds, limits): AbortVerdict;
/// ```
pub fn judge_abort_discard(
    facts: &mut dyn RepositoryFacts,
    command: GuardedCommand,
    identity: &WorktreeIdentity,
    milliseconds: i64,
    limits: Limits,
) -> AbortVerdict {
    // Every other guarded form deletes or overwrites by itself, and this carve-out is only about the abort forms.
    let Some(operation): Option<AbortOperation> = aborted_operation(command) else {
        return AbortVerdict::NotApplicable;
    };
    let discard: AbortDiscard = match facts.abort_discard(command) {
        Ok(answer) => answer,
        // The state files or the listings could not be read: the usual rejection stands, and nothing is saved or let go.
        Err(_message) => return AbortVerdict::NotApplicable,
    };
    let AbortDiscard::Waiting { in_progress, restored, index_only, paths } = discard else {
        return AbortVerdict::LeftToGit;
    };
    let Some((root, common)) = places(identity) else {
        return AbortVerdict::NotApplicable;
    };
    // An ignored file at a path the restored tree tracks is where a person keeps their environment and their local secrets,
    //  and the abort writes that tree back: it is rejected naming the paths, before anything is copied or changed.
    let revision: Option<&std::ffi::OsStr> = Some(restored.as_os_str());
    match facts.ignored_files_the_tree_tracks(revision) {
        Ok(shared) if !shared.is_empty() => {
            return AbortVerdict::CopyFailed(format!(
                "{} Nothing was saved and nothing changed: the {} in progress still waits.",
                ignored_file_message(command, revision, shared.as_slice()),
                operation_name(in_progress)
            ));
        }
        Ok(_) => {}
        Err(message) => {
            return AbortVerdict::CopyFailed(format!(
                "monochromatic-git-policy-cli: {message} The {} in progress still waits, and nothing was saved or changed.",
                operation_name(in_progress)
            ));
        }
    }
    // Content the worktree no longer holds and the index does is work this abort would discard,
    //  and the save step copies worktree files only: the command is rejected naming the paths.
    if !index_only.is_empty() {
        return AbortVerdict::CopyFailed(staged_content_text(operation, in_progress, index_only.as_slice()));
    }
    if paths.is_empty() {
        // Only the operation's own state is there, which is what the abort is for.
        return AbortVerdict::Allowed { notice: String::new(), copies: Vec::<SavedCopy>::new() };
    }
    match save_copies(root, common, paths.as_slice(), milliseconds) {
        Ok(copies) => {
            let unpruned: String = prune_after_save(common, copies.as_slice(), milliseconds, limits);
            return AbortVerdict::Allowed {
                notice: format!("{}{unpruned}", saved_before_abort_notice(operation, in_progress, copies.as_slice())),
                copies,
            };
        }
        Err(failure) => return AbortVerdict::CopyFailed(copy_failed(operation, in_progress, &failure)),
    }
}

/// The verdict and the copies for one abort form.
#[cfg(test)]
#[path = "abort_discard_guard_tests.rs"]
mod tests;
