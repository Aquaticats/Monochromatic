//! PipeWire output:
//!  the thin FFI boundary.
//!  ALL PipeWire/SPA/realtime code is
//! confined to this module so the rest of the player (engine.rs) only ever
//! calls `Output::new`,
//!  `Output::reconfigure`,
//!  and pushes `f32` samples into
//! the returned ring-buffer producer.
//!
//! Mental model for a TypeScript reader:
//!  PipeWire is a C audio server.
//!  We open
//! one background loop (its own OS thread,
//!  managed by C),
//!  create an output
//! "stream",
//!  and register a `process` callback that the C loop calls on a
//! realtime thread whenever the speakers need more samples.
//!  That callback
//! copies samples out of a lock-free queue (the ring buffer) and into the
//! hardware buffer.
//!  We hand the WRITE end of that queue back to the engine.
//!
//! Several types here are `!Send` (they use `Rc` internally,
//!  like a value that
//! must stay on the thread that made it).
//!  So `Output` is created,
//!  used,
//!  and
//! dropped entirely on the engine's controller thread;
//!  only the realtime
//! `process` callback runs elsewhere (on PipeWire's own thread),
//!  and it touches
//! only the ring-buffer consumer it was given.

/// What:
///      `use std::io::Cursor;`.
///  `Cursor` wraps an in-memory `Vec<u8>` and gives it
///           the `Read`/`Write` interface a serializer expects.
/// Why:
///       The SPA pod serializer writes bytes into a `Cursor<Vec<u8>>`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const buf = []; // a sink the serializer appends to
/// ```
use std::io::Cursor;

/// What:
///      `use std::mem::size_of;`.
///  `size_of::<T>()` returns the byte size of a type at
///           compile time (a `usize`).
/// Why:
///       One `f32` sample is 4 bytes;
///  we compute strides from this rather than
///           hard-coding `4`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const F32_BYTES = 4; // Float32 is 4 bytes
/// ```
use std::mem::size_of;

/// What:
///      `use std::sync::atomic::{AtomicBool, Ordering};`.
///  `AtomicBool` is a `bool`
///           that can be read and written from multiple threads WITHOUT a lock (the
///           hardware guarantees the read/write is indivisible).
///  `Ordering` says how
///           strictly this access is ordered against other memory operations;
///  siblings
///           range from `Relaxed` (loosest,
///  just atomicity) up to `SeqCst` (strictest,
///  a
///           global order).
/// Why:
///       The engine thread flips a "playing" flag and the realtime audio callback
///           reads it;
///  a plain `bool` shared across threads is undefined behaviour,
///  so it
///           must be atomic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let playing = false; // a cross-thread boolean flag
/// ```
use std::sync::atomic::{AtomicBool, Ordering};

/// What:
///      `use std::sync::Arc;`.
///  `Arc<T>` is an ATOMICALLY reference-counted shared
///           pointer:
///  cloning it just bumps a thread-safe counter,
///  and the inner `T` is
///           freed when the last clone drops.
///  Sibling:
///  `Rc<T>`,
///  the same idea but NOT
///           thread-safe (single-thread only).
/// Why:
///       The same `AtomicBool` must live in two places at once (the `Output` on the
///           engine thread and the `ProcessData` on the audio thread);
///  `Arc` lets both
///           hold a clone of one shared cell.
///  `Rc` would not compile here because the cell
///           crosses a thread boundary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // both closures capture the same `playing` variable
/// ```
use std::sync::Arc;

/// What:
///      `use std::thread::Thread;`.
///  `Thread` is a cheap,
///  cloneable HANDLE to a running
///           thread (it wraps an internal `Arc`).
///  We only ever call `.unpark()` on it.
///           Sibling you might expect:
///  `JoinHandle`,
///  which OWNS the thread and can wait for
///           it;
///  `Thread` is just a wake-able reference.
/// Why:
///       The realtime audio callback holds one so it can wake (`unpark`) the engine
///           worker after it drains the ring buffer,
///  telling it to decode more.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Thread = WorkerRef; // a handle you can post "wake up" to
/// ```
use std::thread::Thread;

/// What:
///      `use pipewire as pw;`.
///  Import the crate and rename it `pw` for short.
/// Why:
///       Every PipeWire call below is `pw::...`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as pw from "pipewire";
/// ```
use pipewire as pw;

/// What:
///      `use pw::{properties::properties, spa};`.
///  Brings the `properties!` MACRO
///           (builds a key/value dictionary) and the `spa` submodule (the lower-level
///           "Simple Plugin API" types:
///  audio formats,
///  pods) into scope.
/// Why:
///       Stream creation needs properties;
///  format negotiation needs spa.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { properties, spa } from "pipewire";
/// ```
use pw::{properties::properties, spa};

/// What:
///      `use pw::context::ContextRc;`.
///  The PipeWire "context" object:
///  the root handle
///           from which you connect to the server.
/// Why:
///       We create one context and keep it alive for the program.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Context } from "pipewire";
/// ```
use pw::context::ContextRc;

/// What:
///      `use pw::core::CoreRc;`.
///  The connected "core":
///  your session with the PipeWire
///           server,
///  created from a context.
/// Why:
///       Streams are created from a `Core`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Core } from "pipewire";
/// ```
use pw::core::CoreRc;

/// What:
///      `use pw::stream::{StreamFlags, StreamListener, StreamRc};`.
///  `Stream` is an
///           audio stream;
///  `StreamFlags` are connect-time options;
///  a `StreamListener` is
///           the live registration of our callbacks (dropping it unregisters them).
/// Why:
///       We create a stream,
///  register a process callback,
///  and connect it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Stream, StreamFlags, StreamListener } from "pipewire";
/// ```
use pw::stream::{StreamFlags, StreamListener, StreamRc};

/// What:
///      `use pw::thread_loop::ThreadLoopRc;`.
///  A loop that runs on its OWN OS thread,
///           spawned and driven by PipeWire's C code.
/// Why:
///       Audio must run independently of our decode/command loop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a Worker that owns its own event loop, but driven by C
/// ```
use pw::thread_loop::ThreadLoopRc;

/// What:
///      `use spa::param::audio::{AudioFormat, AudioInfoRaw};`.
///  `AudioFormat` names the
///           sample encoding (we use `F32LE` = little-endian 32-bit float).
///  `AudioInfoRaw`
///           is a small struct describing format + rate + channels that we turn into a
///           "pod".
/// Why:
///       To tell PipeWire we will feed interleaved f32 at a given rate/channels.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { AudioFormat, AudioInfoRaw } from "pipewire/spa";
/// ```
use spa::param::audio::{AudioFormat, AudioInfoRaw};

