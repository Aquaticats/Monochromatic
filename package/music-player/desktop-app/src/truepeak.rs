//! Desktop true-peak resolution:
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
//!  This module keeps only the
//! desktop-specific glue:
//!  a [`TruePeakSource`] adapter over the desktop `decode::Source`,
//!  and
//! two openers that hand that adapter to the shared resolvers.
//!  Foreground playback resolves a
//! probe-or-full decision quickly ([`resolve_current`]);
//!  background warming upgrades to an
//! exact full scan ([`resolve_full`]).
//!  See
//! ../../../doc/handover/music-player-truepeak-core-integration.md.

/// What:
///      `use std::path::Path;`.
///  Borrowed filesystem-path type (sibling:
///  the owned
///           `PathBuf`,
///  like `&str` vs `String`).
/// Why:
///       The openers only read the path,
///  so they borrow it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a path is just a string in TS
/// ```
use std::path::Path;

/// What:
///      `use crate::decode;`.
///  The decode module,
///  for `decode::open`.
///  The `Source` trait
///           is NOT imported:
///  its `spec`/`next_chunk`/`seek` methods are callable on the
///           `Box<dyn Source>` value through the trait object itself,
///  with no import.
/// Why:
///       Measurement decodes through the same path playback uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as decode from "./decode";
/// ```
use crate::decode;

/// What:
///      `use truepeak_core::{AudioSpec, Decision, TruePeakError, TruePeakSource,
///           default_policy, resolve_decision, resolve_full_scan};`.
///  The shared source
///           contract and its descriptor,
///  the decision type,
///  the crate error,
///  the shipped
///           policy,
///  and the two resolvers.
/// Why:
///       The adapter implements `TruePeakSource` (returning an `AudioSpec`,
///  mapping decode
///           errors into `TruePeakError`);
///  the openers call the resolvers under
///           `default_policy` and return a `Decision`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { AudioSpec, Decision, TruePeakError, TruePeakSource, defaultPolicy, resolveDecision, resolveFullScan } from "truepeak-core";
/// ```
use truepeak_core::{
    AudioSpec, Decision, TruePeakError, TruePeakSource, default_policy, probe_inputs_from_file,
    resolve_decision_for, resolve_full_scan,
};

/// What:
///      `pub use truepeak_core::normalization_gain;`.
///  Re-export the shared attenuate-only
///           gain function under this module's path.
/// Why:
///       `peak_swap` computes the cold-start fallback gain (`normalization_gain(1.0)`,
///  the
///           -1 dBTP ceiling) from here,
///  so the one shared implementation stays authoritative.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { normalizationGain } from "truepeak-core";
/// ```
pub use truepeak_core::normalization_gain;

/// What:
///      `struct DesktopSource { inner: Box<dyn decode::Source> }`.
///  A newtype wrapping the
///           desktop decoder so it satisfies the shared [`TruePeakSource`] contract.
///  `Box<dyn
///           decode::Source>` is an owned,
///  heap-allocated trait object (the concrete decoder,
///           Symphonia or Opus,
///  erased).
/// Why:
///       The shared resolver drives any `TruePeakSource`;
///  this adapts the desktop decoder
///           to it,
///  bridging the seconds-based `seek` to the frame-based `seek_to_frame` and
///           mapping the desktop `PlayerError` to the crate's `TruePeakError`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class DesktopSource implements TruePeakSource { constructor(private inner: Source) {} }
/// ```
struct DesktopSource {
    /// What:
    ///      `inner: Box<dyn decode::Source>`.
    ///  The owned desktop decoder.
    /// Why:
    ///       The adapter forwards every trait method to it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private inner: Source;
    /// ```
    inner: Box<dyn decode::Source>,
}

