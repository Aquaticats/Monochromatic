//! AAudio output backend (raw `ndk::audio`):
//!  the latency-probe stage of the
//! Android audio path.
//!  It opens a single SILENT,
//!  low-latency output stream,
//! lets it run for a fraction of a second,
//!  reads the stream's presentation
//! timestamp,
//!  and reports how many milliseconds of audio sit buffered ahead of
//! the speaker.
//!  The point is to PROVE the pure-Rust AAudio path can open and
//! run a stream on a real device before the engine starts pushing real audio
//! through it;
//!  the number it returns is a diagnostic,
//!  not something the player
//! consumes.
//!
//! Mental model for a TypeScript reader:
//!  AAudio is Android's low-level C audio
//! API.
//!  `ndk::audio` is a thin Rust wrapper over that C API,
//!  so almost every
//! type here is an `i32` (because the underlying C functions take and return
//! C `int32_t`),
//!  and the data callback hands us a RAW MEMORY ADDRESS (`*mut
//! c_void`) instead of a typed array,
//!  exactly like a C function would.
//!  There is
//! no persistent state,
//!  no struct,
//!  no threads we spawn,
//!  no shared flags:
//!  this
//! is one fire-and-forget measurement function plus two small helpers it calls.
//!
//! Unlike the desktop `output_cpal.rs` / `output_pipewire.rs` backends,
//!  NOTHING
//! here owns a ring buffer,
//!  an `Arc`,
//!  or an `AtomicBool`.
//!  Do not carry mental
//! models over from those files;
//!  this is a standalone probe,
//!  and the only thing
//! it shares with them is the idea of an OS audio "stream" with a realtime fill
//! callback.

/// What:
///      `use std::os::raw::c_void;`.
///  `c_void` is Rust's stand-in for C's
///           `void` type.
///  It exists only to be pointed AT:
///  a `*mut c_void` is "a
///           raw address to some bytes whose type Rust does not know".
///  Siblings
///           you might expect:
///  `u8` (a byte) or `()` (Rust's own empty/unit
///           type);
///  `c_void` is specifically the one that lines up with the C
///           ABI,
///  which is what AAudio speaks.
/// Why:
///       The AAudio data callback (below) receives the output buffer as a
///           `*mut c_void`,
///  mirroring the C signature,
///  so we must name this type
///           to write that signature.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // no equivalent: a raw address to bytes of unknown type
/// ```
use std::os::raw::c_void;

/// What:
///      `use std::time::Duration;`.
///  `Duration` is a span of time (seconds +
///           nanoseconds),
///  not a point in time.
///  Sibling you might confuse it
///           with:
///  `Instant`,
///  which is a TIMESTAMP (a moment on the clock);
///           `Duration` is a LENGTH.
/// Why:
///       We sleep for a fixed `Duration` after starting the stream so audio
///           is actually flowing before we read the latency.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a length of time, expressed in ms as a plain number
/// ```
use std::time::Duration;

/// What:
///      `use ndk::audio::{ ... };`.
///  One `use` pulling in several names from
///           the `ndk::audio` module.
///  The `{...}` is a grouped import (the `::`
///           is Rust's path separator,
///  like `.` between TS module segments).
///           - `AudioCallbackResult`:
///  the enum our data callback returns to tell
///             AAudio whether to keep going (`Continue`) or stop (`Stop`).
///           - `AudioDirection`:
///  enum picking input vs output (`Output` here).
///           - `AudioFormat`:
///  enum naming the sample format (`PCM_Float` here).
///           - `AudioPerformanceMode`:
///  enum picking the latency/power tradeoff
///             (`LowLatency` here).
///           - `AudioStream`:
///  the opened stream handle type (a parameter type of
///             the callback).
///           - `AudioStreamBuilder`:
///  the builder we configure step by step and
///             then `.open_stream()` on.
///           - `Clockid`:
///  enum naming which OS clock the presentation timestamp
///             is measured against (`Monotonic` here).
/// Why:
///       Every one of these names is used below to build,
///  open,
///  run,
///  and
///           query the stream;
///  importing them brings them into scope so we can
///           write them unqualified.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import {
///   AudioCallbackResult, AudioDirection, AudioFormat, AudioPerformanceMode,
///   AudioStream, AudioStreamBuilder, Clockid,
/// } from "ndk/audio";
/// ```
use ndk::audio::{
    AudioCallbackResult, AudioDirection, AudioFormat, AudioPerformanceMode, AudioStream,
    AudioStreamBuilder, Clockid,
};

