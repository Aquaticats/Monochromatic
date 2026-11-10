//! Tests for the per-candidate evaluation:
//!  the gain-error bounds,
//!  the decoded-seconds accounting,
//!  and the short/long/violator split.

use super::*;
use crate::test_fixtures::{bins_with_one_hot, flat_bins, levels_match, track};
use truepeak_core::default_policy;

/// Seconds in every long fixture track,
///  so each one costs its probe under any threshold the tests use.
const LONG_SECS: u64 = 100;

/// The loud-second index a probe window cannot cover:
///  the windows sit at the first and the last second.
const UNCOVERED_HOT_SEC: usize = 50;

/// A peak that only the uncovered second carries,
///  so the probe under-reads it by the whole level.
const HOT_PEAK: f32 = 1.0;

/// The quiet peak every other second carries.
const QUIET_PEAK: f32 = 0.01;

/// The two-window,
/// one-second probe:
///  windows over the first and the last second of a long track,
///  with no margin.
const TWO_WINDOWS: Candidate = Candidate {
    window_count: 2,
    window_seconds: 1.0,
    probe_margin_db: 0.0,
};

/// The long track whose only loud second sits between the probe windows.
fn needle_track() -> Track {
    return track(
        "needle.flac",
        LONG_SECS,
        HOT_PEAK,
        bins_with_one_hot(LONG_SECS as usize, QUIET_PEAK, UNCOVERED_HOT_SEC, HOT_PEAK),
    )
}

/// What:
///  a flat track's probe sees its true level,
///  so the probe gain equals the exact gain and the error is zero.
#[test]
fn a_flat_long_track_scores_zero_error_and_pays_its_probe() {
    let report = evaluate(&[track("flat.flac", LONG_SECS, 0.5, flat_bins(LONG_SECS as usize, 0.5))], TWO_WINDOWS);
    assert_eq!(report.short_count, 0, "a long track is never a short scan");
    assert_eq!(report.accepted_count, 1);
    assert!(report.violators.is_empty(), "a flat track has no violator");
    assert!(
        levels_match(report.decoded_secs, TWO_WINDOWS.window_count as f64 * TWO_WINDOWS.window_seconds),
        "an accepted long track pays only its probe, got {}",
        report.decoded_secs,
    );
    assert_eq!(report.worst_too_loud_db, 0.0, "no error is too loud");
    assert_eq!(report.worst_too_quiet_db, 0.0, "no error is too quiet");
}

/// What:
///  a peak the probe cannot see clamps the probe gain at unity,
///  so the error is the whole ceiling exactly,
/// and the track pays its probe plus a full scan.
#[test]
fn a_peak_between_the_windows_is_a_too_loud_violator() {
    let policy = default_policy();
    let report = evaluate(&[needle_track()], TWO_WINDOWS);
    assert_eq!(report.accepted_count, 0);
    assert_eq!(report.violators.len(), 1, "the needle must be a violator");
    let violator = &report.violators[0];
    assert!(
        levels_match(violator.error_db, -policy.ceiling_dbtp),
        "the error is the ceiling itself, got {}",
        violator.error_db,
    );
    assert!(violator.error_db > policy.max_too_loud_db, "it is too loud");
    assert_eq!(violator.path, "needle.flac");
    assert_eq!(violator.duration_secs, LONG_SECS as f64);
    assert!(levels_match(violator.sampled_max, f64::from(QUIET_PEAK)), "the probe saw the quiet second");
    let threshold = TWO_WINDOWS.window_count as f64 * TWO_WINDOWS.window_seconds;
    assert!(
        levels_match(report.decoded_secs, threshold + LONG_SECS as f64),
        "a violator pays its probe plus the full scan, got {}",
        report.decoded_secs,
    );
}

/// What:
///  a margin wider than the under-read drives the error below the quiet bound,
///  so an over-cautious probe is a violator in the other direction.
#[test]
fn a_margin_wider_than_the_under_read_is_a_too_quiet_violator() {
    let policy = default_policy();
    let margin_db = policy.max_too_quiet_db.abs() * 2.0;
    let candidate = Candidate {
        probe_margin_db: margin_db,
        ..TWO_WINDOWS
    };
    let report = evaluate(&[track("flat.flac", LONG_SECS, HOT_PEAK, flat_bins(LONG_SECS as usize, HOT_PEAK))], candidate);
    assert_eq!(report.violators.len(), 1, "the margin starves the track");
    let error_db = report.violators[0].error_db;
    assert!(levels_match(error_db, -margin_db), "the error is the negated margin, got {error_db}");
    assert!(error_db < policy.max_too_quiet_db, "it is too quiet");
}

/// What:
///  a track at the threshold is scanned exactly,
///  pays its own length,
///  and contributes no error.
#[test]
fn a_track_at_the_threshold_is_a_short_scan_that_pays_its_length() {
    let threshold_secs = (TWO_WINDOWS.window_count as f64 * TWO_WINDOWS.window_seconds) as u64;
    let report = evaluate(
        &[track("short.flac", threshold_secs, HOT_PEAK, flat_bins(threshold_secs as usize, HOT_PEAK))],
        TWO_WINDOWS,
    );
    assert_eq!(report.short_count, 1);
    assert_eq!(report.accepted_count, 0);
    assert!(report.violators.is_empty());
    assert!(levels_match(report.decoded_secs, threshold_secs as f64), "got {}", report.decoded_secs);
    assert_eq!(report.worst_too_loud_db, 0.0);
    assert_eq!(report.worst_too_quiet_db, 0.0);
}

/// What:
///  an empty corpus costs nothing and reports every count at zero.
#[test]
fn an_empty_corpus_scores_zero() {
    let report = evaluate(&[], TWO_WINDOWS);
    assert_eq!(report.short_count, 0);
    assert_eq!(report.accepted_count, 0);
    assert!(report.violators.is_empty());
    assert_eq!(report.decoded_secs, 0.0);
    assert_eq!(report.threshold_secs, TWO_WINDOWS.window_count as f64 * TWO_WINDOWS.window_seconds);
}

/// What:
///  each window samples the loudest bin it covers,
///  one bin per window second at the fixture's rate.
#[test]
fn sampled_windows_takes_the_loudest_bin_each_window_covers() {
    let windows = sampled_windows(&needle_track(), TWO_WINDOWS);
    assert_eq!(windows.len(), TWO_WINDOWS.window_count);
    assert!(levels_match(windows[0], f64::from(QUIET_PEAK)), "first second: {}", windows[0]);
    assert!(levels_match(windows[1], f64::from(QUIET_PEAK)), "last second: {}", windows[1]);

    // A three-second window covers the three bins it spans, so a loud bin inside one is seen.
    let wide = Candidate {
        window_seconds: 3.0,
        ..TWO_WINDOWS
    };
    let mut bins = flat_bins(LONG_SECS as usize, QUIET_PEAK);
    bins[LONG_SECS as usize - 2] = HOT_PEAK;
    let wide_windows = sampled_windows(&track("edge.flac", LONG_SECS, HOT_PEAK, bins), wide);
    assert!(levels_match(wide_windows[1], f64::from(HOT_PEAK)), "last window: {}", wide_windows[1]);
}
