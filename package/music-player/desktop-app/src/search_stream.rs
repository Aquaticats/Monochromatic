//! The stream thread: takes the batches the folder walk reports and appends them to the Search index.
//!
//! What: `stream_loop` receives `(first_index, names)` from the engine thread over a channel, prepares each batch and
//! pushes it, in order, on its own thread. `reset_index` and `append_prepared_batch` are the two operations on the
//! shared index that this thread and the service handle both use.
//! Why: D111 makes tracks searchable while the walk runs, but the walk is on the engine thread, and preparing a batch
//! costs milliseconds and the push may wait for readers. The engine thread must only hand the names over, so the walk,
//! and the first rows of the library that wait for it, are not slowed by the feed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! streamWorker.onmessage = ({ firstIndex, names }) => index.append(prepare(names)); // off the walk's thread
//! ```

use std::sync::atomic::Ordering;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, PoisonError};
use std::time::Duration;

use crate::search_index::SearchIndex;
use crate::search_index_batch::{PreparedBatch, prepare_batch};
use crate::search_worker::{Shared, read_index, write_when_free};

/// What:
///      `pub(crate) const STREAM_RESERVE_TRACKS: usize = 1_000_000`.
///  How many tracks a streamed library reserves room for before its size is known.
/// Why:
///       Reserved memory is address space until written, so a small library does not pay for it, and the buffers never
///  have to be copied under the write lock while the walk runs.
pub(crate) const STREAM_RESERVE_TRACKS: usize = 1_000_000;

/// What:
///      `pub(crate) const STREAM_RESERVE_BYTES: usize = 100`.
///  The bytes of display path reserved per streamed track.
/// Why:
///       See `STREAM_RESERVE_TRACKS`.
pub(crate) const STREAM_RESERVE_BYTES: usize = 100;

/// What:
///      `pub(crate) const FLUSH_PATIENCE: Duration`.
///  How long `flush` waits for the stream thread to catch up.
/// Why:
///       A flush must not hang the window if the thread is gone.
pub(crate) const FLUSH_PATIENCE: Duration = Duration::from_secs(120);

/// What:
///      `pub(crate) enum StreamRequest`.
///  Messages to the stream thread.
/// Why:
///       Batches arrive in walk order; a flush answers once everything sent before it has been appended.
pub(crate) enum StreamRequest {
    /// A batch with the queue position of its first name.
    Batch(usize, Vec<String>),
    /// Answer on the sender once every earlier batch is appended.
    Flush(Sender<()>),
    /// Stop the thread.
    Stop,
}

/// What:
///      `pub(crate) fn reset_index(shared: &Shared, expected_tracks: usize, expected_path_bytes: usize) -> u64`.
///  Replaces the index with an empty one of a new generation and room for the expected tracks.
/// Why:
///       A library that cannot be expressed as an append starts over; the old index is dropped under the write lock.
pub(crate) fn reset_index(shared: &Shared, expected_tracks: usize, expected_path_bytes: usize) -> u64 {
    let _order = shared.append_order.lock().unwrap_or_else(PoisonError::into_inner);
    let generation: u64 = shared.generation.fetch_add(1, Ordering::AcqRel) + 1;
    let mut fresh: SearchIndex = SearchIndex::new(generation);
    fresh.reserve(expected_tracks, expected_path_bytes, expected_path_bytes / 2);
    *write_when_free(shared) = fresh;
    shared.complete.store(false, Ordering::Release);
    shared.signal_grew();
    return generation;
}

/// What:
///      `pub(crate) fn append_prepared_batch(shared: &Shared, generation: u64, batch: PreparedBatch) -> bool`.
///  Appends a prepared batch to library `generation`, or refuses when that library was replaced.
/// Why:
///       A feeder may outlive its library, and its late batch must not land in the new one.
pub(crate) fn append_prepared_batch(shared: &Shared, generation: u64, batch: PreparedBatch) -> bool {
    let _order = shared.append_order.lock().unwrap_or_else(PoisonError::into_inner);
    if shared.generation.load(Ordering::Acquire) != generation {
        return false;
    }
    write_when_free(shared).append(batch);
    shared.signal_grew();
    return true;
}

/// What:
///      `fn apply_batch(shared: &Shared, first_index: usize, names: &[String])`.
///  Appends one streamed batch, starting a new library when it begins at position zero.
/// Why:
///       A batch that does not continue the index would put names at positions that are not queue positions, so it is
///  ignored and logged.
fn apply_batch(shared: &Shared, first_index: usize, names: &[String]) {
    if first_index == 0 {
        let _cancelled = shared.feed_epoch.fetch_add(1, Ordering::AcqRel);
        reset_index(shared, STREAM_RESERVE_TRACKS, STREAM_RESERVE_TRACKS * STREAM_RESERVE_BYTES);
        tracing::info!(batch = names.len(), "scan stream: the first batch is being appended");
    }
    let known: usize = read_index(shared).track_count();
    if known != first_index {
        tracing::warn!(known, first_index, "ignored a scan batch that does not continue the index");
        return;
    }
    let generation: u64 = shared.generation.load(Ordering::Acquire);
    append_prepared_batch(shared, generation, prepare_batch(names));
}

/// What:
///      `pub(crate) fn stream_loop(shared: Arc<Shared>, inbox: Receiver<StreamRequest>)`.
///  The stream thread's loop.
/// Why:
///       See the module description.
pub(crate) fn stream_loop(shared: Arc<Shared>, inbox: Receiver<StreamRequest>) {
    for request in inbox {
        match request {
            StreamRequest::Batch(first_index, names) => apply_batch(&shared, first_index, &names),
            StreamRequest::Flush(done) => {
                let _answered = done.send(());
            }
            StreamRequest::Stop => return,
        }
    }
}
