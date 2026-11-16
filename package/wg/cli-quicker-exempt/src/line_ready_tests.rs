//! Verifies the bound on a wait for a line:
//! a descriptor with a line is reported ready at once,
//! one with nothing is reported not ready when the limit passes,
//! and a closed one is ready,
//! so the handshake of `watch-start` never waits for a watcher that stopped speaking
//! while it holds the lifecycle lock.

/// The waits under test.
use crate::line_ready::{line_is_ready, read_message_within, wait_within};
/// Reading the fixture's line.
use std::io::{BufRead, BufReader};
/// Starting the fixture.
use std::os::fd::AsRawFd;
use std::process::{Command, Stdio};
/// The limit.
use std::time::Duration;

/// A line that is already there is ready at once.
#[test]
fn a_line_that_is_there_is_ready_at_once() {
    let mut child = Command::new("/bin/sh").arg("-c").arg("printf 'READY\\n'").stdout(Stdio::piped()).spawn().expect("start the fixture");
    let stdout = child.stdout.take().expect("the fixture writes");
    let descriptor = stdout.as_raw_fd();
    assert!(matches!(line_is_ready(descriptor, Duration::from_secs(5)), Ok(true)));
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    reader.read_line(&mut line).expect("read the line");
    assert_eq!(line.trim_end(), "READY");
    let _status = child.wait();
}

/// A child that says nothing is not ready when the limit passes,
/// and the wait returns instead of blocking for ever.
#[test]
fn a_child_that_says_nothing_is_not_ready() {
    let mut child = Command::new("/bin/sh").arg("-c").arg("exec /usr/bin/sleep 5").stdout(Stdio::piped()).spawn().expect("start the fixture");
    let stdout = child.stdout.take().expect("the fixture keeps its stream open");
    let started = std::time::Instant::now();
    assert!(matches!(line_is_ready(stdout.as_raw_fd(), Duration::from_millis(100)), Ok(false)));
    assert!(started.elapsed() < Duration::from_secs(2), "the wait returned at its limit");
    let _killed = child.kill();
    let _reaped = child.wait();
}

/// A closed channel counts as ready,
/// so the handshake reads the end of the stream and reports it.
#[test]
fn a_closed_channel_is_ready() {
    let mut child = Command::new("/bin/sh").arg("-c").arg("exit 0").stdout(Stdio::piped()).spawn().expect("start the fixture");
    let stdout = child.stdout.take().expect("the fixture has a stream");
    let descriptor = stdout.as_raw_fd();
    let _status = child.wait();
    assert!(matches!(line_is_ready(descriptor, Duration::from_secs(1)), Ok(true)));
    drop(stdout);
}

/// A handshake that never comes ends at the limit,
/// so the watcher and the keeper cannot hold the lifecycle lock for ever.
#[test]
fn a_child_that_says_nothing_ends_its_handshake_wait_at_the_limit() {
    let mut child = Command::new("/bin/sh").arg("-c").arg("exec /usr/bin/sleep 30").stdout(Stdio::piped()).spawn().expect("start the fixture");
    let stdout = child.stdout.take().expect("the fixture keeps its stream open");
    let mut reader = BufReader::new(stdout);
    let started = std::time::Instant::now();
    let refusal = read_message_within(&mut reader, Duration::from_millis(100), "readiness").expect_err("the handshake is refused");
    assert_eq!(refusal.kind(), std::io::ErrorKind::TimedOut);
    assert!(started.elapsed() < Duration::from_secs(2), "the wait returned at its limit");
    let _killed = child.kill();
    let _reaped = child.wait();
}

/// A child that never ends is ended by the bounded wait,
/// and the wait answers its status instead of holding the caller.
#[test]
fn a_child_that_never_ends_is_ended_by_the_bounded_wait() {
    let mut child = Command::new("/usr/bin/sleep").arg("30").stdout(Stdio::null()).spawn().expect("start the fixture");
    let started = std::time::Instant::now();
    let status = wait_within(&mut child, Duration::from_millis(100)).expect("the wait answers");
    assert!(!status.success());
    assert!(started.elapsed() < Duration::from_secs(2), "the wait returned at its limit");
    assert!(child.try_wait().expect("the child is reaped").is_some(), "the child was reaped");
}

/// The two handshakes read and wait through the bounded helpers,
/// and neither keeps a blocking read or an unbounded child wait of its own.
/// The guard reads the sources because the wait it forbids is invisible in a passing run:
/// a child that speaks in time proves nothing about the one that never speaks.
#[test]
fn both_handshakes_use_the_bounded_read_and_wait() {
    for (name, source) in [
        ("keeper.rs", include_str!("keeper.rs")),
        ("application_watch_service.rs", include_str!("application_watch_service.rs")),
    ] {
        assert!(
            source.contains("read_message_within("),
            "{name} reads its handshake through the bounded reader"
        );
        assert!(
            source.contains("wait_within("),
            "{name} waits for its candidate through the bounded wait"
        );
        assert!(
            !source.contains("output.read_line("),
            "{name} keeps no blocking handshake read"
        );
        assert!(
            !source.contains("child.wait()?"),
            "{name} keeps no unbounded child wait"
        );
    }
}

/// A handshake that arrives without its newline must not hold the caller past the limit.
///
/// Readiness and the line are not one step:
/// a child can write part of a line and then keep its stream open,
/// so a reader that polled once and then read to the newline
/// waits for the rest of a line that may never come,
/// while `watch-start` holds the lifecycle lock.
#[test]
fn a_partial_line_does_not_hold_the_handshake_past_its_limit() {
    let mut child = Command::new("/bin/sh")
        .arg("-c")
        .arg("printf READY; exec /usr/bin/sleep 30")
        .stdout(Stdio::piped())
        .spawn()
        .expect("start a child that writes half a line");
    let stdout = child.stdout.take().expect("the child keeps its stream open");
    let mut reader = BufReader::new(stdout);
    let started = std::time::Instant::now();
    let answer = read_message_within(&mut reader, Duration::from_millis(100), "readiness");
    let elapsed = started.elapsed();
    let _killed = child.kill();
    let _reaped = child.wait();
    assert!(elapsed < Duration::from_secs(2), "the handshake held the caller for {elapsed:?}");
    let refusal = answer.expect_err("half a line is not a handshake");
    assert_eq!(refusal.kind(), std::io::ErrorKind::TimedOut, "{refusal}");
    assert!(refusal.to_string().contains("readiness"), "{refusal}");
}
