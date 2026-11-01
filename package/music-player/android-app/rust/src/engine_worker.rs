//! The engine worker thread and the AAudio realtime callback.
//!  The worker owns the
//! decode `Source`,
//!  the SPSC ring producer,
//!  and the AAudio stream,
//!  and is the only
//! thread that touches them;
//!  the callback (on AAudio's realtime thread) owns the
//! ring consumer and only pops/gates/advances.
//!  Load and seek both rebuild the
//! output through `reconfigure_output`,
//!  which is how the ring is flushed (the
//! desktop engine's mechanism).
//!
//! Mental model for a TypeScript reader:
//!  this is the audio side of an Android music
//! player written in Rust.
//!  AAudio is Android's native low-latency audio output API
//! (the counterpart to cpal/CoreAudio/WASAPI on desktop).
//!  "SPSC ring" is a
//! single-producer/single-consumer ring buffer:
//!  a fixed-size lock-free queue split
//! into a WRITE end (the producer,
//!  on this worker thread) and a READ end (the
//! consumer,
//!  on AAudio's realtime thread).
//!  The worker decodes audio in chunks and
//! pushes samples into the WRITE end;
//!  AAudio calls our callback on a realtime thread
//! whenever the speakers need more samples,
//!  and that callback pops from the READ end.
//! Because the two ends live on different threads,
//!  the queue is the only thing they
//! share,
//!  and it needs no lock.
//!
//! Realtime-thread rule (the reason the code is shaped this way):
//!  the AAudio callback
//! runs on a hard-realtime thread,
//!  so it must NEVER allocate memory,
//!  take a lock,
//!  or
//! block.
//!  All of those can stall and cause an audible glitch.
//!  That is why decoding
//! (which allocates) happens on this worker thread,
//!  and the callback only pops,
//!  scales
//! samples,
//!  and writes into a buffer AAudio handed it.

/// What:
///      `use std::os::raw::c_void;`.
///  `c_void` is Rust's stand-in for C's `void`
///           type,
///  used only inside raw-pointer types like `*mut c_void` ("a pointer
///           to memory of unknown type").
///  The `::` segments are a module path:
///  crate
///           `std`,
///  module `os`,
///  module `raw`,
///  type `c_void`.
///  Siblings you might
///           expect in `std::os::raw`:
///  `c_char`,
///  `c_int`,
///  etc. (the C primitive
///           mirrors);
///  we need only the opaque-pointer one.
/// Why:
///       AAudio's data callback hands us the speaker buffer as a raw `*mut c_void`
///           (a bare memory address with no type),
///  which we later reinterpret as `f32`
///           samples;
///  this import names that pointer's element type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // no import: think of an untyped ArrayBuffer you later view as Float32Array
/// ```
use std::os::raw::c_void;

/// What:
///      `use std::sync::atomic::Ordering;`.
///  `Ordering` is an enum describing how
///           strictly an atomic read/write is ordered against OTHER memory operations
///           on the same thread.
///  Siblings (variants) range from `Relaxed` (loosest:
///           the access is indivisible but not ordered against anything else) through
///           `Acquire`/`Release` (used to publish/observe data across threads) to
///           `SeqCst` (strictest).
///  An "atomic" operation is one the hardware performs
///           indivisibly,
///  so two threads can touch the value without a lock.
/// Why:
///       Every read/write of the shared `Control` atomics below must pass an
///           `Ordering`;
///  we pass `Acquire`/`Release` to safely hand data between the
///           worker thread and the realtime callback,
///  and `AcqRel` for read-modify-write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // no equivalent: JS Atomics don't expose a memory-ordering argument
/// ```
use std::sync::atomic::Ordering;

/// What:
///      `use std::sync::mpsc::{Receiver, TryRecvError};`.
///  `mpsc` is "multi-producer,
///           single-consumer":
///  a thread-safe channel/queue for passing values between
///           threads.
///  `Receiver<T>` is the READ end of that channel (where we pull
///           `Command`s out).
///  `TryRecvError` is the error type returned when a
///           NON-blocking receive cannot produce a value,
///  with two cases:
///  `Empty` (no
///           message right now) and `Disconnected` (every sender is gone).
///  Sibling you
///           might expect:
///  `Sender<T>`,
///  the WRITE end,
///  which lives in `engine.rs` not here.
/// Why:
///       The worker pulls playback commands (load/seek/quit) from the `Receiver`,
///           and distinguishes "nothing queued yet" from "the engine handle was dropped"
///           via the two `TryRecvError` cases.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // the read side of a cross-thread message queue, with EMPTY / DISCONNECTED states
/// ```
use std::sync::mpsc::{Receiver, TryRecvError};

/// What:
///      `use std::sync::Arc;`.
///  `Arc<T>` is an ATOMICALLY reference-counted shared
///           pointer:
///  cloning it bumps a thread-safe counter,
///  and the inner `T` is freed
///           when the last clone drops.
///  Sibling:
///  `Rc<T>`,
///  the same idea but NOT
///           thread-safe (single-thread only).
/// Why:
///       The shared `Control` block must live in two threads at once (this worker
///           and the realtime callback);
///  `Arc` lets both hold a clone of one block.
///           `Rc` would not compile because the block crosses a thread boundary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // both threads close over the same `control` object; GC handles lifetime
/// ```
use std::sync::Arc;

/// What:
///      `use std::thread;`.
///  The standard-library threading module,
///  which provides
///           `thread::sleep` (pause the current thread).
///  We import the MODULE here (and
///           write `thread::sleep` below),
///  not a specific item.
/// Why:
///       The worker naps with `thread::sleep` when there is no work,
///  to top the ring
///           up without burning a CPU core.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // no blocking sleep in TS; mentally `await new Promise(r => setTimeout(r, ms))`
/// ```
use std::thread;

/// What:
///      `use std::time::Duration;`.
///  `Duration` is a span of time (seconds plus
///           nanoseconds) used by `thread::sleep` and timers.
///  It is a value type,
///  not a
///           wall-clock instant;
///  sibling you might expect:
///  `Instant`,
///  a point in time.
/// Why:
///       We build a fixed 5-millisecond `Duration` (the idle nap) as a named const
///           below and pass it to `thread::sleep`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS uses a plain number of milliseconds instead of a Duration type
/// ```
use std::time::Duration;

/// What:
///      `use ndk::audio::{ ... };`.
///  `ndk` is the Rust binding to Android's NDK
///           (Native Development Kit).
///  This pulls in several AAudio types at once:
///           `AudioCallbackResult` (an enum the data callback returns:
///  `Continue` or
///           `Stop`),
///  `AudioDirection` (input vs output),
///  `AudioFormat` (the sample
///           format,
///  e.g. PCM float),
///  `AudioPerformanceMode` (latency-vs-power hint),
///           `AudioStream` (an open stream handle),
///  and `AudioStreamBuilder` (the
///           builder used to configure and open a stream).
/// Why:
///       We build,
///  configure,
///  open,
///  and run an AAudio output stream below,
///  which
///           needs all of these types.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { AudioCallbackResult, AudioDirection, AudioFormat, AudioPerformanceMode, AudioStream, AudioStreamBuilder } from "ndk-audio";
/// ```
use ndk::audio::{
    AudioCallbackResult, AudioDirection, AudioFormat, AudioPerformanceMode, AudioStream,
    AudioStreamBuilder,
};

/// What:
///      `use ringbuf::traits::{Consumer, Producer, Split};`.
///  These are TRAITS
///           (interfaces):
///  importing them brings their methods into scope.
///  `Split`
///           gives `.split()` (cut one ring buffer into a producer half and a consumer
///           half).
///  `Producer` gives `.push_slice(...)` (write samples into the WRITE
///           end).
///  `Consumer` gives `.pop_slice(...)` (drain samples from the READ end).
///           In Rust a trait's methods are callable only when the trait is in scope.
/// Why:
///       We split the ring buffer,
///  push from the producer on this thread,
///  and pop
///           from the consumer in the callback;
///  all three method families need their
///           trait imported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // no equivalent: importing interfaces just to unlock .split()/.pushSlice()/.popSlice()
/// ```
use ringbuf::traits::{Consumer, Producer, Split};

/// What:
///      `use ringbuf::{HeapCons, HeapProd, HeapRb};`.
///  `HeapRb<T>` is a
///           heap-allocated ring buffer (a fixed-size circular queue).
///  `.split()` yields
///           a `HeapProd<T>` (the WRITE end) and a `HeapCons<T>` (the READ end).
///  We name
///           all three because the worker holds the `HeapProd<f32>` and the callback
///           holds the `HeapCons<f32>`.
///  Both halves can live on different threads
///           (single-producer,
///  single-consumer).
/// Why:
///       A lock-free hand-off of `f32` audio samples from the decode thread to the
///           realtime audio thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a fixed-size lock-free Float32Array queue split into a writer and a reader
/// ```
use ringbuf::{HeapCons, HeapProd, HeapRb};

/// What:
///      `use symphonia::core::formats::probe::Hint;`.
///  `symphonia` is a pure-Rust
///           audio-decoding library.
///  `Hint` is a small struct that lets you HINT the
///           container/codec (e.g. a file extension) to speed up format detection;
///           empty hints just mean "figure it out from the bytes".
///  The `::` segments are
///           a module path into the crate.
/// Why:
///       `decode::open_media_source` takes a `Hint`;
///  we hand it a fresh,
///  empty hint
///           and let symphonia probe the actual bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Hint } from "symphonia";
/// ```
use symphonia::core::formats::probe::Hint;

/// What:
///      `use crate::decode::{self, Source};`.
///  `crate::` means "from the root of THIS
///           crate" (this Rust package),
///  not an external dependency.
///  This imports both
///           the `decode` MODULE itself (the `self` keyword,
///  so we can call
///           `decode::open_media_source`) and the `Source` TRAIT (the decoder interface
///           with `spec()`/`next_chunk()`/`seek()`) from inside it.
/// Why:
///       We open a decoder via `decode::open_media_source` and store it behind the
///           `Source` interface so the worker is decoder-agnostic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as decode from "./decode";
/// import type { Source } from "./decode";
/// ```
use crate::decode::{self, Source};

