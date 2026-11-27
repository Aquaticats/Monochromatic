//! What:
//!  The markers this service keeps made ahead of the landing that asks for one:
//!  the marked file that becomes the index lock,
//!  the slot one making fills,
//!  the record of every file made,
//!  and the one step in which a marker's file is created and recorded.
//! Why:
//!   The file of a marker must be flushed before a landing can place it,
//!  so the making runs beside the landing and its file is recorded
//!  before the service's own sweep may remove it.
//!  They live beside the making and the taking,
//!  because that file is at the line bound the repository linter enforces (`rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // landing/serviceIndexPrestageTheMarkersMadeAhead.ts
//! export type Prestaged = { staged: PathBuf; held: File; marker: string };
//! const madeAhead: MadeAhead[] = [];
//! ```

use crate::blocking_job::BlockingJobError;
use crate::lazy_runtime::run_on_runtime;
use std::os::unix::fs::MetadataExt;
use std::path::{
    Path,
    PathBuf,
};
use std::sync::atomic::AtomicBool;
use tokio::task::JoinHandle;
/// A marked file that is flushed and ready to become the lock.
#[derive(Debug)]
pub struct Prestaged {
    /// Where the file is.
    pub staged: PathBuf,
    /// The file,
    /// open:
    /// the holder keeps it for the hold,
    /// because it is the lock's own inode.
    pub held: std::fs::File,
    /// The text in it.
    pub marker: String,
}

/// The marker being made or made for one lock path and one batch.
pub(crate) struct Slot {
    pub(super) lock: PathBuf,
    pub(super) seq: u64,
    /// Whether the marker is still wanted:
    /// a caller that will not use it,
    /// or the service's end,
    /// clears it,
    /// so the job makes no file and no thread waits for the making.
    pub(super) wanted: std::sync::Arc<AtomicBool>,
    pub(super) work: JoinHandle<Result<std::io::Result<Prestaged>, BlockingJobError>>,
}

/// What:
///  The files the service has made ahead,
/// named for the shutdown that ends the process from another thread:
/// each lock and batch the file was made for,
/// the pathname of its file,
/// and the identity of the file written there.
/// Why:
///   The slots that hold the made-ahead markers belong to the landing thread,
/// and a service stopped by a signal ends the process from its shutdown task,
/// where no destructor of the landing thread runs.
/// The lock and the batch let a slot that will not be used find its own file here and remove it without waiting for
///  the job that makes it,
/// the identity keeps a removal from taking a pathname that now names another file,
///  and an entry is cleared whenever the landing thread takes,
/// places or removes the file.
/// Lock kept:
///  the landing thread writes this list and the shutdown task takes it,
/// so two threads reach it,
/// and a pathname with an identity is not one word an atomic could hold.
/// Under it only the list is changed:
/// no Git process,
/// no file and no other lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let madeAhead: { lock: string; seq: number; path: string; dev: number; ino: number }[] = [];
/// ```
pub(super) static MADE_AHEAD: std::sync::Mutex<Vec<MadeAhead>> = std::sync::Mutex::new(Vec::new());

/// What:
///  The one step in which a marker's file is created and recorded,
/// with the end of the service on the same step.
/// Why:
///   The service's markers are removed from another thread,
/// and a file created but not yet recorded is a file such a removal would miss;
/// so a making holds this step from its read of whether its file is still wanted to the record of the file,
/// and the sweep of the service's end sets that end and takes the records inside the same step:
/// either the sweep sees the file or the making makes none.
/// Lock kept:
///  a making job and the two calls that end the keeping,
/// and nothing of the commit's path,
/// because the step holds a creation and its record and never a flush,
/// so the landing thread's own takes and removals never wait for a flush.
/// Limits:
///  a creation and a record are two file calls the page cache answers;
/// a filesystem that stalls them stalls a take as well,
/// which the flush this step replaced never did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mutex(() => { if (!wanted || serviceEnds) return undefined; const made = stageMarker(); record(made); return made })
/// ```
pub(super) static CREATING_AND_RECORDING: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Whether the service has stopped keeping markers made ahead:
/// from its last sweep on no file is made,
/// so no marker can outlive the sweep that would have removed it.
pub(super) static SERVICE_ENDS: AtomicBool = AtomicBool::new(false);

