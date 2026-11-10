//! What:
//!  The marker of the next batch's index lock,
//! created,
//! written and flushed to storage while the service is idle,
//! so that a commit's lock path only places it.
//! Why:
//!  The marker must be on storage before it can become `index.lock`,
//! so that a power loss never leaves a lock with an
//!  empty or partial marker.
//! That flush is a few milliseconds on an idle disk and seconds on a loaded one,
//! and it sat on every commit's path.
//! The batch number of the next landing is known when the last one ends (one more),
//! and so is the index,
//! so the file is made then.
//! One file at most exists for the service,
//! it is kept in this thread's slot and not in a file other processes could take for their own,
//! it is removed when the service ends or the next batch is not the one it was made for,
//! and it names this process,
//! so the next service removes what a crash left (`remove_dead_staging`).
//! A commit that arrives while the flush is still running never waits for it:
//! it stages its own marker the ordinary way,
//! as a landing does when nothing was made ahead,
//! and the marker that is still being flushed finishes beside it and is removed when it is done.
//! The flush itself starts as early as the code allows,
//! while the landing that made the batch before it still holds the lock and moves the branch,
//! so it is usually finished before the next commit arrives.
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
///  The file the service has made ahead,
/// named for the shutdown that ends the process from another thread:
/// the pathname and the identity of the file written there.
/// Why:
///   The slot that holds the made-ahead marker belongs to the landing thread,
/// and a service stopped by a signal ends the process from its shutdown task,
/// where no destructor of the landing thread runs.
/// The identity keeps the removal from taking a pathname that now names another file,
///  and the entry is cleared whenever the landing thread takes,
/// places or removes the file.
/// Lock kept:
///  the landing thread writes this record and the shutdown task takes it,
/// so two threads reach it,
/// and a pathname with an identity is not one word an atomic could hold.
/// Under it only the value is replaced or taken:
/// no Git process,
/// no file and no other lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let madeAhead: { path: string; dev: number; ino: number } | undefined;
/// ```
static MADE_AHEAD: std::sync::Mutex<Option<MadeAhead>> = std::sync::Mutex::new(None);

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
    *kept = Some(MadeAhead { path: made.staged.clone(), device: held.dev(), inode: held.ino() });
}

/// Forgets the file the service made ahead,
/// because the landing thread took,
/// placed or removed it.
fn forget_made_ahead() {
    if let Ok(mut kept) = MADE_AHEAD.lock() {
        *kept = None;
    }
}

/// Takes the record of the file made ahead,
/// so that the file calls of its removal run with nothing held.
fn take_made_ahead() -> Option<MadeAhead> {
    let Ok(mut kept) = MADE_AHEAD.lock() else { return None };
    return kept.take();
}

/// What:
///  Removes the file the service made ahead,
/// when it is still the file the service wrote,
/// and forgets it either way.
/// Why:
///   The service ends by the signal the manager sent,
/// and `SPEC.md` "Removing the lock and what is made durable" says the marker "is removed when the service ends,
///  whichever way its loop ends";
/// the shutdown task runs no landing-thread destructor,
///  so it removes the file itself.
/// A pathname whose file was placed as the lock,
///  or replaced,
/// is not the service's to remove here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const made = madeAhead; madeAhead = undefined; if (made && lstat(made.path).ino === made.ino && lstat(made.path).dev === made.dev) rm(made.path);
/// ```
pub fn remove_the_file_made_ahead() {
    let Some(made) = take_made_ahead() else { return };
    let Ok(on_path) = std::fs::symlink_metadata(made.path.as_path()) else { return };
    if on_path.file_type().is_file() && on_path.dev() == made.device && on_path.ino() == made.inode {
        let _removed = std::fs::remove_file(made.path.as_path());
    }
}

/// Whether a commit has begun to wait for the marker being made,
/// so a control can hold the flush until a commit is waiting for it instead of racing a timer against the flush.
#[cfg(test)]
pub static COMMIT_WAITING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Takes the pre-staged marker out of the state it is kept in.
fn take_the_slot(state: &LandingState, (): ()) -> Option<Slot> {
    return state.prestaged_marker.take();
}

/// Keeps `slot` as the landing's pre-staged marker.
fn keep_the_slot(state: &LandingState, slot: Option<Slot>) {
    *state.prestaged_marker.borrow_mut() = slot;
}

