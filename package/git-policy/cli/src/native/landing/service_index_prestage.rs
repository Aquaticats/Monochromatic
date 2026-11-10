//! What:
//!  The markers of the next batches' index locks,
//! created,
//! written and flushed to storage while the service lands the batch before them,
//! so that a commit's lock path only places one.
//! Why:
//!  The marker must be on storage before it can become `index.lock`,
//! so that a power loss never leaves a lock with an
//!  empty or partial marker.
//! That flush is a few milliseconds on an idle disk and seconds on a loaded one,
//! and it sat on every commit's path.
//! The batch numbers of the next landings are known when a landing takes its lock (one more and two more),
//! and so is the index,
//! so both files are made there,
//! while that landing moves the branch and builds the index:
//! a flush overlaps a landing instead of being the commit before it,
//! and the second file is the spare that keeps one marker finished while another is being made.
//! Two files at most exist for the service,
//! they are kept in this thread's slots and not in a path other processes could take for their own,
//! they are removed when the service ends or the batch a marker was made for is passed,
//! and they name this process,
//! so the next service removes what a crash left (`remove_dead_staging`).
//! A commit that arrives while the flush is still running waits for it and does not start a second.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! prestage.start(index, seq + 1); /* idle */ const ready = await prestage.take(index, seq); // undefined: stage cold
//! ```

use crate::process_identity::ProcessIdentityError;
use crate::service_index_lock::lock_path_of;
use crate::service_index_marker::{marker_text, own_identity};
use crate::service_index_placement::sync_file;
use crate::service_index_staging::stage_marker;
use crate::landing_state_carried_by_its_task::{LandingState, on_the_landing_state};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use crate::blocking_job::{BlockingJobError, run_blocking_job};
use crate::lazy_runtime::{run_on_runtime, spawn_on_runtime};
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
    lock: PathBuf,
    seq: u64,
    work: JoinHandle<Result<std::io::Result<Prestaged>, BlockingJobError>>,
}

/// What:
///  The files the service has made ahead,
/// named for the shutdown that ends the process from another thread:
/// each pathname and the identity of the file written there.
/// Why:
///   The slots that hold the made-ahead markers belong to the landing thread,
/// and a service stopped by a signal ends the process from its shutdown task,
/// where no destructor of the landing thread runs.
/// The identity keeps a removal from taking a pathname that now names another file,
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
/// let madeAhead: { path: string; dev: number; ino: number }[] = [];
/// ```
static MADE_AHEAD: std::sync::Mutex<Vec<MadeAhead>> = std::sync::Mutex::new(Vec::new());

/// How many markers the service keeps made ahead:
/// the one the next batch takes and the one after that,
/// so every flush has a whole landing to finish in.
const MARKERS_MADE_AHEAD: usize = 2;

/// One file made ahead and where it was written.
#[derive(Clone, Debug, Eq, PartialEq)]
struct MadeAhead {
    /// The pathname of the file,
    /// which the landing thread renames onto the lock.
    path: PathBuf,
    /// The device of the file written there.
    device: u64,
    /// The inode of the file written there.
    inode: u64,
}

/// Records the file just made and flushed,
/// so a stop that runs on another thread can remove it.
fn publish(made: &Prestaged) {
    let Ok(held) = made.held.metadata() else { return };
    let Ok(mut kept) = MADE_AHEAD.lock() else { return };
    kept.push(MadeAhead { path: made.staged.clone(), device: held.dev(), inode: held.ino() });
}

/// Forgets the file the service made ahead at `path`,
/// because the landing thread took,
/// placed or removed it.
fn forget_made_ahead(path: &Path) {
    let Ok(mut kept) = MADE_AHEAD.lock() else { return };
    let mut remaining: Vec<MadeAhead> = Vec::new();
    for made in std::mem::take(&mut *kept) {
        if made.path != path {
            remaining.push(made);
        }
    }
    *kept = remaining;
}

/// Takes the records of the files made ahead,
/// so that the file calls of their removal run with nothing held.
fn take_made_aheads() -> Vec<MadeAhead> {
    let Ok(mut kept) = MADE_AHEAD.lock() else { return Vec::new() };
    return std::mem::take(&mut kept);
}

