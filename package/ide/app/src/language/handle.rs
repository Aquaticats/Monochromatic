//! The interface thread's handle:
//!  non-blocking commands in,
//!  polled and fenced results out.

/// Setup of the language registry and the launch seam.
use super::config::LanguageSetup;
/// The fence decides which results are still about the displayed text.
use super::fence::{Fence, FenceCounts};
/// Latest-value results.
use super::{diagnostics::DiagnosticsSnapshot, hints::HintsSnapshot, status::LanguageStatus};
/// Commands and one-shot replies.
use super::{
    hints::HintWindow,
    identity::DocumentStamp,
    reply::{LanguageReply, PositionRequest},
    sync::{DocumentOpen, DocumentReload},
    worker::{self, Command, Outputs},
};
/// The changes the change watcher forwards to servers.
use crate::change_watch::ServerChange;
/// Failures are reported as user-facing errors,
///  never as empty results.
use anyhow::{Result, anyhow};
/// What:
///  `Path` is a borrowed filesystem path;
///  `Arc` is a thread-safe shared pointer;
///       `JoinHandle` lets the owner wait for a thread to end.
/// Why:
///  Published values are shared and immutable;
///  the worker thread is joined,
///  never detached.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const worker = new Worker('language');
/// ```
use std::{path::Path, sync::Arc, thread::JoinHandle};
/// What:
///  `mpsc` is a bounded queue;
///  `watch` is a single-slot latest-value cell.
///  `TryRecvError`
///       and `TrySendError` say why a non-blocking receive or send did not succeed.
/// Why:
///  Nothing on the interface thread may wait for the worker.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const commands = boundedQueue<Command>(64);
/// ```
use tokio::sync::{
    mpsc::{self, error::TryRecvError, error::TrySendError},
    watch,
};

/// Commands that may wait in the queue;
///  a full queue is reported,
///  never waited on.
const COMMAND_CAPACITY: usize = 64;

/// One-shot replies that may wait to be polled.
const REPLY_CAPACITY: usize = 64;

/// What:
///  The interface thread's end of the Language module.
///  `Option<...>` fields are "a value,
///       or nothing";
///  they become nothing when the worker is closed.
/// Why:
///  No Slint handle and no mutable document crosses this boundary;
///  only owned messages do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class LanguageWorker { open(...); reload(...); close(); request(...); requestHints(...);
///                        tryTakeStatus(); tryTakeReply(); tryTakeDiagnostics(); tryTakeHints(); }
/// ```
pub struct LanguageWorker {
    /// Dropping this sender ends the worker loop,
    ///  which then stops every server.
    commands: Option<mpsc::Sender<Command>>,
    /// One-shot replies to position requests.
    replies: mpsc::Receiver<LanguageReply>,
    /// Latest status.
    status: watch::Receiver<Arc<LanguageStatus>>,
    /// Latest diagnostics.
    diagnostics: watch::Receiver<Option<Arc<DiagnosticsSnapshot>>>,
    /// Latest hints.
    hints: watch::Receiver<Option<Arc<HintsSnapshot>>>,
    /// The queue the change watcher sends file changes into;
    ///  the worker reads it.
    file_changes: mpsc::UnboundedSender<ServerChange>,
    /// Whether some server registered file watchers.
    watching: watch::Receiver<bool>,
    /// Drops results for another file,
    ///  revision,
    ///  or server process.
    fence: Fence,
    /// Number given to the next position request.
    ///  `u64` never wraps in practice.
    next_request: u64,
    /// Joined on shutdown.
    thread: Option<JoinHandle<()>>,
    /// Why the worker stopped,
    ///  once that was observed.
    failure: Option<String>,
}

/// Render what a panicking thread left behind as text.
fn panic_text(payload: &(dyn std::any::Any + Send)) -> String {
    // What: `downcast_ref` asks "is this value of that type?" and returns `Option<&T>`.
    // Why: A panic carries either a string literal or a formatted `String`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return typeof payload === 'string' ? payload : 'unknown failure';
    // ```
    if let Some(text) = payload.downcast_ref::<&str>() {
        return text.to_string();
    }
    if let Some(text) = payload.downcast_ref::<String>() {
        return text.clone();
    }
    return "unknown failure".to_string();
}

