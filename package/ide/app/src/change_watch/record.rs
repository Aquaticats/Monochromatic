//! Turn notify events into pending invalidations;
//!  this runs on notify's own event-loop thread.

/// Shared invalidation state,
///  the changes sent to the language servers,
///  and the poison-tolerant lock.
use super::shared::{ServerChange, ServerChangeKind, Shared, SourceChange, lock};
/// What:
///  notify's event types:
///  `Event` holds a kind plus affected paths;
///  the `*Kind` enums classify it.
/// Why:
///  Classification decides whether a directory listing,
///  the displayed file,
///  or nothing is stale.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Event, EventKind, AccessKind, AccessMode, ModifyKind } from 'notify';
/// ```
use notify::{
    Event, EventKind,
    event::{AccessKind, AccessMode, CreateKind, ModifyKind, RenameMode},
};
/// What:
///  `Mutex` guards the shared state;
///  `SyncSender` is the sending half of a bounded channel.
/// Why:
///  The handler must never block notify's thread;
///  it only locks briefly and sends a non-blocking wake.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Mutex, BoundedSender } from 'threads';
/// ```
use std::{
    path::Path,
    sync::{Mutex, mpsc::SyncSender, mpsc::TrySendError},
};

/// What one event means for the window.
///
/// What:
///  an `enum` whose `Content` and `Entries` variants each carry a `SourceChange` payload,
///       like a TS tagged union `{ kind: 'content', change } | { kind: 'entries', change } | { kind: 'ignore' }`.
/// Why:
///  Entry changes (create,
///  remove,
///  rename) also make the parent directory's listing stale;
///      content changes only matter when the path is the displayed file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Reaction {
    /// Opens and read-only closes,
    ///  including the IDE's own reads;
    ///  reacting would reread forever.
    Ignore,
    /// File content or permissions changed;
    ///  the parent listing is unaffected.
    Content(
        /// How finished the change looks when the path is the displayed file.
        SourceChange,
    ),
    /// A name appeared,
    ///  disappeared,
    ///  or moved;
    ///  the parent listing is stale too.
    Entries(
        /// How finished the change looks when the path is the displayed file.
        SourceChange,
    ),
}

/// Classify one event kind;
///  unknown kinds are treated as entry changes so nothing is missed.
fn classify(kind: &EventKind) -> Reaction {
    // What: `match` compares `kind` against each pattern in order; `_` matches anything left.
    //       `EventKind::Access(AccessKind::Close(AccessMode::Write))` is a nested pattern, like checking
    //       `kind.type === 'access' && kind.access.type === 'close' && kind.access.mode === 'write'`.
    // Why: Each arm returns the reaction for one family of inotify events.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (kind.type) { case 'access': return kind.isCloseWrite ? content('settled') : 'ignore'; ... }
    // ```
    match kind {
        EventKind::Access(AccessKind::Close(AccessMode::Write)) => {
            return Reaction::Content(SourceChange::Settled);
        }
        EventKind::Access(_) => {
            return Reaction::Ignore;
        }
        EventKind::Create(_) => {
            return Reaction::Entries(SourceChange::Unsettled);
        }
        EventKind::Modify(ModifyKind::Data(_)) => {
            return Reaction::Content(SourceChange::Unsettled);
        }
        EventKind::Modify(ModifyKind::Metadata(_)) => {
            return Reaction::Content(SourceChange::Settled);
        }
        _ => {
            return Reaction::Entries(SourceChange::Settled);
        }
    }
}

/// What an event means for the language servers,
///  or `None` for events they never hear about.
fn server_kind(kind: &EventKind) -> Option<ServerChangeKind> {
    // What: `match` on nested event kinds; `RenameMode::From` and `RenameMode::To` are the two halves of a move.
    // Why: The protocol knows only created, changed, and deleted; a move is a deletion plus a creation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (kind.type) { case 'create': return 'created'; case 'remove': return 'deleted'; ... }
    // ```
    match kind {
        EventKind::Access(AccessKind::Close(AccessMode::Write)) => {
            return Some(ServerChangeKind::Changed);
        }
        EventKind::Access(_) => {
            return None;
        }
        EventKind::Create(_) | EventKind::Modify(ModifyKind::Name(RenameMode::To)) => {
            return Some(ServerChangeKind::Created);
        }
        EventKind::Modify(ModifyKind::Name(RenameMode::From)) | EventKind::Remove(_) => {
            return Some(ServerChangeKind::Deleted);
        }
        // notify also reports each half of a rename on its own, so this event would repeat them.
        EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
            return None;
        }
        _ => {
            return Some(ServerChangeKind::Changed);
        }
    }
}

