//! Send gathered file changes to the servers that asked for them,
//!  as `workspace/didChangeWatchedFiles`,
//! and ask those servers for the displayed file's diagnostics again,
//!  since another file may have changed them.

/// Pulling diagnostics again after a server heard about changes.
use super::request;
/// The spelling servers use for paths below the project root.
use super::root::RootView;
/// The records of the servers,
///  and what asking them whether one still runs needs.
use super::session::Session;
/// What the registrations ask of the sender of a burst.
use super::watched_files::WatchHost;
/// The worker whose registrations and timers these steps use.
use super::worker::{Internal, Worker};
/// The changes the change watcher forwards.
use crate::change_watch::ServerChange;
/// The identifier helix-lsp gives each server.
use helix_lsp::LanguageServerId;
/// What:
///  `Path`/`PathBuf` are a borrowed and an owned path;
///  `Instant` is a monotonic time point.
/// Why:
///  Changes carry resolved paths and are sent after a quiet period.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const now = performance.now();
/// ```
use std::{
    path::{Path, PathBuf},
    time::Instant,
};

/// What:
///  Gather one change and,
///  for the first of a burst,
///  schedule sending.
/// Why:
///  Bursts become one notification per server;
///  the interface thread is never involved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function receive(worker: Worker, change: ServerChange): void
/// ```
pub(super) fn receive(worker: &mut Worker, change: ServerChange) {
    let now = Instant::now();
    if worker.watched.record(change, now) && !worker.forward_scheduled {
        worker.forward_scheduled = true;
        worker.timer(super::watched_files::FORWARD_QUIET, Internal::ForwardFiles);
    }
}

/// What:
///  The scheduled time came:
///  send the burst if it is due,
///  or wait for the rest of it.
/// Why:
///  A burst still being written is sent once it pauses,
///  or at the latest after its limit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function due(worker: Worker): void
/// ```
pub(super) fn due(worker: &mut Worker) {
    worker.forward_scheduled = false;
    // What: `let Some(wait) = ... else` returns when nothing is pending.
    // Why: A burst taken by an earlier send leaves nothing to do.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const wait = worker.watched.wait(now); if (wait === undefined) return;
    // ```
    let Some(wait) = worker.watched.wait(Instant::now()) else {
        return;
    };
    if !wait.is_zero() {
        worker.forward_scheduled = true;
        worker.timer(wait, Internal::ForwardFiles);
        return;
    }
    send(worker);
}

/// What:
///  The worker's answers for one send:
///  how a path is spelled for the servers,
///  and whether a server still has a running client.
/// Why:
///  The registrations ask these through the `WatchHost` trait,
///  so the answers come from a named type instead of anonymous functions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const host = { respell: (path) => view?.toHelix(path) ?? path, live: (server) => session.client(server) !== undefined };
/// ```
struct WorkerHost<'a> {
    /// The spelling of the root,
    ///  when one was found.
    view: &'a Option<RootView>,
    /// The servers' records.
    session: &'a Session,
}

/// The worker's answers.
impl WatchHost<LanguageServerId> for WorkerHost<'_> {
    /// Respell with the view when there is one.
    fn respell(&self, path: &Path) -> PathBuf {
        return match self.view {
            Some(spelling) => spelling.to_helix(path),
            None => path.to_path_buf(),
        };
    }

    /// A server is live while its record holds a client.
    fn live(&self, server: LanguageServerId) -> bool {
        if let Some(index) = self.session.index_of(server) {
            return self.session.servers[index].client.is_some();
        }
        return false;
    }
}

/// Send every server its share of the burst,
///  then pull the displayed file's diagnostics from each one
/// that holds it open.
fn send(worker: &mut Worker) {
    // What: `match` on the root view: a view respells resolved paths the way servers spell the root.
    // Why: Servers compare addresses textually; a path in another spelling would be another file to them.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const view = RootView.discover(root); const respell = view ? view.toHelix : (path) => path;
    // ```
    let view = match RootView::discover(&worker.root) {
        Ok(view) => Some(view),
        Err(refusal) => {
            tracing::debug!(?refusal, "sending file changes in their resolved spelling");
            None
        }
    };
    let root = match view.as_ref() {
        Some(spelling) => spelling.helix().to_path_buf(),
        None => worker.root.clone(),
    };
    let host = WorkerHost {
        view: &view,
        session: &worker.session,
    };
    let batches = worker.watched.take(&host, &root);
    for (server, events) in batches {
        let Some(index) = worker.session.index_of(server) else {
            continue;
        };
        let record = &worker.session.servers[index];
        // Copied now: the record is borrowed from the worker, which the pull below needs whole.
        let opened = record.opened;
        // A server still starting cannot take notifications; its share of this burst is dropped.
        let Some(client) = record.ready() else {
            tracing::debug!(%server, events = events.len(), "dropped file changes for a language server that is still starting");
            continue;
        };
        tracing::debug!(
            server = client.name(),
            events = events.len(),
            "sent file changes to a language server"
        );
        client.did_change_watched_files(events);
        if opened {
            request::pull_diagnostics(worker, index, 0);
        }
    }
}
