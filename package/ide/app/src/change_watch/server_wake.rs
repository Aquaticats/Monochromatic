//! The watch thread's part for the language servers on each wake:
//!  the feed being set or cleared,
//!  removed
//! folders,
//!  finished scans,
//!  and starting the next scan on its own thread.

/// The record and wake of the event handler,
///  the scan,
///  the servers' bookkeeping,
///  and the shared state.
use super::{
    record::wake,
    server_scan::{Scan, bases, scan},
    server_watch::{ServerWatches, forget_below},
    shared::{ServerChange, ServerChangeKind, ServerRequests, Shared, lock},
};
/// What:
///  ordered sets of owned paths;
///  `Arc`/`Mutex` share the state;
///  `SyncSender` wakes the watch thread;
///       `JoinHandle` joins the scan thread.
/// Why:
///  Scans run on their own thread and hand their results back through the shared state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const worker = new Worker('scan');
/// ```
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, mpsc::SyncSender},
    thread::{self, JoinHandle},
};
/// The queue into the language worker,
///  for the creations a newly watched folder may have missed.
use tokio::sync::mpsc::UnboundedSender;

/// Everything the watch thread keeps for the language servers between wakes.
#[derive(Default)]
pub(super) struct Servers {
    /// Desired,
    ///  live,
    ///  and waiting folders.
    pub(super) watches: ServerWatches,
    /// Some server registered file watchers,
    ///  so folders are watched and changes forwarded.
    active: bool,
    /// The next scan starts watching:
    ///  its files are not reported as created.
    initial: bool,
    /// Which period of watching the next scan belongs to;
    ///  clearing the feed starts a new one.
    generation: u64,
    /// What:
    ///  `Option<JoinHandle<()>>` is the running scan thread,
    ///  or `None`.
    /// Why:
    ///  One scan runs at a time;
    ///  requests that arrive meanwhile wait for the next one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// scanning?: Thread;
    /// ```
    scanning: Option<JoinHandle<()>>,
    /// Paths that may be new folders,
    ///  waiting for the next scan.
    candidates: BTreeSet<PathBuf>,
    /// Folders to scan again,
    ///  waiting for the next scan.
    rescans: BTreeSet<PathBuf>,
}

/// Run the scans of `bases` on their own thread,
///  because ripgrep can take a while on a large project,
/// and hand the results back through the shared state.
fn spawn_scan(
    root: PathBuf,
    bases: Vec<PathBuf>,
    initial: bool,
    generation: u64,
    shared: &Arc<Mutex<Shared>>,
    waker: &SyncSender<()>,
) -> Option<JoinHandle<()>> {
    // `Arc::clone` and `clone` give the thread its own handles to the state and the wake channel.
    let thread_shared = Arc::clone(shared);
    let thread_waker = waker.clone();
    let spawned = thread::Builder::new()
        .name("ide-server-scan".to_string())
        .spawn(move || {
            let mut results = Vec::new();
            for base in &bases {
                if lock(&thread_shared).closing {
                    return;
                }
                let mut found = scan(&root, base, initial);
                found.generation = generation;
                results.push(found);
            }
            lock(&thread_shared).server.requests.scanned.extend(results);
            wake(&thread_waker);
        });
    // What: `match` on the spawn `Result`: a thread handle, or the reason no thread could start.
    // Why: Without a scan the servers' folders stay as they were; the window is unaffected.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { return spawn(...); } catch (error) { log.warn(error); return undefined; }
    // ```
    match spawned {
        Ok(handle) => {
            return Some(handle);
        }
        Err(error) => {
            tracing::warn!(%error, "cannot start a scan of the project for the language servers");
            return None;
        }
    }
}

