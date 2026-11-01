//! Native playback engine:
//!  the per-track primitive behind Kotlin's `AudioEngine`.
//! A single worker thread owns the decode `Source`,
//!  the SPSC ring producer,
//!  and
//! the AAudio output stream,
//!  and processes `Command`s;
//!  the realtime AAudio callback
//! only pops samples,
//!  gates on the play flag,
//!  applies volume,
//!  and advances the
//! played-frame counter.
//!  Kotlin drives it over JNI and POLLS state from `Control`'s
//! atomics (no native-to-JVM callbacks).
//!  Ports the desktop engine's ringbuf + worker
//! model to AAudio;
//!  the queue/advance/shuffle stay in Kotlin's PlayerController.
//!
//! Threads,
//!  for a TypeScript reader:
//!  `Engine::new` starts a background worker
//! (`std::thread`).
//!  Kotlin (running on the JVM,
//!  calling in over JNI) talks to it two
//! ways.
//!  Slow,
//!  owned-state commands (load a file,
//!  seek,
//!  quit) go down a one-way queue
//! (an `mpsc` channel:
//!  many senders,
//!  one receiver).
//!  Fast,
//!  per-sample knobs (play,
//! pause,
//!  volume,
//!  normalization gain) skip the queue and write shared ATOMIC fields
//! in `Control` directly,
//!  so the next audio callback observes them with no round trip.
//! Unlike the desktop engine,
//!  there is NO callback into the caller:
//!  Kotlin polls the
//! same `Control` atomics for position/duration/ended.
//!  The atomics are the entire
//! shared surface;
//!  picture them as a small struct of values updated and read by three
//! threads at once (JNI/main,
//!  the decode worker,
//!  the realtime AAudio callback) without
//! a lock.

/// What:
///      `use std::os::fd::{BorrowedFd, RawFd};`.
///  Two file-descriptor types from
///           the standard library's Unix fd module.
///  A "file descriptor" is the small
///           integer the OS hands you to refer to an open file/socket/pipe.
///  `RawFd` is
///           literally that integer (a bare `i32`) with NO ownership meaning:
///  holding
///           one does not keep the file open and dropping one does not close it.
///           `BorrowedFd<'fd>` is a typed,
///  lifetime-tagged BORROW of an fd you do not
///           own (you may use it but must not close it).
///  Sibling you might expect:
///           `OwnedFd`,
///  which DOES own the fd and closes it on drop;
///  we never construct
///           one of those directly here,
///  we go through `std::fs::File` instead.
/// Why:
///       Kotlin passes the audio file's fd in as a plain integer over JNI;
///  we need
///           `RawFd` to receive it and `BorrowedFd` to wrap it safely before duplicating.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RawFd = number; // the OS handle integer
/// type BorrowedFd = number; // same integer, but "do not close this yourself"
/// ```
use std::os::fd::{BorrowedFd, RawFd};

/// What:
///      `use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};`.
///           Lock-free shared cells plus the memory-ordering enum.
///  An `AtomicBool` /
///           `AtomicU32` / `AtomicU64` is a `bool` / 32-bit / 64-bit unsigned integer
///           that MULTIPLE THREADS may read and write at the same time without a lock
///           or a data race;
///  reads and writes go through `.load()` / `.store()` methods,
///           never `=`.
///  Siblings the reader might expect:
///  `AtomicI32`/`AtomicI64`
///           (signed),
///  `AtomicUsize` (pointer-width);
///  we pick the unsigned fixed-width
///           ones because the values are sample rates,
///  channel counts,
///  frame counters,
///           and bit-patterns of `f32`s,
///  all naturally non-negative.
///  `Ordering` is an
///           enum picking how strongly a given atomic access synchronizes with others
///           (its variants `Relaxed`/`Acquire`/`Release` appear below).
/// Why:
///       Three threads (JNI/main,
///  decode worker,
///  realtime callback) share playback
///           state with no lock so the realtime path never blocks;
///  atomics are how they
///           hand values across thread boundaries safely.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // no equivalent: JS is single-threaded.
/// // mentally: shared cells read/written via Atomics.load / Atomics.store,
/// // never plain assignment, with an Ordering arg saying how synchronized.
/// ```
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

/// What:
///      `use std::sync::mpsc::{self, Sender};`.
///  The multi-producer/single-consumer
///           channel:
///  a thread-safe one-way queue.
///  `Sender` is the push end;
///  the matching
///           pop end is `Receiver` (we do not name it here because only the worker uses
///           it,
///  inside `engine_worker`).
///  `self` also imports the `mpsc` module itself so
///           we can call `mpsc::channel()` below.
/// Why:
///       The JNI/main thread sends slow,
///  owned-state `Command`s (load/seek/quit) to
///           the worker thread through this queue.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a thread-safe queue; we keep only the Sender (push) end here.
/// ```
use std::sync::mpsc::{self, Sender};

/// What:
///      `use std::sync::Arc;`.
///  `Arc<T>` = "Atomically Reference-Counted" shared
///           pointer:
///  a value of `T` lives on the heap,
///  and every `Arc` clone is another
///           owner;
///  the `T` is freed only when the LAST `Arc` is dropped,
///  and the counter
///           is updated atomically so clones can live on different threads.
///  Siblings the
///           reader might expect:
///  `Rc<T>`,
///  the same idea but with a NON-atomic counter,
///           usable only within one thread (cheaper,
///  but would be rejected here because
///           we cross threads);
///  `Box<T>`,
///  a single-owner heap pointer with no sharing.
/// Why:
///       The one `Control` struct must be shared by three threads at once,
///  so we wrap
///           it in `Arc` and hand each thread its own clone of the pointer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // no wrapper: a plain object reference is already shared and GC-freed.
/// ```
use std::sync::Arc;

/// What:
///      `use std::thread::{self, JoinHandle};`.
///  `thread::spawn`/`thread::Builder`
///           start a worker;
///  a `JoinHandle<T>` lets us later wait for that worker to
///           finish (and recover its return value of type `T`).
///  `self` imports the
///           `thread` module itself so we can call `thread::Builder::new()`.
/// Why:
///       The engine runs decode/output on its own OS thread;
///  `Drop` joins it on exit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // JoinHandle ~ a Worker + a promise that resolves when the worker exits.
/// ```
use std::thread::{self, JoinHandle};

/// What:
///      `use crate::engine_worker;`.
///  Import a SIBLING MODULE in this same crate
///           (not a single name) so we can call `engine_worker::worker_run(...)` below.
///           `crate::` means "from the root of this crate" (this package).
/// Why:
///       `new` spawns the worker by handing the channel receiver and shared control
///           to `worker_run`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as engineWorker from "./engine_worker";
/// ```
use crate::engine_worker;

/// What:
///      `use crate::error::PlayerError;`.
///  Import the crate's one unified error enum
///           (defined in `error.rs`) so this file can return and construct it.
/// Why:
///       `load` returns `Result<(), PlayerError>` and builds a `PlayerError` when the
///           worker has gone away.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { PlayerError } from "./error";
/// ```
use crate::error::PlayerError;

