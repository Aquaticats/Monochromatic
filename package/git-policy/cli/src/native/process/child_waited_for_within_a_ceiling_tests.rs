//! Controls for a child waited for,
//! and ended,
//! by the thread that owns it.

use super::*;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// How long a control lets its own steps run before it reports that something was not ended.
const CONTROL_BOUND: Duration = Duration::from_secs(10);

/// A child of a shell command line whose streams go nowhere.
fn quiet(line: &str) -> Child {
    let mut command = Command::new("/bin/sh");
    command.arg("-c").arg(line).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    return command.spawn().expect("the stand-in starts");
}

/// Whether the record of `pid` is in the process table,
/// which a zombie still has until it is reaped.
fn recorded(pid: u32) -> bool {
    return std::fs::read_to_string(format!("/proc/{pid}/stat")).is_ok();
}

/// A child that ends within its ceiling gives its status,
/// and nothing is killed.
#[test]
fn a_child_that_ends_within_its_ceiling_gives_its_status() {
    let child: Child = quiet("exit 3");
    let status: ExitStatus = wait_within(child, Duration::from_secs(30)).expect("the child ends");
    assert_eq!(status.code(), Some(3), "the child's own status comes back");
}

/// What:
///  A child that never ends is ended at its ceiling and reaped there.
/// Why:
///  The thread that waits is the thread that kills,
/// so the kill reaches the child and nothing else,
///  and the reap is what leaves no zombie behind.
#[test]
fn a_child_that_never_ends_is_ended_and_reaped_at_its_ceiling() {
    let child: Child = quiet("sleep 30");
    let pid: u32 = child.id();
    let began: Instant = Instant::now();
    let failure: std::io::Error = wait_within(child, Duration::from_millis(300)).expect_err("the ceiling ends the child");
    let waited: Duration = began.elapsed();
    let left: bool = recorded(pid);
    assert_eq!(failure.kind(), std::io::ErrorKind::TimedOut, "{failure}");
    assert!(waited < CONTROL_BOUND, "the ceiling ended the child: {waited:?}");
    assert!(!left, "the child the ceiling ended was reaped, so no zombie is left behind");
}

/// What:
///  A child the kernel does not let end is not waited for beyond the stop's grace:
/// the wait hands it to a reaper and says it was left running.
/// Why:
///  A child in an uninterruptible call cannot end until that call returns,
/// and the third review's finding 3 was the
///  `wait()` after the kill,
/// which had no bound of its own,
/// so the command could be held for as long as the kernel took.
///  The kernel cannot be made to hold a process here,
/// so the kill is what a control refuses:
/// a child that is not
///  signalled and does not end is the same state from this wait's side.
/// The child ends by itself once the grace is over,
/// and the reaper collects it:
/// no control
///  signals a number here (the fourth review's finding 10),
/// because the child is the reaper's from the moment it
///  is handed over.
#[test]
fn a_child_that_cannot_be_ended_is_left_running_and_says_so() {
    // The child lives a little past the grace and then ends by itself, so the reaper collects a child no control had to kill.
    let child: Child = quiet("sleep 6");
    let pid: u32 = child.id();
    a_control_refuses_the_next_kill();
    let began: Instant = Instant::now();
    let failure: std::io::Error = wait_within(child, Duration::from_millis(200)).expect_err("the ceiling ends the wait");
    let waited: Duration = began.elapsed();
    assert_eq!(failure.kind(), std::io::ErrorKind::TimedOut, "{failure}");
    assert!(left_running_when_the_ceiling_was_reached(&failure), "the wait did not say the child was left running");
    // The grace is the stop's own, and the wait ends with it rather than with the child.
    assert!(waited >= STOP_GRACE, "the wait did not give the child the stop's grace: {waited:?}");
    assert!(waited < STOP_GRACE + CONTROL_BOUND, "the wait outlasted the grace: {waited:?}");
    assert!(recorded(pid), "the premise: the child could not be ended");
    // The child ends by itself, and the thread it was handed to collects it: no zombie of that number is left.
    let reaping: Instant = Instant::now();
    while recorded(pid) && reaping.elapsed() < CONTROL_BOUND {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(!recorded(pid), "the child the wait left running was collected by the thread that watches it");
}