/// What:
///      `use crate::engine::{Command, Control, MILLIS_PER_SEC};`.
///  From this crate's
///           `engine` module:
///  `Command` is the worker's input enum (`Load`/`Seek`/`Quit`);
///           `Control` is the shared,
///  all-atomic control/telemetry block read by Kotlin
///           and written by the worker and callback;
///  `MILLIS_PER_SEC` is the constant
///           `1000.0` for the duration unit.
/// Why:
///       The worker receives `Command`s,
///  reads and writes `Control`'s atomics,
///  and
///           converts seconds to milliseconds with `MILLIS_PER_SEC`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Command, Control, MILLIS_PER_SEC } from "./engine";
/// ```
use crate::engine::{Command, Control, MILLIS_PER_SEC};

/// What:
///      `use crate::error::PlayerError;`.
///  Our one app-wide error type (a tagged
///           union of failure cases) from this crate's `error` module.
/// Why:
///       `reconfigure_output` and the AAudio helpers return `PlayerError` so the
///           `?` operator can propagate any failure uniformly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { PlayerError } from "./error";
/// ```
use crate::error::PlayerError;

/// What:
///      `const MIN_RING: usize = 8192;`.
///  A compile-time constant named `MIN_RING`
///           of type `usize`.
///  `usize` is the unsigned integer wide enough to address any
///           byte/index in memory on this platform (32 bits on a 32-bit OS,
///  64 bits on a
///           64-bit OS);
///  siblings the reader might expect:
///  `u32`,
///  `u64`,
///  `i32`,
///  `i64`.
///           `8192` is the floor capacity,
///  in samples,
///  for the ring buffer.
/// Why:
///       `usize` (not `u32`/`u64`) because this number is used as a buffer length and
///           every std collection sizing API wants `usize`;
///  mixing widths forces casts.
///           A floor keeps the ring big enough to ride out scheduling jitter even for a
///           degenerate low-rate/mono track.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MIN_RING = 8192; // minimum ring capacity in samples
/// ```
const MIN_RING: usize = 8192;

/// What:
///      `const IDLE_SLEEP: Duration = Duration::from_millis(5);`.
///  A constant named
///           `IDLE_SLEEP` of type `Duration` (a time span).
///  `Duration::from_millis(5)`
///           is an "associated function" (a static/factory method on the `Duration`
///           type,
///  reached with `::`) that builds a 5-millisecond span.
/// Why:
///       The nap length when the ring is full or nothing is loaded:
///  short enough to
///           keep the ~1s buffer topped up,
///  long enough not to spin a CPU core.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const IDLE_SLEEP = 5; // milliseconds; mentally a Duration
/// ```
const IDLE_SLEEP: Duration = Duration::from_millis(5);

/// What:
///      `struct WorkerState { ... }` declares a record type holding everything the
///           worker thread owns across iterations.
///  Fields:
///           - `source: Option<Box<dyn Source>>`.
///  `Box<dyn Source>` is an OWNING pointer
///             to a heap-allocated value that implements the `Source` trait,
///  where the
///             exact decoder type is erased ("dyn" = dynamic dispatch).
///  `Option<T>` is
///             Rust's "value or nothing" (it has no `null`);
///  `Some(x)` carries a value,
///             `None` is empty.
///  Siblings of `Box<T>`:
///  `Rc<T>`/`Arc<T>` (shared,
///  refcounted).
///           - `stream: Option<AudioStream>`.
///  The current AAudio stream,
///  owned,
///  or `None`.
///           - `prod: Option<HeapProd<f32>>`.
///  The ring's WRITE end,
///  owned,
///  or `None`.
///           - `pending: Vec<f32>`.
///  A growable,
///  heap-allocated,
///  OWNED array of `f32`
///             samples.
///  Siblings:
///  `&[f32]` (a borrowed view) and `[f32; N]` (a fixed-size
///             stack array);
///  we use `Vec` because the length varies per chunk and we own it.
///           - `pending_pos: usize`.
///  A read cursor (an index) into `pending`.
/// Why:
///       Keeping one struct lets the worker carry the decoder,
///  output stream,
///  ring
///           producer,
///  and the leftover-samples carryover across loop iterations.
///  `Box`
///           (not `Rc`/`Arc`) because only this thread ever touches the source.
///  `Vec`
///           (not `&[f32]`) because the carryover must outlive the decode call that
///           produced it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class WorkerState {
///   source: Source | null = null;
///   stream: AudioStream | null = null;
///   prod: RingProducer | null = null;
///   pending: number[] = [];
///   pendingPos = 0;
/// }
/// ```
struct WorkerState {
    /// What:
    ///      `source: Option<Box<dyn Source>>`.
    ///  The decoder for the loaded track,
    ///  or
    ///           `None` when nothing is loaded.
    ///  `Box<dyn Source>` is an owning heap
    ///           pointer behind the `Source` interface (the concrete decoder type is
    ///           hidden).
    ///  The backing file is a `dup`-ed `content://` Android file
    ///           descriptor.
    ///  Sibling you might expect:
    ///  `&dyn Source` (a borrow) but we OWN
    ///           it so it lives as long as the track is loaded.
    /// Why:
    ///       The worker pulls chunks from this each pump;
    ///  wrapping in `Option` lets
    ///           "nothing loaded" be a first-class state rather than a null pointer.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// source: Source | null;
    /// ```
    source: Option<Box<dyn Source>>,
    /// What:
    ///      `stream: Option<AudioStream>`.
    ///  The current AAudio output stream,
    ///  owned,
    ///           or `None` before the first track.
    ///  Dropping the `AudioStream` value closes
    ///           the stream and stops the realtime callback firing.
    /// Why:
    ///       We rebuild this per track (load/seek),
    ///  and storing it in `Option` lets us
    ///           set it to `None` to tear the old one down before opening a new one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// stream: AudioStream | null;
    /// ```
    stream: Option<AudioStream>,
    /// What:
    ///      `prod: Option<HeapProd<f32>>`.
    ///  The ring buffer's WRITE end for `f32`
    ///           samples,
    ///  owned,
    ///  or `None`.
    ///  The decode pump pushes into this.
    /// Why:
    ///       Recreated per track alongside the stream;
    ///  `Option` models the "no ring
    ///           yet" state.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// prod: RingProducer | null;
    /// ```
    prod: Option<HeapProd<f32>>,
    /// What:
    ///      `pending: Vec<f32>`.
    ///  An OWNED,
    ///  growable array of leftover decoded samples
    ///           that a full ring could not accept yet (backpressure carryover).
    ///  `Vec<f32>`
    ///           (not `&[f32]` borrowed,
    ///  not `[f32; N]` fixed-size) because the length
    ///           varies and the worker owns these bytes until they are pushed.
    /// Why:
    ///       When the ring is full mid-chunk,
    ///  we stash the unpushed tail here and
    ///           retry next pump instead of dropping or re-decoding audio.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// pending: number[];
    /// ```
    pending: Vec<f32>,
    /// What:
    ///      `pending_pos: usize`.
    ///  A read cursor (index) into `pending`:
    ///  how many of
    ///           its samples have already been pushed into the ring.
    ///  `usize` because it is
    ///           an array index (siblings `u32`/`u64`/`i32`;
    ///  std indexing wants `usize`).
    /// Why:
    ///       Lets us resume pushing the carryover from where we left off without
    ///           reallocating or shifting the `Vec`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// pendingPos: number;
    /// ```
    pending_pos: usize,
    /// What:
    ///      `loads_handled: u64`.
    ///  How many `Command::Load` requests the worker has started handling,
    ///           counting the one in progress.
    /// Why:
    ///       Kotlin counts the loads it queues; this count names the load a failure belongs to,
    ///  because the
    ///           worker handles loads in the order they were queued.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// loadsHandled: number;
    /// ```
    loads_handled: u64,
}

/// What:
///      `impl WorkerState { ... }`.
///  An `impl` block attaches methods/associated
///           functions to the `WorkerState` type.
///  Here it holds just the constructor.
/// Why:
///       Group the "how to build a fresh WorkerState" function with the type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class WorkerState { /* methods, e.g. a constructor, go here */ }
/// ```
impl WorkerState {
    /// What:
    ///      `fn new() -> WorkerState`.
    ///  An ASSOCIATED function (no `self` parameter,
    ///           so it is called as `WorkerState::new()`,
    ///  like a static factory) that
    ///           returns a fresh,
    ///  empty `WorkerState`.
    /// Why:
    ///       One place that defines the "nothing loaded" starting state.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static create(): WorkerState { return new WorkerState(); }
    /// ```
    fn new() -> WorkerState {
        // What:     `WorkerState { source: None, stream: None, prod: None, pending: Vec::new(), pending_pos: 0 }`.
        //           Construct the struct. `None` is the empty case of `Option` for the
        //           three owning slots. `Vec::new()` is an associated function (reached
        //           with `::`) that makes an empty, zero-capacity growable array.
        //           `pending_pos: 0` starts the cursor at the front. There is no trailing
        //           `;`, so this struct literal is the function's TAIL EXPRESSION and
        //           therefore its return value.
        // Why:      Hand back a worker state with nothing loaded and an empty carryover.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { source: null, stream: null, prod: null, pending: [], pendingPos: 0 };
        // ```
        return WorkerState {
            source: None,
            stream: None,
            prod: None,
            pending: Vec::new(),
            pending_pos: 0,
            loads_handled: 0,
        }
    }
}