/// What:
///      `pub(crate) const MILLIS_PER_SEC: f64 = 1000.0;`.
///  A compile-time constant
///           named `MILLIS_PER_SEC`.
///  `pub(crate)` = visible to every module in THIS crate
///           but not to outside crates (sibling visibilities:
///  bare `pub` is fully public,
///           no modifier is private to this module).
///  `f64` is a 64-bit IEEE-754 floating
///           point number (Rust's "double");
///  its sibling `f32` is the 32-bit "single".
/// Why:
///       We pick `f64` (not `f32`) because durations are computed and divided in
///           seconds where the extra precision avoids drift;
///  the constant converts the
///           worker's millisecond duration into the seconds Kotlin reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MILLIS_PER_SEC = 1000; // TS number is already an f64
/// ```
pub(crate) const MILLIS_PER_SEC: f64 = 1000.0;

/// What:
///      `pub(crate) struct Control { ... }`.
///  A record type holding the shared
///           lock-free playback state and telemetry.
///  Every field is an atomic (see the
///           imports above),
///  so any of the three threads may touch any field at any time
///           without a lock.
///  `pub(crate)` = visible crate-wide,
///  not to outside crates.
/// Why:
///       This is the ENTIRE shared surface between threads:
///  the worker writes track
///           facts,
///  the realtime callback writes progress,
///  the main thread writes
///           play/volume intent,
///  and Kotlin polls everything.
///  One atomic-only struct keeps
///           the realtime audio path off any lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Control {
///   playing: SharedBool; volumeBits: SharedU32; rate: SharedU32; channels: SharedU32;
///   startFrame: SharedU64; framesPlayed: SharedU64; durationMs: SharedU64;
///   decodeDone: SharedBool; ended: SharedBool; normGainBits: SharedU32;
/// }
/// ```
pub(crate) struct Control {
    /// What:
    ///      `pub(crate) playing: AtomicBool`.
    ///  A thread-safe boolean "play gate":
    ///           true means the callback drains the ring buffer to the speaker,
    ///  false
    ///           means it emits silence.
    ///  `AtomicBool` (not a plain `bool`) so the
    ///           realtime callback can read it while the main thread flips it.
    /// Why:
    ///       Play/pause must take effect on the very next audio buffer with no queue
    ///           round trip,
    ///  so it lives here as an atomic rather than as a `Command`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// playing: SharedBool;
    /// ```
    pub(crate) playing: AtomicBool,
    /// What:
    ///      `pub(crate) volume_bits: AtomicU32`.
    ///  The user volume stored as the RAW
    ///           32 BITS of an `f32`,
    ///  not as a number you can do math on.
    ///  `f32` has no
    ///           atomic type,
    ///  so we keep its bit-pattern in an `AtomicU32` (sibling
    ///           `AtomicU64` would waste half the cell) and reinterpret it with
    ///           `f32::from_bits` when reading.
    /// Why:
    ///       The realtime callback multiplies every sample by this gain;
    ///  storing it as
    ///           atomic bits lets the main thread change volume mid-playback losslessly.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// volumeBits: SharedU32; // raw bits of an f32 gain
    /// ```
    pub(crate) volume_bits: AtomicU32,
    /// What:
    ///      `pub(crate) rate: AtomicU32`.
    ///  The loaded track's output sample rate in
    ///           Hz,
    ///  or 0 when nothing is loaded.
    ///  `AtomicU32` (a 32-bit unsigned integer;
    ///           siblings `AtomicU64`/`AtomicUsize`) is plenty:
    ///  sample rates are small
    ///           positive numbers like 44100 or 48000.
    /// Why:
    ///       `position_sec` divides the played-frame count by this rate to get seconds,
    ///           and the worker writes it on load/seek.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// rate: SharedU32; // Hz, 0 = nothing loaded
    /// ```
    pub(crate) rate: AtomicU32,
    /// What:
    ///      `pub(crate) channels: AtomicU32`.
    ///  The loaded track's channel count
    ///           (1 = mono,
    ///  2 = stereo,
    ///  ...).
    ///  Same `AtomicU32` choice and reasoning as
    ///           `rate`:
    ///  a small non-negative integer shared across threads.
    /// Why:
    ///       The output and frame math need to know how many samples make one frame.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// channels: SharedU32;
    /// ```
    pub(crate) channels: AtomicU32,
    /// What:
    ///      `pub(crate) start_frame: AtomicU64`.
    ///  The frame index the current stream
    ///           STARTED at (the seek target),
    ///  added to `frames_played` to get the true
    ///           position.
    ///  `AtomicU64` (a 64-bit unsigned integer;
    ///  sibling `AtomicU32`
    ///           would overflow) because frame counts on a long track can exceed 4 billion.
    /// Why:
    ///       After a seek we restart the played-frame counter at zero,
    ///  so we must add
    ///           the seek base back to report an absolute position.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// startFrame: SharedU64; // seek base, added to framesPlayed
    /// ```
    pub(crate) start_frame: AtomicU64,
    /// What:
    ///      `pub(crate) frames_played: AtomicU64`.
    ///  How many frames the realtime
    ///           callback has ACTUALLY played since the last load/seek.
    ///  `AtomicU64` for the
    ///           same overflow reason as `start_frame` (a 32-bit sibling would wrap).
    /// Why:
    ///       The callback bumps this each buffer;
    ///  `position_sec` reads it to report how
    ///           far into the track we are.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// framesPlayed: SharedU64;
    /// ```
    pub(crate) frames_played: AtomicU64,
    /// What:
    ///      `pub(crate) duration_ms: AtomicU64`.
    ///  The loaded track's total length in
    ///           MILLISECONDS,
    ///  or 0 when unknown.
    ///  `AtomicU64` because a long track in
    ///           milliseconds easily exceeds the `AtomicU32` (~4.29 million ms ≈ 71 min)
    ///           ceiling.
    /// Why:
    ///       Kotlin polls this (converted to seconds via `MILLIS_PER_SEC`) for the
    ///           progress bar's total length.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// durationMs: SharedU64; // 0 = unknown
    /// ```
    pub(crate) duration_ms: AtomicU64,
    /// What:
    ///      `pub(crate) decode_done: AtomicBool`.
    ///  Set true by the WORKER when the
    ///           decoder hits end-of-file or errors.
    ///  `AtomicBool` so the realtime callback
    ///           can observe it without a lock.
    /// Why:
    ///       The callback needs to know "no more samples are coming" so it can decide
    ///           the track ended once the ring buffer also drains.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// decodeDone: SharedBool;
    /// ```
    pub(crate) decode_done: AtomicBool,
    /// What:
    ///      `pub(crate) ended: AtomicBool`.
    ///  Set true by the CALLBACK once
    ///           `decode_done` is true AND the ring buffer has drained,
    ///  i.e. the track
    ///           truly finished sounding.
    ///  `AtomicBool` shared with the main thread's poller.
    /// Why:
    ///       Kotlin polls this to fire its one `onTrackEnded`;
    ///  separating it from
    ///           `decode_done` means "decoder finished" and "audio finished" are distinct.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// ended: SharedBool;
    /// ```
    pub(crate) ended: AtomicBool,
    /// What:
    ///      `pub(crate) norm_gain_bits: AtomicU32`.
    ///  The per-track true-peak
    ///           normalization gain,
    ///  stored (like `volume_bits`) as the RAW BITS of an
    ///           `f32` inside an `AtomicU32`,
    ///  since `f32` itself has no atomic form.
    /// Why:
    ///       The callback multiplies each sample by this gain alongside the user volume,
    ///           so loud and quiet tracks come out at a matched loudness.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// normGainBits: SharedU32; // raw bits of an f32 normalization gain
    /// ```
    pub(crate) norm_gain_bits: AtomicU32,
    /// What:
    ///      `pub(crate) failed_load: AtomicU64`.
    ///  The 1-based sequence number of the most recent `Command::Load` whose
    ///           file the worker could NOT open for playback (it did not decode, had no rate or
    ///           channels, or no output could be built), or 0 when none has failed.
    ///  Written by the worker,
    ///  read by Kotlin.
    /// Why:
    ///       `nativeEngineLoad` returns as soon as the load is queued,
    ///  so a decode that fails later on the
    ///           worker thread has no other way back to Kotlin;
    ///  the sequence number lets Kotlin tell the load it
    ///           is waiting for from an older one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// failedLoad: SharedU64; // 0 = none failed
    /// ```
    pub(crate) failed_load: AtomicU64,
}

