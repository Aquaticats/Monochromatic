//! Engine:
//!  the worker-thread front door.
//!  `Engine::spawn` starts a background
//! thread running `run`,
//!  which owns a `Controller` (the real playback state,
//!  in
//! `controller.rs`) and drives it from the command channel.
//!
//! Threads,
//!  for a TypeScript reader:
//!  `Engine::spawn` starts a background worker
//! (`std::thread`).
//!  The UI talks to it through a one-way queue of `Command`s
//! (an `mpsc` channel:
//!  many senders,
//!  one receiver).
//!  The worker replies by
//! calling an `on_update` callback the UI supplied.
//!  The PipeWire output runs its
//! OWN thread internally;
//!  the controller thread only decodes and pushes samples
//! into the ring buffer the output hands back.

/// What:
///      `use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};`.
///  The
///           multi-producer/single-consumer channel:
///  `Sender` pushes,
///  `Receiver` pops,
///           `TryRecvError` reports "empty" vs "all senders gone".
///  `self` also imports
///           the `mpsc` module itself (for `mpsc::channel()`).
/// Why:
///       The UI thread sends `Command`s to this worker thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a thread-safe queue split into a Sender (push) and a Receiver (tryPop)
/// ```
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};

/// What:
///      `use std::thread::{self, JoinHandle, Thread};`.
///  `thread::spawn` starts a
///           worker;
///  a `JoinHandle` lets us wait for it to finish;
///  a `Thread` is a cheap,
///           cloneable HANDLE to a running thread (it wraps an internal `Arc`),
///  used here
///           only to call `.unpark()` on the worker.
///  Sibling you might expect:
///  there is no
///           separate "thread id" type you'd pass around;
///  `Thread` is that handle.
/// Why:
///       The engine runs on its own thread,
///  and other threads need a `Thread` handle
///           to wake it from a park (see `park_timeout` in `run`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // JoinHandle ~ a Worker + exit promise; Thread ~ a WorkerRef you can post "wake" to
/// ```
use std::thread::{self, JoinHandle, Thread};
use std::sync::Arc;

/// What:
///      `use std::time::Duration;`.
///  A span of time (here,
///  a sleep interval).
/// Why:
///       We sleep briefly when idle to avoid busy-spinning the CPU.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Duration = number; // milliseconds
/// ```
use std::time::Duration;

/// What:
///      `use crate::command::{Command, Update};`.
///  The UI->engine and engine->UI
///           message enums.
/// Why:
///       The channel carries `Command`s;
///  the callback delivers `Update`s.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Command, Update } from "./command";
/// ```
use crate::command::{Command, Update};

/// What:
///      `use crate::controller::Controller;`.
///  The playback state machine.
/// Why:
///       `run` builds one and forwards commands/audio pumping to it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Controller } from "./controller";
/// ```
use crate::audio_meter::MeterReporter;
use crate::thread_priority::raise_current_thread_for_decode;
use crate::controller::Controller;

/// What:
///      `use crate::peakcache::CacheHandle;`.
///  The synchronous handle to the peak-cache
///           actor.
/// Why:
///       `run` opens the production cache (`CacheHandle::open`) and injects it into the
///           controller,
///  so the controller constructor stays test-friendly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CacheHandle } from "./peakcache";
/// ```
use crate::peakcache::CacheHandle;

/// What:
///      `use crate::output::Output;`.
///  The PipeWire output (FFI boundary).
/// Why:
///       `run` tries to create one and hands it to the controller.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Output } from "./output";
/// ```
use crate::output::Output;

/// What:
///      `const IDLE_PARK_FALLBACK_MS: u64 = 100;`.
///  Milliseconds the worker will PARK
///           (block,
///  using ~0 CPU) when there is no audio work this cycle,
///  if nothing
///           wakes it sooner.
///  `u64` (not `u32`/`i64`) is what `Duration::from_millis`
///           wants.
///  The worker is normally woken EARLY by an `unpark()` call:
///  the audio
///           callback unparks it after draining the ring buffer (space freed -> decode
///           more),
///  and command senders unpark it after queueing a command (act on it
///           now).
///  This timeout is only a SAFETY NET in case an `unpark` is ever missed;
///           it caps any stall well under the ~1 second the ring buffer holds,
///  so a missed
///           wake never causes an audio gap.
/// Why:
///       Replaces the old busy-poll:
///  the worker used to skip its sleep whenever a push
///           accepted even one sample,
///  so during playback it spun a whole CPU core.
///           Parking until explicitly woken drops idle CPU to near zero.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const IDLE_PARK_FALLBACK_MS = 100; // ms safety-net before re-checking when idle
/// ```
const IDLE_PARK_FALLBACK_MS: u64 = 100;

