//! Linux output with an ALSA fallback (built only with the `alsa-fallback` cargo feature, which the Alpine build
//! turns on).
//!  It tries the PipeWire backend first and, when no PipeWire daemon answers,
//!  opens the default ALSA device through cpal.
//!  The engine and the controller keep calling `output::Output` and never learn which backend is behind it.
//!
//! Mental model for a TypeScript reader:
//!  this file is a small dispatcher.
//!  `Output` is a tagged union of the two real backends (the sibling files `output_pipewire.rs` and
//!  `output_cpal.rs`),
//!  and every method forwards to whichever variant `Output::new` managed to create.
//!  The choice is made once,
//!  at startup,
//!  and never changes during the run.

/// What:
///      `#[path = "output_pipewire.rs"] mod pipewire_backend;` loads `src/output_pipewire.rs` as a PRIVATE child
///           module named `pipewire_backend`.
///  `#[path = "..."]` overrides the default file name,
///  which would be `src/pipewire_backend.rs`.
/// Why:
///       The PipeWire backend already defines a type called `Output`;
///  as a child module it can sit beside the cpal backend's own `Output` without a name clash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as pipewireBackend from "./output_pipewire";
/// ```
#[path = "output_pipewire.rs"]
mod pipewire_backend;

/// What:
///      `#[path = "output_cpal.rs"] mod alsa_backend;` loads `src/output_cpal.rs` as a private child module named
///           `alsa_backend`.
///  On Linux cpal drives ALSA.
/// Why:
///       The cpal backend is platform neutral (it only calls cpal's public API),
///  so the macOS and Windows file serves the ALSA path unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as alsaBackend from "./output_cpal";
/// ```
#[path = "output_cpal.rs"]
mod alsa_backend;

/// What:
///      `use std::thread::Thread;` imports the handle type of a running thread.
///  Siblings the reader might expect:
///  `JoinHandle<T>` (owns the right to wait for a thread) and `ThreadId` (only a number naming it).
/// Why:
///       The engine hands `Output::new` the decode thread's handle so the realtime callback can wake it;
///  a `Thread` is exactly that wake-up handle,
///  where `JoinHandle` would be the wrong type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // No equivalent: a worker reference you can post a wake-up message to.
/// ```
use std::thread::Thread;

/// What:
///      `use cpal::traits::{DeviceTrait, HostTrait};` imports two traits, which are named sets of methods.
///  A trait's methods can be called on a value only when the trait is in scope (TS interfaces need no import for that).
/// Why:
///       `open_alsa` calls `default_output_device` (from `HostTrait`) and `default_output_config` (from `DeviceTrait`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { DeviceTrait, HostTrait } from "cpal";
/// ```
use cpal::traits::{DeviceTrait, HostTrait};

/// What:
///      `use ringbuf::HeapProd;` imports the WRITE end of the lock-free ring buffer that carries samples to the
///           realtime audio callback.
/// Why:
///       `reconfigure` returns it to the controller,
///  so this file must name the type in the signature it forwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { RingWriter } from "ringbuf";
/// ```
use ringbuf::HeapProd;

/// What:
///      `use crate::audio_meter::MeterSnapshot;` imports the plain record of underrun counters.
/// Why:
///       `meter_snapshot` returns it,
///  so the forwarding signature names it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { MeterSnapshot } from "./audio_meter";
/// ```
use crate::audio_meter::MeterSnapshot;

/// What:
///      `use crate::error::PlayerError;` imports the app's error enum.
/// Why:
///       Output creation and reconfiguration report failure as a `PlayerError::Audio`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { PlayerError } from "./error";
/// ```
use crate::error::PlayerError;

/// What:
///      `pub const FILL_FRAMES_FLOOR: usize = ...;` re-exports the smallest number of frames one fill step produces.
///  `usize` is the unsigned integer sized to the machine (siblings:  `u32`, `u64`, `i32`).
/// Why:
///       `controller_audio.rs` reads it through `crate::output::FILL_FRAMES_FLOOR`.
///  `usize` matches that call site,
///  which casts it to `u64` once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const FILL_FRAMES_FLOOR: number = pipewireBackend.FILL_FRAMES_FLOOR;
/// ```
pub const FILL_FRAMES_FLOOR: usize = pipewire_backend::FILL_FRAMES_FLOOR;