/// What:
///      `impl Control { ... }`.
///  The methods that belong to the `Control` struct (its
///           constructor and two convenience readers).
///  An `impl` block is where Rust hangs
///           a type's methods,
///  separate from the field declarations above.
/// Why:
///       Group `Control`'s construction and its float-reading helpers with the type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Control { static new(): Control; volume(): number; normGain(): number; }
/// ```
impl Control {
    /// What:
    ///      `pub(crate) fn new() -> Control`.
    ///  An associated CONSTRUCTOR function (no
    ///           `self` parameter) that builds and returns a fresh `Control`.
    ///  The `->
    ///           Control` is the return type;
    ///  `pub(crate)` keeps it crate-visible.
    /// Why:
    ///       Give the engine one place to spin up the shared state with sane defaults.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static new(): Control { ... }
    /// ```
    pub(crate) fn new() -> Control {
        // What:     `Control { ... }`. A struct literal that constructs the value field by
        //           field. There is no trailing `;`, so this whole expression is the
        //           function's TAIL EXPRESSION and becomes the return value.
        // Why:      Hand back a fully-initialized control with unity volume/gain and
        //           nothing loaded yet.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return new Control(/* all fields below */);
        // ```
        return Control {
            // What:     `playing: AtomicBool::new(false)`. `AtomicBool::new(false)` is the
            //           WRAPPER CONSTRUCTOR that builds an atomic boolean cell holding
            //           `false`. The `::new` is an associated function (like a static
            //           factory), not a method on an existing value.
            // Why:      Start paused: nothing should sound until Kotlin asks to play.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // playing: makeSharedBool(false),
            // ```
            playing: AtomicBool::new(false),
            // What:     `volume_bits: AtomicU32::new(1.0f32.to_bits())`. Build an
            //           `AtomicU32` cell. Its initial value is `1.0f32.to_bits()`:
            //           `1.0f32` is the float literal one as an `f32` (the `f32` suffix
            //           pins the type), and `.to_bits()` reinterprets that float as its raw
            //           `u32` bit-pattern (NOT the integer 1).
            // Why:      Unity gain (1.0) means "leave samples untouched"; we store its bits
            //           because the cell holds bits, not a float.
            // Gotcha:   `.to_bits()` is a REINTERPRET, not a cast: `1.0f32.to_bits()` is
            //           `0x3F800000`, not `1`. Reading it back needs `f32::from_bits`.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // volumeBits: makeSharedU32(f32ToBits(1.0)), // bits of 1.0, not the integer 1
            // ```
            volume_bits: AtomicU32::new(1.0f32.to_bits()),
            // What:     `rate: AtomicU32::new(0)`. Wrapper constructor: an `AtomicU32`
            //           starting at 0.
            // Why:      0 is the "nothing loaded" sentinel that `position_sec` checks.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // rate: makeSharedU32(0),
            // ```
            rate: AtomicU32::new(0),
            // What:     `channels: AtomicU32::new(0)`. Wrapper constructor: an `AtomicU32`
            //           starting at 0.
            // Why:      No channel count is known until a track loads.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // channels: makeSharedU32(0),
            // ```
            channels: AtomicU32::new(0),
            // What:     `start_frame: AtomicU64::new(0)`. Wrapper constructor: a 64-bit
            //           atomic frame counter starting at 0 (sibling `AtomicU32::new` would
            //           be the 32-bit version we deliberately avoid here).
            // Why:      No seek has happened, so the seek base is zero.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // startFrame: makeSharedU64(0),
            // ```
            start_frame: AtomicU64::new(0),
            // What:     `frames_played: AtomicU64::new(0)`. Wrapper constructor: a 64-bit
            //           atomic counter starting at 0.
            // Why:      Nothing has played yet.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // framesPlayed: makeSharedU64(0),
            // ```
            frames_played: AtomicU64::new(0),
            // What:     `duration_ms: AtomicU64::new(0)`. Wrapper constructor: a 64-bit
            //           atomic starting at 0.
            // Why:      Duration is unknown until a track is probed.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // durationMs: makeSharedU64(0),
            // ```
            duration_ms: AtomicU64::new(0),
            // What:     `decode_done: AtomicBool::new(false)`. Wrapper constructor: an
            //           atomic boolean starting `false`.
            // Why:      The decoder has not finished (there is nothing to decode yet).
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // decodeDone: makeSharedBool(false),
            // ```
            decode_done: AtomicBool::new(false),
            // What:     `ended: AtomicBool::new(false)`. Wrapper constructor: an atomic
            //           boolean starting `false`.
            // Why:      No track has ended yet.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // ended: makeSharedBool(false),
            // ```
            ended: AtomicBool::new(false),
            // What:     `norm_gain_bits: AtomicU32::new(1.0f32.to_bits())`. Wrapper
            //           constructor: an `AtomicU32` initialized to the raw bits of the
            //           `f32` value `1.0` (see `volume_bits` for the `.to_bits()` trick).
            // Why:      Unity normalization gain until a per-track measurement arrives.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // normGainBits: makeSharedU32(f32ToBits(1.0)),
            // ```
            norm_gain_bits: AtomicU32::new(1.0f32.to_bits()),
            // What:     `failed_load: AtomicU64::new(0)`. Wrapper constructor: an atomic counter
            //           starting at 0.
            // Why:      No load has failed yet.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // failedLoad: makeSharedU64(0),
            // ```
            failed_load: AtomicU64::new(0),
        }
    }

