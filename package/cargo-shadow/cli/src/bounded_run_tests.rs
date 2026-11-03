//! The tests of the bounded child runs: no child, and no reader of its output, is waited for without a bound.

use std::process::Command;
use std::time::{Duration, Instant};

use crate::bounded_run::{Bound, RunError, output_within};

/// A bound with words the tests look for.
fn bound(limit: Duration) -> Bound {
    return Bound { limit, what: "the slow command".to_string(), rerun: "slow-command --again".to_string() };
}

/// A child that outlives its bound is stopped and the error names it and the command to run again.
#[test]
fn a_child_that_outlives_its_bound_is_stopped_and_named() {
    let started = Instant::now();

    let result = output_within(Command::new("sleep").arg("5"), None, &bound(Duration::from_millis(200)));

    assert!(started.elapsed() < Duration::from_secs(3), "the child was waited for: {:?}", started.elapsed());
    let message: String = result.expect_err("a timeout").to_string();
    assert!(message.contains("the slow command") && message.contains("slow-command --again"), "{message}");
}

/// A child that finishes returns its output and status.
#[test]
fn a_child_that_finishes_returns_its_output() {
    let output = output_within(Command::new("sh").args(["-c", "echo out; echo err >&2; exit 3"]), None, &bound(Duration::from_secs(10))).expect("runs");

    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "out");
    assert_eq!(String::from_utf8_lossy(&output.stderr).trim(), "err");
    assert_eq!(output.status.code(), Some(3));
}

/// A grandchild that keeps the pipes open after the child exited does not hold the caller.
#[test]
fn a_grandchild_that_keeps_the_pipes_open_does_not_hold_the_caller() {
    let started = Instant::now();

    let output = output_within(Command::new("sh").args(["-c", "sleep 5 & echo done"]), None, &bound(Duration::from_secs(10))).expect("runs");

    assert!(started.elapsed() < Duration::from_secs(3), "{:?}", started.elapsed());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "done");
}

/// Output larger than a pipe buffer does not deadlock, and input larger than a pipe buffer is delivered.
#[test]
fn large_input_and_output_do_not_deadlock() {
    let input: Vec<u8> = vec![b'x'; 1_000_000];

    let output = output_within(&mut Command::new("cat"), Some(&input), &bound(Duration::from_secs(20))).expect("runs");

    assert_eq!(output.stdout.len(), input.len());
}

/// A child that exits without reading its input is reaped, and the failed write is not an error of the call.
#[test]
fn a_child_that_ignores_its_input_is_reaped() {
    let input: Vec<u8> = vec![b'x'; 1_000_000];

    let result = output_within(Command::new("sh").args(["-c", "exit 0"]), Some(&input), &bound(Duration::from_secs(10)));

    assert!(matches!(result, Ok(_)), "{:?}", result.map(|output| { return output.status; }));
}

/// A program that does not exist is an I/O error naming nothing but the system's words.
#[test]
fn a_program_that_does_not_exist_is_an_io_error() {
    let result = output_within(&mut Command::new("/nonexistent/program"), None, &bound(Duration::from_secs(1)));

    assert!(matches!(result, Err(RunError::Io(_))));
}