/// What:
///      `const SAMPLE_RATE: i32 = 48_000;`.
///  A compile-time constant naming
///           the requested sample rate (48 kHz,
///  a standard rate).
///  `i32` is a
///           signed 32-bit integer.
///  Siblings the reader might expect:
///  `u32`
///           (unsigned 32-bit),
///  `usize` (pointer-wide unsigned),
///  `i64`
///           (signed 64-bit).
///  The `_` in `48_000` is just a digit separator for
///           readability (it is NOT part of the number).
/// Why:
///       We ask AAudio to open the stream at this rate;
///  the device may give
///           us a different actual rate,
///  which is why we re-read it later.
/// Why i32:
///   `ndk::audio`'s `.sample_rate(...)` setter takes an `i32` because the
///           underlying AAudio C API uses C `int32_t`;
///  using `i32` here avoids a
///           cast at that boundary.
///  `u32`/`usize` would force an `as` conversion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SAMPLE_RATE = 48_000;
/// ```
const SAMPLE_RATE: i32 = 48_000;

/// What:
///      `const CHANNELS: i32 = 2;`.
///  Constant for the channel count
///           (2 = stereo).
///  `i32` again,
///  same family as above (`u32`,
///  `usize`,
///           `i64` are the siblings).
/// Why:
///       We request a 2-channel stream and,
///  separately,
///  use the count to
///           compute how many `f32` samples a frame's worth of audio is.
/// Why i32:
///   `.channel_count(...)` on the builder takes an `i32` (the AAudio C
///           API uses `int32_t`),
///  so storing the constant as `i32` matches that
///           setter with no cast;
///  we only convert to `usize` where we index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CHANNELS = 2;
/// ```
const CHANNELS: i32 = 2;

/// What:
///      `const SETTLE: Duration = Duration::from_millis(300);`.
///  A constant
///           `Duration` of 300 milliseconds.
///  `Duration::from_millis(300)` is a
///           CONSTRUCTOR call:
///  it builds a `Duration` value from a millisecond
///           count (the `::` is the path to an associated function on the
///           `Duration` type,
///  like a static method).
/// Why:
///       After starting the stream we wait this long so frames are actually
///           flowing through the hardware before we read the timestamp;
///  reading
///           too early would report a meaningless latency.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SETTLE_MS = 300; // settle time, in milliseconds
/// ```
const SETTLE: Duration = Duration::from_millis(300);

/// What:
///      `const MILLIS_PER_SEC: f64 = 1000.0;`.
///  A constant conversion factor.
///           `f64` is a 64-bit floating-point number (a "double").
///  Sibling you
///           might expect:
///  `f32`,
///  the 32-bit float;
///  `f64` is the wider,
///  more
///           precise one and is Rust's DEFAULT float type.
/// Why:
///       The latency math divides a frame count by a sample rate (giving
///           seconds) and multiplies by this to express the result in
///           milliseconds.
/// Why f64:
///   We use `f64` (not `f32`) because the helper returns `f64` and
///           Rust's default float literal is `f64`;
///  the extra precision is free
///           here and avoids mixing float widths in the division.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MILLIS_PER_SEC = 1000;
/// ```
const MILLIS_PER_SEC: f64 = 1000.0;

