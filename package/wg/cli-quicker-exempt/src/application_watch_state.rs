//! Persists exact application-watcher identity for start and stop commands.

/// Standard state errors and durable writes.
use std::io::{self, Write};
/// Unix file creation mode.
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
/// Watcher state paths.
use std::path::{Path, PathBuf};

/// Private application watcher runtime root.
const WATCH_STATE_ROOT: &str = "/run/wg-quicker-exempt/watchers";
/// Numeric state schema version.
const WATCH_STATE_VERSION: u32 = 1;
/// Hex digits encode arbitrary watcher keys as one safe component.
const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

/// Exact process identity and watcher inputs.
#[derive(Clone, Copy)]
pub struct WatchProcess {
    /// Watcher process identifier.
    pub pid: i32,
    /// Linux process start-time ticks.
    pub start_time: u64,
    /// Socket mark applied to application targets.
    pub mark: u32,
    /// User whose app slice is watched.
    pub uid: u32,
}

/// Encodes watcher key bytes without path syntax.
fn encode_key(key: &str) -> io::Result<String> {
    if key.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "watcher key must not be empty",
        ));
    }
    return Ok(key
        .as_bytes()
        .iter()
        .flat_map(|byte| {
            return [
                HEX_DIGITS[(byte >> 4) as usize] as char,
                HEX_DIGITS[(byte & 0x0f) as usize] as char,
            ];
        })
        .collect());
}

/// Maps watcher key into private state directory.
pub fn watch_state_directory(key: &str) -> io::Result<PathBuf> {
    return Ok(Path::new(WATCH_STATE_ROOT).join(encode_key(key)?));
}

/// Maps watcher key into private state file.
pub fn watch_state_path(key: &str) -> io::Result<PathBuf> {
    return Ok(watch_state_directory(key)?.join("state"));
}

/// Maps watcher key into persistent watcher diagnostic log.
pub fn watch_log_path(key: &str) -> io::Result<PathBuf> {
    return Ok(watch_state_directory(key)?.join("watch.log"));
}

/// Creates private watcher state directory before child log open.
pub fn ensure_watch_state_directory(key: &str) -> io::Result<PathBuf> {
    let directory = watch_state_directory(key)?;
    std::fs::create_dir_all(&directory)?;
    std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700))?;
    return Ok(directory);
}

/// Parses strict comma-delimited numeric schema.
fn parse_state(text: &str) -> io::Result<WatchProcess> {
    let fields: Vec<&str> = text.trim_end().split(',').collect();
    if fields.len() != 5 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "watcher state must contain version,pid,start_time,mark,uid",
        ));
    }
    let version: u32 = fields[0].parse().map_err(io::Error::other)?;
    let pid: i32 = fields[1].parse().map_err(io::Error::other)?;
    let start_time: u64 = fields[2].parse().map_err(io::Error::other)?;
    let mark: u32 = fields[3].parse().map_err(io::Error::other)?;
    let uid: u32 = fields[4].parse().map_err(io::Error::other)?;
    if version != WATCH_STATE_VERSION || pid <= 0 || start_time == 0 || mark == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "watcher state contains unsupported version or invalid identity",
        ));
    }
    return Ok(WatchProcess {
        pid,
        start_time,
        mark,
        uid,
    });
}

/// Reads watcher state when present.
pub fn read_watch_state(key: &str) -> io::Result<Option<WatchProcess>> {
    let path = watch_state_path(key)?;
    match std::fs::read_to_string(path) {
        Ok(text) => return Ok(Some(parse_state(&text)?)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    }
}

/// Atomically writes watcher state and syncs file plus parent directory.
pub fn write_watch_state(key: &str, process: WatchProcess) -> io::Result<()> {
    let path = watch_state_path(key)?;
    let parent = path.parent().ok_or_else(|| return io::Error::other("watch state path lacks parent"))?;
    ensure_watch_state_directory(key)?;
    let temporary = parent.join(format!("state-new-{}", std::process::id()));
    let text = format!(
        "{WATCH_STATE_VERSION},{},{},{},{}\n",
        process.pid, process.start_time, process.mark, process.uid
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

/// Removes watcher state and empty key directory after process disappearance.
pub fn remove_watch_state(key: &str) -> io::Result<()> {
    let path = watch_state_path(key)?;
    match std::fs::remove_file(&path) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    let log_path = watch_log_path(key)?;
    match std::fs::remove_file(log_path) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    match std::fs::remove_dir(parent) {
        Ok(()) => return Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) if error.kind() == io::ErrorKind::DirectoryNotEmpty => return Ok(()),
        Err(error) => return Err(error),
    }
}
