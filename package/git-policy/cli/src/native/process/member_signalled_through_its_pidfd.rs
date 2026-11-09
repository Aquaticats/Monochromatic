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
//! Where the kernel has no pidfds the start time is read again right before the signal,
//! and the interval that leaves is the one a descriptor closes.
//!  Each call also answers what the signal came to (`Delivered`),
//! so a caller records a stop only when a signal was really taken.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const member of tree) member.pinned().signalIfStillThatProcess('SIGKILL');
//! ```

/// The member's number and start time,
/// and the tree it was read from.
use crate::process_tree::TreeMember;
/// Whether the number still names that member,
/// read again right before a signal where the kernel has no pidfd.
use crate::process_tree::still_running;

/// What:
///  What a signal came to.
/// Why:
///   A stop is recorded only when a signal was really sent,
/// so a member the kernel refused to signal is a different answer from one it took.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Delivered = 'sent' | 'gone';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Delivered {
    /// The kernel took the signal for the process that was found.
    Sent,
    /// The process is gone,
    /// or the number names another one now,
    /// so nothing was sent.
    Gone,
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
/// const pinned = PinnedMember.open(found); pinned?.signal('SIGKILL');
/// ```
#[cfg(target_os = "linux")]
pub struct PinnedMember {
    /// The number and start time the tree read.
    member: TreeMember,
    /// The descriptor the kernel made for that process,
    /// where it makes pidfds.
    descriptor: Option<std::os::fd::OwnedFd>,
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
    /// Nothing when the number names another process or none.
    /// Why:
    ///   Both reads must answer the recorded start time for the descriptor to be that process's:
    /// a number that came back would have to be handed to the process that held it before.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const pinned = PinnedMember.open(found);
    /// ```
    pub fn open(member: &TreeMember) -> Option<Self> {
        if crate::process_tree::identity_of(member.pid) != Some((member.started, false)) {
            return None;
        }
        let descriptor: Option<std::os::fd::OwnedFd> = pidfd_of(member.pid);
        // The number named the member before and after the descriptor was made, so the descriptor is that process's.
        if crate::process_tree::identity_of(member.pid) != Some((member.started, false)) {
            return None;
        }
        return Some(Self { member: member.clone(), descriptor });
    }

    /// Send `signal` to the pinned process,
    /// and to no other.
    pub fn signal(&self, signal: i32) -> Delivered {
        if let Some(descriptor) = &self.descriptor
            && let Some(named) = rustix::process::Signal::from_named_raw(signal)
        {
            return match rustix::process::pidfd_send_signal(descriptor, named) {
                Ok(()) => Delivered::Sent,
                // The kernel took the descriptor but the process it names has ended: nothing of the person's was reached.
                Err(_) => Delivered::Gone,
            };
        }
        // Without a descriptor the start time is read again right before the signal,
        // so the interval that remains is the one a descriptor closes.
        if !still_running(&self.member) {
            return Delivered::Gone;
        }
        // SAFETY: `kill` takes two integers and touches no memory of this process.
        let result: libc::c_int = unsafe { libc::kill(self.member.pid, signal) };
        return if result == 0 { Delivered::Sent } else { Delivered::Gone };
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
            Some(pinned) => pinned.signal(signal),
            None => Delivered::Gone,
        };
        reported.push(delivered);
    }
    return reported;
}

/// The same where the kernel has no pidfd:
/// the start time is read again right before each signal,
/// and the interval that leaves is the one a descriptor closes.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(all(unix, not(target_os = "linux")))]
pub fn signal_members(members: &[TreeMember], signal: i32) -> Vec<Delivered> {
    let mut reported: Vec<Delivered> = Vec::with_capacity(members.len());
    for member in members {
        if !still_running(member) {
            reported.push(Delivered::Gone);
            continue;
        }
        // SAFETY: `kill` takes no pointers; a failure is reported because the process may have ended since the check.
        let result: libc::c_int = unsafe { libc::kill(member.pid, signal) };
        reported.push(if result == 0 { Delivered::Sent } else { Delivered::Gone });
    }
    return reported;
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
