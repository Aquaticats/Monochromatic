//! Resolve a track to a gain decision by driving a decoded source through the policy.
//!
//! This is the shared measurement the platforms call.
//!  A short track (or one of unknown
//! length) is scanned in full for an exact peak.
//!  A long track is probed by the frontier
//! zoom under its provenance bucket's coverage:
//!  an even pass of tenth-second bins,
//!  then
//! repeated measurement beside the loudest bin heard so far (each bin has its own meter
//! so a seek seam cannot fabricate a spike),
//!  optionally seeded by lossless frame-size
//! bones;
//!  the loudest sampled bin is inflated by the bucket's margin.

/// What:
///      `use crate::decision::{Decision, DecisionKind};`.
///  The answer type and its tag.
/// Why:
///       This module builds and returns a `Decision`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Decision, DecisionKind } from "./decision";
/// ```
use crate::decision::{Decision, DecisionKind};

/// What:
///      `use crate::error::TruePeakError;`.
///  The fallible-source error.
/// Why:
///       Driving the source propagates decode and seek errors with `?`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { TruePeakError } from "./error";
/// ```
use crate::error::TruePeakError;

/// What:
///      `use crate::gain::{normalization_gain, probe_estimated_peak};`.
///  The gain from
///           a peak,
///  and the margin inflation.
/// Why:
///       Both branches turn a peak into a gain;
///  the probe branch inflates first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { normalizationGain, probeEstimatedPeak } from "./gain";
/// ```
use crate::gain::{normalization_gain, probe_estimated_peak};

/// What:
///      `use crate::meter::TruePeakMeter;`.
///  The shared streaming meter.
/// Why:
///       Every scan and every window is measured by it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { TruePeakMeter } from "./meter";
/// ```
use crate::meter::TruePeakMeter;

/// What:
///      `use crate::policy::Policy;`.
///  The shipped policy parameters.
/// Why:
///       The short-scan cutoff,
///  coverage,
///  window length,
///  and margin come from it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Policy } from "./policy";
/// ```
use crate::policy::Policy;

/// What:
///      `use crate::source::{AudioSpec, TruePeakSource};`.
///  The stream descriptor and
///           the decoded-audio contract.
/// Why:
///       The resolver reads spec,
///  chunks,
///  and seeks through it;
///  the exact-decision
///           helper takes an `AudioSpec` by value (it is `Copy`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { AudioSpec, TruePeakSource } from "./source";
/// ```
use crate::source::{AudioSpec, TruePeakSource};

/// What:
///      `use crate::bucketpolicy::TrackProvenance;`.
///  The zero-cost provenance
///           signals that pick a long track's bucket.
/// Why:
///       `resolve_decision_for` selects coverage and margin from them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { TrackProvenance } from "./bucketpolicy";
/// ```
use crate::bucketpolicy::TrackProvenance;

/// What:
///      `use crate::probe::{ZoomPlan, zoom_probe};`.
///  The frontier-zoom probe and
///           its per-track plan.
/// Why:
///       Long tracks are probed by the zoom,
///  not by static even placement.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ZoomPlan, zoomProbe } from "./probe";
/// ```
use crate::probe::{ZoomPlan, zoom_probe};

/// What:
///      `fn full_scan(source: &mut dyn TruePeakSource, channels: usize) -> Result<(f32,
///           u64), TruePeakError>`.
///  Scan the whole source,
///  returning the true peak and the
///           decoded frame count.
///  `&mut dyn TruePeakSource` is a mutable borrow of the trait
///           object.
/// Why:
///       Short and unknown-length tracks are measured exactly,
///  and the frame count gives
///           the decoded duration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fullScan(source: TruePeakSource, channels: number): [number, number] { ... }
/// ```
fn full_scan(source: &mut dyn TruePeakSource, channels: usize) -> Result<(f32, u64), TruePeakError> {
    // What:     `let mut meter = TruePeakMeter::new(channels);`. The accumulator.
    // Why:      One meter for the whole track.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const meter = new TruePeakMeter(channels);
    // ```
    let mut meter = TruePeakMeter::new(channels);
    // What:     `let mut samples: u64 = 0;`. Running count of interleaved samples fed.
    // Why:      Divided by channels at the end to report decoded frames.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let samples = 0;
    // ```
    let mut samples: u64 = 0;
    // What:     `loop { ... }`. Read and feed chunks until end of stream.
    // Why:      Cover every sample of the track.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // while (true) { ... }
    // ```
    loop {
        // What:     `let chunk = source.next_chunk()?;`. Next block; `?` propagates a
        //           decode error.
        // Why:      Feed it to the meter.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const chunk = source.nextChunk();
        // ```
        let chunk = source.next_chunk()?;
        // What:     `if chunk.is_empty() { break; }`. Empty signals EOF.
        // Why:      Stop at the end.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (chunk.length === 0) break;
        // ```
        if chunk.is_empty() {
            break;
        }
        // What:     `samples += chunk.len() as u64;` then `meter.feed(&chunk);`. Count and
        //           feed. `&chunk` lends the block read-only.
        // Why:      Track length and update the peak.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // samples += chunk.length; meter.feed(chunk);
        // ```
        samples += chunk.len() as u64;
        meter.feed(&chunk);
    }
    // What:     `Ok((meter.peak(), samples / channels as u64))`. The peak and the frame
    //           count (samples over channels). Tail -> return.
    // Why:      Both are needed to build the decision.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return [meter.peak(), Math.floor(samples / channels)];
    // ```
    return Ok((meter.peak(), samples / channels as u64))
}

