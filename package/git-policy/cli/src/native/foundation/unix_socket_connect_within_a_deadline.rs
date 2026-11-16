//! What:
//!  A connection to a unix socket that is made within one deadline,
//!  or nothing.
//! Why:
//!  `UnixStream::connect` waits while a listening socket's backlog is full,
//!  and a backlog drains only when the program that listens accepts:
//!  a service stopped at any instruction never accepts,
//!  so a plain connect there waits for as long as the process stays stopped.
//!  Nothing of this program may wait for ever,
//!  and a connection made for a shortcut must not end a command that would otherwise have run to its end.
//!  The socket is therefore made non-blocking,
//!  connected, and waited for with `poll` until the deadline,
//!  which is the same call `poll` in `service_index_watch.rs` waits with.
//!  A connection that cannot be made in time is no connection,
//!  and the caller takes the path it would have taken without one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stream = connectWithin(socketPath, 250); // a socket, or an error, inside the deadline
//! ```

/// The bytes of a socket path,
/// which the kernel compares as they are.
use crate::path_bytes::path_bytes_of;
/// A failure and the descriptor the connection is made on.
use std::io;
/// Ownership of the descriptor once it is a connected socket.
use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd, OwnedFd};
/// The stream the caller reads and writes.
use std::os::unix::net::UnixStream;
/// The socket path.
use std::path::Path;
/// The deadline.
use std::time::{Duration, Instant};

/// What:
///  A connected socket at `path`, made within `wait`.
/// Why:
///  A socket nobody listens on refuses the connection at once,
///  a socket whose listener is behind refuses nothing and connects nothing,
///  and only the deadline tells the second from a connection still on its way.
///  The answer is a blocking stream, which is what a caller that sets a read timeout expects.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// connectWithin(path, wait) // UnixStream, or the failure that ended the attempt
/// ```
///
/// # Errors
///
/// `io::ErrorKind::TimedOut` when the connection was not made within `wait`;
/// `io::ErrorKind::InvalidInput` for a path longer than one socket address;
/// the operating system's own failure otherwise (a socket file that is gone or nothing listens on it).
pub fn connect_within(path: &Path, wait: Duration) -> io::Result<UnixStream> {
    let deadline: Instant = Instant::now() + wait;
    let descriptor: OwnedFd = socket_of_a_stream()?;
    let (address, length): (libc::sockaddr_un, libc::socklen_t) = address_of(path)?;
    wait_until_connected(descriptor.as_raw_fd(), &address, length, deadline)?;
    // The caller sets a read timeout and reads a line; a descriptor still marked non-blocking answers EAGAIN instead.
    become_blocking(descriptor.as_raw_fd())?;
    // SAFETY: the descriptor is a connected unix stream socket, nothing else holds it, and its ownership moves into the stream.
    return Ok(unsafe { UnixStream::from_raw_fd(descriptor.into_raw_fd()) });
}

/// What:
///  A non-blocking socket of the family and type of a unix stream.
/// Why:
///   The connection has to be asked for on a socket that does not wait,
///  or the call that makes it is the one that waits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fd = socket(AF_UNIX, SOCK_STREAM | SOCK_NONBLOCK | SOCK_CLOEXEC, 0);
/// ```
fn socket_of_a_stream() -> io::Result<OwnedFd> {
    // SAFETY: `socket` creates a descriptor of the family and type it is given and touches no memory of this process.
    let made: libc::c_int = unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_STREAM | libc::SOCK_NONBLOCK | libc::SOCK_CLOEXEC, 0) };
    if made < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `made` is an open descriptor of this process and nothing else owns it.
    return Ok(unsafe { OwnedFd::from_raw_fd(made) });
}

/// What:
///  The socket address of one path,
///  and the length the kernel reads of it.
/// Why:
///  A unix socket's name is bytes in a fixed field,
///  so a path of its own bytes reaches the kernel unchanged,
///  and one that cannot fit is refused rather than cut to a name that is another socket.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { address, length } = addressOf(path);
/// ```
fn address_of(path: &Path) -> io::Result<(libc::sockaddr_un, libc::socklen_t)> {
    let bytes: &[u8] = path_bytes_of(path);
    // SAFETY: `sockaddr_un` is plain data and an all-zero value of it is valid.
    let mut address: libc::sockaddr_un = unsafe { std::mem::zeroed() };
    if bytes.len() >= address.sun_path.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("the socket path {} needs {} bytes and one socket address holds {}", path.display(), bytes.len(), address.sun_path.len() - 1),
        ));
    }
    address.sun_family = libc::AF_UNIX as libc::sa_family_t;
    for (slot, byte) in address.sun_path.iter_mut().zip(bytes) {
        *slot = *byte as libc::c_char;
    }
    let length: libc::socklen_t = std::mem::size_of::<libc::sa_family_t>().saturating_add(bytes.len()).saturating_add(1) as libc::socklen_t;
    return Ok((address, length));
}

/// The pace an ask of a socket whose listener is behind is repeated at.
/// Why:
///  A unix socket whose listen backlog is full answers "try again" and queues nothing,
///  so the ask has to be made again;
///  five milliseconds is a pace and not a limit:
///  the deadline bounds the whole exchange,
///  and a listener that accepts once is noticed within one pause.
const ASK_PAUSE: Duration = Duration::from_millis(5);

