//! What:
//!  The staging files of an index lock:
//! their names,
//! their creation with the marker written,
//! and the removal of the ones a killed attempt left.
//! Why:
//!  The marker is written to a file that is new and becomes the lock in one step,
//! so the lock never exists without it.
//! A staging name carries the process and a token no other attempt has,
//! so no attempt opens a file another left,
//! and a later attempt tells from the name alone whether the writer still runs.
//! The module is split from `service_index_placement.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { path, handle } = await stageMarker(lock, seq, marker)
//! ```

use super::process_identity::process_birth_identity;
use super::random_id::random_uuid;
use std::io::Write;
use std::path::{Path, PathBuf};

/// What follows the lock's file name in the name of a staging file:
/// the marker is written there before it becomes the lock.
pub const STAGED_INFIX: &str = ".cli-git-marker";

/// What:
///  The name of a staging file of the lock at `lock`:
/// this process,
/// the batch,
/// and a token that no other attempt has.
/// Why:
///   The process number alone recurs once the process is gone,
/// and a name that recurs can be the name of a hard-link alias a killed holder left,
/// which is the lock's own inode;
/// the token makes every name new,
/// and the process number lets a later attempt tell from the name alone whether the writer still runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${lock}.cli-git-marker-${process.pid}-${seq}-${token}`
/// ```
pub fn staged_path_for(lock: &Path, seq: u64, token: &str) -> PathBuf {
    let mut name = lock.as_os_str().to_os_string();
    name.push(format!("{STAGED_INFIX}-{}-{seq}-{token}", std::process::id()));
    return PathBuf::from(name);
}

/// What:
///  Creates the staging file of the lock at `lock` for batch `seq`,
/// new and holding `marker`.
/// Why:
///   Opened with exclusive creation and never truncation,
/// so a name that already exists (whatever it is a name of) is an error and no file of another attempt is emptied.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeFile(stagedPath, marker, { flag: 'wx' })
/// ```
///
/// # Errors
///
/// The storage error when the file cannot be created or written;
/// it is removed again when only the write failed.
/// The file is returned open:
/// the holder keeps it for as long as the hold lasts, because it is the lock's own inode.
pub fn stage_marker(lock: &Path, seq: u64, marker: &str) -> std::io::Result<(PathBuf, std::fs::File)> {
    let token: String = random_uuid().map_err(|error| { return std::io::Error::other(error.0); })?;
    let staged: PathBuf = staged_path_for(lock, seq, token.as_str());
    let mut file = std::fs::OpenOptions::new().read(true).write(true).create_new(true).open(&staged)?;
    if let Err(error) = file.write_all(marker.as_bytes()) {
        drop(file);
        let _removed = std::fs::remove_file(&staged);
        return Err(error);
    }
    return Ok((staged, file));
}

/// Whether the process that a staging file's name carries is gone.
/// A name that carries none,
/// and a process that cannot be asked about,
/// count as running,
/// which keeps the file.
fn staging_writer_is_gone(suffix: &[u8]) -> bool {
    if suffix.is_empty() {
        // The fixed name an earlier release used: no attempt of this or the last release writes it.
        return true;
    }
    let digits: Vec<u8> = suffix.strip_prefix(b"-").unwrap_or(&[]).iter().copied().take_while(u8::is_ascii_digit).collect();
    let Some(pid) = std::str::from_utf8(digits.as_slice()).ok().and_then(|text| { return text.parse::<i64>().ok(); }) else { return false };
    return matches!(process_birth_identity(pid), Ok(None));
}

/// What:
///  Removes the staging files of the lock at `lock` that a killed attempt left:
/// the fixed name an earlier release used,
/// and any whose writer is gone.
/// Why:
///   A kill before the rename leaves the staging file behind.
/// The writer is read from the name,
/// never from the content:
/// a file that is empty or half written is the file of a writer that is still at it,
/// and removing it fails that writer's placement.
/// A live writer's file is its own and stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const name of await readdir(dir)) if (name.startsWith(prefix) && writerIsGone(name)) await rm(name)
/// ```
pub fn remove_dead_staging(lock: &Path) {
    use std::os::unix::ffi::OsStrExt;
    let (Some(directory), Some(file)) = (lock.parent(), lock.file_name()) else { return };
    let mut prefix: Vec<u8> = file.as_bytes().to_vec();
    prefix.extend_from_slice(STAGED_INFIX.as_bytes());
    let Ok(entries) = std::fs::read_dir(directory) else { return };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(suffix) = name.as_bytes().strip_prefix(prefix.as_slice()) else { continue };
        if staging_writer_is_gone(suffix) {
            let _removed = std::fs::remove_file(entry.path());
        }
    }
}

/// What:
///  `stage_marker`,
/// with the staged file flushed to storage by `sync` before it is returned.
/// Why:
///   After a power loss the lock's directory entry can outlast the data of the file it names;
/// a marker flushed before the file becomes the lock leaves either no lock or a complete one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { path, handle } = await stageMarker(lock, seq, marker); await handle.sync()
/// ```
///
/// # Errors
///
/// As `stage_marker`, and the flush failure;
/// the staging file is removed again then.
pub fn stage_marker_synced(sync: &dyn Fn(&std::fs::File) -> std::io::Result<()>, lock: &Path, seq: u64, marker: &str) -> std::io::Result<(PathBuf, std::fs::File)> {
    let (staged, file) = stage_marker(lock, seq, marker)?;
    if let Err(error) = sync(&file) {
        drop(file);
        let _removed = std::fs::remove_file(&staged);
        return Err(error);
    }
    return Ok((staged, file));
}

/// Sets the modification time of the staged file to now.
/// Why:
///   The wrapped commands judge an unreadable or holderless hold by the age of the lock file,
/// and the file is the staged one,
/// which was written before a wait of any length;
/// the time of the placement is the time the hold began.
pub fn refresh_age(file: &std::fs::File) {
    let _refreshed = file.set_modified(std::time::SystemTime::now());
}