/// What:
///      `fn silence_decision(spec: AudioSpec) -> Decision`.
///  The decision for a
///           malformed zero-channel stream:
///  unity gain,
///  exact,
///  zero peak.
///  `AudioSpec` is
///           taken by value because it is `Copy`.
/// Why:
///       Both entry points (`resolve_decision` and `resolve_full_scan`) treat a
///           zero-channel stream as silence identically;
///  one helper keeps them in step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function silenceDecision(spec: AudioSpec): Decision {
///   return { gain: 1, kind: "fullScanExact", measuredPeak: 0, durationSecs: spec.durationSecs };
/// }
/// ```
fn silence_decision(spec: AudioSpec) -> Decision {
    // What:     `Decision { gain: 1.0, kind: FullScanExact, measured_peak: 0.0, duration_secs:
    //           spec.duration_secs }`. Unity gain over silence, tagged exact. Tail -> return.
    // Why:      Never amplify a degenerate stream, and never probe it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { gain: 1, kind: "fullScanExact", measuredPeak: 0, durationSecs: spec.durationSecs };
    // ```
    return Decision {
        gain: 1.0,
        kind: DecisionKind::FullScanExact,
        measured_peak: 0.0,
        duration_secs: spec.duration_secs,
    }
}

/// What:
///      `fn exact_decision(policy: &Policy, source: &mut dyn TruePeakSource, spec:
///           AudioSpec, channels: usize) -> Result<Decision, TruePeakError>`.
///  Scan the whole
///           track and build the exact decision,
///  tagging `ShortFullScan` for a known-short
///           track and `FullScanExact` otherwise.
///  `channels` is passed in so the caller's
///           zero-channel guard is not repeated here.
/// Why:
///       The short branch of `resolve_decision` and the always-exact `resolve_full_scan`
///           build the identical exact decision;
///  sharing it keeps the gain and the kind rule
///           in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function exactDecision(policy, source, spec, channels): Decision { ... }
/// ```
fn exact_decision(
    policy: &Policy,
    source: &mut dyn TruePeakSource,
    spec: AudioSpec,
    channels: usize,
) -> Result<Decision, TruePeakError> {
    // What:     `let (peak, frames) = full_scan(source, channels)?;`. Exact peak and decoded
    //           frame count; `?` propagates a decode error.
    // Why:      The gain is exact, and the frames give the decoded duration.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const [peak, frames] = fullScan(source, channels);
    // ```
    let (peak, frames) = full_scan(source, channels)?;
    // What:     `let short = spec.duration_known() && spec.duration_secs <=
    //           policy.short_scan_max_secs;`. Whether this counts as a short track.
    // Why:      Tags the decision kind; an unknown-length or long full scan is `FullScanExact`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const short = durationKnown(spec) && spec.durationSecs <= policy.shortScanMaxSecs;
    // ```
    let short = spec.duration_known() && spec.duration_secs <= policy.short_scan_max_secs;
    // What:     `Ok(Decision { ... })`. Build the exact decision; `normalization_gain` turns
    //           the peak into an attenuate-only gain. Tail -> return.
    // Why:      Hand back the exact gain and its evidence.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { gain: normalizationGain(peak), kind: short ? "shortFullScan" : "fullScanExact", measuredPeak: peak, durationSecs: frames / spec.rate };
    // ```
    return Ok(Decision {
        gain: normalization_gain(peak),
        kind: if short { DecisionKind::ShortFullScan } else { DecisionKind::FullScanExact },
        measured_peak: peak,
        duration_secs: frames as f64 / f64::from(spec.rate),
    })
}

