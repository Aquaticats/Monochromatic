//! Persists crash-recoverable descriptor-keeper replacement phases under `/run`.

/// Collision-free cgroup key shared with pin lifecycle.
use crate::pin::cgroup_key;
/// Standard filesystem and parse failures.
use std::io;
/// Unix creation modes for private state files.
use std::os::unix::fs::OpenOptionsExt;
/// Runtime state paths.
use std::path::{Path, PathBuf};
/// Buffered file output and durability flushes.
use std::io::Write;

/// Private runtime root for descriptor-keeper identities.
const KEEPER_STATE_ROOT: &str = "/run/wg-quicker-exempt/keepers";
/// On-disk schema version.
const STATE_VERSION: &str = "1";

/// Exact process identity and mark expected in holder command line.
#[derive(Clone, Copy)]
pub struct KeeperProcess {
    /// Linux process identifier.
    pub pid: i32,
    /// `/proc/<pid>/stat` start-time ticks prevent PID reuse.
    pub start_time: u64,
    /// Socket mark encoded in holder command.
    pub mark: u32,
}

/// State can describe one active holder or in-flight old-to-new replacement.
pub struct KeeperState {
    /// Prior committed process,
    ///  absent during first attach.
    pub active: Option<KeeperProcess>,
    /// Candidate process,
    ///  present until replacement commit settles.
    pub candidate: Option<KeeperProcess>,
}

/// Returns active-state value after replacement completion.
pub fn active_state(process: KeeperProcess) -> KeeperState {
    return KeeperState {
        active: Some(process),
        candidate: None,
    };
}

/// Returns transition-state value before candidate receives commit message.
pub fn replacing_state(
    active: Option<KeeperProcess>,
    candidate: KeeperProcess,
) -> KeeperState {
    return KeeperState {
        active,
        candidate: Some(candidate),
    };
}

/// Maps cgroup identity into private runtime state file path.
pub fn state_path(cgroup_dir: &Path) -> io::Result<PathBuf> {
    return Ok(Path::new(KEEPER_STATE_ROOT)
        .join(cgroup_key(cgroup_dir)?)
        .join("state"));
}

/// Parses comma-delimited numeric process record or explicit absence sentinel.
fn parse_process(value: &str) -> io::Result<Option<KeeperProcess>> {
    if value == "none" {
        return Ok(None);
    }
    let fields: Vec<&str> = value.split(',').collect();
    if fields.len() != 3 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "keeper process record must contain pid,start_time,mark",
        ));
    }
    let pid: i32 = fields[0].parse().map_err(io::Error::other)?;
    let start_time: u64 = fields[1].parse().map_err(io::Error::other)?;
    let mark: u32 = fields[2].parse().map_err(io::Error::other)?;
    if pid <= 0 || start_time == 0 || mark == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "keeper process record contains zero or negative identity",
        ));
    }
    return Ok(Some(KeeperProcess {
        pid,
        start_time,
        mark,
    }));
}

/// Parses strict line schema and rejects unknown,
///  missing,
///  or duplicate fields.
fn parse_state(text: &str) -> io::Result<KeeperState> {
    let mut version: Option<&str> = None;
    let mut active: Option<Option<KeeperProcess>> = None;
    let mut candidate: Option<Option<KeeperProcess>> = None;
    for line in text.lines() {
        let (key, value) = line.split_once('=').ok_or_else(|| {
            return io::Error::new(io::ErrorKind::InvalidData, "keeper state line lacks equals");
        })?;
        if key == "version" && version.is_none() {
            version = Some(value);
        } else if key == "active" && active.is_none() {
            active = Some(parse_process(value)?);
        } else if key == "candidate" && candidate.is_none() {
            candidate = Some(parse_process(value)?);
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unknown or duplicate keeper state field: {key}"),
            ));
        }
    }
    if version != Some(STATE_VERSION) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsupported keeper state version",
        ));
    }
    let parsed = KeeperState {
        active: active.ok_or_else(|| return io::Error::other("missing active keeper field"))?,
        candidate: candidate
            .ok_or_else(|| return io::Error::other("missing candidate keeper field"))?,
    };
    if parsed.active.is_none() && parsed.candidate.is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "keeper state contains no process",
        ));
    }
    return Ok(parsed);
}

