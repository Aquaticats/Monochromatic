//! What:
//!  The descriptor the signal relay sends through:
//!  made once for the process,
//!  pointed at the child that is being waited for,
//!  and the pidfd that names that child.
//! Why:
//!  The relay's handler runs asynchronously,
//! so it can only read state through atomics and through one number that is never closed
//! (`signal_relay::RELAY_FD`).
//!  `dup3` replaces what that number refers to in one step,
//! so the handler never sees a closed number and never a descriptor of another process,
//! and a pidfd names the child for good,
//! so a signal cannot reach a process that took its number later.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! ensureRelayDescriptor(); pointRelayAt(pidfdOf(child.pid));
//! ```

/// The error the calls here answer with.
use std::io;
/// The number `c_int` names in `dup3`.
use std::os::raw::c_int;

/// What:
///  Make the descriptor number the relay signals through,
/// once for the process.
/// Why:
///   `/dev/null` stands on the number until a child is published;
/// the number is never closed (see `RELAY_FD`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// relayDescriptor ??= openDevNull();
/// ```
#[cfg(target_os = "linux")]
pub(crate) fn ensure_relay_descriptor() -> io::Result<()> {
    if crate::signal_relay::RELAY_FD.get().is_some() {
        return Ok(());
    }
    let nothing: std::os::fd::OwnedFd = rustix::fs::open("/dev/null", rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::CLOEXEC, rustix::fs::Mode::empty())
        .map_err(io::Error::from)?;
    let number: std::os::fd::RawFd = std::os::fd::IntoRawFd::into_raw_fd(nothing);
    // A guard that lost the race closes its own descriptor; the winner's number is the only one that is kept.
    if crate::signal_relay::RELAY_FD.set(number).is_err() {
        // SAFETY: the number was just made here and was not published, so nothing else holds or reads it.
        drop(unsafe { <std::os::fd::OwnedFd as std::os::fd::FromRawFd>::from_raw_fd(number) });
    }
    return Ok(());
}

/// What:
///  Make the relay descriptor name `child` (a pidfd opened for it),
/// or `/dev/null` when `child` is nothing.
/// Why:
///   `dup3` replaces what the number refers to in one step,
/// so the handler never sees a closed number and never a descriptor of
///  another process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pointRelayAt(child);
/// ```
#[cfg(target_os = "linux")]
pub(crate) fn point_relay_at(child: Option<&std::os::fd::OwnedFd>) -> io::Result<()> {
    let Some(target) = crate::signal_relay::RELAY_FD.get() else {
        return Err(io::Error::other("the relay descriptor was never made"));
    };
    let nothing: std::os::fd::OwnedFd;
    let source: std::os::fd::RawFd = match child {
        Some(pidfd) => std::os::fd::AsRawFd::as_raw_fd(pidfd),
        None => {
            nothing = rustix::fs::open("/dev/null", rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::CLOEXEC, rustix::fs::Mode::empty())
                .map_err(io::Error::from)?;
            std::os::fd::AsRawFd::as_raw_fd(&nothing)
        }
    };
    // SAFETY: `dup3` takes two descriptor numbers and no memory; `source` is open for this call and `target` is the process's own
    //  never-closed number. `O_CLOEXEC` keeps the child from inheriting the copy.
    let replaced: c_int = unsafe { libc::dup3(source, *target, libc::O_CLOEXEC) };
    if replaced < 0 {
        return Err(io::Error::last_os_error());
    }
    return Ok(());
}

/// A pidfd for the process with this number,
/// or nothing when the system gives none.
#[cfg(target_os = "linux")]
pub(crate) fn pidfd_of(pid: rustix::process::Pid) -> Option<std::os::fd::OwnedFd> {
    return rustix::process::pidfd_open(pid, rustix::process::PidfdFlags::empty()).ok();
}
