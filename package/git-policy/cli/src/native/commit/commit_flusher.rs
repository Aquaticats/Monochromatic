//! What:
//!  The one-shot receipt flusher: the internal flush entry point that forces `lane/receipt/<id>` files and the receipt directory, under
//!  the non-blocking lock `lane/flusher.v2.lock`, releases the lock after its last empty sweep and sweeps once more without it, taking the
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

use super::commit_helper_launch::{LaunchError, launch_detached_preparing, move_above_streams};
use super::commit_identity::fault_point;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Whether `launch_flusher` behaves as round 19 did (a probe before the fork), for a negative control only.
#[cfg(feature = "test-seams")]
static LEGACY_PROBE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Makes `launch_flusher` probe the lock before it forks, as the removed implementation did (`false` clears it). Test seam: the controls
/// that must show the removed behaviour losing a receipt use it; no production build has it.
#[cfg(feature = "test-seams")]
pub fn inject_legacy_probe(on: bool) {
    LEGACY_PROBE.store(on, std::sync::atomic::Ordering::SeqCst);
}

/// The number of acquisitions that succeed before a test makes one fail; `u32::MAX` is off.
#[cfg(feature = "test-seams")]
static FAIL_AFTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(u32::MAX);

/// Makes the acquisition after the next `n` ones fail with an I/O error (`None` clears it): the lock file cannot be opened for the helper.
#[cfg(feature = "test-seams")]
pub fn inject_lock_failure_after(n: Option<u32>) {
    FAIL_AFTER.store(n.unwrap_or(u32::MAX), std::sync::atomic::Ordering::SeqCst);
}

/// The injected failure of this acquisition, if a test asked for one.
#[cfg(feature = "test-seams")]
fn seam_failure() -> Option<io::Error> {
    use std::sync::atomic::Ordering;
    let left: u32 = FAIL_AFTER.load(Ordering::SeqCst);
    if left == u32::MAX {
        return None;
    }
    if left == 0 {
        FAIL_AFTER.store(u32::MAX, Ordering::SeqCst);
        return Some(io::Error::from_raw_os_error(libc::EIO));
    }
    FAIL_AFTER.store(left - 1, Ordering::SeqCst);
    return None;
}

/// The lane's receipt directory.
pub const RECEIPT_DIR: &str = "receipt";

/// The lane's record of receipts whose forcing failed (V; the next owner's syncer retries them).
pub const RECEIPT_ERRORS: &str = "receipt-errors";

/// The lane's flusher lock: the file name of the new protocol (owner decision 23, "New lock name"). No earlier implementation uses it, so no
/// old client that probes a lock and holds it without sweeping can hold this one; old and new flushers may run at once, both sweep the same
/// receipt directory, and forcing a receipt twice is harmless.
pub const FLUSHER_LOCK: &str = "flusher.v2.lock";

/// The lock file name the earlier implementations used. The new protocol never takes it; it is named so that a test can play the old holder.
pub const LEGACY_FLUSHER_LOCK: &str = "flusher.lock";

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
///  The lane's flusher lock, held for the value's life: an exclusive non-blocking `flock` on `<lane>/flusher.v2.lock`.
/// Why:
///  At most one flusher holds the lock. A client probes it and forks a helper only when it was free; the helper takes the lock itself, so
///  no process but the one that works carries a copy of it. A foreign special file at the path is refused without being opened.
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
    _file: File,
}

impl FlusherLock {
    /// Takes the lock without waiting; `None` when another flusher holds it.
    pub fn try_acquire(lane: &Path) -> Result<Option<FlusherLock>, FlushError> {
        return Self::try_acquire_named(lane, FLUSHER_LOCK);
    }

