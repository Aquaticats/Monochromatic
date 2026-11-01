//! One in-file find job at a time;
//!  the newest request replaces any waiting one.

/// The worker calls the same matching function the reference test pins.
use crate::find::{FindMatches, find_in_source};
/// Worker-start and unexpected-disconnect failures stay visible diagnostics.
use anyhow::{Context, Result, bail};
/// Rope clones share immutable text chunks,
///  so a request does not copy the document.
use helix_core::Rope;
/// What:
///  Channels move owned messages between threads;
///  capacity one bounds queued work.
/// Why:
///  Typing stays independent of a scan over a large file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const worker = new Worker('in-file-find'); // messages are owned snapshots
/// ```
use std::{
    sync::mpsc::{Receiver, SyncSender, TryRecvError, sync_channel},
    thread::{self, JoinHandle},
};

/// What:
///  A copyable three-part tag;
///  `u64` is a fixed 64-bit unsigned integer,
///  unlike pointer-sized `usize`.
/// Why:
///  A reply is usable only for the same displayed file,
///  content revision,
///  and query text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FindIdentity = { file: number; revision: number; query: number };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FindIdentity {
    /// File-open generation;
    ///  changes on every navigation to another file.
    pub file: u64,
    /// Content revision;
    ///  changes on every accepted external reload.
    pub revision: u64,
    /// Query generation;
    ///  changes on every edit of the find text.
    pub query: u64,
}

/// Each component is compared separately so each has its own observed-failing regression test.
impl FindIdentity {
    /// True only when the reply describes the wanted file,
    ///  revision,
    ///  and query.
    pub fn is_current(&self, wanted: &FindIdentity) -> bool {
        return self.file == wanted.file
            && self.revision == wanted.revision
            && self.query == wanted.query;
    }
}

/// An owned snapshot of what to search;
///  nothing in it borrows native state.
pub struct FindRequest {
    /// Tag copied unchanged into the reply.
    pub identity: FindIdentity,
    /// Immutable source snapshot at `identity.revision`.
    pub source: Rope,
    /// Literal find text.
    pub query: String,
}

/// Matches or a diagnostic for exactly one request.
#[derive(Debug)]
pub struct FindReply {
    /// Tag of the request that produced this reply.
    pub identity: FindIdentity,
    /// Bounded matches,
    ///  or why this query or file cannot be searched.
    pub result: Result<FindMatches>,
}

/// What:
///  `Option<T>` holds a value or nothing;
///  the struct owns both channel ends it uses.
/// Why:
///  At most one job runs and at most one newer request waits,
///  so memory stays bounded while typing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class FindWorker { busy = false; waiting?: FindRequest; wanted?: FindIdentity }
/// ```
pub struct FindWorker {
    /// Taking this sender closes the request channel before joining during Drop.
    requests: Option<SyncSender<FindRequest>>,
    /// The native thread polls this receiver without blocking.
    replies: Receiver<FindReply>,
    /// Covers a running job and its unread reply.
    busy: bool,
    /// Newest request not yet sent;
    ///  an older waiting request is dropped unsearched.
    waiting: Option<FindRequest>,
    /// Only replies carrying this identity are returned.
    wanted: Option<FindIdentity>,
    /// Joined on shutdown,
    ///  never detached.
    thread: Option<JoinHandle<()>>,
}

/// Own the blocking receive loop entirely outside native state.
fn run(requests: Receiver<FindRequest>, replies: SyncSender<FindReply>) {
    tracing::debug!("in-file find worker started");
    loop {
        // What: `match` extracts either an owned request or the closed-channel condition.
        // Why: Closing the owner ends this thread without an unhandled channel error.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const request = await receive(); if (closed) return;
        // ```
        let request = match requests.recv() {
            Ok(request) => request,
            Err(error) => {
                tracing::debug!(%error, "in-file find request channel closed");
                return;
            }
        };
        // What: `&request.source` and `&request.query` lend the snapshot to the matcher.
        // Why: The request keeps ownership so its identity can be moved into the reply afterwards.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const result = tryCatch(() => findInSource(request.source, request.query));
        // ```
        let result = find_in_source(&request.source, &request.query);
        // What: `if let Err(error)` runs only when sending failed.
        // Why: A closed receiver means the window is gone; the thread stops instead of retrying.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // try { send({ identity, result }); } catch (error) { log(error); return; }
        // ```
        if let Err(error) = replies.send(FindReply {
            identity: request.identity,
            result,
        }) {
            tracing::debug!(%error, "in-file find reply receiver closed");
            return;
        }
    }
}