/// What:
///      `use spa::pod::{serialize::PodSerializer, Object, Pod, Value};`.
///  A "pod"
///           (Plain Old Data) is SPA's self-describing binary value format.
///  `PodSerializer`
///           turns a `Value` into bytes;
///  `Object`/`Value` build the structured value;
///  `Pod`
///           is a borrowed view over the bytes.
/// Why:
///       Stream format parameters are passed to PipeWire as a serialized pod.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bytes = encodePod({ type: "Format", ... }); // bespoke binary encoding
/// ```
use spa::pod::{serialize::PodSerializer, Object, Pod, Value};

/// What:
///      `use spa::utils::Direction;`.
///  Enum:
///  is this stream `Output` (we produce audio)
///           or `Input` (we capture)?
///  Sibling:
///  `Input`.
/// Why:
///       A music player produces audio,
///  so `Output`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Direction = "input" | "output";
/// ```
use spa::utils::Direction;

/// What:
///      `use ringbuf::traits::{Consumer, Split};`.
///  These TRAITS bring methods into
///           scope:
///  `Split::split` (cut a ring buffer into a producer and a consumer half)
///           and `Consumer::pop_slice` (drain samples out).
/// Why:
///       We split the buffer and the callback pops from the consumer half.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // no equivalent: importing interfaces to unlock .split()/.popSlice()
/// ```
use ringbuf::traits::{Consumer, Split};

/// What:
///      `use ringbuf::{HeapCons, HeapProd, HeapRb};`.
///  `HeapRb<T>` is a heap-allocated
///           ring buffer;
///  `.split()` yields a `HeapProd<T>` (write end) and `HeapCons<T>`
///           (read end).
///  Both halves can live on different threads (single-producer,
///           single-consumer).
/// Why:
///       A lock-free hand-off of samples from the decode thread to the realtime audio
///           thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a fixed-size lock-free queue split into a writer and a reader
/// ```
use ringbuf::{HeapCons, HeapProd, HeapRb};

/// What:
///      `use crate::error::PlayerError;`.
///  Our one app-wide error type.
/// Why:
///       Fallible methods here return `PlayerError`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { PlayerError } from "./error";
/// ```
use crate::error::PlayerError;
use crate::audio_meter::{AudioMeter, MeterSnapshot};

/// What:
///      `const F32_BYTES: usize = size_of::<f32>();`.
///  Bytes in one `f32` sample (4).
///           `usize` because it measures memory and feeds stride math.
/// Why:
///       Stride = bytes per audio frame = `F32_BYTES * channels`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const F32_BYTES = 4;
/// ```
const F32_BYTES: usize = size_of::<f32>();

/// What:
///      `pub const FILL_FRAMES_FLOOR: usize = 2_048;`.
///  The fewest frames one output callback hands to PipeWire when the graph asks for less
///           (about 43 ms at 48 kHz).
///  `usize` because it is a frame count used for buffer arithmetic.
/// Why:
///       One callback per graph cycle (a few milliseconds) would turn every late wake-up of the audio thread
///  into an audible skip,
///  while handing over the whole quarter-second buffer made the audible position trail the ring by that much.
///  This floor bounds the position error and keeps a margin against scheduling jitter.
///  It is also how much audio can still be on its way to the speakers after the ring ran empty,
///  so the controller waits that long before it rebuilds the stream.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FILL_FRAMES_FLOOR = 2048;
/// ```
pub const FILL_FRAMES_FLOOR: usize = 2_048;

/// What:
///      `struct ProcessData { ... }`.
///  The state the realtime callback owns.
///  It is
///           moved INTO the stream listener;
///  the C loop hands it back to our closure as
///           `&mut ProcessData` on every call.
/// Why:
///       The callback must not allocate or lock;
///  it keeps its consumer and a reusable
///           scratch buffer here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ProcessData = { consumer: RingConsumer; channels: number; scratch: Float32Array; playing: { value: boolean }; worker: WorkerRef };
/// ```
struct ProcessData {
    /// What:
    ///      `consumer: HeapCons<f32>`.
    ///  The READ half of the ring buffer.
    /// Why:
    ///       The callback pops decoded samples from it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// consumer: RingConsumer<number>;
    /// ```
    consumer: HeapCons<f32>,
    /// What:
    ///      `channels: usize`.
    ///  Channel count of the current track (1 or 2...).
    ///  `usize`
    ///           for index/stride arithmetic without casts.
    /// Why:
    ///       Needed to compute the per-frame stride and how many samples fill a buffer.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// channels: number;
    /// ```
    channels: usize,
    /// What:
    ///      `scratch: Vec<f32>`.
    ///  A reusable buffer the callback pops into before
    ///           converting to little-endian bytes.
    /// Why:
    ///       Avoid allocating inside the realtime callback (grows at most once when the
    ///           buffer size first stabilises).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// scratch: Float32Array;
    /// ```
    scratch: Vec<f32>,
    /// What:
    ///      `playing: Arc<AtomicBool>`.
    ///  A shared cross-thread flag:
    ///  `true` means feed
    ///           real audio,
    ///  `false` means output silence.
    ///  This is a CLONE of the same cell
    ///           the `Output` (and engine) hold.
    /// Why:
    ///       Lets pause take effect instantly:
    ///  the moment the engine flips the flag,
    ///  the
    ///           very next realtime callback stops draining the ring buffer and emits
    ///           silence,
    ///  instead of playing the ~1 second of audio already buffered.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// playing: { value: boolean };
    /// ```
    playing: Arc<AtomicBool>,
    /// What:
    ///      `worker: Thread`.
    ///  A cloneable handle to the engine worker thread.
    ///  The
    ///           callback calls `.unpark()` on it after popping samples.
    /// Why:
    ///       Popping frees space in the ring buffer;
    ///  waking the worker lets it decode
    ///           more right away instead of waiting for a timeout.
    ///  This is what replaces the
    ///           old busy-poll without re-introducing audio gaps.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// worker: WorkerRef;
    /// ```
    worker: Thread,
    /// What:
    ///      `meter: Arc<AudioMeter>`.
    ///  The shared underrun counters this callback writes.
    /// Why:
    ///       Counting here and reporting from the engine thread keeps logging off the realtime path.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// meter: AudioMeter;
    /// ```
    meter: Arc<AudioMeter>,
    /// What:
    ///      `draining: Arc<AtomicBool>`.
    ///  Set by the engine once the decoder has pushed the last frame of the track;
    ///  the callback then stops counting a short buffer as an underrun.
    /// Why:
    ///       The last callbacks of a track always find the ring empty;
    ///  that is the end of the music, not a stutter.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// draining: { value: boolean };
    /// ```
    draining: Arc<AtomicBool>,
}

