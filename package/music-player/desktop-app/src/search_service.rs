//! Search service: the index behind a lock, a worker thread that answers queries, and the refresh signal.
//!
//! What: `SearchService` appends batches of tracks to a shared `SearchIndex` without blocking readers, answers each
//! query on its own worker thread (never on the caller's thread), and re-answers the latest query after every batch so an
//! open Search page updates in place (D111, D112).
//! Why: the window thread must not wait for ranking, and a scan that adds tracks must not make a keystroke wait. The
//! write lock is taken only for the push; batch preparation runs before it; the writer polls with `try_write` so it
//! never queues ahead of readers; and queries are coalesced so a burst of keystrokes costs one ranking.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const search = new SearchService((answer) => render(answer));
//! search.query("cam");           // answered on a worker, delivered to the callback
//! search.appendBatch(paths);     // push under a short lock, then re-answer the latest query
//! ```

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, PoisonError, RwLock};
use std::thread::{self, JoinHandle};

use crate::search_index::SearchIndex;
use crate::search_index_batch::{PreparedBatch, prepare_batch};
use crate::search_results::SearchState;
use crate::search_stream::{FLUSH_PATIENCE, StreamRequest, append_prepared_batch, reset_index, stream_loop};
use crate::search_worker::{Request, Shared, read_index, warm, work, write_when_free};

pub use crate::search_worker::{REFRESH_INTERVAL, SearchAnswer, WARM_DELAY, WRITE_PATIENCE, WRITE_POLL};

/// What:
///      `pub struct SearchService`.
///  Handle to the shared index and its worker thread.
/// Why:
///       See the module description.
pub struct SearchService {
    /// State shared with the threads.
    shared: Arc<Shared>,
    /// Source of query sequence numbers.
    sequence: AtomicU64,
    /// The worker and the warmer, joined on drop.
    threads: Mutex<Vec<JoinHandle<()>>>,
}

/// Methods of `SearchService`.
impl SearchService {
    /// What: `start` spawns the worker; `deliver` receives every answer on the worker thread.
    /// Why: the caller decides how an answer reaches the window, such as posting it to the event loop.
    pub fn start(deliver: impl Fn(SearchAnswer) + Send + 'static) -> SearchService {
        let (requests, receiver) = mpsc::channel::<Request>();
        let (warm_signals, warm_receiver) = mpsc::channel::<Request>();
        let (stream, stream_receiver) = mpsc::channel::<StreamRequest>();
        let shared: Arc<Shared> = Arc::new(Shared {
            index: RwLock::new(SearchIndex::new(0)),
            append_order: Mutex::new(()),
            generation: AtomicU64::new(0),
            complete: AtomicBool::new(false),
            requests,
            warm_signals,
            feed_epoch: AtomicU64::new(0),
            stream,
        });
        let worker_shared: Arc<Shared> = Arc::clone(&shared);
        let worker: JoinHandle<()> = thread::spawn(move || work(worker_shared, receiver, Box::new(deliver)));
        let warmer_shared: Arc<Shared> = Arc::clone(&shared);
        let warmer: JoinHandle<()> = thread::spawn(move || warm(warmer_shared, warm_receiver));
        let streamer_shared: Arc<Shared> = Arc::clone(&shared);
        let streamer: JoinHandle<()> = thread::spawn(move || stream_loop(streamer_shared, stream_receiver));
        return SearchService {
            shared,
            sequence: AtomicU64::new(0),
            threads: Mutex::new(vec![worker, warmer, streamer]),
        };
    }

    /// What: `query` queues a query and returns its sequence number.
    /// Why: the answer arrives later through the callback; the number lets the receiver drop older answers.
    pub fn query(&self, text: &str) -> u64 {
        let sequence: u64 = self.sequence.fetch_add(1, Ordering::AcqRel) + 1;
        let _sent = self.shared.requests.send(Request::Query(sequence, text.to_string()));
        return sequence;
    }

    /// What: `search_now` ranks a query on the caller's thread.
    /// Why: tests and benchmarks need an answer without a worker round trip.
    #[must_use]
    pub fn search_now(&self, text: &str) -> SearchState {
        return read_index(&self.shared).search(text);
    }

    /// What: `begin_feed` starts a new library feed and returns its number.
    /// Why: a feeder compares the number with `feed_is_current` between batches, so a newer feed cancels an older one.
    pub fn begin_feed(&self) -> u64 {
        return self.shared.feed_epoch.fetch_add(1, Ordering::AcqRel) + 1;
    }

    /// What: `feed_is_current` tells whether feed `epoch` is still the newest.
    /// Why: see `begin_feed`.
    #[must_use]
    pub fn feed_is_current(&self, epoch: u64) -> bool {
        return self.shared.feed_epoch.load(Ordering::Acquire) == epoch;
    }

    /// What: `install` replaces the index with one built on the side and marks it complete.
    /// Why: a rescan builds the new index while the old one keeps answering, then swaps, so results never vanish
    /// and reappear. The swap is one pointer move under the write lock.
    pub fn install(&self, fresh: SearchIndex) -> u64 {
        let _order = self.shared.append_order.lock().unwrap_or_else(PoisonError::into_inner);
        let generation: u64 = self.shared.generation.fetch_add(1, Ordering::AcqRel) + 1;
        let mut replacement: SearchIndex = fresh;
        replacement.set_generation(generation);
        *write_when_free(&self.shared) = replacement;
        self.shared.complete.store(true, Ordering::Release);
        self.shared.signal_grew();
        return generation;
    }