/// What:
///      `pub fn measure_output_latency_ms() -> Option<f64>`.
///  A public
///           function taking no arguments and returning `Option<f64>`.
///  `Option<T>`
///           is Rust's "a value,
///  or nothing" type (Rust has no `null`);
///  its two
///           cases are `Some(value)` and `None`.
///  So the return is "a latency in
///           milliseconds (`f64`),
///  or nothing if any step failed".
/// Why:
///       This is the whole probe:
///  open a silent stream,
///  run it,
///  measure how
///           far ahead of the DAC we are buffered,
///  and hand back the number (or
///           `None` if the device would not cooperate at any step).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function measureOutputLatencyMs(): number | null { ... }
/// ```
pub fn measure_output_latency_ms() -> Option<f64> {
    // What:     `let stream = AudioStreamBuilder::new().ok()? . ... .open_stream().ok()?;`.
    //           This single `let` statement spans many lines: it builds and
    //           opens the stream in one chained expression. Reading top to bottom:
    //           - `AudioStreamBuilder::new()` calls the builder's constructor
    //             (the `::` reaches an associated function). It returns a
    //             `Result<AudioStreamBuilder, _>` (success-or-failure).
    //           - `.ok()` turns that `Result` into an `Option` by DROPPING the
    //             error (keeping `Some(builder)` on success, `None` on failure).
    //           - `?` on the `Option` unwraps `Some` to the inner builder, OR
    //             returns `None` from `measure_output_latency_ms` immediately if
    //             it was `None`. (Early-exit-on-nothing operator.)
    //           - `.direction(AudioDirection::Output)` sets the stream to be an
    //             OUTPUT (playback) stream; `AudioDirection::Output` names one
    //             variant of that enum.
    //           - `.format(AudioFormat::PCM_Float)` asks for 32-bit float
    //             samples (`PCM_Float` variant); this is why the callback writes
    //             `f32`s.
    //           - `.sample_rate(SAMPLE_RATE)` and `.channel_count(CHANNELS)`
    //             pass our `i32` constants straight through (no cast needed,
    //             they match the setters' `i32` parameters).
    //           - `.performance_mode(AudioPerformanceMode::LowLatency)` picks
    //             the lowest-latency mode (the variant we want for an honest
    //             latency reading).
    //           - `.data_callback(Box::new(silent_callback))` registers our fill
    //             function. `Box::new(...)` heap-allocates the callback behind an
    //             owning pointer (`Box<T>` is "an owned value on the heap";
    //             siblings `Rc<T>`/`Arc<T>` are the shared, reference-counted
    //             versions). AAudio needs to store the callback, so it must be
    //             boxed.
    //           - `.open_stream()` actually opens the OS stream, returning a
    //             `Result<AudioStream, _>`; `.ok()?` again drops the error and
    //             early-returns `None` on failure, leaving `stream` bound to the
    //             opened `AudioStream` on success.
    // Why:      One configured, opened, ready-to-start output stream. If ANY
    //           step fails we bail out of the whole probe with `None` (the
    //           function's contract is "a number, or nothing").
    // Gotcha:   each `.ok()?` SILENTLY discards the underlying error and converts
    //           failure into a plain `None`; we lose any detail about WHY a step
    //           failed. That is intentional for a best-effort probe, but it
    //           means a `None` return tells you "something failed", not what.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const stream = new AudioStreamBuilder()
    //   .direction("output")
    //   .format("pcmFloat")
    //   .sampleRate(SAMPLE_RATE)
    //   .channelCount(CHANNELS)
    //   .performanceMode("lowLatency")
    //   .dataCallback(silentCallback)
    //   .openStream();
    // if (!stream) return null; // any failed step => null
    // ```
    let stream = AudioStreamBuilder::new()
        .ok()?
        .direction(AudioDirection::Output)
        .format(AudioFormat::PCM_Float)
        .sample_rate(SAMPLE_RATE)
        .channel_count(CHANNELS)
        .performance_mode(AudioPerformanceMode::LowLatency)
        .data_callback(Box::new(silent_callback))
        .open_stream()
        .ok()?;

    // What:     `stream.request_start().ok()?;`. Ask AAudio to START the stream
    //           playing. `request_start()` returns a `Result<(), _>` where `()`
    //           is the empty "unit" value (like `void` success). `.ok()` drops
    //           any error into an `Option`, and `?` early-returns `None` from
    //           the probe if starting failed.
    // Why:      The data callback only fires once the stream is started; without
    //           this, no frames flow and the timestamp would never advance.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!stream.requestStart()) return null;
    // ```
    stream.request_start().ok()?;

    // What:     `std::thread::sleep(SETTLE);`. Block THIS thread for the `SETTLE`
    //           duration (300 ms). `std::thread::sleep` is the standard-library
    //           blocking sleep; the `::` segments are the path to it.
    // Why:      Give the hardware time to actually begin presenting frames before
    //           we read the timestamp, so the latency we compute reflects a
    //           running stream, not a just-started one.
    // Gotcha:   this is a REAL blocking sleep (it stops the calling thread). TS
    //           has nothing that blocks the event loop like this; you would use
    //           an awaited timeout instead.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // await new Promise((r) => setTimeout(r, SETTLE_MS));
    // ```
    std::thread::sleep(SETTLE);

    // What:     `let rate = stream.sample_rate();`. Read back the stream's ACTUAL
    //           sample rate (an `i32`). The device may have opened at a rate
    //           different from the `SAMPLE_RATE` we requested.
    // Why:      The latency math must divide by the rate the hardware is really
    //           running at, not the one we asked for, or the milliseconds would
    //           be wrong.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const rate = stream.sampleRate();
    // ```
    let rate = stream.sample_rate();

    // What:     `let latency = read_latency_ms(&stream, rate);`. Call our helper
    //           to compute the latency. `&stream` is a BORROW: it lends the
    //           stream to the helper read-only (the `&` means "you may look at
    //           this but you do not own it and must not keep it"). `rate` is a
    //           plain `i32` passed by copy.
    // Why:      Keep the timestamp math in one small, separately testable helper
    //           rather than inlining it here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const latency = readLatencyMs(stream, rate);
    // ```
    let latency = read_latency_ms(&stream, rate);

    // What:     `let _ = stream.request_stop();`. Stop the stream. `request_stop()`
    //           returns a `Result<(), _>`, but `let _ = ...` DISCARDS that result
    //           entirely (the `_` is the "throw this away" pattern). We do not
    //           care whether stopping succeeded.
    // Why:      We are done measuring; stop the silent stream to release the
    //           device. A stop failure here cannot change the latency we already
    //           computed, so we ignore it on purpose.
    // Gotcha:   `let _ = expr;` is the explicit, lint-quiet way to say "I am
    //           deliberately ignoring this `Result`". Without it, Rust would warn
    //           about an unhandled `Result`. Do NOT "fix" this to `?`: a stop
    //           failure must not abort the function before it returns `latency`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // stream.requestStop(); // ignore the result on purpose
    // ```
    let _ = stream.request_stop();

    // What:     `latency`. A bare expression with NO trailing `;`, sitting as the
    //           last line of the function body. In Rust that makes it the TAIL
    //           expression: its value becomes the function's return value. Here
    //           `latency` is already an `Option<f64>` (from `read_latency_ms`).
    // Why:      Hand the measured latency (or `None`) back to the caller.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return latency;
    // ```
    return latency
}

