//! What:
//!  Gets the listening socket of the commit service:
//! the one the service manager passed in,
//! or a socket bound at a path.
//! Why:
//!  `serve` needs a listener before it can accept;
//! where it comes from depends on how the service was started.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const listener = bind(path ?? inheritedDescriptor);
//! ```

use std::os::unix::net::{UnixListener, UnixStream};
/// File descriptor the service manager passes the listening socket on (`SD_LISTEN_FDS_START`).
pub const INHERITED_DESCRIPTOR: i32 = 3;

/// Where the listening socket comes from.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Bind {
    /// A Linux abstract name:
    /// one binder at a time,
    /// gone with the process,
    /// no file to go stale.
    Abstract(String),
    /// A path in a private directory,
    /// for platforms without abstract names.
    Path(std::path::PathBuf),
    /// The socket the service manager opened and passed in.
    Inherited,
}

/// What:
///  The address a service is asked at,
/// as the text a person can act on.
/// Why:
///  A notice and a failure name the address,
/// which is the one fact that tells two worktrees' services apart;
/// an abstract name has no path a person can look at,
/// so it is said to be a name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// addressText({ path: '/run/user/1000/monochromatic-git-policy-cli/abc.sock' }) // that path
/// ```
pub fn address_text(address: &Bind) -> String {
    return match address {
        Bind::Abstract(name) => format!("the name {name}"),
        Bind::Path(path) => path.display().to_string(),
        Bind::Inherited => String::from("the one this service was started with"),
    };
}

/// What:
///  Opens the listening socket.
/// Why:
///   Socket activation hands over a bound socket,
/// which is the normal way;
/// the others serve tests and development runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listener = bind(address)
/// ```
pub fn bind(address: &Bind) -> std::io::Result<UnixListener> {
    return match address {
        #[cfg(target_os = "linux")]
        Bind::Abstract(name) => {
            use std::os::linux::net::SocketAddrExt;
            let socket = std::os::unix::net::SocketAddr::from_abstract_name(name.as_bytes())?;
            UnixListener::bind_addr(&socket)
        }
        #[cfg(not(target_os = "linux"))]
        Bind::Abstract(_) => Err(std::io::Error::other("abstract sockets need Linux")),
        Bind::Path(path) => bind_path(path),
        Bind::Inherited => {
            use std::os::fd::FromRawFd;
            ensure_socket_passed_in(std::env::var("LISTEN_PID").ok().as_deref(), std::env::var("LISTEN_FDS").ok().as_deref(), std::process::id())?;
            // SAFETY: the manager passed this descriptor to this process and nothing else in the process owns it.
            Ok(unsafe { UnixListener::from_raw_fd(INHERITED_DESCRIPTOR) })
        }
    };
}

/// Whether the service manager passed this process a listening socket:
/// the variables `LISTEN_PID` and `LISTEN_FDS` of this
///  process's environment name this process and at least one descriptor.
pub fn socket_passed_in() -> bool {
    return ensure_socket_passed_in(std::env::var("LISTEN_PID").ok().as_deref(), std::env::var("LISTEN_FDS").ok().as_deref(), std::process::id()).is_ok();
}

/// What:
///  Checks the service manager's socket-activation variables:
/// `listen_pid` must name this process (`own_pid`) and
///  `listen_fds` must count at least one descriptor.
/// Why:
///   The descriptor the manager passes is taken over as the listening socket,
/// so a variable set for another process,
/// or a
///  count of none,
/// must be refused instead of adopting a descriptor that was never passed to this process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (listenPid !== String(ownPid) || !(Number.parseInt(listenFds ?? '0') >= 1)) { throw new Error('no socket was passed in') }
/// ```
pub(crate) fn ensure_socket_passed_in(listen_pid: Option<&str>, listen_fds: Option<&str>, own_pid: u32) -> std::io::Result<()> {
    let announced_for: bool = matches!(listen_pid, Some(pid) if pid == own_pid.to_string());
    // A count that is missing or does not parse is none.
    let count: i32 = match listen_fds {
        Some(text) => text.parse::<i32>().unwrap_or(0),
        None => 0,
    };
    if !announced_for || count < 1 {
        return Err(std::io::Error::other("no socket was passed in"));
    }
    return Ok(());
}

/// What:
///  Binds a socket file,
/// taking over one that no service listens on.
/// Why:
///   A service that was killed leaves its socket file behind,
/// and the next bind of the same path then fails with
///  "Address already in use" although nothing answers there.
/// Only a socket that refuses a connection is replaced:
/// one that
///  answers belongs to a running service,
/// and any other file is the person's.
/// Two services started on one path at the same
///  moment are the caller's to avoid (the service manager's socket,
/// which production uses,
/// never comes here).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { return listen(path) } catch (inUse) { if (await refusesConnections(path)) { unlink(path); return listen(path) } throw inUse }
/// ```
pub(crate) fn bind_path(path: &std::path::Path) -> std::io::Result<UnixListener> {
    use std::os::unix::fs::FileTypeExt;
    let first: std::io::Error = match UnixListener::bind(path) {
        Ok(listener) => return Ok(listener),
        Err(error) if error.kind() == std::io::ErrorKind::AddrInUse => error,
        Err(error) => return Err(error),
    };
    let is_socket: bool = match std::fs::symlink_metadata(path) {
        Ok(meta) => meta.file_type().is_socket(),
        Err(_missing) => false,
    };
    let refused: bool = matches!(UnixStream::connect(path), Err(error) if error.kind() == std::io::ErrorKind::ConnectionRefused);
    if !is_socket || !refused {
        return Err(first);
    }
    std::fs::remove_file(path)?;
    return UnixListener::bind(path);
}

/// Controls of the address text stay out of the release executable.
#[cfg(test)]
#[path = "service_server_socket_binding_tests.rs"]
mod tests;