/// Commands,
///  polling,
///  and shutdown.
impl LanguageWorker {
    /// Start the worker for one project root with the production setup.
    ///  No server is started
    /// until a file is displayed.
    pub fn new(project_root: &Path) -> Result<Self> {
        return Self::with_setup(project_root, LanguageSetup::default());
    }

    /// What:
    ///  Start the worker with an explicit setup.
    ///  `Result<Self>` is the handle or a start error.
    /// Why:
    ///  The launch policy,
    ///  the private state directory,
    ///  and application-supplied definitions
    ///      are chosen by the caller;
    ///  the language registry itself is built on the worker thread.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static withSetup(projectRoot: string, setup: LanguageSetup): LanguageWorker
    /// ```
    pub fn with_setup(project_root: &Path, setup: LanguageSetup) -> Result<Self> {
        // What: `canonicalize` resolves symbolic links; `with_context` attaches a message built by
        //       the closure only on failure; the trailing `?` returns the error to the caller.
        // Why: Containment of targets and roots is judged on resolved paths.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const root = realpathSync(projectRoot);
        // ```
        let root = match project_root.canonicalize() {
            Ok(found) => found,
            Err(error) => {
                return Err(anyhow::Error::new(error).context(format!(
                    "Cannot resolve project directory {}",
                    project_root.display()
                )));
            }
        };
        let (command_sender, command_receiver) = mpsc::channel(COMMAND_CAPACITY);
        let (reply_sender, reply_receiver) = mpsc::channel(REPLY_CAPACITY);
        let (status_sender, status_receiver) = watch::channel(Arc::new(LanguageStatus::closed()));
        let (diagnostics_sender, diagnostics_receiver) = watch::channel(None);
        let (hints_sender, hints_receiver) = watch::channel(None);
        let (watching_sender, watching_receiver) = watch::channel(false);
        // What: `unbounded_channel` makes a queue without a size limit.
        // Why: The change watcher sends from notify's thread and must never wait for the worker.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const fileChanges = new Queue<ServerChange>();
        // ```
        let (change_sender, change_receiver) = mpsc::unbounded_channel();
        let outputs = Outputs {
            replies: reply_sender,
            status: status_sender,
            diagnostics: diagnostics_sender,
            hints: hints_sender,
            watching: watching_sender,
        };
        let thread = worker::spawn(root, setup, command_receiver, change_receiver, outputs)?;
        // `Ok(...)` is the success variant of `Result`.
        return Ok(Self {
            commands: Some(command_sender),
            replies: reply_receiver,
            status: status_receiver,
            diagnostics: diagnostics_receiver,
            hints: hints_receiver,
            file_changes: change_sender,
            watching: watching_receiver,
            fence: Fence::default(),
            next_request: 0,
            thread: Some(thread),
            failure: None,
        });
    }

    /// What:
    ///  Build the error for a worker that is gone,
    ///  joining its thread once to learn why.
    /// Why:
    ///  A worker panic is a reported failure of the Language module;
    ///  the window keeps working.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// stopped(): Error
    /// ```
    fn stopped(&mut self) -> anyhow::Error {
        self.commands.take();
        // `take()` moves the handle out, so the thread is joined at most once.
        if let Some(thread) = self.thread.take() {
            let reason = match thread.join() {
                Ok(()) => "its language registry could not be built; see the log".to_string(),
                Err(payload) => panic_text(payload.as_ref()),
            };
            tracing::error!(%reason, "language worker stopped");
            self.failure = Some(reason);
        }
        // `as_deref()` borrows the stored text; `unwrap_or` substitutes a generic reason.
        let reason = self.failure.as_deref().unwrap_or("unknown failure");
        return anyhow!(
            "Language support stopped unexpectedly ({reason}). Definitions, references, hover, inlay hints, and diagnostics are unavailable until the application is restarted"
        );
    }