/// What:
///      `pub(crate) fn worker_run(rx: Receiver<Command>, control: Arc<Control>)`.
///           `pub(crate)` means "visible everywhere inside THIS crate,
///  but not exported
///           to other crates" (a narrower visibility than plain `pub`).
///  This is the
///           worker thread's entry point.
///  `rx: Receiver<Command>` is the READ end of the
///           command channel,
///  taken BY VALUE (the worker now owns it).
///  `control: Arc<Control>`
///           is a shared,
///  refcounted handle to the control block,
///  also taken by value
///           (the caller handed us our own clone).
///  The function returns nothing (no `->`).
/// Why:
///       This runs on the dedicated worker thread:
///  it drains commands,
///  decodes audio
///           into the ring,
///  and naps when idle.
///  Returning ends the thread and drops the
///           AAudio stream (stopping audio).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function workerRun(rx: Receiver<Command>, control: Control): void { ... }
/// ```
pub(crate) fn worker_run(rx: Receiver<Command>, control: Arc<Control>) {
    // What:     `let mut state = WorkerState::new();`. Declare a LOCAL variable `state`
    //           and call the `WorkerState::new()` associated function to fill it. `mut`
    //           marks it MUTABLE (Rust variables are read-only by default); we need it
    //           because every command/pump mutates `state`'s fields.
    // Why:      Hold the worker's owned playback state for the whole thread lifetime.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let state = WorkerState.create();
    // ```
    let mut state = WorkerState::new();
    // What:     `loop { ... }`. Rust's infinite loop (equivalent to `while (true)`). It
    //           runs until something inside `return`s out of the function.
    // Why:      The worker runs forever, one iteration per "drain commands, pump, maybe
    //           nap" cycle, until told to quit or the channel dies.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // while (true) { ... }
    // ```
    loop {
        // What:     `loop { ... }`. An INNER infinite loop that drains all currently
        //           queued commands before doing any decode work. It exits via `break`
        //           (out of this inner loop only) or `return` (out of the whole function).
        // Why:      Process every pending command in one batch so state is up to date
        //           before we pump audio.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // while (true) { /* drain all queued commands */ }
        // ```
        loop {
            // What:     `match rx.try_recv() { ... }`. `rx.try_recv()` is a NON-blocking
            //           receive: it returns `Result<Command, TryRecvError>` immediately,
            //           either `Ok(command)` if one is queued or `Err(...)` if not.
            //           `match` then inspects that result and runs the matching arm,
            //           binding the inner value. This is how Rust unpacks success/failure
            //           without exceptions.
            // Why:      Pull the next command if there is one, otherwise decide whether to
            //           stop draining (empty) or end the thread (disconnected).
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const res = rx.tryRecv(); // { ok: true, value } | { ok: false, error }
            // ```
            match rx.try_recv() {
                // What:     `Ok(Command::Load(file, play)) => handle_load(&mut state, &control, file, play)`.
                //           One match arm. `Ok(...)` is the success case of `Result`;
                //           inside it `Command::Load(file, play)` matches the `Load`
                //           variant of the `Command` enum and binds its two payloads:
                //           `file` (a `std::fs::File`, moved out) and `play` (a `bool`).
                //           The arm calls `handle_load`, passing `&mut state` (an
                //           EXCLUSIVE borrow: lending `state` mutably) and `&control` (a
                //           SHARED read-only borrow).
                // Why:      A `Load` command means "open this file and (maybe) start
                //           playing"; we delegate to `handle_load`.
                // Gotcha:   `&mut state` lends `state` to `handle_load` mutably; while that
                //           borrow is active no other code may touch `state`. `file` is
                //           MOVED into `handle_load` (the channel no longer owns it).
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (cmd.kind === "load") handleLoad(state, control, cmd.file, cmd.play);
                // ```
                Ok(Command::Load(file, play)) => handle_load(&mut state, &control, file, play),
                // What:     `Ok(Command::Seek(position_sec)) => handle_seek(&mut state, &control, position_sec)`.
                //           Match the `Seek` variant inside `Ok`, binding its single `f64`
                //           payload (seconds) to `position_sec`. `f64` is a 64-bit
                //           floating-point number (sibling `f32`, the 32-bit float); we
                //           use `f64` for a precise seek position. Calls `handle_seek` with
                //           the same `&mut state` / `&control` borrows.
                // Why:      A `Seek` command repositions the loaded track.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (cmd.kind === "seek") handleSeek(state, control, cmd.positionSec);
                // ```
                Ok(Command::Seek(position_sec)) => handle_seek(&mut state, &control, position_sec),
                // What:     `Ok(Command::Quit) => return`. Match the `Quit` variant (no
                //           payload) inside `Ok`. The arm is the bare keyword `return`,
                //           which exits `worker_run` entirely (and thus ends the thread).
                // Why:      A `Quit` command stops the worker; returning drops `state`,
                //           which drops the `AudioStream`, which stops audio.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (cmd.kind === "quit") return;
                // ```
                Ok(Command::Quit) => return,
                // What:     `Err(TryRecvError::Empty) => break`. Match the failure case
                //           `Err(...)` whose inner error is `TryRecvError::Empty` (the
                //           queue is momentarily empty). `break` exits the INNER drain
                //           loop only, not the outer worker loop.
                // Why:      No more commands queued right now, so stop draining and go pump
                //           audio.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (e === "empty") break; // nothing queued; go decode
                // ```
                Err(TryRecvError::Empty) => break,
                // What:     `Err(TryRecvError::Disconnected) => return`. Match the failure
                //           case whose inner error is `Disconnected` (every `Sender` was
                //           dropped, so no command can ever arrive again). The arm
                //           `return`s out of the whole function.
                // Why:      The engine handle is gone; shut the worker down (and its stream).
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (e === "disconnected") return; // engine dropped; end thread
                // ```
                Err(TryRecvError::Disconnected) => return,
            }
        }
        // What:     `let did_work = if state.source.is_some() { pump(&mut state, &control) } else { false };`.
        //           An `if`/`else` used as an EXPRESSION (its value is assigned to
        //           `did_work`). `state.source.is_some()` returns `true` when the `Option`
        //           holds a value (a track is loaded). When true we call `pump(...)`
        //           (passing `&mut state` exclusive borrow and `&control` shared borrow),
        //           whose `bool` result becomes `did_work`; otherwise `did_work` is
        //           `false`. Both arms must yield the same type (`bool`).
        // Why:      Only decode when something is loaded; otherwise there is no work, so
        //           we will nap.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const didWork = state.source !== null ? pump(state, control) : false;
        // ```
        let did_work = if state.source.is_some() {
            pump(&mut state, &control)
        } else {
            false
        };
        // What:     `if !did_work { thread::sleep(IDLE_SLEEP); }`. `!did_work` is logical
        //           NOT (the `!` operator) on the `bool`. When no work happened, call
        //           `thread::sleep(IDLE_SLEEP)` to block this thread for the 5ms `Duration`.
        // Why:      Avoid busy-looping a CPU core when the ring is full or nothing is
        //           loaded; nap briefly, then re-check.
        // Gotcha:   `thread::sleep` BLOCKS the whole thread synchronously; it is NOT TS's
        //           async `setTimeout`. Nothing else runs on this thread during the nap.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!didWork) await new Promise(r => setTimeout(r, IDLE_SLEEP));
        // ```
        if !did_work {
            thread::sleep(IDLE_SLEEP);
        }
    }
}

