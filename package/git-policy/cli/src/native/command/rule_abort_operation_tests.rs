//! What:
//!  The abort forms' operations, the state that makes each abort live,
//!  and the record each form restores its tree from.
//! Why:
//!  The mappings are the whole decision:
//!  a form whose own state is absent is Git's to refuse,
//!  and a state read as the wrong operation's,
//!  or a restored revision named from the wrong record,
//!  would let a command go with the person's work left unsaved.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(abortedOperation(GuardedCommand.AbortPick)).toBe('pick');
//! expect(abortIsLive('rebase', state({ mergeHead: true, rebaseMerge: true }))).toBe(true);
//! expect(restoredRevision('rebase', state({ rebaseMerge: true }))).toEqual({ stateFile: 'rebase-merge/orig-head' });
//! ```

use super::{
    AbortOperation, OperationEnd, OperationState, RESTORED_REVISION_NAMES, RestoredRevision, SKIP_RESTORES, aborted_operation, abort_is_live,
    ended_operation, in_progress_operation, operation_name, restored_revision,
};
use crate::rule_linked_worktree::GuardedCommand;
use std::ffi::OsStr;

/// A state with nothing in progress.
fn nothing() -> OperationState {
    return OperationState {
        merge_head: false,
        cherry_pick_head: false,
        revert_head: false,
        rebase_merge: false,
        rebase_apply_rebasing: false,
        rebase_apply_applying: false,
        sequencer: false,
    };
}

/// A conflicted merge: `MERGE_HEAD` alone.
fn merge_conflict() -> OperationState {
    let mut state: OperationState = nothing();
    state.merge_head = true;
    return state;
}

/// A merge step of `git rebase --rebase-merges`: `MERGE_HEAD` beside `rebase-merge`.
fn rebase_merges_merge_step() -> OperationState {
    let mut state: OperationState = merge_conflict();
    state.rebase_merge = true;
    return state;
}

/// A rebase that stopped with conflicts, with the merge backend.
fn rebase_merge_stop() -> OperationState {
    let mut state: OperationState = nothing();
    state.rebase_merge = true;
    return state;
}

/// A rebase that stopped with conflicts, with the apply backend.
fn rebase_apply_stop() -> OperationState {
    let mut state: OperationState = nothing();
    state.rebase_apply_rebasing = true;
    return state;
}

/// An `am` that stopped with conflicts.
fn am_stop() -> OperationState {
    let mut state: OperationState = nothing();
    state.rebase_apply_applying = true;
    return state;
}

/// A single cherry-pick that stopped with conflicts.
fn pick_single() -> OperationState {
    let mut state: OperationState = nothing();
    state.cherry_pick_head = true;
    return state;
}

/// A single revert that stopped with conflicts.
fn revert_single() -> OperationState {
    let mut state: OperationState = nothing();
    state.revert_head = true;
    return state;
}

/// A cherry-pick or revert sequence that stopped, between or at its picks.
fn pick_sequence() -> OperationState {
    let mut state: OperationState = nothing();
    state.sequencer = true;
    return state;
}

/// Every abort form names its operation,
///  and no other guarded form names one.
#[test]
fn only_the_abort_forms_name_an_operation() {
    for command in GuardedCommand::ALL {
        let named: Option<AbortOperation> = aborted_operation(command);
        let expected: Option<AbortOperation> = match command {
            GuardedCommand::AbortMerge => Some(AbortOperation::Merge),
            GuardedCommand::AbortRebase => Some(AbortOperation::Rebase),
            GuardedCommand::AbortPick => Some(AbortOperation::Pick),
            GuardedCommand::AbortAm => Some(AbortOperation::Am),
            _ => None,
        };
        assert_eq!(named, expected, "{command:?}");
    }
}

