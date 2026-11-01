//! Descriptors for shutdown signals:
//! a signalfd the poll loop waits on,
//! and a zero-timeout check a long pass uses to see that a signal is waiting.

/// Stop check trait.
use crate::routing_support::StopCheck;
/// Polling and syscall failures.
use std::io;
/// Raw and owned descriptor support.
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

/// Tells a long pass that a shutdown signal is waiting.
pub struct SignalPending<'a> {
    /// The descriptor that receives the shutdown signals.
    pub signals: &'a OwnedFd,
}

/// The shutdown check.
impl StopCheck for SignalPending<'_> {
    /// Polls the signal descriptor without waiting.
    fn stop_requested(&self) -> bool {
        let mut descriptor = libc::pollfd {
            fd: self.signals.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: the descriptor structure is valid for the synchronous zero-timeout poll.
        let result = unsafe { libc::poll(&mut descriptor, 1, 0) };
        return result > 0 && descriptor.revents & libc::POLLIN != 0;
    }
}

/// Creates owned descriptor from successful nonnegative syscall return.
pub fn owned_descriptor(raw: libc::c_int) -> io::Result<OwnedFd> {
    if raw < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: successful syscall returned a new descriptor owned by caller.
    return Ok(unsafe { OwnedFd::from_raw_fd(raw) });
}

/// Blocks termination signals and returns signalfd receiving them.
pub fn create_signal_fd() -> io::Result<OwnedFd> {
    // SAFETY: zeroed signal set is initialized immediately by sigemptyset.
    let mut mask: libc::sigset_t = unsafe { std::mem::zeroed() };
    // SAFETY: mask points to writable signal set.
    if unsafe { libc::sigemptyset(&mut mask) } < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: mask remains initialized and writable.
    if unsafe { libc::sigaddset(&mut mask, libc::SIGTERM) } < 0
        || unsafe { libc::sigaddset(&mut mask, libc::SIGINT) } < 0
    {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: blocks selected signals in dedicated watcher process before child threads exist.
    let mask_result = unsafe {
        libc::pthread_sigmask(libc::SIG_BLOCK, &mask, std::ptr::null_mut())
    };
    if mask_result != 0 {
        return Err(io::Error::from_raw_os_error(mask_result));
    }
    // SAFETY: signalfd copies mask during call and returns newly owned descriptor.
    let raw = unsafe {
        libc::signalfd(
            -1,
            &mask,
            libc::SFD_CLOEXEC | libc::SFD_NONBLOCK,
        )
    };
    return owned_descriptor(raw);
}

/// Reads one signalfd record and confirms shutdown event was delivered.
pub fn consume_signal(signals: &OwnedFd) -> io::Result<()> {
    // SAFETY: zeroed record is writable output storage for signalfd read.
    let mut signal: libc::signalfd_siginfo = unsafe { std::mem::zeroed() };
    // SAFETY: output pointer and length describe complete signalfd record.
    let count = unsafe {
        libc::read(
            signals.as_raw_fd(),
            std::ptr::addr_of_mut!(signal).cast(),
            std::mem::size_of::<libc::signalfd_siginfo>(),
        )
    };
    if count < 0 {
        return Err(io::Error::last_os_error());
    }
    return Ok(());
}