// What:     `const _: () = assert!(...);` is a compile-time check. `const _` is a constant with no name, `()` is
//           the empty value type, and `assert!` is a macro that fails the BUILD (not the run) when the condition is
//           false.
// Why:      The controller reads one floor for whichever backend is active. If the two backends ever disagree the
//           build must stop instead of letting the drain timing drift on one of them.
//
// In TS you'd write (pseudocode):
// ```ts
// // No equivalent: TS cannot fail the build on a value comparison.
// ```
const _: () = assert!(pipewire_backend::FILL_FRAMES_FLOOR == alsa_backend::FILL_FRAMES_FLOOR);

/// What:
///      `fn open_alsa(worker: Thread) -> Result<alsa_backend::Output, PlayerError>` opens the cpal backend only after
///           proving that the default ALSA device can be opened.
///  `cpal::default_host().default_output_device()` always answers `Some` on ALSA, even on a machine with no sound
///           card, because it only names the device called "default" and never opens it.
///  `default_output_config()` opens the device to read its capabilities, so it fails when there is no card.
/// Why:
///       Without the probe the fallback would claim success on a machine with no sound at all,
///  and the window would never learn that nothing can be heard;  with it, startup fails here and the engine sends the
///  "no sound output" message instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function openAlsa(worker: ThreadRef): AlsaOutput {
///   const device = defaultHost().defaultOutputDevice();
///   if (!device) throw new PlayerError("no default output device");
///   device.defaultOutputConfig(); // throws when the device cannot be opened
///   return AlsaOutput.create(worker);
/// }
/// ```
fn open_alsa(worker: Thread) -> Result<alsa_backend::Output, PlayerError> {
    // What:     `let host = cpal::default_host();` asks cpal for the platform's audio host (ALSA on Linux).
    // Why:      The device is reached through the host.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const host = defaultHost();
    // ```
    let host = cpal::default_host();
    // What:     `host.default_output_device().ok_or_else(|| ...)?` turns the `Option` into a `Result` and returns early
    //           with the error when it is `None`. `?` is Rust's early-return-on-error operator (like a TS `throw`
    //           that needs no `try`). `|| return PlayerError::Audio(...)` is a closure (an inline function with no
    //           arguments); the explicit `return` keeps the repository's `implicit_return` rule satisfied.
    // Why:      The same failure message the cpal backend itself uses when no device exists.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const device = host.defaultOutputDevice() ?? (() => { throw new PlayerError("no default output device"); })();
    // ```
    let device = host
        .default_output_device()
        .ok_or_else(|| return PlayerError::Audio("no default output device".to_string()))?;
    // What:     `device.default_output_config()` opens the device and reads its default format; `.map_err(|e| ...)`
    //           rewrites a failure into `PlayerError::Audio` with `format!`, a template-literal-like macro; `{e:?}`
    //           prints the error in its debug form; `?` returns early on failure. `let _config` binds the success
    //           value only to discard it (the leading underscore says it is unused).
    // Why:      This call is the proof that a sound card or an ALSA plugin answers.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { device.defaultOutputConfig(); } catch (e) { throw new PlayerError(`the default ALSA device cannot be opened: ${e}`); }
    // ```
    let _config = device
        .default_output_config()
        .map_err(|e| return PlayerError::Audio(format!("the default ALSA device cannot be opened: {e:?}")))?;
    // What:     `return alsa_backend::Output::new(worker)` builds the real backend and returns its `Result` as is.
    // Why:      The probe passed, so create the output the controller will use.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return AlsaOutput.create(worker);
    // ```
    return alsa_backend::Output::new(worker);
}

