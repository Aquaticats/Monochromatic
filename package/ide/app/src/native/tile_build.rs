//! Source tile construction,
//! shared by the UI thread and the tile worker,
//!  and the named worker thread that builds scroll re-centers off the UI thread.

/// The background measurement of the whole text,
/// its slots,
/// and its walk.
use super::tile_pass::{PassControl, PassJob, PassOutbox, PassOutput};
/// Worker-start and build failures stay visible diagnostics.
use anyhow::{Context, Result, bail};
/// The shaper a worker is started with.
use ide_app::shaped_text::TextShaper;
/// Owned snapshots are shared between the UI and worker threads.
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, SyncSender, TryRecvError, TrySendError, sync_channel},
    },
    thread::{self, JoinHandle},
};

/// The inputs and result of one tile,
///  and the construction both threads run.
mod job;
/// Which pass the handle counts as open.
#[cfg(test)]
mod pass_tests;
/// The worker thread's loop,
///  its shaper,
///  and the test gates that hold it.
mod worker_thread;

/// The test switch that makes the next build refuse to paint.
#[cfg(test)]
pub(super) use job::FAIL_NEXT_BUILD;
/// The tile types and the construction the window thread and the minimap worker also use.
pub(super) use job::{TileBuilt, TileJob, build_tile, same_typography};
/// Test gates that hold a build or a pass chunk in flight.
#[cfg(test)]
pub(super) use worker_thread::{BUILD_GATE, PASS_GATE};

/// What a worker sends back:
/// once its shaper is ready,
/// then one result per job.
/// Why:
/// The window thread can tell a warm worker from a cold one without waiting for a job.
enum WorkerReply {
    /// The worker's shaper and raster are ready to build.
    Ready,
    /// The finished tile or its failure.
    Built(
        /// The build's own result,
        ///  boxed so the enum stays small.
        Box<Result<TileBuilt>>,
    ),
}

/// What the window sends the worker.
/// Why:
/// A tile job carries its inputs;
/// a wake only tells an idle worker to look at the pass slot.
enum Command {
    /// Build this tile.
    Tile(
        /// The job's inputs,
        ///  boxed so the enum stays small.
        Box<TileJob>,
    ),
    /// A newer pass waits in the pass slot.
    Wake,
}

/// What:
///  The slots and flags the worker shares with the window thread besides its two channels.
/// Why:
///  The pass is not a stream of messages:
/// the newest wanted pass sits in one slot,
///  and what the worker measured coalesces in another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Shared = { control: PassControl; outbox: PassOutbox; wakePending: boolean };
/// ```
#[derive(Clone, Default)]
struct Shared {
    /// The newest wanted pass.
    control: Arc<PassControl>,
    /// What the worker measured and the window has not taken.
    outbox: Arc<PassOutbox>,
    /// A wake is queued or the channel is full of commands that will make the worker look.
    wake_pending: Arc<AtomicBool>,
}

/// What:
///  A worker that builds at most one tile at a time,
/// keeps only the newest waiting job,
///  and measures the whole text between jobs.
/// Why:
///  Scrolling can ask for many tiles;
/// only the newest one still matches the view.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class TileWorker { busy = false; waiting?: TileJob; passOpen = false }
/// ```
pub(super) struct TileWorker {
    /// Taking this sender closes the request channel before joining during Drop.
    requests: Option<SyncSender<Command>>,
    /// The window thread polls this receiver without blocking.
    replies: Receiver<WorkerReply>,
    /// The pass slots shared with the worker thread.
    shared: Shared,
    /// The worker has reported that its shaper is ready.
    #[cfg(test)]
    ready: bool,
    /// A job is running or its reply is unread.
    busy: bool,
    /// The newest posted pass was not finished when its last output was taken.
    pass_open: bool,
    /// How many passes were posted;
    /// the newest post's number tells its output from the output of a pass it replaced.
    pass_generation: u64,
    /// Newest job not yet sent;
    /// an older waiting job is dropped unbuilt.
    waiting: Option<TileJob>,
    /// Joined on shutdown,
    /// never detached.
    thread: Option<JoinHandle<()>>,
}