/// What:
///      `fn handle_load(state: &mut WorkerState, control: &Arc<Control>, file: std::fs::File, play: bool)`.
///           A private function (no `pub`).
///  `state: &mut WorkerState` is an EXCLUSIVE
///           borrow (we mutate the worker state).
///  `control: &Arc<Control>` is a SHARED
///           read-only borrow of the refcounted control handle.
///  `file: std::fs::File` is
///           taken BY VALUE (the worker now owns the open file).
///  `play: bool` says whether
///           to start playing once loaded.
/// Why:
///       Open the file,
///  reset telemetry,
///  build the output,
///  and set the play gate.
///  Any
///           failure leaves the engine idle (no source,
///  silent).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function handleLoad(state: WorkerState, control: Control, file: File, play: boolean): void { ... }
/// ```
fn handle_load(state: &mut WorkerState, control: &Arc<Control>, file: std::fs::File, play: bool) {
    // What:     `state.loads_handled += 1;`. Count this load before any early return below.
    // Why:      A failure names its load by this number, and Kotlin counts queued loads the same way.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // state.loadsHandled += 1;
    // ```
    state.loads_handled += 1;
    // What:     `state.stream = None;`. Overwrite the stream field with the empty `Option`.
    //           Assigning `None` DROPS the previous `AudioStream`, which closes it and
    //           stops its callback.
    // Why:      Tear down any prior track's output before building a new one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // state.stream = null;
    // ```
    state.stream = None;
    // What:     `state.prod = None;`. Drop the previous ring producer by setting the field
    //           to the empty `Option`.
    // Why:      Discard the old ring's WRITE end so the next reconfigure installs a fresh one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // state.prod = null;
    // ```
    state.prod = None;
    // What:     `state.source = None;`. Drop the previous decoder by setting the field to
    //           the empty `Option`.
    // Why:      Forget any previously loaded track before opening the new one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // state.source = null;
    // ```
    state.source = None;
    // What:     `state.pending.clear();`. `.clear()` empties the `Vec<f32>` in place
    //           (length to 0, keeping its allocation).
    // Why:      Drop any leftover carryover samples from the previous track.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // state.pending.length = 0;
    // ```
    state.pending.clear();
    // What:     `state.pending_pos = 0;`. Reset the carryover read cursor to the front.
    //           This is a plain integer assignment with a direct TS analogue.
    // Why:      With `pending` emptied, the cursor must start at 0.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // state.pendingPos = 0;
    // ```
    state.pending_pos = 0;
    // What:     `let source = match decode::open_media_source(Box::new(file), Hint::new()) { ... };`.
    //           `decode::open_media_source(...)` returns `Result<Box<dyn Source>, PlayerError>`.
    //           `Box::new(file)` heap-allocates the owned `File` and yields a `Box`
    //           (owning pointer), needed because the function wants a boxed media source.
    //           `Hint::new()` builds an empty format hint. `match` then unpacks the
    //           `Result` into either the opened source or an early return.
    // Why:      Try to open a decoder for the file; on success keep the source, on failure
    //           bail out leaving the engine idle.
    // Gotcha:   `Box::new(file)` MOVES `file` onto the heap; the local `file` is consumed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let source;
    // try { source = decode.openMediaSource(file, new Hint()); } catch { return; }
    // ```
    let source = match decode::open_media_source(Box::new(file), Hint::new()) {
        // What:     `Ok(source) => source`. The success arm: unwrap the `Ok` and bind the
        //           inner `Box<dyn Source>` to `source`. The bare `source` is the arm's
        //           value, which (assigned by the surrounding `let`) becomes the decoder.
        // Why:      Use the opened decoder.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // source = the opened decoder
        // ```
        Ok(source) => source,
        // What:     `Err(_) => { fail_load(state, control); return; }`. The failure arm. `Err(_)` matches any
        //           error; the `_` is a wildcard that DISCARDS the error value (we don't inspect it).
        //           `fail_load` resets the track facts and publishes this load's number as the failed one,
        //           then `return` exits `handle_load` early, leaving the engine idle.
        // Why:      If the file cannot be decoded, abandon the load, but let Kotlin report it: the error value
        //           carries nothing the report needs.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // on any error: failLoad(state, control); return;
        // ```
        Err(_) => {
            fail_load(state, control);
            return;
        }
    };
    // What:     `let spec = source.spec();`. Call the `spec()` method on the decoder; it
    //           returns an `AudioSpec` value (a small COPY containing `rate: u32`,
    //           `channels: u16`, `duration_secs: f64`).
    // Why:      We need the track's sample rate, channel count, and duration to size the
    //           ring and publish telemetry.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const spec = source.spec();
    // ```
    let spec = source.spec();
    // What:     `if spec.rate == 0 || spec.channels == 0 { fail_load(state, control); return; }`. Guard: `||` is
    //           logical OR. If either the sample rate or channel count is zero (an invalid
    //           or empty track), `return` early. These comparisons and `||` are
    //           character-identical to TS.
    // Why:      A zero rate or zero channels is unplayable; bail before building output.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (spec.rate === 0 || spec.channels === 0) return;
    // ```
    if spec.rate == 0 || spec.channels == 0 {
        fail_load(state, control);
        return;
    }
    // What:     `control.rate.store(spec.rate, Ordering::Release);`. `control.rate` is an
    //           atomic integer; `.store(value, ordering)` is the atomic WRITE. `Ordering::Release`
    //           means "make this write (and everything before it) visible to a thread that
    //           later reads with `Acquire`"; it publishes the value to the realtime callback.
    // Why:      Tell readers (Kotlin via JNI, and the callback) the new track's sample rate.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // Atomics.store(control.rate, 0, spec.rate);
    // ```
    control.rate.store(spec.rate, Ordering::Release);
    // What:     `control.channels.store(spec.channels as u32, Ordering::Release);`.
    //           `spec.channels as u32` is a primitive numeric CAST widening the `u16`
    //           channel count to `u32` (the atomic's element type). `.store(...)` then
    //           atomically writes it with `Release` ordering.
    // Why:      Publish the channel count to readers; the `as u32` matches the atomic's width.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // Atomics.store(control.channels, 0, spec.channels);
    // ```
    control.channels.store(spec.channels as u32, Ordering::Release);
    // What:     `control.start_frame.store(0, Ordering::Release);`. Atomically write `0`
    //           into the `start_frame` telemetry (the frame the current stream begins at),
    //           with `Release` ordering.
    // Why:      A fresh load starts at frame 0 (no seek offset yet).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // Atomics.store(control.startFrame, 0, 0);
    // ```
    control.start_frame.store(0, Ordering::Release);
    // What:     `control.frames_played.store(0, Ordering::Release);`. Atomically reset the
    //           played-frame counter to `0` with `Release` ordering.
    // Why:      A new track has played nothing yet; the position should read 0.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // Atomics.store(control.framesPlayed, 0, 0);
    // ```
    control.frames_played.store(0, Ordering::Release);
    // What:     `control.duration_ms.store((spec.duration_secs * MILLIS_PER_SEC) as u64, Ordering::Release);`.
    //           Compute milliseconds: `spec.duration_secs` (an `f64`) times `MILLIS_PER_SEC`
    //           (`1000.0`), then `as u64` CASTS the resulting float to a 64-bit unsigned
    //           integer (truncating the fraction). `.store(...)` writes it atomically with
    //           `Release`. The multi-line `.store(...)` call is split across lines purely
    //           for width.
    // Why:      Publish the track length in the integer-millisecond unit Kotlin reads.
    // Gotcha:   `as u64` on a float TRUNCATES toward zero (no rounding) and saturates at the
    //           integer bounds; it is not TS's `Number()` behaviour.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // Atomics.store(control.durationMs, 0, Math.trunc(spec.durationSecs * MILLIS_PER_SEC));
    // ```
    control
        .duration_ms
        .store((spec.duration_secs * MILLIS_PER_SEC) as u64, Ordering::Release);
    // What:     `control.decode_done.store(false, Ordering::Release);`. Atomically write
    //           `false` into the `decode_done` flag (an atomic bool) with `Release` ordering.
    // Why:      The new track has not finished decoding; clear any leftover "done" flag.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // Atomics.store(control.decodeDone, 0, 0 /* false */);
    // ```
    control.decode_done.store(false, Ordering::Release);
    // What:     `control.ended.store(false, Ordering::Release);`. Atomically write `false`
    //           into the `ended` flag with `Release` ordering.
    // Why:      The new track has not ended; clear any leftover "ended" flag.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // Atomics.store(control.ended, 0, 0 /* false */);
    // ```
    control.ended.store(false, Ordering::Release);
    // Set the gate before the stream opens, so the new stream's first callback reads the right play
    // state (a load-paused track must not briefly sound at the previous track's state).
    // What:     `control.playing.store(play, Ordering::Release);`. Atomically write the
    //           `play` bool into the shared play gate with `Release` ordering, BEFORE the
    //           stream is built below.
    // Why:      So the new stream's very first callback observes the intended play/pause
    //           state and a load-paused track does not briefly sound.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // Atomics.store(control.playing, 0, play ? 1 : 0);
    // ```
    control.playing.store(play, Ordering::Release);
    // What:     `state.source = Some(source);`. Store the opened decoder into the field,
    //           wrapping it in `Some` (the "has a value" case of `Option`). Moving it in
    //           transfers ownership to `state`.
    // Why:      Mark the track loaded so the pump can decode from it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // state.source = source;
    // ```
    state.source = Some(source);
    // What:     `if reconfigure_output(state, control, spec.rate, spec.channels).is_err() { ... }`.
    //           Call `reconfigure_output(...)`, which returns `Result<(), PlayerError>`.
    //           `.is_err()` returns `true` when that result is the failure variant `Err`.
    //           `spec.rate`/`spec.channels` pass the track's native format. When it failed,
    //           run the recovery block below.
    // Why:      Build the ring + AAudio stream; if that fails, undo the load so we stay idle.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { reconfigureOutput(state, control, spec.rate, spec.channels); }
    // catch { /* recovery below */ }
    // ```
    if reconfigure_output(state, control, spec.rate, spec.channels).is_err() {
        // What:     `fail_load(state, control);`. On output-build failure, force the play gate to `false`,
        //           reset the track facts and report this load as failed.
        // Why:      Nothing can play if the output failed; without the reset the deck keeps the previous track's
        //           duration, and without the report Kotlin never learns the load failed.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // failLoad(state, control);
        // ```
        fail_load(state, control);
        // What:     `state.source = None;`. Drop the just-loaded decoder by setting the
        //           field back to the empty `Option`.
        // Why:      Undo the load so the engine is fully idle (no source, silent) after the
        //           output failure.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // state.source = null;
        // ```
        state.source = None;
    }
}

/// What:
///      `fn fail_load(state: &WorkerState, control: &Arc<Control>)`.
///  Resets every track fact the previous track left in `control` (silent,
///           not ended,
///  no rate,
///  no channels,
///  no length,
///  no position) and then publishes this load's
///           sequence number as the failed one.
/// Why:
///       A load whose file opens but does not decode used to return with the
///           previous track's duration and play state still in `control`,
///  and nothing told Kotlin;
///  the deck then
///           showed a stale length and playback stalled in silence.
///  The sequence number is stored last so a reader
///           that sees it also sees the reset.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function failLoad(state: WorkerState, control: Control): void { ... }
/// ```
fn fail_load(state: &WorkerState, control: &Arc<Control>) {
    control.playing.store(false, Ordering::Release);
    control.ended.store(false, Ordering::Release);
    control.decode_done.store(false, Ordering::Release);
    control.rate.store(0, Ordering::Release);
    control.channels.store(0, Ordering::Release);
    control.start_frame.store(0, Ordering::Release);
    control.frames_played.store(0, Ordering::Release);
    control.duration_ms.store(0, Ordering::Release);
    control.failed_load.store(state.loads_handled, Ordering::Release);
}

