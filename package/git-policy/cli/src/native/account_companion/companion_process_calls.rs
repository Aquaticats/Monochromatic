//! What:
//!  The process calls the companion's parent makes:
//!  the pipe,
//!  the fork with its bounded retry,
//!  the child's process group,
//!  the pid descriptor it waits on,
//!  the kill that ends a group at the deadline,
//!  and the ending a wait answers.
//! Why:
//!  The parent is the add's own process,
//!  which is not a thread the runtime polls,
//!  and its whole wait is one `poll` on the child's pid descriptor bounded by
//!  the companion's deadline
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The account companion",
//! "Its deadline").
//!  A child a signal cannot end is handed to the crate's reaper for a child the
//!  starter holds only as a number, rather than waited on here
//!  (`process/stuck_children_reaped_when_the_kernel_lets_go.rs`,
//!  `reap_soon_pid`), which is that file's rule for a command's child applied
//!  to the raw pid this module forked.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [read, write] = pipe(); const pid = fork(); setpgid(pid, pid);
//! poll(pidfd, deadline) ? collect(pid) : { kill(-pid, SIGKILL); reapSoonPid(pid); };
//! ```

use super::companion_calls::CompanionCall;
use std::os::fd::{
    AsRawFd,
    FromRawFd,
    OwnedFd,
};
use std::time::Duration;

/// How a child ended,
/// as one wait answered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Exit = { kind: 'exited'; status: number } | { kind: 'signaled'; signal: number } | { kind: 'running' };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Exit {
    /// The child exited with a status.
    Exited {
        /// The status.
        status: i32,
    },
    /// The child died of a signal.
    Signaled {
        /// The signal's number.
        signal: i32,
    },
    /// The child has not ended.
    StillRunning,
}

/// What:
///  The status pipe of one child,
///  both ends close-on-exec.
///
/// # Errors
/// The call refused,
///  and the refusal names it.
pub fn pipe_pair() -> Result<(OwnedFd, OwnedFd), std::io::Error> {
    let mut ends: [libc::c_int; 2] = [0; 2];
    // SAFETY: the array is this call's own and the call writes its two descriptors.
    let made: libc::c_int = unsafe { libc::pipe2(ends.as_mut_ptr(), libc::O_CLOEXEC) };
    if made != 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: the call answered two descriptors this process owns, each owned exactly once.
    return Ok(unsafe { (OwnedFd::from_raw_fd(ends[0]), OwnedFd::from_raw_fd(ends[1])) });
}

/// What:
///  Fork once.
/// Why:
///  The answer is which side of the fork this copy is,
///  which is the one shape that cannot be mistaken.
///
/// # Errors
/// The system refused the fork,
///  which the caller retries within the wrapper's budget.
pub fn fork() -> Result<Option<libc::pid_t>, std::io::Error> {
    // SAFETY: `fork` takes no argument and the child continues in a copy of this process.
    let pid: libc::pid_t = unsafe { libc::fork() };
    if pid < 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(if pid == 0 { None } else { Some(pid) });
}

/// What:
///  Put the child in a process group of its own,
///  from the parent's side,
///  so the deadline's kill reaches it even before its own call.
///
/// # Errors
/// The call refused.
pub fn set_process_group(pid: libc::pid_t) -> Result<(), std::io::Error> {
    // SAFETY: `setpgid` reads two process identifiers and touches no memory of ours.
    return if unsafe { libc::setpgid(pid, pid) } == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    };
}

/// What:
///  The pid descriptor of one child,
///  which `poll` makes readable when that child ends.
///
/// # Errors
/// The call refused.
pub fn pid_descriptor(pid: libc::pid_t) -> Result<OwnedFd, std::io::Error> {
    // SAFETY: `pidfd_open` reads two words and answers a descriptor this process owns.
    let opened: libc::c_int = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0) as libc::c_int };
    if opened < 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: the call answered a descriptor this process owns, and it is owned exactly once.
    return Ok(unsafe { OwnedFd::from_raw_fd(opened) });
}