/// What:
///      `pub struct Engine { ... }`.
///  The handle the UI keeps.
///  It is `Send` (only a
///           channel sender + a thread handle),
///  unlike the controller's internal state.
/// Why:
///       Lets the UI send commands and stop the worker on drop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Engine { tx: Sender<Command>; worker: WorkerRef; handle: ThreadHandle | null; }
/// ```
pub struct Engine {
    /// What:
    ///      `tx: Sender<Command>`.
    ///  The send end of the command channel.
    /// Why:
    ///       `send` pushes commands to the worker.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// tx: Sender<Command>;
    /// ```
    tx: Sender<Command>,
    /// What:
    ///      `worker: Thread`.
    ///  A cloneable handle to the worker thread (the one
    ///           running `run`).
    ///  We never join through this;
    ///  we only call `.unpark()` on
    ///           it.
    /// Why:
    ///       After sending a command we must WAKE the worker,
    ///  which is otherwise parked
    ///           (blocked) when idle;
    ///  without this the command would sit unhandled until
    ///           the fallback timeout fires.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// worker: WorkerRef;
    /// ```
    worker: Thread,
    /// What:
    ///      `handle: Option<JoinHandle<()>>`.
    ///  The worker's join handle,
    ///  or `None`
    ///           after we have joined it.
    ///  `JoinHandle<()>` = the thread returns nothing.
    /// Why:
    ///       `Drop` joins the thread so the output cleans up before exit.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// handle: ThreadHandle | null;
    /// ```
    handle: Option<JoinHandle<()>>,
}

/// What:
///      `fn send_and_wake(tx: &Sender<Command>, worker: &Thread, command: Command)`.
///           Queue `command` on the channel,
///  then unpark the worker.
///  Borrows the channel and
///           the worker handle;
///  takes the command by value.
///  Module-private.
/// Why:
///       `Engine::send` and `CommandSender::send` share this exact send-then-wake
///           contract;
///  defining it once keeps the "never queue a command without waking the
///           worker" rule in one place.
///  A free function (rather than `Engine::send`
///           delegating through `sender()`) avoids the per-send `tx`/`worker` clones that
///           `sender()` would add.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sendAndWake(tx: Sender<Command>, worker: WorkerRef, command: Command): void {
///   try { tx.send(command); } catch {}
///   worker.postWakeUp();
/// }
/// ```
fn send_and_wake(tx: &Sender<Command>, worker: &Thread, command: Command) {
    // What:     `let _ = tx.send(command);`. `send` returns a `Result` that errs only if the
    //           worker is gone; `let _ =` DISCARDS it.
    // Why:      A dead worker during shutdown is not worth surfacing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { tx.send(command); } catch {}
    // ```
    let _ = tx.send(command);
    // What:     `worker.unpark();`. Wake the worker if it is parked; if it is not parked yet,
    //           this leaves a one-shot permit so its next `park` returns immediately (so the
    //           wake is never lost).
    // Why:      Make the worker act on the command now, not after the fallback timeout.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // worker.postWakeUp();
    // ```
    worker.unpark();
}

/// What:
///      `#[derive(Clone)] pub struct CommandSender { ... }`.
///  A small bundle of the
///           command channel's send end PLUS the worker's `Thread` handle.
///           `#[derive(Clone)]` auto-generates a `.clone()` that clones both fields (both
///           are cheap:
///  a `Sender` clone shares the channel,
///  a `Thread` clone bumps an
///           internal refcount).
/// Why:
///       Threads other than the UI (the file-picker thread) need to send commands AND
///           wake the worker.
///  Handing out a bare `Sender` would let them queue a command
///           without unparking,
///  so it would not be acted on until the timeout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CommandSender { tx: Sender<Command>; worker: WorkerRef; }
/// ```
#[derive(Clone)]
pub struct CommandSender {
    /// What:
    ///      `tx: Sender<Command>`.
    ///  The send end of the command channel.
    /// Why:
    ///       The picker thread pushes `OpenPaths` through it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// tx: Sender<Command>;
    /// ```
    tx: Sender<Command>,
    /// What:
    ///      `worker: Thread`.
    ///  The same worker handle `Engine` holds.
    /// Why:
    ///       Wake the worker after queueing a command.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// worker: WorkerRef;
    /// ```
    worker: Thread,
}

