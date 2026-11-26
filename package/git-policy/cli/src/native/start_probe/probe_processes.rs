//! What:
//!  The processes the probe runs beside itself:
//!  the fork,
//!  one pipe pair per child,
//!  the child's process group,
//!  its parent-death signal,
//!  the sweep that closes every descriptor it inherited,
//!  and the one bounded wait the mode spends on its worker.
//! Why:
//!  The design names each of these as the protocol between the mode,
//!  the worker,
//!  the holder and the unmount child,
//!  so no child inherits a channel meant for a sibling and an interruptible
//!  child does not outlive the thread that forked it
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The pipes and parent-death signals each child owns").
//!  The wait is one `poll` on the worker's pid descriptor,
//!  bounded by the deadline,
//!  so a wedged worker is killed at the deadline rather than polled with a
//!  clock.
//!  A child ends in `_exit` on every path:
//!  returning through the parent's frames would run a second copy of the mode.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [parentEnd, childEnd] = pipe(); const pid = fork();
//! if (pid === 0) { childEnd.moveTo(4); closeRange(5); _exit(0); }
//! ```

use super::probe_call_error::{
    CallError,
    failed,
};
use std::os::fd::{
    AsRawFd,
    FromRawFd,
    OwnedFd,
    RawFd,
};
use std::time::Duration;

/// What:
///  One pipe pair,
///  each end owned once.
/// Why:
///   A pipe created per child keeps a sibling from holding an end of another
///  child's channel,
///  and the ownership is what closes it when the holder of a value goes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PipeEnds = { read: OwnedFd; write: OwnedFd };
/// ```
#[derive(Debug)]
pub struct PipeEnds {
    /// The read end.
    pub read: OwnedFd,
    /// The write end.
    pub write: OwnedFd,
}

/// What:
///  One pipe pair,
///  both ends close-on-exec.
/// Why:
///  A pair whose ends a started child inherited would keep the channel open
///  after its owner closed it,
///  so end of file would never arrive.
///
/// # Errors
/// The call refused.
pub fn pipe_pair() -> Result<PipeEnds, CallError> {
    let mut ends: [libc::c_int; 2] = [0; 2];
    // SAFETY: the array is this call's own and the call writes its two descriptors.
    let made: libc::c_int = unsafe { libc::pipe2(ends.as_mut_ptr(), libc::O_CLOEXEC) };
    if made != 0 {
        return Err(failed("pipe2"));
    }
    // SAFETY: the call answered two descriptors this process owns, each owned exactly once.
    return Ok(PipeEnds {
        read: unsafe { OwnedFd::from_raw_fd(ends[0]) },
        write: unsafe { OwnedFd::from_raw_fd(ends[1]) },
    });
}

/// What:
///  Fork this process once.
/// Why:
///   Each child is one fork from the single-threaded mode,
///  and the answer is which side of it this copy is:
///  the parent gets the child's pid and the child gets nothing,
///  which is the one shape that cannot be mistaken.
///
/// # Errors
/// The system refused the fork,
///  which the wrapper's own start rule retries at a higher level.
pub fn fork() -> Result<Option<libc::pid_t>, CallError> {
    // SAFETY: `fork` takes no argument and the child continues in a copy of this process.
    let pid: libc::pid_t = unsafe { libc::fork() };
    if pid < 0 {
        return Err(failed("fork"));
    }
    return Ok(if pid == 0 { None } else { Some(pid) });
}

/// What:
///  Put one process in a process group of its own,
///  from the parent's side.
/// Why:
///  The group must exist even if the child dies before its own call,
///  so the deadline's `SIGKILL` to the group reaches the child and every
///  process it started.
///
/// # Errors
/// The call refused.
pub fn set_process_group(pid: libc::pid_t) -> Result<(), CallError> {
    // SAFETY: `setpgid` reads two process identifiers and touches no memory of ours.
    let set: libc::c_int = unsafe { libc::setpgid(pid, pid) };
    return if set == 0 {
        Ok(())
    } else {
        Err(failed("setpgid"))
    };
}

/// What:
///  The pid descriptor of one child,
///  which `poll` makes readable when that child ends.
/// Why:
///  It is the one wait that is bounded by a deadline without a clock:
///  a worker wedged in a kernel wait is not named by any pipe,
///  and its end is.
///
/// # Errors
/// The call refused.
pub fn pid_descriptor(pid: libc::pid_t) -> Result<OwnedFd, CallError> {
    // SAFETY: `pidfd_open` reads two words and answers a descriptor this process owns.
    let opened: libc::c_int = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0) as libc::c_int };
    if opened < 0 {
        return Err(failed("pidfd_open"));
    }
    // SAFETY: the call answered a descriptor this process owns, and it is owned exactly once.
    return Ok(unsafe { OwnedFd::from_raw_fd(opened) });
}