/// True for an event that may have made a new folder appear:
///  a folder creation,
///  a creation of unknown
/// kind,
///  or a move into place,
///  which inotify does not mark as a file or a folder.
fn may_add_folder(kind: &EventKind) -> bool {
    return matches!(
        kind,
        EventKind::Create(CreateKind::Folder | CreateKind::Any | CreateKind::Other)
            | EventKind::Modify(ModifyKind::Name(RenameMode::To | RenameMode::Any))
    );
}

/// Forward one event's path to the language servers when it lies in a folder watched for them,
///  and note
/// what it does to the folder structure;
///  true when the watch thread must look at that structure.
fn record_for_servers(guard: &mut Shared, kind: &EventKind, path: &Path) -> bool {
    // What: `let Some(feed) = ... else` binds the queue into the language worker, or returns when there is none.
    // Why: Nothing is watched or sent for the servers while none registered file watchers.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const feed = shared.server.feed; if (feed === undefined) return false;
    // ```
    let Some(feed) = guard.server.feed.as_ref() else {
        return false;
    };
    let parent = path.parent();
    let watched = &guard.server.watched;
    let mut inside = watched.contains(path);
    if !inside && let Some(folder) = parent {
        inside = watched.contains(folder);
    }
    if !inside {
        return false;
    }
    let Some(change) = server_kind(kind) else {
        return false;
    };
    // A change inside a folder that was empty when scanned is not sent: the folder may be ignored.
    // Its parent is scanned again, and the files that scan lists in it are sent as created.
    if let Some(folder) = parent
        && guard.server.provisional.contains(folder)
    {
        if let Some(base) = folder.parent() {
            guard.server.requests.rescans.insert(base.to_path_buf());
        }
        return true;
    }
    let message = ServerChange {
        path: path.to_path_buf(),
        kind: change,
    };
    // What: `send` queues without waiting and fails only when the language worker is gone.
    // Why: notify's thread must never block on the worker.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!feed.offer(message)) log.debug('language worker stopped');
    // ```
    if let Err(error) = feed.send(message) {
        tracing::debug!(%error, "the language worker stopped; a file change was not forwarded");
    }
    let mut structure = false;
    if may_add_folder(kind) {
        guard.server.requests.candidates.insert(path.to_path_buf());
        structure = true;
    }
    if change == ServerChangeKind::Deleted && guard.server.watched.contains(path) {
        guard.server.requests.gone.insert(path.to_path_buf());
        structure = true;
    }
    return structure;
}

/// True when the optional path ends in the file name `name`.
fn named(candidate: Option<&Path>, name: &str) -> bool {
    if let Some(found) = candidate.and_then(Path::file_name) {
        return found == name;
    }
    return false;
}

/// What:
///  True for the files whose content decides which entries git ignores:
///  a `.gitignore`,
///  or `info/exclude` inside a `.git` folder.
/// Why:
///  A folder's listing does not depend on any file's content except these,
///  because the tree dims the entries they name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isIgnoreRuleFile(path: string): boolean;
/// ```
fn is_ignore_rule_file(path: &Path) -> bool {
    if named(Some(path), ".gitignore") {
        return true;
    }
    let info = path.parent();
    return named(Some(path), "exclude")
        && named(info, "info")
        && named(info.and_then(Path::parent), ".git");
}

/// Wake the watch thread without blocking;
///  a full channel already holds a pending wake.
pub(super) fn wake(sender: &SyncSender<()>) {
    // What: `if let Err(TrySendError::Disconnected(()))` matches only the "receiver is gone" failure;
    //       `TrySendError::Full` (a wake is already queued) needs nothing.
    // Why: The thread re-reads all shared state on each wake, so one queued wake covers any number of changes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!sender.trySend() && sender.closed) log('watch thread has stopped');
    // ```
    if let Err(TrySendError::Disconnected(())) = sender.try_send(()) {
        tracing::debug!("change-watch thread has stopped; wake dropped");
    }
}

