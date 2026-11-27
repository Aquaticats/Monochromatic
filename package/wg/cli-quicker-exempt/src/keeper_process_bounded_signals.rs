//! The bounded waits and the signals a stop is made of:
//! the reap,
//! the poll that ends at a deadline,
//! and the one signal that goes to a validated process group.

/// The start time that tells one process from the next at the same pid.
use super::process_identity::process_start_time;
/// Whether an error means the process is gone.
use crate::process_info::means_vanished;
/// Standard process-file and syscall failures.
use std::io;
/// Bounded shutdown polling interval.
use std::time::{
    Duration,
    Instant,
};

/// Maximum time allowed for a validated holder to disappear.
pub(super) const STOP_WAIT: Duration = Duration::from_secs(5);
/// Poll cadence balances quick teardown with bounded procfs reads.
pub(super) const STOP_POLL: Duration = Duration::from_millis(20);
/// Additional confirmation window after forced termination.
pub(super) const KILL_WAIT: Duration = Duration::from_secs(5);

/// Reaps holder when current process happens to be its parent.
fn reap_if_child(pid: i32) -> io::Result<bool> {
    let mut status: libc::c_int = 0;
    // SAFETY: status points to writable integer and WNOHANG never blocks.
    let result = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) };
    if result == pid {
        return Ok(true);
    }
    if result == 0 {
        return Ok(false);
    }
    let error = io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::ECHILD) {
        return Ok(false);
    }
    return Err(error);
}

/// Waits bounded interval for exact process to disappear or change identity.
fn wait_for_disappearance(
    pid: i32,
    start_time: u64,
    duration: Duration,
) -> io::Result<bool> {
    let deadline = Instant::now() + duration;
    while Instant::now() < deadline {
        if reap_if_child(pid)? {
            return Ok(true);
        }
        match process_start_time(pid) {
            Ok(observed_start) if observed_start == start_time => {
                std::thread::sleep(STOP_POLL);
            }
            Ok(_) => return Ok(true),
            Err(error) if means_vanished(&error) => return Ok(true),
            Err(error) => return Err(error),
        }
    }
    return Ok(false);
}

/// Whether a process group still has a member.
pub(super) fn group_has_members(group: i32) -> bool {
    // SAFETY: signal zero to a negative pid only asks the kernel whether the group exists.
    let result = unsafe { libc::kill(-group, 0) };
    if result == 0 {
        return true;
    }
    return io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH);
}

/// Signals a validated process,
/// and the process group it leads when it leads one,
/// because the watcher and the keeper each start their clients and their `ip` calls as children of a session they lead.
/// A signal to the leader alone leaves those children running,
/// and an orphaned manager client can still change the user's manager after the stop returned.
pub(super) fn signal_validated(
    pid: i32,
    group: Option<i32>,
    signal: libc::c_int,
) -> io::Result<()> {
    let target = match group {
        Some(leader) => -leader,
        None => pid,
    };
    // SAFETY: the caller validated the identity immediately before this call.
    let result = unsafe { libc::kill(target, signal) };
    if result < 0 {
        let error = io::Error::last_os_error();
        // A group whose leader already went has nothing left to signal.
        if (group.is_some()) && (error.raw_os_error() == Some(libc::ESRCH)) {
            return Ok(());
        }
        return Err(error);
    }
    return Ok(());
}

/// Waits bounded interval for the process to disappear and,
/// when it leads a group,
/// for that group to hold no member either.
pub(super) fn wait_for_stop(
    pid: i32,
    start_time: u64,
    group: Option<i32>,
    duration: Duration,
) -> io::Result<bool> {
    let gone = wait_for_disappearance(pid, start_time, duration)?;
    if !gone {
        return Ok(false);
    }
    let Some(leader) = group else {
        return Ok(true);
    };
    let deadline = Instant::now() + duration;
    while group_has_members(leader) {
        if Instant::now() >= deadline {
            return Ok(false);
        }
        std::thread::sleep(STOP_POLL);
    }
    return Ok(true);
}

/// Waits bounded time for a process group to hold no member,
/// and answers whether it became empty.
pub(super) fn wait_for_group_end(
    leader: i32,
    limit: Duration,
) -> bool {
    let deadline = Instant::now() + limit;
    while group_has_members(leader) {
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(STOP_POLL);
    }
    return true;
}

/// Sends one signal to a process group.
pub(super) fn signal_group(
    leader: i32,
    signal: libc::c_int,
) -> io::Result<()> {
    // SAFETY: a negative pid names the process group the caller proved.
    let result = unsafe { libc::kill(-leader, signal) };
    if result < 0 {
        let error = io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ESRCH) {
            return Ok(());
        }
        return Err(error);
    }
    return Ok(());
}