/// What:
///      `fn silent_callback(_stream: &AudioStream, audio_data: *mut c_void,
///           num_frames: i32) -> AudioCallbackResult`.
///  A private function (no
///           `pub`) that AAudio calls ON ITS REALTIME AUDIO THREAD every time the
///           hardware needs more samples.
///  Parameters:
///           - `_stream: &AudioStream`.
///  A read-only BORROW of the stream.
///  The
///             leading `_` says "I accept this argument but do not use it"
///             (silences the unused-parameter warning).
///           - `audio_data: *mut c_void`.
///  A RAW,
///  MUTABLE pointer to the output
///             buffer AAudio wants us to fill.
///  `*mut` = "raw mutable pointer";
///             `c_void` = "bytes of a type Rust does not track".
///  This is NOT a
///             safe Rust reference:
///  there is no length attached and the compiler
///             will not check our writes.
///           - `num_frames: i32`.
///  How many audio FRAMES the buffer holds (one
///             frame = one sample per channel).
///  `i32` because AAudio's C API
///             reports it as `int32_t`.
///           Returns `AudioCallbackResult`,
///  an enum telling AAudio what to do
///           next (`Continue` to keep streaming,
///  `Stop` to end).
/// Why:
///       Our probe never plays real audio;
///  this callback exists only to keep
///           the stream alive and flowing by writing silence,
///  so the presentation
///           timestamp advances and we can measure latency.
/// Gotcha:
///    this runs on a REALTIME thread:
///  it must not allocate,
///  lock,
///  block,
///           or panic.
///  Writing zeros with one bulk memory operation respects
///           that.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function silentCallback(
///   _stream: AudioStream,
///   audioData: Float32Array, // really a raw address in Rust
///   numFrames: number,
/// ): "continue" | "stop" { ... }
/// ```
fn silent_callback(
    _stream: &AudioStream,
    audio_data: *mut c_void,
    num_frames: i32,
) -> AudioCallbackResult {
    // What:     `let count = (num_frames.max(0) as usize) * (CHANNELS as usize);`.
    //           Compute how many `f32` SAMPLES the buffer holds (frames times
    //           channels). Piece by piece:
    //           - `num_frames.max(0)` clamps the `i32` frame count to be at
    //             least `0` (it stays `i32`).
    //           - `... as usize` casts that to `usize`, the pointer-wide unsigned
    //             integer used for sizes and counts. Siblings: `u32`, `u64`,
    //             `i32`; `usize` is the one memory/length APIs expect.
    //           - `(CHANNELS as usize)` casts our `i32` channel constant to
    //             `usize` so the multiplication is `usize * usize`.
    // Why:      `write_bytes` (next) needs a COUNT of `f32` slots to zero, and
    //           that count must be a `usize`.
    // Gotcha:   the `.max(0)` BEFORE the `as usize` cast is load-bearing: a
    //           negative `i32` cast directly to `usize` would WRAP to an enormous
    //           positive number, and we would then try to zero gigabytes. Clamp
    //           first, cast second.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const count = Math.max(numFrames, 0) * CHANNELS;
    // ```
    let count = (num_frames.max(0) as usize) * (CHANNELS as usize);

    // SAFETY: for a PCM_Float output stream AAudio guarantees `audio_data` points
    // to `num_frames * channels` writable f32 slots; the byte 0 fills them with 0.0.
    // What:     `unsafe { std::ptr::write_bytes(audio_data as *mut f32, 0, count); }`.
    //           Fill the output buffer with zeros.
    //           - `unsafe { ... }` is an escape-hatch block: it promises the
    //             compiler we have personally checked a soundness rule it cannot
    //             verify (here, that `audio_data` really points to `count`
    //             writable `f32`s). The `// SAFETY:` comment above states exactly
    //             why that promise holds and is LOAD-BEARING.
    //           - `audio_data as *mut f32` REINTERPRETS the raw `*mut c_void`
    //             (untyped bytes) as a raw pointer to `f32`s, so the count below
    //             is counted in `f32`-sized steps.
    //           - `std::ptr::write_bytes(ptr, 0, count)` writes the byte value
    //             `0` into `count` consecutive `f32` slots starting at `ptr`. A
    //             `f32` made of all-zero bytes IS `0.0`, so this is silence.
    // Why:      The buffer must be filled every callback or the speaker would get
    //           uninitialized garbage; silence (`0.0`) keeps the probe inaudible
    //           while the stream keeps running.
    // Gotcha:   `unsafe` does NOT mean "dangerous magic"; it means "the compiler
    //           trusts ME here". If the AAudio guarantee in the SAFETY comment
    //           were ever false, this would be undefined behaviour, not a
    //           catchable exception. TS has no `unsafe` because it never hands
    //           you a raw, length-less memory address like this.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // viewing the raw buffer as floats, then zeroing `count` of them:
    // new Float32Array(audioData, 0, count).fill(0); // 0 == silence
    // ```
    unsafe {
        std::ptr::write_bytes(audio_data as *mut f32, 0, count);
    }

    // What:     `AudioCallbackResult::Continue`. A bare enum VARIANT with no
    //           trailing `;` as the function's last line, so it is the TAIL
    //           expression and becomes the return value. `AudioCallbackResult` is
    //           the enum; `::Continue` selects its "keep the stream running"
    //           case (sibling variant: `::Stop`, which would end the stream).
    // Why:      Tell AAudio to invoke us again next cycle so the stream keeps
    //           flowing for the whole settle period.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return "continue";
    // ```
    return AudioCallbackResult::Continue
}

