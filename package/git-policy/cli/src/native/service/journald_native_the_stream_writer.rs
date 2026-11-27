//! What:
//!  A writer to the journal's stream (standard error as the service manager connected it)
//!  that never waits for room:
//!  one non-blocking write,
//!  and how much it took.
//! Why:
//!   `O_NONBLOCK` on the datagram socket does not make standard error non-blocking,
//!  and a journal that stops reading would fill the stream and block the service,
//!  and so the commits that wait for it;
//!  each write is `MSG_DONTWAIT` on a socket,
//!  or,
//!  on a pipe or file a test or a redirection put there,
//!  a write made only after a poll shows room and of no more than a pipe takes whole,
//!  so the caller keeps the rest (`Sink::deliver` does) instead of waiting for room.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! try { socket.sendNonBlocking(line) } catch (error) { if (error.code === 'EAGAIN') report(error) }
//! ```

use std::io::Write;
///  A writer to the journal's stream (standard error as the service manager connected it) that never waits for room.
/// Why:
///   `O_NONBLOCK` on the datagram socket does not make standard error non-blocking,
/// and a journal that stops reading would
///  fill the stream and block the service,
/// and so the commits that wait for it.
/// Each write is `MSG_DONTWAIT` on a socket,
/// or,
///  on a pipe or file a test or a redirection put there,
/// a write made only after a poll shows room and of no more than a
///  pipe takes whole;
/// a write that finds no room is refused with `WouldBlock`,
/// and one that takes part of the bytes says how
///  many,
/// so the caller keeps the rest (`Sink::deliver` does) instead of the writer waiting for room.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { socket.sendNonBlocking(line) } catch (error) { if (error.code === 'EAGAIN') report(error) }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct StreamWriter {
    /// The descriptor of the stream.
    descriptor: std::os::fd::RawFd,
}

/// The most a pipe takes in one write once a poll shows room:
/// a page,
/// which a pipe holding a free slot takes whole.
pub const PIPE_CHUNK: usize = 4096;

/// What:
///  Sends bytes to a descriptor without ever waiting for room,
/// and gives the count taken or the error number.
/// Why:
///   One rule for every write the service makes to standard error,
/// including the signal handler's,
/// which may call only
///  functions that `signal-safety(7)` lists (read from <https://man7.org/linux/man-pages/man7/signal-safety.7.html>):
/// `send`,
///  `poll` and `write` are,
/// and this uses nothing else,
/// allocates nothing and takes no lock.
/// A socket is written with
///  `MSG_DONTWAIT`.
/// A pipe or file a test or a redirection put there is written only after a poll shows room,
/// and in a piece
///  of at most a page,
/// which a pipe holding a free slot takes whole;
/// so the write cannot block.
/// No room is `EAGAIN`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!(await pollWritable(fd, 0))) throw errno('EAGAIN'); return write(fd, bytes.slice(0, 4096));
/// ```
///
/// # Errors
/// The error number of the failed call,
/// `EAGAIN` when there is no room.
pub fn send_without_waiting(
    descriptor: std::os::fd::RawFd,
    bytes: &[u8],
) -> Result<usize, libc::c_int> {
    // SAFETY: `bytes` is valid for its length for the call, and `send` is safe in a signal handler.
    let sent: isize = unsafe {
        libc::send(
            descriptor,
            bytes.as_ptr().cast(),
            bytes.len(),
            libc::MSG_DONTWAIT | libc::MSG_NOSIGNAL,
        )
    };
    if let Ok(count) = usize::try_from(sent) {
        return Ok(count);
    }
    // SAFETY: the location is the calling thread's own error number, valid for a read.
    let failure: libc::c_int = unsafe { *libc::__errno_location() };
    if failure != libc::ENOTSOCK {
        return Err(failure);
    }
    // Not a socket (a file or a pipe a test or a redirection put there): an ordinary write, made only when the descriptor
    //  shows room and small enough for a pipe to take whole, so it cannot block.
    let mut request: libc::pollfd = libc::pollfd {
        fd: descriptor,
        events: libc::POLLOUT,
        revents: 0,
    };
    // SAFETY: `request` is valid for the call, which reads and writes one `pollfd`, and `poll` is safe in a signal handler.
    let ready: libc::c_int = unsafe { libc::poll(&raw mut request, 1, 0) };
    if ready <= 0 || request.revents & libc::POLLOUT == 0 {
        return Err(libc::EAGAIN);
    }
    let chunk: &[u8] = &bytes[..bytes.len().min(PIPE_CHUNK)];
    // SAFETY: `chunk` is valid for its length for the call, and `write` is safe in a signal handler.
    let written: isize = unsafe { libc::write(descriptor, chunk.as_ptr().cast(), chunk.len()) };
    return match usize::try_from(written) {
        Ok(count) => Ok(count),
        Err(_negative) => {
            // SAFETY: the location is the calling thread's own error number, valid for a read.
            Err(unsafe { *libc::__errno_location() })
        }
    };
}

impl StreamWriter {
    /// A writer to standard error.
    pub fn stderr() -> Self {
        return Self {
            descriptor: libc::STDERR_FILENO,
        };
    }

    /// A writer to a stream socket descriptor.
    pub fn on(descriptor: std::os::fd::RawFd) -> Self {
        return Self {
            descriptor,
        };
    }
}

impl Write for StreamWriter {
    fn write(
        &mut self,
        bytes: &[u8],
    ) -> std::io::Result<usize> {
        return send_without_waiting(self.descriptor, bytes)
            .map_err(std::io::Error::from_raw_os_error);
    }

    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}
