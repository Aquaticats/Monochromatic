//! Android true-peak resolution:
//!  a thin adapter that drives the shared policy resolver.
//!
//! "True peak" (a.k.a.
//!  inter-sample peak) is the highest level the analog waveform reaches
//! AFTER a DAC reconstructs it between the stored samples;
//!  it can sit above the largest
//! stored sample.
//!  The measurement policy (full-scan short tracks,
//!  probe long ones),
//!  the
//! Catmull-Rom meter,
//!  the attenuate-only gain,
//!  the window placement,
//!  and the decision cache
//! now all live once in the shared `truepeak-core` crate.
//!  Android's old windowed policy (a
//! `1.26` safety factor over four fixed windows) is GONE,
//!  replaced by the shared proportional
//! probe.
//!  This module keeps only the Android glue:
//!  a [`TruePeakSource`] adapter over the
//! Android `decode::Source`,
//!  and two resolvers that drive an already-opened source (the JNI
//! opens it from a `content://` fd).
//!  The native service handle (`src/service.rs`) calls these
//! and caches the resulting decisions.
//!  See
//! ../../../doc/handover/music-player-truepeak-core-integration.md.

/// What:
///      `use crate::decode::Source;`.
///  The decoder trait,
///  named by the adapter's `inner`
///           field and driven through its `spec`/`next_chunk`/`seek` methods.
/// Why:
///       The adapter wraps a `Box<dyn Source>` and forwards to it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Source } from "./decode";
/// ```
use crate::decode::Source;

/// What:
///      `use truepeak_core::{AudioSpec, Decision, TruePeakError, TruePeakSource,
///           default_policy, resolve_decision, resolve_full_scan};`.
///  The shared source
///           contract and descriptor,
///  the decision type,
///  the crate error,
///  the shipped policy,
///           and the two resolvers.
/// Why:
///       The adapter implements `TruePeakSource`;
///  the resolvers drive it under
///           `default_policy` and return a `Decision`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { AudioSpec, Decision, TruePeakError, TruePeakSource, defaultPolicy, resolveDecision, resolveFullScan } from "truepeak-core";
/// ```
use truepeak_core::{
    AudioSpec, Decision, TruePeakError, TruePeakSource, default_policy, resolve_decision,
    resolve_full_scan,
};

/// What:
///      `pub use truepeak_core::true_peak_interleaved;`.
///  Re-export the shared crate's
///           whole-buffer meter helper under this module's path.
/// Why:
///       The test-only `nativeTruePeakSynthetic` JNI entry in `lib.rs` calls
///           `truepeak::true_peak_interleaved(...)` to measure a synthetic signal on device
///           through the exact shared meter,
///  with no Android-local copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { truePeakInterleaved } from "truepeak-core";
/// ```
pub use truepeak_core::true_peak_interleaved;

/// What:
///      `struct AndroidSource { inner: Box<dyn Source> }`.
///  A newtype wrapping the Android
///           decoder so it satisfies the shared [`TruePeakSource`] contract.
/// Why:
///       The shared resolver drives any `TruePeakSource`;
///  this adapts the Android decoder
///           to it,
///  bridging the seconds-based `seek` to the frame-based `seek_to_frame` and
///           mapping the Android `PlayerError` to the crate's `TruePeakError`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class AndroidSource implements TruePeakSource { constructor(private inner: Source) {} }
/// ```
struct AndroidSource {
    /// What:
    ///      `inner: Box<dyn Source>`.
    ///  The owned Android decoder.
    /// Why:
    ///       The adapter forwards every trait method to it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private inner: Source;
    /// ```
    inner: Box<dyn Source>,
}

/// What:
///      `impl TruePeakSource for AndroidSource { ... }`.
///  Implement the shared contract by
///           forwarding to the wrapped decoder.
/// Why:
///       Let the shared resolvers measure Android audio without knowing the decoder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // AndroidSource satisfies TruePeakSource
/// ```
impl TruePeakSource for AndroidSource {
    /// What:
    ///      `fn spec(&self) -> AudioSpec`.
    ///  Map the Android `decode::AudioSpec` to the
    ///           shared `AudioSpec` (identical fields,
    ///  different crate).
    /// Why:
    ///       The resolver reads rate,
    ///  channels,
    ///  and duration through this.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// spec() { const s = this.inner.spec(); return { rate: s.rate, channels: s.channels, durationSecs: s.durationSecs }; }
    /// ```
    fn spec(&self) -> AudioSpec {
        // What:     `let spec = self.inner.spec();`. The Android descriptor (a COPY).
        // Why:      Read its three fields into the shared shape.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const spec = this.inner.spec();
        // ```
        let spec = self.inner.spec();
        // What:     `AudioSpec { rate, channels, duration_secs }`. The shared descriptor.
        //           Tail -> return.
        // Why:      Same values, in the crate's type.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { rate: spec.rate, channels: spec.channels, durationSecs: spec.durationSecs };
        // ```
        return AudioSpec {
            rate: spec.rate,
            channels: spec.channels,
            duration_secs: spec.duration_secs,
        }
    }