/// Reads state when present and preserves malformed state as fail-closed error.
pub fn read_state(cgroup_dir: &Path) -> io::Result<Option<KeeperState>> {
    let path = state_path(cgroup_dir)?;
    match std::fs::read_to_string(path) {
        Ok(text) => return Ok(Some(parse_state(&text)?)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    }
}

/// Serializes one process as numeric syntax without path interpolation.
fn serialize_process(process: Option<KeeperProcess>) -> String {
    let Some(value) = process else {
        return "none".to_owned();
    };
    return format!("{},{},{}", value.pid, value.start_time, value.mark);
}

/// Atomically writes state and syncs file plus parent directory.
pub fn write_state(cgroup_dir: &Path, state: &KeeperState) -> io::Result<()> {
    let path = state_path(cgroup_dir)?;
    let parent = path.parent().ok_or_else(|| return io::Error::other("state path lacks parent"))?;
    std::fs::create_dir_all(parent)?;
    let temporary = parent.join(format!("state-new-{}", std::process::id()));
    let text = format!(
        "version={STATE_VERSION}\nactive={}\ncandidate={}\n",
        serialize_process(state.active),
        serialize_process(state.candidate)
    );
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .mode(0o600)
        .open(&temporary)?;
    file.write_all(text.as_bytes())?;
    file.sync_all()?;
    std::fs::rename(&temporary, &path)?;
    std::fs::File::open(parent)?.sync_all()?;
    return Ok(());
}

/// Returns candidate commit-marker path unique to PID and start time.
fn commit_marker_path(
    cgroup_dir: &Path,
    process: KeeperProcess,
) -> io::Result<PathBuf> {
    let state = state_path(cgroup_dir)?;
    let parent = state.parent().ok_or_else(|| return io::Error::other("state path lacks parent"))?;
    return Ok(parent.join(format!(
        "committed-{}-{}",
        process.pid, process.start_time
    )));
}

/// Records that candidate received commit after transition state reached disk.
pub fn write_commit_marker(
    cgroup_dir: &Path,
    process: KeeperProcess,
) -> io::Result<()> {
    let path = commit_marker_path(cgroup_dir, process)?;
    let file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(path)?;
    return file.sync_all();
}

/// Reports whether candidate completed commit handshake.
pub fn commit_marker_exists(
    cgroup_dir: &Path,
    process: KeeperProcess,
) -> io::Result<bool> {
    return Ok(commit_marker_path(cgroup_dir, process)?.exists());
}

/// Removes candidate commit marker after final state or rollback settles.
pub fn remove_commit_marker(
    cgroup_dir: &Path,
    process: KeeperProcess,
) -> io::Result<()> {
    let path = commit_marker_path(cgroup_dir, process)?;
    match std::fs::remove_file(path) {
        Ok(()) => return Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    }
}

/// Removes now-empty encoded key directories without crossing keeper root.
fn remove_empty_key_directories(path: &Path) -> io::Result<()> {
    let root = Path::new(KEEPER_STATE_ROOT);
    let mut current = path.parent();
    while let Some(directory) = current {
        if directory == root {
            return Ok(());
        }
        match std::fs::remove_dir(directory) {
            Ok(()) => current = directory.parent(),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                current = directory.parent();
            }
            Err(error) if error.kind() == io::ErrorKind::DirectoryNotEmpty => return Ok(()),
            Err(error) => return Err(error),
        }
    }
    return Ok(());
}

/// Removes committed state only after corresponding holder is confirmed gone.
pub fn remove_state(cgroup_dir: &Path) -> io::Result<()> {
    let path = state_path(cgroup_dir)?;
    match std::fs::remove_file(&path) {
        Ok(()) => return remove_empty_key_directories(&path),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return remove_empty_key_directories(&path);
        }
        Err(error) => return Err(error),
    }
}
