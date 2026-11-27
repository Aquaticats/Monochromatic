//! What:
//!  The markers of the next batches' index locks,
//! created and written while the service lands the batch before them,
//!  and their flush handed to the service's background task,
//! so that a commit's lock path only places one.
//! Why:
//!  The flush of the marker kept nothing a commit needs on the drive:
//! the marker names the process that holds the lock and
//!  nothing else,
//! and a power cut that leaves a partial one leaves a lock the wrapper judges stale and removes
//!  (`stale_ref_lock_judge.rs`).
//! That flush is a few milliseconds on an idle disk and seconds on a loaded one,
//! and it sat on every commit's path,
//! 8 to 12.5 s of it under load
//!  (`doc/planning/git-policy-cli-btrfs-transaction-waits.md`).
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
//! A commit that arrives while a marker is still being made never waits for it:
//! it stages its own marker the ordinary way,
//! as a landing does when nothing was made ahead,
//! and a marker that will not be used is let go rather than waited for:
//! one whose file exists has it removed at once,
//! and one whose file does not exist yet makes none.
//! The making is short and starts as early as the code allows,
//! while the landing that started it still holds the lock and moves the branch,
//! because the file is created and written there and its flush is the background task's,
//! so a commit usually finds one of the two finished and places it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! prestage.start(index, seq + 1); /* idle */ const ready = await prestage.take(index, seq); // undefined: stage cold
//! ```

use crate::blocking_job::run_blocking_job;
use crate::landing_state_carried_by_its_task::{
    LandingState,
    on_the_landing_state,
};
use crate::lazy_runtime::spawn_on_runtime;
use crate::process_identity::ProcessIdentityError;
use crate::service_index_identity::own_identity;
use crate::service_index_marker::marker_text;
use crate::service_index_placement::sync_file;
use crate::service_index_staging::stage_marker;
use crate::the_names_of_the_index_lock::lock_path_of;
use std::path::{
    Path,
    PathBuf,
};
use std::sync::atomic::{
    AtomicBool,
    Ordering,
};

#[cfg(test)]
pub use crate::service_index_prestage_the_markers_made_ahead::COMMIT_WAITING;
/// The marker this service keeps made ahead is declared beside this file
/// (`service_index_prestage_the_markers_made_ahead.rs`),
/// because the making and the taking are at their line bound.
pub use crate::service_index_prestage_the_markers_made_ahead::Prestaged;
pub(crate) use crate::service_index_prestage_the_markers_made_ahead::Slot;
use crate::service_index_prestage_the_markers_made_ahead::{
    MADE_AHEAD,
    MadeAhead,
    SERVICE_ENDS,
    in_the_step_of_creating_and_recording,
    made_marker,
    no_longer_wanted,
    record_the_file,
    remove_the_recorded_file,
    still_there,
};

/// The pre-staged markers are kept in the state a landing carries,
/// and not here:
/// only the landing takes locks and lands batches,
/// so there is one list of them per landing.
fn take_the_slots(
    state: &LandingState,
    (): (),
) -> Vec<Slot> {
    return state.prestaged_markers.take();
}

/// Keeps `slots` as the landing's pre-staged markers.
fn keep_the_slots(
    state: &LandingState,
    slots: Vec<Slot>,
) {
    *state.prestaged_markers.borrow_mut() = slots;
}

/// How many markers the service keeps made ahead:
/// the one the next batch takes and the one after that,
/// so every flush has a whole landing to finish in.
/// It is also the bound on the files the service leaves in the Git directory,
/// so a control counts them by it rather than repeat the number,
/// and it is public for those controls.
pub const MARKERS_MADE_AHEAD: usize = 2;

/// What:
///  Creates the marker's file and records it for the shutdown in one step,
/// or makes nothing when the marker was let go or the service has stopped keeping them.
/// Why:
///   A removal from another thread can only take a file it has been told about,
/// so the read of whether the file is still wanted,
///  the creation and the record are one step that the sweep of the service's end excludes itself from.
/// The file is recorded before its flush,
/// because the flush can take seconds on a loaded disk;
///  a file that cannot be recorded is removed here,
/// so nothing no sweep could see stays in the Git directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const step = creatingAndRecording.lock(); if (!wanted || serviceEnds) return undefined; const made = stageMarker(); record(made); return made
/// ```
fn make_and_record(
    lock: &Path,
    seq: u64,
    wanted: &AtomicBool,
    marker: &str,
) -> std::io::Result<Option<Prestaged>> {
    let _step = in_the_step_of_creating_and_recording();
    if !wanted.load(Ordering::SeqCst) || SERVICE_ENDS.load(Ordering::SeqCst) {
        return Ok(None);
    }
    let (staged, held): (PathBuf, std::fs::File) = stage_marker(lock, seq, marker)?;
    let made: Prestaged = Prestaged {
        staged,
        held,
        marker: String::from(marker),
    };
    if let Err(error) = record_the_file(lock, seq, &made) {
        remove_if_ours(made);
        return Err(error);
    }
    return Ok(Some(made));
}

