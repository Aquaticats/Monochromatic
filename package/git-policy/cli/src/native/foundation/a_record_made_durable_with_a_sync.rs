//! What:
//!  Making a record the wrapper already wrote survive a power loss:
//!  the flush of its file,
//!  the sync of the folder whose entry names it,
//!  and the switch a control makes both no-ops with.
//! Why:
//!  A record is durable only once its bytes and the entry that names it reached the disk,
//!  and a filesystem with no folder flush at all
//!  (procfs and some network and userspace filesystems) answers one of three unsupported codes,
//!  which counts as synced instead of failing every commit.
//!  `private_storage.rs` calls the file flush and the folder sync from the routine that publishes a record,
//!  and both live here because this is the one place the wrapper makes anything durable.
//!  A control reads what a step synced from the record this file's sidecar keeps.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await handle.sync(); await syncDirectory(dirname(path));
//! ```

/// Diagnostics for a decision no failure reports.
use crate::diagnostic_log::debug;
/// An open file handle,
/// which a flush needs and which opens a folder for its sync.
use std::fs::File;
/// A borrowed filesystem path of raw bytes.
use std::path::Path;

/// Whether this process's syncs must reach the disk;
/// always,
/// in the release executable.
#[cfg(test)]
pub(crate) fn syncing_is_faithful() -> bool {
    return syncs_test_support::syncing_is_faithful();
}

/// Always,
/// in the release executable.
#[cfg(not(test))]
pub(crate) fn syncing_is_faithful() -> bool {
    return true;
}

/// What:
///  Flushes a file this module did not open to disk,
/// when this process's syncs must reach the disk,
/// and records the sync either way.
/// Why:
///  A writer with a handle of its own (a worktree-copy install log,
/// a registration's owner file) is durability work of the same kind as the records published here,
/// so it obeys the same answer a control gives with `skip_the_durability_syncs_here`;
/// otherwise that control still waits for a filesystem commit it never checks.
/// The record is made whatever the answer,
/// so the order of a routine's flushes stays observable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const flushFileWhenFaithful = async (handle: FileHandle, path: string) => { recordFileSync(path); if (isFaithful()) await handle.sync(); };
/// ```
pub(crate) fn flush_file_when_faithful(
    file: &File,
    path: &Path,
) -> std::io::Result<()> {
    // The path is read only by the test-only record; a release build leaves it unused.
    #[cfg(not(test))]
    let _path_for_tests: &Path = path;
    #[cfg(test)]
    record_file_sync(path);
    if !syncing_is_faithful() {
        return Ok(());
    }
    // `sync_all` flushes data and metadata, the incumbent's `handle.sync()`.
    return file.sync_all();
}

/// What:
///  Flushes the staging file to disk when the write is durable,
/// and does nothing otherwise.
/// Why:
///  `write_then_publish` runs it only after the bytes were written;
/// a record that tolerates a torn write skips the sync.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (durable) { await handle.sync(); }
/// ```
pub(super) fn sync_when_durable(
    file: &File,
    path: &Path,
    durable: bool,
) -> std::io::Result<()> {
    if !durable {
        return Ok(());
    }
    return flush_file_when_faithful(file, path);
}

/// What:
///  Whether a failed directory sync means the filesystem has no such operation,
/// not that the flush failed.
/// Why:
///   `EINVAL`,
/// `ENOTSUP` and `ENOSYS` are what a filesystem without the operation returns;
/// `EIO`,
/// `ENOSPC` and
///  `EDQUOT` mean the flush itself failed and stay errors.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['EINVAL', 'ENOTSUP', 'ENOSYS'].includes(error.code)
/// ```
#[cfg(unix)]
fn directory_sync_is_unsupported(error: &std::io::Error) -> bool {
    return matches!(
        error.raw_os_error(),
        Some(libc::EINVAL | libc::ENOTSUP | libc::ENOSYS)
    );
}

/// What:
///  The outcome of syncing the directory at `path`:
/// a filesystem without the operation counts as synced,
/// any other
///  failure stays an error.
/// Why:
///   A filesystem with no directory flush (procfs,
/// some network and userspace filesystems) answers with one of the
///  unsupported codes;
/// the entry cannot be made durable there by any means,
/// and failing would stop every commit.
/// A
///  function of the sync result lets a test feed the flush failures (`EIO`,
/// `ENOSPC`,
/// `EDQUOT`) that no
///  unprivileged filesystem produces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (error !== undefined && !['EINVAL', 'ENOTSUP', 'ENOSYS'].includes(error.code)) throw error;
/// ```
#[cfg(unix)]
pub(crate) fn settle_directory_sync(
    path: &Path,
    outcome: std::io::Result<()>,
) -> std::io::Result<()> {
    return match outcome {
        Ok(()) => Ok(()),
        Err(error) if directory_sync_is_unsupported(&error) => {
            debug(
                "syncDirectory",
                format!(
                    "the filesystem of {} cannot sync a directory ({error}); skipped",
                    path.display()
                )
                .as_str(),
            );
            Ok(())
        }
        Err(error) => Err(error),
    };
}

/// What:
///  Make a directory's entries durable,
///  where the platform supports directory handles.
/// Why:
///   A new or renamed entry survives a crash only after its directory is synced;
///  Windows
///       has no directory handle for this,
///  as the incumbent's `syncDirectory` notes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function syncDirectory(path: string): Promise<void>;
/// ```
pub fn sync_directory(path: &Path) -> std::io::Result<()> {
    #[cfg(test)]
    record_directory_sync(path);
    if !syncing_is_faithful() {
        return Ok(());
    }
    #[cfg(unix)]
    {
        // Opening a directory read-only and syncing it is the POSIX way to flush its entries.
        let directory: File = File::open(path)?;
        settle_directory_sync(
            path,
            rustix::fs::fsync(&directory).map_err(std::io::Error::from),
        )?;
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    let _ = path;
    return Ok(());
}

#[cfg(test)]
#[path = "private_storage_syncs_test_support.rs"]
mod syncs_test_support;

#[cfg(test)]
pub(crate) use syncs_test_support::skip_the_durability_syncs_here;
#[cfg(test)]
pub(crate) use syncs_test_support::{
    record_directory_sync,
    record_file_sync,
    take_directory_syncs,
    take_file_syncs,
};