/// What:
///      `pub struct Output { ... }`.
///  Owns the whole PipeWire pipeline.
///  FIELD ORDER IS
///           THE DROP ORDER:
///  Rust drops fields top-to-bottom,
///  so the listener (which holds
///           a raw pointer into the stream) drops before the stream,
///  and both before the
///           core/context/loop.
/// Why:
///       Keeps all the `!Send` PipeWire objects alive together and tears them down in a
///           safe order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Output { listener; stream; core; context; threadLoop; playing; worker; }
/// ```
pub struct Output {
    /// What:
    ///      `listener: Option<StreamListener<ProcessData>>`.
    ///  The live callback
    ///           registration,
    ///  or `None` before the first stream.
    ///  Dropping it unregisters
    ///           the callback.
    /// Why:
    ///       Must outlive nothing and be dropped first (it points into stream).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// listener: StreamListener | null;
    /// ```
    listener: Option<StreamListener<ProcessData>>,
    /// What:
    ///      `stream: Option<StreamRc>`.
    ///  The current output stream,
    ///  or `None`.
    /// Why:
    ///       Recreated per track at that track's native rate/channels.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// stream: Stream | null;
    /// ```
    stream: Option<StreamRc>,
    /// What:
    ///      `core: CoreRc`.
    ///  The connected session.
    ///  Held for the program's life.
    /// Why:
    ///       Needed to create streams.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// core: Core;
    /// ```
    core: CoreRc,
    /// What:
    ///      `_context: ContextRc`.
    ///  The context the core came from.
    ///  The leading `_` says
    ///           "kept only to stay alive,
    ///  not otherwise used".
    /// Why:
    ///       The core depends on the context outliving it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private context: Context; // kept alive, never read
    /// ```
    _context: ContextRc,
    /// What:
    ///      `thread_loop: ThreadLoopRc`.
    ///  The background audio loop.
    /// Why:
    ///       Drives the realtime callback;
    ///  dropped last.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// threadLoop: ThreadLoop;
    /// ```
    thread_loop: ThreadLoopRc,
    /// What:
    ///      `playing: Arc<AtomicBool>`.
    ///  The MASTER copy of the play/pause flag.
    ///  Created
    ///           once here and cloned into each new `ProcessData` on `reconfigure`,
    ///  so the
    ///           flag survives track changes.
    /// Why:
    ///       `set_playing` writes it from the engine thread;
    ///  the realtime callback reads
    ///           its clone.
    ///  One shared cell keeps them in sync.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// playing: { value: boolean };
    /// ```
    playing: Arc<AtomicBool>,
    /// What:
    ///      `worker: Thread`.
    ///  The engine worker's thread handle,
    ///  kept so each new
    ///           stream's callback (built in `reconfigure`) can be handed a clone.
    /// Why:
    ///       The callback needs it to `unpark()` the worker when the ring buffer drains;
    ///           storing it here lets it survive across per-track reconfigures.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// worker: WorkerRef;
    /// ```
    worker: Thread,
    /// What:
    ///      `meter: Arc<AudioMeter>`.
    ///  Underrun counters shared with every stream this output creates.
    /// Why:
    ///       The engine thread reads them once a second to report stutter.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// meter: AudioMeter;
    /// ```
    meter: Arc<AudioMeter>,
    /// What:
    ///      `draining: Arc<AtomicBool>`.
    ///  The master copy of the end-of-track flag;
    ///  each stream's callback gets a clone and `reconfigure` clears it.
    /// Why:
    ///       `set_draining` writes it from the engine thread;
    ///  the realtime callback reads its clone.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// draining: { value: boolean };
    /// ```
    draining: Arc<AtomicBool>,
}

/// What:
///      `impl Output { ... }`.
///  Methods for the output pipeline.
/// Why:
///       Construction and per-track reconfiguration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Output { /* new, reconfigure, set_playing */ }
/// ```
impl Output {
    /// What:
    ///      `pub fn new(worker: Thread) -> Result<Output, PlayerError>`.
    ///  Initialise
    ///           PipeWire,
    ///  build the loop/context/core,
    ///  and start the loop thread.
    ///  `worker`
    ///           is the engine worker's thread handle,
    ///  taken by value (the caller hands us
    ///           its own clone).
    /// Why:
    ///       One-time setup of the audio pipeline;
    ///  we keep `worker` so per-track
    ///           callbacks can wake the worker on drain.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static create(worker: WorkerRef): Output { ... }
    /// ```
    pub fn new(worker: Thread) -> Result<Output, PlayerError> {
        // What:     `pw::init();`. Initialises the PipeWire library (global C setup). Safe
        //           to call; idempotent in practice.
        // Why:      Required before any other PipeWire call.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // pw.init();
        // ```
        pw::init();

        // What:     `let thread_loop = unsafe { ThreadLoopRc::new(Some("music-player-audio"), None) }.map_err(...)?`.
        //           `ThreadLoopRc::new` is an `unsafe fn` (its safety contract is about
        //           keeping the loop alive correctly, which the `Output` struct guarantees).
        //           `Some("music-player-audio")` names the thread; `None` = no extra
        //           properties. `.map_err` converts a `pw::Error` into our `PlayerError`;
        //           `?` returns early.
        // Why:      Create the background audio loop.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const threadLoop = new ThreadLoop("music-player-audio");
        // ```
        let thread_loop = unsafe { ThreadLoopRc::new(Some("music-player-audio"), None) }
            // What:     `.map_err(|e| PlayerError::Audio(format!("thread loop: {e:?}")))`.
            //           `|e| ...` is a closure; `format!` builds an owned `String`; `{e:?}`
            //           uses the error's Debug formatting.
            // Why:      Wrap the low-level error in our `Audio` variant.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // throws new PlayerError.Audio(`thread loop: ${e}`) on failure
            // ```
            .map_err(|e| return PlayerError::Audio(format!("thread loop: {e:?}")))?;

        // What:     `let context = ContextRc::new(&thread_loop, None).map_err(...)?`.
        //           `ContextRc::new` takes a reference to anything loop-like
        //           (`&thread_loop` lends it) plus optional properties (`None`). Builds the
        //           refcounted context.
        // Why:      The root from which we connect to the server.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const context = new Context(threadLoop);
        // ```
        let context = ContextRc::new(&thread_loop, None)
            .map_err(|e| return PlayerError::Audio(format!("context: {e:?}")))?;

        // What:     `let core = context.connect_rc(None).map_err(...)?`. `connect_rc` opens
        //           the session; `None` = default connection properties.
        // Why:      We need a `Core` to make streams.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const core = context.connect();
        // ```
        let core = context
            .connect_rc(None)
            .map_err(|e| return PlayerError::Audio(format!("core connect: {e:?}")))?;

        // What:     `thread_loop.start();`. Spawns the loop's OS thread and begins
        //           processing. After this, callbacks can fire on that thread.
        // Why:      Bring the audio pipeline to life.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // threadLoop.start();
        // ```
        thread_loop.start();

        // What:     `let playing = Arc::new(AtomicBool::new(false));`. Build the shared
        //           play/pause flag, initially `false` (paused). `Arc::new` heap-allocates
        //           the cell and starts its reference count at 1; `AtomicBool::new(false)`
        //           is the initial value.
        // Why:      A brand-new output is silent until the engine says play.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const playing = { value: false };
        // ```
        let playing = Arc::new(AtomicBool::new(false));

        // What:     `Ok(Output { listener: None, stream: None, core, _context: context, thread_loop, playing, worker })`.
        //           Build the struct (no stream yet) and wrap in `Ok`. Field shorthand for
        //           `core`/`thread_loop`/`playing`/`worker`. Tail -> return.
        // Why:      Hand back the ready (but silent) output.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return new Output(null, null, core, context, threadLoop, playing, worker);
        // ```
        return Ok(Output {
            listener: None,
            stream: None,
            core,
            _context: context,
            thread_loop,
            playing,
            worker,
            meter: Arc::new(AudioMeter::new()),
            draining: Arc::new(AtomicBool::new(false)),
        })
    }