/// What:
///      `pub(crate) fn both_failed_message(pipewire_error, alsa_error) -> String` words the single error shown when
///           neither backend started.
///  `&PlayerError` borrows each error read only (siblings:  `PlayerError` by value would consume it).
///  `String` is an owned text buffer (sibling:  `&str`, a borrowed view that could not outlive this call).
/// Why:
///       A pure function can be unit tested without a sound server;  the message names both backends so the log
///  explains why the player is silent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function bothFailedMessage(pipewireError: PlayerError, alsaError: PlayerError): string {
///   return `PipeWire: ${pipewireError}; ALSA: ${alsaError}`;
/// }
/// ```
pub(crate) fn both_failed_message(pipewire_error: &PlayerError, alsa_error: &PlayerError) -> String {
    // What:     `format!(...)` builds an owned `String` from a template, like a TS template literal. The `{name}`
    //           holes print each error's human-readable `Display` text. This line is the tail expression, so it is
    //           the return value.
    // Why:      One string carries both causes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return `PipeWire: ${pipewireError}; ALSA: ${alsaError}`;
    // ```
    return format!("PipeWire: {pipewire_error}; ALSA: {alsa_error}");
}

/// What:
///      `pub enum Output { PipeWire(...), Alsa(...) }` declares a tagged union with two variants,
///  each wrapping one backend's own `Output`.
///  Siblings the reader might expect:  a `Box<dyn Trait>` would also work but needs a trait,
///  an allocation and dynamic dispatch.
/// Why:
///       The two backends share a method surface but no trait,
///  and exactly one is chosen at startup,
///  so a plain enum is the smallest correct shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Output = { kind: "pipewire"; inner: PipewireOutput } | { kind: "alsa"; inner: AlsaOutput };
/// ```
pub enum Output {
    /// The PipeWire backend, chosen when a PipeWire daemon answered.
    PipeWire(pipewire_backend::Output),
    /// The ALSA backend through cpal, chosen when PipeWire was unreachable.
    Alsa(alsa_backend::Output),
}

impl Output {
    /// What:
    ///      `pub fn new(worker: Thread) -> Result<Output, PlayerError>` starts the first backend that works.
    ///  `Result<Output, PlayerError>` is Rust's "success value or error value" type (TS has no equivalent;  it
    ///  would throw).
    /// Why:
    ///       PipeWire is preferred because it mixes with the other applications of a desktop;  ALSA is the fallback
    ///  for a machine with no sound server.
    ///  When both fail the error carries both reasons so the log explains the silence.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function create(worker: ThreadRef): Output {
    ///   try { return { kind: "pipewire", inner: PipewireOutput.create(worker) }; }
    ///   catch (pipewireError) { /* fall through */ }
    ///   try { return { kind: "alsa", inner: AlsaOutput.create(worker) }; }
    ///   catch (alsaError) { throw new PlayerError(`${pipewireError}; ${alsaError}`); }
    /// }
    /// ```
    pub fn new(worker: Thread) -> Result<Output, PlayerError> {
        // What:     `worker.clone()` duplicates the `Thread` handle. A `Thread` is a cheap reference-counted handle,
        //           so the clone points at the same running thread.
        // Why:      The first attempt consumes its argument, and the second attempt still needs one.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const first = worker; // a second reference to the same worker
        // ```
        let pipewire_attempt = pipewire_backend::Output::new(worker.clone());
        // What:     `match pipewire_attempt { Ok(output) => ..., Err(error) => ... }` takes the result apart. `Ok`
        //           is the success variant, `Err` the failure variant.
        // Why:      Success returns at once;  failure is logged and the ALSA attempt follows.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (pipewireAttempt.ok) return wrap(pipewireAttempt.value);
        // ```
        match pipewire_attempt {
            // What:     `Ok(output) => return Ok(Output::PipeWire(output))` wraps the working backend in the enum
            //           variant and in the success wrapper `Ok(...)`.
            // Why:      PipeWire answered, so it is the backend for this run.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return { kind: "pipewire", inner: output };
            // ```
            Ok(output) => return Ok(Output::PipeWire(output)),
            // What:     `Err(error) => { ... }` runs when PipeWire could not start. `error` is the `PlayerError`
            //           that explains why.
            // Why:      Fall through to ALSA, keeping the reason for a combined message if ALSA fails as well.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // catch (error) { ... }
            // ```
            Err(pipewire_error) => {
                // What:     `tracing::info!(...)` writes one structured log line. `%pipewire_error` formats the
                //           value with its human-readable `Display` text.
                // Why:      Record the backend choice so a silent run can be explained from the log.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // logger.info("PipeWire unavailable, trying ALSA", pipewireError);
                // ```
                tracing::info!(error = %pipewire_error, "PipeWire is unavailable; trying the default ALSA device");
                // What:     `alsa_backend::Output::new(worker)` tries the cpal backend. `match` takes its result
                //           apart as above.
                // Why:      The last chance for sound on this machine.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // try { return wrap(AlsaOutput.create(worker)); } catch (alsaError) { throw ... }
                // ```
                match open_alsa(worker) {
                    // What:     `Ok(output) => return Ok(Output::Alsa(output))` wraps the working ALSA backend.
                    // Why:      ALSA opened a default device, so it is the backend for this run.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // return { kind: "alsa", inner: output };
                    // ```
                    Ok(output) => {
                        tracing::info!("audio output: the default ALSA device");
                        return Ok(Output::Alsa(output));
                    }
                    // What:     `Err(alsa_error) => return Err(PlayerError::Audio(format!(...)))` builds one error
                    //           from both failures. `format!` builds an owned `String` from a template, like a
                    //           TS template literal. `PlayerError::Audio(...)` is the enum variant for audio errors.
                    // Why:      The caller logs one error and the person sees the whole story.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // throw new PlayerError(`PipeWire: ${pipewireError}; ALSA: ${alsaError}`);
                    // ```
                    Err(alsa_error) => {
                        return Err(PlayerError::Audio(both_failed_message(&pipewire_error, &alsa_error)));
                    }
                }
            }
        }
    }

