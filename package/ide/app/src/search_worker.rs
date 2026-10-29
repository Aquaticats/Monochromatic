//! Latest-query search worker cancels and reaps old ripgrep children before running the newest request.

/// Search data remains immutable when published to the native thread.
use crate::{search::SearchResults, search_cancel::SearchCancellation, workspace::Workspace};
/// Startup and unexpected worker failures must not masquerade as no results.
use anyhow::{Context, Result};
/// Arc shares immutable requests/replies across threads;
///  the join handle owns shutdown.
use std::{
    path::PathBuf,
    sync::Arc,
    thread::{self, JoinHandle},
};
/// One retained request/reply avoids unbounded queues while async process I/O stays off the GUI thread.
use tokio::{
    runtime::{Builder, Runtime},
    sync::watch,
};

/// Per-request directory resolution remains on the reader thread.
mod request;

/// Public reply is tagged separately from displayed-file or document revisions.
#[derive(Debug)]
pub struct SearchReply {
    /// Latest-query identity assigned by the caller's worker handle.
    pub generation: u64,
    /// Original query,
    ///  retained for diagnostic and stale-result inspection.
    pub query: String,
    /// Directory actually searched,
    ///  or the requested scope named in a resolution failure.
    pub scope: PathBuf,
    /// Filename and content results stay independently usable.
    pub results: SearchResults,
}

/// Private request owns its query and cancellation signal for one background run.
struct Request {
    /// Request identity survives replacement in the watch slot.
    generation: u64,
    /// Query is passed as one subprocess argument,
    ///  never shell source.
    query: String,
    /// None searches the project root;
    ///  a selected subtree must resolve inside it before execution.
    scope: Option<PathBuf>,
    /// Shared one-way cancellation signal reaches both child streams.
    cancellation: SearchCancellation,
}

/// Native-owned handle;
///  no Slint handle or mutable document crosses this boundary.
pub struct SearchWorker {
    /// Dropping this sender closes the worker's request loop after active cancellation finishes.
    requests: Option<watch::Sender<Option<Arc<Request>>>>,
    /// At most one immutable latest reply is retained,
    ///  whether or not the UI has consumed it.
    replies: watch::Receiver<Option<Arc<SearchReply>>>,
    /// New requests invalidate old ones before publishing replacement work.
    cancellation: Option<SearchCancellation>,
    /// Monotonic query generation prevents old replies from entering the current result model.
    generation: u64,
    /// Worker thread is joined,
    ///  never detached,
    ///  when the owning window closes.
    thread: Option<JoinHandle<()>>,
}

/// Process each latest request directly;
///  intermediate queued queries are replaced rather than accumulated.
async fn run(
    workspace: Workspace,
    mut requests: watch::Receiver<Option<Arc<Request>>>,
    replies: watch::Sender<Option<Arc<SearchReply>>>,
) {
    loop {
        // What: clone releases the watch borrow before await while sharing the immutable request allocation.
        // Why: Holding a watch read lock during subprocess work would block UI publication of a newer query.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const request = requests.takeLatest();
        // if (request) await runSearch(request);
        // ```
        let current = requests.borrow_and_update().clone();
        if let Some(pending) = current
            && let Some(reply) = request::run(&workspace, &pending).await
        {
            replies.send_replace(Some(Arc::new(reply)));
        }
        if let Err(error) = requests.changed().await {
            tracing::debug!(%error, "project search request channel closed");
            return;
        }
    }
}

/// Move a validated runtime and the owned channel endpoints into a named background thread.
fn spawn(
    runtime: Runtime,
    workspace: Workspace,
    requests: watch::Receiver<Option<Arc<Request>>>,
    replies: watch::Sender<Option<Arc<SearchReply>>>,
) -> Result<JoinHandle<()>> {
    return thread::Builder::new()
        .name("ide-project-search".to_string())
        .spawn(move || {
            runtime.block_on(run(workspace, requests, replies));
        })
        .context("Cannot start the project search worker");
}

/// Search ownership couples latest-result filtering with cancellation and process cleanup.
impl SearchWorker {
    /// Start the worker without starting ripgrep or scanning the workspace.
    pub fn new(workspace: Workspace) -> Result<Self> {
        let runtime = Builder::new_current_thread()
            .enable_all()
            .build()
            .context("Cannot initialize project search I/O")?;
        let (request_sender, request_receiver) = watch::channel(None);
        let (reply_sender, reply_receiver) = watch::channel(None);
        let thread = spawn(runtime, workspace, request_receiver, reply_sender)?;
        return Ok(Self {
            requests: Some(request_sender),
            replies: reply_receiver,
            cancellation: None,
            generation: 0,
            thread: Some(thread),
        });
    }

    /// Clear pending work and invalidate unread replies without manufacturing an empty successful search.
    pub fn clear(&mut self) -> Result<()> {
        let generation = self
            .generation
            .checked_add(1)
            .context("Search request identity exhausted; restart the application")?;
        if let Some(cancellation) = self.cancellation.take() {
            cancellation.cancel();
        }
        self.generation = generation;
        self.requests
            .as_ref()
            .context("Project search worker is closed")?
            .send(None)
            .context("Project search worker stopped; restart the application")?;
        return Ok(());
    }

    /// Search the fixed project root without an additional selected-directory scope.
    pub fn request(&mut self, query: String) -> Result<u64> {
        return self.submit(query, None);
    }

    /// Narrow a query to a selected directory;
    ///  resolution and containment checks run on the worker.
    pub fn request_scoped(&mut self, query: String, scope: PathBuf) -> Result<u64> {
        return self.submit(query, Some(scope));
    }

    /// Cancel earlier work and publish only the newest desired query and scope together.
    fn submit(&mut self, query: String, scope: Option<PathBuf>) -> Result<u64> {
        self.clear()?;
        let cancellation = SearchCancellation::new();
        let request = Arc::new(Request {
            generation: self.generation,
            query,
            scope,
            cancellation: cancellation.clone(),
        });
        self.requests
            .as_ref()
            .context("Project search worker is closed")?
            .send(Some(request))
            .context("Cannot submit project search; restart the application")?;
        self.cancellation = Some(cancellation);
        tracing::debug!(
            generation = self.generation,
            "published latest project search request"
        );
        return Ok(self.generation);
    }

    /// Poll without waiting;
    ///  outdated replies are consumed but never exposed as current results.
    pub fn try_take(&mut self) -> Result<Option<Arc<SearchReply>>> {
        if !self
            .replies
            .has_changed()
            .context("Project search worker stopped unexpectedly; restart the application")?
        {
            return Ok(None);
        }
        let reply = self.replies.borrow_and_update().clone();
        if let Some(current) = reply {
            if current.generation == self.generation {
                return Ok(Some(current));
            }
            tracing::debug!(
                generation = current.generation,
                "discarded stale project search reply"
            );
        }
        return Ok(None);
    }
}

/// In-memory channel controls make stale-reply rejection independent of child execution timing.
#[cfg(test)]
#[path = "search_worker_tests.rs"]
mod tests;

/// Cancellation wakes async pipe reads even if ripgrep has produced no records.
impl Drop for SearchWorker {
    /// Stop child work before closing input and joining the worker thread.
    fn drop(&mut self) {
        if let Some(cancellation) = self.cancellation.take() {
            cancellation.cancel();
        }
        self.requests.take();
        let Some(worker) = self.thread.take() else {
            return;
        };
        if let Err(error) = worker.join() {
            tracing::error!(?error, "project search worker panicked during shutdown");
        }
    }
}
