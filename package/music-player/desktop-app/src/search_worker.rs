//! Search worker threads: the query worker, the warm-up thread and what they share.
//!
//! What: `Shared` holds the index lock, the generation and the queues; `work` answers the newest query and re-answers
//! it after appends; `warm` refreshes single-character answers once the library is quiet.
//! Why: see `search_service.rs`; this file is the thread side of that service, split out to keep each file short.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! worker.onmessage = (m) => m.kind === "query" ? answer(m) : scheduleRefresh();
//! ```

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, TryRecvError};
use std::sync::{Arc, Mutex, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockError};
use std::thread;
use std::time::{Duration, Instant};

use crate::search_index::SearchIndex;
use crate::search_index_top::warm_characters;
use crate::search_results::SearchState;

/// What:
///      `pub const WRITE_POLL: Duration`.
///  How long the appender sleeps between attempts to take the write lock.
/// Why:
///       Polling with `try_write` lets readers keep entering while a long ranking runs, which a blocked `write`
///  would prevent on platforms whose locks prefer a waiting writer.
pub const WRITE_POLL: Duration = Duration::from_micros(200);

/// What:
///      `pub const WRITE_PATIENCE: Duration`.
///  How long the appender polls before it waits for the lock.
/// Why:
///       Polling must end, or a stream of readers could starve the appender and the scan would never finish.
pub const WRITE_PATIENCE: Duration = Duration::from_millis(50);

/// What:
///      `pub const REFRESH_INTERVAL: Duration`.
///  The fewest time between two refresh answers.
/// Why:
///       A scan appends many batches per second; re-answering after each one would only repaint the same rows.
pub const REFRESH_INTERVAL: Duration = Duration::from_millis(120);

/// What:
///      `pub const WARM_DELAY: Duration`.
///  How long the worker sits idle before it refreshes single-character answers.
/// Why:
///       Warming holds the read lock for a ranking, so it waits until the user has stopped typing.
pub const WARM_DELAY: Duration = Duration::from_millis(250);

/// What:
///      `pub struct SearchAnswer`.
///  One answer delivered to the window.
/// Why:
///       The window needs the sequence to drop answers to an older query, the refresh flag to choose between
///  replacing the list and updating it in place, and the completeness flag because a partial scan cannot
///  establish "no match" (D69).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchAnswer {
    /// Sequence number `query` returned for the query this answers.
    pub sequence: u64,
    /// The query text.
    pub query: String,
    /// The ranked state.
    pub state: SearchState,
    /// How many tracks the index held when it answered.
    pub known_tracks: usize,
    /// Whether the scan that feeds the index has finished.
    pub complete: bool,
    /// True when this re-answers the same query because tracks were appended.
    pub refresh: bool,
}

/// What:
///      `enum Request`.
///  Messages to the worker.
/// Why:
///       Queries, refresh signals and stop all arrive on one queue so the worker can coalesce them.
pub(crate) enum Request {
    /// A new query with its sequence number.
    Query(u64, String),
    /// Tracks were appended or the scan finished.
    Refresh,
    /// Stop the worker.
    Stop,
}

/// What:
///      `struct Shared`.
///  State shared by the service handle, the worker and feeder threads.
/// Why:
///       Everything that crosses threads is here, behind `Arc`.
pub(crate) struct Shared {
    /// The index; replaced when a new library starts.
    pub(crate) index: RwLock<SearchIndex>,
    /// Serializes appenders so batch positions stay contiguous.
    pub(crate) append_order: Mutex<()>,
    /// Number of the current library.
    pub(crate) generation: AtomicU64,
    /// Whether the scan feeding the index has finished.
    pub(crate) complete: AtomicBool,
    /// Queue to the worker.
    pub(crate) requests: Sender<Request>,
    /// Queue to the warmer.
    pub(crate) warm_signals: Sender<Request>,
    /// Number of the newest library feed; an older feeder stops when it sees a larger number.
    pub(crate) feed_epoch: AtomicU64,
    /// Queue to the stream thread.
    pub(crate) stream: Sender<crate::search_stream::StreamRequest>,
}

/// Methods of `Shared`.
impl Shared {
    /// What: `signal_grew` tells the worker to re-answer and the warmer to start its quiet wait.
    /// Why: both react to every append, reset and finish.
    pub(crate) fn signal_grew(&self) {
        let _refresh = self.requests.send(Request::Refresh);
        let _warm = self.warm_signals.send(Request::Refresh);
    }
}

/// What:
///      `pub(crate) fn read_index(shared: &Shared) -> RwLockReadGuard<'_, SearchIndex>`.
///  Takes the read lock, ignoring poisoning.
/// Why:
///       The index is append-only, so a panic in another thread leaves it consistent.
pub(crate) fn read_index(shared: &Shared) -> RwLockReadGuard<'_, SearchIndex> {
    return shared.index.read().unwrap_or_else(PoisonError::into_inner);
}

