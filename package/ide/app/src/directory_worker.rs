//! One bounded directory read at a time,
//!  with opaque tree-request identity carried across the thread.

/// Tree state remains on the caller thread;
///  only requests and owned snapshots cross the channel.
use crate::{
    file_tree::{DirectoryRequest, FileTree},
    git_ignore::{GlobalSource, IgnoreAnswer, IgnoreMatcher},
    workspace::{DirectoryEntry, Workspace},
};
/// Thread startup and unexpected disconnects remain actionable errors.
use anyhow::{Context, Result, bail};
/// What:
///  Bounded channels transfer messages;
///  JoinHandle owns worker shutdown;
///  Path borrows the requested name.
/// Why:
///  The native input thread never blocks on directory enumeration or a growing work queue.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reader = new Worker('directory-reader'); // one outstanding request, including unread replies
/// ```
use std::{
    path::Path,
    sync::mpsc::{Receiver, SyncSender, TryRecvError, sync_channel},
    thread::{self, JoinHandle},
};

/// Private reply preserves the exact request identity,
///  not merely equal directory path text.
struct Reply {
    /// Opaque token created by the UI-owned tree.
    request: DirectoryRequest,
    /// Ordered dirents or the original read diagnostic.
    result: Result<Vec<DirectoryEntry>>,
    /// Which of those dirents git ignores,
    ///  decided on the reader thread beside the read,
    ///  so the answer is as current as the listing it describes.
    ignores: IgnoreAnswer,
}

/// UI-owned background reader,
///  including at most one executing or unread reply.
pub struct DirectoryWorker {
    /// Optional sender permits closing input before joining during Drop.
    requests: Option<SyncSender<DirectoryRequest>>,
    /// Replies are polled without blocking the UI thread.
    replies: Receiver<Reply>,
    /// Outstanding work includes a completed response until it is consumed.
    busy: bool,
    /// Thread is joined on shutdown,
    ///  never detached.
    thread: Option<JoinHandle<()>>,
}

/// Perform only read-only workspace operations on the background thread.
fn run(
    workspace: Workspace,
    mut matcher: IgnoreMatcher,
    requests: Receiver<DirectoryRequest>,
    replies: SyncSender<Reply>,
) {
    loop {
        // What: match extracts a received request or the expected shutdown-channel error.
        // Why: Closing the owning reader ends this loop without a bare shutdown error.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const request = await receive(); if (closed) return;
        // ```
        let request = match requests.recv() {
            Ok(request) => request,
            Err(error) => {
                tracing::debug!(%error, "directory reader request channel closed");
                return;
            }
        };
        tracing::debug!(path = %request.path().display(), "reading requested directory");
        let result = workspace.list(request.path());
        // A failed read has no entries to judge; the tree keeps its last snapshot and answer.
        let ignores = match &result {
            Ok(entries) => matcher.evaluate(request.path(), entries),
            Err(error) => {
                tracing::debug!(path = %request.path().display(), %error, "no ignore answer for a directory that could not be read");
                IgnoreAnswer::default()
            }
        };
        // What: if-let extracts a send failure when the caller no longer owns the reply receiver.
        // Why: A closed consumer ends work rather than retrying an abandoned request.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // try { send({ request, result }); } catch (error) { log(error); return; }
        // ```
        if let Err(error) = replies.send(Reply {
            request,
            result,
            ignores,
        }) {
            tracing::debug!(%error, "directory reader reply receiver closed");
            return;
        }
    }
}

/// Queue admission and request creation are one operation so a busy reader cannot orphan a tree token.
impl DirectoryWorker {
    /// Start a named worker bound to one canonical read-only workspace.
    pub fn new(workspace: Workspace) -> Result<Self> {
        return Self::with_global_ignores(workspace, GlobalSource::GitConfig);
    }

