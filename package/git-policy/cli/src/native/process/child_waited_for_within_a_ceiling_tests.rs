//! Controls for a child waited for, and ended, by the thread that owns it.

use super::*;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// How long a control lets its own steps run before it reports that something was not ended.
const CONTROL_BOUND: Duration = Duration::from_secs(10);

/// A child of a shell command line, with both streams piped.
fn piped(line: &str) -> Child {
    let mut command = Command::new("/bin/sh");
    command.arg("-c").arg(line).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    return command.spawn().expect("the stand-in starts");
}

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
    let mut child: Child = quiet("exit 3");
    let status: ExitStatus = wait_within(&mut child, Duration::from_secs(30)).expect("the child ends");
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
    let mut child: Child = quiet("sleep 30");
    let pid: u32 = child.id();
    let began: Instant = Instant::now();
    let failure: std::io::Error = wait_within(&mut child, Duration::from_millis(300)).expect_err("the ceiling ends the child");
    let waited: Duration = began.elapsed();
    let left: bool = recorded(pid);
    assert_eq!(failure.kind(), std::io::ErrorKind::TimedOut, "{failure}");
    assert!(waited < CONTROL_BOUND, "the ceiling ended the child: {waited:?}");
    assert!(!left, "the child the ceiling ended was reaped, so no zombie is left behind");
}

/// Both streams of a query come back whole.
#[test]
fn both_streams_of_a_query_come_back_whole() {
    let child: Child = piped("head -c 300000 /dev/zero; printf there >&2");
    let output: Output = output_within(child, Duration::from_secs(30)).expect("the query answers");
    assert_eq!(output.stdout.len(), 300_000, "the standard output comes back whole");
    assert_eq!(output.stderr, b"there".to_vec(), "the standard error comes back whole");
    assert!(output.status.success());
}

/// What:
///  A query whose standard error a program it started still holds open answers anyway.
/// Why:
///  The query's answer is its exit status and its standard output,
/// so a program that inherited only the standard
///  error must not take them away;
/// the read of that stream stops one drain after the query ended,
/// and a line says so.
#[test]
fn a_standard_error_held_open_still_answers() {
    // The helper holds the standard error alone: its standard output goes elsewhere when the query ends.
    let child: Child = piped("sleep 6 >/dev/null & printf answer; printf warned >&2");
    let began: Instant = Instant::now();
    let output: Output = output_within(child, Duration::from_secs(30)).expect("the exit status and standard output are the answer");
    let waited: Duration = began.elapsed();
    assert!(output.status.success(), "{:?}", output.status);
    assert_eq!(output.stdout, b"answer".to_vec(), "the answer comes back whole");
    assert!(waited < Duration::from_secs(3), "the wait outlasted the query by {waited:?}, as long as the holder lived");
}

/// What:
///  A query whose standard output a program it started still holds open is not answered at all.
/// Why:
///  The standard output is the answer,
/// and what was read of it before the drain ended may be the beginning of it,
///  which a reader of records would take for all of it.
#[test]
fn a_standard_output_held_open_is_not_an_answer() {
    // The helper holds the standard output alone.
    let child: Child = piped("sleep 6 2>/dev/null & printf answer");
    let began: Instant = Instant::now();
    let failure: std::io::Error = output_within(child, Duration::from_secs(30)).expect_err("a cut answer is not an answer");
    let waited: Duration = began.elapsed();
    assert!(failure.to_string().contains("standard output"), "{failure}");
    assert!(failure.to_string().contains("still holds"), "{failure}");
    assert!(waited < Duration::from_secs(3), "the wait outlasted the query by {waited:?}, as long as the holder lived");
}