/// What:
///      `impl CommandSender { ... }`.
///  Its one method.
/// Why:
///       Mirror `Engine::send` for off-UI threads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CommandSender { send(command: Command): void { ... } }
/// ```
impl CommandSender {
    /// What:
    ///      `pub fn send(&self, command: Command)`.
    ///  Queue a command,
    ///  then wake the
    ///           worker.
    ///  Read-only borrow of self.
    /// Why:
    ///       Same contract as `Engine::send`,
    ///  usable from another OS thread.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// send(command: Command): void { ... }
    /// ```
    pub fn send(&self, command: Command) {
        // What:     `send_and_wake(&self.tx, &self.worker, command);`. Delegate to the shared
        //           send-then-wake helper, lending this sender's channel and worker handle.
        // Why:      One implementation of the "queue then unpark" contract, shared with
        //           `Engine::send`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // sendAndWake(this.tx, this.worker, command);
        // ```
        send_and_wake(&self.tx, &self.worker, command);
    }
}

/// What:
///      `impl Engine { ... }`.
///  The handle's methods.
/// Why:
///       Construction and command sending.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Engine { /* spawn, sender, send */ }
/// ```
impl Engine {
    /// What:
    ///      `pub fn spawn<F>(on_update: F) -> Engine where F: Fn(Update) + Send + 'static`.
    ///           Start the worker.
    ///  `F` is the callback type;
    ///  the WHERE clause requires it
    ///           be callable repeatedly (`Fn`),
    ///  movable to another thread (`Send`),
    ///  and own
    ///           no short-lived borrows (`'static`).
    /// Why:
    ///       The UI passes a closure that forwards updates to the Slint loop.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static spawn(onUpdate: (u: Update) => void): Engine { ... }
    /// ```
    pub fn spawn<F>(on_update: F) -> Engine
    where
        F: Fn(Update) + Send + 'static,
    {
        // What:     `Engine::spawn_with_cache(on_update, CacheHandle::open())`. Open the
        //           PRODUCTION peak cache here, then delegate to the cache-injecting body.
        // Why:      Keep the public entry point cache-free; tests call `spawn_with_cache` with
        //           a degraded handle so they never open or create the real peaks.db.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return Engine.spawnWithCache(onUpdate, CacheHandle.open());
        // ```
        return Engine::spawn_with_cache(on_update, CacheHandle::open())
    }