/// What:
///      `impl TruePeakSource for DesktopSource { ... }`.
///  Implement the shared contract by
///           forwarding to the wrapped decoder.
/// Why:
///       Let the shared resolvers measure desktop audio without knowing the decoder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // DesktopSource satisfies TruePeakSource
/// ```
impl TruePeakSource for DesktopSource {
    /// What:
    ///      `fn spec(&self) -> AudioSpec`.
    ///  Map the desktop `decode::AudioSpec` to the
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
        // What:     `let spec = self.inner.spec();`. The desktop descriptor (a COPY).
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
    ///           the desktop decoder to the interleaved frame by converting it to seconds
    ///           (`frame / rate`),
    ///  mapping a seek error to `TruePeakError::Seek`.
    /// Why:
    ///       The probe places windows by frame;
    ///  the desktop decoder seeks by seconds.
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
    ///  The bench
    ///           sidecar (which needs exact placement) uses its own frame-exact source.
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
        // Why:      The desktop `seek` takes seconds.
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
///      `fn open_adapter(path: &Path) -> Result<DesktopSource, TruePeakError>`.
///  Open the
///           desktop decoder for `path` and wrap it in the shared-source adapter,
///  mapping an
///           open failure to `TruePeakError::Decode`.
///  Module-private.
/// Why:
///       Both openers start the same way;
///  one helper keeps the open-and-wrap in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function openAdapter(path) { return new DesktopSource(decode.open(path)); }
/// ```
fn open_adapter(path: &Path) -> Result<DesktopSource, TruePeakError> {
    // What:     `let inner = decode::open(path).map_err(|error| TruePeakError::Decode { ... })?;`.
    //           Open the decoder; `?` propagates a rewrapped open error.
    // Why:      The adapter needs an open decoder to drive.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const inner = decode.open(path);
    // ```
    let inner =
        decode::open(path).map_err(|error| return TruePeakError::Decode { message: error.to_string() })?;
    // What:     `Ok(DesktopSource { inner })`. Wrap it. Tail -> return.
    // Why:      Hand back the adapter the resolver drives.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return new DesktopSource(inner);
    // ```
    return Ok(DesktopSource { inner })
}

/// What:
///      `pub(crate) fn resolve_current(path: &Path) -> Result<Decision, TruePeakError>`.
///           Resolve the foreground gain decision under the shipped policy:
///  full-scan a short
///           track,
///  probe a long one.
///  `pub(crate)` so `peak_swap` calls it.
/// Why:
///       The current-track path wants a usable gain quickly;
///  the probe yields one for a
///           long track without decoding the whole file.
/// Gotcha:
///    This BLOCKS on decode;
///  call it on a worker thread,
///  never on the audio callback.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveCurrent(path: string): Decision { return resolveDecision(defaultPolicy(), openAdapter(path)); }
/// ```
pub(crate) fn resolve_current(path: &Path) -> Result<Decision, TruePeakError> {
    // What:     `let policy = default_policy();` then `let (provenance, bones) =
    //           probe_inputs_from_file(path, &policy);`. The shipped policy and the
    //           file-derived bucket inputs (FLAC sniff -> lossless + bones seeds; any
    //           failure degrades to the bare bucket by the library's contract).
    // Why:      The bucket table only pays off when the resolver knows the provenance;
    //           the library owns that file-format knowledge.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const policy = defaultPolicy(); const [provenance, bones] = probeInputsFromFile(path, policy);
    // ```
    let policy = default_policy();
    let (provenance, bones) = probe_inputs_from_file(path, &policy);
    // What:     `let mut source = open_adapter(path)?;`. Open and wrap the decoder.
    // Why:      The resolver needs a `&mut dyn TruePeakSource`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const source = openAdapter(path);
    // ```
    let mut source = open_adapter(path)?;
    // What:     `resolve_decision_for(&policy, &mut source, provenance, bones.as_deref())`.
    //           Drive the source through the bucket-zoom policy. Tail -> return.
    // Why:      The one shared foreground measurement, under the track's bucket.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return resolveDecisionFor(policy, source, provenance, bones);
    // ```
    return resolve_decision_for(&policy, &mut source, provenance, bones.as_deref())
}

/// What:
///      `pub(crate) fn resolve_full(path: &Path) -> Result<Decision, TruePeakError>`.
///           Resolve an EXACT gain decision by full-scanning the whole track,
///  regardless of
///           length.
///  `pub(crate)` so `measure` (background warming) calls it.
/// Why:
///       Warming upgrades a probe estimate to an exact cached gain over idle time;
///  the
///           cache's exact-over-probe precedence then keeps the exact decision.
/// Gotcha:
///    This BLOCKS on a full decode;
///  warming runs it at idle scheduling priority.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveFull(path: string): Decision { return resolveFullScan(defaultPolicy(), openAdapter(path)); }
/// ```
pub(crate) fn resolve_full(path: &Path) -> Result<Decision, TruePeakError> {
    // What:     `let mut source = open_adapter(path)?;`. Open and wrap the decoder.
    // Why:      The resolver needs a `&mut dyn TruePeakSource`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const source = openAdapter(path);
    // ```
    let mut source = open_adapter(path)?;
    // What:     `resolve_full_scan(&default_policy(), &mut source)`. Always exact. Tail ->
    //           return.
    // Why:      The shared warming-upgrade measurement.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return resolveFullScan(defaultPolicy(), source);
    // ```
    return resolve_full_scan(&default_policy(), &mut source)
}

/// What:
///      `#[cfg(test)] #[path = "truepeak_tests.rs"] mod tests;` declares a test-only
///           submodule whose code lives in the sibling file `truepeak_tests.rs`.
/// Why:
///       Keep `truepeak.rs` to production code;
///  the test lives beside it without inflating
///           this file or its max-lines budget (sibling `*_tests.rs` files are exempt from the
///           linter).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // truepeak.unit.test.ts, run only by the test runner
/// ```
#[cfg(test)]
#[path = "truepeak_tests.rs"]
mod tests;