/// The skip forms end the same operations their aborts do,
///  and say that they go on instead of restoring the state before.
#[test]
fn the_skip_forms_end_the_operation_their_abort_does() {
    for (command, expected) in [
        (GuardedCommand::SkipRebase, (AbortOperation::Rebase, OperationEnd::Skip)),
        (GuardedCommand::SkipPick, (AbortOperation::Pick, OperationEnd::Skip)),
        (GuardedCommand::SkipAm, (AbortOperation::Am, OperationEnd::Skip)),
    ] {
        assert_eq!(ended_operation(command), Some(expected), "{command:?}");
        // A skip is not an abort, so the abort reading names no operation for it.
        assert_eq!(aborted_operation(command), None, "{command:?}");
    }
    // A form that ends no operation ends nothing, whatever it is.
    assert_eq!(ended_operation(GuardedCommand::Clean), None);
    assert_eq!(ended_operation(GuardedCommand::CheckoutPaths), None);
}

/// A skip writes back the tree it stopped at, which is `HEAD`,
///  measured against the operation's recorded original head.
#[test]
fn a_skip_restores_head() {
    assert_eq!(SKIP_RESTORES, "HEAD");
    // The abort of the same operation reads its own record, so the two are not the same question.
    assert_eq!(
        restored_revision(AbortOperation::Rebase, rebase_merge_stop()),
        RestoredRevision::StateFile(OsStr::new("rebase-merge/orig-head"))
    );
}

/// Every operation has a name for a sentence,
///  and the four names are distinct.
#[test]
fn every_operation_has_its_own_name() {
    let mut names: Vec<&str> = Vec::new();
    for operation in [AbortOperation::Merge, AbortOperation::Rebase, AbortOperation::Pick, AbortOperation::Am] {
        let name: &str = operation_name(operation);
        assert!(!name.is_empty(), "{operation:?}");
        assert!(!names.contains(&name), "{operation:?} repeats {name}");
        names.push(name);
    }
    assert_eq!(names.len(), 4);
}

/// Each abort form is live exactly while the state its own command reads is there,
///  and another operation's state never makes it live.
#[test]
fn each_abort_form_is_live_only_with_its_own_state() {
    // What each form's own command reads, as real Git 2.56.0 answered.
    for (operation, state) in [
        (AbortOperation::Merge, merge_conflict()),
        (AbortOperation::Merge, rebase_merges_merge_step()),
        (AbortOperation::Rebase, rebase_merge_stop()),
        (AbortOperation::Rebase, rebase_apply_stop()),
        (AbortOperation::Rebase, rebase_merges_merge_step()),
        (AbortOperation::Pick, pick_single()),
        (AbortOperation::Pick, revert_single()),
        (AbortOperation::Pick, pick_sequence()),
        (AbortOperation::Am, am_stop()),
        // `git am --abort` ends a rebase that applies patches (measured).
        (AbortOperation::Am, rebase_apply_stop()),
    ] {
        assert!(abort_is_live(operation, state), "{operation:?} must be live in {state:?}");
    }
    // The states that only neighbour a form's own state.
    for (operation, state) in [
        // A merge step of a rebase is not a merge to `git rebase --abort`; the rebase state is what makes it live.
        (AbortOperation::Rebase, merge_conflict()),
        // An `am` in progress is refused by `git rebase --abort` (`It looks like 'git am' is in progress.`).
        (AbortOperation::Rebase, am_stop()),
        (AbortOperation::Merge, rebase_merge_stop()),
        (AbortOperation::Merge, pick_sequence()),
        (AbortOperation::Pick, merge_conflict()),
        (AbortOperation::Pick, rebase_apply_stop()),
        // A rebase with the merge backend leaves no `rebase-apply`, and `git am --abort` refuses it (measured).
        (AbortOperation::Am, rebase_merge_stop()),
        (AbortOperation::Am, merge_conflict()),
    ] {
        assert!(!abort_is_live(operation, state), "{operation:?} must not be live in {state:?}");
    }
    for operation in [AbortOperation::Merge, AbortOperation::Rebase, AbortOperation::Pick, AbortOperation::Am] {
        assert!(!abort_is_live(operation, nothing()), "{operation:?} with nothing in progress");
    }
}

