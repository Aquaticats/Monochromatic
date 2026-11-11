//! Whether a file descriptor has a line to read,
//! waited for no longer than a limit.
//! The watcher's handshake runs while `watch-start` holds the lifecycle lock,
//! so a child that never speaks must not hold that lock for ever.

/// Polling a descriptor and reading the clock.
use std::io;
/// The descriptor to watch.
use std::os::fd::RawFd;
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