    /// What:
    ///      `pub fn reconfigure(&mut self, rate, channels, capacity_frames)` forwards to the active backend.
    ///  `&mut self` lends this value out for changing (siblings:  `&self` read only, `self` consumes it).
    /// Why:
    ///       The controller calls it once per track format;  it returns the write end of the ring buffer.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// reconfigure(rate: number, channels: number, capacityFrames: number): RingWriter {
    ///   return this.inner.reconfigure(rate, channels, capacityFrames);
    /// }
    /// ```
    pub fn reconfigure(
        &mut self,
        rate: u32,
        channels: u16,
        capacity_frames: usize,
    ) -> Result<HeapProd<f32>, PlayerError> {
        // What:     `match self { ... }` picks the variant and borrows the inner backend mutably (`output`).
        // Why:      One forwarding arm per backend.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return this.kind === "pipewire" ? this.inner.reconfigure(...) : this.inner.reconfigure(...);
        // ```
        match self {
            Output::PipeWire(output) => return output.reconfigure(rate, channels, capacity_frames),
            Output::Alsa(output) => return output.reconfigure(rate, channels, capacity_frames),
        }
    }

    /// What:
    ///      `pub fn set_playing(&self, on: bool)` forwards the play/pause flag to the active backend.
    /// Why:
    ///       The realtime callback reads the flag to decide between samples and silence.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// setPlaying(on: boolean): void { this.inner.setPlaying(on); }
    /// ```
    pub fn set_playing(&self, on: bool) {
        match self {
            Output::PipeWire(output) => output.set_playing(on),
            Output::Alsa(output) => output.set_playing(on),
        }
    }

    /// What:
    ///      `pub fn set_draining(&self, on: bool)` forwards the "play out what is buffered" flag.
    /// Why:
    ///       A format change plays the ring out before the output is rebuilt.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// setDraining(on: boolean): void { this.inner.setDraining(on); }
    /// ```
    pub fn set_draining(&self, on: bool) {
        match self {
            Output::PipeWire(output) => output.set_draining(on),
            Output::Alsa(output) => output.set_draining(on),
        }
    }

    /// What:
    ///      `pub fn meter_snapshot(&self) -> MeterSnapshot` forwards the underrun counters read.
    /// Why:
    ///       The controller logs an underrun report once a second.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// meterSnapshot(): MeterSnapshot { return this.inner.meterSnapshot(); }
    /// ```
    pub fn meter_snapshot(&self) -> MeterSnapshot {
        match self {
            Output::PipeWire(output) => return output.meter_snapshot(),
            Output::Alsa(output) => return output.meter_snapshot(),
        }
    }
}

#[cfg(test)]
#[path = "output_linux_tests.rs"]
mod tests;
