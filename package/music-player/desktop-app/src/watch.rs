//! The Source Root file watcher.
//!  It watches the current Source Root recursively and,
//! on any debounced filesystem change,
//!  reports a `Change`: the union of the paths the events named,
//!  or "everything" when events were lost (an overflow or an error) and after a registration.
//!  The queue is still the scan of the root,
//!  but D113 refreshes only the named paths,
//!  so the library is not walked and stat'ed again for every event.
//! See `doc/decision/music-player-live-update-rescan.md`.
//!
//! Registering the watch walks every directory of the root, which took 28 to 45 seconds at one million tracks, so it
//! runs on its own thread and `watch` only sends a request. Events are debounced here, not by a library, because the
//! library cannot tell a read of a directory from a change to it, and the rescan's own walk reads every directory.

/// Path types for the watched root.
use std::path::{Path, PathBuf};

/// What: `use crate::change_set::Change;`. What a burst of events means for the library.
/// Why: The debounce thread merges the events of a burst into one `Change`.
use crate::change_set::Change;

/// Time spans and clocks for the debounce window.
use std::time::{Duration, Instant};

/// Channels, a shared counter with its wake-up, and the two threads.
use std::sync::{Arc, Condvar, Mutex, mpsc};
use std::thread;

/// What:
///      `use notify_debouncer_mini::notify::{...};`.
///  The raw watcher, its event type and the recursive mode, from the `notify` crate the debouncer re-exports.
/// Why:
///       Events are filtered by kind before they count as a change, which the debouncer's own events do not allow.
use notify_debouncer_mini::notify::{Event, RecommendedWatcher, RecursiveMode, Watcher, recommended_watcher};

/// What:
///      `const DEBOUNCE_MS: u64 = 500;`.
///  The quiet time after the last change before a rescan is requested.
/// Why:
///       Coalesce bursts (copying an album, an editor's atomic-rename temp files) into one rescan.
const DEBOUNCE_MS: u64 = 500;

/// What:
///      `const MAX_DEBOUNCE_MS: u64 = 5_000;`.
///  The longest a rescan is put off while changes keep arriving.
/// Why:
///       A long copy must still refresh the library now and then, instead of only once it ends.
const MAX_DEBOUNCE_MS: u64 = 5_000;

/// What:
///      `enum WatchRequest { Watch(PathBuf), Stop }`.
///  The two things the registration thread is asked to do.
/// Why:
///       The caller only sends a request, so it never waits for a registration.
enum WatchRequest {
    /// Watch this root recursively, after unwatching the previous one.
    Watch(PathBuf),
    /// Stop the thread.
    Stop,
}

/// What:
///      `pub(crate) struct SourceWatcher { requests, registered }`.
///  A handle to the registration thread, which owns the raw watcher.
/// Why:
///       The controller holds one of these and re-points it whenever the Source Root changes. The engine thread must
///  not wait for the OS watch to be registered: at one million tracks that wait came before the first folder was scanned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class SourceWatcher { requests: Channel<WatchRequest>; registered: Counter; }
/// ```
pub(crate) struct SourceWatcher {
    /// Requests to the registration thread.
    requests: mpsc::Sender<WatchRequest>,
    /// How many watches the thread has finished registering, with the condition variable that announces each one.
    /// Only tests wait for it.
    #[cfg(test)]
    registered: Arc<(Mutex<u64>, Condvar)>,
}

/// What:
///      `impl SourceWatcher { ... }`.
///  The constructor and the re-point method.
/// Why:
///       Create the watcher and its two threads, and watch a given root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class SourceWatcher { static new() {} watch() {} }
/// ```
impl SourceWatcher {
    /// What:
    ///      `pub(crate) fn new<F>(on_change: F) -> Option<SourceWatcher>`, where `F: Fn(Change)`.
    ///  Create the watcher, the debounce thread that calls `on_change`, and the registration thread, or `None`
    ///  when the OS watcher cannot start.
    /// Why:
    ///       The watcher is created here so a failure is reported to the caller, then moved to its thread.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static new(onChange: (change: Change) => void): SourceWatcher | null { ... }
    /// ```
    pub(crate) fn new<F>(on_change: F) -> Option<SourceWatcher>
    where
        F: Fn(Change) + Send + 'static,
    {
        let (changes, quiet_for) = mpsc::channel::<Change>();
        let catch_up: mpsc::Sender<Change> = changes.clone();
        let watcher: RecommendedWatcher = recommended_watcher(move |result: notify_debouncer_mini::notify::Result<Event>| {
            if let Some(change) = Change::of_event(&result) {
                let _sent = changes.send(change);
            }
        })
        .ok()?;
        let (requests, inbox) = mpsc::channel::<WatchRequest>();
        let registered: Arc<(Mutex<u64>, Condvar)> = Arc::new((Mutex::new(0), Condvar::new()));
        let thread_registered: Arc<(Mutex<u64>, Condvar)> = Arc::clone(&registered);
        let _debounce = thread::spawn(move || debounce_loop(quiet_for, on_change));
        let _registration = thread::spawn(move || watch_loop(watcher, inbox, thread_registered, catch_up));
        return Some(SourceWatcher {
            requests,
            #[cfg(test)]
            registered,
        });
    }

