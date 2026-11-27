//! What:
//!  Where the landing's own syncs wait:
//! not on the path the person waits on,
//! unless the landing publishes nothing.
//! Why:
//!  The owner's rule for the commit protocol,
//! 2026-11-03:
//! "Force at leaving,
//! else background":
//! no forced write while the person
//!  waits,
//! so a power cut loses that attempt exactly as it does with native Git (staged changes and files are intact,
//! the
//!  commit is run again),
//! and whatever the command already promised when it left (an error,
//! a withdrawal,
//! a hand-over) was
//!  forced to disk first.
//! A landing that publishes a commit therefore only queues the syncs of its records (the intent,
//! the
//!  lock marker,
//! the completion record,
//! the done mark and the directory syncs);
//! one background task makes them in batches (each batch one job of the runtime's blocking pool),
//!  each sync of a batch overlapped.
//! A landing that publishes nothing syncs what it wrote before it returns.
//! A queue no background task takes is nobody's wait either:
//! the records reach the disk with the operating system's own write-back,
//!  which a machine crash loses as it loses any write the page cache held,
//! and a forced write on the commit's thread would be the whole
//!  device's wait on the person's path.
//! Git's own object,
//! reference
//!  and index writes follow `core.fsync` as they do for native Git:
//! the in-process index writer flushes only when
//!  `core.fsync` names the index (`ConfigSnapshot::hardens_index`).
//!  Nothing here changes what a killed process leaves:
//! the page cache keeps every write whether or not it was synced.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const landing = beginDeferral({ hardensIndex });
//! // ... flushFile(file, 'intent-file') queues; flushDir(dir, 'intent-dir') queues ...
//! landing.settle(published); // true: hand the queue to the background task; false: sync it now
//! ```

use crate::landing_state_carried_by_its_task::{
    LandingState,
    on_the_landing_state,
};
use crate::service_durable_syncs::SyncFigure;
use std::fs::File;
use std::time::{
    Duration,
    Instant,
};

pub(crate) use crate::service_durable_the_background_sync_thread::Pending;
/// The handle a landing queues,
/// the background task that makes those syncs after the landing answered,
/// and the wait a control makes for the queue,
/// are declared beside this file
/// (`service_durable_the_background_sync_thread.rs`),
/// because the deferral that hands it the batches is at its line bound.
pub use crate::service_durable_the_background_sync_thread::{
    BACKGROUND_SYNCED,
    drain,
};
use crate::service_durable_the_background_sync_thread::{
    hand_to_the_background,
    sync_all_at_once,
};

/// What a landing holds while it runs:
/// its syncs are queued,
/// and it settles them when it ends.
pub struct Deferral {
    /// Whether `settle` has run;
    /// a deferral dropped without it (a panic) syncs what it queued.
    settled: bool,
}

/// Starts queueing the syncs of the landing on this thread.
///  `hardens_index` says whether the index file the landing installs is flushed before the rename (`core.fsync` names it).
pub fn begin(hardens_index: bool) -> Deferral {
    on_the_landing_state(hardens_index, start_deferring);
    return Deferral {
        settled: false,
    };
}

/// Begins queueing the syncs of the landing in `state`,
/// with the index hardened or not.
fn start_deferring(
    state: &LandingState,
    hardens_index: bool,
) {
    state.deferring_syncs.set(true);
    state.index_hardened.set(hardens_index);
    state.queued_syncs.borrow_mut().clear();
    state.syncs.borrow_mut().clear();
}

/// What:
///  Records one sync of the landing this code runs in,
/// as its answer states it.
/// Why:
///   A landing that made more syncs and one that made the same syncs on a stalling device look alike
///  in the phases;
/// the record says how many there were and how long each took.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// landing.syncs.push({ what, waited, took })
/// ```
fn note_a_sync(figure: SyncFigure) {
    on_the_landing_state(figure, record_a_sync);
}

/// Pushes one sync into the landing's state.
fn record_a_sync(
    state: &LandingState,
    figure: SyncFigure,
) {
    state.syncs.borrow_mut().push(figure);
}

/// What:
///  Takes the syncs the landing this code runs in has recorded so far,
/// in the order it asked for them.
/// Why:
///   The landing's answer states them,
/// and nothing else reads them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const syncs = landing.syncs.splice(0)
/// ```
pub fn take_syncs() -> Vec<SyncFigure> {
    return on_the_landing_state((), drain_syncs);
}

/// Takes the recorded syncs out of the landing's state.
fn drain_syncs(
    state: &LandingState,
    (): (),
) -> Vec<SyncFigure> {
    return state.syncs.take();
}

/// Ends the deferral on this thread and returns what it queued.
fn end() -> Vec<Pending> {
    return on_the_landing_state((), stop_deferring);
}

/// Ends the deferral in `state` and takes what it queued.
fn stop_deferring(
    state: &LandingState,
    (): (),
) -> Vec<Pending> {
    state.deferring_syncs.set(false);
    state.index_hardened.set(true);
    return state.queued_syncs.take();
}