/// What:
///      `fn read_latency_ms(stream: &AudioStream, rate: i32) -> Option<f64>`.
///           A private helper computing the output latency in milliseconds.
///           Parameters:
///           - `stream: &AudioStream`.
///  A read-only BORROW of the stream (the `&`
///             means "lent,
///  not owned");
///  we only query it,
///  never keep it.
///           - `rate: i32`.
///  The stream's actual sample rate,
///  passed by copy.
///           Returns `Option<f64>`:
///  the latency in ms (`Some(x)`) or `None` when
///           the numbers do not make sense (bad rate,
///  or a negative buffer).
/// Why:
///       Isolate the timestamp arithmetic so it can be reasoned about and
///           tested apart from the stream lifecycle in `measure_output_latency_ms`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readLatencyMs(stream: AudioStream, rate: number): number | null { ... }
/// ```
fn read_latency_ms(stream: &AudioStream, rate: i32) -> Option<f64> {
    // What:     `if rate <= 0 { return None; }`. Guard clause. If the sample rate
    //           is zero or negative the division below would be meaningless (or a
    //           divide-by-zero), so we bail. `return None;` is an EARLY return of
    //           the "nothing" case of `Option`.
    // Why:      Refuse to compute a latency from a nonsensical rate; the caller
    //           treats `None` as "measurement unavailable".
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (rate <= 0) return null;
    // ```
    if rate <= 0 {
        return None;
    }

    // What:     `let timestamp = stream.timestamp(Clockid::Monotonic).ok()?;`.
    //           Read the stream's PRESENTATION timestamp.
    //           - `stream.timestamp(Clockid::Monotonic)` asks for the timestamp
    //             measured against the monotonic clock (`Clockid::Monotonic` is
    //             one variant of the `Clockid` enum: a clock that only ever moves
    //             forward, never jumping like wall-clock time can). It returns a
    //             `Result<Timestamp, _>`.
    //           - `.ok()` converts that `Result` to an `Option`, dropping any
    //             error.
    //           - `?` unwraps `Some(timestamp)`, or returns `None` from
    //             `read_latency_ms` if the timestamp was unavailable.
    // Why:      The timestamp tells us which frame the HARDWARE has actually
    //           presented, which we need to compute how far ahead we are buffered.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const timestamp = stream.timestamp("monotonic");
    // if (!timestamp) return null;
    // ```
    let timestamp = stream.timestamp(Clockid::Monotonic).ok()?;

    // What:     `let buffered = stream.frames_written() - timestamp.frame_position;`.
    //           Compute how many frames are buffered ahead of the speaker.
    //           - `stream.frames_written()` is how many frames the APP has pushed
    //             into the stream so far.
    //           - `timestamp.frame_position` is which frame the HARDWARE has
    //             already presented (played).
    //           - Their difference is the backlog: frames written but not yet
    //             heard. The type is inferred (no annotation); it is a signed
    //             integer frame delta, so it CAN be negative if the numbers race.
    // Why:      That backlog, divided by the sample rate, IS the output latency
    //           (how long until a just-written sample reaches the DAC).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const buffered = stream.framesWritten() - timestamp.framePosition;
    // ```
    let buffered = stream.frames_written() - timestamp.frame_position;

    // What:     `if buffered < 0 { return None; }`. Second guard. A negative
    //           backlog means the two counters were read out of step (a transient
    //           race) and the number is not trustworthy, so bail with `None`.
    // Why:      Better to report "no measurement" than a nonsensical negative
    //           latency.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (buffered < 0) return null;
    // ```
    if buffered < 0 {
        return None;
    }

    // What:     `Some((buffered as f64) / (rate as f64) * MILLIS_PER_SEC)`. The
    //           TAIL expression (no trailing `;`), so it is the return value.
    //           - `buffered as f64` casts the integer frame backlog to a 64-bit
    //             float.
    //           - `rate as f64` casts the `i32` sample rate to a 64-bit float so
    //             the division is float division (integer division would
    //             truncate).
    //           - `(buffered / rate)` is the latency in SECONDS; multiplying by
    //             `MILLIS_PER_SEC` (1000.0) converts to milliseconds.
    //           - `Some(...)` wraps the resulting `f64` in the "has a value" case
    //             of `Option`, because the signature promises `Option<f64>`.
    // Why:      Hand back the computed latency in milliseconds as a present value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return (buffered / rate) * MILLIS_PER_SEC;
    // ```
    return Some((buffered as f64) / (rate as f64) * MILLIS_PER_SEC)
}
