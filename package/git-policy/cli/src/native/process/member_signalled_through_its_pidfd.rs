//! What:
//!  Signalling one member of a process tree,
//!  through a descriptor opened while the number still named it.
//! Why:
//!  Reading a process's start time and sending its signal are two operations,
//! and a number can pass to another process between them
//!  (`process_tree::identity_of`):
//! the signal would then end a command of the person's that has nothing to do with the stop.
//!  A pidfd is made while the number is proven to name the member (its start time read before and after the open),
//! and `pidfd_send_signal` reaches the process that descriptor names or fails.
//!  A member the kernel refuses a descriptor for is not signalled at all:
//! a signal by number after a proof is still a signal to a number,
//! and the process the proof named can end and its number pass to another command before the call
//! (finding 2 of the second review).
//!  Each call also answers what the signal came to (`Delivered`),
//! so a caller records a stop only when a signal was really taken,
//! and keeps its ceiling armed when nothing could be proven.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const member of tree) member.pinned().signalIfStillThatProcess('SIGKILL');
//! ```

/// The member's number and start time,
/// and the tree it was read from.
use crate::process_tree::TreeMember;

/// What:
///  What a signal came to.
/// Why:
///   A stop is recorded only when a signal was really sent,
/// so a member the kernel refused to signal is a different answer from one it took.
///  A member that could not be proven at all is a third answer:
/// it says nothing about whether the process is still there,
/// so a caller must not read it as an end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Delivered = 'sent' | 'gone' | 'unproven';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Delivered {
    /// The kernel took the signal for the process that was pinned.
    Sent,
    /// No process has that number any more,
    /// so nothing was sent.
    Gone,
    /// Nothing was sent, and nothing is known about the member:
    /// the kernel refused a descriptor,
    /// or the signal cannot be named to it.
    Unproven,
}

/// What:
///  One member of a tree,
/// held with the descriptor its signal travels through:
/// the process that was found,
/// and never one that takes its number later.
/// Why:
///   Reading a process's start time and sending its signal are two operations,
/// and a number can pass to another process between them,
/// which would put the signal into a command of the person's that has nothing to do with the stop.
/// A pidfd is made while the number is proven to name the found process,
/// and `pidfd_send_signal` then reaches the process that descriptor names or fails,
/// so no interval is left for a reuse to fall into.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pinned = PinnedMember.open(found); if (pinned.kind === 'held') pinned.signal('SIGKILL');
/// ```
#[cfg(target_os = "linux")]
pub struct PinnedMember {
    /// The descriptor the kernel made for the process the tree found.
    descriptor: std::os::fd::OwnedFd,
}

/// What:
///  What pinning a member came to.
/// Why:
///   Three answers must stay apart:
/// the member is held by a descriptor and can be signalled;
/// the number names no process any more, so the member is gone;
/// and the member could not be proven,
/// so no signal may be sent and no end may be concluded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Pinned = { kind: 'held'; member: PinnedMember } | { kind: 'gone' } | { kind: 'unproven' };
/// ```
#[cfg(target_os = "linux")]
pub enum Pinned {
    /// The member, held by a descriptor that names it for good.
    Held(PinnedMember),
    /// No process has that number any more.
    Gone,
    /// The member could not be proven:
    /// the kernel refused a descriptor, so nothing may be sent.
    Unproven,
}