/// What:
///      `fn handle_seek(state: &mut WorkerState, control: &Arc<Control>, position_sec: f64)`.
///           A private function.
///  `&mut WorkerState` is an exclusive borrow (we mutate it);
///           `&Arc<Control>` a shared read-only borrow;
///  `position_sec: f64` is the target
///           position in seconds (a 64-bit float;
///  sibling `f32` would lose seek precision).
/// Why:
///       Reposition the loaded source and rebuild the output (which flushes the ring).
///           The play gate is left untouched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function handleSeek(state: WorkerState, control: Control, positionSec: number): void { ... }
/// ```
fn handle_seek(state: &mut WorkerState, control: &Arc<Control>, position_sec: f64) {
    // What:     `let (rate, channels) = match state.source.as_mut() { ... };`. The left
    //           side `(rate, channels)` is TUPLE DESTRUCTURING: the `match` yields a
    //           `(u32, u16)` pair and binds each element to a name. `state.source.as_mut()`
    //           turns `&mut Option<Box<dyn Source>>` into `Option<&mut Box<dyn Source>>`
    //           (a mutable borrow of the inside, WITHOUT moving the source out), so we can
    //           call mutating methods on the decoder while it stays in `state`.
    // Why:      Get a mutable handle to the decoder if one is loaded, and read back its
    //           post-seek rate/channels; if nothing is loaded, return early.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let rate: number, channels: number;
    // if (state.source === null) return;
    // ```
    let (rate, channels) = match state.source.as_mut() {
        // What:     `Some(source) => { ... }`. The present-value arm: unwrap `Some` and
        //           bind the inner `&mut Box<dyn Source>` to `source`. The block then
        //           seeks and reads the spec, and its final expression is the arm's value.
        // Why:      We have a loaded decoder; seek it and compute the pair to return.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // { const source = state.source; /* seek + read spec */ }
        // ```
        Some(source) => {
            // What:     `let _ = source.seek(position_sec);`. Call the decoder's `seek`,
            //           which returns `Result<(), PlayerError>`. `let _ =` BINDS the result
            //           to the wildcard `_`, which DISCARDS it (and suppresses the
            //           "unused Result" warning) without inspecting success or failure.
            // Why:      Attempt the seek but tolerate a failure (we will rebuild output from
            //           the current position regardless).
            // Gotcha:   `let _ = expr;` intentionally throws the value away; here it also
            //           silences Rust's must-use warning on the ignored `Result`.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // try { source.seek(positionSec); } catch { /* ignore */ }
            // ```
            let _ = source.seek(position_sec);
            // What:     `let spec = source.spec();`. Read the decoder's `AudioSpec` (a COPY
            //           with `rate`/`channels`/`duration_secs`) AFTER seeking.
            // Why:      The seek may have re-derived the format; we need the current
            //           rate/channels to rebuild the ring.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const spec = source.spec();
            // ```
            let spec = source.spec();
            // What:     `(spec.rate, spec.channels)`. A TUPLE literal `(a, b)` pairing the
            //           `u32` rate and `u16` channel count. No trailing `;`, so it is the
            //           block's TAIL EXPRESSION, hence this arm's value, hence what the
            //           outer `(rate, channels)` destructures.
            // Why:      Hand the post-seek format back out of the `match`.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // value of this branch: [spec.rate, spec.channels]
            // ```
            (spec.rate, spec.channels)
        }
        // What:     `None => return`. The empty arm: when no source is loaded, `return`
        //           exits `handle_seek` entirely (there is nothing to seek).
        // Why:      A seek with nothing loaded is a no-op.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // if no source: return;
        // ```
        None => return,
    };
    // What:     `if rate == 0 || channels == 0 { return; }`. Guard against an invalid
    //           post-seek format (zero rate or zero channels). `||` is logical OR; both
    //           comparisons mirror TS exactly.
    // Why:      An unplayable format after seeking should abort the rebuild.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (rate === 0 || channels === 0) return;
    // ```
    if rate == 0 || channels == 0 {
        return;
    }
    // What:     `state.pending.clear();`. Empty the carryover `Vec<f32>` in place.
    // Why:      Pre-seek leftover samples are stale; drop them so only post-seek audio plays.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // state.pending.length = 0;
    // ```
    state.pending.clear();
    // What:     `state.pending_pos = 0;`. Reset the carryover cursor to the front (plain
    //           integer assignment).
    // Why:      With `pending` cleared, the cursor must restart at 0.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // state.pendingPos = 0;
    // ```
    state.pending_pos = 0;
    // What:     `control.decode_done.store(false, Ordering::Release);`. Atomically clear the
    //           `decode_done` flag with `Release` ordering.
    // Why:      After seeking there is more to decode, even if we had hit EOF before.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // Atomics.store(control.decodeDone, 0, 0 /* false */);
    // ```
    control.decode_done.store(false, Ordering::Release);
    // What:     `control.ended.store(false, Ordering::Release);`. Atomically clear the
    //           `ended` flag with `Release` ordering.
    // Why:      A seek un-ends a track that had finished.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // Atomics.store(control.ended, 0, 0 /* false */);
    // ```
    control.ended.store(false, Ordering::Release);
    // What:     `let clamped = if position_sec > 0.0 { position_sec } else { 0.0 };`. An
    //           `if`/`else` used as an expression: when the requested position is positive,
    //           use it; otherwise use `0.0`. `0.0` is an `f64` literal. Both arms yield `f64`.
    // Why:      Never store a negative start position; clamp it up to zero.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const clamped = positionSec > 0 ? positionSec : 0;
    // ```
    let clamped = if position_sec > 0.0 { position_sec } else { 0.0 };
    // What:     `control.start_frame.store((clamped * rate as f64).round() as u64, Ordering::Release);`.
    //           Convert the seek position (seconds) into a frame index: `rate as f64` CASTS
    //           the `u32` rate to `f64` so the multiply is in floating point; `.round()`
    //           rounds to the nearest whole frame; `as u64` then CASTS that float to a
    //           64-bit unsigned integer. `.store(...)` writes it atomically with `Release`.
    // Why:      The position counter is in frames; publish where the new stream begins so
    //           the reported position includes the seek offset.
    // Gotcha:   `.round()` here is float rounding (nearest, ties away from zero), THEN
    //           `as u64` truncates/saturates; chain order matters.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // Atomics.store(control.startFrame, 0, Math.round(clamped * rate));
    // ```
    control
        .start_frame
        .store((clamped * rate as f64).round() as u64, Ordering::Release);
    // What:     `control.frames_played.store(0, Ordering::Release);`. Atomically reset the
    //           played-frame counter to `0` with `Release`.
    // Why:      The new (post-seek) stream has played nothing yet; the offset lives in
    //           `start_frame`, so this counts only frames since the seek.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // Atomics.store(control.framesPlayed, 0, 0);
    // ```
    control.frames_played.store(0, Ordering::Release);
    // What:     `let _ = reconfigure_output(state, control, rate, channels);`. Rebuild the
    //           ring + AAudio stream at the post-seek format. `reconfigure_output` returns
    //           `Result<(), PlayerError>`; `let _ =` DISCARDS that result (and silences the
    //           must-use warning) because a seek failure simply leaves playback where it was.
    // Why:      Flush the ring and reopen output so post-seek audio plays; tolerate failure
    //           without tearing the track down.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { reconfigureOutput(state, control, rate, channels); } catch { /* ignore */ }
    // ```
    let _ = reconfigure_output(state, control, rate, channels);
}

