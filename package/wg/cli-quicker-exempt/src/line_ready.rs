//! Whether a file descriptor has a line to read,
//! waited for no longer than a limit.
//! The watcher's handshake runs while `watch-start` holds the lifecycle lock,
//! so a child that never speaks must not hold that lock for ever.

/// Polling a descriptor, reading a line and waiting for a child.
use std::io::{self, BufRead, BufReader};
/// The descriptor to watch.
use std::os::fd::{AsRawFd, RawFd};
/// The handshake child.
use std::process::{Child, ChildStdout, ExitStatus};
/// Bounded waits.
use std::time::{Duration, Instant};

/// Polls one descriptor for reading until `limit` passes,
/// and reports whether it became readable.
/// A limit of zero asks once and returns at once.
pub fn line_is_ready(descriptor: RawFd, limit: Duration) -> io::Result<bool> {
    let deadline = Instant::now() + limit;
    loop {
        let left = deadline.saturating_duration_since(Instant::now());
        let mut polled = libc::pollfd { fd: descriptor, events: libc::POLLIN, revents: 0 };
        let milliseconds = i32::try_from(left.as_millis()).unwrap_or(i32::MAX);
        // SAFETY: one `pollfd` of the length passed, for a descriptor the caller keeps open.
        let ready = unsafe { libc::poll(std::ptr::from_mut(&mut polled), 1, milliseconds) };
        if ready < 0 {
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(error);
        }
        if ready > 0 {
            return Ok(true);
        }
        if Instant::now() >= deadline {
            return Ok(false);
        }
    }
}

/// Reads one newline-delimited handshake message,
/// and answers a timeout instead of waiting for one that never comes.
/// A child that closed the channel answers the end of the stream.
pub fn read_message_within(output: &mut BufReader<ChildStdout>, limit: Duration, what: &str) -> io::Result<String> {
    if !line_is_ready(output.get_ref().as_raw_fd(), limit)? {
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            format!("the child did not report {what} within {limit:?}"),
        ));
    }
    let mut line = String::new();
    let count = output.read_line(&mut line)?;
    if count == 0 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "the child closed the handshake channel",
        ));
    }
    return Ok(line.trim_end().to_owned());
}

/// Waits for a child to end,
/// ends it when `limit` passes,
/// and reaps it either way,
/// so a child that never ends cannot hold the caller.
pub fn wait_within(child: &mut Child, limit: Duration) -> io::Result<ExitStatus> {
    let deadline = Instant::now() + limit;
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        if Instant::now() >= deadline {
            let _killed = child.kill();
            return child.wait();
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