    /// Takes the lock file `name` of the lane without waiting.
    fn try_acquire_named(lane: &Path, name: &str) -> Result<Option<FlusherLock>, FlushError> {
        #[cfg(feature = "test-seams")]
        if let Some(error) = seam_failure() {
            return Err(FlushError::Io(error));
        }
        use std::os::unix::fs::OpenOptionsExt;
        let path: PathBuf = lane.join(name);
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
            Ok(()) => Ok(Some(FlusherLock { _file: file })),
            Err(std::fs::TryLockError::WouldBlock) => Ok(None),
            Err(std::fs::TryLockError::Error(error)) => Err(FlushError::Io(error)),
        };
    }

}

/// Takes the old implementation's lock without sweeping, as an old probe-capable client did (a test plays the old holder with it); an error
/// when another process holds it.
#[cfg(feature = "test-seams")]
pub fn hold_legacy_lock(lane: &Path) -> Result<FlusherLock, FlushError> {
    return FlusherLock::try_acquire_named(lane, LEGACY_FLUSHER_LOCK)?.ok_or(FlushError::Invalid("the old lock is held by another process"));
}

/// Forces one path (a regular file, or a directory) to disk. The path is opened without following a symbolic link and without blocking, and
/// anything but a regular file or a directory is refused: a FIFO named like a receipt would otherwise block the flusher for ever.
pub fn sync_path(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::OpenOptionsExt;
    let file: File = OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK).open(path)?;
    let kind = file.metadata()?.file_type();
    if !kind.is_file() && !kind.is_dir() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "a receipt is a regular file; this path is something else and is not forced"));
    }
    return file.sync_all();
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
    let mut record: File = OpenOptions::new().create(true).append(true).read(true).open(lane.join(RECEIPT_ERRORS))?;
    // Shared lock for the writes: the syncer empties the record in place under the exclusive lock, so no line is written while it does.
    record.lock_shared()?;
    let mut present: String = String::new();
    std::io::Read::read_to_string(&mut record, &mut present)?;
    for name in names {
        let Some(leaf) = name.file_name() else { continue };
        if !recorded.insert(leaf.to_os_string()) || present.lines().any(|line| { return line.split('\t').next() == leaf.to_str(); }) {
            continue;
        }
        // One write per line: with `O_APPEND` a single write is not interleaved with another recorder's (a launcher and its helper can both
        // record), whereas `writeln!` on an unbuffered file makes several.
        let line: String = format!("{}\t{}\n", leaf.to_string_lossy(), error.to_string().replace('\n', " "));
        record.write_all(line.as_bytes())?;
    }
    return Ok(());
}

