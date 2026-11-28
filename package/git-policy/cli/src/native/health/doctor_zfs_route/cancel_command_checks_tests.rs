//! What:
//!  Controls of the cancel command's line:
//!  it answers a pass,
//!  and the sentence it answers carries the command's own usage line rather
//!  than a second copy written beside the command.
//! Why:
//!  the line exists so a person beside a blocked removal learns the way back
//!  exists,
//!  and a sentence that drifted from the command's grammar would send that
//!  person to an argument the command refuses
//!  (`AGENTS.md` rules PKG and TCV).
//!  The facts here are read from the command's own grammar,
//!  which is the copy the wiring run also reads,
//!  so nothing in this control is a sentence somebody typed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(checkCancelCommand(facts)).toMatchObject({ kind: 'pass' });
//! ```

use super::{
    CancelCommandFacts,
    check_cancel_command,
};
use crate::health::doctor_zfs_route::check_outcome::{
    Check,
    CheckOutcome,
    CheckReport,
};
use crate::worktree_clone_removal_cancel::cancel_arguments::{
    CANCEL_COMMAND_WORD,
    usage,
};

/// The facts a wiring run reads from the command's own grammar.
fn facts() -> CancelCommandFacts {
    let text: String = usage();
    let first_line: &str = text.lines().next().expect("the usage has a line");
    return CancelCommandFacts {
        word: String::from(CANCEL_COMMAND_WORD),
        usage_line: String::from(first_line),
    };
}

/// The command's line is a pass,
/// and it is never unknown because the only fact it reads is the built
/// command.
#[test]
fn the_cancel_command_line_is_a_pass() {
    let outcome: CheckOutcome = check_cancel_command(&facts());
    assert!(matches!(outcome, CheckOutcome::Pass { .. }), "{outcome:?}");
}

/// The line carries the usage line the command itself prints,
/// so the grammar a person reads here is the grammar the command parses.
#[test]
fn the_line_carries_the_commands_own_usage_line() {
    let text: String = usage();
    let first_line: String = String::from(text.lines().next().expect("the usage has a line"));
    let report: CheckReport =
        CheckReport::new(Check::CancelCommand, check_cancel_command(&facts()));
    let line: String = report.line();
    assert!(
        line.contains(first_line.as_str()),
        "{first_line} is in {line}"
    );
}

/// The line names the command's own word and its one argument,
/// which is what a person types.
#[test]
fn the_line_names_the_word_and_the_slot() {
    let report: CheckReport =
        CheckReport::new(Check::CancelCommand, check_cancel_command(&facts()));
    let line: String = report.line();
    assert!(line.contains(CANCEL_COMMAND_WORD), "{line}");
    assert!(line.contains("cancel-worktree-removal <slot>"), "{line}");
    assert!(
        line.contains("destroys nothing, forces no unmount, ends no holder"),
        "{line}"
    );
}
