//! What:
//!  The one-shot receipt flusher: the internal flush entry point that forces `lane/receipt/<id>` files and the receipt directory, under
//!  the non-blocking lock `lane/flusher.lock`, releases the lock after its last empty sweep and sweeps once more without it, taking the
//!  lock again when that sweep finds work. A forcing error is recorded in `lane/receipt-errors` for every receipt whose name is not yet
//!  forced (the receipt directory included), the lock is released, the lane is listed once more for the receipts that joined meanwhile,
//!  and the flusher ends.
//! Why:
//!  The receipt directory is itself the registration of the obligation to force (rule V, "Receipt responsibility"): responsibility never
//!  depends on a message having been delivered. At most one flusher holds the lane lock and forces at a time (a helper that has released it and is in its recheck or exit step holds nothing, and a launch in that step starts another; the number of such helpers is bounded only by the clients that launch in that step, review 10a section 3B), and the recheck sweep means a client that found the
//!  lock held (and so started no flusher) is still covered by the holder. The entry point is a function, started by
//!  `commit_helper_launch::launch_detached` or as a manager's transient unit; it is never the wrapper's ordinary command line.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // launchFlusher(lane): lock first, fork only the holder; flushLane(lane, force): while (sweep()) {}; unlock; if (pending()) { relock or hand over }
//! ```

use super::commit_helper_launch::{Holding, LaunchError, launch_detached_holding, move_above_streams};
use super::commit_identity::fault_point;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// The lane's receipt directory.
pub const RECEIPT_DIR: &str = "receipt";

/// The lane's record of receipts whose forcing failed (V; the next owner's syncer retries them).
pub const RECEIPT_ERRORS: &str = "receipt-errors";

/// The lane's flusher lock.
pub const FLUSHER_LOCK: &str = "flusher.lock";

/// The exit code of a flusher that ended on a forcing error.
pub const FORCING_ERROR_EXIT: i32 = 1;

/// The exit code of a flusher that could not even record its error, or hit another failure.
pub const FLUSHER_FAILED_EXIT: i32 = 2;