/// What:
///      `pub fn resolve_full_scan(policy: &Policy, source: &mut dyn TruePeakSource) ->
///           Result<Decision, TruePeakError>`.
///  Always scan the whole track for an exact gain,
///           regardless of length,
///  tagging `ShortFullScan` or `FullScanExact` by duration.
/// Why:
///       The warming upgrade path:
///  a long track that `resolve_decision` would probe is
///           heard in full here,
///  so the cache's exact-over-probe precedence can replace a
///           probe estimate with the exact gain over idle time.
///  Desktop warming uses this;
///           the probe estimate never returns once an exact row lands.
/// Gotcha:
///    This performs a BLOCKING full decode of the whole track;
///  call it on a worker,
///           never on a latency-sensitive path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveFullScan(policy: Policy, source: TruePeakSource): Decision { ... }
/// ```
pub fn resolve_full_scan(
    policy: &Policy,
    source: &mut dyn TruePeakSource,
) -> Result<Decision, TruePeakError> {
    // What:     `let spec = source.spec();`. Rate, channels, and duration of the stream.
    // Why:      Channels size the meter; duration tags the kind.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const spec = source.spec();
    // ```
    let spec = source.spec();
    // What:     `let channels = spec.channels as usize;`. Interleave width for the meter.
    // Why:      The meter and the frame math need it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const channels = spec.channels;
    // ```
    let channels = spec.channels as usize;
    // What:     `if channels == 0 { return Ok(silence_decision(spec)); }`. Degenerate stream.
    // Why:      Avoid a divide-by-zero and never amplify silence.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (channels === 0) return silenceDecision(spec);
    // ```
    if channels == 0 {
        return Ok(silence_decision(spec));
    }
    // What:     `exact_decision(policy, source, spec, channels)`. The full exact scan.
    //           Tail -> return.
    // Why:      Always exact here, unlike `resolve_decision`'s long-track probe.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return exactDecision(policy, source, spec, channels);
    // ```
    return exact_decision(policy, source, spec, channels)
}

/// What:
///      `pub fn resolve_decision(policy: &Policy, source: &mut dyn TruePeakSource) ->
///           Result<Decision, TruePeakError>`.
///  Turn a track into a gain decision under the
///           policy.
///  `&Policy` borrows the parameters;
///  `&mut dyn TruePeakSource` drives the
///           decoder.
/// Why:
///       This is the one shared measurement both platforms call to get a track's gain.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveDecision(policy: Policy, source: TruePeakSource): Decision { ... }
/// ```
pub fn resolve_decision(
    policy: &Policy,
    source: &mut dyn TruePeakSource,
) -> Result<Decision, TruePeakError> {
    // What:     `resolve_decision_for(policy, source, TrackProvenance::unknown(), None)`.
    //           Delegate with the uninformed provenance and no bones. Tail -> return.
    // Why:      Uninformed callers land in the bare bucket, which has the deepest
    //           coverage, so they never under-probe; platforms that know provenance call
    //           `resolve_decision_for` directly.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return resolveDecisionFor(policy, source, TrackProvenance.unknown(), null);
    // ```
    return resolve_decision_for(policy, source, TrackProvenance::unknown(), None)
}