/// What:
///  Wait,
/// bounded,
/// for one of the descriptors to be ready.
/// Why:
///  The mode's whole loop is one `poll` per turn,
///  so no thread waits on a channel while the deadline passes.
///
/// # Errors
/// The call refused.
pub fn poll_descriptors(
    watched: &mut [libc::pollfd],
    timeout: Option<Duration>,
) -> Result<libc::c_int, CallError> {
    let milliseconds: libc::c_int = match timeout {
        None => -1,
        Some(left) => libc::c_int::try_from(left.as_millis().max(1)).unwrap_or(libc::c_int::MAX),
    };
    // SAFETY: the array is the caller's and holds its length; the call writes each entry's answer.
    let ready: libc::c_int = unsafe {
        libc::poll(
            watched.as_mut_ptr(),
            libc::nfds_t::try_from(watched.len()).unwrap_or(0),
            milliseconds,
        )
    };
    return if ready < 0 {
        Err(failed("poll"))
    } else {
        Ok(ready)
    };
}

/// What:
///  How a child ended,
///  as one wait answered.
/// Why:
///  A signal death and an exit status are different record lines,
///  and a child that is still running is not an answer at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Exit = { kind: 'exited'; status: number } | { kind: 'signaled'; signal: number } | { kind: 'still-running' };
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
///  Take one child's ending,
///  without waiting when it has not ended.
/// Why:
///  The mode reaps with `WNOHANG` so a missed deadline does not wait for a
///  worker a `SIGKILL` cannot interrupt,
///  and its one blocking wait is the pid descriptor rather than this call.
///
/// # Errors
/// The call refused.
pub fn reap_now(pid: libc::pid_t) -> Result<Exit, CallError> {
    let mut status: libc::c_int = 0;
    // SAFETY: the status word is this call's own and `waitpid` writes it.
    let waited: libc::pid_t = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) };
    if waited < 0 {
        return Err(failed("waitpid"));
    }
    if waited == 0 {
        return Ok(Exit::StillRunning);
    }
    return Ok(exit_from_status(status));
}

/// What:
///  Wait for one child to end,
///  however long its own steps take.
/// Why:
///  The unmount child is the worker's own and the worker has no clock:
///  the mode's deadline is what bounds this wait,
///  by killing the worker's whole process group.
///
/// # Errors
/// The call refused.
pub fn wait_blocking(pid: libc::pid_t) -> Result<Exit, CallError> {
    let mut status: libc::c_int = 0;
    loop {
        // SAFETY: the status word is this call's own and `waitpid` writes it.
        let waited: libc::pid_t = unsafe { libc::waitpid(pid, &mut status, 0) };
        if waited < 0 {
            let error: std::io::Error = std::io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::EINTR) {
                continue;
            }
            return Err(failed("waitpid"));
        }
        return Ok(exit_from_status(status));
    }
}

/// The ending one wait status holds.
pub fn exit_from_status(status: libc::c_int) -> Exit {
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
///  Kill one process group,
///  which is how the mode ends a wedged worker and every process it started.
/// Why:
///  A `SIGKILL` to the group reaches children a signal to one pid would leave,
///  and the group is the worker's own.
///
/// # Errors
/// The call refused.
pub fn kill_process_group(pid: libc::pid_t) -> Result<(), CallError> {
    // SAFETY: `kill` reads two numbers and touches no memory of ours; the group is the worker's own.
    let killed: libc::c_int = unsafe { libc::kill(-pid, libc::SIGKILL) };
    return if killed == 0 {
        Ok(())
    } else {
        Err(failed("kill"))
    };
}

/// What:
///  Kill one process,
///  which is how the mode ends the holder it owns.
/// Why:
///  The holder is the mode's own child and may be wedged,
///  so the deadline's answer to it is a `SIGKILL` rather than a stop byte.
///
/// # Errors
/// The call refused.
pub fn kill_process(pid: libc::pid_t) -> Result<(), CallError> {
    // SAFETY: `kill` reads two numbers and touches no memory of ours.
    let killed: libc::c_int = unsafe { libc::kill(pid, libc::SIGKILL) };
    return if killed == 0 {
        Ok(())
    } else {
        Err(failed("kill"))
    };
}

/// The descriptor a pipe's end is,
/// for a watch list or a call.
pub fn raw(descriptor: &OwnedFd) -> RawFd {
    return descriptor.as_raw_fd();
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_processes_tests.rs"]
mod tests;
