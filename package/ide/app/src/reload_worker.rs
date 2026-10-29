//! One bounded source-read/diff job at a time,
//!  outside the native event loop.

/// Snapshots share immutable rope chunks;
///  replies carry only prepared changes.
use crate::{
    document::{Document, Reload},
    source_style::SourceStyles,
    syntax::SyntaxEngine,
    workspace::Workspace,
};
/// Preserve worker-start,
///  request,
///  and unexpected-disconnect diagnostics.
use anyhow::{Context, Result, bail};
/// What:
///  Channels transfer owned messages between threads;
///  capacity one bounds queued work.
/// Why:
///  UI input remains independent of filesystem reads and Helix diff computation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const worker = new Worker('source-reader'); // messages are owned snapshots
/// ```
use std::{
    path::PathBuf,
    sync::mpsc::{Receiver, SyncSender, TryRecvError, sync_channel},
    thread::{self, JoinHandle},
    time::Duration,
};

/// A file generation prevents applying an old file's result after navigation.
pub struct ReloadRequest {
    /// Source target supplied by the UI,
    ///  never inferred from a background result.
    pub path: PathBuf,
    /// Displayed base revision used by correspondence.
    pub snapshot: Document,
    /// Monotonic file-open identity,
    ///  separate from each document's revision.
    pub generation: u64,
    /// Request initial/retried classification even without a text change;
    ///  changed source always classifies.
    pub highlight_unchanged: bool,
    /// What:
    ///  `Option<Duration>` is a time span or nothing (`number | undefined`).
    /// Why:
    ///  `Some(quiet)` accepts the bytes only when the file was last modified at least `quiet` before
    ///      the read began;
    ///  a read no change notification asked for sets it,
    ///  because such a read can meet a
    ///      save in progress.
    ///  `None` accepts any read:
    ///  a notification already waited for the writer.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// requireQuiet?: number;
    /// ```
    pub require_quiet: Option<Duration>,
}

/// Classifications identify the exact source revision they describe.
pub struct SyntaxReply {
    /// Expected revision after accepting the associated reload,
    ///  or the unchanged revision.
    pub revision: u64,
    /// None means unrecognized plain text;
    ///  missing parser assets remain an error.
    pub result: Result<Option<SourceStyles>>,
}

/// A successful read can report unchanged text without creating a revision.
pub struct ReloadReply {
    /// File-open generation that requested this work.
    pub generation: u64,
    /// Resolved project-open target or the existing refresh target;
    ///  absent after resolution failure.
    pub resolved_path: Option<PathBuf>,
    /// Prepared change or unchanged result;
    ///  failure leaves displayed source intact.
    pub result: Result<Option<Reload>>,
    /// Present for changed source and explicitly requested unchanged-source classification.
    pub syntax: Option<SyntaxReply>,
    /// The request required quiet and the file was written too recently:
    ///  `result` is `Ok(None)`,
    ///  nothing
    /// is applied,
    ///  and the caller treats the file as being written,
    ///  as for an unfinished-write notification.
    pub recent_write: bool,
}

/// Queue envelope keeps project resolution on the worker without changing existing reload requests.
struct QueuedRead {
    /// Source snapshot and file-open identity.
    request: ReloadRequest,
    /// Present only for a new project-relative open;
    ///  ordinary refreshes retain their accepted path.
    workspace: Option<Workspace>,
}

/// UI-owned worker handle with at most one requested or unread reply.
pub struct ReloadWorker {
    /// Option permits closing the request channel before joining during Drop.
    requests: Option<SyncSender<QueuedRead>>,
    /// The UI polls this receiver without blocking.
    replies: Receiver<ReloadReply>,
    /// Includes both executing work and an unread response.
    busy: bool,
    /// Joined on shutdown after the request channel closes.
    thread: Option<JoinHandle<()>>,
}

