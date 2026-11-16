//! What:
//!  Connects to the commit service's address and gives up when the socket takes no connection within the bound.
//! Why:
//!   The standard library's connect waits in the kernel with no end of its own when the listening socket's queue of
//! unaccepted connections is full,
//! which is what a service that stopped accepting connections leaves behind,
//! and the owner's
//!  rule is that nothing waits for ever (`wait_ceilings.rs`, `CONNECT_CEILING`).
//! The address a service listens at is one of two shapes,
//! a file name or a Linux abstract name,
//! and both become the
//!  same address structure for the kernel,
//! so the one type here builds either.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const socket = connectWithin(address); // an error when the socket takes nothing within 5 s
//! ```

use crate::service_server::Bind;
use crate::wait_ceilings::CONNECT_CEILING;
use std::io::ErrorKind;
use std::os::fd::{FromRawFd, RawFd};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::{Duration, Instant};

/// The shortest wait the kernel is asked for,
/// so a bound that is spent still names a wait it accepts.
const SHORTEST_WAIT: Duration = Duration::from_millis(1);

/// One address of a local socket,
/// how long the connect to it is waited for,
/// and what a failure names it with.
struct Address {
    /// The address structure the kernel reads,
    /// all zero bytes where nothing is written.
    structure: libc::sockaddr_un,
    /// How many bytes of `structure` belong to the address:
    /// the family and the name,
    /// and never the whole structure,
    /// which is longer than a name.
    length: libc::socklen_t,
    /// The longest the socket may take to accept the connection.
    within: Duration,
    /// What a failure names the address with.
    named: String,
}

impl Address {
    /// A socket named by a file.
    fn path(path: &Path, within: Duration) -> Self {
        use std::os::unix::ffi::OsStrExt as _;
        return Self::named(path.as_os_str().as_bytes(), within, path.display().to_string(), false);
    }

    /// A socket named by the kernel,
    /// whose name is not a file and is not ended by a zero byte.
    #[cfg(target_os = "linux")]
    fn abstract_name(name: &str, within: Duration) -> Self {
        return Self::named(name.as_bytes(), within, format!("{name}, a name the kernel holds rather than a file"), true);
    }

    /// A socket whose name is `bytes`,
    /// in the structure that holds one name,
    /// which the kernel keeps rather than a file when `kernel_named` says so.
    fn named(bytes: &[u8], within: Duration, named: String, kernel_named: bool) -> Self {
        // SAFETY: `sockaddr_un` is a plain structure of integers and a byte array, for which all zero bytes are a valid value:
        //  the family says which kind of address it is, and the name is written after this.
        let mut structure: libc::sockaddr_un = unsafe { std::mem::zeroed() };
        structure.sun_family = libc::sa_family_t::try_from(libc::AF_UNIX).unwrap_or(0);
        // A name the kernel holds starts in the second byte, and the first stays zero; a name that is a file starts in the
        //  first and the last stays zero, which is where a name of that kind ends.
        let start: usize = usize::from(kernel_named);
        let ending: usize = usize::from(!kernel_named);
        let room: usize = structure.sun_path.len().saturating_sub(start + ending);
        let taken: usize = bytes.len().min(room);
        for (slot, byte) in structure.sun_path.iter_mut().skip(start).zip(bytes.iter()).take(taken) {
            *slot = libc::c_char::try_from(*byte).unwrap_or(0);
        }
        let offset: usize = size_of::<libc::sa_family_t>();
        let length: libc::socklen_t = libc::socklen_t::try_from(offset + start + taken + ending).unwrap_or(0);
        return Self { structure, length, within, named };
    }