    /// What:
    ///      `pub(crate) fn volume(&self) -> f32`.
    ///  A reader method.
    ///  `&self` is a
    ///           READ-ONLY BORROW of the `Control` (it lends the struct to the method
    ///           without transferring ownership and without allowing mutation).
    ///  Returns an
    ///           `f32` (the 32-bit float;
    ///  sibling `f64` is the wider double).
    /// Why:
    ///       The callback wants the volume as a usable float,
    ///  not raw bits.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// volume(): number { ... }
    /// ```
    pub(crate) fn volume(&self) -> f32 {
        // What:     `f32::from_bits(self.volume_bits.load(Ordering::Relaxed))`. Inner part
        //           first: `self.volume_bits.load(Ordering::Relaxed)` atomically READS the
        //           `u32` out of the cell; `Ordering::Relaxed` means "no synchronization
        //           with other memory, just give me a non-torn value" (siblings: `Acquire`
        //           pairs with a `Release` store to make prior writes visible; `Relaxed`
        //           is fine here because volume needs no happens-before relationship).
        //           `f32::from_bits(...)` reinterprets those bits back into the original
        //           `f32`. No trailing `;`, so this is the function's tail expression and
        //           the return value.
        // Why:      Convert the stored bit-pattern back into the gain the callback applies.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return bitsToF32(Atomics.load(this.volumeBits, 0));
        // ```
        return f32::from_bits(self.volume_bits.load(Ordering::Relaxed))
    }

    /// What:
    ///      `pub(crate) fn norm_gain(&self) -> f32`.
    ///  The same shape as `volume`:
    ///  a
    ///           read-only-borrow (`&self`) reader returning an `f32`.
    /// Why:
    ///       Hand the callback the normalization gain as a usable float.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// normGain(): number { ... }
    /// ```
    pub(crate) fn norm_gain(&self) -> f32 {
        // What:     `f32::from_bits(self.norm_gain_bits.load(Ordering::Relaxed))`. Same
        //           two-step as in `volume`: `.load(Ordering::Relaxed)` atomically reads
        //           the `u32` bits, `f32::from_bits` reinterprets them as the float. Tail
        //           expression, so it is the return value.
        // Why:      Return the normalization gain as a float the callback can multiply by.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return bitsToF32(Atomics.load(this.normGainBits, 0));
        // ```
        return f32::from_bits(self.norm_gain_bits.load(Ordering::Relaxed))
    }
}

/// What:
///      `pub(crate) enum Command { ... }`.
///  A SUM TYPE:
///  a `Command` value is exactly
///           ONE of the listed variants at a time.
///  Two variants are tuple-style
///           (`Name(types...)`),
///  wrapping inner data.
///  These are the worker inputs that
///           need the worker's OWNED state (the open file and the output stream);
///  the fast
///           play/pause/volume knobs deliberately skip this enum and write atomics instead.
/// Why:
///       Only commands that touch state the worker exclusively owns travel down the
///           channel;
///  everything cheap stays lock-free.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Command =
///   | { kind: "load"; file: FileHandle; play: boolean }
///   | { kind: "seek"; positionSec: number }
///   | { kind: "quit" };
/// ```
pub(crate) enum Command {
    /// What:
    ///      `Load(std::fs::File, bool)`.
    ///  A tuple variant carrying TWO inner values:
    ///           a `std::fs::File` (an OWNED open file handle,
    ///  here backed by a duplicate
    ///           of the `content://` fd Kotlin passed) and a `bool` (whether to start
    ///           playing immediately).
    ///  `std::fs::File` owns its fd and closes it on drop.
    /// Why:
    ///       Opening and building the output needs the worker's owned state,
    ///  so loading
    ///           is a command,
    ///  not an atomic write.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: "load"; file: FileHandle; play: boolean }
    /// ```
    Load(
        /// What:
        ///      First field `.0` of the `Load` variant:
        ///  an OWNED `std::fs::File`
        ///           (a handle that owns its underlying OS file descriptor and closes it
        ///           on drop;
        ///  here it wraps a DUPLICATE of the `content://` fd Kotlin
        ///           passed across JNI).
        ///  Siblings the reader might expect:
        ///  the bare
        ///           `RawFd`/`OwnedFd` integer handle,
        ///  or a borrowed `&File`.
        /// Why:
        ///       The worker must OWN the open file so it can decode from it and close
        ///           it deterministically when the load is replaced or the worker quits.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// // the `file: FileHandle` payload of { kind: "load" }
        /// ```
        std::fs::File,
        /// What:
        ///      Second field `.1` of the `Load` variant:
        ///  a `bool` flag,
        ///  `true` to
        ///           start playing immediately and `false` to load paused.
        /// Why:
        ///       Lets a single `Load` command both open the file and set the initial
        ///           transport state,
        ///  with no second round-trip down the channel.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// // the `play: boolean` payload of { kind: "load" }
        /// ```
        bool,
    ),
    /// What:
    ///      `Seek(f64)`.
    ///  A tuple variant wrapping one `f64` (the target position in
    ///           seconds;
    ///  `f64` not `f32` for sub-millisecond precision over long tracks).
    /// Why:
    ///       Repositioning the source and flushing the ring is worker-owned work.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: "seek"; positionSec: number }
    /// ```
    Seek(
        /// What:
        ///      Unnamed field `.0` of the `Seek` variant:
        ///  the target position in
        ///           SECONDS as an `f64` (64-bit IEEE double;
        ///  sibling `f32` is the 32-bit
        ///           float,
        ///  too coarse for sub-millisecond accuracy over long tracks).
        /// Why:
        ///       `f64` (not `f32`) keeps repositioning sample-accurate even on
        ///           hour-long files.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// // the `positionSec: number` payload of { kind: "seek" }
        /// ```
        f64,
    ),
    /// What:
    ///      `Quit`.
    ///  A UNIT variant (no payload):
    ///  just a tag telling the worker to stop.
    /// Why:
    ///       Dropping the worker's owned state closes the AAudio stream,
    ///  so quitting is
    ///           a command that lets the worker tear itself down cleanly.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: "quit" }
    /// ```
    Quit,
}