/// One file made ahead and where it was written.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct MadeAhead {
    /// The lock whose marker it is,
    /// so the slot that made it finds its own file here.
    pub(super) lock: PathBuf,
    /// The batch it was made for.
    pub(super) seq: u64,
    /// The pathname of the file,
    /// which the landing thread renames onto the lock.
    pub(super) path: PathBuf,
    /// The device of the file written there.
    pub(super) device: u64,
    /// The inode of the file written there.
    pub(super) inode: u64,
}

/// What:
///  The step of a making and of the service's end,
/// taken the one way by every holder,
/// and kept after a panic that gave it up,
/// so a making and the sweep still exclude each other.
/// Why:
///   One place takes the lock,
/// so no caller can take it in another order than its two other holders.
/// Lock kept:
///  `CREATING_AND_RECORDING`,
///  for the reason written above it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const step = creatingAndRecording.lock()
/// ```
pub(super) fn in_the_step_of_creating_and_recording() -> std::sync::MutexGuard<'static, ()> {
    return CREATING_AND_RECORDING
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
}

/// Records a file that was just made,
/// so a stop that runs on another thread removes it.
pub(super) fn record_the_file(
    lock: &Path,
    seq: u64,
    made: &Prestaged,
) -> std::io::Result<()> {
    let on_disk: std::fs::Metadata = made.held.metadata()?;
    let Ok(mut kept) = MADE_AHEAD.lock() else {
        return Err(std::io::Error::other(
            "the record of the files made ahead could not be written",
        ));
    };
    kept.push(MadeAhead {
        lock: lock.to_path_buf(),
        seq,
        path: made.staged.clone(),
        device: on_disk.dev(),
        inode: on_disk.ino(),
    });
    return Ok(());
}

/// The error of a marker that was let go,
/// or of one the service stopped keeping,
/// before its file was made:
/// nothing waits for such a job,
/// so no caller reads this.
pub(super) fn no_longer_wanted() -> std::io::Error {
    return std::io::Error::other("the marker made ahead is no longer wanted");
}

/// Removes the file a record names when the pathname still names that file,
/// and leaves any other file at the pathname alone.
pub(super) fn remove_the_recorded_file(made: MadeAhead) {
    let Ok(on_path) = std::fs::symlink_metadata(made.path.as_path()) else {
        return;
    };
    if on_path.file_type().is_file() && on_path.dev() == made.device && on_path.ino() == made.inode
    {
        let _removed = std::fs::remove_file(made.path.as_path());
    }
}

/// Whether a commit has begun to wait for the marker being made,
/// so a control can hold the flush until a commit is waiting for it instead of racing a timer against the flush.
#[cfg(test)]
pub static COMMIT_WAITING: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// What:
///  Whether the pathname of the staged file still names the file that was made and flushed:
/// a regular file,
/// not a link,
/// with the device and inode of the descriptor the service holds.
/// Why:
///   `metadata` follows links,
/// and a link to the original file would pass a check by it and be placed as the link,
/// which is not the file that was flushed.
/// The check is a preflight,
/// not a guard:
/// a replacement after it is not seen.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const on = await lstat(path); on.isFile() && on.ino === (await held.stat()).ino && on.dev === held.dev
/// ```
pub(super) fn still_there(made: &Prestaged) -> bool {
    let (Ok(on_path), Ok(open)) = (
        std::fs::symlink_metadata(&made.staged),
        made.held.metadata(),
    ) else {
        return false;
    };
    return on_path.file_type().is_file()
        && on_path.dev() == open.dev()
        && on_path.ino() == open.ino();
}

/// Waits for the marker being made and gives it;
/// nothing when it could not be made,
/// or the job could not run,
/// or the runtime could not be entered.
/// The service's landing thread is the one that waits,
/// outside the runtime.
pub(super) fn made_marker(
    work: JoinHandle<Result<std::io::Result<Prestaged>, BlockingJobError>>
) -> Option<Prestaged> {
    return match run_on_runtime(work) {
        Ok(Ok(Ok(Ok(made)))) => Some(made),
        // A marker that could not be made, a job that could not run, a task that was cancelled, a runtime that could not start.
        _ => None,
    };
}
