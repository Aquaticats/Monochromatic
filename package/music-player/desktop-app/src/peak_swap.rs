//! Current-track true-peak swap strategy.
//!
//! Cache hits keep the old zero-delay path:
//!  the measured gain is available before
//! any samples are decoded.
//!  Cache misses start a dedicated measurement thread for
//! the current track,
//!  immediately use the safe -1 dBTP ceiling gain,
//!  and let the
//! controller apply it on the next poll, so playback never waits for it.
//!  If the
//! measurement finishes later,
//!  the controller swaps future decoded samples to the
//! measured gain while already-buffered fallback samples drain unchanged.

/// What:
///      `use std::path::{Path, PathBuf};`.
///  `Path` is a borrowed filesystem
///           path view;
///  `PathBuf` is the owned,
///  growable path buffer sibling.
/// Why:
///       Public helpers borrow the loaded track path,
///  while the worker thread
///           must own its path after the caller returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Path = string;
/// ```
use std::path::{Path, PathBuf};

/// What:
///      `use std::sync::mpsc::{self, Receiver, TryRecvError};`.
///           Rust's multi-producer/single-consumer channel module plus the receive
///           half and an error enum.
///  `Receiver<T>` is the owned read end;
///           `TryRecvError` reports non-blocking empty/disconnected states.
/// Why:
///       The measurement worker sends exactly one result back to the controller
///           without blocking the audio thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const channel = makeOneShotChannel<PeakGainResult>();
/// ```
use std::sync::mpsc::{self, Receiver, TryRecvError};

/// What:
///      `use std::thread;`.
///  Rust's standard OS-thread API.
/// Why:
///       Current-track measurement decodes the whole file away from the
///           controller thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const worker = new Worker("measure-current-track.js");
/// ```
use std::thread;

/// What:
///      `use crate::peakcache::{self, CacheHandle};`.
///  Import the peak-cache
///           module itself for `peakcache::fingerprint`,
///  plus the `CacheHandle` type.
/// Why:
///       The strategy checks cached peaks before spawning and stores fresh
///           measurements after decoding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as peakcache from "./peakcache";
/// import { CacheHandle } from "./peakcache";
/// ```
use crate::peakcache::{self, CacheHandle};

/// What:
///      `use crate::truepeak::{normalization_gain, resolve_current};`.
///  The shared
///           foreground resolver (probe-or-full) and the peak-to-gain conversion used only
///           for the cold-start fallback.
/// Why:
///       Cache misses resolve the current track's decision;
///  `normalization_gain` builds
///           the -1 dBTP fallback while measurement runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { normalizationGain, resolveCurrent } from "./truepeak";
/// ```
use crate::truepeak::{normalization_gain, resolve_current};
use truepeak_core::Decision;

// What:     `#[derive(Clone, Copy, Debug)]` auto-generates cheap copying and debug
//           formatting for `PeakGainResult`. `Copy` is valid because both fields
//           are plain numbers.
// Why:      Tests can print the result, and the controller can pass it around
//           without ownership ceremony.
//
// In TS you'd write (pseudocode):
// ```ts
// type PeakGainResult = { generation: number; gain: number };
// ```
#[derive(Clone, Debug)]
/// What:
///      `pub(crate) struct PeakGainResult { ... }`.
///  A crate-visible result
///           from one current-track measurement.
///  It carries the generation that
///           was current when the worker spawned plus the measured gain.
/// Why:
///       Generation lets the controller ignore stale results from tracks that
///           are no longer current while still allowing those workers to warm the cache.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PeakGainResult = { generation: number; gain: number };
/// ```
pub(crate) struct PeakGainResult {
    /// What:
    ///      `pub(crate) generation: u64`.
    ///  Monotonic track-load generation,
    ///           visible inside the crate.
    ///  `u64` is used instead of `usize` so the
    ///           value is platform-independent and effectively never wraps.
    /// Why:
    ///       Identify which loaded track this result belongs to.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// generation: number;
    /// ```
    pub(crate) generation: u64,
    /// What:
    ///      `pub(crate) path: PathBuf`. The track this measurement belongs to.
    /// Why:
    ///       The window's row for this track takes the measure from the decision, keyed by path.
    pub(crate) path: PathBuf,
    /// What:
    ///      `pub(crate) decision: Decision`. The full decision the measurement stored in the cache.
    /// Why:
    ///       The result carries the decision itself, so the row can show it without a racing cache read.
    pub(crate) decision: Decision,
    /// What:
    ///      `pub(crate) gain: f32`.
    ///  Linear playback gain from true-peak
    ///           normalization.
    ///  Sibling `f64` would add precision the PCM path does
    ///           not use.
    /// Why:
    ///       The controller multiplies future decoded samples by this value.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// gain: number;
    /// ```
    pub(crate) gain: f32,
}