/// What:
///      `pub struct Engine { ... }`.
///  The main-thread handle to the engine,
///  i.e. the
///           value that lives behind the JNI `jlong` Kotlin stores.
///  `pub` (fully public)
///           because Kotlin's JNI glue constructs and calls it.
///  It holds the channel
///           sender,
///  the worker's join handle,
///  the shared control,
///  and the play intent;
///           because the AAudio stream lives in the WORKER,
///  this handle contains nothing
///           thread-bound and is therefore `Send` (movable across threads).
/// Why:
///       One owner object Kotlin can hold,
///  call methods on,
///  and drop to shut down.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Engine {
///   tx: Sender<Command>;
///   worker: JoinHandle | null;
///   control: Control; // shared
///   playIntent: boolean;
/// }
/// ```
pub struct Engine {
    /// What:
    ///      `tx: Sender<Command>`.
    ///  The SEND end of the command channel.
    ///  `Sender<T>`
    ///           is generic over the message type,
    ///  pinned here to `Command` via the
    ///           `<Command>` type argument.
    /// Why:
    ///       `load`/`seek_to`/`drop` push commands to the worker through it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// tx: Sender<Command>;
    /// ```
    tx: Sender<Command>,
    /// What:
    ///      `worker: Option<JoinHandle<()>>`.
    ///  The worker's join handle wrapped in
    ///           `Option`.
    ///  `Option<T>` is Rust's null-safe "maybe a value" type:
    ///  it is
    ///           either `Some(value)` or `None`.
    ///  `JoinHandle<()>` returns the unit type
    ///           `()` (pronounced "unit",
    ///  Rust's "nothing" / `void`-like value),
    ///  meaning
    ///           the worker thread produces no result.
    /// Why:
    ///       `Drop` calls `.take()` to MOVE the handle out (leaving `None`) so it can
    ///           join the worker exactly once;
    ///  `Option` models "already joined".
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// worker: JoinHandle | null;
    /// ```
    worker: Option<JoinHandle<()>>,
    /// What:
    ///      `control: Arc<Control>`.
    ///  The shared control,
    ///  held through an `Arc` (the
    ///           atomically reference-counted shared pointer from the imports).
    ///  This handle
    ///           owns one of the several `Arc` clones;
    ///  the worker owns another.
    /// Why:
    ///       The main thread reads position/duration/playing/ended off this same shared
    ///           `Control` that the worker and callback write.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// control: Control; // shared object reference
    /// ```
    control: Arc<Control>,
    /// What:
    ///      `play_intent: bool`.
    ///  A plain (non-atomic) boolean:
    ///  what Kotlin LAST asked
    ///           for (its `playWhenReady`),
    ///  distinct from whether the engine is actually
    ///           sounding right now.
    ///  `bool` not `AtomicBool` because only this main-thread
    ///           handle reads and writes it.
    /// Why:
    ///       Lets `play_when_ready` report intent even when,
    ///  say,
    ///  the track has ended
    ///           but the user never pressed pause.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// playIntent: boolean;
    /// ```
    play_intent: bool,
}

/// What:
///      `impl Engine { ... }`.
///  The handle's methods:
///  construction,
///  the JNI-facing
///           control surface (load/play/pause/seek/volume/normalization),
///  and the pollers
///           Kotlin reads (position/duration/is_playing/is_ended/play_when_ready).
/// Why:
///       Group everything Kotlin calls over JNI with the `Engine` type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Engine { /* new, load, play, pause, seekTo, ... pollers */ }
/// ```
impl Engine {
    /// What:
    ///      `pub fn new() -> Result<Engine, std::io::Error>`.
    ///  The constructor.
    ///  Returns
    ///           a `Result<Engine, std::io::Error>`:
    ///  Rust's two-channel "either it worked or
    ///           it failed" type,
    ///  holding `Ok(Engine)` on success or `Err(io::Error)` on
    ///           failure.
    ///  The only failure is the OS refusing to create the thread.
    /// Why:
    ///       Spawn the worker and hand back a ready-to-use handle,
    ///  surfacing the rare
    ///           thread-spawn failure instead of crashing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static new(): Engine { /* throws on OS thread-spawn failure */ }
    /// ```
    pub fn new() -> Result<Engine, std::io::Error> {
        // What:     `let control = Arc::new(Control::new());`. Inner first: `Control::new()`
        //           builds a fresh control value; `Arc::new(...)` is the WRAPPER
        //           CONSTRUCTOR that moves it onto the heap behind an atomic refcount so it
        //           can be shared across threads. `let` binds the result to `control`.
        // Why:      We need a shareable control because the worker thread also holds a
        //           clone of it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const control = new Control();
        // ```
        let control = Arc::new(Control::new());
        // What:     `let (tx, rx) = mpsc::channel::<Command>();`. Create the channel and
        //           DESTRUCTURE the returned pair into `tx` (the `Sender`) and `rx` (the
        //           `Receiver`). `::<Command>` is the "turbofish" pinning the message type.
        // Why:      `tx` stays on this handle; `rx` is handed to the worker so it can pop
        //           commands.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const [tx, rx] = makeChannel<Command>();
        // ```
        let (tx, rx) = mpsc::channel::<Command>();
        // What:     `let worker_control = Arc::clone(&control);`. `&control` BORROWS the
        //           `Arc` read-only (we are only lending it, not giving it away);
        //           `Arc::clone(...)` makes ANOTHER owning handle to the SAME heap
        //           `Control` (it bumps the refcount, it does NOT copy the `Control`).
        // Why:      The worker thread needs its own owning handle to the shared control;
        //           cloning the `Arc` is how two threads co-own one value.
        // Gotcha:   `Arc::clone` clones the POINTER, not the data. Both handles see the
        //           same mutations through the inner atomics.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const workerControl = control; // same shared object, no copy
        // ```
        let worker_control = Arc::clone(&control);
        // What:     `let worker = thread::Builder::new() .name("mp-engine".to_string())
        //           .spawn(move || engine_worker::worker_run(rx, worker_control))?;`. A
        //           builder chain: `Builder::new()` starts a configurable thread spec;
        //           `.name("mp-engine".to_string())` gives the OS thread a name (the
        //           `&str` literal is converted to an owned `String` via `.to_string()`,
        //           which the API requires); `.spawn(...)` actually starts it. The argument
        //           `move || engine_worker::worker_run(rx, worker_control)` is a CLOSURE
        //           (`move ||` = "take ownership of `rx` and `worker_control` and run this
        //           on the new thread"). `.spawn` returns a `Result`, and the trailing `?`
        //           is the PROPAGATION OPERATOR: on `Ok` it unwraps the `JoinHandle`, on
        //           `Err` it returns that error from `new` immediately.
        // Why:      All decode/output runs on this named worker thread; the `?` bubbles a
        //           thread-spawn failure up as `new`'s `Err`.
        // Gotcha:   `?` is an early-return-on-error, not a "maybe" operator like TS's
        //           optional chaining `?.`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const worker = startNamedWorker("mp-engine", () =>
        //   engineWorker.workerRun(rx, workerControl)); // throws on thread-spawn failure
        // ```
        let worker = thread::Builder::new()
            .name("mp-engine".to_string())
            .spawn(move || engine_worker::worker_run(rx, worker_control))?;
        // What:     `Ok(Engine { tx, worker: Some(worker), control, play_intent: false })`.
        //           `Engine { ... }` is a struct literal (with field shorthand for `tx` and
        //           `control`, whose variable names match the field names). `Some(worker)`
        //           wraps the join handle in the present variant of `Option` so `Drop` can
        //           later `.take()` it. `Ok(...)` wraps the whole `Engine` in the success
        //           variant of `Result`. No trailing `;`, so this is the tail expression
        //           and the return value.
        // Why:      Hand back a fully-built engine on the success channel.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return new Engine(tx, /* worker */ worker, control, /* playIntent */ false);
        // ```
        return Ok(Engine {
            tx,
            worker: Some(worker),
            control,
            play_intent: false,
        })
    }