/// Record a notify error:
///  log it,
///  mark its paths' watches stale,
///  and request a full reread.
fn record_error(shared: &Mutex<Shared>, sender: &SyncSender<()>, error: &notify::Error) {
    tracing::warn!(%error, paths = ?error.paths, "file-change notification error; rereading everything shown");
    let mut guard = lock(shared);
    guard.pending.everything = true;
    let mut lost = false;
    for path in &error.paths {
        if guard.watched.contains(path) {
            // `clone` copies the borrowed path into the owned stale set.
            guard.stale.insert(path.clone());
            lost = true;
        }
    }
    drop(guard);
    if lost {
        wake(sender);
    }
}

/// Record one event,
///  or one error,
///  from notify as invalidations;
///  never touches the filesystem.
pub(super) fn record(
    shared: &Mutex<Shared>,
    sender: &SyncSender<()>,
    received: notify::Result<Event>,
) {
    // What: `match` on the `Result`: `Ok(event)` binds the event, `Err(error)` handles the failure and returns.
    // Why: Errors take a separate path that always rereads everything shown.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (received instanceof Error) { recordError(received); return; }
    // ```
    let event = match received {
        Ok(event) => event,
        Err(error) => {
            record_error(shared, sender, &error);
            return;
        }
    };
    if event.need_rescan() {
        tracing::warn!(
            "file-change queue overflowed (inotify IN_Q_OVERFLOW); rereading everything shown"
        );
        lock(shared).pending.everything = true;
        return;
    }
    let reaction = classify(&event.kind);
    // A close after writing is `Content` for the window too, so only reads end here.
    if reaction == Reaction::Ignore {
        return;
    }
    if event.paths.is_empty() {
        tracing::warn!(kind = ?event.kind, "file-change event without a path; rereading everything shown");
        lock(shared).pending.everything = true;
        return;
    }
    let mut guard = lock(shared);
    let mut wake_thread = false;
    for path in &event.paths {
        // Watched paths are canonical and inside the root, but a moved watch can report the old name.
        if !path.starts_with(&guard.root) {
            tracing::debug!(path = %path.display(), "ignored a change outside the project root");
            continue;
        }
        // The servers hear about every change in their folders, the IDE's own reads excepted.
        if record_for_servers(&mut guard, &event.kind, path) {
            wake_thread = true;
        }
        // What: `if let Reaction::Entries(_) = reaction` tests the variant without using its payload.
        // Why: Only entry changes alter listings or can remove a watched directory itself.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (reaction.kind === 'entries') { ... }
        // ```
        if let Reaction::Entries(_) = reaction {
            if guard.watched.contains(path) {
                // inotify drops a removed directory's watch, and a renamed one keeps following the moved inode.
                tracing::warn!(path = %path.display(), "watched directory was removed or renamed; rereading everything shown");
                guard.stale.insert(path.clone());
                guard.pending.everything = true;
                wake_thread = true;
            }
            // What: `if let Some(parent) = path.parent()` extracts the parent path when one exists.
            // Why: The parent's listing gains or loses this name; only watched parents are shown.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const parent = dirname(path); if (watched.has(parent)) pending.directories.add(parent);
            // ```
            if let Some(parent) = path.parent()
                && guard.watched.contains(parent)
            {
                // `to_path_buf` copies the borrowed parent into an owned set entry.
                let owned = parent.to_path_buf();
                guard.pending.directories.insert(owned);
            }
        }
        // A permission change on a watched folder decides whether it can still be listed.
        if let Reaction::Content(_) = reaction
            && guard.watched.contains(path)
        {
            guard.pending.directories.insert(path.clone());
        }
        // A rule file edited in place leaves every name as it was, yet changes which entries are ignored;
        // its folder is stale like after a name change, and the reread decides the rows again.
        if let Reaction::Content(_) = reaction
            && is_ignore_rule_file(path)
            && let Some(parent) = path.parent()
            && guard.watched.contains(parent)
        {
            tracing::debug!(path = %path.display(), "an ignore rule file changed; its folder is stale");
            guard.pending.directories.insert(parent.to_path_buf());
        }
        // What: `as_deref` turns `Option<PathBuf>` into `Option<&Path>` so it compares with the borrowed `path`.
        // Why: Only the displayed file's own events refresh the source; the latest event wins.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (shared.file === path) pending.source = change;
        // ```
        if guard.file.as_deref() == Some(path.as_path()) {
            let change = match reaction {
                Reaction::Content(change) | Reaction::Entries(change) => change,
                Reaction::Ignore => {
                    continue;
                }
            };
            guard.pending.source = Some(change);
        }
    }
    drop(guard);
    if wake_thread {
        wake(sender);
    }
}