/// Replace what was known below the scanned folder with what the scan found.
///  Files in folders that had
/// no watch before,
///  or only a provisional one whose events were held back,
///  appeared without any event
/// the servers heard,
///  so they hear about them as created.
fn apply_scan(
    servers: &mut ServerWatches,
    found: Scan,
    feed: Option<&UnboundedSender<ServerChange>>,
) {
    let base = found.base.clone();
    // What: `filter` keeps the folders below the base that were source folders; `cloned` copies them;
    //       `collect` builds the set.
    // Why: Only those folders have had their changes delivered.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const before = new Set([...desired].filter(path => isInside(path, base)));
    // ```
    let mut before: BTreeSet<PathBuf> = BTreeSet::new();
    for path in servers.desired.iter() {
        if path.starts_with(&base) && !servers.provisional.contains(path) {
            before.insert(path.clone());
        }
    }
    forget_below(&mut servers.desired, &base);
    forget_below(&mut servers.provisional, &base);
    servers.desired.extend(found.directories);
    servers.desired.extend(found.provisional.iter().cloned());
    servers.provisional.extend(found.provisional);
    if found.initial {
        return;
    }
    let Some(sender) = feed else {
        return;
    };
    for file in found.files {
        let known = match file.parent() {
            Some(folder) => before.contains(folder),
            None => false,
        };
        if known {
            continue;
        }
        let message = ServerChange {
            path: file,
            kind: ServerChangeKind::Created,
        };
        if let Err(error) = sender.send(message) {
            tracing::debug!(%error, "the language worker stopped; a found file was not forwarded");
            return;
        }
    }
}

/// What:
///  Apply what arrived for the servers since the last wake:
///  the feed set or cleared,
///  removed folders,
///       and finished scans.
///  Returns true when the desired folders changed.
/// Why:
///  The servers' watches are reconciled right after the tree's,
///  from these desired folders.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function prepare(servers: Servers, requests: ServerRequests, feed?: Queue, root: string): boolean
/// ```
pub(super) fn prepare(
    servers: &mut Servers,
    requests: ServerRequests,
    feed: Option<&UnboundedSender<ServerChange>>,
    root: &Path,
) -> bool {
    let mut desired_changed = false;
    if requests.feed_changed {
        if feed.is_some() && !servers.active {
            tracing::info!(
                "a language server registered file watchers; watching the project's source folders for it"
            );
            servers.active = true;
            servers.initial = true;
            servers.rescans.insert(root.to_path_buf());
        } else if feed.is_none() && servers.active {
            tracing::info!(
                "no language server registers file watchers any more; releasing their folder watches"
            );
            servers.active = false;
            servers.generation += 1;
            servers.candidates.clear();
            servers.rescans.clear();
            servers.watches.desired.clear();
            servers.watches.provisional.clear();
            desired_changed = true;
        }
    }
    if !servers.active {
        return desired_changed;
    }
    servers.candidates.extend(requests.candidates);
    servers.rescans.extend(requests.rescans);
    for path in &requests.gone {
        forget_below(&mut servers.watches.desired, path);
        forget_below(&mut servers.watches.provisional, path);
        desired_changed = true;
    }
    for found in requests.scanned {
        if found.generation == servers.generation {
            apply_scan(&mut servers.watches, found, feed);
            desired_changed = true;
        }
    }
    return desired_changed;
}

/// One scan at a time:
///  requests that arrived during the running one start the next.
pub(super) fn start_scan(
    servers: &mut Servers,
    root: &Path,
    shared: &Arc<Mutex<Shared>>,
    waker: &SyncSender<()>,
) {
    let finished = match servers.scanning.as_ref() {
        Some(handle) => handle.is_finished(),
        None => true,
    };
    let requested = !servers.candidates.is_empty() || !servers.rescans.is_empty();
    if !servers.active || !finished || !requested {
        return;
    }
    servers.finish();
    let wanted = bases(root, &servers.candidates, &servers.rescans);
    servers.candidates.clear();
    servers.rescans.clear();
    if wanted.is_empty() {
        return;
    }
    servers.scanning = spawn_scan(
        root.to_path_buf(),
        wanted,
        servers.initial,
        servers.generation,
        shared,
        waker,
    );
    servers.initial = false;
}

/// Joining the scan thread.
impl Servers {
    /// Join a finished or still running scan;
    ///  it ends on its own once ripgrep finishes.
    pub(super) fn finish(&mut self) {
        if let Some(handle) = self.scanning.take()
            && handle.join().is_err()
        {
            tracing::warn!("a scan of the project for the language servers panicked");
        }
    }
}