// What:
//  A control that refuses the descriptors `pidfd_open` would make,
// as a descriptor or memory limit does with `EMFILE` and `ENFILE`.
// Why:
//   The refusal cannot be produced on demand here:
//   a limit that refuses `pidfd_open` refuses the reads of `/proc` too,
//   so the kernel's answer is injected instead
//   (`child_awaited_through_a_pidfd.rs` injects the same failure the same way).
#[cfg(all(test, target_os = "linux"))]
thread_local! {
    /// How many descriptors the next opens are refused.
    static DESCRIPTOR_REFUSALS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Refuse the next `count` descriptors the kernel would make.
#[cfg(all(test, target_os = "linux"))]
pub(crate) fn refuse_the_next_descriptors(count: usize) {
    DESCRIPTOR_REFUSALS.with(|left| { left.set(count); });
}

/// Take one refusal, when one is armed.
#[cfg(all(test, target_os = "linux"))]
fn a_descriptor_is_refused() -> bool {
    let left: usize = DESCRIPTOR_REFUSALS.with(std::cell::Cell::get);
    if left == 0 {
        return false;
    }
    DESCRIPTOR_REFUSALS.with(|left| { left.set(left.get().saturating_sub(1)); });
    return true;
}

/// The descriptor the kernel makes for `pid`,
/// or nothing where it refuses one at a limit or under a sandbox.
#[cfg(target_os = "linux")]
fn pidfd_of(pid: i32) -> Option<std::os::fd::OwnedFd> {
    #[cfg(test)]
    if a_descriptor_is_refused() {
        return None;
    }
    let process: rustix::process::Pid = rustix::process::Pid::from_raw(pid)?;
    return rustix::process::pidfd_open(process, rustix::process::PidfdFlags::empty()).ok();
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "member_signalled_through_its_pidfd_tests.rs"]
mod tests;

#[cfg(target_os = "linux")]
impl PinnedMember {
    /// What:
    ///  Pin `member`:
    /// its start time is read,
    /// the descriptor is opened,
    /// and its start time is read again.
    /// `Gone` when the number names another process or none,
    /// and `Unproven` when the kernel refuses a descriptor for it.
    /// Why:
    ///   Both reads must answer the recorded start time for the descriptor to be that process's:
    /// a number that came back would have to be handed to the process that held it before.
    ///  A member without a descriptor is never signalled:
    /// the last proof and the signal would be two operations again,
    /// which is the interval a descriptor exists to close.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const pinned = PinnedMember.open(found);
    /// ```
    pub fn open(member: &TreeMember) -> Pinned {
        if crate::process_tree::identity_of(member.pid) != Some((member.started, false)) {
            return Pinned::Gone;
        }
        let Some(descriptor) = pidfd_of(member.pid) else {
            return Pinned::Unproven;
        };
        // The number named the member before and after the descriptor was made, so the descriptor is that process's.
        if crate::process_tree::identity_of(member.pid) != Some((member.started, false)) {
            return Pinned::Gone;
        }
        return Pinned::Held(Self { descriptor });
    }

    /// Send `signal` to the pinned process,
    /// and to no other.
    pub fn signal(&self, signal: i32) -> Delivered {
        let Some(named) = rustix::process::Signal::from_named_raw(signal) else {
            // A signal this build cannot name to the kernel: nothing is sent, and a real-time signal is the only case.
            return Delivered::Unproven;
        };
        return match rustix::process::pidfd_send_signal(&self.descriptor, named) {
            Ok(()) => Delivered::Sent,
            // The kernel took the descriptor but the process it names has ended: nothing of the person's was reached.
            Err(_) => Delivered::Gone,
        };
    }
}

/// What:
///  Send `signal` to each member that is still the process that was found;
/// the others are left alone.
/// Answers what each signal came to,
/// in the order of the members.
/// Why:
///   A number that was reused by an unrelated process must never be signalled,
/// and the signal travels through a descriptor opened while the number was proven to name the member.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const member of tree) member.signal('SIGKILL');
/// ```
#[cfg(target_os = "linux")]
pub fn signal_members(members: &[TreeMember], signal: i32) -> Vec<Delivered> {
    let mut reported: Vec<Delivered> = Vec::with_capacity(members.len());
    for member in members {
        let delivered: Delivered = match PinnedMember::open(member) {
            Pinned::Held(pinned) => pinned.signal(signal),
            Pinned::Gone => Delivered::Gone,
            Pinned::Unproven => Delivered::Unproven,
        };
        reported.push(delivered);
    }
    return reported;
}

/// Where the kernel has no pidfd at all,
/// every member is unproven:
/// a signal by number after a proof is the race this module exists to remove,
/// and a platform without descriptors cannot close it.
/// Owner decision, 2026-11-09: "Ignore Win and mac until I say so."
// UNTESTED: no gate compiles or runs this branch, and the owner keeps macOS and Windows out of scope.
#[cfg(all(unix, not(target_os = "linux")))]
pub fn signal_members(members: &[TreeMember], _signal: i32) -> Vec<Delivered> {
    return vec![Delivered::Unproven; members.len()];
}

/// Platforms without signals stop only the root.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
pub fn signal_members(_members: &[TreeMember], _signal: i32) -> Vec<Delivered> {
    return Vec::new();
}

/// Ask each member that is still the process that was found to end,
/// with `SIGTERM`,
/// answering what each signal came to.
#[cfg(unix)]
pub fn terminate_members(members: &[TreeMember]) -> Vec<Delivered> {
    return signal_members(members, libc::SIGTERM);
}

/// Platforms without signals stop only the root.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
pub fn terminate_members(_members: &[TreeMember]) -> Vec<Delivered> {
    return Vec::new();
}

/// End each member that is still the process that was found,
/// with `SIGKILL`,
/// answering what each signal came to.
#[cfg(unix)]
pub fn kill_members(members: &[TreeMember]) -> Vec<Delivered> {
    return signal_members(members, libc::SIGKILL);
}

/// Platforms without signals stop only the root.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
pub fn kill_members(_members: &[TreeMember]) -> Vec<Delivered> {
    return Vec::new();
}
