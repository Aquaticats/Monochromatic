//! Reads the facts about one process that the update-traffic policy needs from a procfs tree.
//! The root is injectable so tests use a fake tree.

/// Raw Unix path bytes for the executable link.
use std::io;
/// Path views.
use std::path::{Path, PathBuf};

/// What the policy knows about one live process.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessInfo {
    /// Process id.
    pub pid: i32,
    /// Resolved executable path.
    pub exe: PathBuf,
    /// Raw command line,
    /// program name first.
    pub args: Vec<Vec<u8>>,
    /// Real user id.
    pub uid: u32,
    /// Real group id.
    pub gid: u32,
    /// Unified cgroup path,
    /// such as `/user.slice/user-1000.slice/user@1000.service/app.slice/x.scope`.
    pub cgroup: String,
    /// Parent process id.
    pub parent: i32,
    /// Start time in clock ticks since boot,
    /// which tells a reused pid from the original.
    pub start_time: u64,
}

/// Index of the start-time field after the command name in `/proc/<pid>/stat`.
const START_TIME_FIELD: usize = 19;

/// Reads a file that may vanish with its process.
fn read_vanishing(path: &Path) -> io::Result<Option<Vec<u8>>> {
    return match std::fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if matches!(error.kind(), io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied) => Ok(None),
        Err(error) if error.raw_os_error() == Some(libc::ESRCH) => Ok(None),
        Err(error) => Err(error),
    };
}

/// Parses the parent id and start time from `/proc/<pid>/stat`,
/// skipping the parenthesized command name safely.
pub fn parse_stat(text: &str) -> Option<(i32, u64)> {
    let command_end = text.rfind(')')?;
    let tail = text.get(command_end + 1..)?;
    let fields: Vec<&str> = tail.split_whitespace().collect();
    let parent: i32 = fields.get(1)?.parse().ok()?;
    let start_time: u64 = fields.get(START_TIME_FIELD)?.parse().ok()?;
    return Some((parent, start_time));
}

/// Reads the real id from a `Uid:` or `Gid:` line of `/proc/<pid>/status`.
pub fn parse_status_id(status: &str, label: &str) -> Option<u32> {
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix(label) {
            return rest.split_whitespace().next()?.parse().ok();
        }
    }
    return None;
}

/// Reads the unified cgroup path from `/proc/<pid>/cgroup` text.
pub fn parse_cgroup(text: &str) -> Option<String> {
    for line in text.lines() {
        if let Some(path) = line.strip_prefix("0::") {
            return Some(path.to_string());
        }
    }
    return None;
}

/// Splits a `/proc/<pid>/cmdline` buffer into raw arguments.
pub fn split_cmdline(bytes: &[u8]) -> Vec<Vec<u8>> {
    let mut arguments: Vec<Vec<u8>> = bytes.split(is_nul).map(<[u8]>::to_vec).collect();
    if arguments.last().is_some_and(Vec::is_empty) {
        arguments.pop();
    }
    return arguments;
}

/// Reports the NUL byte that ends each argument.
fn is_nul(byte: &u8) -> bool {
    return *byte == 0;
}

/// Reads one process from `proc_root`,
/// or `None` when it vanished or is not readable.
pub fn read_process(proc_root: &Path, pid: i32) -> io::Result<Option<ProcessInfo>> {
    let base = proc_root.join(pid.to_string());
    let exe = match std::fs::read_link(base.join("exe")) {
        Ok(path) => path,
        Err(error) if matches!(error.kind(), io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied) => return Ok(None),
        Err(error) if error.raw_os_error() == Some(libc::ESRCH) => return Ok(None),
        Err(error) => return Err(error),
    };
    let Some(cmdline) = read_vanishing(&base.join("cmdline"))? else {
        return Ok(None);
    };
    let Some(status_bytes) = read_vanishing(&base.join("status"))? else {
        return Ok(None);
    };
    let Some(cgroup_bytes) = read_vanishing(&base.join("cgroup"))? else {
        return Ok(None);
    };
    let Some(stat_bytes) = read_vanishing(&base.join("stat"))? else {
        return Ok(None);
    };
    let status = String::from_utf8_lossy(&status_bytes);
    let (Some(uid), Some(gid)) = (parse_status_id(&status, "Uid:"), parse_status_id(&status, "Gid:")) else {
        return Ok(None);
    };
    let Some(cgroup) = parse_cgroup(&String::from_utf8_lossy(&cgroup_bytes)) else {
        return Ok(None);
    };
    let Some((parent, start_time)) = parse_stat(&String::from_utf8_lossy(&stat_bytes)) else {
        return Ok(None);
    };
    return Ok(Some(ProcessInfo {
        pid,
        exe,
        args: split_cmdline(&cmdline),
        uid,
        gid,
        cgroup,
        parent,
        start_time,
    }));
}

/// Lists the numeric process directories of `proc_root`.
pub fn list_pids(proc_root: &Path) -> io::Result<Vec<i32>> {
    let mut pids: Vec<i32> = Vec::new();
    for listed in std::fs::read_dir(proc_root)? {
        let name = listed?.file_name();
        if let Some(text) = name.to_str()
            && let Ok(pid) = text.parse::<i32>()
        {
            pids.push(pid);
        }
    }
    return Ok(pids);
}