/// What:
///  Wait,
///  bounded,
///  for one descriptor to be ready.
/// Why:
///  The parent's whole wait is this one call,
///  so nothing here holds a thread the runtime polls.
///
/// # Errors
/// The call refused.
pub fn ready_within(
    descriptor: &OwnedFd,
    within: Duration,
) -> Result<bool, std::io::Error> {
    let milliseconds: libc::c_int =
        libc::c_int::try_from(within.as_millis().max(1)).unwrap_or(libc::c_int::MAX);
    let mut watched: libc::pollfd = libc::pollfd {
        fd: descriptor.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    // SAFETY: `watched` is one live entry, which is what the count says.
    let ready: libc::c_int = unsafe { libc::poll(&raw mut watched, 1, milliseconds) };
    if ready < 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(ready > 0);
}

/// What:
///  Pause for a bounded while,
///  which the fork's retry does between its attempts.
/// Why:
///  A refused fork is answered by pausing and asking again,
///  and a pause is a sleep rather than a spin.
pub fn pause(within: Duration) {
    let milliseconds: libc::c_int =
        libc::c_int::try_from(within.as_millis().max(1)).unwrap_or(libc::c_int::MAX);
    let mut watched: [libc::pollfd; 0] = [];
    // SAFETY: a poll of no descriptor with a timeout sleeps and reads nothing of ours.
    let _slept: libc::c_int = unsafe { libc::poll(watched.as_mut_ptr(), 0, milliseconds) };
}

/// What:
///  Every byte the status pipe holds at this moment,
///  read once into the buffer.
/// Why:
///  A step's record is eight bytes,
///  and a full run writes less than one pipe holds,
///  so one read after the child ended takes them all.
///
/// # Errors
/// The read refused.
pub fn read_available(
    descriptor: &OwnedFd,
    into: &mut [u8],
) -> Result<usize, std::io::Error> {
    // SAFETY: the buffer holds the room the call is given.
    let read: libc::ssize_t = unsafe {
        libc::read(
            descriptor.as_raw_fd(),
            into.as_mut_ptr().cast::<libc::c_void>(),
            into.len(),
        )
    };
    if read < 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(usize::try_from(read).unwrap_or(0));
}

/// What:
///  Take one child's ending without waiting when it has not ended.
///
/// # Errors
/// The call refused.
pub fn reap_now(pid: libc::pid_t) -> Result<Exit, std::io::Error> {
    let mut status: libc::c_int = 0;
    // SAFETY: the status word is this call's own and `waitpid` writes it.
    let waited: libc::pid_t = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) };
    if waited < 0 {
        return Err(std::io::Error::last_os_error());
    }
    if waited == 0 {
        return Ok(Exit::StillRunning);
    }
    return Ok(exit_from(status));
}

/// The ending one wait status holds.
fn exit_from(status: libc::c_int) -> Exit {
    if libc::WIFSIGNALED(status) {
        return Exit::Signaled {
            signal: libc::WTERMSIG(status),
        };
    }
    return Exit::Exited {
        status: libc::WEXITSTATUS(status),
    };
}

/// What:
///  Kill one child's whole process group,
///  which is how the deadline ends a child and everything it started.
///
/// # Errors
/// The call refused.
pub fn kill_group(pid: libc::pid_t) -> Result<(), std::io::Error> {
    // SAFETY: `kill` reads two numbers and touches no memory of ours, and the group is this child's own.
    return if unsafe { libc::kill(-pid, libc::SIGKILL) } == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    };
}

/// What:
///  Whether one process of this namespace is alive,
///  which the scratch sweep reads before it removes anything.
/// Why:
///  `ESRCH` is the kernel's answer for a pid that names nothing here,
///  and the sweep removes only what that answer proves dead.
pub fn is_alive(pid: libc::pid_t) -> bool {
    // SAFETY: `kill` with no signal reads two numbers and touches no memory of ours.
    let answered: libc::c_int = unsafe { libc::kill(pid, 0) };
    if answered == 0 {
        return true;
    }
    // SAFETY: `__errno_location` answers this thread's own error number, which the call above has just set.
    return unsafe { *libc::__errno_location() } == libc::EPERM;
}

/// This process's effective user and group,
/// which the mapping lines name.
pub fn effective_ids() -> (u32, u32) {
    // SAFETY: both calls read this process's own identity and have no preconditions.
    return unsafe { (libc::geteuid(), libc::getegid()) };
}

/// The name a refused call is logged under,
/// which is this module's own call vocabulary.
pub const COMPANION_START_CALL: CompanionCall = CompanionCall::Fork;
