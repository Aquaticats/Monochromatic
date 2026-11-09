//! What:
//!  One process of a tree held by a descriptor that names it for good,
//! so a signal reaches that process and never a number that was used for another one.
//! Why:
//!  A process number is reused once its process is reaped,
//! so a signal sent to a number that was read a moment ago can end an unrelated
//!  process of the same account,
//! which is what the cargo-shadow review called a P1 ("the process-tree ownership heuristic can
//!  kill unrelated work").
//! `binary/binary_supervisor_pin.rs` is the same pattern for the hang guard of the binary tests:
//!  a pidfd names one process for good,
//! `pidfd_send_signal` on it reaches that process or fails,
//! and it never reaches a successor.
//!  A pin is made only after the start time read before the pidfd was opened equals the one read after,
//! which shows that the
//!  number named one process throughout.
//! Where the kernel gives no pidfd the signal goes to the number after the start time is read
//!  again right before it,
//! and that window is the one a pidfd closes.
//!  A process whose record cannot be read is signalled by its number all the same:
//! it was a member of the tree a moment ago,
//! and a
//!  stop that sends nothing because a look was refused ends nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pin = pinOf(member); // held unless the number names the same process throughout
//! pin.signal('SIGKILL');     // pidfd_send_signal: never reaches a successor of the process
//! ```

/// The C integer type signal numbers use.
#[cfg(unix)]
use std::ffi::c_int;
/// One process of a tree and the start time that proves it is still that process.
use crate::process_tree::{ProcessLook, TreeMember, look_of, look_of_through};
/// The descriptor that names one process until it is closed.
#[cfg(target_os = "linux")]
use std::os::fd::{AsRawFd as _, FromRawFd as _, OwnedFd};

/// What a signal came to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Delivered {
    /// The process received the signal.
    Sent,
    /// The process is gone,
    /// or is not the one that was pinned,
    /// so nothing was sent.
    Gone,
}

/// What:
///  One process,
/// pinned by a descriptor where the kernel gives one,
/// and by its number and start time otherwise.
/// Why:
///  A signal sent through the descriptor reaches this process or nothing;
/// a number checked a moment earlier cannot
///  promise that.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Pinned = { member: Member; descriptor?: number };
/// ```
#[cfg(unix)]
#[derive(Debug)]
pub struct PinnedProcess {
    /// The process and the start time it was pinned with.
    member: TreeMember,
    /// The pidfd,
    /// where the kernel has pidfds.
    #[cfg(target_os = "linux")]
    descriptor: Option<OwnedFd>,
}

/// What:
///  What a pin came to.
/// Why:
///   The three answers a look at a record gives have three different signals:
/// a pinned process is signalled through its
///  descriptor,
/// one that is gone is never signalled,
/// and one whose record could not be read is signalled by its number without an
///  identity check,
/// as the platform without `/proc` always did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PinOutcome = { kind: 'held'; pin: Pinned } | { kind: 'gone' } | { kind: 'unreadable' };
/// ```
#[cfg(unix)]
#[derive(Debug)]
pub enum PinOutcome {
    /// The number names the process that was found:
    /// it is held,
    /// by a pidfd where the kernel gives one.
    Held(PinnedProcess),
    /// The record says the process is gone,
    /// or is another process now:
    /// nothing is signalled.
    Gone,
    /// The record could not be read,
    /// so the process is signalled by its number.
    Unreadable,
}

/// What:
///  What a pin of process `pid` below `proc_root` came to.
/// Why:
///   The record's own directory is the seam a control uses to read a record that refuses,
/// so the unknown case is
///  proven without exhausting this process's descriptors.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pinOfThrough(procRoot: string, member: Member): PinOutcome;
/// ```
#[cfg(all(unix, target_os = "linux"))]
pub fn pin_of_through(proc_root: &std::path::Path, member: &TreeMember) -> PinOutcome {
    let started: u64 = match look_of_through(proc_root, member.pid) {
        ProcessLook::Alive(found) if found.started == member.started => found.started,
        ProcessLook::Alive(_) | ProcessLook::Gone => return PinOutcome::Gone,
        ProcessLook::Unknown => return PinOutcome::Unreadable,
    };
    // SAFETY: `pidfd_open` takes two integers and returns a descriptor or -1; it touches no memory of this process.
    let raw: libc::c_long = unsafe { libc::syscall(libc::SYS_pidfd_open, libc::c_long::from(member.pid), 0 as libc::c_long) };
    if raw < 0 {
        let error: std::io::Error = std::io::Error::last_os_error();
        // The process went between the read and the call, or the number is not a process.
        if error.raw_os_error() == Some(libc::ESRCH) {
            return PinOutcome::Gone;
        }
        // No pidfd on this kernel or under this sandbox: the pin signals by number after reading the start time again.
        return PinOutcome::Held(PinnedProcess { member: TreeMember { pid: member.pid, started }, descriptor: None });
    }
    let Ok(number) = i32::try_from(raw) else {
        return PinOutcome::Gone;
    };
    // SAFETY: `raw` is a descriptor `pidfd_open` just made for this process alone, and nothing else owns it.
    let descriptor: OwnedFd = unsafe { OwnedFd::from_raw_fd(number) };
    // The number named the process before and after the descriptor was made, so the descriptor is that process's.
    if look_of_through(proc_root, member.pid) != ProcessLook::Alive(TreeMember { pid: member.pid, started }) {
        return PinOutcome::Gone;
    }
    return PinOutcome::Held(PinnedProcess { member: TreeMember { pid: member.pid, started }, descriptor: Some(descriptor) });
}