/// Takes the service's one pre-staged marker out of the landing's state:
/// the state `prestaged_marker` of `landing_state_carried_by_its_task.rs` is per landing,
/// because only the landing takes locks and lands batches.
fn take_slot() -> Option<Slot> {
    return on_the_landing_state((), take_the_slot);
}

/// Puts the pre-staged marker back into the landing's state.
fn put_slot(slot: Option<Slot>) {
    on_the_landing_state(slot, keep_the_slot);
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
    // The slot is looked at without being dropped, because this runs on a landing's own path: a marker still being flushed
    //  is left to finish there rather than waited for, and a finished one for another batch or index is removed, which
    // waits for nothing.
    match take_slot() {
        // Made or being made for this batch and this index: one file, one flush.
        Some(slot) if slot.lock == lock && slot.seq == seq => {
            put_slot(Some(slot));
            return;
        }
        Some(slot) if !slot.work.is_finished() => {
            put_slot(Some(slot));
            return;
        }
        Some(slot) => remove_unused(slot),
        None => {}
    }
    let work_lock: PathBuf = lock.clone();
    // The flush can wait on the disk, so the marker is made by a job of the blocking pool, which a task of the runtime awaits.
    let spawned = spawn_on_runtime(run_blocking_job(move || -> std::io::Result<Prestaged> {
        // The three points of the marker's own life, where a control kills the service: before its file exists, with the
        //  file recorded and unflushed, and with it flushed and ready.
        crate::service_fault::reach(crate::service_fault::PRESTAGE_START);
        let marker: String = marker_text(seq, &own_identity).map_err(identity_error)?;
        // The file is recorded before the flush, because the flush can take seconds on a loaded disk and a stop that
        // arrives while it runs must find the file: a record made after the flush would leave it behind.
        let (staged, held) = stage_marker(work_lock.as_path(), seq, marker.as_str())?;
        let made: Prestaged = Prestaged { staged, held, marker };
        publish(&made);
        crate::service_fault::reach(crate::service_fault::PRESTAGE_STAGED);
        if let Err(error) = sync(&made.held) {
            forget_made_ahead();
            drop(made.held);
            let _removed = std::fs::remove_file(made.staged.as_path());
            return Err(error);
        }
        crate::service_fault::reach(crate::service_fault::PRESTAGE_FLUSHED);
        return Ok(made);
    }));
    // No worker to make it on is no marker made ahead: the commit stages its own.
    if let Some(work) = spawned {
        put_slot(Some(Slot { lock, seq, work }));
    }
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
    forget_made_ahead();
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
///   A marker made for another batch or index is removed and the commit stages its own.
/// A marker still being flushed is no marker for this landing: it is left to finish and the commit stages its own.
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
    let slot: Slot = take_slot()?;
    #[cfg(test)]
    COMMIT_WAITING.store(true, std::sync::atomic::Ordering::SeqCst);
    // The flush has not returned: the landing stages its own marker the ordinary way and waits for nothing, because this
    //  is on the commit's path. What is still being flushed stays in the slot, so it is removed once it is finished and no
    // file is left behind.
    if !slot.work.is_finished() {
        put_slot(Some(slot));
        return None;
    }
    if slot.lock != lock_path_of(index) || slot.seq != seq {
        remove_unused(slot);
        return None;
    }
    let made: Prestaged = made_marker(slot.work)?;
    forget_made_ahead();
    if !still_there(&made) {
        return None;
    }
    return Some(made);
}

/// Whether the marker made ahead is still being flushed,
/// which no landing waits for.
#[cfg(test)]
pub fn is_flushing() -> bool {
    return crate::landing_state_carried_by_its_task::with_the_landing_state(|state| { return state.prestaged_marker.borrow().as_ref().is_some_and(|held| { return !held.work.is_finished(); }); });
}

/// Whether the marker made ahead is finished,
/// so that taking it waits for nothing.
#[cfg(test)]
pub fn is_made() -> bool {
    return crate::landing_state_carried_by_its_task::with_the_landing_state(|state| { return state.prestaged_marker.borrow().as_ref().is_some_and(|held| { return held.work.is_finished(); }); });
}

/// Removes the marker made ahead,
/// waiting for it first if it is still being made.
/// Why:
///  This is the service's end and the idle time between landings, where nothing of a commit waits for the disk;
/// a landing stands aside from a flush still running rather than calling this.

pub fn discard() {
    if let Some(slot) = take_slot() {
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
