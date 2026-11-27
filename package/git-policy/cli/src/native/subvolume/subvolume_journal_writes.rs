//! What:
//!  The two durable writes the journal makes:
//! one file written to a sibling created exclusively and renamed over its target,
//! and the error for a journal file that cannot be understood.
//! Why:
//!  A kill must leave either the old journal or the new one,
//! never half of either,
//! so the new text is written to a name made for that write alone,
//! synchronized,
//! and renamed,
//! with the directory that holds the new name made durable too;
//! the names such a write may leave behind are `subvolume_journal_leftover_names.rs`,
//! and the journal that uses these writes is `subvolume_journal.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await writeExclusively(sibling, bytes); await rename(sibling, target); await fsync(dirname(target));
//! ```

use crate::subvolume_error::MigrateError;
use crate::subvolume_file_write_steps::{
    sync_directory_to_disk,
    write_all_then_sync,
};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

/// The permissions of the journal:
/// its owner reads and writes it.
pub(crate) const PRIVATE_FILE_MODE: u32 = 0o600;

/// The error for a journal file that cannot be understood:
/// it says what is wrong with the file at `path`.
pub(crate) fn broken_journal(
    path: &Path,
    why: &str,
) -> MigrateError {
    return MigrateError::Journal(format!(
        "the journal {} {why}; read it, and delete it only if no migration is in progress",
        path.display()
    ));
}

/// Write `bytes` to a new file at `sibling`,
/// rename it over `target`,
/// and make the rename durable with the directory that holds the new name.
pub(crate) fn write_and_rename(
    sibling: &Path,
    target: &Path,
    bytes: &[u8],
) -> std::io::Result<()> {
    write_exclusively(sibling, bytes)?;
    if let Err(error) = std::fs::rename(sibling, target) {
        let _removed: std::io::Result<()> = std::fs::remove_file(sibling);
        return Err(error);
    }
    // The rename is made durable with the directory that holds the new name.
    return match target.parent() {
        Some(directory) => sync_directory_to_disk(directory),
        None => Ok(()),
    };
}

/// What:
///  Write `bytes` to a file made for them alone at `path`:
/// created exclusively,
/// so a path that already exists (a plain file,
///  a link,
/// a dangling link) is refused and never truncated or followed;
/// synchronized;
/// removed again when the write fails.
/// Why:
///   A fixed temporary name that an earlier run or another user left or planted must not redirect the write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeFile(path, bytes, { flag: 'wx', mode: 0o600 }); // rejects with EEXIST on anything there, a link included
/// ```
pub(crate) fn write_exclusively(
    path: &Path,
    bytes: &[u8],
) -> std::io::Result<()> {
    let mut file: std::fs::File = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(PRIVATE_FILE_MODE)
        .open(path)?;
    let written: std::io::Result<()> = write_all_then_sync(&mut file, bytes);
    if let Err(error) = written {
        let _removed: std::io::Result<()> = std::fs::remove_file(path);
        return Err(error);
    }
    return Ok(());
}
