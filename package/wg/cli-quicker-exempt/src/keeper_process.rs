//! Validates descriptor-keeper process identity before lifecycle signals.

/// Persisted process identity and socket mark.
use crate::keeper_state::KeeperProcess;
/// Whether an error means the process is gone.
use crate::process_info::means_vanished;
/// Standard process-file and syscall failures.
use std::io;
/// Unix path bytes for exact `/proc` command-line comparison.
use std::os::unix::ffi::OsStrExt;
/// Cgroup identity passed to holder child.
use std::path::Path;
/// Bounded shutdown polling interval.
use std::time::{Duration, Instant};

/// Maximum time allowed for a validated holder to disappear.
const STOP_WAIT: Duration = Duration::from_secs(5);
/// Poll cadence balances quick teardown with bounded procfs reads.
const STOP_POLL: Duration = Duration::from_millis(20);
/// Additional confirmation window after forced termination.
const KILL_WAIT: Duration = Duration::from_secs(5);

/// Reads Linux process start-time field after safely skipping parenthesized command name.
pub fn process_start_time(pid: i32) -> io::Result<u64> {
    let text = std::fs::read_to_string(format!("/proc/{pid}/stat"))?;
    let Some(command_end) = text.rfind(')') else {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "process stat lacks command terminator"));
    };
    let Some(tail) = text.get(command_end + 1..) else {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "process stat tail is absent"));
    };
    let fields: Vec<&str> = tail.split_whitespace().collect();
    let Some(start_time) = fields.get(19) else {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "process stat lacks start time"));
    };
    return start_time.parse().map_err(io::Error::other);
}

/// Builds exact NUL-separated argument vector expected for one holder.
fn expected_command(
    process: KeeperProcess,
    cgroup_dir: &Path,
) -> io::Result<Vec<Vec<u8>>> {
    let executable = std::env::current_exe()?;
    let cgroup = cgroup_dir.as_os_str().as_bytes().to_vec();
    return Ok(vec![
        executable.as_os_str().as_bytes().to_vec(),
        b"__hold".to_vec(),
        process.mark.to_string().into_bytes(),
        cgroup,
    ]);
}

/// Reports the NUL byte that separates command-line arguments.
fn is_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// Reads command-line arguments without interpreting path bytes as text.
fn process_command(pid: i32) -> io::Result<Vec<Vec<u8>>> {
    let bytes = std::fs::read(format!("/proc/{pid}/cmdline"))?;
    let mut arguments: Vec<Vec<u8>> = bytes
        .split(is_nul_byte)
        .map(<[u8]>::to_vec)
        .collect();
    if arguments.last().is_some_and(Vec::is_empty) {
        arguments.pop();
    }
    return Ok(arguments);
}

/// Confirms PID start time and exact byte-level command arguments.
pub fn validate_command_identity(
    pid: i32,
    start_time: u64,
    expected: &[Vec<u8>],
    role: &str,
) -> io::Result<()> {
    let observed_start = process_start_time(pid)?;
    if observed_start != start_time {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "{role} PID {pid} start time changed: expected {start_time}, observed {observed_start}"
            ),
        ));
    }
    let observed_command = process_command(pid)?;
    if observed_command != expected {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{role} PID {pid} command does not match owned process"),
        ));
    }
    return Ok(());
}

/// Reports whether exact command remains live and rejects PID reuse or command mismatch.
pub fn command_process_is_live(
    pid: i32,
    start_time: u64,
    expected: &[Vec<u8>],
    role: &str,
) -> io::Result<bool> {
    match validate_command_identity(pid, start_time, expected, role) {
        Ok(()) => return Ok(true),
        // A reaped process leaves a file its reader opened behind, and the kernel answers `ESRCH` there.
        Err(error) if means_vanished(&error) => return Ok(false),
        Err(error) => return Err(error),
    }
}

/// Reports whether exact holder remains live and rejects PID reuse or command mismatch.
pub fn keeper_is_live(process: KeeperProcess, cgroup_dir: &Path) -> io::Result<bool> {
    let expected = expected_command(process, cgroup_dir)?;
    return command_process_is_live(
        process.pid,
        process.start_time,
        &expected,
        "keeper",
    );
}

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

/// The process group a process belongs to,
/// read from the `pgrp` field of its `/proc` stat file.
pub fn process_group_of(pid: i32) -> io::Result<i32> {
    let text = std::fs::read_to_string(format!("/proc/{pid}/stat"))?;
    let Some(command_end) = text.rfind(')') else {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "process stat lacks command terminator"));
    };
    let Some(tail) = text.get(command_end + 1..) else {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "process stat tail is absent"));
    };
    // The fields after the command name start with `state`, so `pgrp` is the third of them.
    let Some(group) = tail.split_whitespace().nth(2).and_then(|field| return field.parse::<i32>().ok()) else {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "process stat has no process group"));
    };
    return Ok(group);
}

/// Whether a process group still has a member.
fn group_has_members(group: i32) -> bool {
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
fn signal_validated(pid: i32, group: Option<i32>, signal: libc::c_int) -> io::Result<()> {
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
fn wait_for_stop(
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

/// Waits bounded time for a process group to hold no member,
/// and answers whether it became empty.
fn wait_for_group_end(leader: i32, limit: Duration) -> bool {
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
fn signal_group(leader: i32, signal: libc::c_int) -> io::Result<()> {
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
pub fn end_leftover_group(pid: i32, start_time: u64, role: &str) -> io::Result<LeftoverGroup> {
    if !group_has_members(pid) {
        return Ok(LeftoverGroup::Absent);
    }
    match process_start_time(pid) {
        Ok(observed) if observed == start_time => {}
        Ok(_other) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{role} PID {pid} now belongs to another process, so the group it leads is not signalled"),
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
        format!("the process group of {role} PID {pid} outlived SIGTERM and SIGKILL, so its record is kept"),
    ));
}

/// Sends SIGTERM,
///  escalates exact surviving holder to SIGKILL,
///  and confirms disappearance.
pub fn stop_process(process: KeeperProcess, cgroup_dir: &Path) -> io::Result<()> {
    let expected = expected_command(process, cgroup_dir)?;
    return stop_command_process(
        process.pid,
        process.start_time,
        &expected,
        "keeper",
    );
}