/// What a pin of a member came to on this host.
#[cfg(all(unix, target_os = "linux"))]
pub fn pin_of(member: &TreeMember) -> PinOutcome {
    return pin_of_through(std::path::Path::new(crate::process_tree::PROC), member);
}

/// Without `/proc` nothing is known about a number,
/// so the pin signals by number as it always did.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(all(unix, not(target_os = "linux")))]
pub fn pin_of(_member: &TreeMember) -> PinOutcome {
    return PinOutcome::Unreadable;
}

/// Methods over a pinned process.
#[cfg(unix)]
impl PinnedProcess {
    /// What:
    ///  Whether the signal of this pin goes through a descriptor,
    /// which never reaches a successor of the process.
    /// Why:
    ///   A control reads it to show which of the two ways a signal takes,
    /// and the number-then-signal window it
    ///  closes is the reason this module exists.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// pin.heldByAPidfd;
    /// ```
    pub fn held_by_a_pidfd(&self) -> bool {
        #[cfg(target_os = "linux")]
        return self.descriptor.is_some();
        #[cfg(not(target_os = "linux"))]
        return false;
    }

    /// Send `signal` to the pinned process,
    /// and to no other.
    pub fn signal(&self, signal: c_int) -> Delivered {
        #[cfg(target_os = "linux")]
        if let Some(descriptor) = &self.descriptor {
            // SAFETY: `pidfd_send_signal` takes a descriptor, a signal number, a null `siginfo` and flags; it touches no memory.
            let result: libc::c_long = unsafe {
                libc::syscall(
                    libc::SYS_pidfd_send_signal,
                    libc::c_long::from(descriptor.as_raw_fd()),
                    libc::c_long::from(signal),
                    0 as libc::c_long,
                    0 as libc::c_long,
                )
            };
            return if result == 0 { Delivered::Sent } else { Delivered::Gone };
        }
        // Without a pidfd the start time is read again right before the signal: the window that remains is the pidfd's to close.
        if !matches!(look_of(self.member.pid), ProcessLook::Alive(found) if found.started == self.member.started) {
            return Delivered::Gone;
        }
        // SAFETY: `kill` takes two integers and touches no memory of this process.
        let result: c_int = unsafe { libc::kill(self.member.pid, signal) };
        return if result == 0 { Delivered::Sent } else { Delivered::Gone };
    }
}

/// What:
///  Send `signal` to the member through a pin of it,
/// or by its number when its record could not be read.
/// Why:
///   One decision for every caller,
/// so the root of a step and the members below it cannot be treated
///  differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pin = pinOf(member); if (pin.held) pin.signal(sig); else if (pin.unreadable) process.kill(member.pid, sig);
/// ```
#[cfg(unix)]
pub fn signal_member(member: &TreeMember, signal: c_int) -> Delivered {
    return match pin_of(member) {
        PinOutcome::Held(pinned) => pinned.signal(signal),
        PinOutcome::Gone => Delivered::Gone,
        PinOutcome::Unreadable => {
            // SAFETY: `kill` takes two integers and touches no memory of this process; the number named a member of the tree a
            //  moment ago and its record could not be read to say otherwise.
            let sent: c_int = unsafe { libc::kill(member.pid, signal) };
            if sent == 0 { Delivered::Sent } else { Delivered::Gone }
        }
    };
}

/// Pin controls stay out of the release executable.
#[cfg(test)]
#[path = "pinned_member_signalled_through_a_pidfd_tests.rs"]
mod tests;
