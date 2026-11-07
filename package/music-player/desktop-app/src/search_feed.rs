//! Feeding a whole library into the Search service in batches,
//! off the caller's thread.
//!
//! What:
//! `feed_library` splits a list of display paths into batches and gives them to the service on a background
//! thread.
//! An empty index is filled in place,
//! so results appear while the list is still being consumed;
//! an index that
//! already holds a library is replaced by one built on the side,
//! so the open page never loses its results.
//! Why:
//! the engine reports a rescanned library as one list,
//! and normalizing a million names takes seconds.
//! D111 asks
//! that Search work meanwhile:
//! the first case makes tracks searchable from the first batch,
//! the second keeps answering
//! from the old library until the new one is complete.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! feedLibrary(service, names, 10_000, 0); // returns at once; batches arrive on a worker
//! ```

/// Imports the ordered map that keeps prepared batches in order until they can be handed on.
use std::collections::BTreeMap;
/// Imports the atomic flag,
/// the counter and their ordering that coordinate the preparer threads.
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
/// Imports the channel module that carries prepared batches back to the feeding thread.
use std::sync::mpsc;
/// Imports the shared ownership type that shares the names with the preparer threads.
use std::sync::Arc;
/// Imports the thread module that runs the preparers.
use std::thread;
/// Imports the duration type of the pause between batches,
/// which tests lengthen to imitate a slow scan.
use std::time::Duration;

/// Imports the index that receives the batches.
use crate::search_index::SearchIndex;
/// Imports the prepared batch and the function that prepares it.
use crate::search_index_batch::{PreparedBatch, prepare_batch};
/// Imports the service that owns the index and the stream thread.
use crate::search_service::SearchService;

/// What:
///      `const MAX_PREPARERS: usize = 4`.
///  The most threads that prepare batches at once.
/// Why:
///       Preparation is the only work that grows with the text,
/// and it parallelizes by batch;
/// four threads cut a
///  million-name feed to about a quarter of the time while leaving cores for the window and for playback.
const MAX_PREPARERS: usize = 4;

/// What:
/// `BatchSink` receives the prepared batches of a feed,
/// in library order.
/// Why:
/// the feed that fills the index in place and the feed that builds a fresh index take each batch differently,
/// and a trait with named implementations carries their state without closures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface BatchSink { accept(number: number, prepared: PreparedBatch): boolean }
/// ```
pub(crate) trait BatchSink {
    /// What:
    /// `accept` takes one prepared batch and says whether the feed goes on.
    /// Why:
    /// a cancelled feed answers `false`,
    /// which stops the preparation.
    fn accept(&mut self, number: usize, prepared: PreparedBatch) -> bool;
}

/// A callable of the same shape is a sink too,
/// so a test can pass one.
impl<F: FnMut(usize, PreparedBatch) -> bool> BatchSink for F {
    /// What:
    /// `accept` calls the callable.
    /// Why:
    /// it lets a plain callable stand in for a sink.
    fn accept(&mut self, number: usize, prepared: PreparedBatch) -> bool {
        return self(number, prepared);
    }
}

/// What:
/// `InPlaceFeed` is the sink that appends each batch to the index the service already holds.
/// Why:
/// the feed that refills the live index keeps the generation it started under,
/// and stops when a newer feed starts or the service refuses a batch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class InPlaceFeed implements BatchSink { finished = true; accept(number, prepared) { ... } }
/// ```
struct InPlaceFeed<'service> {
    /// The service whose index is appended to.
    service: &'service SearchService,
    /// The number of this feed,
    /// which a newer feed replaces.
    epoch: u64,
    /// The generation of the index when this feed started.
    generation: u64,
    /// The pause after each batch.
    pause: Duration,
    /// Whether every batch was appended.
    finished: bool,
}

/// The in-place feed takes batches into the live index.
impl BatchSink for InPlaceFeed<'_> {
    /// What:
    /// `accept` appends one batch unless the feed was cancelled or refused.
    /// Why:
    /// a newer feed must win,
    /// and a refused batch ends this one.
    fn accept(&mut self, _number: usize, prepared: PreparedBatch) -> bool {
        if !self.service.feed_is_current(self.epoch) {
            tracing::info!(epoch = self.epoch, "library feed cancelled by a newer one");
            self.finished = false;
            return false;
        }
        if !self.service.append_prepared(self.generation, prepared) {
            self.finished = false;
            return false;
        }
        if !self.pause.is_zero() {
            thread::sleep(self.pause);
        }
        return true;
    }
}