/// What:
///      `fn reconfigure_output(state: &mut WorkerState, control: &Arc<Control>, rate: u32, channels: u16) -> Result<(), PlayerError>`.
///           A private function.
///  `&mut WorkerState` exclusive borrow (we replace its
///           stream/prod).
///  `&Arc<Control>` shared borrow.
///  `rate: u32` the sample rate;
///           `channels: u16` the channel count (a small unsigned count;
///  siblings `u8`/`u32`).
///           Returns `Result<(), PlayerError>`:
///  success carries the empty unit `()` (like
///           `void`),
///  failure carries our error.
///  The signature is split across lines for width.
/// Why:
///       Drop the old output and build a fresh ring + AAudio stream at the track's rate,
///           moving the new consumer into the stream's data callback.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reconfigureOutput(state: WorkerState, control: Control, rate: number, channels: number): void { ... }
/// ```
fn reconfigure_output(
    state: &mut WorkerState,
    control: &Arc<Control>,
    rate: u32,
    channels: u16,
) -> Result<(), PlayerError> {
    // What:     `state.stream = None;`. Drop the previous AAudio stream by assigning the
    //           empty `Option`. Dropping the stream closes it and stops its callback.
    // Why:      Stop the old track's output before opening the new stream, freeing the old
    //           consumer the callback held.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // state.stream = null;
    // ```
    state.stream = None;
    // What:     `state.prod = None;`. Drop the previous ring producer (the old WRITE end).
    // Why:      We are about to build a brand-new ring; discard the stale producer first.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // state.prod = null;
    // ```
    state.prod = None;
    // What:     `let capacity = ((rate as usize) * (channels as usize)).max(MIN_RING);`.
    //           `rate as usize` and `channels as usize` CAST the `u32`/`u16` to `usize`
    //           (pointer-wide) so the multiply produces a `usize` length. `.max(MIN_RING)`
    //           returns whichever is larger, the product or the 8192-sample floor.
    // Why:      Size the ring to about one second of interleaved audio (rate times
    //           channels), but never below `MIN_RING` for degenerate low-rate/mono tracks.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const capacity = Math.max(rate * channels, MIN_RING);
    // ```
    let capacity = ((rate as usize) * (channels as usize)).max(MIN_RING);
    // What:     `let ring = HeapRb::<f32>::new(capacity);`. Allocate a heap ring buffer of
    //           `capacity` `f32` slots. `::<f32>` is the "turbofish" syntax that pins the
    //           element type, and `::new(...)` is the associated constructor.
    // Why:      The shared lock-free queue between this decode thread and the audio thread.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const ring = new RingBuffer<number>(capacity);
    // ```
    let ring = HeapRb::<f32>::new(capacity);
    // What:     `let (prod, mut cons) = ring.split();`. `ring.split()` CONSUMES the ring and
    //           returns its two halves, TUPLE-DESTRUCTURED into `prod` (the `HeapProd<f32>`
    //           write end) and `cons` (the `HeapCons<f32>` read end). `mut cons` because the
    //           callback will mutate it (popping advances its read cursor).
    // Why:      `prod` stays with the worker to push samples; `cons` is moved into the
    //           callback below to pop them.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const { prod, cons } = ring.split();
    // ```
    let (prod, mut cons) = ring.split();
    // What:     `state.prod = Some(prod);`. Store the WRITE end in the field, wrapped in
    //           `Some`. Moving it in transfers ownership to `state`.
    // Why:      The pump pushes decoded samples through `state.prod`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // state.prod = prod;
    // ```
    state.prod = Some(prod);
    // What:     `let callback_control = Arc::clone(control);`. `Arc::clone(control)` makes
    //           ANOTHER refcounted handle to the SAME shared `Control` block (bumps the
    //           atomic reference count; does not copy the block). `control` is already a
    //           `&Arc<Control>`, so this clones through the borrow.
    // Why:      The realtime callback (built next) must read/write the control atomics, so
    //           it needs its own owned handle that it can keep after this function returns.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const callbackControl = control; // same shared object, GC-managed
    // ```
    let callback_control = Arc::clone(control);
    // What:     `let callback_channels = channels as usize;`. CAST the `u16` channel count
    //           to `usize` for use as a multiplier/length inside the callback.
    // Why:      The callback computes `frames * channels` as a `usize` sample count; pre-cast
    //           so it is the right width.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const callbackChannels = channels;
    // ```
    let callback_channels = channels as usize;
    // What:     `let stream = AudioStreamBuilder::new() ... .open_stream().map_err(audio_error)?;`.
    //           A BUILDER chain: `AudioStreamBuilder::new()` makes a builder (returning a
    //           `Result`, so `.map_err(audio_error)?` converts and propagates any error),
    //           then each `.direction(...)`, `.format(...)`, `.sample_rate(...)`,
    //           `.channel_count(...)`, `.performance_mode(...)`, `.data_callback(...)` sets
    //           one option and returns the builder for the next call, and `.open_stream()`
    //           finally opens the configured AAudio stream. The whole expression's value is
    //           bound to `stream`. Each call is annotated on its own line below.
    // Why:      Configure and open the output stream at this track's rate/format with our
    //           realtime data callback.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const stream = new AudioStreamBuilder()
    //   .direction("output").format("pcmFloat").sampleRate(rate).channelCount(channels)
    //   .performanceMode("lowLatency").dataCallback((stream, data, frames) => audioCallback(...))
    //   .openStream();
    // ```
    let stream = AudioStreamBuilder::new()
        // What:     `.map_err(audio_error)?`. `AudioStreamBuilder::new()` returns a
        //           `Result<AudioStreamBuilder, _>`. `.map_err(audio_error)` converts any
        //           error into our `PlayerError` by passing it to the `audio_error` helper
        //           (named function used as a value). The trailing `?` then unwraps the `Ok`
        //           builder or RETURNS the `Err` from `reconfigure_output` early.
        // Why:      Creating the builder can fail (no AAudio support); convert and propagate.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // builder creation throws a PlayerError on failure
        // ```
        .map_err(audio_error)?
        // What:     `.direction(AudioDirection::Output)`. Set the stream direction to the
        //           `Output` variant of the `AudioDirection` enum (sibling: `Input`).
        // Why:      We are playing audio, so the stream must be an output stream.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // .direction("output")
        // ```
        .direction(AudioDirection::Output)
        // What:     `.format(AudioFormat::PCM_Float)`. Set the sample format to the
        //           `PCM_Float` variant of `AudioFormat` (32-bit float samples). Siblings you
        //           might expect: `PCM_I16` (16-bit integer), etc.
        // Why:      Our ring holds `f32` samples, so the stream must expect float samples;
        //           this keeps the callback a straight memory copy with no format conversion.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // .format("pcmFloat")
        // ```
        .format(AudioFormat::PCM_Float)
        // What:     `.sample_rate(rate as i32)`. Set the stream's sample rate. `rate as i32`
        //           CASTS the `u32` rate to `i32` (a 32-bit SIGNED integer; sibling `u32` is
        //           unsigned) because AAudio's binding takes an `i32` here.
        // Why:      Open the device at this track's native rate; AAudio resamples to the
        //           hardware clock, so the engine never resamples.
        // Gotcha:   `as i32` is a width/signedness cast; an enormous `u32` would wrap to a
        //           negative `i32`, but real sample rates are tiny so it is safe here.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // .sampleRate(rate)
        // ```
        .sample_rate(rate as i32)
        // What:     `.channel_count(channels as i32)`. Set the channel count. `channels as i32`
        //           CASTS the `u16` count to the `i32` the AAudio binding wants.
        // Why:      Open the stream with the track's channel count (mono/stereo/etc.).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // .channelCount(channels)
        // ```
        .channel_count(channels as i32)
        // What:     `.performance_mode(AudioPerformanceMode::LowLatency)`. Set the
        //           performance hint to the `LowLatency` variant of `AudioPerformanceMode`
        //           (siblings: `None` (default) and `PowerSaving`).
        // Why:      A music player wants small, snappy buffers (low latency) over maximum
        //           power saving.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // .performanceMode("lowLatency")
        // ```
        .performance_mode(AudioPerformanceMode::LowLatency)
        // What:     `.data_callback(Box::new(move |_stream: &AudioStream, data: *mut c_void, frames: i32| { ... }))`.
        //           Register the realtime data callback. `Box::new(...)` heap-allocates the
        //           closure behind an owning pointer (AAudio stores it as a boxed trait
        //           object). `move |...| { ... }` is a CLOSURE (anonymous function) whose
        //           `move` keyword transfers ownership of the captured variables (`cons`,
        //           `callback_control`, `callback_channels`) INTO it. Parameters:
        //           `_stream: &AudioStream` (the stream, borrowed, ignored via the leading
        //           `_`), `data: *mut c_void` (a RAW mutable pointer to the speaker buffer,
        //           untyped), `frames: i32` (how many frames AAudio wants). The closure body
        //           just delegates to `audio_callback`.
        // Why:      AAudio invokes this on its realtime thread whenever the speakers need
        //           more samples; it is where we copy audio out of the ring.
        // Gotcha:   `data: *mut c_void` is a RAW POINTER (a bare address with no type or
        //           bounds), nothing like a JS array; we reinterpret it inside `audio_callback`
        //           under `unsafe`. `move` means the closure now OWNS `cons` and the cloned
        //           control handle.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // .dataCallback((_stream, data, frames) =>
        //   audioCallback(cons, data, frames, callbackControl, callbackChannels))
        // ```
        .data_callback(Box::new(
            // What:     `move |_stream: &AudioStream, data: *mut c_void, frames: i32| { ... }`.
            //           The closure itself (the inner `Box::new` argument). `move` captures
            //           `cons`/`callback_control`/`callback_channels` by ownership. `&AudioStream`
            //           is a shared borrow of the stream (we don't use it, hence `_stream`).
            //           `*mut c_void` is the raw writable speaker buffer; `i32` the frame count.
            // Why:      Bridge AAudio's C-style callback signature to our `audio_callback`.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // (_stream, data, frames) => audioCallback(cons, data, frames, callbackControl, callbackChannels)
            // ```
            move |_stream: &AudioStream, data: *mut c_void, frames: i32| {
                // What:     `audio_callback(&mut cons, data, frames, &callback_control, callback_channels)`.
                //           Call the real callback. `&mut cons` lends the captured consumer
                //           MUTABLY (popping advances its cursor). `data` and `frames` pass
                //           the raw buffer and frame count straight through. `&callback_control`
                //           lends the control handle read-only. `callback_channels` is a plain
                //           `usize` copy. No trailing `;`, so this call's `AudioCallbackResult`
                //           is the closure's TAIL EXPRESSION and therefore the value AAudio
                //           receives (telling it to continue or stop).
                // Why:      Do the actual pop/gate/scale/advance work in a named function the
                //           closure forwards to.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // return audioCallback(cons, data, frames, callbackControl, callbackChannels);
                // ```
                return audio_callback(&mut cons, data, frames, &callback_control, callback_channels)
            },
        ))
        // What:     `.open_stream()`. Finalize the builder and actually OPEN the configured
        //           AAudio stream. Returns `Result<AudioStream, _>`.
        // Why:      Turn the accumulated configuration into a live stream we can start.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // .openStream()
        // ```
        .open_stream()
        // What:     `.map_err(audio_error)?`. Convert any open-stream error into our
        //           `PlayerError` via the `audio_error` helper, then `?` unwraps the `Ok`
        //           stream or returns the `Err` from `reconfigure_output` early.
        // Why:      Opening can fail (device busy, bad config); surface it as our error.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // openStream throws a PlayerError on failure
        // ```
        .map_err(audio_error)?;
    // What:     `stream.request_start().map_err(audio_error)?;`. `request_start()` asks
    //           AAudio to start playing the stream (it returns a `Result`). `.map_err(audio_error)`
    //           converts a failure into our `PlayerError`, and `?` propagates it early.
    // Why:      The callback only fires once the stream is started; start it now. The play
    //           GATE (the `playing` atomic) decides whether the callback actually sounds.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // stream.requestStart(); // throws a PlayerError on failure
    // ```
    stream.request_start().map_err(audio_error)?;
    // What:     `state.stream = Some(stream);`. Store the started stream in the field,
    //           wrapped in `Some`. Moving it in keeps it alive (and playing) while this
    //           track plays.
    // Why:      Dropping the stream would close it; the worker state must own it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // state.stream = stream;
    // ```
    state.stream = Some(stream);
    // What:     `Ok(())`. Construct the success variant of `Result`, wrapping the empty
    //           unit value `()` (Rust's "nothing meaningful", like `void`). No trailing
    //           `;`, so this is the function's TAIL EXPRESSION and its return value.
    // Why:      Signal "output rebuilt successfully" with no payload.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return; // success
    // ```
    return Ok(())
}

