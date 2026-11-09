//! What:
//!  The process's own standard error descriptor as a stream,
//! written without Rust's shared standard-error lock.
//! Why:
//!  `std::io::stderr()` hands out one shared handle behind a lock,
//! and every write through it holds that lock for
//!  as long as the write takes.
//!  A warning is written on a thread of its own and abandoned at its ceiling (`work_awaited_within_a_ceiling.rs`),
//! so a worker
//!  still waiting inside a write would hold that lock,
//! and the commit's next synchronous write to standard error would
//!  wait for it with no end (`doc/planning/cli-git-health-route-findings-on-main.md`,
//! the round-2 review).
//!  Writing the bytes to the descriptor itself takes no lock this process shares,
//! so an abandoned worker owns nothing another
//!  writer can wait for.
//!  A line that goes to a terminal or a file arrives unbuffered,
//! which is what an unbuffered `stderr` does anyway.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const descriptor = { write: (bytes) => fs.writeSync(2, bytes) };
//! ```

/// The trait that gives this stream its `write` and `write_all`.
use std::io::Write;

/// What:
///  This process's standard error descriptor,
/// as a stream.
///  `StandardErrorDescriptor` is a zero-sized value;
/// the descriptor number is the process's own.
/// Why:
///   A caller passes it where a stream is expected,
/// and nothing is shared between threads,
/// so a write that blocks
///  holds no lock another writer needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stream = new StandardErrorDescriptor();
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct StandardErrorDescriptor;

/// Writes go to descriptor 2,
/// one system call each.
impl Write for StandardErrorDescriptor {
    /// Write the bytes to descriptor 2.
    #[cfg(unix)]
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        loop {
            // SAFETY: descriptor 2 is this process's own standard error, and the buffer is read for the length given
            //  and nothing else.
            let written: libc::ssize_t = unsafe { libc::write(libc::STDERR_FILENO, bytes.as_ptr().cast(), bytes.len()) };
            if written < 0 {
                let error: std::io::Error = std::io::Error::last_os_error();
                // `.write_all(..)` retries an interrupted write, so the interruption is handed back as it is.
                if error.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(error);
            }
            return Ok(usize::try_from(written).unwrap_or_default());
        }
    }

    /// Write the bytes through the standard library on a platform without descriptors.
    /// UNTESTED ON WINDOWS:
    /// no gate compiles or runs this branch (the crate does not build for Windows;
    /// README,
    /// Requirements).
    #[cfg(not(unix))]
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        return std::io::stderr().write(bytes);
    }

    /// Nothing is buffered,
    /// so there is nothing to flush.
    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// The descriptor's controls stay out of the release executable.
#[cfg(test)]
#[path = "standard_error_descriptor_tests.rs"]
mod tests;