/// A rebase that applies patches is a rebase to `git am --abort`,
///  which ends it; a rebase to `git rebase --abort` stays a rebase.
#[test]
fn an_am_abort_that_ends_a_rebase_names_the_rebase() {
    assert_eq!(in_progress_operation(AbortOperation::Am, am_stop()), AbortOperation::Am);
    assert_eq!(in_progress_operation(AbortOperation::Am, rebase_apply_stop()), AbortOperation::Rebase);
    assert_eq!(in_progress_operation(AbortOperation::Rebase, rebase_apply_stop()), AbortOperation::Rebase);
    assert_eq!(in_progress_operation(AbortOperation::Merge, am_stop()), AbortOperation::Merge);
}

/// Each abort form restores the commit its own implementation reads,
///  measured with Git 2.56.0 by moving `ORIG_HEAD` and each record apart.
#[test]
fn each_abort_form_names_the_record_it_restores() {
    for (operation, state, expected) in [
        // `git merge --abort` resets to `HEAD`, whatever `ORIG_HEAD` says.
        (AbortOperation::Merge, merge_conflict(), RestoredRevision::Named(OsStr::new("HEAD"))),
        (AbortOperation::Merge, rebase_merges_merge_step(), RestoredRevision::Named(OsStr::new("HEAD"))),
        // Each rebase backend records its original head in its own directory.
        (AbortOperation::Rebase, rebase_merge_stop(), RestoredRevision::StateFile(OsStr::new("rebase-merge/orig-head"))),
        (AbortOperation::Rebase, rebase_apply_stop(), RestoredRevision::StateFile(OsStr::new("rebase-apply/orig-head"))),
        // One pick never moved `HEAD`; a sequence recorded the start it resets to.
        (AbortOperation::Pick, pick_single(), RestoredRevision::Named(OsStr::new("HEAD"))),
        (AbortOperation::Pick, revert_single(), RestoredRevision::Named(OsStr::new("HEAD"))),
        (AbortOperation::Pick, pick_sequence(), RestoredRevision::StateFile(OsStr::new("sequencer/head"))),
        // The one form that reads `ORIG_HEAD`, which it also reads during a rebase it ends.
        (AbortOperation::Am, am_stop(), RestoredRevision::Named(OsStr::new("ORIG_HEAD"))),
        (AbortOperation::Am, rebase_apply_stop(), RestoredRevision::Named(OsStr::new("ORIG_HEAD"))),
    ] {
        assert_eq!(restored_revision(operation, state), expected, "{operation:?} in {state:?}");
    }
}

/// Every record a form restores from is one the state query asks Git for,
///  so reading it never needs a second start.
#[test]
fn every_record_a_form_restores_is_asked_for() {
    for (operation, state) in [
        (AbortOperation::Merge, merge_conflict()),
        (AbortOperation::Rebase, rebase_merge_stop()),
        (AbortOperation::Rebase, rebase_apply_stop()),
        (AbortOperation::Pick, pick_single()),
        (AbortOperation::Pick, pick_sequence()),
        (AbortOperation::Am, am_stop()),
    ] {
        if let RestoredRevision::StateFile(record) = restored_revision(operation, state) {
            assert!(
                RESTORED_REVISION_NAMES.contains(&record.to_str().expect("a record of text")),
                "{record:?} is not asked for"
            );
        }
    }
}

/// The state query asks for one path per operation state,
///  in the order the presence facts read,
///  and one for every record a form restores.
#[test]
fn the_state_query_names_every_state_entry() {
    let asked: Vec<String> = super::OPERATION_STATE_NAMES.iter().map(|name| { return (*name).to_string(); }).collect();
    assert_eq!(
        asked,
        vec!["MERGE_HEAD", "CHERRY_PICK_HEAD", "REVERT_HEAD", "rebase-merge", "rebase-apply/rebasing", "rebase-apply/applying"]
    );
    assert_eq!(super::SEQUENCER_DIRECTORY, "sequencer");
    assert_eq!(
        RESTORED_REVISION_NAMES.to_vec(),
        vec!["rebase-merge/orig-head", "rebase-apply/orig-head", "sequencer/head"]
    );
}
