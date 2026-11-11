//! Reads the facts about one process that the update-traffic policy needs from a procfs tree.
//! The root is injectable so tests use a fake tree.
//! A process that is exiting is reported as absent:
//! it is going away,
//! and handing it to the service manager leaves an empty scope behind.
//! Files are read with a bounded number of system calls,
//! because a pass over every process multiplies each call by the process count.

/// Syscall and process failures.
use std::io;
/// Bounded reads of procfs files.
use std::io::Read;
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

/// Index of the state letter after the command name in `/proc/<pid>/stat`.
const STATE_FIELD: usize = 0;
/// Index of the parent id after the command name in `/proc/<pid>/stat`.
const PARENT_FIELD: usize = 1;
/// Index of the task flags after the command name in `/proc/<pid>/stat`,
/// field 9 of `proc_pid_stat(5)`.
const FLAGS_FIELD: usize = 6;
/// Index of the start-time field after the command name in `/proc/<pid>/stat`.
const START_TIME_FIELD: usize = 19;
/// The `PF_EXITING` task flag of the kernel's `include/linux/sched.h`:
/// set when the task starts to shut down,
/// before its memory is released and long before it is a zombie.
/// The kernel's `cgroup_migrate_add_task` skips such a task while the write to `cgroup.procs` still succeeds.
const PF_EXITING: u64 = 0x4;
/// Bytes a procfs file is read into with the first call,
/// large enough for the status,
/// stat,
/// cgroup and usual command line files,
/// so one call returns the content and the next returns the end of the file.
const READ_BUFFER_BYTES: usize = 8192;
/// Bytes read from the start of `/proc/<pid>/status`,
/// which holds the `Uid:` and `Gid:` lines within its first 300 bytes.
const STATUS_HEAD_BYTES: usize = 1024;

/// Reports an error that means the process exited while it was being read:
/// its entry is gone,
/// or it was reaped while the file it left open was read,
/// which the kernel answers with `ESRCH` (`No such process`).
/// A permission error is not one of them:
/// the process is alive and this reader may not inspect it.
pub fn means_vanished(error: &io::Error) -> bool {
    return error.kind() == io::ErrorKind::NotFound || error.raw_os_error() == Some(libc::ESRCH);
}

/// Reads a whole procfs file with a buffer that is large enough for the first call.
/// `std::fs::read` sizes its buffer from the file length,
/// which is zero for procfs,
/// so it starts with a 32-byte probe and doubles:
/// measured on this host,
/// that is about five read calls per file against two here.
pub fn read_whole(path: &Path) -> io::Result<Vec<u8>> {
    let mut file = std::fs::File::open(path)?;
    let mut bytes: Vec<u8> = Vec::with_capacity(READ_BUFFER_BYTES);
    file.read_to_end(&mut bytes)?;
    return Ok(bytes);
}

/// Reads the first bytes of a procfs file with a single read call.
fn read_head(path: &Path) -> io::Result<Vec<u8>> {
    let mut file = std::fs::File::open(path)?;
    let mut bytes: Vec<u8> = vec![0; STATUS_HEAD_BYTES];
    let count = file.read(&mut bytes)?;
    bytes.truncate(count);
    return Ok(bytes);
}

/// Turns the result of reading a file that may vanish with its process into the policy's shape.
/// A vanished process is `None`,
/// and every other failure,
/// a permission error included,
/// is returned.
fn vanishing(result: io::Result<Vec<u8>>) -> io::Result<Option<Vec<u8>>> {
    return match result {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if means_vanished(&error) => Ok(None),
        Err(error) => Err(error),
    };
}

/// Reads a file that may vanish with its process.
fn read_vanishing(path: &Path) -> io::Result<Option<Vec<u8>>> {
    return vanishing(read_whole(path));
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

/// What the policy reads from `/proc/<pid>/stat`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StatFields {
    /// The state letter,
    /// such as `S` for sleeping and `Z` for a zombie.
    pub state: char,
    /// Parent process id.
    pub parent: i32,
    /// Task flags.
    pub flags: u64,
    /// Start time in clock ticks since boot.
    pub start_time: u64,
}

/// Reporting.
impl StatFields {
    /// Reports whether the process is on its way out:
    /// a zombie or dead,
    /// or flagged `PF_EXITING` while it still looks alive.
    pub fn exiting(&self) -> bool {
        return matches!(self.state, 'Z' | 'X' | 'x') || self.flags & PF_EXITING != 0;
    }
}

/// Parses the state,
/// parent id,
/// flags and start time from `/proc/<pid>/stat`,
/// skipping the parenthesized command name safely.
pub fn parse_stat_fields(text: &str) -> Option<StatFields> {
    let command_end = text.rfind(')')?;
    let tail = text.get(command_end + 1..)?;
    let fields: Vec<&str> = tail.split_whitespace().collect();
    let state: char = fields.get(STATE_FIELD)?.chars().next()?;
    let parent: i32 = fields.get(PARENT_FIELD)?.parse().ok()?;
    let flags: u64 = fields.get(FLAGS_FIELD)?.parse().ok()?;
    let start_time: u64 = fields.get(START_TIME_FIELD)?.parse().ok()?;
    return Some(StatFields { state, parent, flags, start_time });
}

/// Parses the parent id and start time from `/proc/<pid>/stat`,
/// skipping the parenthesized command name safely.
pub fn parse_stat(text: &str) -> Option<(i32, u64)> {
    let fields = parse_stat_fields(text)?;
    return Some((fields.parent, fields.start_time));
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
    let Some(status_bytes) = vanishing(read_head(&base.join("status")))? else {
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
    let Some(stat) = parse_stat_fields(&String::from_utf8_lossy(&stat_bytes)) else {
        return Ok(None);
    };
    // An exiting process is as good as gone: it cannot be moved, and a scope handed only such processes stays empty.
    if stat.exiting() {
        return Ok(None);
    }
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
        parent: stat.parent,
        start_time: stat.start_time,
        pid_namespace: read_pid_namespace(&base),
    }));
}

/// Reads one pid per line from a `cgroup.procs` text,
/// skipping lines that are not a number.
fn parse_pid_line(line: &str) -> Option<i32> {
    return line.trim().parse().ok();
}

/// Lists the processes of a cgroup from its `cgroup.procs` file,
/// or `None` when the file cannot be read.
/// One small file against a pass over every process of the host:
/// on this host a pass reads 949 process directories (about 26 ms of CPU),
/// and the cgroup of `claude-code-bash` holds a fraction of them.
pub fn read_cgroup_pids(cgroup_root: &Path, cgroup: &str) -> Option<Vec<i32>> {
    let path = cgroup_root.join(cgroup.trim_start_matches('/')).join("cgroup.procs");
    let bytes = read_whole(&path).ok()?;
    return Some(String::from_utf8_lossy(&bytes).lines().filter_map(parse_pid_line).collect());
}

/// Reads only the start time of a process,
/// from the world-readable stat file.
/// Any failure is `None`,
/// which callers use only to deduplicate reports.
pub fn read_start_time(proc_root: &Path, pid: i32) -> Option<u64> {
    let bytes = read_whole(&proc_root.join(pid.to_string()).join("stat")).ok()?;
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