    /// What:
    ///  A sender for the change watcher's file changes;
    ///  `clone` makes another handle to one queue.
    /// Why:
    ///  The change watcher feeds the worker directly,
    ///  without the interface thread in between.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// fileChangeSender(): Queue<ServerChange>
    /// ```
    pub fn file_change_sender(&self) -> mpsc::UnboundedSender<ServerChange> {
        return self.file_changes.clone();
    }

    /// What:
    ///  Whether some server registered file watchers;
    ///  `borrow` reads the latest value without waiting.
    /// Why:
    ///  The project's folders are watched for servers only while one asked.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// wantsFileChanges(): boolean
    /// ```
    pub fn wants_file_changes(&self) -> bool {
        return *self.watching.borrow();
    }

    /// What:
    ///  Queue one command without waiting.
    ///  `Ok(false)` means the queue is full and nothing
    ///       was sent;
    ///  `Err` means the worker is gone.
    /// Why:
    ///  The interface thread never blocks;
    ///  a full queue is retried at the next poll.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// send(command: Command): boolean // throws when the worker stopped
    /// ```
    fn send(&mut self, command: Command) -> Result<bool> {
        // `as_ref()` borrows the sender inside the `Option`.
        let Some(sender) = self.commands.as_ref() else {
            return Err(self.stopped());
        };
        return match sender.try_send(command) {
            Ok(()) => Ok(true),
            Err(TrySendError::Full(_)) => {
                tracing::warn!("language command queue is full; the command was not sent");
                Ok(false)
            }
            Err(TrySendError::Closed(_)) => Err(self.stopped()),
        };
    }

    /// A file was displayed.
    ///  From now on only results for `open.stamp` are handed out.
    pub fn open(&mut self, open: DocumentOpen) -> Result<bool> {
        let stamp = open.stamp;
        let sent = self.send(Command::Open(open))?;
        if sent {
            // `Some(...)` is the "value present" variant of `Option`.
            self.fence.display(Some(stamp));
        }
        return Ok(sent);
    }

    /// The displayed file was reloaded.
    ///  Results for the previous revision are dropped from now on.
    pub fn reload(&mut self, reload: DocumentReload) -> Result<bool> {
        let stamp = DocumentStamp {
            file: reload.file,
            revision: reload.revision,
        };
        let sent = self.send(Command::Reload(reload))?;
        if sent {
            self.fence.display(Some(stamp));
        }
        return Ok(sent);
    }

    /// No file is displayed.
    ///  Every pending result is dropped from now on.
    pub fn close(&mut self) -> Result<bool> {
        let sent = self.send(Command::Close)?;
        if sent {
            // `None` is the "nothing" variant: nothing is displayed.
            self.fence.display(None);
        }
        return Ok(sent);
    }

    /// What:
    ///  Ask for a definition,
    ///  references,
    ///  or hover.
    ///  Returns the request number its replies
    ///       carry,
    ///  or nothing when the queue is full.
    /// Why:
    ///  Each asked server replies separately;
    ///  the number ties the replies together.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// request(request: PositionRequest): number | undefined
    /// ```
    pub fn request(&mut self, request: PositionRequest) -> Result<Option<u64>> {
        let number = self.next_request + 1;
        let sent = self.send(Command::Request { number, request })?;
        if !sent {
            return Ok(None);
        }
        self.next_request = number;
        return Ok(Some(number));
    }

    /// Report the visible lines;
    ///  hints for them arrive through `try_take_hints`.
    pub fn request_hints(&mut self, stamp: DocumentStamp, window: HintWindow) -> Result<bool> {
        return self.send(Command::Hints { stamp, window });
    }

