//! Scoring a window policy against the corpus with the shared truepeak-core math.
//!
//! For one candidate `(window_count, window_seconds, probe_margin_db)` this simulates
//! the short/long split,
//!  the probe windows (placed by `truepeak_core::WindowPlacement`
//! and read from the per-second bins),
//!  the probe and exact gains (`truepeak_core` gain
//! math),
//!  the gain error against the bounds,
//!  and the decoded-seconds cost under the
//! plan's amended accounting (a probe-then-full track pays both the probe and the full
//! scan).
//!  A long track whose probe gain leaves the `+0.5 / -2.0 dB` bounds is a
//! violator the classifier must route to a full scan.

/// Imports the corpus track record.
use crate::corpus::Track;

/// Imports the shared gain,
///  dB,
///  window,
///  and policy math the score is built from.
use truepeak_core::{
    Policy, WindowPlacement, default_policy, peak_dbtp, probe_estimated_peak,
};

/// One policy under test:
///  the window count,
///  the seconds per window,
///  and the probe margin.
#[derive(Clone, Copy, Debug)]
pub struct Candidate {
    /// Number of probe windows on a long track.
    pub window_count: usize,
    /// Seconds of audio per probe window.
    pub window_seconds: f64,
    /// Safety margin in dB added to the sampled peak before deciding the probe gain.
    pub probe_margin_db: f64,
}

/// The aggregate score of one candidate over the whole corpus.
#[derive(Clone, Debug)]
pub struct Report {
    /// The candidate that produced this score.
    pub candidate: Candidate,
    /// Short/long threshold in seconds (`window_count * window_seconds`).
    pub threshold_secs: f64,
    /// Tracks short enough for an exact full scan.
    pub short_count: usize,
    /// Long tracks whose probe gain stayed within bounds.
    pub accepted_count: usize,
    /// Long tracks whose probe gain left the bounds (the classifier must catch these).
    pub violators: Vec<Violator>,
    /// Total decoded seconds under the amended accounting.
    pub decoded_secs: f64,
    /// Worst too-loud error in dB among accepted long tracks (at most `+0.5`).
    pub worst_too_loud_db: f64,
    /// Worst too-quiet error in dB among accepted long tracks (at least `-2.0`).
    pub worst_too_quiet_db: f64,
}

/// A long track whose probe gain would leave the bounds,
///  plus its probe features.
#[derive(Clone, Debug)]
pub struct Violator {
    /// Track path,
    ///  for the reviewable exception list (never a classifier input).
    pub path: String,
    /// Gain error in dB the probe would have shipped (signed:
    ///  positive is too loud).
    pub error_db: f64,
    /// Sampled maximum peak the probe saw (linear),
    ///  a legal classifier feature.
    pub sampled_max: f64,
    /// Track duration in seconds,
    ///  a legal classifier feature.
    pub duration_secs: f64,
}

/// Compute the exact gain in dB for a full-track peak,
///  clamped to never amplify.
///
/// What:
///  `min(0, -1 - 20*log10(peak))`.
///  Why:
///  this is the truth the probe gain is
/// scored against;
///  using `truepeak_core::peak_dbtp` keeps it identical to production.
fn exact_gain_db(full_peak: f64, ceiling_dbtp: f64) -> f64 {
    return (ceiling_dbtp - peak_dbtp(full_peak)).min(0.0)
}

/// Compute the probe gain in dB from a sampled peak and the margin,
///  clamped to unity.
///
/// What:
///  inflate the sampled peak by the margin,
///  then `min(0, -1 - dBTP)`.
///  Why:
///  the
/// shared `probe_estimated_peak` is the production inflation step.
fn probe_gain_db(sampled_max: f64, margin_db: f64, ceiling_dbtp: f64) -> f64 {
    let estimated = probe_estimated_peak(sampled_max, margin_db);
    return (ceiling_dbtp - peak_dbtp(estimated)).min(0.0)
}

