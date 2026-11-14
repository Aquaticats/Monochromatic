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

use std::os::unix::net::UnixListener;
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
/// taking over one that no process holds.
/// Why:
///   A service that was killed leaves its socket file behind,
/// and the next bind of the same path then fails with
///  "Address already in use" although nothing answers there.
/// What proves that no process holds it is the kernel's own refusal of a connection:
///  a socket file another process still has bound accepts a connection whether or not that process is willing to greet
///  on it.
/// A silence is never that proof.
///   The review of `53726ade2d` reproduced, with the real binary,
/// a second server taking the address of a service that was alive and stopped
///  mid-instruction,
/// and the first one's own cleanup then unlinking the second's:
/// two live services for one worktree,
/// and a note that named neither.
///  A service is silent for legitimate reasons:
/// it is recovering the batches of an earlier service before its acceptor starts
///  (`service_main.rs`, seconds),
/// the machine starves its acceptor,
/// or a debugger stops it.
/// So a socket that accepts is left to its service, and the bind fails;
/// the command that wanted a server waits,
/// bounded,
/// and starts one when the address is free
///  (`service_server_start_by_the_wrapper.rs`,
/// which does not spawn a server while something accepts at the address).
///  The cost is the wait that the earlier takeover was written to avoid: a service that is leaving keeps its address until
///  it has,
/// which the commit's own ceiling bounds.
///
/// What:
///  The takeover is one process's at a time, and the refusal is asked again while that process holds it.
/// Why:
///   The question and the two system calls after it are not one step:
///  two servers that both asked and both were refused both unlinked and bound, and the second unlink removed the
///  first's live socket (the review of `76a08c9fac`, finding 3).
///  The claim the kernel grants one process at a time (`service_address_takeover_claim.rs`) is what makes the three
///  one step,
///  and asking again under it is what keeps a refusal that another taker acted on from being acted on here as well.
///  A process that finds the claim held leaves the address alone:
///  the holder is binding a server there now, and this one would only unlink it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { return listen(path) } catch (inUse) { const claim = tryClaim(address); if (!claim.held || !refusesConnections(path)) throw inUse; unlink(path); return listen(path) }
/// ```
pub(crate) fn bind_path(path: &std::path::Path) -> std::io::Result<UnixListener> {
    return bind_path_seamed(path, &mut do_nothing);
}

/// The call a takeover makes between its question and its unlink:
/// none.
fn do_nothing() {}

/// `bind_path` with a call between the refusal and the unlink, where a control may let another server act.
/// A control holds one server there and shows that the other leaves the address alone.
pub(crate) fn bind_path_seamed(path: &std::path::Path, after_the_probe: &mut dyn FnMut()) -> std::io::Result<UnixListener> {
    use std::os::unix::fs::FileTypeExt;
    let first: std::io::Error = match UnixListener::bind(path) {
        Ok(listener) => return Ok(listener),
        Err(error) if error.kind() == std::io::ErrorKind::AddrInUse => error,
        Err(error) => return Err(error),
    };
    let _claim: crate::service_address_takeover_claim::AddressTakeoverClaim = match crate::service_address_takeover_claim::claim(path) {
        Ok(crate::service_address_takeover_claim::Claim::Held(held)) => held,
        // What: Another process is taking this address over right now, so this one takes nothing.
        // Why:  It is mid-takeover: it has asked, and it will unlink and bind. Unlinking now would remove the socket it binds.
        Ok(crate::service_address_takeover_claim::Claim::AnotherProcessTakesOver) => return Err(first),
        // A claim that cannot be made is no takeover: the address stays as it is, and the caller's own wait covers it.
        Err(_cannot_claim) => return Err(first),
    };
    let is_socket: bool = match std::fs::symlink_metadata(path) {
        Ok(meta) => meta.file_type().is_socket(),
        Err(_missing) => false,
    };
    // What: The kernel's refusal of a connection is the only proof that no process holds this address.
    // Why:  A socket that accepts belongs to a live process, whatever it says or does not say: taking it over unlinked the
    //       address of a running service and let a second one bind the same path (the review's finding 1).
    // What: The probe is made within `CONNECT_CEILING`, and only a refusal is a socket nothing holds.
    // Why:  A socket whose backlog is full takes no connection until its program accepts, so a plain connect here waits for a
    //       stopped service to move; and a connect that only queued is not a refusal, so nothing is taken over.
    let refused: bool = matches!(
        crate::unix_socket_connect_within_a_deadline::connect_within(path, crate::wait_ceilings::CONNECT_CEILING),
        Err(error) if error.kind() == std::io::ErrorKind::ConnectionRefused
    );
    if !is_socket || !refused {
        return Err(first);
    }
    after_the_probe();
    std::fs::remove_file(path)?;
    return UnixListener::bind(path);
}

/// Controls of the address text stay out of the release executable.
#[cfg(test)]
#[path = "service_server_socket_binding_tests.rs"]
mod tests;