    /// What:
    ///  Copy the current server processes from the latest status into the fence.
    /// Why:
    ///  A result from a process that was since replaced must not be mixed with the new one's.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// refreshServers() { fence.servers = status.value.servers.map(row => row.server); }
    /// ```
    fn refresh_servers(&mut self) {
        // `borrow()` reads the slot without marking it as seen; `clone` copies the shared pointer.
        let status = self.status.borrow().clone();
        let mut servers = Vec::new();
        for row in &status.servers {
            // Generation zero marks a server that never ran; it cannot have produced a result.
            if row.server.instance > 0 {
                servers.push(row.server.clone());
            }
        }
        self.fence.set_servers(servers);
    }

    /// What:
    ///  Take the latest status if it changed since the last call.
    ///  `Ok(None)` means no change.
    /// Why:
    ///  Polling from a timer keeps all interface state on the interface thread.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// tryTakeStatus(): LanguageStatus | undefined
    /// ```
    pub fn try_take_status(&mut self) -> Result<Option<Arc<LanguageStatus>>> {
        // `has_changed` returns `Err` when the worker dropped its sender.
        let Ok(changed) = self.status.has_changed() else {
            return Err(self.stopped());
        };
        if !changed {
            return Ok(None);
        }
        let status = self.status.borrow_and_update().clone();
        return Ok(Some(status));
    }

    /// What:
    ///  Take the next reply that is still about the displayed text and a current server
    ///       process.
    ///  Stale replies are consumed and never returned.
    /// Why:
    ///  The fence is applied when a reply is read,
    ///  so a reload or file switch that happened
    ///      while it waited in the queue is taken into account.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// tryTakeReply(): LanguageReply | undefined
    /// ```
    pub fn try_take_reply(&mut self) -> Result<Option<LanguageReply>> {
        self.refresh_servers();
        loop {
            match self.replies.try_recv() {
                Ok(reply) => {
                    if self.fence.admit(reply.stamp, reply.server.as_ref()) {
                        return Ok(Some(reply));
                    }
                }
                Err(TryRecvError::Empty) => return Ok(None),
                Err(TryRecvError::Disconnected) => return Err(self.stopped()),
            }
        }
    }

    /// Take the latest diagnostics if they changed and describe the displayed text.
    pub fn try_take_diagnostics(&mut self) -> Result<Option<Arc<DiagnosticsSnapshot>>> {
        let Ok(changed) = self.diagnostics.has_changed() else {
            return Err(self.stopped());
        };
        if !changed {
            return Ok(None);
        }
        let Some(snapshot) = self.diagnostics.borrow_and_update().clone() else {
            return Ok(None);
        };
        if !self.fence.admit(snapshot.stamp, None) {
            return Ok(None);
        }
        return Ok(Some(snapshot));
    }

    /// Take the latest hints if they changed and describe the displayed text.
    pub fn try_take_hints(&mut self) -> Result<Option<Arc<HintsSnapshot>>> {
        let Ok(changed) = self.hints.has_changed() else {
            return Err(self.stopped());
        };
        if !changed {
            return Ok(None);
        }
        let Some(snapshot) = self.hints.borrow_and_update().clone() else {
            return Ok(None);
        };
        if !self.fence.admit(snapshot.stamp, None) {
            return Ok(None);
        }
        return Ok(Some(snapshot));
    }

    /// How many results the fence accepted and dropped,
    ///  per reason.
    pub fn fence_counts(&self) -> FenceCounts {
        return self.fence.counts();
    }
}

/// What:
///  `impl Drop` runs when the handle goes out of scope.
/// Why:
///  Closing the command queue makes the worker stop every server;
///  joining guarantees no
///      server process outlives the window through this module.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [Symbol.dispose]() { this.commands.close(); this.thread.join(); }
/// ```
impl Drop for LanguageWorker {
    /// Close the queue,
    ///  then wait for the worker to finish its shutdown.
    fn drop(&mut self) {
        self.commands.take();
        let Some(thread) = self.thread.take() else {
            return;
        };
        if let Err(payload) = thread.join() {
            tracing::error!(reason = %panic_text(payload.as_ref()), "language worker panicked");
        }
    }
}
