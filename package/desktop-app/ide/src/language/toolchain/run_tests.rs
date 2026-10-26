//! The result of a finished run,
//!  the time limit,
//!  and a grandchild that holds the streams.

use super::run_bounded;
use std::{
    process::Command,
    time::{Duration, Instant},
};

#[test]
fn a_finished_run_gives_its_streams_and_status() {
    let mut command = Command::new("sh");
    command.args(["-c", "echo out; echo problem >&2; exit 3"]);
    let done = run_bounded(command, Duration::from_secs(10)).expect("run");
    assert!(!done.success);
    assert_eq!(done.stdout, "out\n");
    assert_eq!(done.stderr, "problem\n");
    assert!(done.status.contains('3'), "{}", done.status);
    let mut fine = Command::new("sh");
    fine.args(["-c", "exit 0"]);
    assert!(
        run_bounded(fine, Duration::from_secs(10))
            .expect("run")
            .success
    );
}

#[test]
fn a_run_that_hangs_is_stopped_at_the_limit() {
    let mut command = Command::new("sh");
    command.args(["-c", "exec sleep 60"]);
    let started = Instant::now();
    let reason = run_bounded(command, Duration::from_millis(300)).expect_err("timed out");
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "{:?}",
        started.elapsed()
    );
    assert!(reason.contains("did not finish within"), "{reason}");
}

#[test]
fn a_program_that_cannot_start_is_reported_with_its_name() {
    let reason = run_bounded(
        Command::new("/definitely/not/a/program"),
        Duration::from_secs(5),
    )
    .expect_err("spawn");
    assert!(reason.contains("/definitely/not/a/program"), "{reason}");
}

#[test]
fn a_grandchild_that_keeps_the_streams_open_does_not_hold_the_result() {
    let mut command = Command::new("sh");
    command.args(["-c", "sleep 20 & echo started"]);
    let started = Instant::now();
    let done = run_bounded(command, Duration::from_secs(10)).expect("run");
    assert!(done.success);
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "{:?}",
        started.elapsed()
    );
}
