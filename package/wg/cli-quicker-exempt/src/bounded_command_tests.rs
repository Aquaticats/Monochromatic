//! Verifies that a bounded command captures what it printed,
//! runs with a cleared environment and a fixed search path,
//! and is ended when it runs past its limit.

/// The command under test.
use crate::bounded_command::run_bounded;
/// The time limit.
use std::time::{Duration, Instant};

/// Builds arguments from words.
fn words(items: &[&str]) -> Vec<String> {
    return items.iter().map(|item| return (*item).to_string()).collect();
}

/// What a command printed and how it ended are returned.
#[test]
fn what_the_command_printed_and_how_it_ended_are_returned() {
    let output = run_bounded("sh", &words(&["-c", "echo out; echo err >&2; exit 3"]), Duration::from_secs(10)).expect("the command ends in time");
    assert!(!output.success);
    assert!(output.status.contains('3'), "{}", output.status);
    assert_eq!(output.stdout.trim(), "out");
    assert_eq!(output.stderr.trim(), "err");
    let fine = run_bounded("sh", &words(&["-c", "exit 0"]), Duration::from_secs(10)).expect("the command ends in time");
    assert!(fine.success);
}

/// The environment holds only the fixed search path.
#[test]
fn the_environment_is_cleared_and_the_search_path_is_fixed() {
    let output = run_bounded("sh", &words(&["-c", "echo \"home=[$HOME] path=[$PATH]\""]), Duration::from_secs(10)).expect("the command ends in time");
    assert_eq!(output.stdout.trim(), "home=[] path=[/usr/bin:/bin]");
}

/// A command that runs past its limit is ended and named.
#[test]
fn a_command_past_its_limit_is_ended() {
    let started = Instant::now();
    let error = run_bounded("sleep", &words(&["30"]), Duration::from_millis(200)).expect_err("the command is ended");
    assert!(error.contains("did not finish within"), "{error}");
    assert!(error.contains("sleep"), "{error}");
    assert!(started.elapsed() < Duration::from_secs(10), "{:?}", started.elapsed());
}

/// A program that does not exist is an error and not a hang.
#[test]
fn a_missing_program_is_an_error() {
    let error = run_bounded("no-such-program-for-the-test", &[], Duration::from_secs(5)).expect_err("the program is missing");
    assert!(!error.is_empty());
}
