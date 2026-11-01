//! The feasible no-classifier model.
//!
//! The corrected `/4` budget is far too tight for a full-scan classifier:
//!  it has room
//! for only a couple of extra full scans,
//!  while no probe or metadata feature separates
//! the ~120 hot-master violators that precisely.
//!  So the feasible policy ships NO
//! violator classifier at all.
//!  Instead it probes every long track and applies a single
//! fixed margin large enough to cover the worst under-read,
//!  which guarantees zero
//! violations.
//!  The objective then becomes:
//!  spend the budget on probe density (more,
//! shorter windows cover more distinct regions and shrink the gaps that cause
//! under-read) so the required margin,
//!  and thus the worst-case too-quiet error,
//!  is as
//! small as possible.
//!  This module sweeps density and reports the best feasible margin.

/// Imports the corpus track record.
use crate::corpus::Track;
/// Imports the candidate policy and the shared per-window sampling.
use crate::evaluate::{Candidate, sampled_windows};
/// Imports the safe-provenance path set.
use std::collections::HashSet;
/// Imports the shared dB and policy math.
use truepeak_core::{Policy, default_policy, peak_dbtp};

/// Convert a linear peak to dBTP,
///  treating silence as a very negative level.
fn db(peak: f64) -> f64 {
    if peak <= 0.0 {
        return f64::NEG_INFINITY
    } else {
        return peak_dbtp(peak)
    }
}

/// A density's feasible no-classifier outcome.
#[derive(Clone, Copy, Debug)]
pub struct Feasible {
    /// The probe density evaluated.
    pub candidate: Candidate,
    /// Fixed margin needed for zero violators (worst under-read minus the too-loud bound).
    pub margin_db: f64,
    /// Worst-case too-quiet error this margin causes (the negated margin).
    pub worst_quiet_db: f64,
    /// Decoded seconds:
    ///  short tracks full-scanned plus one probe per long track.
    pub probe_decoded_secs: f64,
    /// Whether the margin stays inside the `-2.0 dB` bound and the budget holds.
    pub feasible: bool,
}

/// Evaluate one density:
///  the fixed margin it needs and the decoded cost it pays.
///
/// What:
///  for the loud long tracks (true peak above the ceiling,
///  where gain applies),
/// find the worst under-read against the probe;
///  the margin must cover it.
///  Why:
///  a single
/// margin at or above the worst under-read minus the too-loud bound guarantees no track
/// exceeds the bounds,
///  with no full-scan classifier.
pub fn evaluate_density(
    policy: &Policy,
    tracks: &[Track],
    candidate: Candidate,
    target_secs: f64,
    quiet_bound_db: f64,
) -> Feasible {
    let threshold = candidate.window_count as f64 * candidate.window_seconds;

    // Short tracks pay their length; long tracks pay one probe; track the worst under-read.
    let mut decoded = 0.0f64;
    let mut max_under_read = 0.0f64;
    for track in tracks {
        if track.duration_secs <= threshold {
            decoded += track.duration_secs;
            continue;
        }
        decoded += threshold;
        let windows = sampled_windows(track, candidate);
        let sampled_max = windows.iter().fold(0.0_f64, |peak, &window| return peak.max(window));
        let full_db = db(f64::from(track.full_peak));
        if full_db > policy.ceiling_dbtp {
            max_under_read = max_under_read.max(full_db - db(sampled_max));
        }
    }

    let margin_db = (max_under_read - policy.max_too_loud_db).max(0.0);
    let worst_quiet_db = -margin_db;
    // Feasible when the margin stays inside the too-quiet bound and the probe cost fits.
    let feasible = worst_quiet_db >= quiet_bound_db && decoded <= target_secs;
    return Feasible {
        candidate,
        margin_db,
        worst_quiet_db,
        probe_decoded_secs: decoded,
        feasible,
    }
}

/// A provenance-dependent margin:
///  a smaller fixed margin for reliably-not-hot sources
/// (lossless and yt-dlp) and a larger one for the rest.
#[derive(Clone, Copy, Debug)]
pub struct ProvenanceMargin {
    /// Margin for safe-provenance tracks (lossless or yt-dlp).
    pub margin_safe_db: f64,
    /// Margin for the remaining (untagged lossy) tracks.
    pub margin_unsafe_db: f64,
    /// Worst-case too-quiet error,
    ///  the negated larger margin.
    pub worst_quiet_db: f64,
}

/// Compute the provenance-dependent margins for a probe density.
///
/// What:
///  split the loud long tracks by provenance and take each group's worst under-read;
/// each group's margin must cover its own worst gap-miss.
///  Why:
///  the safe group under-reads
/// less on average,
///  so a smaller margin keeps its tracks louder,
///  lowering the average
/// too-quiet error even though the worst case is still set by the louder margin.
pub fn provenance_margin(
    policy: &Policy,
    tracks: &[Track],
    candidate: Candidate,
    safe_paths: &HashSet<String>,
) -> ProvenanceMargin {
    let threshold = candidate.window_count as f64 * candidate.window_seconds;
    let mut max_under_read_safe = 0.0f64;
    let mut max_under_read_unsafe = 0.0f64;
    for track in tracks {
        if track.duration_secs <= threshold {
            continue;
        }
        let full_db = db(f64::from(track.full_peak));
        if full_db <= policy.ceiling_dbtp {
            continue;
        }
        let windows = sampled_windows(track, candidate);
        let sampled_max = windows.iter().fold(0.0_f64, |peak, &window| return peak.max(window));
        let under_read = full_db - db(sampled_max);
        if safe_paths.contains(&track.path) {
            max_under_read_safe = max_under_read_safe.max(under_read);
        } else {
            max_under_read_unsafe = max_under_read_unsafe.max(under_read);
        }
    }
    let margin_safe_db = (max_under_read_safe - policy.max_too_loud_db).max(0.0);
    let margin_unsafe_db = (max_under_read_unsafe - policy.max_too_loud_db).max(0.0);
    return ProvenanceMargin {
        margin_safe_db,
        margin_unsafe_db,
        worst_quiet_db: -margin_safe_db.max(margin_unsafe_db),
    }
}

/// Sweep probe densities and return the feasible one with the smallest margin.
///
/// What:
///  tries a grid of `(window_count, threshold)` and keeps the in-bound,
///  in-budget
/// density whose fixed margin (and thus worst-case too-quiet error) is smallest.
///  Why:
/// this is the real objective once the classifier is dropped:
///  minimize the worst-case
/// too-quiet error by spending the budget on probe coverage.
pub fn best_feasible(
    tracks: &[Track],
    counts: &[usize],
    thresholds: &[f64],
    target_secs: f64,
    quiet_bound_db: f64,
) -> Vec<Feasible> {
    let policy: Policy = default_policy();
    let mut results = Vec::new();
    for &window_count in counts {
        for &threshold in thresholds {
            let candidate = Candidate {
                window_count,
                window_seconds: threshold / window_count as f64,
                probe_margin_db: 0.0,
            };
            results.push(evaluate_density(&policy, tracks, candidate, target_secs, quiet_bound_db));
        }
    }
    // Best feasible first: smallest margin, then smallest decoded cost as a tie-break.
    results.sort_by(|left, right| {
        return right
            .feasible
            .cmp(&left.feasible)
            .then(left.margin_db.total_cmp(&right.margin_db))
            .then(left.probe_decoded_secs.total_cmp(&right.probe_decoded_secs))
    });
    return results
}

/// Tests for the feasible no-classifier model.
#[cfg(test)]
#[path = "feasible_tests.rs"]
mod tests;