/// Own the blocking receive loop entirely outside UI state.
fn run(requests: Receiver<QueuedRead>, replies: SyncSender<ReloadReply>) {
    let syntax = SyntaxEngine::new();
    loop {
        // What: match extracts either an owned job or the expected closed-channel condition.
        // Why: Shutdown must end the worker without an unhandled channel error.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const request = await receive(); if (closed) return;
        // ```
        let request = match requests.recv() {
            Ok(request) => request,
            Err(error) => {
                tracing::debug!(%error, "source reload worker request channel closed");
                return;
            }
        };
        // Resolve new project opens and classify source using the same reusable syntax engine.
        let reply = crate::reload_read::prepare(request.request, request.workspace, &syntax);
        if let Err(error) = replies.send(reply) {
            tracing::debug!(%error, "source reload worker reply receiver closed");
            return;
        }
    }
}

/// Manage bounded requests and non-blocking UI consumption of background results.
impl ReloadWorker {
    /// Start a named worker with bounded request and response channels.
    pub fn new() -> Result<Self> {
        // What: sync_channel(1) creates sender/receiver endpoints with one message slot.
        // Why: Rapid UI callbacks cannot accumulate copies of document snapshots.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const requests = boundedChannel(1); const replies = boundedChannel(1);
        // ```
        let (request_sender, request_receiver) = sync_channel(1);
        let (reply_sender, reply_receiver) = sync_channel(1);
        // What: move transfers the receiving and sending endpoints into a named OS thread.
        // Why: No Rc/RefCell UI state or Slint handle crosses threads.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const worker = startWorker(() => run(requestReceiver, replySender));
        // ```
        let worker = thread::Builder::new()
            .name("ide-source-reload".to_string())
            .spawn(move || {
                run(request_receiver, reply_sender);
            })
            .context("Cannot start the source reload worker")?;
        return Ok(Self {
            requests: Some(request_sender),
            replies: reply_receiver,
            busy: false,
            thread: Some(worker),
        });
    }

    /// Refresh an already accepted source path;
    ///  false means the worker still owns earlier work.
    pub fn request(&mut self, request: ReloadRequest) -> Result<bool> {
        // None retains the existing accepted target rather than adding another project-path lookup.
        return self.send(QueuedRead {
            request,
            workspace: None,
        });
    }

    /// Resolve a new tree/search open inside its explicit workspace on the background thread.
    pub fn request_project(
        &mut self,
        workspace: Workspace,
        request: ReloadRequest,
    ) -> Result<bool> {
        // Some transfers the read-only boundary with this request; no path resolution runs on the UI thread.
        return self.send(QueuedRead {
            request,
            workspace: Some(workspace),
        });
    }

    /// Admit at most one executing job or unread response regardless of the source-read mode.
    fn send(&mut self, request: QueuedRead) -> Result<bool> {
        if self.busy {
            return Ok(false);
        }
        let sender = self
            .requests
            .as_ref()
            .context("Source reload worker is closed")?;
        sender
            .try_send(request)
            .context("Cannot send a source reload request")?;
        self.busy = true;
        return Ok(true);
    }

    /// An unread response still occupies the reader even after its filesystem work has finished.
    pub fn is_busy(&self) -> bool {
        return self.busy;
    }

    /// Poll without blocking the native event loop;
    ///  None means no reply yet.
    pub fn try_take(&mut self) -> Result<Option<ReloadReply>> {
        match self.replies.try_recv() {
            Ok(reply) => {
                self.busy = false;
                return Ok(Some(reply));
            }
            Err(TryRecvError::Empty) => {
                return Ok(None);
            }
            Err(TryRecvError::Disconnected) => {
                // Release admission state before reporting terminal transport failure to the UI.
                self.busy = false;
                self.requests.take();
                bail!(
                    "Source reader stopped unexpectedly; restart the application to resume reading and refreshing files"
                );
            }
        }
    }
}

/// Channel-disconnect fixtures exercise stopped-reader admission and polling state.
#[cfg(test)]
#[path = "reload_worker_tests.rs"]
mod tests;

/// Close input before joining,
///  allowing the last bounded reply to complete.
impl Drop for ReloadWorker {
    /// End the owned thread without leaving a blocked reply sender behind.
    fn drop(&mut self) {
        // Taking the sender drops its owned endpoint at the end of this statement.
        self.requests.take();
        let Some(worker) = self.thread.take() else {
            return;
        };
        if let Err(error) = worker.join() {
            tracing::error!(?error, "source reload worker panicked during shutdown");
        }
    }
}
