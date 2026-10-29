//! The thread that owns the inotify watcher and applies the UI's desired directory set,
//!  then the language
//! servers' folders.
//!  Adding a watch blocks until notify's loop replies,
//!  so it must never run on the UI thread.

/// The wake's decisions,
///  classified invalidations,
///  the shared state,
///  the servers' part,
///  and the kernel calls.
use super::{
    reconcile::{Kernel, Request, Watches, reconcile},
    record::record,
    server_wake::{Servers, prepare, start_scan},
    server_watch::{ServerRequest, TreeFirst, reconcile_servers},
    shared::{Shared, SourceChange, lock},
    watch_ops::{WatchFailure, add, remove},
};
/// Containment uses the same canonical check as every project read.
use crate::workspace::Workspace;
/// What:
///  notify's inotify backend,
///  its configuration,
///  and the `Watcher` trait that provides `new`.
/// Why:
///  Naming `INotifyWatcher` (not `RecommendedWatcher`) keeps a polling backend from ever being chosen.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { INotifyWatcher, Config } from 'notify';
/// ```
use notify::{Config, Event, EventHandler, INotifyWatcher, Watcher};
/// What:
///  `Arc` shares the state across threads;
///  `Receiver`/`SyncSender` are the bounded wake channel's ends;
///       `RecvTimeoutError` says why a timed wait ended;
///  `Duration` and `Instant` are a time span and a
///       monotonic time point for the watch-limit backoff.
/// Why:
///  The handler and the UI wake this thread;
///  it sleeps in `recv` otherwise,
///  or until a backoff ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Shared, Receiver, BoundedSender } from 'threads';
/// ```
use std::{
    path::Path,
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, RecvTimeoutError, SyncSender},
    },
    time::{Duration, Instant},
};

/// Shortest sleep while waiting for the servers' backoff,
///  so a wait that just ended does not spin.
const SHORTEST_BACKOFF_SLEEP: Duration = Duration::from_millis(50);

/// What:
///  The handler notify calls for every event:
///  it owns the shared state and the wake sender,
///  and records the event.
/// Why:
///  notify calls it on its own thread,
///  so it must own what it uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const handler = (event) => record(shared, waker, event);
/// ```
struct EventRecorder {
    /// The state the UI thread reads.
    shared: Arc<Mutex<Shared>>,
    /// Wakes the watch thread after an event was recorded.
    waker: SyncSender<()>,
}

/// Each event is recorded as a change or an invalidation.
impl EventHandler for EventRecorder {
    /// Record one event or error from notify.
    fn handle_event(&mut self, event: notify::Result<Event>) {
        record(&self.shared, &self.waker, event);
    }
}

/// The real kernel calls:
///  one borrowed watcher and the project boundary its watches must stay inside.
struct Inotify<'a> {
    /// What:
    ///  `&'a mut INotifyWatcher` lends the watcher for as long as `'a`,
    ///  the life of this value.
    /// Why:
    ///  The watch thread keeps owning the watcher;
    ///  a wake only borrows it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// watcher: INotifyWatcher;
    /// ```
    watcher: &'a mut INotifyWatcher,
    /// Canonical project boundary.
    workspace: &'a Workspace,
}

/// Pass each call to notify,
///  after the containment check for adds.
impl Kernel for Inotify<'_> {
    /// Add one non-recursive watch inside the root.
    fn add(&mut self, path: &Path) -> Result<(), WatchFailure> {
        return add(self.watcher, self.workspace, path);
    }

    /// Remove one watch;
    ///  one the kernel already dropped is only logged.
    fn remove(&mut self, path: &Path) {
        remove(self.watcher, path);
    }
}