/// What:
///      `pub(crate) fn write_when_free(shared: &Shared) -> RwLockWriteGuard<'_, SearchIndex>`.
///  Takes the write lock without queueing ahead of readers for long.
/// Why:
///       See `WRITE_POLL` and `WRITE_PATIENCE`.
pub(crate) fn write_when_free(shared: &Shared) -> RwLockWriteGuard<'_, SearchIndex> {
    let started: Instant = Instant::now();
    loop {
        match shared.index.try_write() {
            Ok(guard) => return guard,
            Err(TryLockError::Poisoned(poisoned)) => return poisoned.into_inner(),
            Err(TryLockError::WouldBlock) => {
                if started.elapsed() > WRITE_PATIENCE {
                    return shared.index.write().unwrap_or_else(PoisonError::into_inner);
                }
                thread::sleep(WRITE_POLL);
            }
        }
    }
}

/// What:
///      `fn answer_for(shared: &Shared, sequence: u64, query: &str, refresh: bool) -> SearchAnswer`.
///  Ranks one query under the read lock.
/// Why:
///       The worker calls it for new queries and for refreshes.
fn answer_for(shared: &Shared, sequence: u64, query: &str, refresh: bool) -> SearchAnswer {
    let guard = read_index(shared);
    return SearchAnswer {
        sequence,
        query: query.to_string(),
        state: guard.search(query),
        known_tracks: guard.track_count(),
        complete: shared.complete.load(Ordering::Acquire),
        refresh,
    };
}

/// What:
///      `fn warm_one(shared: &Shared) -> bool`.
///  Refreshes the cached answer of the largest first-character bucket that is behind.
/// Why:
///       One character per call keeps each read-lock hold short and lets the warmer see a new signal between calls.
fn warm_one(shared: &Shared) -> bool {
    let guard = read_index(shared);
    return match warm_characters(&guard).first() {
        Some(character) => {
            guard.warm(*character);
            true
        }
        None => false,
    };
}

/// What:
///      `fn work(shared: Arc<Shared>, requests: Receiver<Request>, deliver: Box<dyn Fn(SearchAnswer) + Send>)`.
///  The query worker: coalesces requests, answers the latest query, and throttles refreshes.
/// Why:
///       Search must run off the window thread, and only the newest query matters.
pub(crate) fn work(shared: Arc<Shared>, requests: Receiver<Request>, deliver: Box<dyn Fn(SearchAnswer) + Send>) {
    let mut current: Option<(u64, String)> = None;
    let mut refresh_due: Option<Instant> = None;
    let mut last_answer: Instant = Instant::now();
    loop {
        let wait: Duration = refresh_due.map_or(Duration::from_secs(3600), |due| return due.saturating_duration_since(Instant::now()));
        let mut inbox: Vec<Request> = Vec::new();
        match requests.recv_timeout(wait) {
            Ok(first) => {
                inbox.push(first);
                inbox.extend(requests.try_iter());
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }
        let mut new_query: Option<(u64, String)> = None;
        for request in inbox {
            match request {
                Request::Query(sequence, text) => new_query = Some((sequence, text)),
                Request::Refresh => {
                    refresh_due.get_or_insert(last_answer + REFRESH_INTERVAL);
                }
                Request::Stop => return,
            }
        }
        if let Some((sequence, text)) = new_query {
            tracing::debug!(sequence, query = %text, "search worker answers a new query");
            let computing: Instant = Instant::now();
            let answer: SearchAnswer = answer_for(&shared, sequence, &text, false);
            tracing::debug!(sequence, compute_ms = computing.elapsed().as_millis(), "search answer computed; delivering it");
            deliver(answer);
            last_answer = Instant::now();
            current = Some((sequence, text));
            refresh_due = None;
        } else if refresh_due.is_some_and(|due| return Instant::now() >= due) {
            refresh_due = None;
            if let Some((sequence, text)) = &current {
                tracing::debug!(sequence, "search worker refreshes the current query");
                let computing: Instant = Instant::now();
                let answer: SearchAnswer = answer_for(&shared, *sequence, text, true);
                tracing::debug!(sequence, compute_ms = computing.elapsed().as_millis(), "refresh answer computed; delivering it");
                deliver(answer);
                last_answer = Instant::now();
            }
        }
    }
}

/// What:
///      `fn warm(shared: Arc<Shared>, signals: Receiver<Request>)`.
///  The warmer: after the library has been quiet for `WARM_DELAY`, refreshes every stale single-character answer.
/// Why:
///       Doing it on its own thread means a keystroke never waits for a warm-up ranking, and read locks are shared.
pub(crate) fn warm(shared: Arc<Shared>, signals: Receiver<Request>) {
    let mut pending: bool = false;
    loop {
        if !pending {
            match signals.recv() {
                Ok(Request::Stop) | Err(_) => return,
                Ok(_) => pending = true,
            }
        }
        match signals.recv_timeout(WARM_DELAY) {
            Ok(Request::Stop) | Err(RecvTimeoutError::Disconnected) => return,
            Ok(_) => continue,
            Err(RecvTimeoutError::Timeout) => {}
        }
        pending = false;
        loop {
            match signals.try_recv() {
                Ok(Request::Stop) | Err(TryRecvError::Disconnected) => return,
                Ok(_) => {
                    pending = true;
                    break;
                }
                Err(TryRecvError::Empty) => {}
            }
            if !warm_one(&shared) {
                break;
            }
        }
    }
}
