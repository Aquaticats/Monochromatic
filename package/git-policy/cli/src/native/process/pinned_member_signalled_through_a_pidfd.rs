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
//!  A process whose record cannot be read is never signalled:
//! nothing about the read proves the number still names the member,
//!  so it may have been given to an unrelated process.
//! The stop waits for such a process as if it were alive,
//! and says in one line that it could not be
//!  stopped.
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
use crate::process_tree::{ProcessLook, TreeMember, look_of_through};
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
/// pinned by a descriptor that names it for good.
/// Why:
///  A signal sent through the descriptor reaches this process or nothing;
/// a number checked a moment earlier cannot
///  promise that.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Pinned = { descriptor: number };
/// ```
#[cfg(unix)]
#[derive(Debug)]
pub struct PinnedProcess {
    /// The pidfd.
    /// It names one process for good,
    /// and it is the only way this module ever signals something.
    #[cfg(target_os = "linux")]
    descriptor: OwnedFd,
}

/// What:
///  What a pin came to.
/// Why:
///   The three answers a look at a record gives have three different signals:
/// a pinned process is signalled through its
///  descriptor,
/// one that is gone is never signalled,
/// and one whose record could not be read is never signalled either,
///  because nothing proves which process its number names by now.
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
    /// so nothing proves what the number names:
    /// nothing is signalled,
    /// and the stop waits for the process instead.
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
    return pin_after_two_looks_of(proc_root, proc_root, member);
}

/// What:
///  `pin_of_through` where the two looks read two record roots,
/// so a control can make the second look fail without exhausting this process's descriptors:
///  the first reads the real record,
/// and the second reads a root where that record refuses.
/// Why:
///   The record's own directory is read twice,
/// before and after the descriptor is opened,
/// so the number is proven to have
///  named one process throughout;
/// only the two answers together decide whether the descriptor is kept,
/// and a control
///  that could only make both fail could not tell the two answers apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pinAfterTwoLooksOf(first: string, second: string, member: Member): PinOutcome;
/// ```
#[cfg(all(unix, target_os = "linux"))]
fn pin_after_two_looks_of(first: &std::path::Path, second: &std::path::Path, member: &TreeMember) -> PinOutcome {
    let started: u64 = match look_of_through(first, member.pid) {
        ProcessLook::Alive(found) if found.started == member.started => found.started,
        ProcessLook::Alive(_) | ProcessLook::Gone => return PinOutcome::Gone,
        ProcessLook::Unknown => return PinOutcome::Unreadable,
    };
    // SAFETY: `pidfd_open` takes two integers and returns a descriptor or -1; it touches no memory of this process.
    let raw: libc::c_long = unsafe { libc::syscall(libc::SYS_pidfd_open, libc::c_long::from(member.pid), 0 as libc::c_long) };
    if raw < 0 {
        // The process went between the read and the call,
        // the number is not a process,
        // or this host has no descriptor to give (an old kernel,
        //  a sandbox, or a descriptor limit).
        // Either way nothing here can name the process for good, so nothing is signalled.
        return PinOutcome::Gone;
    }
    let Ok(number) = i32::try_from(raw) else {
        return PinOutcome::Gone;
    };
    // SAFETY: `raw` is a descriptor `pidfd_open` just made for this process alone, and nothing else owns it.
    let descriptor: OwnedFd = unsafe { OwnedFd::from_raw_fd(number) };
    return match look_of_through(second, member.pid) {
        // The number named the process before and after the descriptor was made, so the descriptor is that process's.
        ProcessLook::Alive(found) if found.started == started => PinOutcome::Held(PinnedProcess { descriptor }),
        ProcessLook::Alive(_) | ProcessLook::Gone => PinOutcome::Gone,
        // The descriptor was opened after the first look,
        // and a second look that fails does not establish that the number still
        //  named the process the first look found:
        // between the two, that process can have ended and been reaped, and the number
        //  can name an unrelated process the descriptor then names.
        // Nothing is signalled.
        ProcessLook::Unknown => PinOutcome::Gone,
    };
}

/// What a pin of a member came to on this host.
#[cfg(all(unix, target_os = "linux"))]
pub fn pin_of(member: &TreeMember) -> PinOutcome {
    return pin_of_through(std::path::Path::new(crate::process_tree::PROC), member);
}

/// Without `/proc` nothing is known about a number,
/// so no pin is made and nothing is signalled.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(all(unix, not(target_os = "linux")))]
pub fn pin_of(_member: &TreeMember) -> PinOutcome {
    return PinOutcome::Unreadable;
}

/// Methods over a pinned process.
#[cfg(unix)]
impl PinnedProcess {
    /// Send `signal` to the pinned process,
    /// and to no other.
    pub fn signal(&self, signal: c_int) -> Delivered {
        // SAFETY: `pidfd_send_signal` takes a descriptor, a signal number, a null `siginfo` and flags; it touches no memory.
        let result: libc::c_long = unsafe {
            libc::syscall(
                libc::SYS_pidfd_send_signal,
                libc::c_long::from(self.descriptor.as_raw_fd()),
                libc::c_long::from(signal),
                0 as libc::c_long,
                0 as libc::c_long,
            )
        };
        return if result == 0 { Delivered::Sent } else { Delivered::Gone };
    }
}

/// What:
///  Send `signal` to the member through a pin of it,
/// or nothing when the member is gone or its record could not be read.
/// Why:
///   One decision for every caller,
/// so the root of a step and the members below it cannot be treated
///  differently.
/// A record that could not be read says nothing about the process,
/// so nothing is signalled through it:
///  the number named a member a moment ago,
/// and it may name an unrelated process by now,
/// which no signal
///  may ever reach.
/// The stop waits for such a member instead (`still_running`),
/// and one line says it could
///  not be stopped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pin = pinOf(member); if (pin.kind === 'held') pin.pin.signal(sig); else return 'gone';
/// ```
#[cfg(unix)]
pub fn signal_member(member: &TreeMember, signal: c_int) -> Delivered {
    let pin: PinOutcome = pin_of(member);
    return delivered_by(pin, signal);
}

/// What:
///  `signal_member` over a pin the caller made,
/// so a control can read a record that refuses without starving the whole test process.
/// Why:
///  The whole decision of a signal is the pin's answer,
/// and the refusal is a branch no real `/proc` gives a control
///  on demand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function deliveredBy(pin: PinOutcome, signal: number): Delivered;
/// ```
#[cfg(unix)]
fn delivered_by(pin: PinOutcome, signal: c_int) -> Delivered {
    return match pin {
        PinOutcome::Held(pinned) => pinned.signal(signal),
        PinOutcome::Gone | PinOutcome::Unreadable => Delivered::Gone,
    };
}

/// Pin controls stay out of the release executable.
#[cfg(test)]
#[path = "pinned_member_signalled_through_a_pidfd_tests.rs"]
mod tests;
