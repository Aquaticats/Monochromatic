//! What:
//!  The abort forms' operations and the state that shows an operation waiting.
//! Why:
//!  The mappings are the whole decision:
//!  a form that names another operation's abort must be left to Git,
//!  and a state file read as the wrong operation would let the wrong command go.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(abortedOperation(GuardedCommand.AbortPick)).toBe('pick');
//! expect(waitingOperation(state({ rebaseApplyApplying: true }))).toBe('am');
//! ```

use super::{
    AbortOperation, OperationState, aborted_operation, aborts_the_waiting_operation, operation_name, restored_revision,
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

/// Each state names the operation Git's own abort reads.
#[test]
fn each_state_names_the_operation_that_writes_it() {
    let mut state: OperationState = nothing();
    assert_eq!(super::waiting_operation(state), None);
    state.merge_head = true;
    assert_eq!(super::waiting_operation(state), Some(AbortOperation::Merge));
    state = nothing();
    state.rebase_merge = true;
    assert_eq!(super::waiting_operation(state), Some(AbortOperation::Rebase));
    state = nothing();
    state.rebase_apply_rebasing = true;
    assert_eq!(super::waiting_operation(state), Some(AbortOperation::Rebase));
    state = nothing();
    state.cherry_pick_head = true;
    assert_eq!(super::waiting_operation(state), Some(AbortOperation::Pick));
    state = nothing();
    state.revert_head = true;
    assert_eq!(super::waiting_operation(state), Some(AbortOperation::Pick));
    state = nothing();
    state.sequencer = true;
    assert_eq!(super::waiting_operation(state), Some(AbortOperation::Pick));
    state = nothing();
    state.rebase_apply_applying = true;
    assert_eq!(super::waiting_operation(state), Some(AbortOperation::Am));
}

/// An abort that names another operation's state does not match the one waiting.
#[test]
fn an_abort_matches_only_its_own_operation() {
    for (aborted, waiting, matches) in [
        (AbortOperation::Merge, AbortOperation::Merge, true),
        (AbortOperation::Rebase, AbortOperation::Rebase, true),
        (AbortOperation::Pick, AbortOperation::Pick, true),
        (AbortOperation::Am, AbortOperation::Am, true),
        (AbortOperation::Merge, AbortOperation::Rebase, false),
        (AbortOperation::Pick, AbortOperation::Merge, false),
        (AbortOperation::Am, AbortOperation::Rebase, false),
        (AbortOperation::Rebase, AbortOperation::Am, false),
    ] {
        assert_eq!(aborts_the_waiting_operation(aborted, waiting), matches, "{aborted:?} against {waiting:?}");
    }
}

/// The revision an abort writes back is `ORIG_HEAD` for every abort form,
///  which Git records before each of them stops.
#[test]
fn the_restored_revision_is_named_for_each_operation() {
    for operation in [AbortOperation::Merge, AbortOperation::Rebase, AbortOperation::Pick, AbortOperation::Am] {
        assert_eq!(restored_revision(operation), OsStr::new("ORIG_HEAD"), "{operation:?}");
    }
}

/// The state query asks for one path per operation state,
///  in the order the presence facts read.
#[test]
fn the_state_query_names_every_state_entry() {
    let asked: Vec<String> = super::OPERATION_STATE_NAMES.iter().map(|name| { return (*name).to_string(); }).collect();
    assert_eq!(
        asked,
        vec!["MERGE_HEAD", "CHERRY_PICK_HEAD", "REVERT_HEAD", "rebase-merge", "rebase-apply/rebasing", "rebase-apply/applying"]
    );
    assert_eq!(super::SEQUENCER_DIRECTORY, "sequencer");
}