    /// Start a reader whose ignored entries come from the project's rule files and the given global excludes file;
    ///  tests name a file so they never read the real home folder.
    pub fn with_global_ignores(workspace: Workspace, global: GlobalSource) -> Result<Self> {
        // What: sync_channel(1) creates sender/receiver endpoints with one message slot.
        // Why: A caller cannot accumulate unbounded directory snapshots while the UI is occupied.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const requests = boundedChannel(1); const replies = boundedChannel(1);
        // ```
        let (request_sender, request_receiver) = sync_channel(1);
        let (reply_sender, reply_receiver) = sync_channel(1);
        // What: move transfers the workspace and channel endpoints into the named OS thread; ? propagates startup failure.
        // Why: No toolkit handle or mutable tree state crosses the worker boundary.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const worker = startWorker(() => run(workspace, requests, replies));
        // ```
        let worker = thread::Builder::new()
            .name("ide-directory-reader".to_string())
            .spawn(move || {
                run(
                    workspace,
                    IgnoreMatcher::new(global),
                    request_receiver,
                    reply_sender,
                );
            })
            .context("Cannot start the directory reader")?;
        // What: Some stores owned endpoints; Ok returns the successfully started reader.
        // Why: Drop later takes each owner exactly once to close and join the thread in order.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { requests: requestSender, replies: replyReceiver, busy: false, thread: worker };
        // ```
        return Ok(Self {
            requests: Some(request_sender),
            replies: reply_receiver,
            busy: false,
            thread: Some(worker),
        });
    }

    /// Return false while busy,
    ///  without creating or superseding any tree request.
    pub fn request(&mut self, tree: &mut FileTree, directory: &Path) -> Result<bool> {
        if self.busy {
            return Ok(false);
        }
        // Borrow the sender before changing tree state; a closed reader cannot leave a pending token.
        let sender = self
            .requests
            .as_ref()
            .context("Directory reader is closed")?;
        let request = tree.begin_listing(directory)?;
        // What: clone shares the request identity while retaining a local owner for send-failure cleanup.
        // Why: A rejected send releases its tree slot rather than leaving an indefinitely loading directory.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // try { send(request); } catch (error) { tree.complete(request, failure(error)); }
        // ```
        if let Err(error) = sender.try_send(request.clone()) {
            tracing::error!(%error, path = %directory.display(), "cannot send directory request");
            return tree.complete_listing(
                &request,
                Err(anyhow::anyhow!(
                    "Cannot send a read request for {}: {error}; restart the application",
                    directory.display()
                )),
            );
        }
        self.busy = true;
        return Ok(true);
    }

    /// Apply a ready current reply;
    ///  false means no ready change or a discarded stale reply.
    pub fn poll(&mut self, tree: &mut FileTree) -> Result<bool> {
        // Extract a ready reply or distinguish an empty queue from an unexpected worker disconnect.
        match self.replies.try_recv() {
            Ok(reply) => {
                self.busy = false;
                return tree.complete_listing_with(&reply.request, reply.result, reply.ignores);
            }
            Err(TryRecvError::Empty) => {
                return Ok(false);
            }
            Err(TryRecvError::Disconnected) => {
                bail!(
                    "Directory reader stopped unexpectedly; restart the application to read project directories"
                );
            }
        }
    }

    /// An unread reply still occupies the reader even when its filesystem operation has finished.
    pub fn is_busy(&self) -> bool {
        return self.busy;
    }
}

/// Close input before joining,
///  allowing the sole outstanding reply to fit without blocking shutdown.
impl Drop for DirectoryWorker {
    /// Join the owned thread and log unexpected panics rather than detaching background work.
    fn drop(&mut self) {
        // Taking the sender drops its channel endpoint at the statement boundary.
        self.requests.take();
        // Extract the optional join owner; absence means there is no thread left to stop.
        let Some(worker) = self.thread.take() else {
            return;
        };
        if let Err(error) = worker.join() {
            tracing::error!(?error, "directory reader panicked during shutdown");
        }
    }
}