    /// What:
    ///      `pub(crate) fn watch(&mut self, root: &Path)`.
    ///  Ask the registration thread to watch `root` recursively, after unwatching any previously-watched root,
    ///  and return at once.
    /// Why:
    ///       Re-point the single watch when the Source Root changes (open, restore).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// watch(root: string): void { this.requests.send({ kind: "watch", root }); }
    /// ```
    pub(crate) fn watch(&mut self, root: &Path) {
        tracing::info!(root = %root.display(), "watch requested; the registration thread registers it");
        let _sent = self.requests.send(WatchRequest::Watch(root.to_path_buf()));
    }

    /// What:
    ///      `pub(crate) fn wait_until_registered(&self, count: u64, patience: Duration) -> bool`.
    ///  Wait until the thread has finished registering `count` watches.
    /// Why:
    ///       A change made before the registration finishes is not seen, so tests that change files wait for it first.
    #[cfg(test)]
    pub(crate) fn wait_until_registered(&self, count: u64, patience: Duration) -> bool {
        let (finished, wake) = &*self.registered;
        let before = finished.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let (after, _timeout) = wake
            .wait_timeout_while(before, patience, |done| return *done < count)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        return *after >= count;
    }
}

/// Teardown of `SourceWatcher`.
impl Drop for SourceWatcher {
    /// What:
    ///      `fn drop(&mut self)`.
    ///  Ask the registration thread to stop, without waiting for it.
    /// Why:
    ///       A registration still running would hold shutdown for as long as it takes; the thread ends when it is done,
    ///  and its watcher closing ends the debounce thread.
    fn drop(&mut self) {
        let _sent = self.requests.send(WatchRequest::Stop);
    }
}

/// What:
///      `fn debounce_loop(quiet_for, on_change)`.
///  The debounce thread: after a change it waits for `DEBOUNCE_MS` without another (at most `MAX_DEBOUNCE_MS`),
///  then calls `on_change` once with the union of what it saw.
/// Why:
///       A burst of events becomes one refresh, and a continuous one still produces a refresh every few seconds.
fn debounce_loop<F: Fn(Change)>(quiet_for: mpsc::Receiver<Change>, on_change: F) {
    while let Ok(first_change) = quiet_for.recv() {
        let mut gathered: Change = first_change;
        let first: Instant = Instant::now();
        loop {
            let waited: Duration = first.elapsed();
            let patience: Duration = Duration::from_millis(DEBOUNCE_MS).min(Duration::from_millis(MAX_DEBOUNCE_MS).saturating_sub(waited));
            if patience.is_zero() {
                break;
            }
            match quiet_for.recv_timeout(patience) {
                Ok(change) => gathered = gathered.merge(change),
                Err(mpsc::RecvTimeoutError::Timeout) => break,
                Err(mpsc::RecvTimeoutError::Disconnected) => return,
            }
        }
        tracing::debug!(waited_ms = first.elapsed().as_millis(), "source change settled; a refresh is requested");
        on_change(gathered);
    }
}

/// What:
///      `fn watch_loop(watcher, inbox, registered, catch_up)`.
///  The registration thread: registers each requested root, counts it, and asks for one rescan once it is registered.
/// Why:
///       Keeps the slow recursive registration off the engine thread. A change made while the registration was still
///  running (28 seconds at one million tracks, after the scan had finished) was not seen by the watch, so one rescan
///  after the registration catches it up. It cannot start a loop, because the rescan's own reads are not changes.
fn watch_loop(
    mut watcher: RecommendedWatcher,
    inbox: mpsc::Receiver<WatchRequest>,
    registered: Arc<(Mutex<u64>, Condvar)>,
    catch_up: mpsc::Sender<Change>,
) {
    let mut watched: Option<PathBuf> = None;
    for request in inbox {
        let root: PathBuf = match request {
            WatchRequest::Watch(root) => root,
            WatchRequest::Stop => return,
        };
        if let Some(previous) = watched.take() {
            let _unwatched = watcher.unwatch(&previous);
        }
        let started: Instant = Instant::now();
        match watcher.watch(&root, RecursiveMode::Recursive) {
            Ok(()) => {
                tracing::info!(root = %root.display(), elapsed_ms = started.elapsed().as_millis(), "watch registered; one rescan catches up");
                watched = Some(root);
                let _sent = catch_up.send(Change::Everything);
            }
            Err(error) => tracing::warn!(root = %root.display(), error = %error, "watch failed"),
        }
        let (finished, wake) = &*registered;
        *finished.lock().unwrap_or_else(std::sync::PoisonError::into_inner) += 1;
        wake.notify_all();
    }
}

/// What:
///      `#[cfg(test)] #[path = "watch_tests.rs"] mod tests;`.
///  Pull the integration test
///           in from the sibling file `watch_tests.rs`;
///  test builds only.
/// Why:
///       Keep `watch.rs` to production code;
///  the test lives beside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // watch_tests.rs is watch.unit.test.ts beside watch.ts
/// ```
#[cfg(test)]
#[path = "watch_tests.rs"]
mod tests;