/// What:
///      `fn pump(state: &mut WorkerState, control: &Control) -> bool`.
///  A private
///           function.
///  `&mut WorkerState` exclusive borrow (we mutate `pending`/`prod`).
///           `&Control` is a shared READ-ONLY borrow of the control block directly (note:
///           NOT `&Arc<Control>` here,
///  just `&Control`;
///  the `Arc` deref-coerces to a plain
///           reference).
///  Returns a `bool`:
///  did any samples get accepted this call?
/// Why:
///       Push one unit of decoded audio (carryover first,
///  then a fresh chunk) and report
///           whether work happened,
///  so the caller knows whether to nap.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pump(state: WorkerState, control: Control): boolean { ... }
/// ```
fn pump(state: &mut WorkerState, control: &Control) -> bool {
    // What:     `let mut did_work = false;`. A MUTABLE local boolean, initially `false`.
    //           `mut` because we OR new progress into it below.
    // Why:      Accumulate whether any samples were pushed during this pump.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let didWork = false;
    // ```
    let mut did_work = false;
    // What:     `if state.pending_pos < state.pending.len() { ... }`. `state.pending.len()`
    //           returns the `Vec`'s length as a `usize`. The `<` compares the read cursor to
    //           the length; true means there is still carryover to push first. Comparison is
    //           TS-identical.
    // Why:      Drain the backpressure carryover before decoding anything new, so samples
    //           stay in order.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (state.pendingPos < state.pending.length) { ... }
    // ```
    if state.pending_pos < state.pending.len() {
        // What:     `let pushed = match state.prod.as_mut() { ... };`. `state.prod.as_mut()`
        //           turns `&mut Option<HeapProd<f32>>` into `Option<&mut HeapProd<f32>>` (a
        //           mutable borrow of the inner producer without moving it out). `match`
        //           unpacks it: a producer exists or it does not.
        // Why:      Get a mutable handle to the ring's write end to push carryover, or bail
        //           if there is no ring.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let pushed: number;
        // if (state.prod === null) return false;
        // ```
        let pushed = match state.prod.as_mut() {
            // What:     `Some(prod) => prod.push_slice(&state.pending[state.pending_pos..])`.
            //           Unwrap `Some` binding the producer to `prod`. `&state.pending[state.pending_pos..]`
            //           borrows a SLICE of the carryover from the cursor to the end (the `..`
            //           is range/slice syntax with no TS equivalent; `&` lends it read-only).
            //           `push_slice(...)` copies as many of those samples into the ring as
            //           fit and returns the count pushed (a `usize`). That count is the arm's
            //           value, assigned to `pushed`.
            // Why:      Push the not-yet-accepted carryover into the freed ring space.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // pushed = prod.pushSlice(state.pending.slice(state.pendingPos));
            // ```
            Some(prod) => prod.push_slice(&state.pending[state.pending_pos..]),
            // What:     `None => return false`. No ring producer exists; `return false` exits
            //           `pump` reporting "no work" (false).
            // Why:      Without a ring there is nowhere to push; report idle.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // if no producer: return false;
            // ```
            None => return false,
        };
        // What:     `state.pending_pos += pushed;`. Advance the carryover cursor by the
        //           number of samples just pushed. `+=` is TS-identical integer increment.
        // Why:      Record how much of the carryover the ring accepted.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // state.pendingPos += pushed;
        // ```
        state.pending_pos += pushed;
        // What:     `did_work |= pushed > 0;`. `pushed > 0` is a boolean (did we push
        //           anything?). `|=` is the OR-assign operator: set `did_work` to true if it
        //           or the right side is true. Both operators exist in TS.
        // Why:      Mark that work happened if at least one sample went in.
        // Gotcha:   `|=` on a `bool` here is logical-or-assign; on integers `|=` would be
        //           BITWISE or, but both operands are `bool` so it behaves like `||=`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // didWork ||= pushed > 0;
        // ```
        did_work |= pushed > 0;
        // What:     `if state.pending_pos < state.pending.len() { return did_work; }`. If the
        //           cursor still has not reached the end, the ring filled up mid-carryover;
        //           `return did_work` exits early (more carryover remains for next pump).
        // Why:      The ring is full, so stop here; do not decode a fresh chunk on top.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (state.pendingPos < state.pending.length) return didWork;
        // ```
        if state.pending_pos < state.pending.len() {
            return did_work;
        }
        // What:     `state.pending.clear();`. The carryover was fully pushed; empty the `Vec`.
        // Why:      Free the carryover buffer so the next chunk can reuse `pending`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // state.pending.length = 0;
        // ```
        state.pending.clear();
        // What:     `state.pending_pos = 0;`. Reset the now-empty carryover cursor to 0.
        // Why:      Keep the cursor consistent with the emptied `pending`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // state.pendingPos = 0;
        // ```
        state.pending_pos = 0;
    }
    // What:     `if control.decode_done.load(Ordering::Acquire) { return did_work; }`.
    //           `control.decode_done.load(Ordering::Acquire)` is the atomic READ of the
    //           "fully decoded" flag. `Acquire` ordering pairs with the `Release` writes
    //           elsewhere so we observe everything published before the flag was set. If the
    //           track is fully decoded, `return did_work`.
    // Why:      Once decoding is finished there is nothing more to read from the source;
    //           skip the decode step.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (Atomics.load(control.decodeDone, 0) !== 0) return didWork;
    // ```
    if control.decode_done.load(Ordering::Acquire) {
        return did_work;
    }
    // What:     `let chunk = match state.source.as_mut() { ... };`. `state.source.as_mut()`
    //           gives `Option<&mut Box<dyn Source>>` (a mutable borrow of the decoder
    //           without moving it). `match` unpacks: decode the next chunk, or bail if no
    //           source.
    // Why:      Pull the next block of decoded samples, handling EOF/errors and the
    //           no-source case.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let chunk: number[];
    // if (state.source === null) return false;
    // ```
    let chunk = match state.source.as_mut() {
        // What:     `Some(source) => match source.next_chunk() { ... }`. Unwrap `Some`
        //           binding the decoder to `source`, then a NESTED `match` on
        //           `source.next_chunk()`, which returns `Result<Vec<f32>, PlayerError>` (a
        //           fresh owned sample buffer or an error).
        // Why:      Ask the decoder for more audio and branch on success vs failure.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // try { chunk = source.nextChunk(); } catch { /* mark done, return */ }
        // ```
        Some(source) => match source.next_chunk() {
            // What:     `Ok(chunk) => chunk`. Success arm: unwrap `Ok` and bind the decoded
            //           `Vec<f32>` to `chunk`; the bare `chunk` is the arm's value (the
            //           samples we will push).
            // Why:      Use the freshly decoded samples.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // chunk = decoded samples
            // ```
            Ok(chunk) => chunk,
            // What:     `Err(_) => { control.decode_done.store(true, Ordering::Release); return did_work; }`.
            //           Failure arm: `Err(_)` matches and DISCARDS the error. We atomically
            //           set `decode_done` to `true` (with `Release`, publishing it) and then
            //           `return did_work`.
            // Why:      A decode error (including EOF surfaced as an error) means no more
            //           audio; flag done so the callback can later mark the track ended.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // catch { Atomics.store(control.decodeDone, 0, 1); return didWork; }
            // ```
            Err(_) => {
                control.decode_done.store(true, Ordering::Release);
                return did_work;
            }
        },
        // What:     `None => return false`. No source loaded; `return false` reports idle.
        // Why:      Nothing to decode without a source.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // if no source: return false;
        // ```
        None => return false,
    };
    // What:     `if chunk.is_empty() { ... }`. `.is_empty()` returns `true` when the `Vec`
    //           has zero elements.
    // Why:      An empty chunk signals end-of-stream (the decoder produced no more samples).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (chunk.length === 0) { ... }
    // ```
    if chunk.is_empty() {
        // What:     `control.decode_done.store(true, Ordering::Release);`. Atomically flag
        //           decoding finished, with `Release` ordering so the callback observes it.
        // Why:      Tell the rest of the engine the track is fully decoded.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // Atomics.store(control.decodeDone, 0, 1 /* true */);
        // ```
        control.decode_done.store(true, Ordering::Release);
        // What:     `return did_work;`. Exit `pump`, reporting whatever progress the
        //           carryover push made earlier.
        // Why:      Nothing decoded this call; hand back the accumulated `did_work`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return didWork;
        // ```
        return did_work;
    }
    // What:     `let pushed = match state.prod.as_mut() { ... };`. As before, `as_mut()`
    //           gives `Option<&mut HeapProd<f32>>`; `match` pushes the fresh chunk if a
    //           producer exists, else bails.
    // Why:      Push the just-decoded chunk into the ring.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let pushed: number;
    // if (state.prod === null) return false;
    // ```
    let pushed = match state.prod.as_mut() {
        // What:     `Some(prod) => prod.push_slice(&chunk)`. Unwrap `Some` to `prod`, then
        //           `push_slice(&chunk)` copies as many of `chunk`'s samples into the ring as
        //           fit, returning the count pushed. `&chunk` lends the whole `Vec` read-only
        //           as a slice.
        // Why:      Move the fresh samples into the ring for the callback to play.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // pushed = prod.pushSlice(chunk);
        // ```
        Some(prod) => prod.push_slice(&chunk),
        // What:     `None => return false`. No producer; `return false` reports idle.
        // Why:      Nowhere to push without a ring.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // if no producer: return false;
        // ```
        None => return false,
    };
    // What:     `did_work |= pushed > 0;`. OR into `did_work` whether this push moved any
    //           samples (`pushed > 0`). `|=` on bools is logical-or-assign.
    // Why:      Mark progress if the fresh chunk was at least partly accepted.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // didWork ||= pushed > 0;
    // ```
    did_work |= pushed > 0;
    // What:     `if pushed < chunk.len() { ... }`. Compare how many samples were accepted to
    //           the chunk's length. If fewer than all fit, the ring filled mid-chunk.
    // Why:      Stash the unaccepted tail as carryover instead of dropping or re-decoding it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (pushed < chunk.length) { ... }
    // ```
    if pushed < chunk.len() {
        // What:     `state.pending = chunk;`. MOVE the whole `chunk` `Vec` into the `pending`
        //           field (ownership transfers; `chunk` is consumed). The unpushed tail lives
        //           on as carryover.
        // Why:      Keep the leftover samples so the next pump resumes from `pending_pos`.
        // Gotcha:   This MOVES `chunk` (no copy); `chunk` is unusable afterward in Rust.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // state.pending = chunk;
        // ```
        state.pending = chunk;
        // What:     `state.pending_pos = pushed;`. Set the carryover cursor to `pushed`, the
        //           count already accepted, so the next pump starts at the first unpushed
        //           sample.
        // Why:      Resume pushing exactly where the ring stopped accepting.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // state.pendingPos = pushed;
        // ```
        state.pending_pos = pushed;
    }
    // What:     `did_work`. The bare variable as the function's TAIL EXPRESSION (no trailing
    //           `;`), so its `bool` value is what `pump` returns.
    // Why:      Report to `worker_run` whether any samples were accepted this call (false
    //           means "nap").
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return didWork;
    // ```
    return did_work
}