    /// What:
    ///      `fn next_chunk(&mut self) -> Result<Vec<f32>, TruePeakError>`.
    ///  Forward the
    ///           next decoded block,
    ///  mapping a decode error to `TruePeakError::Decode`.
    /// Why:
    ///       The meter feeds these blocks;
    ///  the error type must be the crate's.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// nextChunk() { try { return this.inner.nextChunk(); } catch (e) { throw decodeError(e); } }
    /// ```
    fn next_chunk(&mut self) -> Result<Vec<f32>, TruePeakError> {
        // What:     `self.inner.next_chunk().map_err(|error| TruePeakError::Decode { ... })`.
        //           Forward; `.map_err` rewraps a `PlayerError` as a crate decode error.
        //           Tail -> return.
        // Why:      Keep the decoder's message while crossing the crate boundary.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return mapErr(this.inner.nextChunk(), (e) => ({ kind: "decode", message: String(e) }));
        // ```
        return self.inner
            .next_chunk()
            .map_err(|error| return TruePeakError::Decode { message: error.to_string() })
    }

    /// What:
    ///      `fn seek_to_frame(&mut self, frame: u64) -> Result<(), TruePeakError>`.
    ///  Seek
    ///           the Android decoder to the interleaved frame by converting it to seconds
    ///           (`frame / rate`),
    ///  mapping a seek error to `TruePeakError::Seek`.
    /// Why:
    ///       The probe places windows by frame;
    ///  the Android decoder seeks by seconds.
    /// Gotcha:
    ///    The seconds-granular seek lands at the nearest packet boundary,
    ///  not the exact
    ///           frame,
    ///  so probe windows are placed approximately.
    ///  That is acceptable for the
    ///           runtime:
    ///  the probe takes the loudest of several spread windows,
    ///  and a few
    ///           milliseconds of drift does not change which window is loudest.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// seekToFrame(frame) { try { this.inner.seek(frame / this.inner.spec().rate); } catch (e) { throw seekError(e); } }
    /// ```
    fn seek_to_frame(&mut self, frame: u64) -> Result<(), TruePeakError> {
        // What:     `let rate = self.inner.spec().rate;`. Samples per second per channel.
        // Why:      Convert the frame index to a time offset.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const rate = this.inner.spec().rate;
        // ```
        let rate = self.inner.spec().rate;
        // What:     `let seconds = frame as f64 / f64::from(rate);`. Frame index over rate.
        // Why:      The Android `seek` takes seconds.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const seconds = frame / rate;
        // ```
        let seconds = frame as f64 / f64::from(rate);
        // What:     `self.inner.seek(seconds).map_err(|error| TruePeakError::Seek { ... })`.
        //           Seek; rewrap a `PlayerError` as a crate seek error. Tail -> return.
        // Why:      Keep the decoder's message while crossing the crate boundary.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return mapErr(this.inner.seek(seconds), (e) => ({ kind: "seek", message: String(e) }));
        // ```
        return self.inner
            .seek(seconds)
            .map_err(|error| return TruePeakError::Seek { message: error.to_string() })
    }
}

/// What:
///      `pub fn resolve_current(source: Box<dyn Source>) -> Result<Decision,
///           TruePeakError>`.
///  Resolve the foreground gain decision under the shipped policy:
///           full-scan a short track,
///  probe a long one.
///  Takes an already-opened decoder.
/// Why:
///       The current-track path wants a usable gain quickly;
///  the probe yields one for a
///           long track without decoding the whole file.
/// Gotcha:
///    This BLOCKS on decode;
///  the JNI calls it off the caller's UI path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveCurrent(source: Source): Decision { return resolveDecision(defaultPolicy(), new AndroidSource(source)); }
/// ```
pub fn resolve_current(source: Box<dyn Source>) -> Result<Decision, TruePeakError> {
    // What:     `let mut adapter = AndroidSource { inner: source };`. Wrap the decoder.
    // Why:      The resolver needs a `&mut dyn TruePeakSource`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const adapter = new AndroidSource(source);
    // ```
    let mut adapter = AndroidSource { inner: source };
    // What:     `resolve_decision(&default_policy(), &mut adapter)`. Drive the source through
    //           the probe-or-full policy. Tail -> return.
    // Why:      The one shared foreground measurement.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return resolveDecision(defaultPolicy(), adapter);
    // ```
    return resolve_decision(&default_policy(), &mut adapter)
}

/// What:
///      `pub fn resolve_full(source: Box<dyn Source>) -> Result<Decision, TruePeakError>`.
///           Resolve an EXACT gain decision by full-scanning the whole track,
///  regardless of
///           length.
///  Takes an already-opened decoder.
/// Why:
///       Warming upgrades a probe estimate to an exact cached gain over idle time;
///  the
///           cache's exact-over-probe precedence then keeps the exact decision.
/// Gotcha:
///    This BLOCKS on a full decode;
///  warming runs it at low thread priority.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveFull(source: Source): Decision { return resolveFullScan(defaultPolicy(), new AndroidSource(source)); }
/// ```
pub fn resolve_full(source: Box<dyn Source>) -> Result<Decision, TruePeakError> {
    // What:     `let mut adapter = AndroidSource { inner: source };`. Wrap the decoder.
    // Why:      The resolver needs a `&mut dyn TruePeakSource`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const adapter = new AndroidSource(source);
    // ```
    let mut adapter = AndroidSource { inner: source };
    // What:     `resolve_full_scan(&default_policy(), &mut adapter)`. Always exact. Tail ->
    //           return.
    // Why:      The shared warming-upgrade measurement.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return resolveFullScan(defaultPolicy(), adapter);
    // ```
    return resolve_full_scan(&default_policy(), &mut adapter)
}