    /// What:
    ///      `pub fn reconfigure(&mut self, rate: u32, channels: u16, capacity_frames: usize) -> Result<HeapProd<f32>, PlayerError>`.
    ///           Tear down any existing stream and build a fresh one negotiating the given
    ///           native `rate`/`channels`,
    ///  with a ring buffer holding `capacity_frames`
    ///           frames.
    ///  Returns the WRITE half of that buffer.
    /// Why:
    ///       Per-track native rate:
    ///  each new track gets a stream at its own rate,
    ///  and a
    ///           fresh empty buffer so no stale audio leaks across.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// reconfigure(rate: number, channels: number, capacityFrames: number): RingProducer { ... }
    /// ```
    pub fn reconfigure(
        &mut self,
        rate: u32,
        channels: u16,
        capacity_frames: usize,
    ) -> Result<HeapProd<f32>, PlayerError> {
        // What:     `let channels_usize = channels as usize;`. Widen `u16` to `usize` for
        //           buffer arithmetic.
        // Why:      Sample counts and strides are `usize`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const channelsUsize = channels;
        // ```
        let channels_usize = channels as usize;

        // What:     `let capacity_samples = capacity_frames * channels_usize;`. Ring-buffer
        //           capacity is in INTERLEAVED SAMPLES, so frames times channels.
        // Why:      Size the buffer to hold `capacity_frames` frames of audio.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const capacitySamples = capacityFrames * channels;
        // ```
        let capacity_samples = capacity_frames * channels_usize;

        // What:     `let rb = HeapRb::<f32>::new(capacity_samples);`. Allocate a ring buffer
        //           of that many `f32` slots. `::<f32>` is the turbofish that pins the
        //           element type.
        // Why:      The shared queue between decode thread and audio thread.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const rb = new RingBuffer<number>(capacitySamples);
        // ```
        let rb = HeapRb::<f32>::new(capacity_samples);

        // What:     `let (producer, consumer) = rb.split();`. `split()` CONSUMES the buffer
        //           and returns its two halves; destructured into the write end `producer`
        //           and the read end `consumer`.
        // Why:      Producer goes to the engine; consumer goes into the callback.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const { producer, consumer } = rb.split();
        // ```
        let (producer, consumer) = rb.split();

        // What:     `let process_data = ProcessData { consumer, channels: channels_usize, scratch: Vec::new(), playing: Arc::clone(&self.playing), worker: self.worker.clone() };`.
        //           Bundle the callback's state. `Vec::new()` is an empty buffer (grows on
        //           first use). `Arc::clone(&self.playing)` makes another handle to the SAME
        //           shared flag (bumps the reference count, does not copy the bool);
        //           `&self.playing` lends the field to clone from. `self.worker.clone()`
        //           clones the worker handle (a refcount bump, not a new thread).
        // Why:      This is moved into the listener as the callback's `&mut` data; the
        //           cloned flag lets the callback see pause/play decisions made by the
        //           engine on the other thread, and the cloned worker lets it `unpark()` on
        //           drain.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const processData = { consumer, channels, scratch: new Float32Array(0), playing, worker };
        // ```
        let process_data = ProcessData {
            consumer,
            channels: channels_usize,
            scratch: Vec::new(),
            playing: Arc::clone(&self.playing),
            worker: self.worker.clone(),
            meter: Arc::clone(&self.meter),
            draining: Arc::clone(&self.draining),
        };

        // What:     `self.draining.store(false, Ordering::Relaxed);`. A new stream starts a new track.
        // Why:      The end-of-track flag of the previous stream must not silence this stream's underrun count.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.draining.value = false;
        // ```
        self.draining.store(false, Ordering::Relaxed);

        // What:     `let guard = self.thread_loop.lock();`. Lock the audio loop: while held,
        //           the loop thread will not touch loop objects, so we can safely
        //           create/destroy streams. The returned `guard` UNLOCKS automatically when
        //           it goes out of scope (even on an early `?` return).
        // Why:      PipeWire requires the loop locked when mutating its objects from another
        //           thread.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const guard = threadLoop.lock(); // released when it falls out of scope
        // ```
        let guard = self.thread_loop.lock();

        // What:     `self.listener = None;`. Drop the previous callback registration FIRST
        //           (it holds a raw pointer into the old stream).
        // Why:      Unregister before destroying the stream it points at.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.listener?.unregister(); this.listener = null;
        // ```
        self.listener = None;

        // What:     `self.stream = None;`. Drop the previous stream (destroys the C object),
        //           now that no listener references it.
        // Why:      Replace it with a fresh stream below.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.stream = null;
        // ```
        self.stream = None;

        // What:     `let stream = StreamRc::new(self.core.clone(), "music-player", properties! { ... }).map_err(...)?`.
        //           Create a new stream on our core, named "music-player", with media
        //           metadata. `properties! { *KEY => "val" }` builds the dictionary; the `*`
        //           dereferences each key constant.
        // Why:      A fresh stream to negotiate this track's format.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const stream = new Stream(core, "music-player", { MEDIA_TYPE: "Audio", MEDIA_CATEGORY: "Playback", MEDIA_ROLE: "Music" });
        // ```
        let stream = StreamRc::new(
            self.core.clone(),
            "music-player",
            properties! {
                *pw::keys::MEDIA_TYPE => "Audio",
                *pw::keys::MEDIA_CATEGORY => "Playback",
                *pw::keys::MEDIA_ROLE => "Music",
            },
        )
        .map_err(|e| return PlayerError::Audio(format!("stream new: {e:?}")))?;

        // What:     `let listener = stream.add_local_listener_with_user_data(process_data).process(|stream_ref, pd| { ... }).register().map_err(...)?`.
        //           Attach our state as the callback's user data, set the `process`
        //           callback, and register it (returns an owned `StreamListener`). The
        //           closure receives `&StreamRef` and `&mut ProcessData`.
        // Why:      This callback is how PipeWire pulls samples from us in realtime.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const listener = stream.onProcess(processData, (stream, pd) => { ...fill buffer... });
        // ```
        let listener = stream
            .add_local_listener_with_user_data(process_data)
            // What:     `.process(|stream_ref, pd| { ... })`. Register the realtime callback.
            //           `|stream_ref, pd|` is the closure's parameter list; the callback's
            //           stream reference keeps its own name so it never shadows the `stream`
            //           being built here. This runs on PipeWire's thread; it must not
            //           block/allocate.
            // Why:      Feed the hardware buffer from our ring buffer.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // (stream, pd) => { ... }
            // ```
            .process(|stream_ref, pd| {
                // What:     `match stream_ref.dequeue_buffer() { ... }`. Ask the stream for a
                //           buffer to fill. Returns `Option<Buffer>`: `Some(buffer)` if one
                //           is available, else `None`.
                // Why:      We can only write when PipeWire gives us a buffer.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const buffer = stream.dequeueBuffer(); if (!buffer) return;
                // ```
                match stream_ref.dequeue_buffer() {
                    // What:     `None => {}`. No buffer right now: do nothing.
                    // Why:      Skip this cycle.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // // no buffer: skip
                    // ```
                    None => {}
                    // What:     `Some(mut buffer) => { ... }`. We got a buffer; `mut` because
                    //           we write into it.
                    // Why:      Fill it with samples.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // else { ... }
                    // ```
                    Some(mut buffer) => {
                        // What:     `let requested_frames = buffer.requested();`. The frame count
                        //           the graph asks for in this cycle (0 when it has not set one).
                        // Why:      Recorded with the buffer size so the log shows the quantum.
                        //
                        // In TS you'd write (pseudocode):
                        // ```ts
                        // const requestedFrames = buffer.requested;
                        // ```
                        let requested_frames = buffer.requested();

                        // What:     `let datas = buffer.datas_mut();`. A buffer can hold
                        //           several data planes; `datas_mut()` borrows them mutably
                        //           as a slice.
                        // Why:      Interleaved audio uses plane 0.
                        //
                        // In TS you'd write (pseudocode):
                        // ```ts
                        // const datas = buffer.datas;
                        // ```
                        let datas = buffer.datas_mut();

                        // What:     `if datas.is_empty() { return; }`. Defensive: no planes
                        //           means nothing to fill. `return` exits the closure (the
                        //           buffer is queued back on drop).
                        // Why:      Avoid indexing an empty slice.
                        //
                        // In TS you'd write (pseudocode):
                        // ```ts
                        // if (datas.length === 0) return;
                        // ```
                        if datas.is_empty() {
                            return;
                        }

                        // What:     `let bdata = &mut datas[0];`. Mutable reference to the
                        //           first data plane.
                        // Why:      The interleaved samples go here.
                        //
                        // In TS you'd write (pseudocode):
                        // ```ts
                        // const bdata = datas[0];
                        // ```
                        let bdata = &mut datas[0];

                        // What:     `let stride = F32_BYTES * pd.channels;`. Bytes per audio
                        //           frame (one sample per channel).
                        // Why:      Used to size and tag the chunk.
                        //
                        // In TS you'd write (pseudocode):
                        // ```ts
                        // const stride = 4 * pd.channels;
                        // ```
                        let stride = F32_BYTES * pd.channels;

                        // What:     `let n_frames = match bdata.data() { ... };`.
                        //           `bdata.data()` returns `Option<&mut [u8]>`: the mapped
                        //           byte buffer to write, or `None`.
                        // Why:      Fill the bytes and report how many frames we wrote.
                        //
                        // In TS you'd write (pseudocode):
                        // ```ts
                        // const slice = bdata.data; const nFrames = slice ? fill(slice) : 0;
                        // ```
                        let n_frames = match bdata.data() {
                            // What:     `Some(slice) => { ... }`. `slice: &mut [u8]` is the
                            //           writable hardware buffer bytes.
                            // Why:      Copy decoded samples into it.
                            //
                            // In TS you'd write (pseudocode):
                            // ```ts
                            // if (slice) { ... }
                            // ```
                            Some(slice) => {
                                // What:     `let avail_frames = slice.len() / stride;`. How
                                //           many whole frames fit in this buffer.
                                // Why:      The upper bound for the frames this callback fills.
                                //
                                // In TS you'd write (pseudocode):
                                // ```ts
                                // const availFrames = Math.floor(slice.length / stride);
                                // ```
                                let avail_frames = slice.len() / stride;

                                // What:     `let wanted_frames = usize::try_from(requested_frames).map_or(FILL_FRAMES_FLOOR, |requested| return requested.max(FILL_FRAMES_FLOOR));`
                                //           then `let fill_frames = avail_frames.min(wanted_frames);`. Fill what the graph
                                //           asks for this cycle, but at least `FILL_FRAMES_FLOOR` frames, and never more than
                                //           fits in the buffer.
                                // Why:      The buffer PipeWire hands over holds about a quarter of a second.
                                //           Filling all of it moved that much audio out of the ring in one
                                //           callback, so the audible position trailed what was taken from
                                //           the ring by up to that much, and a stream that started with an
                                //           empty ring committed a quarter second of silence at once.
                                //           Filling only one graph cycle would make every missed deadline of
                                //           this thread an audible skip; the floor keeps tens of milliseconds
                                //           of margin while the position error stays below the floor.
                                //
                                // In TS you'd write (pseudocode):
                                // ```ts
                                // const fillFrames = Math.min(availFrames, Math.max(requestedFrames, FILL_FRAMES_FLOOR));
                                // ```
                                let wanted_frames =
                                    usize::try_from(requested_frames).map_or(FILL_FRAMES_FLOOR, |requested| return requested.max(FILL_FRAMES_FLOOR));
                                let fill_frames = avail_frames.min(wanted_frames);

                                // What:     `let want = fill_frames * pd.channels;`.
                                //           Interleaved sample count to produce.
                                // Why:      Total f32 values to pop/convert.
                                //
                                // In TS you'd write (pseudocode):
                                // ```ts
                                // const want = fillFrames * pd.channels;
                                // ```
                                let want = fill_frames * pd.channels;

                                // What:     `pd.meter.record_buffer(requested_frames, fill_frames as u64);`.
                                //           Store the requested and filled frame counts.
                                // Why:      The engine thread logs them, so the quantum and the
                                //           frames actually filled can be checked against each other.
                                //
                                // In TS you'd write (pseudocode):
                                // ```ts
                                // meter.recordBuffer(requestedFrames, fillFrames);
                                // ```
                                pd.meter.record_buffer(requested_frames, fill_frames as u64);

                                // What:     `if pd.scratch.len() < want { pd.scratch.resize(want, 0.0); }`.
                                //           Grow the scratch buffer if needed, filling new
                                //           slots with `0.0`.
                                // Why:      Have room to pop `want` samples (grows at most
                                //           once after sizes stabilise).
                                //
                                // In TS you'd write (pseudocode):
                                // ```ts
                                // if (pd.scratch.length < want) pd.scratch = new Float32Array(want);
                                // ```
                                if pd.scratch.len() < want {
                                    pd.scratch.resize(want, 0.0);
                                }

                                // What:     `let playing = pd.playing.load(Ordering::Relaxed);`.
                                //           Read the shared pause/play flag. `.load(...)` is
                                //           the atomic READ; `Relaxed` is the loosest
                                //           ordering (we only need the value itself to be
                                //           read indivisibly, with no ordering guarantee
                                //           against other memory, which is correct for a lone
                                //           flag).
                                // Why:      Decide, on the realtime thread, whether to feed
                                //           real audio or silence this cycle.
                                //
                                // In TS you'd write (pseudocode):
                                // ```ts
                                // const playing = Atomics.load(playingFlag, 0) !== 0;
                                // ```
                                let playing = pd.playing.load(Ordering::Relaxed);

                                // What:     `let got = if playing { pd.consumer.pop_slice(&mut pd.scratch[..want]) } else { 0 };`.
                                //           When playing, pop up to `want` samples from the
                                //           ring buffer (returns how many were available, a
                                //           `usize`). When paused, pop NOTHING (`0`) so the
                                //           buffered audio is preserved for a seamless
                                //           resume.
                                // Why:      Pausing must stop draining the buffer at once;
                                //           otherwise the ~1 second already queued keeps
                                //           playing (the pause-delay bug).
                                //
                                // In TS you'd write (pseudocode):
                                // ```ts
                                // const got = playing ? pd.consumer.popSlice(pd.scratch.subarray(0, want)) : 0;
                                // ```
                                let got = if playing {
                                    pd.consumer.pop_slice(&mut pd.scratch[..want])
                                } else {
                                    0
                                };

                                // What:     `if got > 0 { pd.worker.unpark(); }`. When we
                                //           popped at least one sample we freed space in the
                                //           ring buffer; `unpark()` wakes the engine worker
                                //           (or leaves a one-shot permit if it is not parked
                                //           yet) so it decodes more to refill that space.
                                // Why:      This is the backpressure signal that lets the
                                //           worker BLOCK when the buffer is full instead of
                                //           busy-looping. `unpark` is wait-free (an atomic
                                //           plus a futex wake on Linux): no lock, no
                                //           allocation, so it is safe to call from this
                                //           realtime callback.
                                //
                                // In TS you'd write (pseudocode):
                                // ```ts
                                // if (got > 0) worker.postWakeUp();
                                // ```
                                if got > 0 {
                                    pd.worker.unpark();
                                }

                                // What:     `if playing && got < want { pd.meter.record_short(...); }`.
                                //           When audio should be flowing but the ring buffer held
                                //           fewer samples than this buffer needs, count the callback
                                //           and the silent frames that will be written below.
                                // Why:      This is the ring-buffer underrun that the stutter
                                //           analysis is built on.
                                //
                                // In TS you'd write (pseudocode):
                                // ```ts
                                // if (playing && got < want) meter.recordShort((want - got) / channels);
                                // ```
                                //
                                // What:     `&& !pd.draining.load(Ordering::Relaxed)`. After the engine
                                //           pushed the last frame of the track, a short buffer is the end
                                //           of the music, so it is not counted.
                                // Why:      Otherwise every track end would log "audio output underran".
                                if playing && got < want && !pd.draining.load(Ordering::Relaxed) {
                                    pd.meter.record_short(((want - got) / pd.channels) as u64);
                                }

                                // What:     `for s in pd.scratch[got..want].iter_mut() { *s = 0.0; }`.
                                //           Zero every sample we did NOT fill. `iter_mut`
                                //           yields mutable references `s`; `*s` writes
                                //           through the reference. When paused `got` is `0`,
                                //           so this zeroes the WHOLE range: the callback
                                //           emits pure silence.
                                // Why:      Output silence on underrun (and while paused)
                                //           instead of stale/garbage data.
                                //
                                // In TS you'd write (pseudocode):
                                // ```ts
                                // for (let i = got; i < want; i++) pd.scratch[i] = 0;
                                // ```
                                for s in pd.scratch[got..want].iter_mut() {
                                    *s = 0.0;
                                }

                                // What:     `for (i, sample) in pd.scratch[..want].iter().enumerate() { ... }`.
                                //           Iterate the samples with their index `i`.
                                //           `.iter()` borrows them read-only; `.enumerate()`
                                //           pairs each with its index.
                                // Why:      Write each f32 as 4 little-endian bytes into the
                                //           hardware buffer.
                                //
                                // In TS you'd write (pseudocode):
                                // ```ts
                                // pd.scratch.slice(0, want).forEach((sample, i) => { ... });
                                // ```
                                for (i, sample) in pd.scratch[..want].iter().enumerate() {
                                    // What:     `let bytes = sample.to_le_bytes();`.
                                    //           `to_le_bytes()` converts the `f32` into a
                                    //           `[u8; 4]` in little-endian order.
                                    // Why:      The buffer format is F32LE bytes.
                                    //
                                    // In TS you'd write (pseudocode):
                                    // ```ts
                                    // dataView.setFloat32(off, sample, true);
                                    // ```
                                    let bytes = sample.to_le_bytes();

                                    // What:     `let off = i * F32_BYTES;`. Byte offset of
                                    //           this sample in the buffer.
                                    // Why:      Where to write the 4 bytes.
                                    //
                                    // In TS you'd write (pseudocode):
                                    // ```ts
                                    // const off = i * 4;
                                    // ```
                                    let off = i * F32_BYTES;

                                    // What:     `slice[off..off + F32_BYTES].copy_from_slice(&bytes);`.
                                    //           Copy the 4 bytes into the buffer at `off`.
                                    //           `&bytes` lends the array.
                                    // Why:      Place the sample.
                                    //
                                    // In TS you'd write (pseudocode):
                                    // ```ts
                                    // slice.set(bytes, off);
                                    // ```
                                    slice[off..off + F32_BYTES].copy_from_slice(&bytes);
                                }

                                // What:     `fill_frames`. Tail of the arm: the number of
                                //           frames we filled becomes `n_frames`.
                                // Why:      Report the frame count.
                                //
                                // In TS you'd write (pseudocode):
                                // ```ts
                                // return fillFrames;
                                // ```
                                fill_frames
                            }
                            // What:     `None => 0`. No mapped buffer: zero frames.
                            // Why:      Nothing written.
                            //
                            // In TS you'd write (pseudocode):
                            // ```ts
                            // else nFrames = 0;
                            // ```
                            None => 0,
                        };

                        // What:     `let chunk = bdata.chunk_mut();`. The "chunk" is the
                        //           metadata describing the valid region of the buffer;
                        //           `chunk_mut()` borrows it mutably.
                        // Why:      We must tell PipeWire how much we wrote.
                        //
                        // In TS you'd write (pseudocode):
                        // ```ts
                        // const chunk = bdata.chunk;
                        // ```
                        let chunk = bdata.chunk_mut();

                        // What:     `*chunk.offset_mut() = 0;`. `offset_mut()` returns a
                        //           mutable reference to the offset field; `*... =` writes
                        //           through it. Data starts at byte 0.
                        // Why:      No leading padding.
                        //
                        // In TS you'd write (pseudocode):
                        // ```ts
                        // chunk.offset = 0;
                        // ```
                        *chunk.offset_mut() = 0;

                        // What:     `*chunk.stride_mut() = stride as _;`. Set the per-frame
                        //           byte stride. `as _` lets the compiler infer the exact
                        //           integer type the field needs.
                        // Why:      PipeWire needs the frame size.
                        //
                        // In TS you'd write (pseudocode):
                        // ```ts
                        // chunk.stride = stride;
                        // ```
                        *chunk.stride_mut() = stride as _;

                        // What:     `*chunk.size_mut() = (stride * n_frames) as _;`. Total
                        //           valid bytes = stride times frames written.
                        // Why:      Tells PipeWire how many bytes to play.
                        //
                        // In TS you'd write (pseudocode):
                        // ```ts
                        // chunk.size = stride * nFrames;
                        // ```
                        *chunk.size_mut() = (stride * n_frames) as _;
                    }
                }
            })
            // What:     `.register()`. Finish building and register the listener; returns
            //           `Result<StreamListener<ProcessData>, pw::Error>`.
            // Why:      Activate the callbacks on the stream.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // .register()
            // ```
            .register()
            .map_err(|e| return PlayerError::Audio(format!("listener: {e:?}")))?;

        // What:     `let mut info = AudioInfoRaw::new();`. Build the format descriptor. `mut`
        //           because we set fields next.
        // Why:      Describe the audio we will send.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const info = new AudioInfoRaw();
        // ```
        let mut info = AudioInfoRaw::new();

        // What:     `info.set_format(AudioFormat::F32LE);`. Interleaved 32-bit little-endian
        //           float samples. Sibling formats: `S16LE`, etc.
        // Why:      Our decoders output `f32`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // info.format = "F32LE";
        // ```
        info.set_format(AudioFormat::F32LE);

        // What:     `info.set_rate(rate);`. Samples per second for THIS track.
        // Why:      Per-track native rate; PipeWire resamples to the device.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // info.rate = rate;
        // ```
        info.set_rate(rate);

        // What:     `info.set_channels(channels as u32);`. Channel count; the API wants
        //           `u32`, so widen our `u16`.
        // Why:      Stereo/mono layout.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // info.channels = channels;
        // ```
        info.set_channels(channels as u32);

        // What:     `let values: Vec<u8> = PodSerializer::serialize(Cursor::new(Vec::new()), &Value::Object(Object { type_, id, properties: info.into() })).map_err(...)?.0.into_inner();`.
        //           Serialize the format object into bytes. `Value::Object` wraps an
        //           `Object` whose `type_` and `id` are SPA constants (object is a Format /
        //           EnumFormat). `info.into()` converts the `AudioInfoRaw` into the object's
        //           property list. `serialize(...)` returns `Result<(Cursor, ...)>`; `?`
        //           unwraps; `.0` takes the cursor; `.into_inner()` extracts the `Vec<u8>`.
        // Why:      Stream parameters must be passed as serialized pod bytes.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const values = encodeFormatPod({ type: "Format", id: "EnumFormat", info });
        // ```
        let values: Vec<u8> = PodSerializer::serialize(
            Cursor::new(Vec::new()),
            &Value::Object(Object {
                // What:     `type_: pw::spa::sys::SPA_TYPE_OBJECT_Format`. The raw SPA
                //           constant marking this object as a "Format" object. `pw::spa::sys`
                //           is the raw C-binding submodule.
                // Why:      PipeWire identifies the object kind by this tag.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // type: SPA_TYPE_OBJECT_Format,
                // ```
                type_: pw::spa::sys::SPA_TYPE_OBJECT_Format,
                // What:     `id: pw::spa::sys::SPA_PARAM_EnumFormat`. Marks the parameter as
                //           the set of formats we can accept.
                // Why:      Stream negotiation reads `EnumFormat`.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // id: SPA_PARAM_EnumFormat,
                // ```
                id: pw::spa::sys::SPA_PARAM_EnumFormat,
                // What:     `properties: info.into()`. `.into()` converts the `AudioInfoRaw`
                //           into the `Vec<Property>` the object wants.
                // Why:      The format fields become the object's properties.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // properties: infoToProps(info),
                // ```
                properties: info.into(),
            }),
        )
        .map_err(|e| return PlayerError::Audio(format!("pod serialize: {e:?}")))?
        .0
        .into_inner();

        // What:     `let pod = Pod::from_bytes(&values).ok_or_else(|| ...)?`.
        //           `Pod::from_bytes` returns `Option<&Pod>` (a borrowed view over `values`);
        //           `.ok_or_else(...)` turns `None` into an error; `?` unwraps the `&Pod`.
        // Why:      `connect` takes pod references, not raw bytes.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const pod = Pod.fromBytes(values); if (!pod) throw new PlayerError("invalid format pod");
        // ```
        let pod = Pod::from_bytes(&values)
            .ok_or_else(|| return PlayerError::Audio("invalid format pod".to_string()))?;

        // What:     `let mut params = [pod];`. A fixed array of one pod reference; `mut`
        //           because `connect` takes `&mut [&Pod]`.
        // Why:      The parameter list for stream negotiation.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const params = [pod];
        // ```
        let mut params = [pod];

        // What:     `stream.connect(Direction::Output, None, StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS | StreamFlags::RT_PROCESS, &mut params).map_err(...)?`.
        //           Connect the stream as an output. `None` = auto-pick a target sink. The
        //           flags: AUTOCONNECT (link to a sink automatically), MAP_BUFFERS
        //           (memory-map buffers so our callback gets a byte slice), RT_PROCESS (run
        //           `process` on the realtime thread). `|` combines the bit flags.
        //           `&mut params` lends the param list.
        // Why:      Start streaming audio at the negotiated format.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // stream.connect("output", null, AUTOCONNECT | MAP_BUFFERS | RT_PROCESS, params);
        // ```
        stream
            .connect(
                Direction::Output,
                None,
                StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS | StreamFlags::RT_PROCESS,
                &mut params,
            )
            .map_err(|e| return PlayerError::Audio(format!("connect: {e:?}")))?;

        // What:     `self.stream = Some(stream);`. Store the new stream (moves it into the
        //           struct; the C object it points to stays put, so the listener's pointer
        //           remains valid).
        // Why:      Keep it alive while playing this track.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.stream = stream;
        // ```
        self.stream = Some(stream);

        // What:     `self.listener = Some(listener);`. Store the registration so the
        //           callbacks stay active.
        // Why:      Dropping it would unregister the callback.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.listener = listener;
        // ```
        self.listener = Some(listener);

        // What:     `drop(guard);`. Explicitly release the loop lock now (it would also
        //           release at end of scope; doing it here is clearer).
        // Why:      Let the audio loop resume processing.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // guard.unlock();
        // ```
        drop(guard);

        // What:     `Ok(producer)`. Return the WRITE half of the ring buffer. Tail -> return.
        // Why:      The engine pushes decoded samples into it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return producer;
        // ```
        return Ok(producer)
    }