/// What:
///      `fn audio_callback(cons: &mut HeapCons<f32>, data: *mut c_void, frames: i32, control: &Control, channels: usize) -> AudioCallbackResult`.
///           The realtime data callback (called by AAudio on its realtime thread).
///  Params:
///           `cons: &mut HeapCons<f32>` (the ring's READ end,
///  borrowed MUTABLY so popping
///           advances its cursor);
///  `data: *mut c_void` (a RAW writable pointer to the speaker
///           buffer,
///  untyped);
///  `frames: i32` (how many frames to fill);
///  `control: &Control`
///           (shared read-only borrow of the control block);
///  `channels: usize` (interleaved
///           channel count).
///  Returns `AudioCallbackResult` (`Continue` or `Stop`).
///  The
///           signature is split across lines for width.
/// Why:
///       Fill `data` with the next frames:
///  silence when paused,
///  otherwise pop from the
///           ring,
///  apply volume,
///  zero-fill underrun,
///  flag end-of-track,
///  advance the counter.
/// Gotcha:
///    This runs on a HARD-REALTIME thread:
///  it must never allocate,
///  lock,
///  or block,
///  or
///           the audio glitches.
///  That constraint shapes every line below.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function audioCallback(cons, data, frames, control, channels): "continue" | "stop" { ... }
/// ```
fn audio_callback(
    cons: &mut HeapCons<f32>,
    data: *mut c_void,
    frames: i32,
    control: &Control,
    channels: usize,
) -> AudioCallbackResult {
    // What:     `let total = (frames.max(0) as usize) * channels;`. `frames.max(0)` clamps a
    //           possibly-negative `i32` up to `0` (defensive). `as usize` CASTS that to the
    //           pointer-wide unsigned index type, then `* channels` gives the total number of
    //           INTERLEAVED samples (frames times channels).
    // Why:      We need the total `f32` slot count to view the raw buffer and to size the
    //           silence/copy operations.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const total = Math.max(frames, 0) * channels;
    // ```
    let total = (frames.max(0) as usize) * channels;
    // SAFETY: for a PCM_Float output stream AAudio guarantees `data` points to
    // `frames * channels` writable f32 slots.
    // What:     `let out = unsafe { std::slice::from_raw_parts_mut(data as *mut f32, total) };`.
    //           `unsafe { ... }` is a block where we promise the compiler we have upheld
    //           invariants it cannot check. `data as *mut f32` reinterprets the raw
    //           `*mut c_void` (untyped pointer) as a raw pointer to `f32`. `std::slice::from_raw_parts_mut(ptr, len)`
    //           builds a safe mutable slice `&mut [f32]` of length `total` over that memory.
    // Why:      Turn AAudio's bare buffer pointer into a bounds-checked `f32` slice we can
    //           fill safely from here on.
    // Gotcha:   `unsafe` does NOT disable checks for fun; it asserts the `SAFETY` comment's
    //           promise (`data` really points to `total` writable `f32`s) holds. A wrong
    //           length here is undefined behaviour, not a thrown error.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const out = new Float32Array(data, 0, total); // typed view over AAudio's buffer
    // ```
    let out = unsafe { std::slice::from_raw_parts_mut(data as *mut f32, total) };
    // What:     `if !control.playing.load(Ordering::Acquire) { ... }`. `control.playing.load(Ordering::Acquire)`
    //           atomically READS the play gate; `Acquire` pairs with the engine's `Release`
    //           writes. `!` negates it: the block runs when we are PAUSED.
    // Why:      When paused we must output silence and NOT drain the ring, so the buffered
    //           audio survives for a seamless resume.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (Atomics.load(control.playing, 0) === 0) { ... }
    // ```
    if !control.playing.load(Ordering::Acquire) {
        // What:     `out.fill(0.0);`. `.fill(0.0)` writes the `f32` value `0.0` into every
        //           slot of the `out` slice (the speaker buffer).
        // Why:      Output pure silence this cycle while paused.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // out.fill(0);
        // ```
        out.fill(0.0);
        // What:     `return AudioCallbackResult::Continue;`. Return the `Continue` variant of
        //           `AudioCallbackResult` (sibling: `Stop`, which would end the stream). This
        //           exits the callback early, telling AAudio to keep calling us.
        // Why:      We handled this cycle (silence); keep the stream alive for the next one.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return "continue";
        // ```
        return AudioCallbackResult::Continue;
    }
    // What:     `let popped = cons.pop_slice(out);`. `pop_slice(out)` pops up to `out.len()`
    //           samples from the ring's read end STRAIGHT into the `out` buffer and returns
    //           how many were actually available (a `usize`). `out` is `&mut [f32]`, lent to
    //           the popper to write into.
    // Why:      Copy decoded audio from the ring into the speaker buffer in one move (no
    //           allocation, realtime-safe).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const popped = cons.popSlice(out);
    // ```
    let popped = cons.pop_slice(out);
    // User volume times the per-track normalization gain (both <= 1), clamped as a final
    // guard against a clipped master's over-unity samples.
    // What:     `let gain = control.volume() * control.norm_gain();`. Call two methods that
    //           each atomically read an `f32` and return it: `volume()` (user volume) and
    //           `norm_gain()` (per-track normalization). Multiply them into one combined
    //           `f32` gain.
    // Why:      Apply both the user's volume and loudness normalization in a single scale
    //           factor per sample.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const gain = control.volume() * control.normGain();
    // ```
    let gain = control.volume() * control.norm_gain();
    // What:     `if gain != 1.0 { ... }`. `!=` is "not equal"; `1.0` is an `f32` literal
    //           (unity gain). The block runs only when the gain actually changes the samples.
    // Why:      Skip the per-sample multiply entirely at unity gain (the common case), saving
    //           work in the hot realtime path.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (gain !== 1) { ... }
    // ```
    if gain != 1.0 {
        // What:     `for sample in &mut out[..popped] { ... }`. A `for...of`-style loop over
        //           `&mut out[..popped]`: `out[..popped]` is the SLICE from the start up to
        //           index `popped` (range syntax, no TS equivalent); `&mut` lends it MUTABLY,
        //           so iterating yields mutable references `sample` (each a `&mut f32`).
        // Why:      Scale only the samples we actually popped (the filled portion), leaving the
        //           rest for the underrun zero-fill below.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (let i = 0; i < popped; i++) out[i] = clamp(out[i] * gain, -1, 1);
        // ```
        for sample in &mut out[..popped] {
            // What:     `*sample = (*sample * gain).clamp(-1.0, 1.0);`. `*sample` DEREFERENCES
            //           the mutable reference to read/write the underlying `f32`. `(*sample * gain)`
            //           scales it; `.clamp(-1.0, 1.0)` bounds the result into the valid audio
            //           range. The leading `*sample =` writes the clamped value back through the
            //           reference.
            // Why:      Apply the combined gain and hard-limit so a clipped master can't push
            //           samples past +/-1.0 (which would distort).
            // Gotcha:   `*sample` is pointer DEREFERENCE (read/write through the borrow), not
            //           multiplication; the `*` in `*sample * gain` is the dereference followed
            //           by an ordinary multiply.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // out[i] = Math.max(-1, Math.min(1, out[i] * gain));
            // ```
            *sample = (*sample * gain).clamp(-1.0, 1.0);
        }
    }
    // What:     `if popped < total { ... }`. Compare popped samples to the total requested.
    //           Fewer popped than requested means the ring under-ran (not enough decoded yet).
    // Why:      Detect underrun so we can zero the tail and possibly flag end-of-track.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (popped < total) { ... }
    // ```
    if popped < total {
        // What:     `out[popped..].fill(0.0);`. `out[popped..]` is the SLICE from index
        //           `popped` to the end (range syntax). `.fill(0.0)` writes `0.0` into every
        //           slot of that tail.
        // Why:      Output silence for the unfilled tail instead of leaving stale/garbage data.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // out.fill(0, popped); // zero from `popped` to the end
        // ```
        out[popped..].fill(0.0);
        // What:     `if control.decode_done.load(Ordering::Acquire) { ... }`. Atomically READ
        //           the `decode_done` flag with `Acquire`. True means the decoder is finished.
        // Why:      An underrun WHILE decoding is done means the track has fully drained, i.e.
        //           it ended.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (Atomics.load(control.decodeDone, 0) !== 0) { ... }
        // ```
        if control.decode_done.load(Ordering::Acquire) {
            // What:     `control.ended.store(true, Ordering::Release);`. Atomically set the
            //           `ended` flag to `true` with `Release`, publishing it to Kotlin's
            //           pollers.
            // Why:      Signal "track ended" so the controller can advance to the next track.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // Atomics.store(control.ended, 0, 1 /* true */);
            // ```
            control.ended.store(true, Ordering::Release);
        }
    }
    // What:     `if let Some(played_frames) = popped.checked_div(channels) { ... }`.
    //           `checked_div` divides `popped` (a SAMPLE count) by `channels`, returning
    //           `None` when `channels == 0` (a checked divide that cannot divide-by-zero)
    //           and `Some(played_frames)` otherwise, where `played_frames` is the FRAME
    //           count (samples per frame equals channels). The `if let` runs the body only
    //           on `Some`. The name differs from the `frames` callback parameter so the
    //           played count never shadows the requested one.
    // Why:      Only advance the played-frame counter when channels is valid; `checked_div`
    //           folds the zero-guard and the divide into one call (no separate `> 0` check).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const playedFrames = channels > 0 ? Math.floor(popped / channels) : null;
    // if (playedFrames !== null) { ... }
    // ```
    if let Some(played_frames) = popped.checked_div(channels) {
        // What:     `control.frames_played.fetch_add(played_frames as u64, Ordering::AcqRel);`.
        //           `as u64` CASTS the `usize` frame count to the atomic's 64-bit width.
        //           `.fetch_add(delta, ordering)` atomically ADDS the delta to the counter
        //           (a read-modify-write). `Ordering::AcqRel` is both Acquire and Release at
        //           once, correct for a read-modify-write that both observes and publishes.
        // Why:      Advance the position counter by the frames we just played, so Kotlin's
        //           polled position moves forward.
        // Gotcha:   `fetch_add` is an ATOMIC increment (not a plain `+=`); it returns the OLD
        //           value, which we ignore here.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // Atomics.add(control.framesPlayed, 0, playedFrames);
        // ```
        control
            .frames_played
            .fetch_add(played_frames as u64, Ordering::AcqRel);
    }
    // What:     `AudioCallbackResult::Continue`. The `Continue` variant as the function's
    //           TAIL EXPRESSION (no trailing `;`), so it is what the callback returns,
    //           telling AAudio to keep calling us. Sibling: `Stop`.
    // Why:      We finished this cycle successfully; keep the stream running.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return "continue";
    // ```
    return AudioCallbackResult::Continue
}

/// What:
///      `fn audio_error<E: std::fmt::Debug>(error: E) -> PlayerError`.
///  A GENERIC
///           function:
///  `<E: std::fmt::Debug>` introduces one type parameter `E` constrained
///           (the `:` is a trait BOUND) to types that implement `std::fmt::Debug` (the
///           developer-facing `{:?}` formatting trait).
///  `error: E` takes any such value BY
///           VALUE;
///  it returns a `PlayerError`.
/// Why:
///       AAudio builder/stream calls return DIFFERENT error types;
///  one generic helper
///           turns any of them into our single `PlayerError` so callers can `?` them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function audioError(error: unknown): PlayerError { return new PlayerError(`${error}`); }
/// ```
fn audio_error<E: std::fmt::Debug>(error: E) -> PlayerError {
    // What:     `PlayerError::Audio(format!("{error:?}"))`. `format!(...)` is a MACRO (the `!`
    //           marks it) that builds an owned `String` from a template; `{error:?}` formats
    //           `error` with its `Debug` representation. `PlayerError::Audio(...)` wraps that
    //           string in the `Audio` variant of our error enum. No trailing `;`, so this is
    //           the TAIL EXPRESSION the function returns.
    // Why:      Flatten the opaque AAudio error to readable text and hand it back as our
    //           `Audio` error case.
    // Gotcha:   `format!` ALLOCATES a `String`; that is fine here because `audio_error` runs
    //           on the worker thread (during setup), never inside the realtime callback.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return new PlayerError({ kind: "audio", message: String(error) });
    // ```
    return PlayerError::Audio(format!("{error:?}"))
}