/// Syncs every handle now,
/// all at once,
/// and names each as waited for.
fn force(batch: Vec<Pending>) {
    for pending in &batch {
        crate::service_fault::note(format!("sync-waited:{}", pending.what).as_str());
    }
    let figures: Vec<(&'static str, Duration)> = sync_all_at_once(batch.as_slice(), false);
    for (what, took) in figures {
        note_a_sync(SyncFigure {
            what,
            waited: true,
            took,
        });
    }
}

impl Deferral {
    /// Ends the deferral.
    /// A landing that published a commit hands its queue to the background task and returns at once;
    /// a
    ///  landing that published nothing (a withdrawal,
    /// an error,
    /// a commit sent back) syncs its records before it returns.
    pub fn settle(
        self,
        published: bool,
    ) {
        return self.settle_with(published, &hand_to_the_background);
    }

    /// What:
    ///  `settle` with the handing of the queue to the background task given,
    /// so a control proves what a published landing does when no task takes it.
    /// Why:
    ///   The sender is one for the process,
    /// so only the hand can be made to refuse in a control.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// deferral.settleWith(published, hand)
    /// ```
    fn settle_with(
        mut self,
        published: bool,
        hand: &dyn Fn(Vec<Pending>) -> Option<Vec<Pending>>,
    ) {
        self.settled = true;
        let batch: Vec<Pending> = end();
        if batch.is_empty() {
            return;
        }
        if published {
            if hand(batch).is_some() {
                // No task takes the queue, and the decision for a landing that published is "Force at leaving, else background":
                //  the records reach the disk with the operating system's own write-back rather than on the commit's thread.
                crate::service_warn!(
                    "the runtime has no task for the syncs of a landing that published, so none waits for them here and the records reach the disk with the operating system's own write-back"
                );
            }
        } else {
            force(batch);
        }
    }
}

impl Drop for Deferral {
    fn drop(&mut self) {
        if !self.settled {
            force(end());
        }
    }
}

/// Whether the landing in `state` queues its syncs.
fn is_deferring(
    state: &LandingState,
    (): (),
) -> bool {
    return state.deferring_syncs.get();
}

/// Queues one sync in the landing's state.
fn queue_a_sync(
    state: &LandingState,
    pending: Pending,
) {
    state.queued_syncs.borrow_mut().push(pending);
}

/// Whether the index the landing in `state` writes is flushed before it is installed.
fn index_hardened_in(
    state: &LandingState,
    (): (),
) -> bool {
    return state.index_hardened.get();
}

/// Queues a flush in the background task and never waits for it.
///
/// # Errors
///
/// The error of duplicating the handle;
/// a queue that the background task no longer takes is left to the operating system's write-back.
pub fn defer_file(
    file: &File,
    what: &'static str,
) -> std::io::Result<()> {
    return defer_file_with(&hand_to_the_background, file, what);
}

/// What:
///  `defer_file` with the handing of the batch to the background task given,
/// so a control proves what happens when no task takes it.
/// Why:
///   Which task holds the queue is one process-wide sender,
/// so a control cannot make it refuse by way of the runtime;
/// the hand is the one step that decides.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// deferFileWith(hand, file, what)
/// ```
fn defer_file_with(
    hand: &dyn Fn(Vec<Pending>) -> Option<Vec<Pending>>,
    file: &File,
    what: &'static str,
) -> std::io::Result<()> {
    let handle: File = file.try_clone()?;
    note_a_sync(SyncFigure {
        what,
        waited: false,
        took: Duration::ZERO,
    });
    crate::service_fault::note(format!("sync-queued:{what}").as_str());
    if hand(vec![Pending {
        handle,
        what,
    }])
    .is_some()
    {
        // No task takes the flush, so it is the operating system's: the marker names the holder and nothing else, which a power
        //  cut loses as it loses any write the page cache held, and the step before a landing removes such a marker at once.
        crate::service_warn!(
            "the runtime has no task for the flush of the {what}, so nothing waits for it here and the write reaches the disk with the operating system's own write-back"
        );
    }
    return Ok(());
}

/// Flushes a file that is a landing's record:
/// queued while a landing defers,
/// made now otherwise.
///
/// # Errors
///
/// The error of the flush when it is made now.
pub fn flush_file(
    file: &File,
    what: &'static str,
) -> std::io::Result<()> {
    if on_the_landing_state((), is_deferring) {
        crate::service_fault::note(format!("sync-queued:{what}").as_str());
        note_a_sync(SyncFigure {
            what,
            waited: false,
            took: Duration::ZERO,
        });
        let handle: File = file.try_clone()?;
        on_the_landing_state(
            Pending {
                handle,
                what,
            },
            queue_a_sync,
        );
        return Ok(());
    }
    let began: Instant = Instant::now();
    let synced: std::io::Result<()> = file.sync_all();
    note_a_sync(SyncFigure {
        what,
        waited: true,
        took: began.elapsed(),
    });
    crate::service_fault::note(format!("sync-waited:{what}").as_str());
    return synced;
}

/// Flushes a directory whose entries are a landing's records,
/// as `flush_file` does for a file.
///
/// # Errors
///
/// The error of opening the directory,
/// or of the flush when it is made now.
pub fn flush_directory(
    directory: &std::path::Path,
    what: &'static str,
) -> std::io::Result<()> {
    return flush_file(&File::open(directory)?, what);
}

/// Flushes the index file the landing writes,
/// when the index is hardened:
/// `core.fsync` names it,
/// as native Git's index write
///  is hardened then.
/// Otherwise the index is written as native Git writes it,
/// unflushed.
/// It is made now,
/// on the landing's path,
///  as Git's own write is.
///
/// # Errors
///
/// The error of the flush.
pub fn flush_index(
    file: &File,
    what: &'static str,
) -> std::io::Result<()> {
    if index_is_hardened() {
        let began: Instant = Instant::now();
        let synced: std::io::Result<()> = file.sync_all();
        note_a_sync(SyncFigure {
            what,
            waited: true,
            took: began.elapsed(),
        });
        crate::service_fault::note(format!("sync-waited:{what}").as_str());
        return synced;
    }
    return Ok(());
}

/// Whether the index file the landing writes is flushed before it is installed.
pub fn index_is_hardened() -> bool {
    return on_the_landing_state((), index_hardened_in);
}

/// Controls for the queue stay out of the release executable.
#[cfg(test)]
#[path = "service_durable_tests.rs"]
mod tests;
