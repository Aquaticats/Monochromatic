//! What:
//!  The answered map of a size read an add will not use,
//!  and the thread that frees it,
//!  with the control seams that show where the free happened.
//! Why:
//!   The read parses on its own thread,
//! where `main` parsed it,
//!  so an add that uses the answer never parses on its waiting path
//!  (the integration pass's Medium 1 of 2026-11-23);
//!  an add that declines has that map on its waiting path when the abandon takes it,
//!  where dropping 24,634 entries cost 1.8 to 3.2 ms and leaving them resident cost more
//!  (the integration recheck's Medium 1 of 2026-11-22).
//!  The free therefore runs on a thread of its own,
//! and this file is where that thread,
//!  its cost and its counters live,
//!  so the size-read module stays one subject per file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! freeBesideTheAdd(answered); // a thread of its own drops it; the adding thread only starts that thread
//! ```

use crate::scoped_start::spawn_named;
use crate::worktree_clone_conversion_git_reads::BlobSizes;

/// The name the thread that frees an abandoned answer carries in the process.
/// Why:
///   A diagnostic that lists threads names it for what it does.
///  It lives a few milliseconds and holds one map (3.1 MB on this crate's fixture).
pub(crate) const FREEING_THREAD: &str = "clone-size-read-freed-map";

/// What:
///  Free the answer of a read this add will not use
///  on a thread of its own,
///  so the adding thread pays neither the map's free nor its later allocations.
/// Why:
///   See the module's own doc.
/// The map travels into the closure,
/// so nothing is copied,
/// and the thread ends as soon as it has freed it.
/// A thread the system refuses loses only the free's place:
///  `spawn_named` consumes the closure,
/// so the map is freed here,
///  which is what the fence did before the free moved,
/// and the add pays those milliseconds.
/// The map stays resident for the few milliseconds the freeing thread takes,
/// not to the end of the add;
///  one listing is the whole of it
/// (3.1 MB for 24,634 paths,
/// about 128 bytes a path).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// freeBesideTheAdd(answered) // a thread of its own drops it; the adding thread only starts that thread
/// ```
pub(crate) fn free_the_answered_map_beside_the_add(answered: Option<BlobSizes>) {
    let Some(sizes) = answered else { return };
    #[cfg(test)]
    THE_ABANDONING_THREAD.with(|held| held.set(Some(std::thread::current().id())));
    let freeing = move || {
        drop(TheAnsweredMap(Some(sizes)));
    };
    let _started: std::io::Result<std::thread::JoinHandle<()>> =
        spawn_named(FREEING_THREAD, freeing);
    #[cfg(test)]
    THE_ABANDONING_THREAD.with(|held| held.set(None));
}

/// What:
///  The answered map on its way to a thread that frees it,
///  which counts,
/// in a test build,
/// the thread that paid the free.
/// Why:
///   A control has to show that the adding thread of a declined add frees no map,
///  and a plain `drop` of the map inside `abandon` would be a free on that thread;
/// one wrapper answers which thread dropped it without changing the shipped path,
///  because its `Drop` only drops the map.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class TheAnsweredMap { constructor(readonly sizes: BlobSizes) {} } // dropped wherever it ends up
/// ```
struct TheAnsweredMap(Option<BlobSizes>);

impl Drop for TheAnsweredMap {
    fn drop(&mut self) {
        #[cfg(test)]
        note_an_answered_map_was_freed();
        let _freed = self.0.take();
    }
}

#[cfg(test)]
thread_local! {
    /// The thread that abandoned a read,
    /// while it is in `abandon`,
    /// so a control can tell whether the answered map was freed there.
    static THE_ABANDONING_THREAD: std::cell::Cell<Option<std::thread::ThreadId>> = const { std::cell::Cell::new(None) };
}

#[cfg(test)]
static THE_ANSWER_WAS_SENT: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

#[cfg(test)]
static ANSWERED_MAPS_FREED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[cfg(test)]
static ANSWERED_MAPS_FREED_ON_THE_ADDING_THREAD: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

/// What:
///  Note that the read's thread has sent its answer,
/// for a control that abandons a read which already answered.
/// Why:
///   An abandon of a read that has not answered states it as started and not joined and frees no map,
/// so a control of the free has to look after the send.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// noteTheAnswerWasSent();
/// ```
#[cfg(test)]
pub(crate) fn note_the_answer_was_sent() {
    THE_ANSWER_WAS_SENT.store(true, std::sync::atomic::Ordering::SeqCst);
}

/// What:
///  Whether the read's thread has sent its answer,
/// for a control of the free.
/// Why:
///   See `note_the_answer_was_sent`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answered = () => THE_ANSWER_WAS_SENT;
/// ```
#[cfg(test)]
pub(crate) fn the_answer_was_sent_for_a_control() -> bool {
    return THE_ANSWER_WAS_SENT.load(std::sync::atomic::Ordering::SeqCst);
}

/// What:
///  Count a freed answered map,
///  and whether the adding thread paid it.
/// Why:
///   A control shows the free happens,
/// and that it does not happen on the thread that abandoned the read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function noteAnAnsweredMapWasFreed() { freed += 1; if (thread.current === abandoning) freedHere += 1; }
/// ```
#[cfg(test)]
fn note_an_answered_map_was_freed() {
    let _counted = ANSWERED_MAPS_FREED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let abandoning: Option<std::thread::ThreadId> =
        THE_ABANDONING_THREAD.with(std::cell::Cell::get);
    if abandoning == Some(std::thread::current().id()) {
        let _here = ANSWERED_MAPS_FREED_ON_THE_ADDING_THREAD
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

/// What:
///  How many answered maps were freed,
///  and how many of them on the thread that abandoned the read,
///  for a control that waits for the free and shows where it happened.
/// Why:
///   The free runs on a thread of its own,
/// so a control reads the counters after the free rather than at the abandon.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answeredMapsFreed = () => [freed, freedHere];
/// ```
#[cfg(test)]
pub(crate) fn answered_maps_freed_for_a_control() -> (usize, usize) {
    return (
        ANSWERED_MAPS_FREED.load(std::sync::atomic::Ordering::SeqCst),
        ANSWERED_MAPS_FREED_ON_THE_ADDING_THREAD.load(std::sync::atomic::Ordering::SeqCst),
    );
}
