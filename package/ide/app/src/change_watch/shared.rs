//! State shared by the UI handle,
//!  the watch thread,
//!  and notify's event-handler thread.

/// A finished scan of folders for the language servers.
use super::server_scan::Scan;
/// What:
///  `BTreeSet` is an ordered set of owned paths;
///  `PathBuf` owns a path,
///  `Path` borrows one.
/// Why:
///  Pending invalidations collapse repeated events for one directory into a single entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pending = new Set<string>();
/// ```
use std::{
    collections::BTreeSet,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};
/// What:
///  `UnboundedSender` is the sending end of tokio's queue without a size limit;
///  sending never waits.
/// Why:
///  Changes go from notify's thread to the language worker's async loop,
///  and neither may block.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const feed = new Queue<ServerChange>();
/// ```
use tokio::sync::mpsc::UnboundedSender;

/// What happened to a path inside a folder watched for the language servers.
///
/// What:
///  an `enum` with three payload-free variants,
///  like a TS string-literal union.
/// Why:
///  The protocol tells servers whether a file was created,
///  changed,
///  or deleted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ServerChangeKind = 'created' | 'changed' | 'deleted';
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServerChangeKind {
    /// The path appeared:
    ///  created,
    ///  or moved or renamed into place.
    Created,
    /// The path's content or attributes changed.
    Changed,
    /// The path disappeared:
    ///  deleted,
    ///  or moved or renamed away.
    Deleted,
}

/// One change for the language servers:
///  an absolute path inside the project and what happened to it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerChange {
    /// The changed file or folder.
    pub path: PathBuf,
    /// What happened to it.
    pub kind: ServerChangeKind,
}

/// What arrived for the language servers since the watch thread last looked;
///  taken as one value.
#[derive(Debug, Default)]
pub(super) struct ServerRequests {
    /// The feed was set or cleared.
    pub(super) feed_changed: bool,
    /// Paths that may be new folders inside a watched folder;
    ///  the scan thread checks which are.
    pub(super) candidates: BTreeSet<PathBuf>,
    /// Folders whose contents must be scanned again.
    pub(super) rescans: BTreeSet<PathBuf>,
    /// Watched folders that were removed or moved away;
    ///  their watches and everything below them go.
    pub(super) gone: BTreeSet<PathBuf>,
    /// Finished scans.
    pub(super) scanned: Vec<Scan>,
}

/// The language servers' part of the shared state:
///  where changes go and which folders are watched for them.
#[derive(Debug, Default)]
pub(super) struct ServerShared {
    /// What:
    ///  `Option<UnboundedSender<ServerChange>>` is the queue into the language worker,
    ///  or `None`.
    /// Why:
    ///  Folders are watched for the servers only while some server registered file watchers.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// feed?: Queue<ServerChange>;
    /// ```
    pub(super) feed: Option<UnboundedSender<ServerChange>>,
    /// Folders with a live watch for the servers,
    ///  as last published by the watch thread.
    pub(super) watched: BTreeSet<PathBuf>,
    /// Watched folders that held no entries when scanned;
    ///  their first change asks to classify them.
    pub(super) provisional: BTreeSet<PathBuf>,
    /// What arrived for the watch thread since it last looked.
    pub(super) requests: ServerRequests,
}

/// How finished an observed change to the displayed file looks.
///
/// What:
///  an `enum` with three payload-free variants,
///  like a TS string-literal union.
/// Why:
///  A closed write can be read now;
///  a write still in progress waits briefly,
///      so a truncated or half-written file never replaces the displayed text;
///  a reread with no
///      write behind it is read promptly,
///  but only once the file has been quiet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SourceChange = 'settled' | 'unsettled' | 'reread';
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceChange {
    /// Closed after writing,
    ///  renamed into place,
    ///  removed,
    ///  or changed in permissions:
    ///  read now.
    Settled,
    /// Created or written but not yet closed:
    ///  read after the writer goes quiet.
    Unsettled,
    /// No write was seen:
    ///  a new watch,
    ///  a newly displayed file,
    ///  or a full reread after lost events.
    /// Read promptly,
    ///  but like a timer read:
    ///  only bytes that have been quiet for the write-quiet period.
    Reread,
}