/// Takes the record of the marker made for batch `seq` of the lock at `lock`,
/// so the removal of its file runs with nothing held.
fn take_the_record_of(
    lock: &Path,
    seq: u64,
) -> Option<MadeAhead> {
    let Ok(mut kept) = MADE_AHEAD.lock() else {
        return None;
    };
    let mut wanted: Option<MadeAhead> = None;
    let mut remaining: Vec<MadeAhead> = Vec::new();
    for made in std::mem::take(&mut *kept) {
        if wanted.is_none() && made.lock == lock && made.seq == seq {
            wanted = Some(made);
        } else {
            remaining.push(made);
        }
    }
    *kept = remaining;
    return wanted;
}

/// Forgets the file the service made ahead at `path`,
/// because the landing thread took,
/// placed or removed it.
fn forget_made_ahead(path: &Path) {
    let Ok(mut kept) = MADE_AHEAD.lock() else {
        return;
    };
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
    let Ok(mut kept) = MADE_AHEAD.lock() else {
        return Vec::new();
    };
    return std::mem::take(&mut kept);
}

/// What:
///  Removes the files the service made ahead,
/// when each is still the file the service wrote,
/// and stops the service from making any more of them.
/// Why:
///   The service ends by the signal the manager sent,
/// and `SPEC.md` "Removing the lock and what is made durable" says the markers "are removed when the service ends,
///  whichever way its loop ends";
/// the shutdown task runs no landing-thread destructor,
///  so it removes the files itself.
/// A pathname whose file was placed as the lock,
///  or replaced,
/// is not the service's to remove here.
/// The end of the keeping is set inside the step a making excludes itself from,
/// so a marker is either recorded before this sweep and removed by it,
/// or never made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const step = creatingAndRecording.lock(); serviceEnds = true; madeAhead.splice(0).forEach(rmIfOurs)
/// ```
pub fn remove_the_file_made_ahead() {
    let recorded: Vec<MadeAhead> = {
        let _step = in_the_step_of_creating_and_recording();
        SERVICE_ENDS.store(true, Ordering::SeqCst);
        take_made_aheads()
    };
    for made in recorded {
        remove_the_recorded_file(made);
    }
}

