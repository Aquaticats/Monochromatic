//! Stopping one command,
//! and ending the process group a watcher left behind.

/// The waits and the signals of a stop.
use super::bounded_signals::{
    KILL_WAIT,
    STOP_WAIT,
    group_has_members,
    signal_group,
    signal_validated,
    wait_for_group_end,
    wait_for_stop,
};
/// The identity a signal rests on.
use super::process_identity::{
    command_process_is_live,
    expected_command,
    process_group_of,
    process_start_time,
    validate_command_identity,
};
/// Persisted process identity and socket mark.
use crate::keeper_state::KeeperProcess;
/// Whether an error means the process is gone.
use crate::process_info::means_vanished;
/// Standard process-file and syscall failures.
use std::io;
/// Cgroup identity passed to holder child.
use std::path::Path;

/// Stops exact command after typed validation and bounded signal escalation.
/// When the process leads its own process group,
/// which is how the watcher and the keeper are started,
/// the signal goes to the group so the children it started end with it.
pub fn stop_command_process(
    pid: i32,
    start_time: u64,
    expected: &[Vec<u8>],
    role: &str,
) -> io::Result<()> {
    validate_command_identity(pid, start_time, expected, role)?;
    let group = match process_group_of(pid)? {
        own if own == pid => Some(pid),
        _other => None,
    };
    signal_validated(pid, group, libc::SIGTERM)?;
    if wait_for_stop(pid, start_time, group, STOP_WAIT)? {
        return Ok(());
    }
    if command_process_is_live(pid, start_time, expected, role)? {
        validate_command_identity(pid, start_time, expected, role)?;
    }
    // SAFETY: identity was revalidated immediately before forced termination,
    // and a group whose leader is gone is the group that leader left.
    signal_validated(pid, group, libc::SIGKILL)?;
    if wait_for_stop(pid, start_time, group, KILL_WAIT)? {
        return Ok(());
    }
    return Err(io::Error::new(
        io::ErrorKind::TimedOut,
        format!("{role} PID {pid} survived SIGTERM and SIGKILL"),
    ));
}

/// What ending the group of a watcher whose leader is gone did.
#[derive(Debug, Eq, PartialEq)]
pub enum LeftoverGroup {
    /// The group was gone already,
    /// so nothing was signalled.
    Absent,
    /// The group held a member and ended after the signals sent.
    Ended,
}

/// Ends the process group a watcher led when the watcher itself is gone.
///
/// A watcher leads its own session and starts its `ip` calls and manager clients in that group.
/// When the kernel kills the watcher alone,
/// which an out-of-memory kill does,
/// those children stay running in the recorded group,
/// and a stop that only looks for the leader finds nothing and would forget the record,
/// although a client left alive can still change the user's manager.
/// The group is ended before the record goes.
///
/// The group is signalled only when it is proven to be the recorded one:
/// while a process group exists the kernel keeps its number,
/// so the leader's pid cannot name another process meanwhile,
/// and a live process at that pid with the recorded start time is the leader itself.
/// A different start time means the record names something else,
/// and nothing is signalled.
///
/// @throws {@link io::Error} with `InvalidData` when the pid names another process,
/// and with `TimedOut` when the group survives `SIGKILL`.
pub fn end_leftover_group(
    pid: i32,
    start_time: u64,
    role: &str,
) -> io::Result<LeftoverGroup> {
    if !group_has_members(pid) {
        return Ok(LeftoverGroup::Absent);
    }
    match process_start_time(pid) {
        Ok(observed) if observed == start_time => {}
        Ok(_other) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "{role} PID {pid} now belongs to another process, so the group it leads is not signalled"
                ),
            ));
        }
        Err(error) if means_vanished(&error) => {}
        Err(error) => return Err(error),
    }
    signal_group(pid, libc::SIGTERM)?;
    if wait_for_group_end(pid, STOP_WAIT) {
        return Ok(LeftoverGroup::Ended);
    }
    signal_group(pid, libc::SIGKILL)?;
    if wait_for_group_end(pid, KILL_WAIT) {
        return Ok(LeftoverGroup::Ended);
    }
    return Err(io::Error::new(
        io::ErrorKind::TimedOut,
        format!(
            "the process group of {role} PID {pid} outlived SIGTERM and SIGKILL, so its record is kept"
        ),
    ));
}

/// Sends SIGTERM,
///  escalates exact surviving holder to SIGKILL,
///  and confirms disappearance.
pub fn stop_process(
    process: KeeperProcess,
    cgroup_dir: &Path,
) -> io::Result<()> {
    let expected = expected_command(process, cgroup_dir)?;
    return stop_command_process(process.pid, process.start_time, &expected, "keeper");
}
