//! Feeding a whole library into the Search service in batches, off the caller's thread.
//!
//! What: `feed_library` splits a list of display paths into batches and gives them to the service on a background
//! thread. An empty index is filled in place, so results appear while the list is still being consumed; an index that
//! already holds a library is replaced by one built on the side, so the open page never loses its results.
//! Why: the engine reports a rescanned library as one list, and normalizing a million names takes seconds. D111 asks
//! that Search work meanwhile: the first case makes tracks searchable from the first batch, the second keeps answering
//! from the old library until the new one is complete.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! feedLibrary(service, names, 10_000, 0); // returns at once; batches arrive on a worker
//! ```

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::search_index::SearchIndex;
use crate::search_index_batch::{PreparedBatch, prepare_batch};
use crate::search_service::SearchService;

/// What:
///      `const MAX_PREPARERS: usize = 4`.
///  The most threads that prepare batches at once.
/// Why:
///       Preparation is the only work that grows with the text, and it parallelizes by batch; four threads cut a
///  million-name feed to about a quarter of the time while leaving cores for the window and for playback.
const MAX_PREPARERS: usize = 4;

/// What:
///      `fn prepare_in_order(names: &[String], batch: usize, sink: impl FnMut(usize, PreparedBatch) -> bool)`.
///  Prepares batches on several threads and hands them to `sink` in library order.
/// Why:
///       The index must receive batches in order, because a track's position is its place in the list, but preparing
///  them can happen in any order. `sink` returns false to stop early, which also stops the preparers.
pub(crate) fn prepare_in_order(names: &[String], batch: usize, mut sink: impl FnMut(usize, PreparedBatch) -> bool) {
    let chunks: Vec<&[String]> = names.chunks(batch.max(1)).collect();
    let workers: usize = thread::available_parallelism().map_or(1, |count| return count.get()).min(MAX_PREPARERS);
    if workers <= 1 || chunks.len() <= 1 {
        for (number, chunk) in chunks.iter().enumerate() {
            if !sink(number, prepare_batch(chunk)) {
                return;
            }
        }
        return;
    }
    let next_chunk: AtomicUsize = AtomicUsize::new(0);
    let halted: AtomicBool = AtomicBool::new(false);
    thread::scope(|scope| {
        let (outbox, receiver) = mpsc::sync_channel::<(usize, PreparedBatch)>(workers * 2);
        for _ in 0..workers {
            let worker_outbox = outbox.clone();
            let (claim, halt, work_list) = (&next_chunk, &halted, &chunks);
            scope.spawn(move || {
                loop {
                    let number: usize = claim.fetch_add(1, Ordering::AcqRel);
                    if number >= work_list.len() || halt.load(Ordering::Acquire) {
                        return;
                    }
                    if worker_outbox.send((number, prepare_batch(work_list[number]))).is_err() {
                        return;
                    }
                }
            });
        }
        drop(outbox);
        let mut waiting: BTreeMap<usize, PreparedBatch> = BTreeMap::new();
        let mut expected: usize = 0;
        for (number, prepared) in receiver.iter() {
            waiting.insert(number, prepared);
            while let Some(ready) = waiting.remove(&expected) {
                if !sink(expected, ready) {
                    halted.store(true, Ordering::Release);
                    return;
                }
                expected += 1;
            }
        }
    });
}

/// What:
///      `pub const FEED_BATCH: usize = 10_000`.
///  How many tracks one batch holds.
/// Why:
///       A batch of ten thousand prepares in about thirty milliseconds and is pushed in about a millisecond, so no
///  lock hold or preparation step is long enough to notice.
pub const FEED_BATCH: usize = 10_000;

/// What:
///      `pub fn feed_library(service: &Arc<SearchService>, names: Arc<Vec<String>>, batch: usize, pause: Duration)`.
///  Starts a feed and returns at once; a newer feed cancels this one.
/// Why:
///       See the module description. `pause` sleeps between batches; production passes zero and tests pass a delay to
///  imitate a slow scan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function feedLibrary(service: SearchService, names: string[], batch: number, pauseMs: number): void { /* thread */ }
/// ```
pub fn feed_library(service: &Arc<SearchService>, names: Arc<Vec<String>>, batch: usize, pause: Duration) {
    let epoch: u64 = service.begin_feed();
    let worker: Arc<SearchService> = Arc::clone(service);
    tracing::info!(tracks = names.len(), epoch, "library feed started");
    let _detached = thread::spawn(move || {
        if worker.adopt_streamed(&names) {
            tracing::info!(tracks = names.len(), epoch, "the streamed index already holds this library");
            return;
        }
        if worker.track_count() == 0 {
            feed_in_place(&worker, &names, epoch, batch, pause);
        } else {
            feed_on_the_side(&worker, &names, epoch, batch, pause);
        }
    });
}

/// What: `path_bytes` sums the lengths of the names.
/// Why: reserving the arenas up front keeps buffer growth, which copies the buffer, out of the write lock.
fn path_bytes(names: &[String]) -> usize {
    return names.iter().map(String::len).sum();
}

/// What: `feed_in_place` fills the live index batch by batch.
/// Why: the index is empty, so every track added is a track that was not searchable before.
fn feed_in_place(service: &SearchService, names: &[String], epoch: u64, batch: usize, pause: Duration) {
    let generation: u64 = service.reset(names.len(), path_bytes(names));
    let mut finished: bool = true;
    prepare_in_order(names, batch, |_number, prepared| {
        if !service.feed_is_current(epoch) {
            tracing::info!(epoch, "library feed cancelled by a newer one");
            finished = false;
            return false;
        }
        if !service.append_prepared(generation, prepared) {
            finished = false;
            return false;
        }
        if !pause.is_zero() {
            thread::sleep(pause);
        }
        return true;
    });
    if finished {
        service.finish(generation);
        tracing::info!(tracks = names.len(), epoch, "library feed finished in place");
    }
}

/// What: `feed_on_the_side` builds a second index and swaps it in when complete.
/// Why: the live index still answers for the previous library, and replacing it at the end avoids a flash of
/// empty results during a rescan.
fn feed_on_the_side(service: &SearchService, names: &[String], epoch: u64, batch: usize, pause: Duration) {
    let mut fresh: SearchIndex = SearchIndex::new(0);
    fresh.reserve(names.len(), path_bytes(names), path_bytes(names) / 2);
    let mut finished: bool = true;
    prepare_in_order(names, batch, |_number, prepared| {
        if !service.feed_is_current(epoch) {
            tracing::info!(epoch, "library feed cancelled by a newer one");
            finished = false;
            return false;
        }
        fresh.append(prepared);
        if !pause.is_zero() {
            thread::sleep(pause);
        }
        return true;
    });
    if finished && service.feed_is_current(epoch) {
        service.install(fresh);
        tracing::info!(tracks = names.len(), epoch, "library feed installed from the side");
    }
}