/// What:
///      `pub(crate) enum PendingPeakStatus { ... }`.
///  The three states a
///           non-blocking or timed receive can report:
///  result ready,
///  still pending,
///           or the worker ended without sending.
/// Why:
///       The controller needs one state machine for polling the measurement.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PendingPeakStatus =
///   | { kind: "ready"; result: PeakGainResult }
///   | { kind: "pending" }
///   | { kind: "closed" };
/// ```
pub(crate) enum PendingPeakStatus {
    /// What:
    ///      `Ready(PeakGainResult)` wraps the measured result.
    /// Why:
    ///       Hand the generation and gain to the controller.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: "ready", result }
    /// ```
    Ready(
        /// What:
        ///      Unnamed field `.0` of the `Ready` variant:
        ///  the measured
        ///           `PeakGainResult` (the sibling struct declared above,
        ///  carrying a
        ///           `generation` and a `gain`).
        ///  Not an `f32` alone:
        ///  the controller
        ///           needs the generation too,
        ///  to reject stale results.
        /// Why:
        ///       Hands the generation and gain to the controller when a result
        ///           lands.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// // the `result: PeakGainResult` payload of { kind: "ready" }
        /// ```
        PeakGainResult,
    ),
    /// What:
    ///      `Pending` has no payload.
    /// Why:
    ///       The worker is still running,
    ///  so fallback gain remains active.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: "pending" }
    /// ```
    Pending,
    /// What:
    ///      `Closed` has no payload.
    /// Why:
    ///       Measurement failed or the worker exited without a result,
    ///  so the
    ///           controller can stop polling and keep the fallback gain.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: "closed" }
    /// ```
    Closed,
}

/// What:
///      `pub(crate) struct PendingPeakMeasurement { ... }`.
///  The controller's
///           handle to one in-flight current-track measurement.
/// Why:
///       Encapsulates the channel receiver so command handling can poll
///           without knowing mpsc error details.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PendingPeakMeasurement { constructor(private receiver: Receiver<PeakGainResult>) {} }
/// ```
pub(crate) struct PendingPeakMeasurement {
    /// What:
    ///      `receiver: Receiver<PeakGainResult>`.
    ///  The owned read end of the
    ///           one-shot channel.
    ///  The sibling write end lives in the measurement thread.
    /// Why:
    ///       Poll for the measured gain.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// receiver: Receiver<PeakGainResult>;
    /// ```
    receiver: Receiver<PeakGainResult>,
}

/// What:
///      `impl PendingPeakMeasurement { ... }`.
///  Methods for the pending
///           measurement wrapper.
/// Why:
///       Keep raw channel details local to this module.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PendingPeakMeasurement { /* fromReceiver, tryResult */ }
/// ```
impl PendingPeakMeasurement {
    /// What:
    ///      `pub(crate) fn from_receiver(receiver: Receiver<PeakGainResult>) -> PendingPeakMeasurement`.
    ///           Build a wrapper from an owned channel receiver.
    /// Why:
    ///       Production spawn and unit tests both need to construct pending handles.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static fromReceiver(receiver: Receiver<PeakGainResult>): PendingPeakMeasurement {
    ///   return new PendingPeakMeasurement(receiver);
    /// }
    /// ```
    pub(crate) fn from_receiver(receiver: Receiver<PeakGainResult>) -> PendingPeakMeasurement {
        // What:     `PendingPeakMeasurement { receiver }`. Struct literal using field
        //           shorthand. Tail expression returns the wrapper.
        // Why:      Store the receiver.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { receiver };
        // ```
        return PendingPeakMeasurement { receiver }
    }