/// Requests replace each other;
/// polling returns each finished tile in turn.
impl TileWorker {
    /// Start a named worker with a short command queue and one reply slot,
    /// building with `shaper`.
    pub(super) fn new(shaper: TextShaper) -> Result<Self> {
        // A tile job and a wake can wait together; the window never queues more of either.
        let (request_sender, request_receiver) = sync_channel(4);
        let (reply_sender, reply_receiver) = sync_channel(1);
        let shared = Shared::default();
        let worker_shared = shared.clone();
        let worker = thread::Builder::new()
            .name("ide-source-tile".to_string())
            .spawn(move || {
                // What: `catch_unwind` (std::panic) stops an unwinding build at the thread boundary.
                // Why: A detached thread has no join to report a panic, so the panic is logged here instead.
                let outcome = catch_unwind(AssertUnwindSafe(
                    #[cfg_attr(
                        dylint_lib = "monochromatic_lint_closure",
                        expect(
                            avoidable_closure,
                            reason = "catch_unwind accepts only a callable without parameters, so the shaper and the channel ends the worker runs with must be moved into a closure"
                        )
                    )]
                    || {
                        worker_thread::run(shaper, request_receiver, reply_sender, worker_shared);
                    },
                ));
                if outcome.is_err() {
                    tracing::error!(
                        "source tile worker panicked; the next scroll re-center starts a new worker"
                    );
                }
            })
            .context("Cannot start the source tile worker")?;
        return Ok(Self {
            requests: Some(request_sender),
            replies: reply_receiver,
            shared,
            #[cfg(test)]
            ready: false,
            busy: false,
            pass_open: false,
            pass_generation: 0,
            waiting: None,
            thread: Some(worker),
        });
    }

    /// Make `job` the only waiting request;
    /// a job already running finishes first.
    pub(super) fn request(&mut self, job: TileJob) -> Result<()> {
        self.waiting = Some(job);
        return self.dispatch();
    }

    /// What:
    ///  Make `job` the pass the worker carries on between tiles,
    /// and wake an idle worker.
    /// Why:
    ///  A newer pass replaces an older one,
    /// so the slot holds only the newest.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// postPass(job: PassJob): void;
    /// ```
    pub(super) fn post_pass(&mut self, mut job: PassJob) -> Result<()> {
        self.pass_generation += 1;
        job.generation = self.pass_generation;
        self.shared.control.set(job);
        self.pass_open = true;
        let sender = self
            .requests
            .as_ref()
            .context("Source tile worker is closed; restart the application")?;
        if self.shared.wake_pending.swap(true, Ordering::SeqCst) {
            return Ok(());
        }
        return match sender.try_send(Command::Wake) {
            // A full channel holds commands that make the worker look at the slot anyway.
            Ok(()) | Err(TrySendError::Full(_)) => Ok(()),
            Err(TrySendError::Disconnected(_)) => {
                bail!("Source tile worker is closed; restart the application")
            }
        };
    }

    /// Take what the pass measured since the last take,
    /// at most `limit` lines;
    ///  the pass is no longer open once the last output of the newest post is taken.
    ///  The finishing output of a replaced pass still arrives after the newer pass was posted,
    ///  and closing on it would hide the newer pass from `is_busy` while it runs,
    ///  so the poll would stop and the newer pass's output would never be taken.
    pub(super) fn take_pass(&mut self, limit: usize) -> Option<PassOutput> {
        let output = self.shared.outbox.take(limit)?;
        if output.done && output.generation == self.pass_generation {
            self.pass_open = false;
        }
        return Some(output);
    }

    /// Measured lines the window has not taken yet;
    /// test-only.
    #[cfg(test)]
    pub(super) fn pending_pass_lines(&self) -> usize {
        return self.shared.outbox.waiting_lines();
    }

    /// True once the worker has reported that its shaper is ready to build.
    #[cfg(test)]
    pub(super) fn is_ready(&self) -> bool {
        return self.ready;
    }

    /// True while a job is running,
    /// its reply is unread,
    /// a job waits,
    /// or the pass is not finished;
    ///  the poll must continue while this holds.
    pub(super) fn is_busy(&self) -> bool {
        return self.busy || self.waiting.is_some() || self.pass_open;
    }

    /// Close the request channel so the idle worker thread ends,
    /// as a crashed worker would;
    /// test-only.
    #[cfg(test)]
    pub(super) fn close_for_test(&mut self) {
        self.requests.take();
    }

    /// Drop the waiting job;
    /// a running job finishes and its reply is left for the caller to discard.
    pub(super) fn cancel(&mut self) {
        self.waiting = None;
    }

    /// Send the waiting job when the single build slot is free.
    fn dispatch(&mut self) -> Result<()> {
        if self.busy {
            return Ok(());
        }
        let Some(job) = self.waiting.take() else {
            return Ok(());
        };
        let sender = self
            .requests
            .as_ref()
            .context("Source tile worker is closed; restart the application")?;
        if let Err(error) = sender.try_send(Command::Tile(Box::new(job))) {
            return Err(anyhow::anyhow!("{error}")
                .context("Cannot send a source tile request; restart the application"));
        }
        self.busy = true;
        return Ok(());
    }

    /// Poll without blocking;
    /// the inner result is the build's own success or failure.
    pub(super) fn poll(&mut self) -> Result<Option<Result<TileBuilt>>> {
        loop {
            match self.replies.try_recv() {
                Ok(WorkerReply::Ready) => {
                    #[cfg(test)]
                    {
                        self.ready = true;
                    }
                }
                Ok(WorkerReply::Built(reply)) => {
                    self.busy = false;
                    self.dispatch()?;
                    return Ok(Some(*reply));
                }
                Err(TryRecvError::Empty) => {
                    return Ok(None);
                }
                Err(TryRecvError::Disconnected) => {
                    self.busy = false;
                    self.waiting = None;
                    self.pass_open = false;
                    self.requests.take();
                    bail!(
                        "The source tile worker stopped unexpectedly; restart the application to scroll the source"
                    );
                }
            }
        }
    }
}

/// Close input before joining so the last bounded reply can complete.
impl Drop for TileWorker {
    /// Close the request channel and detach the thread,
    /// so closing the window never waits for a build in flight.
    /// The detached thread ends after its current build,
    /// when the closed channel ends its loop.
    fn drop(&mut self) {
        self.requests.take();
        // Dropping the join handle detaches the thread; there is no cancellation point inside a build to stop it earlier.
        if self.thread.take().is_some() {
            tracing::debug!("source tile worker detached");
        }
    }
}
