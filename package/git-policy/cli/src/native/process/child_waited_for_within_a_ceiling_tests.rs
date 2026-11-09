//! Controls for a child waited for,
//! and ended,
//! by the thread that owns it.

use super::*;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// How long a control lets its own steps run before it reports that something was not ended.
const CONTROL_BOUND: Duration = Duration::from_secs(10);

/// A child of a shell command line,
/// with both streams piped.
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

/// A ceiling that refuses the next kill,
/// which is how a child the kernel will not let end looks from this wait's side.
fn refuse_the_next_kill() {
    A_KILL_IS_REFUSED.with(|refused| { refused.set(true); });
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
#[test]
fn a_child_that_cannot_be_ended_is_left_running_and_says_so() {
    let child: Child = quiet("sleep 30");
    let pid: u32 = child.id();
    refuse_the_next_kill();
    let began: Instant = Instant::now();
    let failure: std::io::Error = wait_within(child, Duration::from_millis(200)).expect_err("the ceiling ends the wait");
    let waited: Duration = began.elapsed();
    let alive: bool = recorded(pid);
    assert_eq!(failure.kind(), std::io::ErrorKind::TimedOut, "{failure}");
    assert!(left_running_when_the_ceiling_was_reached(&failure), "the wait did not say the child was left running");
    // The grace is the stop's own, and the wait ends with it rather than with the child.
    assert!(waited >= STOP_GRACE, "the wait did not give the child the stop's grace: {waited:?}");
    assert!(waited < STOP_GRACE + CONTROL_BOUND, "the wait outlasted the grace: {waited:?}");
    assert!(alive, "the premise: the child could not be ended");
    // The control ends the child it could not have ended, and the reaper that was handed it collects it.
    // SAFETY: `kill` takes two integers and touches no memory; the number is this control's own child, unreaped.
    let _ended: libc::c_int = unsafe { libc::kill(i32::try_from(pid).expect("a process number"), libc::SIGKILL) };
    let reaping: Instant = Instant::now();
    while recorded(pid) && reaping.elapsed() < CONTROL_BOUND {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(!recorded(pid), "the child the wait left running was collected by the thread that watched it");
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

/// What:
///  A stream that outgrows what this command keeps of one ends the read at the cap,
/// rather than being gathered for as long as the program writes.
/// Why:
///  A program the query started can keep writing into a stream it holds open after the query exited,
/// and a reader
///  that gathers it without a cap grows for as long as that program writes.
/// The third review's finding 4: the
///  reader stayed alive and kept growing after the wait had given up.
/// What is kept is bounded by `STEP_OUTPUT_LIMIT`,
/// and once it is reached the wait stops wanting the
///  rest,
/// which ends the reader,
/// so the read returns at the cap and not at the drain bound.
#[test]
fn a_stream_that_outgrows_the_cap_ends_the_read_at_the_cap() {
    // The helper writes without end into the query's standard output, which it inherits, and the query exits at once.
    let child: Child = piped("yes 0123456789 2>/dev/null & printf answer");
    let began: Instant = Instant::now();
    let failure: std::io::Error = output_within(child, Duration::from_secs(30)).expect_err("a cut standard output is not an answer");
    let waited: Duration = began.elapsed();
    assert!(failure.to_string().contains("standard output"), "{failure}");
    assert!(
        failure.to_string().contains(format!("{} bytes", crate::output_capture::STEP_OUTPUT_LIMIT).as_str()),
        "the failure names the cap that ended the read: {failure}"
    );
    assert!(
        waited < Duration::from_millis(900),
        "the read waited for the drain rather than ending at the cap: {waited:?}"
    );
}

/// What:
///  The diagnostics a query wrote before a program held its standard error open come back with the answer.
/// Why:
///  A query that fails writes its explanation to standard error,
/// so throwing that away leaves the caller a failure with
///  nothing to give the person.
/// The third review's finding 6: the reader kept what it read until an end of stream that never
///  came.
#[test]
fn diagnostics_written_before_the_standard_error_was_held_come_back() {
    // The helper holds the standard error alone; the query writes its explanation there and fails.
    let child: Child = piped("sleep 6 >/dev/null & printf explained >&2; exit 1");
    let began: Instant = Instant::now();
    let output: Output = output_within(child, Duration::from_secs(30)).expect("the exit status is the answer");
    let waited: Duration = began.elapsed();
    assert_eq!(output.status.code(), Some(1), "the query's own status comes back");
    assert_eq!(output.stderr, b"explained".to_vec(), "what the query wrote to its standard error comes back");
    assert!(waited < Duration::from_secs(3), "the wait outlasted the query by {waited:?}");
}
