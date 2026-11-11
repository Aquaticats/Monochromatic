//! What:
//!  The pure core of "which operation does this abort form undo,
//!  and does that operation wait right now?".
//! Why:
//!  `git merge --abort` was refused by the discarding-command guard because the
//!  merge it undoes has made the worktree unclean by itself.
//!  Whether it may run turns on one fact the guard has to ask for anyway:
//!  the operation's own state in the worktree's Git directory.
//!  This file reads that answer and nothing else,
//!  so the decision can be checked without a repository.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! abortedOperation(GuardedCommand.AbortRebase) === 'rebase';
//! waitingOperation({ mergeHead: true, /* ... */ }) === 'merge';
//! ```

use crate::rule_linked_worktree::GuardedCommand;
/// `OsStr` is borrowed operating-system text of raw bytes.
use std::ffi::OsStr;

/// What:
///  An operation whose abort form settles what it left behind.
/// Why:
///   Each operation writes its own state and reads it again in its abort,
///  and `git cherry-pick --abort` and `git revert --abort` are one abort:
///  both stop the sequencer and return to the state before it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AbortOperation = 'merge' | 'rebase' | 'pick' | 'am';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbortOperation {
    /// `git merge --abort`.
    Merge,
    /// `git rebase --abort`,
    /// either backend.
    Rebase,
    /// `git cherry-pick --abort` or `git revert --abort`.
    Pick,
    /// `git am --abort`.
    Am,
}

/// The name of an operation in a sentence about it.
pub fn operation_name(operation: AbortOperation) -> &'static str {
    return match operation {
        AbortOperation::Merge => "merge",
        AbortOperation::Rebase => "rebase",
        AbortOperation::Pick => "cherry-pick or revert",
        AbortOperation::Am => "am",
    };
}

/// What:
///  The operation a guarded form aborts,
///  or nothing for every form that aborts none.
/// Why:
///   The guard decides on the form,
///  and only the five abort forms carry an operation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function abortedOperation(command: GuardedCommand): AbortOperation | undefined;
/// ```
pub fn aborted_operation(command: GuardedCommand) -> Option<AbortOperation> {
    return match command {
        GuardedCommand::AbortMerge => Some(AbortOperation::Merge),
        GuardedCommand::AbortRebase => Some(AbortOperation::Rebase),
        GuardedCommand::AbortPick => Some(AbortOperation::Pick),
        GuardedCommand::AbortAm => Some(AbortOperation::Am),
        // Every other guarded form deletes or overwrites by itself; none of them undoes an operation.
        _ => None,
    };
}

/// What:
///  Which of the state files and directories an operation keeps while it waits.
///  `bool` is true or false.
/// Why:
///   These are the only facts the question needs,
///  and reading them in process costs no Git start
///  when the layout reader can answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OperationState = { mergeHead: boolean; cherryPickHead: boolean; revertHead: boolean;
///   rebaseMerge: boolean; rebaseApplyRebasing: boolean; rebaseApplyApplying: boolean; sequencer: boolean };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OperationState {
    /// `MERGE_HEAD` exists:
    ///  `git merge` waits for its concluding commit.
    pub merge_head: bool,
    /// `CHERRY_PICK_HEAD` exists.
    pub cherry_pick_head: bool,
    /// `REVERT_HEAD` exists.
    pub revert_head: bool,
    /// The `rebase-merge` directory exists:
    ///  `git rebase` waits, whichever backend started it.
    pub rebase_merge: bool,
    /// `rebase-apply/rebasing` exists:
    ///  a `git rebase` with the apply backend waits.
    pub rebase_apply_rebasing: bool,
    /// `rebase-apply/applying` exists:
    ///  `git am` waits.
    pub rebase_apply_applying: bool,
    /// The `sequencer` directory exists:
    ///  a cherry-pick or revert sequence waits,
    ///  also between two picks.
    pub sequencer: bool,
}

/// What:
///  Which operation waits in the worktree the facts describe,
///  if any.
/// Why:
///   The abort form a person runs must match the operation that waits,
///  because Git's abort writes one operation's state and refuses another's:
///  leaving the command to Git where the two do not match
///  loses nothing and keeps the guard out of Git's own message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function waitingOperation(state: OperationState): AbortOperation | undefined;
/// ```
pub fn waiting_operation(state: OperationState) -> Option<AbortOperation> {
    if state.merge_head {
        return Some(AbortOperation::Merge);
    }
    if state.rebase_merge || state.rebase_apply_rebasing {
        return Some(AbortOperation::Rebase);
    }
    if state.cherry_pick_head || state.revert_head || state.sequencer {
        return Some(AbortOperation::Pick);
    }
    if state.rebase_apply_applying {
        return Some(AbortOperation::Am);
    }
    return None;
}

/// What:
///  Whether the operation `command` aborts is the one waiting,
///  so the command can run once the person's own work is saved.
/// Why:
///   A merge is in progress and the person runs `git cherry-pick --abort`:
///  Git refuses that,
///  and the guard must not save anything or let anything run on the strength of the merge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// resolvedAbort('rebase', 'merge') === false;
/// ```
pub fn aborts_the_waiting_operation(operation: AbortOperation, waiting: AbortOperation) -> bool {
    return operation == waiting;
}

/// The path names inside the Git directory the question reads.
pub const OPERATION_STATE_NAMES: [&str; 6] = ["MERGE_HEAD", "CHERRY_PICK_HEAD", "REVERT_HEAD", "rebase-merge", "rebase-apply/rebasing", "rebase-apply/applying"];

/// The name of the sequencer directory,
/// which holds the picks a cherry-pick or revert sequence still owes.
pub const SEQUENCER_DIRECTORY: &str = "sequencer";

/// What:
///  The revision whose tree an abort writes back into the worktree.
/// Why:
///   The ignored-file guard asks which paths of a tree the worktree holds as ignored files;
///  every abort form restores the state that stood before the operation,
///  which Git records as `ORIG_HEAD` before it stops
///  (measured with Git 2.56.0 on a conflicted single pick,
///  where `ORIG_HEAD` held the commit `HEAD` was at before the pick).
///  A repository whose `ORIG_HEAD` cannot be resolved falls back to `HEAD`,
///  which is the commit the operation started at until it moves `HEAD`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// restoredRevision('merge') === 'ORIG_HEAD';
/// ```
pub fn restored_revision(operation: AbortOperation) -> &'static OsStr {
    return match operation {
        AbortOperation::Merge | AbortOperation::Rebase | AbortOperation::Pick | AbortOperation::Am => OsStr::new("ORIG_HEAD"),
    };
}

/// The operations, the state facts and the names.
#[cfg(test)]
#[path = "rule_abort_operation_tests.rs"]
mod tests;
