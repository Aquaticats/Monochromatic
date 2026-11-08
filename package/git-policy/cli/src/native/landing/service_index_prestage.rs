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
use crate::service_index_staging::stage_marker_synced;
use crate::landing_state_carried_by_its_task::with_the_landing_state;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;

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
    work: JoinHandle<std::io::Result<Prestaged>>,
}

/// Whether a commit has begun to wait for the marker being made,
/// so a control can hold the flush until a commit is waiting for it instead of racing a timer against the flush.
#[cfg(test)]
pub static COMMIT_WAITING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Takes the service's one pre-staged marker out of the landing's state:
/// the state `prestaged_marker` of `landing_state_carried_by_its_task.rs` is per landing,
/// because only the landing takes locks and lands batches.
fn take_slot() -> Option<Slot> {
    return with_the_landing_state(|state| state.prestaged_marker.take());
}

/// Puts the pre-staged marker back into the landing's state.
fn put_slot(slot: Option<Slot>) {
    with_the_landing_state(|state| *state.prestaged_marker.borrow_mut() = slot);
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
    // Look at the slot without changing it: take it out, compare, and put it back.
    let looked_at: Option<Slot> = take_slot();
    let same: bool = matches!(&looked_at, Some(slot) if slot.lock == lock && slot.seq == seq);
    put_slot(looked_at);
    if same {
        return;
    }
    discard();
    let work_lock: PathBuf = lock.clone();
    let spawned = std::thread::Builder::new().name(String::from("cli-git-prestage")).spawn(move || -> std::io::Result<Prestaged> {
        let marker: String = marker_text(seq, &own_identity).map_err(identity_error)?;
        let (staged, held) = stage_marker_synced(&sync, work_lock.as_path(), seq, marker.as_str())?;
        return Ok(Prestaged { staged, held, marker });
    });
    // No thread to make it in is no marker made ahead: the commit stages its own.
    if let Ok(work) = spawned {
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
    if still_there(&made) {
        let _removed = std::fs::remove_file(&made.staged);
    }
}

/// Removes a marker that will not be used.
fn remove_unused(slot: Slot) {
    if let Ok(Ok(made)) = slot.work.join() {
        remove_if_ours(made);
    }
}

/// What:
///  The marker made for batch `seq` of the index `index`,
/// or nothing when none was made for exactly that.
/// Why:
///   A marker made for another batch or index is removed and the commit stages its own.
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
    let slot: Slot = take_slot()?;
    #[cfg(test)]
    COMMIT_WAITING.store(true, std::sync::atomic::Ordering::SeqCst);
    if slot.lock != lock_path_of(index) || slot.seq != seq {
        remove_unused(slot);
        return None;
    }
    let made: Prestaged = slot.work.join().ok()?.ok()?;
    if !still_there(&made) {
        return None;
    }
    return Some(made);
}

/// Whether the marker made ahead is finished,
/// so that taking it waits for nothing.
#[cfg(test)]
pub fn is_made() -> bool {
    return with_the_landing_state(|state| { return state.prestaged_marker.borrow().as_ref().is_some_and(|held| { return held.work.is_finished(); }); });
}

/// Removes the marker made ahead,
/// waiting for it first if it is still being made.
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
