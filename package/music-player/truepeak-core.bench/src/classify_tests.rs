//! Tests for the probe-only full-scan classifier:
//!  the full-recall cutoff,
//!  the routed decoded cost it implies,
//!  and the report line the fitted rule prints.

use super::*;
use crate::test_fixtures::{bins_with_one_hot, flat_bins, levels_match, track};
use truepeak_core::peak_dbtp;

/// Seconds in every fixture track,
///  so each one is long under the two-second threshold.
const LONG_SECS: u64 = 100;

/// The two-window,
/// one-second probe:
///  windows over the first and the last second,
///  so a loud second in the middle is invisible.
const TWO_WINDOWS: Candidate = Candidate {
    window_count: 2,
    window_seconds: 1.0,
    probe_margin_db: 0.0,
};

/// The loud second the probe windows cannot cover.
const UNCOVERED_SEC: usize = 50;

/// The peak of that one loud second,
///  which is the track's true peak as well.
const HOT_PEAK: f32 = 1.0;

/// The quiet peak of a track whose probe under-reads the loud second badly.
const DIMMEST_PEAK: f32 = 0.001;

/// The quiet peak of a track whose probe under-reads it less.
const DIM_PEAK: f32 = 0.01;

/// A peak below every violator's sampled level,
///  so the rule trusts the track.
const SAFE_PEAK: f32 = 0.0001;

/// A track whose only loud second sits between the probe windows.
fn needle(path: &str, quiet_peak: f32) -> Track {
    return track(
        path,
        LONG_SECS,
        HOT_PEAK,
        bins_with_one_hot(LONG_SECS as usize, quiet_peak, UNCOVERED_SEC, HOT_PEAK),
    )
}

/// A quiet flat track the probe sees completely,
///  so it is no violator and its error is zero.
fn quiet_track(path: &str) -> Track {
    return track(path, LONG_SECS, SAFE_PEAK, flat_bins(LONG_SECS as usize, SAFE_PEAK))
}

/// The probe threshold the rule charges each long track.
fn threshold_secs() -> f64 {
    return TWO_WINDOWS.window_count as f64 * TWO_WINDOWS.window_seconds
}

/// What:
///  the cutoff is the quietest violator's loudest window,
///  so routing at or above it catches every violator and only them.
#[test]
fn the_cutoff_catches_every_violator_and_routes_nothing_quieter() {
    let tracks = [needle("dim.flac", DIM_PEAK), needle("dimmest.flac", DIMMEST_PEAK), quiet_track("safe.flac")];
    let rule = fit_full_scan_rule(&tracks, TWO_WINDOWS, 1.0e9);
    assert!(
        levels_match(rule.cutoff_db, peak_dbtp(f64::from(DIMMEST_PEAK))),
        "the cutoff is the quietest violator's window, got {}",
        rule.cutoff_db,
    );
    assert_eq!(rule.violators, 2, "both needles under-read past the bound");
    assert_eq!(rule.long_tracks, tracks.len());
    assert_eq!(rule.routed, 2, "the quiet track stays below the cutoff");
    let expected = 2.0 * (threshold_secs() + LONG_SECS as f64) + threshold_secs();
    assert!(levels_match(rule.decoded_secs, expected), "routed cost {}, expected {expected}", rule.decoded_secs);
    assert!(levels_match(rule.worst_quiet_db, 0.0), "the trusted track's error, got {}", rule.worst_quiet_db);
}

/// What:
///  a corpus with no violator needs no routing,
///  so the cutoff is infinite and every long track pays only its probe.
#[test]
fn no_violators_leave_the_cutoff_infinite_and_route_nothing() {
    let rule = fit_full_scan_rule(&[quiet_track("safe.flac")], TWO_WINDOWS, 1.0e9);
    assert_eq!(rule.violators, 0);
    assert!(rule.cutoff_db.is_infinite(), "cutoff {}", rule.cutoff_db);
    assert_eq!(rule.routed, 0);
    assert_eq!(rule.long_tracks, 1);
    assert!(levels_match(rule.decoded_secs, threshold_secs()), "cost {}", rule.decoded_secs);
    assert!(levels_match(rule.worst_quiet_db, 0.0));
}

/// What:
///  an empty corpus fits no rule and costs nothing.
#[test]
fn an_empty_corpus_fits_no_rule_and_costs_nothing() {
    let rule = fit_full_scan_rule(&[], TWO_WINDOWS, 1.0e9);
    assert_eq!(rule.violators, 0);
    assert_eq!(rule.routed, 0);
    assert_eq!(rule.long_tracks, 0);
    assert!(rule.cutoff_db.is_infinite());
    assert_eq!(rule.decoded_secs, 0.0);
    assert_eq!(rule.worst_quiet_db, 0.0);
}

/// What:
///  the report line states the cutoff,
///  the routed counts,
///  and the gap to the target,
/// so the over-routing is visible where the rule prints.
#[test]
fn the_report_line_states_the_cutoff_the_counts_and_the_gap() {
    let target_secs = 1.0;
    let rule = fit_full_scan_rule(&[needle("dim.flac", DIM_PEAK), quiet_track("safe.flac")], TWO_WINDOWS, target_secs);
    let line = rule.to_string();
    assert!(line.contains(&format!("{:.3}dB", rule.cutoff_db)), "cutoff missing from {line}");
    assert!(line.contains("routes 1/2 long tracks"), "routed counts missing from {line}");
    assert!(line.contains("catches all 1 violators"), "violator count missing from {line}");
    assert!(line.contains(&format!("real_decoded={:.1}s", rule.decoded_secs)), "cost missing from {line}");
    assert!(line.contains(&format!("delta {:+.1}s", rule.decoded_secs - target_secs)), "gap missing from {line}");
}