/// What:
///      `pub fn resolve_decision_for(policy: &Policy, source: &mut dyn
///           TruePeakSource, provenance: TrackProvenance, bones_hot_bins:
///           Option<&[usize]>) -> Result<Decision, TruePeakError>`.
///  Turn a track into a
///           gain decision under the policy,
///  with the track's provenance picking its
///           bucket and optional bones seeds guiding the probe.
/// Why:
///       The bucket table is the allocation layer of the shipped policy;
///  provenance
///           and bones are zero-cost inputs the platform reads from tags and framing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveDecisionFor(policy, source, provenance, bonesHotBins): Decision { ... }
/// ```
pub fn resolve_decision_for(
    policy: &Policy,
    source: &mut dyn TruePeakSource,
    provenance: TrackProvenance,
    bones_hot_bins: Option<&[usize]>,
) -> Result<Decision, TruePeakError> {
    // What:     `let span = tracing::debug_span!("resolve_decision_for"); let _guard =
    //           span.enter();`. Open a function-scoped span; `enter()` returns a guard that
    //           tags every event below with this function name and drops on every return
    //           path, closing the span. The module path is the outer tag.
    // Why:      Mirrors the TS tagged logger's per-function tag without a proc-macro; the
    //           span also correlates one track's events when warming resolves interleave.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const rl = tagged({ tag: "resolveDecisionFor", l });
    // ```
    let span = tracing::debug_span!("resolve_decision_for");
    let _guard = span.enter();
    // What:     `let spec = source.spec();` then `let channels = spec.channels as usize;`.
    //           Rate, channels, and duration of the stream.
    // Why:      They pick the branch and size the probe bins.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const spec = source.spec(); const channels = spec.channels;
    // ```
    let spec = source.spec();
    let channels = spec.channels as usize;
    // What:     `if channels == 0 { return Ok(silence_decision(spec)); }`. A malformed
    //           zero-channel stream is treated as silence: unity gain, exact.
    // Why:      Avoid a divide-by-zero and never amplify silence.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (channels === 0) return silenceDecision(spec);
    // ```
    if channels == 0 {
        // Degenerate stream: record the silence branch before returning.
        tracing::debug!("zero channels; silence decision");
        return Ok(silence_decision(spec));
    }
    // What:     `if !spec.duration_known() || spec.duration_secs <= policy.short_scan_max_secs`.
    //           Full-scan when the length is unknown or short.
    // Why:      Short and unknown tracks are cheap to measure exactly, so they carry no
    //           probe error and need no bucket.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!durationKnown(spec) || spec.durationSecs <= policy.shortScanMaxSecs) return exactDecision(policy, source, spec, channels);
    // ```
    if !spec.duration_known() || spec.duration_secs <= policy.short_scan_max_secs {
        // The full-scan branch: log the deciding length values.
        tracing::debug!(
            duration_known = spec.duration_known(),
            duration_secs = spec.duration_secs,
            short_scan_max_secs = policy.short_scan_max_secs,
            "full-scan branch"
        );
        return exact_decision(policy, source, spec, channels);
    }
    // What:     `let bucket = provenance.select(&policy.buckets, bones_hot_bins.is_some());`.
    //           The bucket's coverage and margin for this track.
    // Why:      Lossless probes cheaply (cheaper still with bones); untagged lossy gets
    //           the deepest coverage.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const bucket = provenance.select(policy.buckets, bonesHotBins !== null);
    // ```
    let bucket = provenance.select(&policy.buckets, bones_hot_bins.is_some());
    // What:     `let plan = ZoomPlan { ... };`. The bin grid and coverage for the probe:
    //           tenth-second bins over the whole track, the bucket's total coverage, and
    //           the bones-aware even pass share.
    // Why:      One value carries everything the zoom needs.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const plan = { binFrames, totalFrames, coverage: bucket.coverageFraction, evenCoverage, bonesHotBins };
    // ```
    let plan = ZoomPlan {
        bin_frames: ((policy.probe_window_secs * f64::from(spec.rate)).round() as u64).max(1),
        total_frames: (spec.duration_secs * f64::from(spec.rate)).round() as u64,
        coverage_fraction: bucket.coverage_fraction,
        even_coverage_fraction: if bones_hot_bins.is_some() {
            policy.bones_even_coverage_fraction
        } else {
            policy.pass1_coverage_fraction.min(bucket.coverage_fraction)
        },
        bones_hot_bins,
    };
    // The probe branch: log the bucket coverage and whether bones seeded the plan.
    tracing::debug!(
        lossless = provenance.lossless,
        bones = bones_hot_bins.is_some(),
        coverage_fraction = plan.coverage_fraction,
        margin_db = bucket.probe_margin_db,
        "probe branch"
    );
    // What:     `let sampled_max = zoom_probe(source, channels, &plan)?;`. The frontier
    //           zoom's loudest measured window; `?` propagates decode and seek errors.
    // Why:      The climb collapses the shoulder misses an even probe leaves behind.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const sampledMax = zoomProbe(source, channels, plan);
    // ```
    let sampled_max = zoom_probe(source, channels, &plan)?;
    // What:     `let estimated = probe_estimated_peak(f64::from(sampled_max),
    //           bucket.probe_margin_db) as f32;`. Inflate by the bucket's margin, then
    //           narrow back to `f32` for the gain.
    // Why:      The margin covers the crest the probe may still have missed; each bucket
    //           carries its own measured dial.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const estimated = probeEstimatedPeak(sampledMax, bucket.probeMarginDb);
    // ```
    let estimated = probe_estimated_peak(f64::from(sampled_max), bucket.probe_margin_db) as f32;
    // The probe outcome: the loudest sampled window, the margin-inflated estimate.
    tracing::debug!(sampled_max, estimated, "probe estimate");
    // What:     `Ok(Decision { ... })`. Build the probe decision; `normalization_gain` on
    //           the inflated peak. Tail -> return.
    // Why:      Hand back the probe estimate and its sampled evidence.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { gain: normalizationGain(estimated), kind: "probeEstimate", measuredPeak: sampledMax, durationSecs: spec.durationSecs };
    // ```
    return Ok(Decision {
        gain: normalization_gain(estimated),
        kind: DecisionKind::ProbeEstimate,
        measured_peak: sampled_max,
        duration_secs: spec.duration_secs,
    })
}

/// What:
///      `#[cfg(test)] #[path = "resolve_tests.rs"] mod tests;`.
///  Test-only submodule in
///           the sibling file,
///  gated to test builds.
/// Why:
///       Keep this file to production code;
///  sibling `*_tests.rs` is max-lines exempt.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // resolve.integration.test.ts
/// ```
#[cfg(test)]
#[path = "resolve_tests.rs"]
mod tests;
