//! Unit tests of the message for a real `mpm` that could not be started:
//! one test per cause,
//! plus the parts every message has.

use std::ffi::OsString;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::time::Duration;

use crate::constants::EXEC_FAILED_EXIT_CODE;
use crate::start_failure::{StartFailure, describe_start_failure};
use crate::start_retry::RetryReport;

const PROGRAM: &str = "/repo/.cache/mpm-shadow-bin/mpm";

fn words(items: &[&str]) -> Vec<OsString> {
    return items.iter().map(OsString::from).collect();
}

fn message(outer: &[&str], error: &Error, retry: RetryReport, depth: u32) -> String {
    let arguments: Vec<OsString> = words(outer);
    return describe_start_failure(&StartFailure { outer: &arguments, program: Path::new(PROGRAM), error, retry, depth });
}

fn no_retry() -> RetryReport {
    return RetryReport::default();
}

/// The parts every message has,
/// whatever the cause.
fn assert_has_the_common_parts(text: &str) {
    assert!(text.starts_with("mpm-shadow: "), "{text}");
    assert!(text.contains("mpm i -g"), "the command the person ran: {text}");
    assert!(text.contains(PROGRAM), "the program it tried: {text}");
    assert!(text.contains("starting the real `mpm`"), "the stage: {text}");
    assert!(text.contains(&EXEC_FAILED_EXIT_CODE.to_string()), "the exit status: {text}");
    assert!(text.contains("could not be started"), "why that status: {text}");
    assert!(text.contains("nothing") && text.contains("post-run cleanup did not run"), "what completed: {text}");
}

#[test]
fn a_refusal_for_lack_of_processes_says_so_in_plain_words_with_the_retries_and_both_kinds_of_remedy() {
    let retry = RetryReport { refusals: 7, paused: Duration::from_millis(4980) };
    let error = Error::from(ErrorKind::WouldBlock);
    let text = message(&["i", "-g"], &error, retry, 0);
    assert_has_the_common_parts(&text);
    assert!(text.contains("refused to create another process"), "{text}");
    assert!(text.contains("7 times") && text.contains("4980 ms"), "{text}");
    for remedy in ["ulimit -u", "TasksMax", "wait", "run the command again"] {
        assert!(text.contains(remedy), "{remedy}: {text}");
    }
}

#[test]
fn a_program_without_permission_names_the_execute_bit_and_the_directory_search_bit() {
    let text = message(&["i", "-g"], &Error::from(ErrorKind::PermissionDenied), no_retry(), 0);
    assert_has_the_common_parts(&text);
    assert!(text.contains("permission"), "{text}");
    assert!(text.contains("chmod"), "{text}");
}

#[test]
fn a_program_that_vanished_or_has_a_missing_interpreter_says_which_to_check() {
    let text = message(&["i", "-g"], &Error::from(ErrorKind::NotFound), no_retry(), 0);
    assert_has_the_common_parts(&text);
    assert!(text.contains("interpreter"), "{text}");
    assert!(text.contains("no longer exists") || text.contains("does not exist"), "{text}");
}

#[test]
fn any_other_cause_keeps_the_systems_words_and_points_at_running_the_program_by_hand() {
    let error = Error::new(ErrorKind::InvalidInput, "argument list too long");
    let text = message(&["i", "-g"], &error, no_retry(), 0);
    assert_has_the_common_parts(&text);
    assert!(text.contains("argument list too long"), "{text}");
    assert!(text.contains("by hand"), "{text}");
}

#[test]
fn a_shadow_started_by_another_shadow_says_how_deep_it_is_and_what_that_means() {
    let text = message(&["i", "-g"], &Error::from(ErrorKind::WouldBlock), no_retry(), 3);
    assert!(text.contains("started by another mpm-shadow"), "{text}");
    assert!(text.contains("level 3"), "{text}");
    let top = message(&["i", "-g"], &Error::from(ErrorKind::WouldBlock), no_retry(), 0);
    assert!(!top.contains("started by another mpm-shadow"), "{top}");
}

#[test]
fn the_command_line_is_quoted_so_an_argument_with_a_space_stays_one_argument_and_no_arguments_is_just_mpm() {
    let text = message(&["install", "a b"], &Error::from(ErrorKind::NotFound), no_retry(), 0);
    assert!(text.contains("mpm install 'a b'"), "{text}");
    let bare = message(&[], &Error::from(ErrorKind::NotFound), no_retry(), 0);
    assert!(bare.contains("`mpm`"), "{bare}");
}

#[test]
fn the_nesting_refusal_names_the_command_the_program_the_level_the_limit_the_cause_the_remedy_and_the_status() {
    use crate::nesting::MAX_NESTING;
    use crate::start_failure::describe_nesting_refusal;
    let arguments: Vec<OsString> = words(&["i", "-g"]);
    let text = describe_nesting_refusal(&arguments, Path::new(PROGRAM), MAX_NESTING);
    for expected in [
        "mpm i -g",
        PROGRAM,
        &format!("nesting level {MAX_NESTING}"),
        &format!("limit {MAX_NESTING}"),
        "started by another mpm-shadow",
        "command -v -a mpm",
        &EXEC_FAILED_EXIT_CODE.to_string(),
        "post-run cleanup did not run",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}