/// Invalidations accumulated since the last `ChangeWatcher::take`;
///  events carry no file data.
#[derive(Debug, Default)]
pub struct Changes {
    /// Watched directories whose entries changed,
    ///  or whose watch just started and needs a fresh read.
    pub directories: BTreeSet<PathBuf>,
    /// What:
    ///  `Option<SourceChange>` is either `Some(change)` or `None`,
    ///  like `SourceChange | undefined`.
    /// Why:
    ///  Only the latest event for the displayed file matters;
    ///  `None` means it was untouched.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// source?: SourceChange;
    /// ```
    pub source: Option<SourceChange>,
    /// Reread everything shown:
    ///  queue overflow,
    ///  a notification error,
    ///  a lost or failed watch,
    ///  or a stopped watcher.
    pub everything: bool,
    /// The directories with a live watch,
    ///  present only when that set changed since the last take.
    pub watched: Option<BTreeSet<PathBuf>>,
}

/// One lock guards all cross-thread state;
///  no holder performs a filesystem call while holding it.
#[derive(Debug)]
pub(super) struct Shared {
    /// Canonical project root;
    ///  events and watches outside it are ignored.
    pub(super) root: PathBuf,
    /// Latest directory set the UI wants watched,
    ///  not yet applied by the watch thread.
    pub(super) desired: Option<BTreeSet<PathBuf>>,
    /// Displayed file whose own events become source invalidations.
    pub(super) file: Option<PathBuf>,
    /// Retry watches that failed earlier,
    ///  for example after a directory was recreated.
    pub(super) retry: bool,
    /// The user acted on the tree (scrolled),
    ///  so watches waiting on the limit are tried without the backoff.
    pub(super) user_retry: bool,
    /// Set once by the UI handle's Drop;
    ///  the watch thread exits at its next wake.
    pub(super) closing: bool,
    /// Live watches whose directory was removed or renamed;
    ///  the watch thread drops and re-adds them.
    pub(super) stale: BTreeSet<PathBuf>,
    /// Invalidations for the UI,
    ///  drained by `take`.
    pub(super) pending: Changes,
    /// Directories with a live watch,
    ///  as last published by the watch thread.
    pub(super) watched: BTreeSet<PathBuf>,
    /// The watched set changed since the UI last took it.
    pub(super) watched_changed: bool,
    /// Folders watched for the language servers and the changes sent to them.
    pub(super) server: ServerShared,
}

/// Construct the empty shared state for one project root.
impl Shared {
    /// Nothing is watched or pending until the UI names what it shows.
    pub(super) fn new(root: PathBuf) -> Self {
        return Self {
            root,
            // What: `None` is the empty variant of `Option`, like `undefined`.
            // Why: No desired set has been sent yet, which differs from an empty set.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // desired: undefined,
            // ```
            desired: None,
            file: None,
            retry: false,
            user_retry: false,
            closing: false,
            stale: BTreeSet::new(),
            pending: Changes::default(),
            watched: BTreeSet::new(),
            watched_changed: false,
            server: ServerShared::default(),
        };
    }
}

/// Lock the shared state,
///  recovering it if a thread panicked while holding the lock.
///
/// What:
///  `MutexGuard<'_, Shared>` is the unlocked view;
///  the lock is released when it goes out of scope.
///       `'_` ties the guard's lifetime to the borrowed `Mutex`.
/// Why:
///  A poisoned lock only means another thread panicked;
///  the sets inside stay valid,
///      and refusing them would stop all refresh instead of degrading to rereads.
/// Gotcha:
///  Unlike a TS object,
///  the data is reachable only through the guard.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lock(shared: Mutex<Shared>): Shared { return shared.acquire(); }
/// ```
pub(super) fn lock(shared: &Mutex<Shared>) -> MutexGuard<'_, Shared> {
    // What: `match` on the lock `Result`: `Ok(guard)` is the normal case, `Err(poisoned)` wraps the guard
    //       of a lock whose previous holder panicked; `into_inner` takes the guard out anyway.
    // Why: Keep watching after an unexpected panic elsewhere, and say so in the log.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { return shared.acquire(); } catch (poisoned) { log(poisoned); return poisoned.guard; }
    // ```
    match shared.lock() {
        Ok(guard) => {
            return guard;
        }
        Err(poisoned) => {
            tracing::error!(
                "change-watch state lock was poisoned by a panic; continuing with its contents"
            );
            return poisoned.into_inner();
        }
    }
}
