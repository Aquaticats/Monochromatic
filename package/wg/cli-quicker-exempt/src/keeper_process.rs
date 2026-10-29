//! Validates descriptor-keeper process identity before lifecycle signals.

/// Persisted process identity and socket mark.
use crate::keeper_state::KeeperProcess;
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
    let command_end = text.rfind(')').ok_or_else(|| {
        return io::Error::new(io::ErrorKind::InvalidData, "process stat lacks command terminator");
    })?;
    let tail = text.get(command_end + 1..).ok_or_else(|| {
        return io::Error::new(io::ErrorKind::InvalidData, "process stat tail is absent");
    })?;
    let fields: Vec<&str> = tail.split_whitespace().collect();
    let start_time = fields.get(19).ok_or_else(|| {
        return io::Error::new(io::ErrorKind::InvalidData, "process stat lacks start time");
    })?;
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

/// Reads command-line arguments without interpreting path bytes as text.
fn process_command(pid: i32) -> io::Result<Vec<Vec<u8>>> {
    let bytes = std::fs::read(format!("/proc/{pid}/cmdline"))?;
    let mut arguments: Vec<Vec<u8>> = bytes
        .split(|byte| return *byte == 0)
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
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
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
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(true),
            Err(error) => return Err(error),
        }
    }
    return Ok(false);
}

/// Stops exact command after typed validation and bounded signal escalation.
pub fn stop_command_process(
    pid: i32,
    start_time: u64,
    expected: &[Vec<u8>],
    role: &str,
) -> io::Result<()> {
    validate_command_identity(pid, start_time, expected, role)?;
    // SAFETY: validated positive PID names exact owned process.
    let terminate_result = unsafe { libc::kill(pid, libc::SIGTERM) };
    if terminate_result < 0 {
        return Err(io::Error::last_os_error());
    }
    if wait_for_disappearance(pid, start_time, STOP_WAIT)? {
        return Ok(());
    }
    validate_command_identity(pid, start_time, expected, role)?;
    // SAFETY: identity was revalidated immediately before forced termination.
    let kill_result = unsafe { libc::kill(pid, libc::SIGKILL) };
    if kill_result < 0 {
        return Err(io::Error::last_os_error());
    }
    if wait_for_disappearance(pid, start_time, KILL_WAIT)? {
        return Ok(());
    }
    return Err(io::Error::new(
        io::ErrorKind::TimedOut,
        format!("{role} PID {pid} survived SIGTERM and SIGKILL"),
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