    /// What:
    ///      `pub fn set_playing(&self, on: bool)`.
    ///  Flip the shared pause/play flag.
    ///           Takes `&self` (read-only borrow) because writing an atomic does NOT need
    ///           exclusive access;
    ///  the cell handles concurrent writes.
    /// Why:
    ///       The engine calls this on pause/play so the realtime callback can react
    ///           immediately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// setPlaying(on: boolean): void { Atomics.store(playingFlag, 0, on ? 1 : 0); }
    /// ```
    pub fn set_playing(&self, on: bool) {
        // What:     `self.playing.store(on, Ordering::Relaxed);`. The atomic WRITE: store
        //           `on` into the shared flag. `Relaxed` matches the loose ordering used by
        //           the callback's `.load` (a lone flag needs no stronger guarantee).
        // Why:      Make the new state visible to the audio thread.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // Atomics.store(playingFlag, 0, on ? 1 : 0);
        // ```
        self.playing.store(on, Ordering::Relaxed);
    }

    /// What:
    ///      `pub fn set_draining(&self, on: bool)`.
    ///  Tell the output the decoder pushed the last frame of the track,
    ///  so the empty buffers that follow are not counted as underruns.
    ///  `reconfigure` clears it.
    /// Why:
    ///       The end of a track is silence by design;
    ///  counting it would make the underrun log cry wolf at every track change.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// setDraining(on: boolean): void { Atomics.store(drainingFlag, 0, on ? 1 : 0); }
    /// ```
    pub fn set_draining(&self, on: bool) {
        // What:     `self.draining.store(on, Ordering::Relaxed);`. Atomic write of the flag.
        // Why:      Make the new state visible to the audio thread.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // Atomics.store(drainingFlag, 0, on ? 1 : 0);
        // ```
        self.draining.store(on, Ordering::Relaxed);
    }

