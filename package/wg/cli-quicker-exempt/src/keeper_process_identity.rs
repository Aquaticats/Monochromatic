//! The identity a lifecycle signal is allowed to rest on:
//! a process's start time,
//! the exact bytes of its command line,
//! and the process group it leads.

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

/// Reads Linux process start-time field after safely skipping parenthesized command name.
pub fn process_start_time(pid: i32) -> io::Result<u64> {
    let text = std::fs::read_to_string(format!("/proc/{pid}/stat"))?;
    let Some(command_end) = text.rfind(')') else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "process stat lacks command terminator",
        ));
    };
    let Some(tail) = text.get(command_end + 1..) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "process stat tail is absent",
        ));
    };
    let fields: Vec<&str> = tail.split_whitespace().collect();
    let Some(start_time) = fields.get(19) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "process stat lacks start time",
        ));
    };
    return start_time.parse().map_err(io::Error::other);
}

/// Builds exact NUL-separated argument vector expected for one holder.
pub(super) fn expected_command(
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
    let mut arguments: Vec<Vec<u8>> = bytes.split(is_nul_byte).map(<[u8]>::to_vec).collect();
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
pub fn keeper_is_live(
    process: KeeperProcess,
    cgroup_dir: &Path,
) -> io::Result<bool> {
    let expected = expected_command(process, cgroup_dir)?;
    return command_process_is_live(process.pid, process.start_time, &expected, "keeper");
}

/// The process group a process belongs to,
/// read from the `pgrp` field of its `/proc` stat file.
pub fn process_group_of(pid: i32) -> io::Result<i32> {
    let text = std::fs::read_to_string(format!("/proc/{pid}/stat"))?;
    let Some(command_end) = text.rfind(')') else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "process stat lacks command terminator",
        ));
    };
    let Some(tail) = text.get(command_end + 1..) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "process stat tail is absent",
        ));
    };
    // The fields after the command name start with `state`, so `pgrp` is the third of them.
    let Some(group) = tail
        .split_whitespace()
        .nth(2)
        .and_then(|field| return field.parse::<i32>().ok())
    else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "process stat has no process group",
        ));
    };
    return Ok(group);
}
