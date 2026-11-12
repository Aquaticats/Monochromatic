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
    ///  `git rebase` waits,
    ///  whichever backend started it.
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
///  Whether the state of the operation `operation` aborts is present,
///  which is what lets its own abort run.
/// Why:
///   Each abort form is decided by the state its own command reads,
///  not by one operation named for the whole Git directory:
///  a `git rebase --rebase-merges` merge step writes `MERGE_HEAD` beside `rebase-merge`,
///  and its rebase is still abandoned by `git rebase --abort`;
///  an apply-backend rebase writes `rebase-apply/rebasing`,
///  and `git am --abort` abandons it as well.
///  A form whose own state is absent is Git's to refuse and discards nothing.
///  All of these are measured with Git 2.56.0.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// abortIsLive('rebase', state({ mergeHead: true, rebaseMerge: true })) === true;
/// abortIsLive('am', state({ rebaseApplyApplying: true })) === true;
/// abortIsLive('pick', state({ mergeHead: true })) === false;
/// ```
pub fn abort_is_live(operation: AbortOperation, state: OperationState) -> bool {
    return match operation {
        AbortOperation::Merge => state.merge_head,
        // `git rebase --abort` refuses an `am` (`It looks like 'git am' is in progress. Cannot rebase.`).
        AbortOperation::Rebase => state.rebase_merge || state.rebase_apply_rebasing,
        AbortOperation::Pick => state.cherry_pick_head || state.revert_head || state.sequencer,
        // `git am --abort` ends whichever operation the apply backend runs, a rebase that applies patches included.
        AbortOperation::Am => state.rebase_apply_rebasing || state.rebase_apply_applying,
    };
}

/// What:
///  The operation a form actually ends,
///  named in a sentence about it.
/// Why:
///   `git am --abort` during an apply-backend rebase ends that rebase,
///  and the notice and refusal must say "before the rebase",
///  never "before the am".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// inProgressOperation('am', state({ rebaseApplyRebasing: true })) === 'rebase';
/// ```
pub fn in_progress_operation(operation: AbortOperation, state: OperationState) -> AbortOperation {
    return match operation {
        AbortOperation::Am if state.rebase_apply_rebasing => AbortOperation::Rebase,
        other => other,
    };
}

/// The path names inside the Git directory the question reads.
pub const OPERATION_STATE_NAMES: [&str; 6] = ["MERGE_HEAD", "CHERRY_PICK_HEAD", "REVERT_HEAD", "rebase-merge", "rebase-apply/rebasing", "rebase-apply/applying"];

/// The name of the sequencer directory,
/// which holds the picks a cherry-pick or revert sequence still owes.
pub const SEQUENCER_DIRECTORY: &str = "sequencer";

/// What:
///  The files Git writes the commit an abort restores into,
///  named relative to the worktree's Git directory.
/// Why:
///   `git rebase --abort` resets to the commit in `rebase-merge/orig-head`
///  or `rebase-apply/orig-head`,
///  and `git cherry-pick --abort` and `git revert --abort` reset to
///  `sequencer/head` for a sequence.
///  `git merge --abort` reads `HEAD` and `git am --abort` reads `ORIG_HEAD`,
///  which are names Git resolves itself and not files of this list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// RESTORED_REVISION_NAMES.includes('rebase-merge/orig-head');
/// ```
pub const RESTORED_REVISION_NAMES: [&str; 3] = ["rebase-merge/orig-head", "rebase-apply/orig-head", "sequencer/head"];

/// What:
///  Where the commit an abort writes back into the worktree is recorded.
/// Why:
///   The ignored-file guard asks which paths of that commit's tree the worktree holds as
///  ignored files,
///  and only `HEAD` and `ORIG_HEAD` are names Git resolves itself;
///  a rebase and a pick sequence record the commit in a file inside the Git directory,
///  which has to be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RestoredRevision = { named: 'HEAD' | 'ORIG_HEAD' } | { stateFile: 'sequencer/head' };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RestoredRevision {
    /// A revision Git resolves itself.
    Named(&'static OsStr),
    /// A commit Git wrote into this file of the Git directory,
    ///  named relative to it.
    StateFile(&'static OsStr),
}

/// What:
///  The commit whose tree an abort writes back into the worktree,
///  as the command's own implementation finds it.
/// Why:
///   Every form answers this from a different record,
///  and none of them reads `ORIG_HEAD` except `am`:
///  `merge --abort` resets to `HEAD`;
///  `rebase --abort` reads the original head of its backend;
///  `cherry-pick --abort` and `revert --abort` read `HEAD` for one pick
///  and `sequencer/head` for a sequence;
///  `am --abort` reads `ORIG_HEAD`.
///  Every one is measured with Git 2.56.0,
///  with a moved `ORIG_HEAD` and with each record moved separately.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// restoredRevision('rebase', state({ rebaseMerge: true })) === stateFile('rebase-merge/orig-head');
/// restoredRevision('pick', state({ sequencer: true })) === stateFile('sequencer/head');
/// restoredRevision('am', state({ rebaseApplyApplying: true })) === named('ORIG_HEAD');
/// ```
pub fn restored_revision(operation: AbortOperation, state: OperationState) -> RestoredRevision {
    return match operation {
        // `git merge --abort` is a merge-semantics reset to the current `HEAD`, and a moved `ORIG_HEAD` is ignored (measured).
        AbortOperation::Merge => RestoredRevision::Named(OsStr::new("HEAD")),
        // Whichever backend started the rebase records its original head in its own directory.
        AbortOperation::Rebase if state.rebase_merge => RestoredRevision::StateFile(OsStr::new("rebase-merge/orig-head")),
        AbortOperation::Rebase => RestoredRevision::StateFile(OsStr::new("rebase-apply/orig-head")),
        // A sequence resets to its recorded start; one pick resets to the `HEAD` it never moved.
        AbortOperation::Pick if state.sequencer => RestoredRevision::StateFile(OsStr::new("sequencer/head")),
        AbortOperation::Pick => RestoredRevision::Named(OsStr::new("HEAD")),
        // `git am --abort` resets to `ORIG_HEAD`, which `git am` writes when it starts (measured, also during a rebase it ends).
        AbortOperation::Am => RestoredRevision::Named(OsStr::new("ORIG_HEAD")),
    };
}

/// The operations,
/// the state facts and the names.
#[cfg(test)]
#[path = "rule_abort_operation_tests.rs"]
mod tests;