    /// What:
    ///      `pub fn meter_snapshot(&self) -> MeterSnapshot`.
    ///  Reads the underrun counters the realtime callback has written so far.
    /// Why:
    ///       The engine thread calls it once a second and logs what changed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// meterSnapshot(): MeterSnapshot { ... }
    /// ```
    pub fn meter_snapshot(&self) -> MeterSnapshot {
        return self.meter.snapshot();
    }
}

/// What:
///      `impl Drop for Output { ... }`.
///  Custom cleanup when an `Output` is dropped
///           (goes out of scope).
///  `Drop` is the destructor trait.
/// Why:
///       Stop the loop thread BEFORE the fields (stream/core/...) are destroyed,
///  so no
///           realtime callback runs during teardown.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Output { [Symbol.dispose]() { this.threadLoop.stop(); } }
/// ```
impl Drop for Output {
    /// What:
    ///      `fn drop(&mut self)`.
    ///  Runs once,
    ///  automatically,
    ///  at end of life.
    /// Why:
    ///       Stop the audio loop first.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// [Symbol.dispose]() { ... }
    /// ```
    fn drop(&mut self) {
        // What:     `self.thread_loop.stop();`. Stops the loop and joins its OS thread.
        //           After this no callbacks fire.
        // Why:      Make destroying the stream/core safe (no concurrent access).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.threadLoop.stop();
        // ```
        self.thread_loop.stop();
        // After this method returns, the struct fields drop in declaration order:
        // listener (unregister) -> stream (destroy) -> core -> context -> loop.
    }
}
