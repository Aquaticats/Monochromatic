//! What:
//!  What a child does to itself before it runs its steps:
//!  join a process group of its own,
//!  ask for a parent-death signal,
//!  recheck the parent that signal follows,
//!  move its two pipe ends below its sweep and close every descriptor above
//!  them,
//!  and end in `_exit`.
//! Why:
//!  Every one of these is a rule the design states for the probe's children and
//!  for the companion
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The pipes and parent-death signals each child owns").
//!  A child that skipped the sweep holds a pipe or a lock its parent closed,
//!  a child that skipped the recheck misses a parent that died before the
//!  signal was set,
//!  and a child that returned through the parent's frames would run a second
//!  copy of the mode.
//!  The sweep keeps the two ends at 3 and 4,
//!  which is the order every child follows.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! setpgid(0, 0); prctl(PR_SET_PDEATHSIG, SIGKILL); if (getppid() === 1) _exit(1);
//! dup2(control, 3); dup2(status, 4); closeRange(5); _exit(0);
//! ```

use super::probe_call_error::{CallError, failed};
use std::os::fd::RawFd;

/// The lowest descriptor a child keeps after its sweep,
/// its two pipe ends moved to 3 and 4 first.
pub const CHILD_FIRST_KEPT_DESCRIPTOR: RawFd = 5;
/// The descriptor a child's control read end is moved to.
pub const CHILD_CONTROL_DESCRIPTOR: RawFd = 3;
/// The descriptor a child's status write end is moved to.
pub const CHILD_STATUS_DESCRIPTOR: RawFd = 4;

/// What:
///  Put this process in a process group of its own,
///  from the child's side.
/// Why:
///  The parent's call covers a child that died at once;
///  this one covers a child whose parent had not reached its call yet.
///
/// # Errors
/// The call refused.
pub fn join_own_process_group() -> Result<(), CallError> {
    // SAFETY: `setpgid` reads two process identifiers and touches no memory of ours.
    let set: libc::c_int = unsafe { libc::setpgid(0, 0) };
    return if set == 0 { Ok(()) } else { Err(failed("setpgid")) };
}

/// What:
///  Have the kernel kill this child when its parent's thread dies.
/// Why:
///  A wrapper killed outright must take its child with it,
///  so the child's namespace and every mount inside it go,
///  and the signal follows the forking thread rather than the process.
///
/// # Errors
/// The call refused.
pub fn parent_death_signal() -> Result<(), CallError> {
    // SAFETY: `prctl` reads its four words and writes nothing of ours.
    let set: libc::c_int = unsafe { libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL, 0, 0, 0) };
    return if set == 0 { Ok(()) } else { Err(failed("prctl PR_SET_PDEATHSIG")) };
}

/// The process that started this one,
/// which a child rechecks after its parent-death signal.
pub fn parent_id() -> libc::pid_t {
    // SAFETY: `getppid` takes no argument and reads one number.
    return unsafe { libc::getppid() };
}

/// What:
///  Close every descriptor numbered `first` or higher.
/// Why:
///  The runtime,
/// a caller and the mode itself may hold descriptors a child
///  inherited,
///  and a copy of a pipe end or a lock descriptor in a child that outlives its
///  parent holds that pipe or lock open.
///
/// # Errors
/// The call refused.
pub fn close_from(first: RawFd) -> Result<(), CallError> {
    // SAFETY: `close_range` closes descriptors this process holds and touches no memory of ours.
    let closed: libc::c_int = unsafe { libc::syscall(libc::SYS_close_range, first as libc::c_uint, libc::c_uint::MAX, 0) as libc::c_int };
    return if closed == 0 { Ok(()) } else { Err(failed("close_range")) };
}

/// What:
///  Move one descriptor to another number,
///  keeping the source number closed.
/// Why:
///  A child's two pipe ends must survive its sweep,
///  so they are moved below the first descriptor the sweep closes.
///
/// # Errors
/// The call refused.
pub fn move_descriptor(from: RawFd, to: RawFd) -> Result<(), CallError> {
    // SAFETY: `dup2` reads two descriptor numbers and touches no memory of ours.
    let moved: libc::c_int = unsafe { libc::dup2(from, to) };
    if moved < 0 {
        return Err(failed("dup2"));
    }
    if from != to {
        // SAFETY: the source descriptor is this process's own and is closed exactly once; the copy at `to` stands.
        let _closed: libc::c_int = unsafe { libc::close(from) };
    }
    return Ok(());
}

/// What:
///  End this child at once,
///  never through the parent's frames.
/// Why:
///  The release profile unwinds a panic,
///  and a child that returned would run a second copy of the mode.
pub fn exit_now(status: i32) -> ! {
    // SAFETY: `_exit` ends this process and returns nothing.
    unsafe { libc::_exit(status) };
}


/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_child_prelude_tests.rs"]
mod tests;