    /// What:
    ///      `pub(crate) fn try_result(&self) -> PendingPeakStatus`.
    ///  Poll once
    ///           without blocking.
    /// Why:
    ///       The engine loop checks whether a measured gain landed before
    ///           decoding another chunk.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// tryResult(): PendingPeakStatus { return this.receiver.tryRead(); }
    /// ```
    pub(crate) fn try_result(&self) -> PendingPeakStatus {
        // What:     `match self.receiver.try_recv() { ... }`. Non-blocking channel
        //           receive, mapped into the simpler status enum.
        // Why:      Avoid parking the engine while audio should keep flowing.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // switch (this.receiver.tryRead()) { /* value, empty, closed */ }
        // ```
        match self.receiver.try_recv() {
            // What:     `Ok(result) => PendingPeakStatus::Ready(result)`. Channel had
            //           a result, so wrap it in the ready status.
            // Why:      Hand the measured gain upward.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return { kind: "ready", result };
            // ```
            Ok(result) => return PendingPeakStatus::Ready(result),
            // What:     `Err(TryRecvError::Empty) => PendingPeakStatus::Pending`.
            //           The worker has not sent yet.
            // Why:      Keep the fallback active and poll later.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return { kind: "pending" };
            // ```
            Err(TryRecvError::Empty) => return PendingPeakStatus::Pending,
            // What:     `Err(TryRecvError::Disconnected) => PendingPeakStatus::Closed`.
            //           The sender disappeared without a value.
            // Why:      Measurement failed, so there is nothing left to wait for.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return { kind: "closed" };
            // ```
            Err(TryRecvError::Disconnected) => return PendingPeakStatus::Closed,
        }
    }
}

/// What:
///      `pub(crate) enum TrackGainResolution { ... }`.
///  Result of preparing a
///           loaded track's gain:
///  either immediately ready from cache,
///  or pending
///           on an async measurement.
/// Why:
///       `install_source` can set `track_gain` without blocking on a cache miss.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TrackGainResolution =
///   | { kind: "ready"; gain: number }
///   | { kind: "pending"; pending: PendingPeakMeasurement };
/// ```
pub(crate) enum TrackGainResolution {
    /// What:
    ///      `Ready(f32)` carries an already-known gain.
    /// Why:
    ///       Cache hit path keeps exact normalization from the first sample.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: "ready", gain }
    /// ```
    Ready(
        /// What:
        ///      Unnamed field `.0` of the `Ready` variant:
        ///  an already-known
        ///           linear playback `gain` as an `f32` (32-bit float).
        ///  Sibling the
        ///           reader might expect:
        ///  `f64` (64-bit double).
        /// Why:
        ///       `f32` (not `f64`) matches the PCM path's sample width,
        ///  which
        ///           never uses double precision;
        ///  this is the cache-hit gain the
        ///           controller can apply from the first sample.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// // the `gain: number` payload of { kind: "ready" }
        /// ```
        f32,
    ),
    /// What:
    ///      `Pending(PendingPeakMeasurement)` carries a measurement handle.
    /// Why:
    ///       Cache miss path can start with fallback while the worker decodes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: "pending", pending }
    /// ```
    Pending(
        /// What:
        ///      Unnamed field `.0` of the `Pending` variant:
        ///  a
        ///           `PendingPeakMeasurement` handle (the sibling struct declared
        ///           above,
        ///  wrapping the channel receiver).
        ///  Not a raw `Receiver`:
        ///  the
        ///           wrapper hides the mpsc error details behind the poll method.
        /// Why:
        ///       Lets the controller start with fallback gain while the worker
        ///           decodes,
        ///  then poll this handle for the measured gain.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// // the `pending: PendingPeakMeasurement` payload of { kind: "pending" }
        /// ```
        PendingPeakMeasurement,
    ),
}

/// What:
///      `pub(crate) fn fallback_track_gain() -> f32`.
///  Return the temporary
///           gain used while an uncached true peak is still measuring.
/// Why:
///       `normalization_gain(1.0)` is exactly the existing -1 dBTP ceiling value,
///           so fallback keeps a clipping-safety bias without waiting forever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fallbackTrackGain(): number { return normalizationGain(1); }
/// ```
pub(crate) fn fallback_track_gain() -> f32 {
    // What:     `normalization_gain(1.0)`. Convert a full-scale peak into its
    //           attenuate-only gain, which is the -1 dBTP ceiling.
    // Why:      Reuse the true-peak module's canonical constant without duplicating it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return normalizationGain(1);
    // ```
    return normalization_gain(1.0)
}

