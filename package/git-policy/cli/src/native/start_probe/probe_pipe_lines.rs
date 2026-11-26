//! What:
//!  The bytes a child writes and a parent reads on a pipe:
//!  one whole status line,
//!  a non-blocking drain,
//!  and the flag that makes a drain possible.
//! Why:
//!  The mode reads whole lines and never half of one,
//!  and a child writes each line in one call,
//!  so a line a reader can parse is a line a child finished
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The pipes and parent-death signals each child owns").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! writeAll(status, 'probe-step ...\n'); setNonblocking(status); const bytes = readAvailable(status, buffer);
//! ```

use super::probe_call_error::{
    CallError,
    failed,
};
use std::os::fd::RawFd;

/// What:
///  Write every byte of one line to a descriptor.
/// Why:
///  A status line is small and must arrive whole or not at all,
///  and a partial write would leave a reader waiting for the rest of a line it
///  cannot parse.
///
/// # Errors
/// The write answered an error.
pub fn write_all(
    fd: RawFd,
    bytes: &[u8],
) -> Result<(), CallError> {
    let mut written: usize = 0;
    while written < bytes.len() {
        // SAFETY: the slice's remaining bytes are this call's own and outlive it.
        let wrote: libc::ssize_t = unsafe {
            libc::write(
                fd,
                bytes[written..].as_ptr().cast::<libc::c_void>(),
                bytes.len() - written,
            )
        };
        if wrote < 0 {
            let error: std::io::Error = std::io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::EINTR) {
                continue;
            }
            return Err(failed("write"));
        }
        written += usize::try_from(wrote).unwrap_or(bytes.len());
    }
    return Ok(());
}

/// What:
///  Read what is available on one non-blocking descriptor,
///  up to a bound,
///  and tell nothing from nothing-yet from the pipe's end.
/// Why:
///  A status pipe that ended is a child that wrote no more,
///  which is an arm failure,
///  while a pipe with no bytes yet is a child still working:
///  one number cannot say both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const read = readAvailable(fd, buffer); // { kind: 'bytes'; count } | { kind: 'nothing-yet' } | { kind: 'end' }
/// ```
///
/// # Errors
/// The read answered an error other than "nothing there yet".
pub fn read_available(
    fd: RawFd,
    buffer: &mut [u8],
) -> Result<ReadSome, CallError> {
    // SAFETY: the buffer is this call's own and the call writes at most its length.
    let read: libc::ssize_t =
        unsafe { libc::read(fd, buffer.as_mut_ptr().cast::<libc::c_void>(), buffer.len()) };
    if read < 0 {
        let error: std::io::Error = std::io::Error::last_os_error();
        return match error.raw_os_error() {
            Some(libc::EAGAIN) | Some(libc::EINTR) => Ok(ReadSome::NothingYet),
            _ => Err(failed("read")),
        };
    }
    if read == 0 {
        return Ok(ReadSome::End);
    }
    return Ok(ReadSome::Bytes(usize::try_from(read).unwrap_or(0)));
}

/// What:
///  What one non-blocking read answered.
/// Why:
///  The three answers drive three different decisions,
///  and the mode's records name an end of file rather than an empty read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReadSome = { kind: 'bytes'; count: number } | { kind: 'nothing-yet' } | { kind: 'end' };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReadSome {
    /// This many bytes arrived.
    Bytes(usize),
    /// Nothing was there yet.
    NothingYet,
    /// The pipe is at its end.
    End,
}

/// What:
///  Read exactly one byte,
///  waiting for it.
/// Why:
///  The worker's request to the mode is one byte and its reply is one byte,
///  so each side waits for the other's single byte rather than polling.
///
/// # Errors
/// The read answered an error or the pipe ended.
pub fn read_one_blocking(fd: RawFd) -> Result<u8, CallError> {
    let mut byte: [u8; 1] = [0];
    loop {
        // SAFETY: the buffer is this call's own and the call writes at most its length.
        let read: libc::ssize_t =
            unsafe { libc::read(fd, byte.as_mut_ptr().cast::<libc::c_void>(), 1) };
        if read == 1 {
            return Ok(byte[0]);
        }
        if read == 0 {
            return Err(CallError {
                call: "read",
                errno: libc::EPIPE,
            });
        }
        let error: std::io::Error = std::io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::EINTR) {
            return Err(failed("read"));
        }
    }
}

/// What:
///  Read one whole line,
///  waiting for its newline,
///  within a bound.
/// Why:
///  The mode answers the worker's holder request with the holder's namespace,
///  which is a word rather than one byte,
///  and a line keeps the pipe's framing the same as every other message.
///
/// # Errors
/// The read answered an error,
///  the pipe ended,
///  or the line passed the bound.
pub fn read_line_blocking(
    fd: RawFd,
    limit: usize,
) -> Result<String, CallError> {
    let mut line: Vec<u8> = Vec::new();
    loop {
        let mut byte: [u8; 1] = [0];
        // SAFETY: the buffer is this call's own and the call writes at most its length.
        let read: libc::ssize_t =
            unsafe { libc::read(fd, byte.as_mut_ptr().cast::<libc::c_void>(), 1) };
        if read == 0 {
            return Err(CallError {
                call: "read",
                errno: libc::EPIPE,
            });
        }
        if read < 0 {
            let error: std::io::Error = std::io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::EINTR) {
                continue;
            }
            return Err(failed("read"));
        }
        if byte[0] == b'\n' {
            return String::from_utf8(line).map_err(|_| {
                return CallError {
                    call: "read",
                    errno: libc::EINVAL,
                };
            });
        }
        line.push(byte[0]);
        if line.len() > limit {
            return Err(CallError {
                call: "read",
                errno: libc::E2BIG,
            });
        }
    }
}

/// What:
///  Make one descriptor non-blocking.
/// Why:
///  A pipe the mode drains is read until it answers "nothing there yet",
///  which only a non-blocking descriptor says.
///
/// # Errors
/// The call refused.
pub fn set_nonblocking(fd: RawFd) -> Result<(), CallError> {
    // SAFETY: `fcntl` reads the descriptor's flags and writes the one word it was given.
    let flags: libc::c_int = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 {
        return Err(failed("fcntl"));
    }
    // SAFETY: the same descriptor, with the one bit this process sets.
    let set: libc::c_int = unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) };
    return if set == 0 {
        Ok(())
    } else {
        Err(failed("fcntl"))
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_pipe_lines_tests.rs"]
mod tests;