/// Requests replace each other;
///  polling returns only the reply for the newest request.
impl FindWorker {
    /// Start a named worker with one request slot and one reply slot.
    pub fn new() -> Result<Self> {
        // What: `sync_channel(1)` creates a sender and a receiver sharing one message slot.
        // Why: Rapid typing cannot accumulate source snapshots in a queue.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const requests = boundedChannel(1); const replies = boundedChannel(1);
        // ```
        let (request_sender, request_receiver) = sync_channel(1);
        let (reply_sender, reply_receiver) = sync_channel(1);
        // What: `move ||` transfers both channel ends into a named OS thread; `?` returns a start failure.
        // Why: No window handle or shared native state crosses the thread boundary.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const thread = startWorker(() => run(requestReceiver, replySender));
        // ```
        let worker = thread::Builder::new()
            .name("ide-in-file-find".to_string())
            .spawn(move || {
                run(request_receiver, reply_sender);
            })
            .context("Cannot start the in-file find worker")?;
        // What: `Some` stores present owners; `None` records that nothing is requested yet.
        // Why: Drop later takes each owner exactly once to close and join in order.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { requests, replies, busy: false, waiting: undefined, wanted: undefined, thread };
        // ```
        return Ok(Self {
            requests: Some(request_sender),
            replies: reply_receiver,
            busy: false,
            waiting: None,
            wanted: None,
            thread: Some(worker),
        });
    }

    /// Make this the only wanted request;
    ///  a job already running finishes and its reply is discarded.
    pub fn request(&mut self, request: FindRequest) -> Result<()> {
        tracing::debug!(identity = ?request.identity, "requested in-file find");
        self.wanted = Some(request.identity);
        self.waiting = Some(request);
        return self.dispatch();
    }

    /// Want nothing:
    ///  drop the waiting request and discard whatever reply is still coming.
    pub fn cancel(&mut self) {
        tracing::debug!("cancelled in-file find requests");
        self.wanted = None;
        self.waiting = None;
    }

    /// True while a job runs,
    ///  its reply is unread,
    ///  or a request waits.
    pub fn is_pending(&self) -> bool {
        return self.busy || self.waiting.is_some();
    }

    /// Send the waiting request when the single job slot is free.
    fn dispatch(&mut self) -> Result<()> {
        if self.busy {
            return Ok(());
        }
        // What: `take` moves the waiting request out and leaves `None` behind.
        // Why: A request is sent at most once.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const request = this.waiting; this.waiting = undefined; if (!request) return;
        // ```
        let Some(request) = self.waiting.take() else {
            return Ok(());
        };
        // What: `as_ref` borrows the optional sender; `context` turns absence into an error.
        // Why: A stopped worker reports a restart diagnostic instead of silently never answering.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!this.requests) throw new Error('In-file find worker is closed');
        // ```
        let sender = self
            .requests
            .as_ref()
            .context("In-file find worker is closed; restart the application")?;
        sender
            .try_send(request)
            .context("Cannot send an in-file find request; restart the application")?;
        self.busy = true;
        return Ok(());
    }

    /// Poll without blocking;
    ///  a reply for anything but the wanted identity is consumed and dropped.
    pub fn poll(&mut self) -> Result<Option<FindReply>> {
        let reply = match self.replies.try_recv() {
            Ok(reply) => reply,
            Err(TryRecvError::Empty) => {
                return Ok(None);
            }
            Err(TryRecvError::Disconnected) => {
                // Release admission state before reporting the terminal transport failure.
                self.busy = false;
                self.waiting = None;
                self.requests.take();
                bail!(
                    "In-file find worker stopped unexpectedly; restart the application to search inside files"
                );
            }
        };
        self.busy = false;
        self.dispatch()?;
        if let Some(wanted) = &self.wanted
            && reply.identity.is_current(wanted)
        {
            return Ok(Some(reply));
        }
        tracing::debug!(identity = ?reply.identity, wanted = ?self.wanted, "discarded stale in-file find reply");
        return Ok(None);
    }
}

/// Synthetic channels make each identity component's rejection independent of thread timing.
#[cfg(test)]
#[path = "find_worker_tests.rs"]
mod tests;

/// Close input before joining so the last bounded reply can complete.
impl Drop for FindWorker {
    /// End the owned thread and log an unexpected panic instead of detaching it.
    fn drop(&mut self) {
        // Taking the sender drops its channel end at this statement.
        self.requests.take();
        let Some(worker) = self.thread.take() else {
            return;
        };
        if let Err(error) = worker.join() {
            tracing::error!(?error, "in-file find worker panicked during shutdown");
        } else {
            tracing::debug!("in-file find worker joined");
        }
    }
}
