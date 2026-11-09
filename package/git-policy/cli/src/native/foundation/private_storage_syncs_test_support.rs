//! What:
//!  Test-only records of the durability syncs `private_storage` makes,
//! and the switch that makes them no-ops.
//! Why:
//!  A control reads which files and directories a step synced,
//! which no file on disk shows,
//! and a control that walks every step of a durable routine pays one filesystem
//!  commit per step.
//! Kept in a file of its own so the shipped `private_storage.rs` holds no inline test module (`rust/tests-in-sidecars`);
//! it is declared there under `#[cfg(test)]`,
//! so the release executable never contains it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // a module that exists only in test builds
//! export const syncs = { skipTheDurabilitySyncsHere, isFaithful, recordFileSync, takeFileSyncs, recordDirectorySync, takeDirectorySyncs };
//! ```

use std::path::{Path, PathBuf};

static SYNCING_IS_FAITHFUL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

/// What:
///  Makes this process's durability syncs no-ops,
/// for a control whose subject is not what a crash leaves.
/// Why:
///   A control that walks every step of a routine that syncs pays one filesystem commit per step,
/// and a disk having a bad minute takes up to a second and a half for one of them,
/// so the control's own durability work decides whether it finishes inside the run's own bound.
/// The switch is the process's because the steps sync on threads the control did not make,
/// and a separate test process is what makes that safe (nextest runs one test per process).
/// The records of the syncs are still made,
/// so their order stays observable.
pub(crate) fn skip_the_durability_syncs_here() {
    SYNCING_IS_FAITHFUL.store(false, std::sync::atomic::Ordering::SeqCst);
}

/// Whether this process's syncs must reach the disk.
pub(crate) fn syncing_is_faithful() -> bool {
    return SYNCING_IS_FAITHFUL.load(std::sync::atomic::Ordering::SeqCst);
}

thread_local! {
    /// Paths of the files this thread asked `write_then_publish` to sync,
    /// in call order.
    static FILE_SYNCS: std::cell::RefCell<Vec<PathBuf>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// What:
///  Remember that this thread synced a file it published at `path`.
/// Why:
///   Unit tests pin which records keep their file sync,
/// which no file on disk shows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// syncedFiles.push(path);
/// ```
pub(crate) fn record_file_sync(path: &Path) {
    FILE_SYNCS.with(|syncs| {
        syncs.borrow_mut().push(path.to_path_buf());
    });
}

/// What:
///  Take and clear the files this thread has synced so far.
/// Why:
///   A test calls it before the step under test to drop earlier syncs,
/// and after it to read exactly that step's syncs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const synced = syncedFiles.splice(0);
/// ```
pub fn take_file_syncs() -> Vec<PathBuf> {
    return FILE_SYNCS.with(|syncs| {
        return std::mem::take(&mut *syncs.borrow_mut());
    });
}

thread_local! {
    /// Canonical paths of the directories this thread asked `sync_directory` to flush,
    /// in call order.
    static DIRECTORY_SYNCS: std::cell::RefCell<Vec<PathBuf>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// What:
///  Remember that this thread asked for a directory sync.
/// Why:
///   Unit tests prove that a helper syncs a directory exactly when it changed one,
/// which no file on disk shows;
/// the list is per thread,
/// so parallel tests never see each other's syncs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// syncedDirectories.push(await realpath(path));
/// ```
pub(crate) fn record_directory_sync(path: &Path) {
    let canonical: PathBuf = std::fs::canonicalize(path).unwrap_or_else(|_| return path.to_path_buf());
    DIRECTORY_SYNCS.with(|syncs| {
        syncs.borrow_mut().push(canonical);
    });
}

/// What:
///  Take and clear the directories this thread has synced so far.
/// Why:
///   A test calls it before the step under test to drop earlier syncs,
/// and after it to read exactly that step's syncs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const synced = syncedDirectories.splice(0);
/// ```
pub fn take_directory_syncs() -> Vec<PathBuf> {
    return DIRECTORY_SYNCS.with(|syncs| {
        return std::mem::take(&mut *syncs.borrow_mut());
    });
}
