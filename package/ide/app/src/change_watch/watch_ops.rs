//! Add and remove one inotify watch,
//!  and describe why adding one failed.
//! Only the watch thread calls these,
//!  because adding a watch blocks until notify's loop replies.

/// Containment uses the same canonical check as every project read.
use crate::workspace::Workspace;
/// What:
///  notify's inotify backend,
///  its error kinds,
///  the non-recursive mode,
///  and the `Watcher` trait
///       whose methods (`watch`,
///  `unwatch`) the backend implements.
/// Why:
///  Naming `INotifyWatcher` (not `RecommendedWatcher`) keeps a polling backend from ever being chosen.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { INotifyWatcher, ErrorKind, RecursiveMode } from 'notify';
/// ```
use notify::{ErrorKind, INotifyWatcher, RecursiveMode, Watcher};
/// Borrowed native paths name the affected directory.
use std::path::Path;

/// Why adding one watch failed.
///
/// What:
///  an `enum` with a payload-free `Limit` variant and an `Other` variant carrying a message,
///       like a TS union `{ kind: 'limit' } | { kind: 'other'; message: string }`.
/// Why:
///  The watch limit is one state shared by every directory,
///  logged once and retried with backoff;
///      every other failure belongs to its own directory and keeps its own message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WatchFailure = { kind: 'limit' } | { kind: 'other'; message: string };
/// ```
pub(super) enum WatchFailure {
    /// `inotify_add_watch` answered `ENOSPC`:
    ///  this user's inotify watch limit is reached.
    Limit,
    /// Refused (outside the root,
    ///  a symbolic-link alias) or failed for another reason.
    Other(
        /// The failure,
        ///  naming the directory.
        String,
    ),
}

/// Watch one directory non-recursively after checking that it is its own canonical path inside the root.
pub(super) fn add(
    watcher: &mut INotifyWatcher,
    workspace: &Workspace,
    path: &Path,
) -> Result<(), WatchFailure> {
    // What: `match` on the resolve `Result`: `Ok(resolved)` is the canonical path, `Err(error)` a refusal.
    // Why: inotify follows symbolic links, so a symlinked alias could otherwise watch outside the project.
    // Gotcha: A swap between this check and the watch is not prevented; reads have the same window.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let resolved; try { resolved = workspace.resolve(path); } catch (error) { return { kind: 'other', message: String(error) }; }
    // ```
    let resolved = match workspace.resolve(path) {
        Ok(resolved) => resolved,
        Err(error) => {
            return Err(WatchFailure::Other(format!("{error:#}")));
        }
    };
    if resolved != path {
        return Err(WatchFailure::Other(format!(
            "Not watching {}: it resolves to {} through a symbolic link",
            path.display(),
            resolved.display()
        )));
    }
    // What: `if let Err(error) = ...` runs the block only when `watch` failed.
    // Why: The watch limit becomes its own failure; anything else is described with its directory.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { watcher.watch(path, 'non-recursive'); } catch (error) { return classify(error); }
    // ```
    if let Err(error) = watcher.watch(path, RecursiveMode::NonRecursive) {
        if let ErrorKind::MaxFilesWatch = error.kind {
            return Err(WatchFailure::Limit);
        }
        return Err(WatchFailure::Other(describe(&error, path)));
    }
    return Ok(());
}

/// Name the directory;
///  the watch limit (`ENOSPC` from `inotify_add_watch`) gets the sysctl that sets it.
pub(super) fn describe(error: &notify::Error, path: &Path) -> String {
    if let ErrorKind::MaxFilesWatch = error.kind {
        return format!(
            "Cannot watch {}: the inotify watch limit (fs.inotify.max_user_watches) is reached",
            path.display()
        );
    }
    return format!("Cannot watch {}: {error}", path.display());
}

/// Failure descriptions without exhausting the host's inotify watches.
#[cfg(test)]
#[path = "watch_ops_tests.rs"]
mod tests;

/// Remove one watch;
///  a watch the kernel already dropped (removed directory) is expected and only logged.
pub(super) fn remove(watcher: &mut INotifyWatcher, path: &Path) {
    if let Err(error) = watcher.unwatch(path) {
        tracing::debug!(path = %path.display(), %error, "watch was already gone");
    }
}
