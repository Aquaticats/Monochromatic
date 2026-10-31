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
    /// Pid namespace link text such as `pid:[4026531836]`,
    /// absent when the kernel did not expose it.
    pub pid_namespace: Option<String>,
}

/// Index of the start-time field after the command name in `/proc/<pid>/stat`.
const START_TIME_FIELD: usize = 19;

/// Reports an error that means the process exited while it was being read.
/// A permission error is not one of them:
/// the process is alive and this reader may not inspect it.
fn means_vanished(error: &io::Error) -> bool {
    return error.kind() == io::ErrorKind::NotFound || error.raw_os_error() == Some(libc::ESRCH);
}

/// Reads a file that may vanish with its process.
/// A vanished process is `None`,
/// and every other failure,
/// a permission error included,
/// is returned.
fn read_vanishing(path: &Path) -> io::Result<Option<Vec<u8>>> {
    return match std::fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if means_vanished(&error) => Ok(None),
        Err(error) => Err(error),
    };
}

/// Reads the pid namespace link of the process directory `base`,
/// or `None` when the kernel does not expose it.
fn read_pid_namespace(base: &Path) -> Option<String> {
    return match std::fs::read_link(base.join("ns/pid")) {
        Ok(link) => Some(link.to_string_lossy().into_owned()),
        Err(_error) => None,
    };
}

/// Reads the pid namespace of the reading process itself through `<proc_root>/self`.
pub fn read_own_pid_namespace(proc_root: &Path) -> Option<String> {
    return read_pid_namespace(&proc_root.join("self"));
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

/// Reads one process of `only_uid` from `proc_root`.
/// The result is `None` when the process exited meanwhile or belongs to another user,
/// whose files a non-root reader may not open and whose facts the policy does not need.
/// A process of `only_uid` that this reader may not inspect is an error naming the file,
/// so the caller can report it instead of mistaking it for an exit.
pub fn read_process(proc_root: &Path, pid: i32, only_uid: u32) -> io::Result<Option<ProcessInfo>> {
    let base = proc_root.join(pid.to_string());
    // The status file is world-readable, so the owner check never fails for another user's process.
    let Some(status_bytes) = read_vanishing(&base.join("status"))? else {
        return Ok(None);
    };
    let status = String::from_utf8_lossy(&status_bytes);
    let (Some(uid), Some(gid)) = (parse_status_id(&status, "Uid:"), parse_status_id(&status, "Gid:")) else {
        return Ok(None);
    };
    if uid != only_uid {
        return Ok(None);
    }
    let Some(stat_bytes) = read_vanishing(&base.join("stat"))? else {
        return Ok(None);
    };
    let Some((parent, start_time)) = parse_stat(&String::from_utf8_lossy(&stat_bytes)) else {
        return Ok(None);
    };
    let exe = match std::fs::read_link(base.join("exe")) {
        Ok(path) => path,
        Err(error) if means_vanished(&error) => return Ok(None),
        Err(error) => return Err(io::Error::new(error.kind(), format!("{}: {error}", base.join("exe").display()))),
    };
    let Some(cmdline) = read_vanishing(&base.join("cmdline"))? else {
        return Ok(None);
    };
    let Some(cgroup_bytes) = read_vanishing(&base.join("cgroup"))? else {
        return Ok(None);
    };
    let Some(cgroup) = parse_cgroup(&String::from_utf8_lossy(&cgroup_bytes)) else {
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
        pid_namespace: read_pid_namespace(&base),
    }));
}

/// Reads only the start time of a process,
/// from the world-readable stat file.
/// Any failure is `None`,
/// which callers use only to deduplicate reports.
pub fn read_start_time(proc_root: &Path, pid: i32) -> Option<u64> {
    let bytes = std::fs::read(proc_root.join(pid.to_string()).join("stat")).ok()?;
    let (_parent, start_time) = parse_stat(&String::from_utf8_lossy(&bytes))?;
    return Some(start_time);
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