    /// What:
    ///      `pub fn load(&mut self, fd: RawFd, play: bool) -> Result<(), PlayerError>`.
    ///           `&mut self` is a MUTABLE BORROW of the engine (this method may change its
    ///           fields,
    ///  here `play_intent`).
    ///  `fd: RawFd` is the raw OS file-descriptor
    ///           integer Kotlin passed over JNI;
    ///  `play: bool` says whether to start playing.
    ///           Returns `Result<(), PlayerError>`:
    ///  `Ok(())` (success carrying the unit
    ///           value,
    ///  i.e. "nothing") or `Err(PlayerError)`.
    /// Why:
    ///       Receive Kotlin's fd,
    ///  duplicate it so the worker can own a copy,
    ///  and hand
    ///           the file to the worker to open and (optionally) play.
    ///  The caller guarantees
    ///           `fd >= 0` and that the fd is alive for this synchronous call (Kotlin is
    ///           inside the file descriptor's `use {}`).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// load(fd: number, play: boolean): void { /* throws PlayerError on worker-gone */ }
    /// ```
    pub fn load(&mut self, fd: RawFd, play: bool) -> Result<(), PlayerError> {
        // What:     `let borrowed = unsafe { BorrowedFd::borrow_raw(fd) };`. `unsafe { ... }`
        //           is a block where we promise the compiler we have manually upheld a
        //           safety rule it cannot check. Inside, `BorrowedFd::borrow_raw(fd)` wraps
        //           the raw integer fd into a typed BORROW (`BorrowedFd`) without taking
        //           ownership of it.
        // Why:      We only need to read the fd long enough to duplicate it; wrapping it as
        //           a borrow (not an owner) means we will NOT close Kotlin's original fd.
        //           The `unsafe` is required because the compiler cannot verify the fd is
        //           actually valid; the caller's contract (see the method's Why) provides
        //           the guarantee.
        // Gotcha:   `unsafe` does NOT turn off safety everywhere; it just lets you call a
        //           few operations the compiler trusts you to use correctly.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const borrowed = fd; // "borrow": use it, but it is the JVM's to close
        // ```
        let borrowed = unsafe { BorrowedFd::borrow_raw(fd) };
        // What:     `let file = std::fs::File::from(borrowed.try_clone_to_owned()?);`. Inner
        //           first: `borrowed.try_clone_to_owned()` asks the OS to DUPLICATE the fd
        //           into a new OWNED one (`OwnedFd`), returning a `Result`; the trailing
        //           `?` unwraps the `OwnedFd` on success or returns the error from `load`.
        //           `std::fs::File::from(...)` then converts that `OwnedFd` into a
        //           `std::fs::File` (an owned file handle that closes the dup on drop).
        // Why:      The worker outlives this call and Kotlin will close the original fd, so
        //           we must hand the worker its OWN duplicated, owned file, not a borrow.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const file = openFromDuplicatedFd(borrowed); // throws on dup failure
        // ```
        let file = std::fs::File::from(borrowed.try_clone_to_owned()?);
        // What:     `self.play_intent = play;`. A plain field assignment on the mutably
        //           borrowed engine: record whether the user wants playback.
        // Why:      So `play_when_ready` reports the latest intent even before the worker
        //           has finished loading.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.playIntent = play;
        // ```
        self.play_intent = play;
        // What:     `self.tx.send(Command::Load(file, play)) .map_err(|_|
        //           PlayerError::Unsupported("engine worker gone".to_string()))?;`.
        //           `Command::Load(file, play)` is a WRAPPER CONSTRUCTOR building the `Load`
        //           variant (it MOVES the owned `file` into the message). `self.tx.send(...)`
        //           returns a `Result` that errs only if the worker is gone. `.map_err(|_|
        //           ...)` is a CLOSURE `|_| ...` (the `_` ignores the original error) that
        //           REPLACES any send error with a `PlayerError::Unsupported(...)` whose
        //           message is an owned `String` (`.to_string()` on the `&str` literal).
        //           The trailing `?` then propagates that converted error out of `load`.
        // Why:      Convert the channel's low-level send error into the engine's unified
        //           `PlayerError`, and bail out of `load` if the worker has already exited.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // try {
        //   this.tx.send({ kind: "load", file, play });
        // } catch {
        //   throw new PlayerError.Unsupported("engine worker gone");
        // }
        // ```
        self.tx
            .send(Command::Load(file, play))
            .map_err(|_| return PlayerError::Unsupported("engine worker gone".to_string()))?;
        // What:     `Ok(())`. `Ok(...)` wraps the success channel of `Result`, and `()` is
        //           the unit value ("nothing"). No trailing `;`, so this is the tail
        //           expression and the function's return value.
        // Why:      Signal "loaded successfully, no payload" to the caller.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return; // success, no value
        // ```
        return Ok(())
    }

    /// What:
    ///      `pub fn play(&mut self)`.
    ///  A control method that takes a MUTABLE BORROW
    ///           (`&mut self`) because it changes `play_intent`.
    ///  No return type means it
    ///           returns the unit value `()`.
    /// Why:
    ///       Resume playback:
    ///  the realtime callback un-gates on its next buffer.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// play(): void { ... }
    /// ```
    pub fn play(&mut self) {
        // What:     `self.play_intent = true;`. Plain boolean field assignment.
        // Why:      Record that the user wants sound.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.playIntent = true;
        // ```
        self.play_intent = true;
        // What:     `self.control.playing.store(true, Ordering::Release);`. `.store(true,
        //           Ordering::Release)` atomically WRITES `true` into the `playing`
        //           `AtomicBool`. `Ordering::Release` means "make all my earlier writes
        //           visible to any thread that later reads this with `Acquire`" (siblings:
        //           `Relaxed` gives no such guarantee, `Acquire` is the matching read side).
        // Why:      Flip the lock-free play gate the realtime callback reads; `Release`
        //           pairs with the callback's `Acquire` load so the callback sees a
        //           consistent state.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // Atomics.store(this.control.playing, 0, 1); // 1 = true
        // ```
        self.control.playing.store(true, Ordering::Release);
    }

    /// What:
    ///      `pub fn pause(&mut self)`.
    ///  The mirror of `play`:
    ///  a `&mut self` control
    ///           method returning nothing.
    /// Why:
    ///       Pause:
    ///  the callback emits silence on its next buffer while KEEPING the
    ///           audio already buffered (so resume is instant).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// pause(): void { ... }
    /// ```
    pub fn pause(&mut self) {
        // What:     `self.play_intent = false;`. Plain boolean field assignment.
        // Why:      Record that the user wants silence.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.playIntent = false;
        // ```
        self.play_intent = false;
        // What:     `self.control.playing.store(false, Ordering::Release);`. Atomically
        //           write `false` into the `playing` gate with `Release` ordering (same
        //           pairing as in `play`).
        // Why:      Tell the realtime callback to stop draining the ring on its next buffer.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // Atomics.store(this.control.playing, 0, 0); // 0 = false
        // ```
        self.control.playing.store(false, Ordering::Release);
    }

