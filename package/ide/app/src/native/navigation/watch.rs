//! Watch exactly what the window shows,
//!  and turn change notifications into due rereads.
//! Notifications never carry listings or text:
//!  the existing readers reread and fence their replies.

/// Navigation owns the watcher and the directory schedule;
///  the source state owns its own schedule.
use super::{AppWindow, Navigation, State};
/// A full reread asks for a reread of the displayed file once it has been quiet;
///  the servers' changes go to
/// the language worker.
use ide_app::change_watch::{ServerChange, SourceChange};
/// What:
///  `Rc<RefCell<State>>` is the UI-thread shared source state;
///  `BTreeSet` is an ordered set.
/// Why:
///  The watcher compares whole sets,
///  so an unchanged tree sends nothing to the watch thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shown = new Set<string>(paths);
/// ```
use std::{
    cell::RefCell,
    collections::BTreeSet,
    path::{Path, PathBuf},
    rc::Rc,
    time::{Duration, Instant},
};
/// What:
///  `UnboundedSender` is the sending end of the language worker's queue without a size limit.
/// Why:
///  The change watcher sends from notify's thread and never waits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Feed = Queue<ServerChange>;
/// ```
use tokio::sync::mpsc::UnboundedSender;

/// What:
///  Watch the project's source folders for the language servers and send their changes to `feed`,
///  or,
///       with `None`,
///  release those watches.
///  `UnboundedSender<ServerChange>` is the language worker's queue.
/// Why:
///  The language worker says whether some server registered file watchers;
///  the watcher lives here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function feedServers(navigation: Navigation, feed: Queue<ServerChange> | undefined): void
/// ```
pub(in crate::native) fn feed_servers(
    navigation: &mut Navigation,
    feed: Option<UnboundedSender<ServerChange>>,
) {
    navigation.watcher.feed_servers(feed);
}

/// Shortest time between two watch retries asked for by scrolling.
const SCROLL_RETRY_GAP: Duration = Duration::from_secs(1);

/// The user scrolling the tree is a moment to retry watches that wait on the inotify limit,
///  without the
/// backoff;
///  while every shown folder is watched,
///  scrolling asks for nothing.
pub(super) fn scrolled(window: &AppWindow, navigation: &mut Navigation, now: Instant) {
    let offset = window.get_tree_scroll_y();
    // What: `f32` comparison of the toolkit's scroll offset with the previous tick's.
    // Why: Any change is a scroll by the user or by a reveal; both mean the tree is in use.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (offset === navigation.treeScroll) return;
    // ```
    if (offset - navigation.tree_scroll).abs() < f32::EPSILON {
        return;
    }
    navigation.tree_scroll = offset;
    let mut unwatched = false;
    for path in navigation.shown.iter() {
        if !navigation.watched.contains(path) {
            unwatched = true;
            break;
        }
    }
    let rested = match navigation.scroll_retry {
        Some(at) => now.saturating_duration_since(at) >= SCROLL_RETRY_GAP,
        None => true,
    };
    if unwatched && rested {
        tracing::debug!(
            offset,
            "the tree scrolled while shown folders lack a watch; retrying their watches now"
        );
        navigation.scroll_retry = Some(now);
        navigation.watcher.retry_now();
    }
}

/// Recompute the shown directories from the visible rows and send them with the displayed file.
/// Collapsed folders drop out,
///  so their watches are removed;
///  descendants of collapsed folders are not shown.
pub(super) fn show(source: &Rc<RefCell<State>>, navigation: &mut Navigation) {
    // `to_path_buf` copies the canonical root; it is always shown, even before its first listing.
    let mut shown = vec![navigation.workspace.root().to_path_buf()];
    for row in &navigation.rows {
        if row.entry.is_directory && row.expanded {
            shown.push(row.entry.path.clone());
        }
    }
    // What: `collect` builds a set from the iterator of cloned paths; the type annotation picks `BTreeSet`.
    // Why: The watcher keys watches by path; the visible order stays in `shown` for scheduling.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const wanted = new Set(shown);
    // ```
    let wanted: BTreeSet<PathBuf> = shown.iter().cloned().collect();
    let file = source.borrow().file_path.clone();
    navigation.watcher.watch_only(&wanted, file.as_deref());
    navigation.shown = shown;
}

/// Apply notifications:
///  notified shown directories and the displayed file become due for a reread.
pub(super) fn update(source: &Rc<RefCell<State>>, navigation: &mut Navigation) {
    let changes = navigation.watcher.take();
    let now = Instant::now();
    // What: `if let Some(watched)` moves a newly reported live-watch set into navigation.
    // Why: Shown directories without a live watch keep the old 500 ms round robin.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (changes.watched !== undefined) navigation.watched = changes.watched;
    // ```
    if let Some(watched) = changes.watched {
        navigation.watched = watched;
    }
    if changes.everything {
        navigation.directories.changed_all(&navigation.shown);
    }
    for directory in changes.directories {
        // The displayed file's directory may be watched without being a shown tree folder.
        if navigation.shown.contains(&directory) {
            navigation.directories.changed(directory);
        }
    }
    let mut current = source.borrow_mut();
    if changes.everything {
        // Events may have been lost, so nothing says whether a write is in progress: reread once quiet.
        current.refresh.changed(SourceChange::Reread, now);
    }
    if let Some(change) = changes.source {
        current.refresh.changed(change, now);
    }
    // What: `as_deref` borrows the optional owned path as `Option<&Path>`; `and_then(Path::parent)` takes its directory.
    // Why: The source keeps the old 250 ms timer until its directory has a live watch.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const parent = filePath === undefined ? undefined : dirname(filePath);
    // ```
    let parent = current.file_path.as_deref().and_then(Path::parent);
    let watched = match parent {
        Some(directory) => navigation.watched.contains(directory),
        None => false,
    };
    let outside = current.outside_project;
    // A displayed file needs its folder watched; no displayed file, or one outside the project, needs nothing.
    let file_covered = watched || parent.is_none() || outside;
    current.refresh.set_watched(watched);
    current.refresh.set_outside_project(outside);
    drop(current);
    if navigation
        .directories
        .start_sweep_if_due(&navigation.shown, now)
    {
        // Retry failed watches once per sweep, and only while something shown lacks a watch,
        // so a fully watched window does not wake the watch thread every second.
        let mut shown_watched = true;
        for path in navigation.shown.iter() {
            if !navigation.watched.contains(path) {
                shown_watched = false;
                break;
            }
        }
        let all_watched = file_covered && shown_watched;
        if !all_watched {
            navigation.watcher.retry();
        }
    }
}