/// What:
///      `pub(crate) fn cached_track_gain(path: &Path, cache: &CacheHandle) -> Option<f32>`.
///           Try to read a measured peak from the cache and convert it to gain.
/// Why:
///       Cache hits must avoid spawning a redundant current-track worker.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cachedTrackGain(path: string, cache: CacheHandle): number | null { ... }
/// ```
pub(crate) fn cached_track_gain(path: &Path, cache: &CacheHandle) -> Option<f32> {
    // What:     `let key = peakcache::fingerprint(path)?;`. Compute the opaque `u64` cache
    //           key; `?` returns `None` if the file cannot be stat'd.
    // Why:      No key means there can be no cache hit.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const key = fingerprint(path); if (!key) return null;
    // ```
    let key = peakcache::fingerprint(path)?;
    // What:     `let decision = cache.get(key)?;`. Ask the cache actor for the decision and
    //           use `?` to return `None` on a miss. `key` is a `Copy` `u64`.
    // Why:      Use only real cache hits.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const decision = cache.get(key); if (decision == null) return null;
    // ```
    let decision = cache.get(key)?;
    // What:     `Some(decision.gain)`. The cached decision already carries the gain (the
    //           shared resolver computed it), so no re-conversion is needed. Tail expression
    //           returns the cache-hit gain.
    // Why:      Callers need gain; the shared cache stores decisions, not raw peaks.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return decision.gain;
    // ```
    return Some(decision.gain)
}

/// What:
///      `pub(crate) fn prepare_track_gain(...) -> TrackGainResolution`.
///  Prepare
///           the loaded track's gain with the swap strategy.
/// Why:
///       A cache hit returns exact gain now;
///  a miss starts measurement and lets
///           playback begin at once with the fallback gain until the result lands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function prepareTrackGain(path: string, cache: CacheHandle, generation: number): TrackGainResolution { ... }
/// ```
pub(crate) fn prepare_track_gain(
    path: &Path,
    cache: &CacheHandle,
    generation: u64,
    worker: thread::Thread,
) -> TrackGainResolution {
    // What:     `if let Some(gain) = cached_track_gain(path, cache) { ... }`. Check
    //           the cache before spawning a worker.
    // Why:      Keep known tracks instant and exact.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const gain = cachedTrackGain(path, cache); if (gain != null) return { kind: "ready", gain };
    // ```
    if let Some(gain) = cached_track_gain(path, cache) {
        // What:     `return TrackGainResolution::Ready(gain);`. Early-return the
        //           ready enum variant.
        // Why:      No async work needed.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { kind: "ready", gain };
        // ```
        return TrackGainResolution::Ready(gain);
    }

    // What:     `let fingerprint = peakcache::fingerprint(path);`. Compute the key
    //           once more for the worker. It may be `None`; measurement can still
    //           run, but caching will be skipped.
    // Why:      The worker owns this key and avoids repeating stat after spawn.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const fingerprint = peakcache.fingerprint(path);
    // ```
    let fingerprint = peakcache::fingerprint(path);
    // What:     `let pending = spawn_current_track_measurement(...)`. Start the
    //           dedicated worker. `path.to_path_buf()` clones the borrowed path into
    //           owned storage; `cache.clone()` shares the same cache.
    // Why:      The thread outlives this function, so it needs owned inputs, and the
    //           worker thread handle lets measurement completion wake the engine promptly.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const pending = spawnCurrentTrackMeasurement(path, cache, fingerprint, generation, worker);
    // ```
    let pending = spawn_current_track_measurement(
        path.to_path_buf(),
        cache.clone(),
        fingerprint,
        generation,
        worker,
    );
    // What:     `TrackGainResolution::Pending(pending)`. Wrap the pending handle.
    //           Tail expression returns it.
    // Why:      The controller stores this and uses fallback gain until a result arrives.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { kind: "pending", pending };
    // ```
    return TrackGainResolution::Pending(pending)
}