/// What:
///  Removes the files the service made ahead,
/// when each is still the file the service wrote,
/// and forgets them either way.
/// Why:
///   The service ends by the signal the manager sent,
/// and `SPEC.md` "Removing the lock and what is made durable" says the markers "are removed when the service ends,
///  whichever way its loop ends";
/// the shutdown task runs no landing-thread destructor,
///  so it removes the files itself.
/// A pathname whose file was placed as the lock,
///  or replaced,
/// is not the service's to remove here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const made of madeAhead) if (lstat(made.path).ino === made.ino && lstat(made.path).dev === made.dev) rm(made.path);
/// ```
pub fn remove_the_file_made_ahead() {
    for made in take_made_aheads() {
        let Ok(on_path) = std::fs::symlink_metadata(made.path.as_path()) else { continue };
        if on_path.file_type().is_file() && on_path.dev() == made.device && on_path.ino() == made.inode {
            let _removed = std::fs::remove_file(made.path.as_path());
        }
    }
}

/// Whether a commit has begun to wait for the marker being made,
/// so a control can hold the flush until a commit is waiting for it instead of racing a timer against the flush.
#[cfg(test)]
pub static COMMIT_WAITING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Takes the pre-staged markers out of the state they are kept in.
fn take_the_slots(state: &LandingState, (): ()) -> Vec<Slot> {
    return state.prestaged_markers.take();
}

/// Keeps `slots` as the landing's pre-staged markers.
fn keep_the_slots(state: &LandingState, slots: Vec<Slot>) {
    *state.prestaged_markers.borrow_mut() = slots;
}

/// Takes the service's pre-staged markers out of the landing's state:
/// the state `prestaged_markers` of `landing_state_carried_by_its_task.rs` is per landing,
/// because only the landing takes locks and lands batches.
fn take_slots() -> Vec<Slot> {
    return on_the_landing_state((), take_the_slots);
}

/// Puts the pre-staged markers back into the landing's state.
fn put_slots(slots: Vec<Slot>) {
    on_the_landing_state(slots, keep_the_slots);
}

/// The error for an identity that could not be read.
fn identity_error(error: ProcessIdentityError) -> std::io::Error {
    return std::io::Error::other(error.to_string());
}

/// What:
///  Starts making the marker of batch `seq` for the index `index`,
/// unless that is already made or being made.
/// Why:
///   A marker for another batch or another index is replaced:
/// it is waited for and removed first,
/// so one file at most exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// prestage.start(index, seq)
/// ```
pub fn start(index: &Path, seq: u64) {
    start_with(sync_file, index, seq);
}

/// `start` with the flush given,
/// which a control makes slow.
pub fn start_with(sync: fn(&std::fs::File) -> std::io::Result<()>, index: &Path, seq: u64) {
    let lock: PathBuf = lock_path_of(index);
    // The markers that can still be used here: every one for this index, and the one this call asks for among them.
    let mut kept: Vec<Slot> = Vec::new();
    let mut same: bool = false;
    for slot in take_slots() {
        if slot.lock == lock {
            same = same || slot.seq == seq;
            kept.push(slot);
        } else {
            // A marker for another index is never used here: its file goes.
            remove_unused(slot);
        }
    }
    if same {
        put_slots(kept);
        return;
    }
    // One marker for each batch and at most `MARKERS_MADE_AHEAD` in all: the oldest goes before another is made.
    while kept.len() >= MARKERS_MADE_AHEAD {
        let mut oldest: usize = 0;
        let mut oldest_seq: u64 = u64::MAX;
        for (at, slot) in kept.iter().enumerate() {
            if slot.seq < oldest_seq {
                oldest_seq = slot.seq;
                oldest = at;
            }
        }
        remove_unused(kept.remove(oldest));
    }
    let work_lock: PathBuf = lock.clone();
    // The flush can wait on the disk, so the marker is made by a job of the blocking pool, which a task of the runtime awaits.
    let spawned = spawn_on_runtime(run_blocking_job(move || -> std::io::Result<Prestaged> {
        let marker: String = marker_text(seq, &own_identity).map_err(identity_error)?;
        // The file is recorded before the flush, because the flush can take seconds on a loaded disk and a stop that
        // arrives while it runs must find the file: a record made after the flush would leave it behind.
        let (staged, held) = stage_marker(work_lock.as_path(), seq, marker.as_str())?;
        let made: Prestaged = Prestaged { staged, held, marker };
        publish(&made);
        if let Err(error) = sync(&made.held) {
            forget_made_ahead(made.staged.as_path());
            drop(made.held);
            let _removed = std::fs::remove_file(made.staged.as_path());
            return Err(error);
        }
        return Ok(made);
    }));
    // No worker to make it on is no marker made ahead: the commit stages its own.
    if let Some(work) = spawned {
        kept.push(Slot { lock, seq, work });
    }
    put_slots(kept);
}

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
fn still_there(made: &Prestaged) -> bool {
    let (Ok(on_path), Ok(open)) = (std::fs::symlink_metadata(&made.staged), made.held.metadata()) else { return false };
    return on_path.file_type().is_file() && on_path.dev() == open.dev() && on_path.ino() == open.ino();
}