    /// What:
    ///      `pub(crate) fn spawn_with_cache<F>(on_update: F, cache: CacheHandle) -> Engine where F: Fn(Update) + Send + 'static`.
    ///           Start the worker around an INJECTED cache handle (the public `spawn` body,
    ///           minus opening the cache).
    /// Why:
    ///       Production `spawn` passes `CacheHandle::open()`;
    ///  the engine tests pass a
    ///           degraded handle,
    ///  so the worker never touches the real config dir.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static spawnWithCache(onUpdate, cache): Engine { ... }
    /// ```
    pub(crate) fn spawn_with_cache<F>(on_update: F, cache: CacheHandle) -> Engine
    where
        F: Fn(Update) + Send + 'static,
    {
        // What:     `let (tx, rx) = mpsc::channel::<Command>();`. Create the channel;
        //           destructure into sender `tx` and receiver `rx`. `::<Command>` is the
        //           turbofish pinning the element type.
        // Why:      The link between UI and worker.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const { tx, rx } = makeChannel<Command>();
        // ```
        let (tx, rx) = mpsc::channel::<Command>();

        // What:     `let callback: Box<dyn Fn(Update) + Send> = Box::new(on_update);`.
        //           `Box::new(...)` moves the callback onto the heap as a TRAIT OBJECT
        //           (`dyn Fn(...)`), so the worker can store one fixed type regardless of
        //           the concrete `F`. Sibling: `Rc`/`Arc` would add sharing we do not need.
        // Why:      Erase `F` to a uniform callback type.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const callback = onUpdate; // TS functions are already "boxed"
        // ```
        let callback: Box<dyn Fn(Update) + Send> = Box::new(on_update);

        // What:     `let self_tx = tx.clone();`. A second sender clone moved into the worker.
        // Why:      The worker builds a `CommandSender` from it so the file watcher can inject
        //           `Command::Rescan` back into this same channel.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const selfTx = tx; // same channel, second handle
        // ```
        let self_tx = tx.clone();

        // What:     `let handle = thread::spawn(move || run(rx, callback, self_tx, cache));`.
        //           `thread::spawn` starts the worker; the `move` closure takes ownership of
        //           `rx`, `callback`, `self_tx`, and the injected `cache` and runs `run(...)`
        //           on the new thread.
        // Why:      Decode/playback happens off the UI thread; the worker owns the self-sender
        //           used to wire the watcher and the cache handle it queries.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const handle = startWorker(() => run(rx, callback, selfTx, cache));
        // ```
        let handle = thread::spawn(move || run(rx, callback, self_tx, cache));

        // What:     `let worker = handle.thread().clone();`. `handle.thread()` borrows the
        //           spawned thread's `Thread` handle (`&Thread`); `.clone()` makes an owned
        //           copy (bumps an internal refcount, does not copy the thread).
        // Why:      Store a handle we can `unpark()` to wake the worker after sending a
        //           command. It refers to the SAME thread that `run` sees via
        //           `thread::current()`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const worker = handle.workerRef;
        // ```
        let worker = handle.thread().clone();

        // What:     `Engine { tx, worker, handle: Some(handle) }`. Build the handle (field
        //           shorthand for `tx`/`worker`). `Some(handle)` wraps the join handle so
        //           `Drop` can `.take()` it. Tail -> return.
        // Why:      Hand the UI its control surface.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return new Engine(tx, worker, handle);
        // ```
        return Engine {
            tx,
            worker,
            handle: Some(handle),
        }
    }

    /// What:
    ///      `pub fn sender(&self) -> CommandSender`.
    ///  Hand out a `CommandSender` (a
    ///           CLONE of the channel's send end bundled with the worker `Thread` handle).
    ///           Both inner parts are `Send`,
    ///  so the bundle can be moved to another OS
    ///           thread (the file-picker thread).
    /// Why:
    ///       The file dialog runs on its own thread and must send `OpenPaths` back AND
    ///           wake the worker;
    ///  it cannot hold the `!Send` `Rc<Engine>` the UI uses.
    ///           Returning the bundle (not a bare `Sender`) guarantees that off-UI sends
    ///           also unpark the worker.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// sender(): CommandSender { return new CommandSender(this.tx.clone(), this.worker); }
    /// ```
    pub fn sender(&self) -> CommandSender {
        // What:     `CommandSender { tx: self.tx.clone(), worker: self.worker.clone() }`.
        //           `self.tx.clone()` duplicates the sender (both refer to the same
        //           channel); `self.worker.clone()` duplicates the worker handle (refcount
        //           bump). Tail -> return.
        // Why:      Give the caller its own send-and-wake handle.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return new CommandSender(this.tx.clone(), this.worker);
        // ```
        return CommandSender {
            tx: self.tx.clone(),
            worker: self.worker.clone(),
        }
    }

    /// What:
    ///      `pub fn send(&self, command: Command)`.
    ///  Forward a command to the worker.
    ///           Read-only borrow of self.
    /// Why:
    ///       The UI's only way to control playback.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// send(command: Command): void { ... }
    /// ```
    pub fn send(&self, command: Command) {
        // What:     `send_and_wake(&self.tx, &self.worker, command);`. Delegate to the shared
        //           send-then-wake helper, lending this engine's channel and worker handle.
        // Why:      One implementation of the "queue then unpark" contract, shared with
        //           `CommandSender::send` (the off-UI-thread handle).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // sendAndWake(this.tx, this.worker, command);
        // ```
        send_and_wake(&self.tx, &self.worker, command);
    }
}