/// Why a flush did not complete.
#[derive(Debug)]
pub enum FlushError {
    /// The lane's files could not be read or the lock could not be taken.
    Io(io::Error),
    /// The lock path is not a regular file.
    Invalid(&'static str),
    /// Forcing failed; the receipts not yet forced were recorded in `receipt-errors`.
    Forcing(io::Error),
    /// Forcing failed and the error record could not be written either.
    ForcingUnrecorded { forcing: io::Error, recording: io::Error },
}

impl From<io::Error> for FlushError {
    fn from(error: io::Error) -> FlushError {
        return FlushError::Io(error);
    }
}

/// How a flush that did not fail ended.
#[derive(Debug, Eq, PartialEq)]
pub enum Flushed {
    /// Another flusher holds the lane's lock from the start; this one forced nothing, and the holder's recheck sweep covers its receipts.
    Held,
    /// A sweep without the lock found nothing unforced: `forced` receipts were forced by this run.
    Finished { forced: usize },
    /// The recheck sweep found work and another flusher had taken the lock: that flusher carries it.
    Handed { forced: usize },
}

/// What:
///  The lane's flusher lock, held for the value's life: an exclusive non-blocking `flock` on `<lane>/flusher.lock`.
/// Why:
///  At most one flusher holds the lock, and at most one helper is forked per holder: the launcher takes the lock before it forks and hands the open
///  file description to the helper. A foreign special file at the path is refused without being opened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class FlusherLock { static tryAcquire(lane: string): FlusherLock | null; }
/// ```
#[derive(Debug)]
pub struct FlusherLock {
    // Lock kept: at most one flusher holds it and forces a lane's receipts at a time (the helpers in their release, recheck and exit step hold nothing), and a client forks a helper only when it took this lock, so many
    // outstanding receipts start one process and not one per client. It covers one lane, is never waited for (`try_lock`), and is released
    // after the last empty sweep, before the recheck sweep.
    file: File,
}

impl FlusherLock {
    /// Takes the lock without waiting; `None` when another flusher holds it.
    pub fn try_acquire(lane: &Path) -> Result<Option<FlusherLock>, FlushError> {
        use std::os::unix::fs::OpenOptionsExt;
        let path: PathBuf = lane.join(FLUSHER_LOCK);
        if let Ok(metadata) = fs::symlink_metadata(&path)
            && !metadata.file_type().is_file()
        {
            return Err(FlushError::Invalid("the flusher lock path is not a regular file"));
        }
        let opened: File = OpenOptions::new().create(true).write(true).truncate(false).custom_flags(libc::O_NONBLOCK | libc::O_NOFOLLOW).open(&path)?;
        // The lock is handed to a helper that redirects descriptors 0 to 2; a process with one of them closed would otherwise hold it there.
        // The descriptor has an owner at every step: a failed move closes it.
        let file: File = File::from(move_above_streams(std::os::fd::OwnedFd::from(opened))?);
        return match file.try_lock() {
            Ok(()) => Ok(Some(FlusherLock { file })),
            Err(std::fs::TryLockError::WouldBlock) => Ok(None),
            Err(std::fs::TryLockError::Error(error)) => Err(FlushError::Io(error)),
        };
    }

}

impl Holding for FlusherLock {
    fn descriptors(&self) -> Vec<libc::c_int> {
        use std::os::fd::AsRawFd;
        return vec![self.file.as_raw_fd()];
    }
}

/// Forces one path (a file, or the receipt directory) to disk.
pub fn sync_path(path: &Path) -> io::Result<()> {
    return File::open(path)?.sync_all();
}

/// The receipts of `lane` whose names are not in `forced`, in name order; a lane without a receipt directory has none.
fn unforced(lane: &Path, forced: &BTreeSet<OsString>) -> io::Result<Vec<PathBuf>> {
    let directory: PathBuf = lane.join(RECEIPT_DIR);
    let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let mut found: Vec<PathBuf> = Vec::new();
    for listed in entries {
        let entry = listed?;
        if !forced.contains(&entry.file_name()) {
            found.push(entry.path());
        }
    }
    found.sort();
    return Ok(found);
}

/// Appends one line per receipt in `names` that `recorded` does not hold yet to `lane/receipt-errors` (not forced: the record is V, and the
/// next owner's startup sweep retries every receipt in the directory whether or not it is named here), and adds them to `recorded`.
fn record_errors(lane: &Path, names: &[PathBuf], error: &io::Error, recorded: &mut BTreeSet<OsString>) -> io::Result<()> {
    let mut record: File = OpenOptions::new().create(true).append(true).open(lane.join(RECEIPT_ERRORS))?;
    for name in names {
        let Some(leaf) = name.file_name() else { continue };
        if !recorded.insert(leaf.to_os_string()) {
            continue;
        }
        writeln!(record, "{}\t{}", leaf.to_string_lossy(), error)?;
    }
    return Ok(());
}

/// One sweep: forces every receipt file not yet forced, then the receipt directory, and only then counts the names as forced; returns how
/// many receipts it forced. A receipt whose file was forced but whose directory entry was not is still unforced (a power loss can drop the
/// name), so on an error nothing of this sweep is in `forced`.
fn sweep(lane: &Path, forced: &mut BTreeSet<OsString>, force: &mut dyn FnMut(&Path) -> io::Result<()>) -> io::Result<usize> {
    let pending: Vec<PathBuf> = unforced(lane, forced)?;
    for path in &pending {
        force(path)?;
    }
    if !pending.is_empty() {
        force(&lane.join(RECEIPT_DIR))?;
    }
    for path in &pending {
        if let Some(leaf) = path.file_name() {
            forced.insert(leaf.to_os_string());
        }
    }
    return Ok(pending.len());
}

/// The error path of a flush that holds the lock: records every receipt whose name is not forced, releases the lock, and lists the lane once
/// more, recording the receipts that joined between the first listing and the release (a client that found the lock held started no
/// flusher). A client later than the release finds the lock free and starts its own. Returns the error to hand back.
fn end_on_forcing_error(lane: &Path, forced: &BTreeSet<OsString>, lock: FlusherLock, error: io::Error, after_release: &mut dyn FnMut()) -> FlushError {
    let mut recorded: BTreeSet<OsString> = BTreeSet::new();
    let first: io::Result<Vec<PathBuf>> = unforced(lane, forced);
    fault_point("flusher:after-error-snapshot");
    let mut failed_recording: Option<io::Error> = None;
    match first {
        Ok(names) => {
            if let Err(failed) = record_errors(lane, &names, &error, &mut recorded) {
                failed_recording = Some(failed);
            }
        }
        Err(failed) => failed_recording = Some(failed),
    }
    drop(lock);
    after_release();
    match unforced(lane, forced) {
        Ok(names) => {
            if let Err(failed) = record_errors(lane, &names, &error, &mut recorded) {
                failed_recording = failed_recording.or(Some(failed));
            }
        }
        Err(failed) => failed_recording = failed_recording.or(Some(failed)),
    }
    return match failed_recording {
        None => FlushError::Forcing(error),
        Some(recording) => FlushError::ForcingUnrecorded { forcing: error, recording },
    };
}

/// What:
///  Runs the flusher for `lane` with the lock already held: sweeps until a sweep finds nothing unforced; releases the lock; calls
///  `after_release`; sweeps once more without the lock and, if that sweep finds work, takes the lock again (or hands over when another
///  flusher has it). A forcing error ends the flush through `end_on_forcing_error`.
/// Why:
///  `after_release` is the window where a client that found the lock held has created its receipt and started no flusher; the tests put a
///  receipt there. The launcher takes the lock before it forks and the helper arrives here holding it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function flushHeld(lane: string, lock: Lock, force: (path: string) => void, afterRelease: () => void): Flushed; // throws FlushError
/// ```
pub fn flush_held(lane: &Path, first_lock: FlusherLock, force: &mut dyn FnMut(&Path) -> io::Result<()>, after_release: &mut dyn FnMut()) -> Result<Flushed, FlushError> {
    let mut forced: BTreeSet<OsString> = BTreeSet::new();
    let mut total: usize = 0;
    let mut lock: FlusherLock = first_lock;
    loop {
        loop {
            match sweep(lane, &mut forced, force) {
                Ok(0) => break,
                Ok(count) => total += count,
                Err(error) => return Err(end_on_forcing_error(lane, &forced, lock, error, after_release)),
            }
        }
        drop(lock);
        after_release();
        if unforced(lane, &forced)?.is_empty() {
            return Ok(Flushed::Finished { forced: total });
        }
        match FlusherLock::try_acquire(lane)? {
            Some(next) => lock = next,
            None => return Ok(Flushed::Handed { forced: total }),
        }
    }
}

/// What:
///  Runs the flusher for `lane`: takes the lock without waiting (`Held` when another flusher has it) and flushes with `flush_held`.
/// Why:
///  The entry for callers that did not launch a helper themselves; the tests drive it with their own forcing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function flushLaneWith(lane: string, force: (path: string) => void, afterRelease: () => void): Flushed; // throws FlushError
/// ```
pub fn flush_lane_with(lane: &Path, force: &mut dyn FnMut(&Path) -> io::Result<()>, after_release: &mut dyn FnMut()) -> Result<Flushed, FlushError> {
    let Some(lock) = FlusherLock::try_acquire(lane)? else {
        return Ok(Flushed::Held);
    };
    return flush_held(lane, lock, force, after_release);
}

/// Runs the flusher for `lane` with the real forcing (`sync_path`).
pub fn flush_lane(lane: &Path) -> Result<Flushed, FlushError> {
    return flush_lane_with(lane, &mut sync_path, &mut || {});
}

/// The exit code of a flush that ended as `result`: 0 when it finished, held or handed over.
fn exit_code(result: Result<Flushed, FlushError>) -> i32 {
    return match result {
        Ok(_) => 0,
        Err(FlushError::Forcing(_)) => FORCING_ERROR_EXIT,
        Err(_) => FLUSHER_FAILED_EXIT,
    };
}

/// What:
///  The internal flush entry point: flushes `lane` and returns the exit code of the helper (0 when it finished, held or handed over).
/// Why:
///  A function, never a command line; a manager's transient unit ends here, and `launch_flusher` ends in `flusher_entry_holding`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function flusherEntry(lane: string): number;
/// ```
pub fn flusher_entry(lane: &Path) -> i32 {
    return exit_code(flush_lane(lane));
}

/// The entry of a helper that was handed the lane's lock by its launcher.
pub fn flusher_entry_holding(lane: &Path, lock: FlusherLock) -> i32 {
    return exit_code(flush_held(lane, lock, &mut sync_path, &mut || {}));
}

/// What a launch of the flusher did.
#[derive(Debug, Eq, PartialEq)]
pub enum Launched {
    /// A helper was forked and holds the lane's lock: it flushes this client's receipt.
    Started,
    /// A flusher already holds the lock, so no helper was forked: the holder's recheck sweep covers this client's receipt.
    LockHeld,
}

/// Why the flusher was not launched.
#[derive(Debug)]
pub enum FlusherLaunchError {
    /// The lock could not be taken (not a refusal by another holder).
    Lock(FlushError),
    /// The helper could not be started; every receipt of the lane was recorded as not forced.
    Launch(LaunchError),
    /// The helper could not be started and the receipts could not be recorded either.
    LaunchUnrecorded { launch: LaunchError, recording: io::Error },
}

/// What:
///  Starts the one-shot flusher for `lane` as a detached helper that holds none of this process's descriptors but the lane's flusher lock,
///  which this call took first and handed over (the launcher's copy is closed before the helper works); when another flusher holds the
///  lock nothing is forked. When the helper cannot be started, the lock is dropped and every receipt in the lane, those of clients that
///  found the lock held meanwhile included, is recorded as not forced.
/// Why:
///  Review 9a section 3C: forking before trying the lock gave one live helper per client. Review 10a section 3A: the launcher's copy of the
///  lock must not outlive the helper's release, and clients that joined a launch that failed must not be left with no executor and no record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function launchFlusher(lane: string): 'started' | 'lock-held'; // throws FlusherLaunchError
/// ```
pub fn launch_flusher(lane: &Path) -> Result<Launched, FlusherLaunchError> {
    let Some(lock) = FlusherLock::try_acquire(lane).map_err(FlusherLaunchError::Lock)? else {
        return Ok(Launched::LockHeld);
    };
    let owned: PathBuf = lane.to_path_buf();
    return match launch_detached_holding(lock, move |held| { return flusher_entry_holding(&owned, held); }) {
        Ok(()) => Ok(Launched::Started),
        Err(launch) => {
            // The launch dropped the lock; a client that arrives now finds it free and starts its own flusher.
            let failure: io::Error = io::Error::other(launch.to_string());
            let mut recorded: BTreeSet<OsString> = BTreeSet::new();
            let outcome: io::Result<()> = unforced(lane, &BTreeSet::new()).and_then(|names| { return record_errors(lane, &names, &failure, &mut recorded); });
            Err(match outcome {
                Ok(()) => FlusherLaunchError::Launch(launch),
                Err(recording) => FlusherLaunchError::LaunchUnrecorded { launch, recording },
            })
        }
    };
}