/// Apply one wake's worth of requests;
///  returns false when the UI handle is closing.
fn apply(
    watcher: &mut INotifyWatcher,
    workspace: &Workspace,
    shared: &Arc<Mutex<Shared>>,
    waker: &SyncSender<()>,
    watches: &mut Watches,
    servers: &mut Servers,
) -> bool {
    let mut guard = lock(shared);
    if guard.closing {
        return false;
    }
    // What: `take` moves the value out and leaves the default (`None`, `false`, empty set) behind.
    // Why: Each request is handled once; later wakes see only newer requests.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const request = { desired: shared.desired, retry: shared.retry, ... }; shared.desired = undefined;
    // ```
    let request = Request {
        desired: guard.desired.take(),
        retry: std::mem::take(&mut guard.retry),
        user_retry: std::mem::take(&mut guard.user_retry),
        stale: std::mem::take(&mut guard.stale),
        file: guard.file.clone(),
    };
    let requests = std::mem::take(&mut guard.server.requests);
    let feed = guard.server.feed.clone();
    drop(guard);
    let desired_changed = prepare(servers, requests, feed.as_ref(), workspace.root());
    let now = Instant::now();
    let mut kernel = Inotify { watcher, workspace };
    // What: `TreeFirst { ... }` wraps the real kernel so the tree shares folders with the servers and
    //       takes one of their watches when the limit refuses it; the block ends that borrow.
    // Why: The decisions stay testable with fake calls; only here do they reach the kernel.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const outcome = reconcile(watches, request, now, new TreeFirst(kernel, servers));
    // ```
    let outcome = {
        let mut first = TreeFirst {
            inner: &mut kernel,
            servers: &mut servers.watches,
            tree_held: watches.active.clone(),
        };
        reconcile(watches, request, now, &mut first)
    };
    let server_request = ServerRequest {
        desired_changed,
        tree_limited: !watches.limited.is_empty(),
    };
    reconcile_servers(
        &mut servers.watches,
        &watches.active,
        server_request,
        now,
        &mut kernel,
    );
    start_scan(servers, workspace.root(), shared, waker);
    let mut published = lock(shared);
    // Retries that change nothing must not make the UI copy the set again.
    if published.watched != watches.active {
        published.watched = watches.active.clone();
        published.watched_changed = true;
    }
    published.server.watched = servers.watches.active.clone();
    published.server.provisional = servers.watches.provisional.clone();
    if outcome.everything {
        published.pending.everything = true;
    }
    // A new watch can miss changes made before it existed, so its directory is read once more.
    // The same holds for a newly displayed file in a directory that was already watched.
    // No write was seen, so a write event recorded in the meantime keeps its own classification.
    let parent = watches.file.as_deref().and_then(Path::parent);
    let reached = match parent {
        Some(directory) => outcome.established.contains(directory),
        None => false,
    };
    if (outcome.switched || reached) && published.pending.source.is_none() {
        published.pending.source = Some(SourceChange::Reread);
    }
    published.pending.directories.extend(outcome.established);
    return true;
}

/// Thread body:
///  create the watcher,
///  then apply requests on every wake until the UI handle closes.
pub(super) fn run(
    workspace: Workspace,
    shared: Arc<Mutex<Shared>>,
    wakes: Receiver<()>,
    waker: SyncSender<()>,
) {
    let handler_shared = Arc::clone(&shared);
    let handler_waker = waker.clone();
    // What: `EventRecorder` is a struct that takes ownership of `handler_shared` and `handler_waker`.
    // Why: notify calls it on its own thread for every event, so it must own what it uses.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const handler = (event) => record(handlerShared, handlerWaker, event);
    // ```
    let created = INotifyWatcher::new(
        EventRecorder {
            shared: handler_shared,
            waker: handler_waker,
        },
        Config::default().with_follow_symlinks(false),
    );
    let mut watcher = match created {
        Ok(watcher) => watcher,
        Err(error) => {
            tracing::warn!(%error, "cannot start inotify file-change watching; rereading shown directories and the displayed file on a timer");
            return;
        }
    };
    tracing::info!(root = %workspace.root().display(), "started inotify file-change watching");
    // `default()` starts with nothing watched, failed, or desired, and the limit not reached.
    let mut watches = Watches::default();
    let mut servers = Servers::default();
    loop {
        // While the servers wait on the limit, the thread also wakes when their backoff ends.
        let mut backoff = None;
        if let Some(limit) = servers.watches.limit {
            backoff = Some(
                limit
                    .until_retry(Instant::now())
                    .max(SHORTEST_BACKOFF_SLEEP),
            );
        }
        // What: `recv_timeout` waits for a wake or the end of the backoff; `Timeout` is the latter.
        // Why: The servers' retries need no UI tick, so they also work in the headless inspection tool.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const woke = backoff === undefined ? await wakes.next() : await Promise.race([wakes.next(), sleep(backoff)]);
        // ```
        let received: Result<(), String> = match backoff {
            Some(wait) => match wakes.recv_timeout(wait) {
                Ok(()) | Err(RecvTimeoutError::Timeout) => Ok(()),
                Err(error) => Err(error.to_string()),
            },
            None => match wakes.recv() {
                Ok(()) => Ok(()),
                Err(error) => Err(error.to_string()),
            },
        };
        if let Err(error) = received {
            tracing::debug!(%error, "change-watch wake channel closed");
            break;
        }
        if !apply(
            &mut watcher,
            &workspace,
            &shared,
            &waker,
            &mut watches,
            &mut servers,
        ) {
            break;
        }
    }
    // A scan still running ends on its own once ripgrep finishes; it is joined so no thread outlives this one.
    servers.finish();
    // Dropping the watcher asks notify's loop to remove every watch and close the inotify descriptor.
    drop(watcher);
    tracing::debug!("stopped inotify file-change watching");
}