/// Removes the staged file when the pathname still names it,
/// and leaves any other file at the pathname alone:
/// the descriptor is dropped either way.
fn remove_if_ours(made: Prestaged) {
    forget_made_ahead(made.staged.as_path());
    if still_there(&made) {
        let _removed = std::fs::remove_file(&made.staged);
    }
}

/// Waits for the marker being made and gives it;
/// nothing when it could not be made,
/// or the job could not run,
/// or the runtime could not be entered.
/// The service's landing thread is the one that waits,
/// outside the runtime.
fn made_marker(work: JoinHandle<Result<std::io::Result<Prestaged>, BlockingJobError>>) -> Option<Prestaged> {
    return match run_on_runtime(work) {
        Ok(Ok(Ok(Ok(made)))) => Some(made),
        // A marker that could not be made, a job that could not run, a task that was cancelled, a runtime that could not start.
        _ => None,
    };
}

/// Removes a marker that will not be used.
fn remove_unused(slot: Slot) {
    if let Some(made) = made_marker(slot.work) {
        remove_if_ours(made);
    }
}

/// What:
///  The marker made for batch `seq` of the index `index`,
/// or nothing when none was made for exactly that.
/// Why:
///   A marker for another batch or index is removed and the commit stages its own.
/// A marker of a later batch is kept:
/// it is the spare the next landing takes.
/// A marker still being flushed is waited for.
/// A file that is no longer the file that was flushed (the Git directory was replaced,
/// or the pathname now names another file)
/// is no marker,
/// and whatever is at the pathname is not the service's to remove.
/// The time all this takes is the caller's to measure,
/// from before the call to after it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ready = await prestage.take(index, seq) // Prestaged | undefined
/// ```
pub fn take(index: &Path, seq: u64) -> Option<Prestaged> {
    let lock: PathBuf = lock_path_of(index);
    let slots: Vec<Slot> = take_slots();
    #[cfg(test)]
    COMMIT_WAITING.store(true, std::sync::atomic::Ordering::SeqCst);
    let mut kept: Vec<Slot> = Vec::new();
    let mut mine: Option<Slot> = None;
    for slot in slots {
        if slot.lock == lock && slot.seq >= seq {
            if slot.seq == seq && mine.is_none() {
                mine = Some(slot);
            } else {
                kept.push(slot);
            }
        } else {
            // A marker for another index, or for a batch this one has passed, is never used: its file goes.
            remove_unused(slot);
        }
    }
    put_slots(kept);
    let slot: Slot = mine?;
    let made: Prestaged = made_marker(slot.work)?;
    forget_made_ahead(made.staged.as_path());
    if !still_there(&made) {
        return None;
    }
    return Some(made);
}

/// Whether any marker made ahead is finished,
/// so that taking it waits for nothing.
#[cfg(test)]
pub fn is_made() -> bool {
    return crate::landing_state_carried_by_its_task::with_the_landing_state(|state| {
        return state.prestaged_markers.borrow().iter().any(|held| { return held.work.is_finished(); });
    });
}

/// Removes every marker made ahead,
/// waiting for a flush that is still running.
pub fn discard() {
    for slot in take_slots() {
        remove_unused(slot);
    }
}

/// Removes the marker made ahead when it goes out of scope,
/// whichever way the service's loop ends.
pub struct PrestageGuard;

impl Drop for PrestageGuard {
    fn drop(&mut self) {
        discard();
    }
}

/// The controls of the pre-staged marker.
#[cfg(test)]
#[path = "service_index_prestage_tests.rs"]
mod tests;