/// Whether markers made ahead are made again,
/// for a control that proved the service's last sweep stops them:
/// the flag is the process's,
/// as the list of the files is.
#[cfg(test)]
pub fn make_markers_again() {
    SERVICE_ENDS.store(false, Ordering::SeqCst);
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
///   A marker for another index is let go of and removed first,
/// without waiting for its making.
/// A marker of another batch of this index is kept:
/// it is the spare a following landing takes,
/// so the files waiting are the batches after this one,
/// and at most `MARKERS_MADE_AHEAD` are made,
/// the oldest going before another is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// prestage.start(index, seq)
/// ```
pub fn start(
    index: &Path,
    seq: u64,
) {
    start_with(sync_file, index, seq);
}

/// `start` with the flush given,
/// which a control makes slow.
pub fn start_with(
    sync: fn(&std::fs::File) -> std::io::Result<()>,
    index: &Path,
    seq: u64,
) {
    let lock: PathBuf = lock_path_of(index);
    // The markers that can still be used here: every one for this index, and the one this call asks for among them.
    // A marker of this batch is kept exactly as it is, being made or finished, so the landing that asks again for it
    //  starts nothing; nothing here is dropped, because a marker still being made is nobody's to wait for.
    let mut kept: Vec<Slot> = Vec::new();
    let mut same: bool = false;
    for slot in take_slots() {
        if slot.lock == lock {
            same = same || slot.seq == seq;
            kept.push(slot);
        } else {
            // A marker for another index is never used here: it is let go, and its file goes with it.
            let_go_of_unused(slot);
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
        let_go_of_unused(kept.remove(oldest));
    }
    let work_lock: PathBuf = lock.clone();
    let wanted: std::sync::Arc<AtomicBool> = std::sync::Arc::new(AtomicBool::new(true));
    let wanted_by_the_job: std::sync::Arc<AtomicBool> = std::sync::Arc::clone(&wanted);
    // The flush is handed to the service's background task, so a commit that takes this marker never waits for a flush;
    //  see the module doc.
    let spawned = spawn_on_runtime(run_blocking_job(move || -> std::io::Result<Prestaged> {
        // The three points of the marker's own life, where a control kills the service: before its file exists, with the
        //  file recorded and its flush not yet asked for, and with the file complete and its flush asked for.
        crate::service_fault::reach(crate::service_fault::PRESTAGE_START);
        let marker: String = marker_text(seq, &own_identity).map_err(identity_error)?;
        // The file is made and recorded in one step, before the flush, because a stop or a batch that will not use this
        //  marker must find the file to remove it, and the landing thread must wait for neither.
        let Some(made) = make_and_record(
            work_lock.as_path(),
            seq,
            wanted_by_the_job.as_ref(),
            marker.as_str(),
        )?
        else {
            // The marker was let go, or the service stopped keeping them: nothing is made and nothing waits for it.
            return Err(no_longer_wanted());
        };
        crate::service_fault::reach(crate::service_fault::PRESTAGE_STAGED);
        if let Err(error) = sync(&made.held) {
            // A flush that failed leaves no file of the service's own: its own pathname goes with it.
            remove_if_ours(made);
            return Err(error);
        }
        crate::service_fault::reach(crate::service_fault::PRESTAGE_FLUSHED);
        return Ok(made);
    }));
    // No worker to make it on is no marker made ahead: the commit stages its own.
    if let Some(work) = spawned {
        kept.push(Slot {
            lock,
            seq,
            wanted,
            work,
        });
    }
    put_slots(kept);
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

/// What:
///  Lets go of a marker that will not be used:
/// its file goes now when it exists,
/// it is never made when it does not,
/// and no thread waits for its making.
/// Why:
///   This runs on the commit's path,
///  in `start_with` and in `take`,
/// where a marker still flushing or still queued behind another job of the pool may cost nothing:
/// the owner's rules are that nothing waits indefinitely and that a commit never waits on another's work,
/// and the commit holds an index lock while it waits.
/// The record of the file is found here by the lock and the batch it was made for,
/// so the removal is exact and needs no join;
/// a making that has not reached its file yet makes none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const step = creatingAndRecording.lock(); slot.wanted = false; takeRecord(slot.lock, slot.seq)?.remove()
/// ```
fn let_go_of_unused(slot: Slot) {
    let recorded: Option<MadeAhead> = {
        let _step = in_the_step_of_creating_and_recording();
        slot.wanted.store(false, Ordering::SeqCst);
        take_the_record_of(slot.lock.as_path(), slot.seq)
    };
    if let Some(made) = recorded {
        remove_the_recorded_file(made);
    }
    // The making ends beside this on the pool's own thread, and its result is nobody's: aborting the task gives the
    //  runtime the piece of work back, while the block it handed to the pool ends by itself (`blocking_job.rs`).
    slot.work.abort();
}

/// What:
///  The marker made for batch `seq` of the index `index`,
/// or nothing when none was made for exactly that.
/// Why:
///   A marker for another batch or index is let go of,
/// and the commit stages its own.
/// A marker of a later batch is kept:
/// it is the spare the next landing takes.
/// A marker still being made is no marker for this landing:
/// it is left to finish where it is,
/// and the commit stages its own,
/// because nothing on the commit's path waits for a disk.
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
pub fn take(
    index: &Path,
    seq: u64,
) -> Option<Prestaged> {
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
        } else if slot.work.is_finished() {
            // A marker for another index, or for a batch this one has passed, is never used: it is let go, and its file
            //  goes with it.
            let_go_of_unused(slot);
        } else {
            // Its making has not returned, and nothing on the commit's path waits for a disk: it stays where it is until
            //  it is finished, and the batch that follows or the service's end lets it go then.
            kept.push(slot);
        }
    }
    let Some(asked_for) = mine else {
        put_slots(kept);
        return None;
    };
    if !asked_for.work.is_finished() {
        // The making has not returned: the landing stages its own marker the ordinary way and waits for nothing, because
        //  this is on the commit's path. The marker stays in the slots, so it is removed once it is finished and no file is
        //  left behind.
        kept.push(asked_for);
        put_slots(kept);
        return None;
    }
    put_slots(kept);
    let made: Prestaged = made_marker(asked_for.work)?;
    forget_made_ahead(made.staged.as_path());
    if !still_there(&made) {
        return None;
    }
    return Some(made);
}

/// How many markers made ahead are finished,
/// so a control waits for the idle time's making rather than for a clock:
/// a file appears in the directory before its text is written.
#[cfg(test)]
pub fn finished_markers() -> usize {
    return crate::landing_state_carried_by_its_task::with_the_landing_state(|state| {
        return state
            .prestaged_markers
            .borrow()
            .iter()
            .filter(|held| {
                return held.work.is_finished();
            })
            .count();
    });
}

/// Whether any marker made ahead is finished,
/// so that taking it waits for nothing.
#[cfg(test)]
pub fn is_made() -> bool {
    return finished_markers() > 0;
}

/// What:
///  Removes every marker made ahead,
/// without waiting for a making that is still running.
/// Why:
///  This is the service's end,
/// where its own files may not be left behind:
/// a marker whose file exists goes now,
/// and one whose file was not made yet makes none,
/// so nothing here waits for a flush on a pool another command may be occupying.
pub fn discard() {
    for slot in take_slots() {
        let_go_of_unused(slot);
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