    /// What: `generation` is the number of the current library.
    /// Why: a feeder compares it to stop appending to a library that was replaced.
    #[must_use]
    pub fn generation(&self) -> u64 {
        return self.shared.generation.load(Ordering::Acquire);
    }

    /// What: `track_count` is the number of tracks in the index.
    /// Why: tests and status text read it.
    #[must_use]
    pub fn track_count(&self) -> usize {
        return read_index(&self.shared).track_count();
    }

    /// What: `heap_bytes` estimates the index memory.
    /// Why: the scale tests report the cost per million tracks.
    #[must_use]
    pub fn heap_bytes(&self) -> usize {
        return read_index(&self.shared).heap_bytes();
    }

    /// What: `reset` starts a new library: an empty index with room for the expected tracks, and a new generation.
    /// Why: a rescan that is not a pure append invalidates every position, so the old index is dropped.
    pub fn reset(&self, expected_tracks: usize, expected_path_bytes: usize) -> u64 {
        return reset_index(&self.shared, expected_tracks, expected_path_bytes);
    }

    /// What: `append_for` adds tracks to library `generation`; it does nothing when that library was replaced.
    /// Why: a feeder thread may outlive the library it was feeding, and its late batch must not land in the new one.
    pub fn append_for<S: AsRef<str>>(&self, generation: u64, display_paths: &[S]) -> bool {
        return append_prepared_batch(&self.shared, generation, prepare_batch(display_paths));
    }

    /// What: `append_prepared` adds an already prepared batch to library `generation`.
    /// Why: the feeder prepares batches on several threads and pushes them here in order.
    pub(crate) fn append_prepared(&self, generation: u64, batch: PreparedBatch) -> bool {
        return append_prepared_batch(&self.shared, generation, batch);
    }

    /// What: `append_batch` adds tracks to the current library.
    /// Why: a scan that owns the library calls it once per batch.
    pub fn append_batch<S: AsRef<str>>(&self, display_paths: &[S]) {
        let generation: u64 = self.generation();
        self.append_for(generation, display_paths);
    }

    /// What: `stream_batch` hands a batch found by the folder walk to the stream thread, placed by its first position.
    /// Why: D111: tracks become searchable while the walk still runs. Only the move of the names happens on the
    /// caller's thread, which is the walk's. A batch that starts at position zero begins a new library and cancels any
    /// feed still running; a batch that does not continue the index is ignored by the stream thread.
    pub fn stream_batch(&self, first_index: usize, names: Vec<String>) {
        let _sent = self.shared.stream.send(StreamRequest::Batch(first_index, names));
    }

    /// What: `stream_flush` waits until the stream thread has appended every batch handed to it so far.
    /// Why: the final queue is compared with the streamed index only after the index has caught up.
    pub fn stream_flush(&self) {
        let (done, answered) = mpsc::channel::<()>();
        if self.shared.stream.send(StreamRequest::Flush(done)).is_ok() {
            let _waited = answered.recv_timeout(FLUSH_PATIENCE);
        }
    }

    /// What: `adopt_streamed` tells whether the index already holds exactly `names`, and if so marks it complete.
    /// Why: when the walk ends the queue arrives as one list; if the streamed index equals it, there is nothing to
    /// rebuild. Equality is checked at evenly spaced positions, which a wrong prefix or a different order would fail.
    pub fn adopt_streamed(&self, names: &[String]) -> bool {
        self.stream_flush();
        let (same, generation): (bool, u64) = {
            let guard = read_index(&self.shared);
            let count: usize = names.len();
            let same: bool = guard.track_count() == count
                && count > 0
                && (0..16_usize).map(|step| return step * (count - 1) / 15).all(|position| return guard.path(position) == names[position]);
            (same, guard.generation())
        };
        if same {
            self.finish(generation);
        }
        return same;
    }

    /// What: `finish` marks the scan of library `generation` as complete and signals a refresh.
    /// Why: only a complete scan can support the statement that nothing matches (D69).
    pub fn finish(&self, generation: u64) {
        if self.generation() == generation {
            self.shared.complete.store(true, Ordering::Release);
            self.shared.signal_grew();
        }
    }
}

impl SearchService {
    /// What: `shutdown` stops the worker, the warmer and the streamer and waits for all three; a second call does nothing.
    /// Why: the window thread keeps the last handle in a `thread_local!`. On Windows that value is dropped while the
    /// process is already ending, after the other threads were killed, and joining a killed thread panics inside the
    /// thread-local destructor (`threads should not terminate unexpectedly`, then an abort with exit code 0xC0000409).
    /// `main` calls this while every thread is still alive.
    pub fn shutdown(&self) {
        let _worker_stop = self.shared.requests.send(Request::Stop);
        let _warmer_stop = self.shared.warm_signals.send(Request::Stop);
        let _streamer_stop = self.shared.stream.send(StreamRequest::Stop);
        let running: Vec<JoinHandle<()>> = std::mem::take(&mut *self.threads.lock().unwrap_or_else(PoisonError::into_inner));
        for handle in running {
            let _joined = handle.join();
        }
    }
}

/// Teardown of `SearchService`.
impl Drop for SearchService {
    /// What: `drop` stops the worker and waits for it.
    /// Why: the worker must not outlive the callback target.
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Service and feeder tests in `search_service_tests.rs`,
///  pulled in at this point.
#[cfg(test)]
#[path = "search_service_tests.rs"]
mod tests;
