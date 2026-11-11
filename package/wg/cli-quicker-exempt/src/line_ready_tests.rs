//! Verifies the bound on a wait for a line:
//! a descriptor with a line is reported ready at once,
//! one with nothing is reported not ready when the limit passes,
//! and a closed one is ready,
//! so the handshake of `watch-start` never waits for a watcher that stopped speaking
//! while it holds the lifecycle lock.

/// The wait under test.
use crate::line_ready::line_is_ready;
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
