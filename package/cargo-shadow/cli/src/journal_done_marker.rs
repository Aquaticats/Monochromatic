//! What:
//! The completion marker of a journal:
//! its publication by an atomic rename,
//! its removal,
//! and the finish that does both.
//! Why:
//! Split out of `journal_transaction.rs`,
//! which writes the journal of a run,
//! to keep that file under the code-line limit.
//! The items are re-exported from `journal_transaction`,
//! so their paths there are unchanged.

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
/// Imports `std::fs::{self, File, OpenOptions}` for this file.
use std::fs::{self};
// Imports `std::io`.
/// Imports `std::io` for this file.
use std::io::{self};
// Imports `std::path::{Path, PathBuf}`.
// What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
// Why:      Joined or stored paths are owned; paths passed in are borrowed.
//
// In TS you'd write (pseudocode):
// ```ts
// // TS has one type for this: string
// ```
/// Imports `std::path::{Path, PathBuf}` for this file.
use std::path::{Path, PathBuf};


// Imports `crate::journal::{DONE_SUFFIX, GROUPS_FILE, ...`.
/// Imports `crate::journal::{DONE_SUFFIX, GROUPS_FILE, INDEX_FILE, JOURNAL_DIR, MANIFEST_INDEX_FILE, KEPT_FILES_WARNING_COUNT, QUARANTINE_DIR, OWNER_FILE, READY_FILE, WORKSPACE_LOCK, atomic_write, create_directories_synced, make_durable, sync_directory, write_synced}` for this file.
use crate::journal::{DONE_SUFFIX, sync_directory};

/// Imports `crate::journal_finish::FinishError` for this file.
use crate::journal_finish::FinishError;


/// What:
/// `pub fn publish_done(directory: &Path) -> io::Result<PathBuf>` renames a journal to its `.done` name and syncs the directory that
/// lists it,
/// and returns the `.done` path.
/// Why:
/// The rename is the completion marker,
/// and it counts only once the parent's sync has returned
/// (round 7,
/// 6b2 5:
/// a failure here is not a cleanup problem).
/// The parent is synced even when the name already ends in `.done` (review finding 4b2 5):
/// another run's rename may not be durable yet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function publishDone(directory: string): string
/// ```
pub fn publish_done(directory: &Path) -> io::Result<PathBuf> {
    let parent: PathBuf = match directory.parent() {
        Some(found) => found.to_path_buf(),
        None => PathBuf::from("."),
    };
    let name: String = match directory.file_name() {
        Some(found) => found.to_string_lossy().to_string(),
        None => String::new(),
    };
    let done: PathBuf = if name.ends_with(DONE_SUFFIX) { directory.to_path_buf() } else { parent.join(format!("{name}{DONE_SUFFIX}")) };
    // The two real steps of the marker each have a failpoint, so a test fails the rename itself and the sync that follows it.
    #[cfg(test)]
    if crate::failpoints::fires(crate::failpoints::Point::CompletionRename) {
        return Err(io::Error::other("injected failure of the completion rename"));
    }
    if done != directory {
        fs::rename(directory, &done)?;
    }
    #[cfg(test)]
    crate::bounded_run::trace(format_args!("marker renamed"));
    #[cfg(test)]
    if crate::failpoints::fires(crate::failpoints::Point::CompletionSync) {
        return Err(io::Error::other("injected failure of the completion directory sync"));
    }
    sync_marker_parent(&parent)?;
    return Ok(done);
}

/// What:
/// `fn sync_marker_parent(parent: &Path) -> io::Result<()>` makes the entry of the completion marker durable.
/// Why:
/// The marker counts only once this has returned;
/// a test build traces the success,
/// so the order of finishing is observed at the call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function syncMarkerParent(parent: string): void
/// ```
pub(crate) fn sync_marker_parent(parent: &Path) -> io::Result<()> {
    sync_directory(parent)?;
    #[cfg(test)]
    crate::bounded_run::trace(format_args!("marker parent synced"));
    return Ok(());
}

/// What:
/// `pub fn remove_done(done: &Path) -> io::Result<()>` removes the journal directory and syncs its parent.
/// Why:
/// Nothing in a `.done` journal is needed any more.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function removeDone(done: string): void
/// ```
pub fn remove_done(done: &Path) -> io::Result<()> {
    fs::remove_dir_all(done)?;
    return match done.parent() {
        Some(parent) => sync_directory(parent),
        None => Ok(()),
    };
}

/// What:
/// `pub fn mark_done_and_remove(directory: &Path) -> io::Result<()>` publishes the completion marker of a journal and removes it.
/// Why:
/// A recovery settles a journal in one step;
/// finishing a transaction uses the two phases separately.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function markDoneAndRemove(directory: string): void
/// ```
pub fn mark_done_and_remove(directory: &Path) -> io::Result<()> {
    return finish_journal(directory).map_err(finish_failure_text);
}

/// What:
/// `fn finish_failure_text(error: FinishError) -> io::Error` turns a failure to finish a journal into an `io::Error` whose text is the cause when there is one,
/// else the failure's own text.
/// Why:
/// `Result::map_err` takes it by path instead of a closure.
pub(crate) fn finish_failure_text(error: FinishError) -> io::Error {
    let text: String = match std::error::Error::source(&error) {
        Some(cause) => cause.to_string(),
        None => error.to_string(),
    };
    return io::Error::other(text);
}

/// What:
/// `pub fn finish_journal(directory: &Path) -> Result<(), FinishError>` publishes the completion marker of a journal and removes it,
/// and tells the phase that failed.
/// Why:
/// A recovery that cannot publish the durable marker has not finished the journal and its caller must stop;
/// the removal of a journal that is already marked done is the cleanup,
/// which only warns (V5,
/// certification second pass).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function finishJournal(directory: string): void  // throws a FinishError with its phase
/// ```
pub fn finish_journal(directory: &Path) -> Result<(), FinishError> {
    let done: PathBuf = publish_done(directory).map_err(FinishError::completion)?;
    // The removal of a journal that is already marked done is the cleanup, which a test can make fail after the marker (V5).
    #[cfg(test)]
    crate::bounded_run::trace(format_args!("cleanup attempted"));
    #[cfg(test)]
    if crate::failpoints::fires(crate::failpoints::Point::FinishCleanup) {
        return Err(FinishError::cleanup(io::Error::other("injected cleanup failure")));
    }
    return remove_done(&done).map_err(FinishError::cleanup);
}
