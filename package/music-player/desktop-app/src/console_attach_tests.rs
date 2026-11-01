// What:     Unit tests for `console_attach.rs`, pulled in by `#[cfg(test)] #[path = "console_attach_tests.rs"] mod tests;`.
// Why:      Cover every branch of the two decision functions and the log function, because the Win32 shell that uses them
//           only compiles on Windows.

use super::*;

// A passed-on handle value, as `GetStdHandle` returns it.
const PASSED_ON: isize = 0x1c4;
// A different handle value, as a console attach could leave behind.
const REPLACED: isize = 0x2d8;

#[test]
fn attach_is_skipped_when_both_output_handles_were_passed_on() {
    assert!(!needs_console_attach(Some(PASSED_ON), Some(REPLACED)));
}

#[test]
fn attach_is_tried_when_stdout_is_missing() {
    assert!(needs_console_attach(None, Some(PASSED_ON)));
}

#[test]
fn attach_is_tried_when_stderr_is_missing() {
    assert!(needs_console_attach(Some(PASSED_ON), None));
}

#[test]
fn attach_is_tried_when_both_are_missing() {
    assert!(needs_console_attach(None, None));
}

#[test]
fn a_passed_on_handle_the_attach_left_alone_is_kept() {
    assert_eq!(std_handle_action_after_attach(Some(PASSED_ON), Some(PASSED_ON)), StdHandleAction::Keep);
}

#[test]
fn a_passed_on_handle_the_attach_replaced_is_restored() {
    assert_eq!(std_handle_action_after_attach(Some(PASSED_ON), Some(REPLACED)), StdHandleAction::Restore { original: PASSED_ON });
}

#[test]
fn a_passed_on_handle_the_attach_cleared_is_restored() {
    assert_eq!(std_handle_action_after_attach(Some(PASSED_ON), None), StdHandleAction::Restore { original: PASSED_ON });
}

#[test]
fn a_missing_handle_the_attach_supplied_is_kept() {
    assert_eq!(std_handle_action_after_attach(None, Some(REPLACED)), StdHandleAction::Keep);
}

#[test]
fn a_missing_handle_the_attach_did_not_supply_opens_the_console_output() {
    assert_eq!(std_handle_action_after_attach(None, None), StdHandleAction::OpenConsoleOutput);
}

#[test]
fn every_outcome_can_be_logged_without_a_subscriber() {
    let report = StreamReport { action: StdHandleAction::Keep, failure_code: None };
    log_console_attach(&ConsoleAttachOutcome::HandlesPassedOn);
    log_console_attach(&ConsoleAttachOutcome::NoParentConsole { error_code: 6 });
    log_console_attach(&ConsoleAttachOutcome::Attached { stdout: report, stderr: report });
}
