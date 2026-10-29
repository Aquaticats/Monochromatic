//! Native project tree,
//!  bounded directory refresh,
//!  and successful-file navigation.

/// One source owner and one native window remain independent of background filesystem reads.
use super::{AppWindow, State};
/// Startup failures must not silently disable project navigation.
use anyhow::Result;
/// Reuse filesystem-free tree state,
///  bounded workers,
///  and editord-compatible session-local history.
use ide_app::{
    change_watch::ChangeWatcher,
    directory_worker::DirectoryWorker,
    file_open::FileOpener,
    file_tree::{FileTree, TreeRow},
    recent::RecentFiles,
    refresh_policy::DirectoryRefresh,
    workspace::Workspace,
};
/// Weak window handles and a retained timer bind worker results to the native event loop.
use slint::{ComponentHandle, SharedString, Timer, TimerMode};
/// UI-thread shared state is separate from worker-owned snapshots and native path identities.
use std::{
    cell::RefCell,
    collections::BTreeSet,
    path::PathBuf,
    rc::Rc,
    time::{Duration, Instant},
};

/// Click,
///  directory-navigation,
///  and Ctrl+digit bindings.
mod actions;
/// Content results set a reading position only after their file is successfully available.
mod line;
/// Successful opens replace source atomically without losing the prior document on errors.
mod open;
/// Visible row presentation and ancestor expansion for reveal.
mod present;
/// Transient combined search shares project and file-open ownership.
mod search;
/// Nonblocking reply consumption and bounded directory refresh scheduling.
mod tick;
/// Watch what is shown and turn change notifications into due rereads.
mod watch;

/// Language targets in other files open through the same latest-request-wins path as tree rows.
pub(super) use open::request_jump;
/// The language worker decides whether folders are watched for the servers;
///  the watcher lives here.
pub(super) use watch::feed_servers;

/// All tree interaction stays on the native event-loop thread;
///  its fields stay private to navigation.
pub(super) struct Navigation {
    /// Sole canonical root,
    ///  also used for visible project context.
    workspace: Workspace,
    /// Cached directory snapshots and expansion/request state.
    tree: FileTree,
    /// One directory read or unread reply at a time.
    reader: DirectoryWorker,
    /// Unexpected reader disconnects suspend retries until restart rather than alternating diagnostics every tick.
    reader_available: bool,
    /// Path associated with the current directory reply for diagnostic recovery.
    reading: Option<PathBuf>,
    /// Last requested directory read;
    ///  a failed first listing is retried at most this often plus 500 ms.
    last_read: Option<Instant>,
    /// inotify watches for `shown` and the displayed file's directory;
    ///  events only mark reads due.
    watcher: ChangeWatcher,
    /// Which shown directory to reread next:
    ///  notified,
    ///  unwatched on the old timer,
    ///  or the safety sweep.
    directories: DirectoryRefresh,
    /// The root plus visible expanded folders,
    ///  in visible order;
    ///  exactly the watched tree directories.
    shown: Vec<PathBuf>,
    /// Directories with a live watch,
    ///  as last reported by the watcher.
    watched: BTreeSet<PathBuf>,
    /// Tree scroll offset at the previous tick,
    ///  to notice the user scrolling.
    tree_scroll: f32,
    /// When scrolling last asked the watcher to retry watches waiting on the limit.
    scroll_retry: Option<Instant>,
    /// Latest directory failure;
    ///  unrelated successes must not erase its diagnostic.
    directory_error: Option<(PathBuf, String)>,
    /// Background source opens retain only the latest requested target.
    opener: FileOpener,
    /// Optional search line belongs to the latest file-open intent,
    ///  never an older reply.
    pending_line: Option<usize>,
    /// Transient search worker and input state remain window-local.
    search: search::Search,
    /// Current flat rows preserve paths independently of their lossy display labels.
    rows: Vec<TreeRow>,
    /// Ten session-local promoted file slots.
    recent: RecentFiles,
    /// Continue expanding/loading ancestors until this successfully opened file can be revealed.
    reveal: Option<PathBuf>,
}

/// Start navigation against the canonical root without synchronously enumerating its directories.
/// Window tests without language navigation keep only the timer;
///  the application uses `bind_shared`.
#[cfg(test)]
pub(super) fn bind(
    window: &AppWindow,
    source: &Rc<RefCell<State>>,
    workspace: Workspace,
) -> Result<Timer> {
    // Only the timer is kept; callers that open no language targets need no navigation handle.
    let (timer, _navigation) = bind_shared(window, source, workspace)?;
    return Ok(timer);
}

/// What:
///  Start navigation and also return its shared state.
///  `(Timer, Rc<RefCell<Navigation>>)`
///       is a pair:
///  the polling timer and the state the timer and callbacks share.
/// Why:
///  The Language module opens definition and reference targets through this state,
///  so they
///      get history,
///  tree reveal,
///  and the latest-request-wins rule of every other open.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function bindShared(window, source, workspace): [Timer, { current: Navigation }]
/// ```
pub(super) fn bind_shared(
    window: &AppWindow,
    source: &Rc<RefCell<State>>,
    workspace: Workspace,
) -> Result<(Timer, Rc<RefCell<Navigation>>)> {
    // What: clone owns the initial file identity without keeping a UI borrow alive through callbacks.
    // Why: History records only the already successful startup open, not pending requests.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const initial = source.current.filePath;
    // ```
    let initial = source.borrow().file_path.clone();
    let mut recent = RecentFiles::default();
    if let Some(path) = &initial {
        recent.opened(path.clone());
    }
    // Workspace clones copy canonical path metadata; workers own their independent read-only boundary.
    let reader = DirectoryWorker::new(workspace.clone())?;
    let opener = FileOpener::new(workspace.clone())?;
    let mut watcher = ChangeWatcher::new(workspace.clone())?;
    // The root is shown before its first listing arrives; watching it now avoids missing early changes.
    let shown = vec![workspace.root().to_path_buf()];
    watcher.watch_only(&shown.iter().cloned().collect(), initial.as_deref());
    let tree = FileTree::new(workspace.root());
    let search = search::Search::new(workspace.clone())?;
    // Keep the full path available to accessibility while showing the distinguishing project name.
    let project_path = workspace.root().display().to_string();
    let project_label = if let Some(name) = workspace.root().file_name() {
        name.to_string_lossy().into_owned()
    } else {
        project_path.clone()
    };
    // Rc/RefCell shares checked mutable UI ownership; workers never receive this shared object.
    let navigation = Rc::new(RefCell::new(Navigation {
        workspace,
        tree,
        reader,
        opener,
        pending_line: None,
        search,
        recent,
        reader_available: true,
        rows: Vec::new(),
        reading: None,
        last_read: None,
        watcher,
        directories: DirectoryRefresh::default(),
        shown,
        watched: BTreeSet::new(),
        tree_scroll: 0.0,
        scroll_retry: None,
        directory_error: None,
        reveal: initial,
    }));
    window.set_project_label(SharedString::from(project_label));
    window.set_project_path(SharedString::from(project_path));
    window.set_project_visible(true);
    actions::bind(window, source, &navigation);
    search::bind(window, source, &navigation);
    // The timer and callback owners retain navigation until window shutdown; Drop joins both readers.
    let timer = Timer::default();
    let active_source = Rc::clone(source);
    let weak_window = window.as_weak();
    let shared = Rc::clone(&navigation);
    timer.start(TimerMode::Repeated, Duration::from_millis(20), move || {
        if let Some(active) = weak_window.upgrade() {
            tick::update(&active, &active_source, &navigation);
        }
    });
    return Ok((timer, shared));
}