/// What:
/// `SideFeed` is the sink that appends each batch to a fresh index built beside the live one.
/// Why:
/// the live index keeps answering queries while the new one is built,
/// and replaces it only when the feed finished and is still current.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class SideFeed implements BatchSink { fresh: SearchIndex; finished = true; accept(number, prepared) { ... } }
/// ```
struct SideFeed<'service> {
    /// The service that installs the finished index.
    service: &'service SearchService,
    /// The number of this feed,
    /// which a newer feed replaces.
    epoch: u64,
    /// The index under construction.
    fresh: SearchIndex,
    /// The pause after each batch.
    pause: Duration,
    /// Whether every batch was appended.
    finished: bool,
}

/// The side feed takes batches into the fresh index.
impl BatchSink for SideFeed<'_> {
    /// What:
    /// `accept` appends one batch to the fresh index unless the feed was cancelled.
    /// Why:
    /// a newer feed must win.
    fn accept(&mut self, _number: usize, prepared: PreparedBatch) -> bool {
        if !self.service.feed_is_current(self.epoch) {
            tracing::info!(epoch = self.epoch, "library feed cancelled by a newer one");
            self.finished = false;
            return false;
        }
        self.fresh.append(prepared);
        if !self.pause.is_zero() {
            thread::sleep(self.pause);
        }
        return true;
    }
}

/// What:
///      `fn prepare_in_order(names: &[String], batch: usize, sink: impl FnMut(usize, PreparedBatch) -> bool)`.
///  Prepares batches on several threads and hands them to `sink` in library order.
/// Why:
///       The index must receive batches in order,
/// because a track's position is its place in the list,
/// but preparing
///  them can happen in any order.
/// `sink` returns false to stop early,
/// which also stops the preparers.
pub(crate) fn prepare_in_order(names: &[String], batch: usize, sink: &mut impl BatchSink) {
    let chunks: Vec<&[String]> = names.chunks(batch.max(1)).collect();
    let available: usize = match thread::available_parallelism() {
        Ok(count) => count.get(),
        Err(_error) => 1,
    };
    let workers: usize = available.min(MAX_PREPARERS);
    if workers <= 1 || chunks.len() <= 1 {
        for (number, chunk) in chunks.iter().enumerate() {
            if !sink.accept(number, prepare_batch(chunk)) {
                return;
            }
        }
        return;
    }
    let next_chunk: AtomicUsize = AtomicUsize::new(0);
    let halted: AtomicBool = AtomicBool::new(false);
    #[cfg_attr(
        dylint_lib = "monochromatic_lint_closure",
        expect(
            avoidable_closure,
            reason = "std::thread::scope takes only a callable bounded by the scope, and the scoped spawn takes only a callable that borrows the counters and the work list; no named function can carry them and no loop can start the threads"
        )
    )]
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
                if !sink.accept(expected, ready) {
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
///       A batch of ten thousand prepares in about thirty milliseconds and is pushed in about a millisecond,
/// so no
///  lock hold or preparation step is long enough to notice.
pub const FEED_BATCH: usize = 10_000;

/// What:
///      `pub fn feed_library(service: &Arc<SearchService>, names: Arc<Vec<String>>, batch: usize, pause: Duration)`.
///  Starts a feed and returns at once;
/// a newer feed cancels this one.
/// Why:
///       See the module description.
/// `pause` sleeps between batches;
/// production passes zero and tests pass a delay to
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

/// What:
/// `path_bytes` sums the lengths of the names.
/// Why:
/// reserving the arenas up front keeps buffer growth,
/// which copies the buffer,
/// out of the write lock.
fn path_bytes(names: &[String]) -> usize {
    return names.iter().map(String::len).sum();
}

/// What:
/// `feed_in_place` fills the live index batch by batch.
/// Why:
/// the index is empty,
/// so every track added is a track that was not searchable before.
fn feed_in_place(service: &SearchService, names: &[String], epoch: u64, batch: usize, pause: Duration) {
    let generation: u64 = service.reset(names.len(), path_bytes(names));
    let mut feed = InPlaceFeed { service, epoch, generation, pause, finished: true };
    prepare_in_order(names, batch, &mut feed);
    if feed.finished {
        service.finish(generation);
        tracing::info!(tracks = names.len(), epoch, "library feed finished in place");
    }
}

/// What:
/// `feed_on_the_side` builds a second index and swaps it in when complete.
/// Why:
/// the live index still answers for the previous library,
/// and replacing it at the end avoids a flash of
/// empty results during a rescan.
fn feed_on_the_side(service: &SearchService, names: &[String], epoch: u64, batch: usize, pause: Duration) {
    let mut fresh: SearchIndex = SearchIndex::new(0);
    fresh.reserve(names.len(), path_bytes(names), path_bytes(names) / 2);
    let mut feed = SideFeed { service, epoch, fresh, pause, finished: true };
    prepare_in_order(names, batch, &mut feed);
    if feed.finished && service.feed_is_current(epoch) {
        service.install(feed.fresh);
        tracing::info!(tracks = names.len(), epoch, "library feed installed from the side");
    }
}