/// The per-window sampled peaks (linear) of one long track's placed probe windows.
///
/// What:
///  place `window_count` windows by frame with `truepeak_core::WindowPlacement`,
/// map each to the seconds it covers,
///  and take the max of the per-second bins it spans.
/// Why:
///  the bins are exact per-second meter peaks,
///  so this reproduces the window probe
/// without decoding,
///  using the shared window placement;
///  the classifier reads the spread
/// across these windows,
///  so the whole vector is returned,
///  not just its max.
pub fn sampled_windows(track: &Track, candidate: Candidate) -> Vec<f64> {
    let placement = WindowPlacement::plan(
        track.decoded_frames,
        candidate.window_count,
        candidate.window_seconds,
        track.rate,
    );
    let bins = &track.bin_peaks;
    let rate = f64::from(track.rate);
    // Each window covers the bins overlapping its frame span; take the loudest bin.
    return placement
        .starts
        .iter()
        .map(|&start| {
            let begin_secs = start as f64 / rate;
            let end_secs = (start + placement.window_frames) as f64 / rate;
            let lo = begin_secs.floor() as usize;
            let hi = (end_secs.ceil() as usize).min(bins.len());
            let window_peak = bins
                .get(lo..hi)
                .unwrap_or(&[])
                .iter()
                .fold(0.0_f32, |peak, &bin| return peak.max(bin));
            return f64::from(window_peak)
        })
        .collect()
}

/// The largest sampled window peak (linear) of one long track.
///
/// What:
///  the max over `sampled_windows`.
///  Why:
///  the probe gain is decided from the loudest
/// window the probe saw.
fn sampled_max_peak(track: &Track, candidate: &Candidate) -> f64 {
    return sampled_windows(track, *candidate)
        .iter()
        .fold(0.0_f64, |peak, &window| return peak.max(window))
}

/// Score one candidate over every track in the corpus.
///
/// What:
///  classifies each track short/accepted/violator,
///  sums the amended decoded cost,
/// and tracks the worst accepted errors.
///  Why:
///  this is the single evaluation the search
/// and the final report both call.
pub fn evaluate(tracks: &[Track], candidate: Candidate) -> Report {
    let policy: Policy = default_policy();
    let threshold = candidate.window_count as f64 * candidate.window_seconds;

    // Per-track classification and cost folded into the running report state.
    let mut short_count = 0usize;
    let mut accepted_count = 0usize;
    let mut violators = Vec::new();
    let mut decoded_secs = 0.0f64;
    let mut worst_too_loud = 0.0f64;
    let mut worst_too_quiet = 0.0f64;

    for track in tracks {
        let duration = track.duration_secs;
        if duration <= threshold {
            // Short: exact full scan, no error, pays its own length.
            short_count += 1;
            decoded_secs += duration;
            continue;
        }

        let sampled = sampled_max_peak(track, &candidate);
        let probe = probe_gain_db(sampled, candidate.probe_margin_db, policy.ceiling_dbtp);
        let exact = exact_gain_db(f64::from(track.full_peak), policy.ceiling_dbtp);
        let error = probe - exact;

        let too_loud = error > policy.max_too_loud_db;
        let too_quiet = error < policy.max_too_quiet_db;
        if too_loud || too_quiet {
            // Violator: classifier routes it to a full scan, so it pays probe plus full.
            decoded_secs += threshold + duration;
            violators.push(Violator {
                path: track.path.clone(),
                error_db: error,
                sampled_max: sampled,
                duration_secs: duration,
            });
        } else {
            // Accepted probe: pays only the probe windows.
            accepted_count += 1;
            decoded_secs += threshold;
            worst_too_loud = worst_too_loud.max(error);
            worst_too_quiet = worst_too_quiet.min(error);
        }
    }

    return Report {
        candidate,
        threshold_secs: threshold,
        short_count,
        accepted_count,
        violators,
        decoded_secs,
        worst_too_loud_db: worst_too_loud,
        worst_too_quiet_db: worst_too_quiet,
    }
}
