//! What:
//!  The one-shot receipt flusher: the internal flush entry point that forces `lane/receipt/<id>` files and the receipt directory, under
//!  the non-blocking lock `lane/flusher.lock`, releases the lock after its last empty sweep and sweeps once more without it, taking the
//!  lock again when that sweep finds work. A forcing error is recorded in `lane/receipt-errors` for every receipt not yet forced, and
//!  the flusher ends.
//! Why:
//!  The receipt directory is itself the registration of the obligation to force (rule V, "Receipt responsibility"): responsibility never
//!  depends on a message having been delivered. At most one flusher runs per lane, and the recheck sweep means a client that found the
//!  lock held (and so started no flusher) is still covered by the holder. The entry point is a function, started by
//!  `commit_helper_launch::launch_detached` or as a manager's transient unit; it is never the wrapper's ordinary command line.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // flushLane(lane, force): lock; while (sweep()) {}; unlock; if (pending()) { relock or hand over }
//! ```

use super::commit_helper_launch::{LaunchError, launch_detached};
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
///  At most one flusher per lane. A foreign special file at the path is refused without being opened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class FlusherLock { static tryAcquire(lane: string): FlusherLock | null; }
/// ```
#[derive(Debug)]
pub struct FlusherLock {
    // Lock kept: at most one flusher forces a lane's receipts at a time, so many outstanding receipts start one process and not one per
    // client. It covers one lane, is never waited for (`try_lock`), and is released after the last empty sweep, before the recheck sweep.
    _file: File,
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
        let file: File = OpenOptions::new().create(true).write(true).truncate(false).custom_flags(libc::O_NONBLOCK | libc::O_NOFOLLOW).open(&path)?;
        return match file.try_lock() {
            Ok(()) => Ok(Some(FlusherLock { _file: file })),
            Err(std::fs::TryLockError::WouldBlock) => Ok(None),
            Err(std::fs::TryLockError::Error(error)) => Err(FlushError::Io(error)),
        };
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

/// Appends one line per receipt in `names` to `lane/receipt-errors` (not forced: the record is V, and the next owner's startup sweep
/// retries every receipt in the directory whether or not it is named here).
fn record_errors(lane: &Path, names: &[PathBuf], error: &io::Error) -> io::Result<()> {
    let mut record: File = OpenOptions::new().create(true).append(true).open(lane.join(RECEIPT_ERRORS))?;
    for name in names {
        let leaf: String = name.file_name().map_or_else(String::new, |leaf| { return leaf.to_string_lossy().into_owned(); });
        writeln!(record, "{}\t{}", leaf, error)?;
    }
    return Ok(());
}

/// One sweep: forces every receipt not yet forced, then the receipt directory; returns how many receipts it forced. On a forcing error it
/// records every receipt still unforced (the ones a client created after the snapshot included) and returns the error.
fn sweep(lane: &Path, forced: &mut BTreeSet<OsString>, force: &mut dyn FnMut(&Path) -> io::Result<()>) -> Result<usize, FlushError> {
    let pending: Vec<PathBuf> = unforced(lane, forced)?;
    let mut count: usize = 0;
    for path in &pending {
        if let Err(error) = force(path) {
            let remaining: Vec<PathBuf> = unforced(lane, forced)?;
            return Err(match record_errors(lane, &remaining, &error) {
                Ok(()) => FlushError::Forcing(error),
                Err(recording) => FlushError::ForcingUnrecorded { forcing: error, recording },
            });
        }
        if let Some(leaf) = path.file_name() {
            forced.insert(leaf.to_os_string());
        }
        count += 1;
    }
    if count > 0 {
        let directory: PathBuf = lane.join(RECEIPT_DIR);
        if let Err(error) = force(&directory) {
            for path in &pending {
                if let Some(leaf) = path.file_name() {
                    forced.remove(leaf);
                }
            }
            let remaining: Vec<PathBuf> = unforced(lane, forced)?;
            return Err(match record_errors(lane, &remaining, &error) {
                Ok(()) => FlushError::Forcing(error),
                Err(recording) => FlushError::ForcingUnrecorded { forcing: error, recording },
            });
        }
    }
    return Ok(count);
}

/// What:
///  Runs the flusher for `lane`: under the lock, sweeps until a sweep finds nothing unforced; releases the lock; calls `after_release`;
///  sweeps once more without the lock and, if that sweep finds work, takes the lock again (or hands over when another flusher has it).
/// Why:
///  `after_release` is the window where a client that found the lock held has created its receipt and started no flusher; the tests put a
///  receipt there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function flushLaneWith(lane: string, force: (path: string) => void, afterRelease: () => void): Flushed; // throws FlushError
/// ```
pub fn flush_lane_with(lane: &Path, force: &mut dyn FnMut(&Path) -> io::Result<()>, after_release: &mut dyn FnMut()) -> Result<Flushed, FlushError> {
    let mut forced: BTreeSet<OsString> = BTreeSet::new();
    let Some(mut lock) = FlusherLock::try_acquire(lane)? else {
        return Ok(Flushed::Held);
    };
    let mut total: usize = 0;
    loop {
        while sweep_counted(lane, &mut forced, force, &mut total)? {}
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

/// One counted sweep; true when it forced something (so another sweep is needed to prove the lane empty).
fn sweep_counted(lane: &Path, forced: &mut BTreeSet<OsString>, force: &mut dyn FnMut(&Path) -> io::Result<()>, total: &mut usize) -> Result<bool, FlushError> {
    let count: usize = sweep(lane, forced, force)?;
    *total += count;
    return Ok(count > 0);
}

/// Runs the flusher for `lane` with the real forcing (`sync_path`).
pub fn flush_lane(lane: &Path) -> Result<Flushed, FlushError> {
    return flush_lane_with(lane, &mut sync_path, &mut || {});
}

/// What:
///  The internal flush entry point: flushes `lane` and returns the exit code of the helper (0 when it finished, held or handed over).
/// Why:
///  A function, never a command line; `launch_flusher` and a manager's transient unit both end here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function flusherEntry(lane: string): number;
/// ```
pub fn flusher_entry(lane: &Path) -> i32 {
    return match flush_lane(lane) {
        Ok(_) => 0,
        Err(FlushError::Forcing(_)) => FORCING_ERROR_EXIT,
        Err(_) => FLUSHER_FAILED_EXIT,
    };
}

/// Starts the one-shot flusher for `lane` as a detached helper that holds none of this process's descriptors.
pub fn launch_flusher(lane: &Path) -> Result<(), LaunchError> {
    let owned: PathBuf = lane.to_path_buf();
    return launch_detached(move || { return flusher_entry(&owned); });
}