/// One sweep: forces every receipt file not yet forced, then the receipt directory, and only then counts the names as forced; returns how
/// many receipts it forced. A receipt whose file was forced but whose directory entry was not is still unforced (a power loss can drop the
/// name), so on an error nothing of this sweep is in `forced`.
fn sweep(lane: &Path, forced: &mut BTreeSet<OsString>, force: &mut dyn FnMut(&Path) -> io::Result<()>) -> io::Result<usize> {
    let pending: Vec<PathBuf> = unforced(lane, forced)?;
    for path in &pending {
        fault_point("flusher:before-force");
        force(path)?;
    }
    if !pending.is_empty() {
        force(&lane.join(RECEIPT_DIR))?;
        // The lane directory holds the entry of the receipt directory itself, which a new lane has just created.
        force(lane)?;
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
///  receipt there. The helper arrives here holding the lock it took itself.
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
        fault_point("flusher:after-last-empty-sweep");
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
    let result: Result<Flushed, FlushError> = flush_lane(lane);
    // A flusher that could not even take the lock records every receipt as not forced, as the fallback helper does, so the next sweep is told.
    if let Err(FlushError::Io(failure)) = &result {
        let mut recorded: BTreeSet<OsString> = BTreeSet::new();
        let _ = unforced(lane, &BTreeSet::new()).and_then(|names| { return record_errors(lane, &names, failure, &mut recorded); });
    }
    return exit_code(result);
}

/// The entry of a helper that took the lane's lock itself.
pub fn flusher_entry_holding(lane: &Path, lock: FlusherLock) -> i32 {
    return exit_code(flush_held(lane, lock, &mut sync_path, &mut || {}));
}

/// What a launch of the flusher did.
#[derive(Debug, Eq, PartialEq)]
pub enum Launched {
    /// A helper was forked and took the lane's lock, or found a flusher holding it: this client's receipt, visible before that failed
    /// acquisition, is covered by the holder's next locked sweep or by its release and recheck (see `launch_flusher`).
    Started,
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
///  Starts the one-shot flusher for `lane` as a detached helper that holds none of this process's descriptors; the helper takes the lane's
///  flusher lock itself. The client takes no lock and looks at none: it always forks a helper. When the helper cannot be started or
///  cannot take the lock, every receipt in the lane is recorded as not forced.
/// Why:
///  Only a process that will list and force after it acquires the lock ever holds it. A client that probed the lock held it without
///  listing anything, and a client or helper that saw it held took that for an executor (review 14); looking at the lock without taking
///  it is impossible with `flock`. A helper that finds the lock held ends. Why that is sound, stated as it is: the receipt was visible
///  before the helper's failed acquisition, and every holder is a flusher (the inventory below), so either the holder has not yet done
///  its last locked sweep, which then lists the receipt, or it has, and then it has still to release the lock and recheck, which lists
///  it, takes the lock again or finds another flusher holding it, and accounts for a forcing failure. (The holder need not have acquired
///  after the receipt existed: it may have swept empty first.) The cost is a fork per client that needs the flusher path, and a helper
///  that ends when another one works, as soon as it runs; the number of helpers is arrival-dependent with no time bound (the owner's
///  decision of 2026-11-04), and the number that hold the lock is one.
///
///  Acquisition sites of `FlusherLock` in production code, all three followed by a locked sweep: the reacquisition of the recheck in
///  `flush_held` (it loops to a sweep), `flush_lane_with` (it calls `flush_held`), and the helper's `prepare` in this function (its
///  entry calls `flush_held`). The tests also hold the lock without sweeping, as fixtures (for example the lock a test takes to show that
///  clients fork helpers that end), and with the `legacy` probe seam below, which reproduces the removed probe for a negative control;
///  neither is a production holder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function launchFlusher(lane: string): 'started'; // throws FlusherLaunchError
/// ```
pub fn launch_flusher(lane: &Path) -> Result<Launched, FlusherLaunchError> {
    #[cfg(feature = "test-seams")]
    if LEGACY_PROBE.load(std::sync::atomic::Ordering::SeqCst) {
        // The removed behaviour, for a negative control: a probe that holds the lock without listing, and a client that returns when it
        // finds the lock held.
        match FlusherLock::try_acquire_named(lane, LEGACY_FLUSHER_LOCK).map_err(FlusherLaunchError::Lock)? {
            Some(probe) => {
                fault_point("flusher:probe-held");
                drop(probe);
            }
            None => return Ok(Launched::Started),
        }
    }
    let owned: PathBuf = lane.to_path_buf();
    let lane_for_helper: PathBuf = lane.to_path_buf();
    return match launch_detached_preparing(
        move || {
            fault_point("flusher:helper-before-acquire");
            let acquired = FlusherLock::try_acquire(&lane_for_helper).map_err(|error| { return io::Error::other(format!("{:?}", error)); });
            if matches!(acquired, Ok(None)) {
                fault_point("flusher:helper-lost-the-lock");
            }
            return acquired;
        },
        move |acquired| {
            // When someone else holds the lock, this client's receipt, visible before this failed acquisition, is covered by that flusher's
            // next locked sweep or by its release and recheck.
            return match acquired {
                Ok(Some(held)) => flusher_entry_holding(&owned, held),
                Ok(None) => 0,
                Err(failure) => {
                    // The helper could not take the lock: it records every receipt of the lane as not forced, as a failed launch does.
                    let mut recorded: BTreeSet<OsString> = BTreeSet::new();
                    let _ = unforced(&owned, &BTreeSet::new()).and_then(|names| { return record_errors(&owned, &names, &failure, &mut recorded); });
                    FLUSHER_FAILED_EXIT
                }
            };
        },
    ) {
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
