//! Tests that the feasible model scores against the policy's own too-loud bound.

use super::*;
use crate::test_fixtures::{flat_bins, levels_match, track};
use truepeak_core::peak_dbtp;

/// The test density:
/// two one-second windows,
/// so the short/long threshold is two seconds.
const CANDIDATE: Candidate = Candidate {
    window_count: 2,
    window_seconds: 1.0,
    probe_margin_db: 0.0,
};

/// The linear peak of every second the two probe windows cover on `gap_track`.
const QUIET_PEAK: f32 = 0.05;

/// What:
/// `gap_track` builds a hundred-second track whose only loud second (full-track true peak 1.0,
/// 0 dBTP,
/// above the
/// shipped ceiling) sits between the two probe windows.
/// Why:
/// its under-read is the known `peak_dbtp(1.0) - peak_dbtp(QUIET_PEAK)`,
/// so the margin has an exact expectation.
fn gap_track(path: &str) -> Track {
    let mut bin_peaks = vec![QUIET_PEAK; 100];
    bin_peaks[50] = 1.0;
    return Track {
        path: path.to_string(),
        duration_secs: 100.0,
        rate: 10,
        decoded_frames: 1000,
        full_peak: 1.0,
        bin_seconds: 1.0,
        bin_peaks,
    };
}

/// What:
/// `under_read_db` is the level the probe misses on `gap_track`.
fn under_read_db() -> f64 {
    return peak_dbtp(1.0) - peak_dbtp(f64::from(QUIET_PEAK));
}

/// What:
/// `policy_with_bound` is the shipped policy with a different too-loud bound.
fn policy_with_bound(max_too_loud_db: f64) -> Policy {
    return Policy {
        max_too_loud_db,
        ..default_policy()
    };
}

#[test]
fn the_density_margin_is_the_under_read_minus_the_policy_too_loud_bound() {
    let tracks = [gap_track("a.flac")];
    for bound in [0.5, 1.25, 2.0] {
        let policy = policy_with_bound(bound);
        let feasible = evaluate_density(&policy, &tracks, CANDIDATE, 1.0e9, -100.0);
        let expected = under_read_db() - bound;
        assert!(
            (feasible.margin_db - expected).abs() < 1.0e-9,
            "bound {bound}: margin {} is not under-read minus the bound {expected}",
            feasible.margin_db,
        );
    }
}

#[test]
fn the_provenance_margins_follow_the_policy_too_loud_bound_for_both_groups() {
    let safe_track = gap_track("safe.flac");
    let unsafe_track = gap_track("lossy.mp3");
    let safe: HashSet<String> = HashSet::from([safe_track.path.clone()]);
    let tracks = [safe_track, unsafe_track];
    let bound = 1.25;
    let policy = policy_with_bound(bound);
    let split = provenance_margin(&policy, &tracks, CANDIDATE, &safe);
    let expected = under_read_db() - bound;
    assert!((split.margin_safe_db - expected).abs() < 1.0e-9, "safe margin {}", split.margin_safe_db);
    assert!((split.margin_unsafe_db - expected).abs() < 1.0e-9, "unsafe margin {}", split.margin_unsafe_db);
}

#[test]
fn the_shipped_policy_gives_the_margin_the_half_decibel_bound_gave() {
    let tracks = [gap_track("a.flac")];
    let policy = default_policy();
    let feasible = evaluate_density(&policy, &tracks, CANDIDATE, 1.0e9, -100.0);
    assert!((feasible.margin_db - (under_read_db() - policy.max_too_loud_db)).abs() < 1.0e-9);
}

/// Seconds in every fixture track of the density tests,
///  past the two-second threshold so each one is probed.
const LONG_SECS: u64 = 100;

/// The peak of a flat fixture track,
///  below the ceiling so its under-read is zero.
const FLAT_PEAK: f32 = 0.5;

/// What:
///  a density charges a short track its whole length and a long track one probe.
#[test]
fn the_density_charges_shorts_in_full_and_longs_one_probe() {
    let short_secs = CANDIDATE.window_count as u64;
    let short = track("short.flac", short_secs, FLAT_PEAK, flat_bins(short_secs as usize, FLAT_PEAK));
    let long = track("long.flac", LONG_SECS, FLAT_PEAK, flat_bins(LONG_SECS as usize, FLAT_PEAK));
    let feasible = evaluate_density(&default_policy(), &[short, long], CANDIDATE, 1.0e9, -100.0);
    let threshold = CANDIDATE.window_count as f64 * CANDIDATE.window_seconds;
    let expected = short_secs as f64 + threshold;
    assert!(levels_match(feasible.probe_decoded_secs, expected), "decoded {}", feasible.probe_decoded_secs);
    assert!(levels_match(feasible.margin_db, 0.0), "a flat quiet corpus needs no margin");
    assert!(feasible.feasible, "zero margin inside a wide bound and budget is feasible");
}

/// What:
///  a density is infeasible when its margin leaves the quiet bound,
///  and when its probe cost overruns the budget.
#[test]
fn a_density_outside_the_bound_or_the_budget_is_infeasible() {
    let gap = gap_track("gap.flac");
    let policy = default_policy();
    let over_bound = evaluate_density(&policy, &[gap], CANDIDATE, 1.0e9, -1.0);
    assert!(!over_bound.feasible, "a {:.3} dB margin breaks a -1 dB bound", over_bound.margin_db);
    assert!(over_bound.worst_quiet_db < -1.0);

    let flat = track("flat.flac", LONG_SECS, FLAT_PEAK, flat_bins(LONG_SECS as usize, FLAT_PEAK));
    let over_budget = evaluate_density(&policy, &[flat], CANDIDATE, 0.0, -100.0);
    assert!(!over_budget.feasible, "a zero target cannot afford even one probe");
    assert!(over_budget.probe_decoded_secs > 0.0);
}

/// What:
///  the density search puts feasible points first,
///  then the smallest margin,
///  then the smallest probe cost.
#[test]
fn the_density_search_orders_by_margin_then_cost() {
    let flat = track("flat.flac", LONG_SECS, FLAT_PEAK, flat_bins(LONG_SECS as usize, FLAT_PEAK));
    let thresholds = [CANDIDATE.window_count as f64 * CANDIDATE.window_seconds, 4.0];
    let results = best_feasible(&[flat], &[CANDIDATE.window_count], &thresholds, 1.0e9, -100.0);
    assert_eq!(results.len(), thresholds.len());
    assert!(results.iter().all(|point| return point.feasible), "a flat corpus is feasible everywhere");
    let costs: Vec<f64> = results.iter().map(|point| return point.probe_decoded_secs).collect();
    assert_eq!(costs, vec![thresholds[0], thresholds[1]], "the cheaper probe ranks first");
}