/// What:
///  Asks for the connection and waits for it until the deadline.
/// Why:
///  A non-blocking connect answers at once in one of three ways:
///  connected, refused (an error),
///  or "under way",
///  and only the last two are waited for.
///  A socket whose listener is behind answers `EAGAIN` and queues nothing,
///  so the ask is made again after a pause instead of being read as a connection:
///  `poll` on an unconnected socket reports it writable at once,
///  and a control of a full backlog showed the connection being taken for made.
///  A socket that becomes writable after `EINPROGRESS` may still carry the connection's own failure,
///  which `SO_ERROR` holds and the descriptor's flags do not,
///  so the answer is read there instead of guessed from writability.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (;;) { const answer = connect(fd, address); if (!answer) return; if (answer === EINPROGRESS) return await socketError(await pollWritable(fd, deadline)); if (answer === EAGAIN && Date.now() < deadline) { await sleep(5); continue; } throw answer; }
/// ```
fn wait_until_connected(descriptor: libc::c_int, address: &libc::sockaddr_un, length: libc::socklen_t, deadline: Instant) -> io::Result<()> {
    loop {
        // SAFETY: `connect` reads the address it is given and the length says how much of it; the socket is this process's.
        let asked: libc::c_int = unsafe { libc::connect(descriptor, std::ptr::from_ref(address).cast::<libc::sockaddr>(), length) };
        if asked == 0 {
            return Ok(());
        }
        let refused: io::Error = io::Error::last_os_error();
        match refused.raw_os_error() {
            Some(libc::EINPROGRESS) => {
                wait_until_writable(descriptor, deadline)?;
                return carried_failure(descriptor);
            }
            Some(libc::EAGAIN) => {
                if Instant::now() >= deadline {
                    return Err(no_connection_in_time());
                }
                std::thread::sleep(ASK_PAUSE);
            }
            _ => return Err(refused),
        }
    }
}

/// What:
///  Waits until a connection that is under way has an answer,
///  or until the deadline.
/// Why:
///   The only thing that ends such a wait is the kernel having the answer,
///  and a socket that never answers it ends at the deadline instead of waiting for one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await pollWritable(fd, deadline);
/// ```
fn wait_until_writable(descriptor: libc::c_int, deadline: Instant) -> io::Result<()> {
    loop {
        let left: Duration = deadline.saturating_duration_since(Instant::now());
        let milliseconds: libc::c_int = libc::c_int::try_from(left.as_millis()).unwrap_or(libc::c_int::MAX);
        let mut watched: libc::pollfd = libc::pollfd { fd: descriptor, events: libc::POLLOUT, revents: 0 };
        // SAFETY: `poll` reads and writes the one `pollfd` it is given, and writes nothing else.
        let ready: libc::c_int = unsafe { libc::poll(std::ptr::from_mut(&mut watched), 1, milliseconds) };
        if ready == 0 {
            return Err(no_connection_in_time());
        }
        if ready < 0 {
            let failed: io::Error = io::Error::last_os_error();
            // A signal that ended the wait is not the connection's answer: the wait goes on with what is left of the deadline.
            if failed.kind() == io::ErrorKind::Interrupted && Instant::now() < deadline {
                continue;
            }
            return Err(failed);
        }
        return Ok(());
    }
}

/// What:
///  The failure of a socket that took no connection inside the deadline.
/// Why:
///   A caller reads this as "no answer", which is a service that is stopped or behind and never a faulty path,
///  so the words say what ended the wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noConnectionInTime = () => new Error('the socket took no connection within the deadline');
/// ```
fn no_connection_in_time() -> io::Error {
    return io::Error::new(io::ErrorKind::TimedOut, "the socket took no connection within the wait it was given");
}

/// What:
///  The failure a connected socket carries,
///  and nothing when the connection was made.
/// Why:
///   Writable means the attempt is over,
///  not that it worked:
///  a refused connection answers `POLLOUT` too,
///  and the reason sits in `SO_ERROR` until it is read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const carried = getsockopt(fd, SOL_SOCKET, SO_ERROR); if (carried) throw errno(carried);
/// ```
fn carried_failure(descriptor: libc::c_int) -> io::Result<()> {
    let mut carried: libc::c_int = 0;
    let mut length: libc::socklen_t = std::mem::size_of::<libc::c_int>() as libc::socklen_t;
    // SAFETY: `getsockopt` writes at most `length` bytes into `carried`, which is a `c_int` and that length is its own size.
    let asked: libc::c_int = unsafe { libc::getsockopt(descriptor, libc::SOL_SOCKET, libc::SO_ERROR, std::ptr::from_mut(&mut carried).cast::<libc::c_void>(), std::ptr::from_mut(&mut length)) };
    if asked != 0 {
        return Err(io::Error::last_os_error());
    }
    if carried != 0 {
        return Err(io::Error::from_raw_os_error(carried));
    }
    return Ok(());
}

/// What:
///  Clears the socket's non-blocking flag.
/// Why:
///   Everything above is written as if the socket blocks,
///  because the socket's flags say so:
///  a read that a caller bounded with a timeout would otherwise answer "would block" at once,
///  and the line it is waiting for would read as a service that said nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// fcntl(fd, F_SETFL, fcntl(fd, F_GETFL) & ~O_NONBLOCK);
/// ```
fn become_blocking(descriptor: libc::c_int) -> io::Result<()> {
    // SAFETY: `fcntl` with `F_GETFL` reads the descriptor's status flags and touches no memory.
    let flags: libc::c_int = unsafe { libc::fcntl(descriptor, libc::F_GETFL) };
    if flags < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `fcntl` with `F_SETFL` sets the flags it is given on this process's descriptor.
    let cleared: libc::c_int = unsafe { libc::fcntl(descriptor, libc::F_SETFL, flags & !libc::O_NONBLOCK) };
    if cleared < 0 {
        return Err(io::Error::last_os_error());
    }
    return Ok(());
}

/// The connection and its deadline are controlled in the test module.
#[cfg(test)]
#[path = "unix_socket_connect_within_a_deadline_tests.rs"]
mod tests;