/// What:
///      `fn spawn_current_track_measurement(...) -> PendingPeakMeasurement`.
///  Spawn
///           a detached worker for one current track.
/// Why:
///       Current-track measurement must not block load,
///  but it should run at normal
///           priority so the exact gain lands soon after a track change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function spawnCurrentTrackMeasurement(...args): PendingPeakMeasurement { ... }
/// ```
fn spawn_current_track_measurement(
    path: PathBuf,
    cache: CacheHandle,
    fingerprint: Option<u64>,
    generation: u64,
    worker: thread::Thread,
) -> PendingPeakMeasurement {
    // What:     `let (sender, receiver) = mpsc::channel::<PeakGainResult>();`. Create
    //           a typed one-shot channel and split it into write/read halves.
    // Why:      The worker sends the measured gain back to the controller.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const { sender, receiver } = makeChannel<PeakGainResult>();
    // ```
    let (sender, receiver) = mpsc::channel::<PeakGainResult>();
    // What:     `thread::spawn(move || { ... });`. Start an OS thread. `move ||`
    //           takes ownership of `path`, `cache`, `fingerprint`, `generation`, `worker`,
    //           and `sender` into the closure.
    // Why:      Decode and cache the peak without tying up the controller thread.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // startWorker(() => { /* measure, cache, send */ });
    // ```
    thread::spawn(move || {
        // What:     `if let Some(gain) = measure_and_store_gain(&path, &cache, fingerprint) { ... }`.
        //           Measure the track, store it when possible, and continue only on
        //           success. `&path` and `&cache` lend the owned worker values.
        // Why:      A decode failure closes the channel, leaving fallback gain active.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const gain = measureAndStoreGain(path, cache, fingerprint); if (gain != null) { ... }
        // ```
        if let Some(decision) = measure_and_store_gain(&path, &cache, fingerprint) {
            // What:     `let _ = sender.send(PeakGainResult { generation, gain });`.
            //           Send the result; `let _ =` discards the error if the controller
            //           already moved to another track and dropped the receiver.
            // Why:      Stale measurements still warm the cache, but do not need a live
            //           consumer.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // try { sender.send({ generation, gain }); } catch {}
            // ```
            let _ = sender.send(PeakGainResult {
                generation,
                gain: decision.gain,
                path,
                decision,
            });
        }
        // What:     `worker.unpark();`. Wake the controller/engine thread after the
        //           measurement either sent a value or ended without one.
        // Why:      The engine polls pending peak results immediately instead of waiting
        //           for its fallback park timeout.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // worker.postWakeUp();
        // ```
        worker.unpark();
    });
    // What:     `PendingPeakMeasurement::from_receiver(receiver)`. Wrap the channel's
    //           read half. Tail expression returns the pending handle.
    // Why:      Hide raw channel details from the controller.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return PendingPeakMeasurement.fromReceiver(receiver);
    // ```
    return PendingPeakMeasurement::from_receiver(receiver)
}

/// What:
///      `fn measure_and_store_gain(...) -> Option<Decision>`.
///  Resolve the current track's
///           decision (probe-or-full),
///  cache it if a fingerprint exists,
///  and return the decision.
/// Why:
///       Shared worker body for the current-track async path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function measureAndStoreGain(path: string, cache: CacheHandle, fingerprint: bigint | null): number | null { ... }
/// ```
fn measure_and_store_gain(
    path: &Path,
    cache: &CacheHandle,
    fingerprint: Option<u64>,
) -> Option<Decision> {
    // What:     `let decision = resolve_current(path).ok()?;`. Resolve the foreground
    //           decision and convert `Result` to `Option`; `?` returns `None` on failure.
    // Why:      A failed measurement should not replace the fallback gain.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let decision; try { decision = resolveCurrent(path); } catch { return null; }
    // ```
    let decision = resolve_current(path).ok()?;
    // What:     `if let Some(key) = fingerprint { ... }`. Cache only when a fingerprint was
    //           available; this consumes the optional `u64`.
    // Why:      Unstatable files can still produce a gain for this play, but cannot be
    //           memoized safely.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (fingerprint) { ... }
    // ```
    if let Some(key) = fingerprint {
        // What:     `cache.upsert(key, decision);`. Fire-and-forget the decision to the cache
        //           actor, which commits it durably (a probe estimate here; warming may later
        //           upgrade it to exact).
        // Why:      Warm the cache even if this result later becomes stale for playback; the
        //           current-track worker must not block on persistence.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // cache.upsert(key, decision);
        // ```
        cache.upsert(key, decision);
    }
    // What:     `Some(decision.gain)`. The decision already carries the gain. Tail expression
    //           returns the worker's result.
    // Why:      The controller applies gains; the shared resolver already computed it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return decision.gain;
    // ```
    return Some(decision)
}

/// What:
///      `#[cfg(test)] #[path = "peak_swap_tests.rs"] mod tests;` declares a
///           test-only child module loaded from the sibling file.
/// Why:
///       Keep tests beside this module without adding production code lines.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // test runner imports peak_swap.unit.test.ts only for tests
/// ```
#[cfg(test)]
#[path = "peak_swap_tests.rs"]
mod tests;
