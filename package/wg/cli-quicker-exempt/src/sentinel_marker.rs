//! The marker a running sentinel leaves in the state directory of its watcher,
//! so the process that started it can tell it got as far as running.
//! The marker names the watcher and the sentinel's own pid and start time.
//! It is written to a new file that is renamed over the marker,
//! so a link planted at the marker's name is replaced and never followed.

/// Where the state of one watcher lives.
use crate::application_watch_state::watch_state_directory;
/// Process start times.
use crate::process_info::read_start_time;
/// Writing a file without following a link.
use std::io::Write;
/// Creating a file with a mode.
use std::os::unix::fs::OpenOptionsExt;
/// Paths of the marker.
use std::path::Path;

/// File name of the marker inside the state directory of the watcher.
pub const MARKER_NAME: &str = "sentinel";

/// What a running sentinel leaves for the process that started it.
#[derive(Debug, Eq, PartialEq)]
pub struct SentinelMarker {
    /// Process identifier of the watcher the sentinel waits for.
    pub watcher_pid: i32,
    /// Process identifier of the sentinel.
    pub pid: i32,
    /// Start time of the sentinel process,
    /// so a reused pid is not mistaken for it.
    pub start_time: u64,
}

/// Parses the text of a marker file.
pub fn parse_marker(text: &str) -> Option<SentinelMarker> {
    let mut words = text.split_whitespace();
    let watcher_pid = words.next()?.parse().ok()?;
    let pid = words.next()?.parse().ok()?;
    let start_time = words.next()?.parse().ok()?;
    if words.next().is_some() {
        return None;
    }
    return Some(SentinelMarker { watcher_pid, pid, start_time });
}

/// Writes the marker of the running sentinel into `directory`.
/// The text goes to a new file that is renamed over the marker,
/// so a link planted at the marker's name is replaced and never followed.
pub fn write_marker_in(directory: &Path, watcher_pid: i32) -> std::io::Result<()> {
    let own = i32::try_from(std::process::id()).map_err(std::io::Error::other)?;
    let Some(start_time) = read_start_time(Path::new("/proc"), own) else {
        return Err(std::io::Error::other("the sentinel process is not in procfs"));
    };
    let temporary = directory.join(format!("{MARKER_NAME}.{own}.tmp"));
    let _removed = std::fs::remove_file(&temporary);
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&temporary)?;
    file.write_all(format!("{watcher_pid} {own} {start_time}\n").as_bytes())?;
    file.sync_all()?;
    return std::fs::rename(&temporary, directory.join(MARKER_NAME));
}

/// Reads the marker in `directory`,
/// or nothing when there is none or it is malformed.
pub fn read_marker_in(directory: &Path) -> Option<SentinelMarker> {
    let text = std::fs::read_to_string(directory.join(MARKER_NAME)).ok()?;
    return parse_marker(&text);
}

/// Writes the marker of the running sentinel for the watcher stored under `key`.
pub fn write_marker(key: &str, watcher_pid: i32) -> std::io::Result<()> {
    return write_marker_in(&watch_state_directory(key)?, watcher_pid);
}