    /// What:
    ///      `pub fn seek_to(&self, position_sec: f64)`.
    ///  `&self` is a READ-ONLY BORROW
    ///           (this method does not mutate the engine;
    ///  it only sends a command).
    ///           `position_sec: f64` is the seek target in seconds (`f64`,
    ///  not `f32`,
    ///  for
    ///           precision).
    ///  Returns nothing.
    /// Why:
    ///       Ask the worker to reposition the loaded track;
    ///  the worker re-flushes the
    ///           ring buffer.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// seekTo(positionSec: number): void { ... }
    /// ```
    pub fn seek_to(&self, position_sec: f64) {
        // What:     `let _ = self.tx.send(Command::Seek(position_sec));`.
        //           `Command::Seek(position_sec)` is a WRAPPER CONSTRUCTOR building the
        //           `Seek` variant around the position. `self.tx.send(...)` returns a
        //           `Result`; `let _ =` DISCARDS it (the `_` is the throw-away binding).
        // Why:      A seek after the worker has exited is harmless, so the send error is not
        //           worth surfacing.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // try { this.tx.send({ kind: "seek", positionSec }); } catch {}
        // ```
        let _ = self.tx.send(Command::Seek(position_sec));
    }

    /// What:
    ///      `pub fn set_volume(&self, volume: f32)`.
    ///  A read-only-borrow (`&self`)
    ///           method taking `volume: f32` (a 32-bit float linear gain;
    ///  sibling `f64`
    ///           would be needless precision for a volume knob).
    ///  Returns nothing.
    /// Why:
    ///       Set the user volume;
    ///  the callback multiplies every sample by it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// setVolume(volume: number): void { ... }
    /// ```
    pub fn set_volume(&self, volume: f32) {
        // What:     `self.control.volume_bits.store(volume.to_bits(), Ordering::Relaxed);`.
        //           `volume.to_bits()` REINTERPRETS the `f32` as its raw `u32` bit-pattern
        //           (a conversion, not a numeric cast). `.store(..., Ordering::Relaxed)`
        //           atomically writes those bits into `volume_bits`; `Relaxed` because a
        //           volume change needs no happens-before relationship with other memory.
        // Why:      The cell stores bits, so we store the bit-pattern; `Relaxed` is the
        //           cheapest ordering and is correct for an independent scalar like volume.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // Atomics.store(this.control.volumeBits, 0, f32ToBits(volume));
        // ```
        self.control
            .volume_bits
            .store(volume.to_bits(), Ordering::Relaxed);
    }

    /// What:
    ///      `pub fn set_normalization_gain(&self, gain: f32)`.
    ///  The same shape as
    ///           `set_volume`:
    ///  a read-only-borrow method taking an `f32` linear gain.
    /// Why:
    ///       Set the per-track true-peak normalization gain,
    ///  applied together with the
    ///           user volume in the callback.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// setNormalizationGain(gain: number): void { ... }
    /// ```
    pub fn set_normalization_gain(&self, gain: f32) {
        // What:     `self.control.norm_gain_bits.store(gain.to_bits(), Ordering::Relaxed);`.
        //           `gain.to_bits()` reinterprets the `f32` gain as raw `u32` bits;
        //           `.store(..., Ordering::Relaxed)` atomically writes them into
        //           `norm_gain_bits` with the cheapest ordering.
        // Why:      Same bit-storage trick as volume; `Relaxed` is correct for an
        //           independent scalar gain.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // Atomics.store(this.control.normGainBits, 0, f32ToBits(gain));
        // ```
        self.control
            .norm_gain_bits
            .store(gain.to_bits(), Ordering::Relaxed);
    }

    /// What:
    ///      `pub fn position_sec(&self) -> f64`.
    ///  A read-only-borrow poller returning
    ///           the current playback position in seconds as an `f64` (the wide double;
    ///           sibling `f32` would lose precision for long-running positions).
    /// Why:
    ///       Kotlin polls this for the progress bar;
    ///  0 when nothing is loaded.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// positionSec(): number { ... }
    /// ```
    pub fn position_sec(&self) -> f64 {
        // What:     `let rate = self.control.rate.load(Ordering::Acquire);`. `.load(
        //           Ordering::Acquire)` atomically READS the sample rate. `Ordering::Acquire`
        //           means "and make visible all writes the writer published with a matching
        //           `Release`" (sibling `Relaxed` would not establish that ordering).
        // Why:      We need the rate to convert frames into seconds, and `Acquire` ensures
        //           we see a coherent set of `rate`/`start_frame`/`frames_played`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const rate = Atomics.load(this.control.rate, 0);
        // ```
        let rate = self.control.rate.load(Ordering::Acquire);
        // What:     `if rate == 0 { return 0.0; }`. A plain guard: if no track is loaded
        //           (rate is the 0 sentinel), early-`return` the float `0.0`. `0.0` is an
        //           `f64` literal (a float, matching the return type, not the integer `0`).
        // Why:      Avoid dividing by zero and report "no position" cleanly.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (rate === 0) return 0;
        // ```
        if rate == 0 {
            return 0.0;
        }
        // What:     `let frames = self.control.start_frame.load(Ordering::Acquire) +
        //           self.control.frames_played.load(Ordering::Acquire);`. Two atomic
        //           `.load(Ordering::Acquire)` reads (the seek base and frames played),
        //           added with plain `+`. Both use `Acquire` so the pair is coherent with
        //           the worker/callback's `Release` writes.
        // Why:      Absolute position = where the stream started (the seek target) plus how
        //           many frames have played since.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const frames =
        //   Atomics.load(this.control.startFrame, 0) + Atomics.load(this.control.framesPlayed, 0);
        // ```
        let frames =
            self.control.start_frame.load(Ordering::Acquire) + self.control.frames_played.load(Ordering::Acquire);
        // What:     `frames as f64 / rate as f64`. `frames as f64` and `rate as f64` are
        //           explicit numeric CASTS from the unsigned 64-bit integers to `f64`
        //           floats (the `as` keyword); `/` is then floating-point division. No
        //           trailing `;`, so this is the tail expression and the return value.
        // Why:      Cast to float so the division yields fractional seconds, not truncated
        //           integer division.
        // Gotcha:   `as f64` on a `u64` can lose precision for astronomically huge frame
        //           counts, but real audio never reaches that range.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return frames / rate; // both already floats in TS
        // ```
        return frames as f64 / rate as f64
    }