    /// What:
    ///  Connects,
    /// and gives up when the socket takes nothing within the bound.
    /// Why:
    ///   A connect that waits for a queue nothing empties is the wait this file exists to bound,
    /// so it is asked on a socket that does not block and waited for here,
    /// where the clock is.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// address.connect() // a connected socket, or an error naming the address
    /// ```
    fn connect(self) -> std::io::Result<UnixStream> {
        // SAFETY: `socket` takes three integers and touches no memory of this process.
        let descriptor: RawFd = unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_STREAM | libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK, 0) };
        if descriptor < 0 {
            return Err(refused(&self.named, std::io::Error::last_os_error()));
        }
        if let Err(error) = self.ask(descriptor) {
            close(descriptor);
            return Err(error);
        }
        // The wait is over, so the socket is put back in the blocking mode the steps after this one write in: a request asks
        //  its own bound for every send it makes, and a line that a whole `write_all` carries waits in the kernel with no end
        //  on a socket that blocks.
        if let Err(error) = set_blocking(descriptor, &self.named) {
            close(descriptor);
            return Err(error);
        }
        // SAFETY: the descriptor is a connected socket this function owns, and it is handed to the stream alone.
        return Ok(unsafe { UnixStream::from_raw_fd(descriptor) });
    }

    /// What:
    ///  The connect itself, and the wait for the socket to take it,
    ///  retried while the queue of a listening socket has no room.
    /// Why:
    ///   On a socket that does not block, a connect into a queue nothing accepts returns `EAGAIN` and does not
    /// happen at all, and a peer that stops accepting leaves the poll that follows reporting the socket as writable:
    /// asking the kernel whether a peer is there is what keeps an unconnected socket from being handed on as connected
    /// (the control `a_queue_nothing_accepts_ends_the_wait_at_its_bound` caught exactly that).
    /// So the connect is asked again after each wait, and the answer is proved by `getpeername` before it is given.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// while (true) { const rc = connect(); if (rc === 0) return proveConnected(); if (!retryable(errno)) throw refused(); if (expired()) throw timedOut(); await poll(POLLOUT, left) }
    /// ```
    fn ask(&self, descriptor: RawFd) -> std::io::Result<()> {
        let until: Instant = Instant::now() + self.within.max(SHORTEST_WAIT);
        loop {
            // SAFETY: `descriptor` is an open socket, and the structure with its length is one address the call reads and does
            //  not write.
            let asked: libc::c_int = unsafe { libc::connect(descriptor, (&raw const self.structure).cast(), self.length) };
            if asked == 0 {
                return self.proved(descriptor);
            }
            let error: std::io::Error = std::io::Error::last_os_error();
            match error.raw_os_error() {
                // A second connect of a socket that is already connected says so.
                Some(libc::EISCONN) => return self.proved(descriptor),
                // No room in the queue, a connect under way, or a signal that came first: each is waited for below, and the
                //  ask is made again, because a connect into a full queue has connected nothing.
                Some(libc::EAGAIN | libc::EINPROGRESS | libc::EALREADY | libc::EINTR) => {}
                _ => return Err(refused(&self.named, error)),
            }
            let left: Duration = until.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return Err(std::io::Error::new(ErrorKind::TimedOut, format!(
                    "the socket at {} took no connection within {} ms, so nothing there is accepting connections",
                    self.named,
                    self.within.as_millis()
                )));
            }
            let mut waiting: libc::pollfd = libc::pollfd { fd: descriptor, events: libc::POLLOUT, revents: 0 };
            let millis: libc::c_int = libc::c_int::try_from(left.as_millis().max(1)).unwrap_or(libc::c_int::MAX);
            // SAFETY: `waiting` is one valid `pollfd` for the call, which reads and writes only that.
            let ready: libc::c_int = unsafe { libc::poll(&raw mut waiting, 1, millis) };
            if ready < 0 {
                let failed: std::io::Error = std::io::Error::last_os_error();
                if failed.kind() == ErrorKind::Interrupted {
                    continue;
                }
                return Err(refused(&self.named, failed));
            }
            // Ready or not, the connect is asked again: only it can say whether a peer took this socket.
        }
    }

    /// What:
    ///  Says that the socket has a peer,
    ///  or fails naming the address.
    /// Why:
    ///   A socket that was bound and never connected has no peer,
    /// and handing one on would make every later step fail with a message about a connection that was never made.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// getpeername(fd).errno === 0 ? ok() : throw refused()
    /// ```
    fn proved(&self, descriptor: RawFd) -> std::io::Result<()> {
        // SAFETY: `sockaddr_un` is a plain structure of integers and a byte array, for which all zero bytes are a valid value;
        //  this call only writes it, with the length it was given.
        let mut peer: libc::sockaddr_un = unsafe { std::mem::zeroed() };
        let mut length: libc::socklen_t = libc::socklen_t::try_from(size_of::<libc::sockaddr_un>()).unwrap_or(0);
        // SAFETY: the two pointers are this function's own variables, and the length passed is their size.
        let asked: libc::c_int = unsafe { libc::getpeername(descriptor, (&raw mut peer).cast(), &raw mut length) };
        if asked < 0 {
            return Err(refused(&self.named, std::io::Error::last_os_error()));
        }
        return Ok(());
    }
}

