//! Replacing the session file atomically.
//!
//! What:
//!  The one place a session file is replaced:
//!  the process-wide save lock,
//!  `replace_file`, which writes a temporary file beside the target and renames it over,
//!  and `write_synced`, which flushes the bytes to disk before that rename.
//! Why:
//!  A save must publish either the whole new session or the previous one,
//!  never a half-written file,
//!  and this machinery has nothing to do with the record's shape or its migration,
//!  so it gets a file of its own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // One lock for the whole process: overlapping saves would share one temporary file name.
//! const saveLock = new Mutex();
//! function replaceFile(path: string, bytes: Uint8Array): void {
//!   const temporary = `${path}.tmp`;
//!   writeSynced(temporary, bytes); // write, then flush to disk
//!   renameSync(temporary, path); // atomic inside one directory
//! }
//! ```

/// Borrowed filesystem path of the file being replaced.
use std::path::Path;

/// What:
/// `SAVE_LOCK` serializes the replacements of session files inside this process.
/// Why:
/// the controller and the settings and template editors save from different threads,
/// and the replacement uses one
/// temporary file name per target,
/// so two overlapping saves would write into the same temporary file.
static SAVE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// What:
/// `replace_file` writes `bytes` to a temporary file beside `path`,
/// flushes it to disk and renames it over `path`.
/// Why:
/// `std::fs::write` truncates the target before the new bytes land,
/// so a crash,
/// kill or power loss during the
/// write left a half-written session that failed to parse on the next launch.
/// A rename inside one directory is atomic
/// on one filesystem,
/// so `path` is always either the previous complete file or the new complete file.
/// The temporary
/// file shares the directory because a rename across filesystems is not atomic.
/// A failed save removes its temporary file and leaves `path` as it was;
/// a temporary file left by a killed save is
/// truncated by the next one.
pub(super) fn replace_file(
    path: &Path,
    bytes: &[u8],
) -> std::io::Result<()> {
    let Some(name) = path.file_name() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("the session path {} has no file name", path.display()),
        ));
    };
    let mut temporary_name = name.to_os_string();
    temporary_name.push(".tmp");
    let temporary = path.with_file_name(temporary_name);
    let _guard = SAVE_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let written = match write_synced(&temporary, bytes) {
        Ok(()) => std::fs::rename(&temporary, path),
        Err(error) => Err(error),
    };
    if written.is_err()
        && let Err(error) = std::fs::remove_file(&temporary)
        && error.kind() != std::io::ErrorKind::NotFound
    {
        tracing::warn!(?error, path = %temporary.display(), "failed to remove the temporary playback session file");
    }
    return written;
}

/// What:
/// `write_synced` creates (or truncates) the file at `path`,
/// writes `bytes` and flushes them to the disk.
/// Why:
/// the rename in `replace_file` must only publish bytes that already reached the disk,
/// or a power loss after the
/// rename could leave the new name on an empty file.
fn write_synced(
    path: &Path,
    bytes: &[u8],
) -> std::io::Result<()> {
    /// Imports the write trait that the synced file write needs.
    use std::io::Write;
    let mut file = std::fs::File::create(path)?;
    file.write_all(bytes)?;
    return file.sync_all();
}