/// What:
///      `impl Drop for Engine { ... }`.
///  Cleanup when the UI drops the engine.
///  `Drop`
///           is the destructor trait;
///  its `drop` runs at end of scope.
/// Why:
///       Tell the worker to quit and wait for it,
///  so PipeWire shuts down.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Engine { [Symbol.dispose]() { /* stop + join worker */ } }
/// ```
impl Drop for Engine {
    /// What:
    ///      `fn drop(&mut self)`.
    ///  Runs at end of life.
    ///  `&mut self` because it tears the
    ///           engine down.
    /// Why:
    ///       Graceful shutdown.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// [Symbol.dispose]() { ... }
    /// ```
    fn drop(&mut self) {
        // What:     `let _ = self.tx.send(Command::Quit);`. Ask the worker to stop; ignore
        //           the error if it already exited.
        // Why:      Break the worker's loop.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // try { this.tx.send(Command.Quit); } catch {}
        // ```
        let _ = self.tx.send(Command::Quit);

        // What:     `self.worker.unpark();`. Wake the worker if it is parked, so it sees the
        //           `Quit` immediately rather than after the timeout.
        // Why:      Make shutdown prompt.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.worker.postWakeUp();
        // ```
        self.worker.unpark();

        // What:     `if let Some(handle) = self.handle.take() { let _ = handle.join(); }`.
        //           `.take()` moves the handle out (leaving `None`); `join()` waits for the
        //           worker to finish; `let _ =` ignores a panic result.
        // Why:      Ensure the output (and PipeWire) is fully torn down before exit.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.handle) { await this.handle; this.handle = null; }
        // ```
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// What:
///      `fn run(rx: Receiver<Command>, on_update: Box<dyn Fn(Update) + Send>, self_tx: Sender<Command>, cache: CacheHandle)`.
///           The worker's entry point:
///  set up state around the injected `cache`,
///  then loop
///           handling commands and pumping audio until told to quit.
///  `Box<dyn Fn(...)>` is
///           the heap-boxed callback trait object.
/// Why:
///       Everything playback-related lives on this one thread.
///  The cache is injected (not
///           opened here) so the engine tests can pass a degraded,
///  no-disk handle.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function run(rx, onUpdate, selfTx, cache): void { ... }
/// ```
fn run(
    rx: Receiver<Command>,
    on_update: Box<dyn Fn(Update) + Send>,
    self_tx: Sender<Command>,
    cache: CacheHandle,
) {
    // What:     `raise_current_thread_for_decode();`. Ask rtkit to raise this thread's nice level
    //           so decoding keeps up when the machine is busy.
    // Why:      Decoding runs on this thread. Under load at normal priority it can get too little
    //           CPU, and the ring buffer runs dry. Refusal is logged and playback continues.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // raiseCurrentThreadForDecode();
    // ```
    raise_current_thread_for_decode();
    // What:     `let output = match Output::new(thread::current()) { ... };`. Try to start
    //           PipeWire, handing it a handle to THIS thread. `thread::current()` returns
    //           the running thread's `Thread` handle; it is the same thread `Engine`
    //           stored via `handle.thread()`. On failure we log and run WITHOUT audio (the
    //           UI still works).
    // Why:      Never crash the app if audio init fails; the output keeps the handle so its
    //           realtime callback can `unpark()` us when the ring buffer drains.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let output;
    // try { output = Output.create(currentWorkerRef); } catch (e) { console.error(e); output = null; }
    // ```
    let output = match Output::new(thread::current()) {
        // What:     `Ok(o) => Some(o)`. Audio is available; wrap it as present.
        // Why:      Keep the output.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // output = o;
        // ```
        Ok(o) => Some(o),
        // What:     `Err(e) => { tracing::warn!(...); None }`. Log a structured event; `%e`
        //           uses `PlayerError`'s Display. The block's tail `None` becomes the match
        //           value (silent mode).
        // Why:      Degrade gracefully to silent mode.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // logger.warn("audio init failed:", e); output = null;
        // ```
        Err(e) => {
            tracing::warn!(error = %e, "audio init failed; degrading to silent mode");
            on_update(Update::AudioUnavailable { reason: e.to_string() });
            None
        }
    };

    // What:     `let mut controller = Controller::new(on_update, output, cache);`. Build the
    //           mutable controller state around the injected cache handle. `mut` because the
    //           loop mutates it.
    // Why:      Holds the queue, source, producer, and flags across the loop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const controller = new Controller(onUpdate, output, cache);
    // ```
    let mut controller = Controller::new(on_update, output, cache);

    // What:     `let self_sender = CommandSender { tx: self_tx, worker: thread::current() };`.
    //           Bundle the self-sender with THIS worker thread's handle. `thread::current()`
    //           on the worker thread is the same thread the run loop parks on, so a watcher
    //           send both enqueues `Rescan` and unparks us.
    // Why:      The file watcher needs a send-and-wake handle to inject `Rescan`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const selfSender = new CommandSender(selfTx, currentWorkerRef);
    // ```
    let self_sender = CommandSender {
        tx: self_tx,
        worker: thread::current(),
    };
    // What:     `controller.sweep_sink = Arc::new(move |report| sweep_sender.send(Command::SweepReport(report)));`.
    //           Route every sweep and re-analysis report back through this engine's command channel.
    // Why:      Sweep threads cannot call the update callback, but they can send a command that wakes this loop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // controller.sweepSink = (report) => selfSender.send({ kind: "sweepReport", report });
    // ```
    let sweep_sender = self_sender.clone();
    controller.sweep_sink = Arc::new(move |report| return sweep_sender.send(Command::SweepReport(report)));
    // What:     no `emit_source_root` here any more; `Command::NoLibrary` says "nothing to open" when that is true.
    // Why:      The old start-up signal said "no folder is open" before the launch had decided whether a saved folder or the
    //           music folder was about to open, so a library being read showed the "No folder is open" copy (EM1).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // nothing: the window learns about the folder from `reading` and `queue`
    // ```
    // What:     `controller.watcher = crate::watch::SourceWatcher::new(move |change| self_sender.send(...));`.
    //           Create the file watcher (or `None` if the OS watcher fails) with a change
    //           callback that enqueues `Rescan` and wakes this worker. The closure owns
    //           `self_sender`.
    // Why:      Once attached, every open/restore re-points it at the current Source Root, so
    //           on-disk changes drive live `Rescan`s; the watcher itself stays engine-agnostic.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // controller.watcher = SourceWatcher.new((change) => selfSender.send(change.kind === "everything" ? Command.Rescan : Command.SourceChanged(change)));
    // ```
    controller.watcher = crate::watch::SourceWatcher::new(move |change| {
        return self_sender.send(match change {
            crate::change_set::Change::Everything => Command::Rescan,
            paths @ crate::change_set::Change::Paths(_) => Command::SourceChanged(paths),
        });
    });

    // What:     `loop { ... }`. The main worker loop (Rust's `while (true)`); we `break` on
    //           quit.
    // Why:      Keep handling commands and feeding audio.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // while (true) { ... }
    // ```
    // What:     `let mut meter_reporter = MeterReporter::new();`. Tracks when the underrun
    //           counters were last logged, so the log shows at most one line a second.
    // Why:      Created once per worker, outside the loop, so its cadence survives iterations.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const meterReporter = new MeterReporter();
    // ```
    let mut meter_reporter = MeterReporter::new();
    loop {
        // What:     `let mut quitting = false;`. Flag set when a quit/disconnect is seen
        //           while draining commands.
        // Why:      Break the outer loop after the drain.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let quitting = false;
        // ```
        let mut quitting = false;

        // What:     `loop { ... }`. Inner loop: drain ALL queued commands without blocking,
        //           so the UI feels responsive.
        // Why:      Apply every pending command before decoding more audio.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // while (true) { const r = rx.tryRecv(); ... }
        // ```
        loop {
            // What:     `match rx.try_recv() { ... }`. Non-blocking receive: `Ok(cmd)` a
            //           command, `Err(Empty)` nothing right now, `Err(Disconnected)` all
            //           senders dropped.
            // Why:      Pull commands until the channel is momentarily empty.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // switch (rx.tryRecv()) { ... }
            // ```
            match rx.try_recv() {
                // What:     `Ok(Command::Quit) => { quitting = true; break; }`. Stop
                //           requested: flag it and leave the drain loop.
                // Why:      Begin shutdown.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // case Quit: quitting = true; break;
                // ```
                Ok(Command::Quit) => {
                    quitting = true;
                    break;
                }
                // What:     `Ok(command) => controller.handle_command(command)`. Any other
                //           command: apply it.
                // Why:      React to UI input.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // default: controller.handleCommand(command);
                // ```
                Ok(command) => controller.handle_command(command),
                // What:     `Err(TryRecvError::Empty) => break`. No more commands now.
                // Why:      Move on to pumping audio.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // case empty: break;
                // ```
                Err(TryRecvError::Empty) => break,
                // What:     `Err(TryRecvError::Disconnected) => { quitting = true; break; }`.
                //           The UI dropped the engine; shut down.
                // Why:      Nothing left to serve.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // case disconnected: quitting = true; break;
                // ```
                Err(TryRecvError::Disconnected) => {
                    quitting = true;
                    break;
                }
            }
        }

        // What:     `if quitting { controller.save_session(); break; }`. Persist the
        //           session, then leave the main loop -> `run` returns -> `controller` (and
        //           its `Output`) drops -> PipeWire stops.
        // Why:      Save where the user left off before shutting down.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (quitting) { controller.saveSession(); break; }
        // ```
        if quitting {
            controller.save_session();
            break;
        }

        // What:     `let applied_peak = controller.poll_pending_peak();`. Poll any
        //           in-flight current-track peak measurement without blocking.
        // Why:      If a measured gain landed, apply it before decoding the next audio
        //           chunk so future samples use exact normalization.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const appliedPeak = controller.pollPendingPeak();
        // ```
        let applied_peak = controller.poll_pending_peak();
        // What:     `let did_work = controller.pump_audio();`. Try to push one block of
        //           audio; returns whether anything happened.
        // Why:      Decide whether to sleep.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const didWork = controller.pumpAudio();
        // ```
        let pumped = controller.pump_audio();

        // What:     `let handed_off = controller.tick_playback();`. Emit the audible position when due and
        //           start the next track once the output consumed the last frame of the old one.
        // Why:      Position follows the speakers (frames taken from the ring), not the decoder, and the
        //           track-end handoff waits for the ring to play out instead of discarding its tail.
        //           A handoff counts as work so the new track is pumped at once.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const handedOff = controller.tickPlayback();
        // ```
        let handed_off = controller.tick_playback();
        let did_work = pumped || handed_off;

        // What:     `controller.report_audio_meters(&mut meter_reporter);`. Log the output's
        //           underrun counters if a second has passed and they moved.
        // Why:      Logging happens here, on the worker thread, never in the realtime callback.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // controller.reportAudioMeters(meterReporter);
        // ```
        controller.report_audio_meters(&mut meter_reporter);

        // What:     `if !applied_peak && !did_work { thread::park_timeout(Duration::from_millis(IDLE_PARK_FALLBACK_MS)); }`.
        //           When neither a peak swap nor audio pumping did work, PARK: block the
        //           thread (using ~0 CPU) until someone calls `unpark()` on this thread,
        //           or the timeout elapses, whichever comes first. `park_timeout` also
        //           returns immediately if an `unpark` permit was already left while we
        //           were busy, so a wake racing with this call is never lost.
        // Why:      Peak application counts as work because the next loop should observe
        //           fresh state immediately; otherwise the audio callback unparks us when
        //           it drains the ring buffer and command senders unpark us when they queue
        //           work. Blocking instead of looping is what stops the worker from pegging
        //           a CPU core; the timeout is only a safety net.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!appliedPeak && !didWork) await Promise.race([wokenSignal, sleep(IDLE_PARK_FALLBACK_MS)]);
        // ```
        if !applied_peak && !did_work {
            thread::park_timeout(controller.idle_park_timeout(Duration::from_millis(IDLE_PARK_FALLBACK_MS)));
        }
    }
}

/// What:
///      `#[cfg(test)] #[path = "engine_tests.rs"] mod tests;`.
///  Pull the end-to-end
///           integration test in from the sibling file `engine_tests.rs`;
///  test builds only.
/// Why:
///       Keep `engine.rs` to production code;
///  the live-update seam test lives beside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // engine_tests.rs is engine.integration.test.ts beside engine.ts
/// ```
#[cfg(test)]
#[path = "engine_tests.rs"]
mod tests;