/// What:
///  Connects to the commit service's address,
/// waiting no longer than `CONNECT_CEILING`.
/// Why:
///   A socket that takes no connection within the time a service that is working may stay silent is one nothing
/// accepts on,
/// and the wait for it must end so the command can say so:
/// the failure names the
///  address,
/// as a connect that is refused does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// connectWithin(address) // an error when the socket takes nothing within 5 s
/// ```
pub(crate) fn connect_within(address: &Bind) -> std::io::Result<UnixStream> {
    return connect_within_for(address, CONNECT_CEILING);
}

/// What:
///  `connect_within`, waiting no longer than `within`.
/// Why:
///   A caller whose own answer is waited for with a shorter bound (a status command) connects within that bound,
/// and
///  a control gives a bound of milliseconds so it ends in seconds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// connectWithinFor(address, 200) // a control's bound
/// ```
pub(crate) fn connect_within_for(address: &Bind, within: Duration) -> std::io::Result<UnixStream> {
    let addressed: Address = match address {
        #[cfg(target_os = "linux")]
        Bind::Abstract(name) => Address::abstract_name(name, within),
        #[cfg(not(target_os = "linux"))]
        Bind::Abstract(_) => return Err(std::io::Error::other("abstract sockets need Linux")),
        Bind::Path(path) => Address::path(path, within),
        Bind::Inherited => return Err(std::io::Error::other("a client does not inherit the socket")),
    };
    return addressed.connect();
}

/// The error a refused connect ends with,
/// naming the address the service was looked for at.
fn refused(named: &str, error: std::io::Error) -> std::io::Error {
    return std::io::Error::new(error.kind(), format!("the socket at {named} refused the connection: {error}"));
}

/// Closes a descriptor this file opened and did not hand on.
fn close(descriptor: RawFd) {
    // SAFETY: `close` takes one descriptor and touches no memory of this process.
    let _closed = unsafe { libc::close(descriptor) };
}

/// Puts a connected socket back in the blocking mode,
/// so a step that writes a whole line at once keeps the behaviour it had.
fn set_blocking(descriptor: RawFd, named: &str) -> std::io::Result<()> {
    // SAFETY: `fcntl` with `F_GETFL` takes a descriptor and returns its flags without touching memory.
    let flags: libc::c_int = unsafe { libc::fcntl(descriptor, libc::F_GETFL) };
    if flags < 0 {
        return Err(refused(named, std::io::Error::last_os_error()));
    }
    // SAFETY: as above, with `F_SETFL` and a flag word.
    let set: libc::c_int = unsafe { libc::fcntl(descriptor, libc::F_SETFL, flags & !libc::O_NONBLOCK) };
    if set < 0 {
        return Err(refused(named, std::io::Error::last_os_error()));
    }
    return Ok(());
}

/// Controls of the bound stay out of the release executable.
#[cfg(test)]
#[path = "service_client_connect_within_a_bound_tests.rs"]
mod tests;