    /// What:
    ///      `pub fn duration_sec(&self) -> f64`.
    ///  A read-only-borrow poller returning the
    ///           loaded track's duration in seconds as an `f64`.
    /// Why:
    ///       Kotlin polls this for the progress bar's total length;
    ///  0 when unknown.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// durationSec(): number { ... }
    /// ```
    pub fn duration_sec(&self) -> f64 {
        // What:     `self.control.duration_ms.load(Ordering::Acquire) as f64 /
        //           MILLIS_PER_SEC`. `.load(Ordering::Acquire)` atomically reads the
        //           duration in milliseconds; `as f64` CASTS that `u64` to a float; `/
        //           MILLIS_PER_SEC` divides by 1000.0 to get seconds. Tail expression, so
        //           it is the return value.
        // Why:      Convert the stored milliseconds into the seconds Kotlin's UI expects.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return Atomics.load(this.control.durationMs, 0) / MILLIS_PER_SEC;
        // ```
        return self.control.duration_ms.load(Ordering::Acquire) as f64 / MILLIS_PER_SEC
    }

    /// What:
    ///      `pub fn is_playing(&self) -> bool`.
    ///  A read-only-borrow poller returning a
    ///           plain `bool`:
    ///  whether the engine is ACTUALLY sounding (playing and not yet
    ///           ended),
    ///  which differs from `play_when_ready` (mere intent).
    /// Why:
    ///       Kotlin polls this to know if sound is really coming out right now.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// isPlaying(): boolean { ... }
    /// ```
    pub fn is_playing(&self) -> bool {
        // What:     `self.control.playing.load(Ordering::Acquire) &&
        //           !self.control.ended.load(Ordering::Acquire)`. Two atomic
        //           `.load(Ordering::Acquire)` reads combined with `&&` (boolean AND). The
        //           `!` before the second is boolean NOT: "playing AND not ended". Tail
        //           expression, so it is the return value.
        // Why:      A track that played to its end is "not playing" even though the play
        //           gate may still be set, so we AND in `!ended`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return (
        //   Atomics.load(this.control.playing, 0) && !Atomics.load(this.control.ended, 0)
        // );
        // ```
        return self.control.playing.load(Ordering::Acquire) && !self.control.ended.load(Ordering::Acquire)
    }

    /// What:
    ///      `pub fn is_ended(&self) -> bool`.
    ///  A read-only-borrow poller returning a
    ///           plain `bool`:
    ///  whether the loaded track has played through to its end.
    /// Why:
    ///       Kotlin's poller de-duplicates this into a single `onTrackEnded` callback.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// isEnded(): boolean { ... }
    /// ```
    pub fn is_ended(&self) -> bool {
        // What:     `self.control.ended.load(Ordering::Acquire)`. A single atomic
        //           `.load(Ordering::Acquire)` of the `ended` flag. Tail expression, so it
        //           is the return value.
        // Why:      Report whether the callback has marked the track finished.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return Atomics.load(this.control.ended, 0);
        // ```
        return self.control.ended.load(Ordering::Acquire)
    }

    /// What:
    ///      `pub fn failed_load(&self) -> u64`.
    ///  A read-only-borrow poller returning the sequence number of the most
    ///           recent queued load that the worker could not open for playback,
    ///  or 0 when none has failed.
    /// Why:
    ///       Kotlin compares it with the number of loads it has queued,
    ///  and reports the load failure
    ///           when the failed one is the load it is still waiting for.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// failedLoad(): number { ... }
    /// ```
    pub fn failed_load(&self) -> u64 {
        // What:     `self.control.failed_load.load(Ordering::Acquire)`. One atomic load; tail expression.
        // Why:      Report the worker's last load failure.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return Atomics.load(this.control.failedLoad, 0);
        // ```
        return self.control.failed_load.load(Ordering::Acquire)
    }

    /// What:
    ///      `pub fn play_when_ready(&self) -> bool`.
    ///  A read-only-borrow poller returning
    ///           a plain `bool`:
    ///  the last play INTENT Kotlin asked for (its `playWhenReady`),
    ///           true from a play / load-and-play request until a pause.
    /// Why:
    ///       Lets Kotlin distinguish "the user wants playback" from "audio is actually
    ///           sounding" (which `is_playing` reports).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// playWhenReady(): boolean { ... }
    /// ```
    pub fn play_when_ready(&self) -> bool {
        // What:     `self.play_intent`. Read the plain (non-atomic) boolean field directly.
        //           No trailing `;`, so this bare field access is the tail expression and
        //           the return value.
        // Why:      Hand back the stored intent unchanged.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return this.playIntent;
        // ```
        return self.play_intent
    }
}

/// What:
///      `impl Drop for Engine { ... }`.
///  Implement the `Drop` TRAIT for `Engine`.
///           `Drop` is Rust's DESTRUCTOR trait:
///  its `drop` method runs automatically the
///           moment an `Engine` goes out of scope (when Kotlin's JNI glue frees the handle).
/// Why:
///       Tell the worker to quit and wait for it,
///  so the AAudio stream the worker owns
///           is closed cleanly before the process moves on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Engine { [Symbol.dispose]() { /* send quit, then await worker */ } }
/// ```
impl Drop for Engine {
    /// What:
    ///      `fn drop(&mut self)`.
    ///  The destructor method.
    ///  `&mut self` because tearing the
    ///           engine down mutates it (it `.take()`s the worker handle out).
    ///  The runtime,
    ///           not your code,
    ///  calls this;
    ///  you never call `drop` by name.
    /// Why:
    ///       Run graceful shutdown at end of the engine's life.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// [Symbol.dispose]() { ... }
    /// ```
    fn drop(&mut self) {
        // What:     `let _ = self.tx.send(Command::Quit);`. `Command::Quit` is the unit
        //           WRAPPER CONSTRUCTOR for the `Quit` variant. `self.tx.send(...)` returns
        //           a `Result`; `let _ =` DISCARDS it (the worker may already be gone).
        // Why:      Ask the worker to break its loop and tear down its owned state.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // try { this.tx.send({ kind: "quit" }); } catch {}
        // ```
        let _ = self.tx.send(Command::Quit);
        // What:     `if let Some(worker) = self.worker.take() { let _ = worker.join(); }`.
        //           `self.worker.take()` MOVES the `JoinHandle` out of the `Option`, leaving
        //           `None` behind and returning the old value (`Some(handle)` or `None`).
        //           `if let Some(worker) = ...` is a one-arm PATTERN MATCH that runs the body
        //           ONLY when the `Option` was `Some`, binding the inner handle to `worker`.
        //           Inside, `worker.join()` BLOCKS until the worker thread finishes and
        //           returns a `Result` (errs if the worker panicked); `let _ =` discards it.
        // Why:      Join exactly once (the `.take()` guarantees it) and wait for the worker
        //           so its AAudio stream is fully closed before we return; a worker panic is
        //           not worth acting on during shutdown.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const worker = this.worker;
        // this.worker = null;
        // if (worker) {
        //   try { await worker; } catch {} // ignore a worker panic on shutdown
        // }
        // ```
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
