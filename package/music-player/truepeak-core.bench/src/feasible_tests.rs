//! Tests that the feasible model scores against the policy's own too-loud bound.

use super::*;
use truepeak_core::peak_dbtp;

/// The test density: two one-second windows, so the short/long threshold is two seconds.
const CANDIDATE: Candidate = Candidate {
    window_count: 2,
    window_seconds: 1.0,
    probe_margin_db: 0.0,
};

/// The linear peak of every second the two probe windows cover on `gap_track`.
const QUIET_PEAK: f32 = 0.05;

/// What: `gap_track` builds a hundred-second track whose only loud second (full-track true peak 1.0, 0 dBTP, above the
/// shipped ceiling) sits between the two probe windows.
/// Why: its under-read is the known `peak_dbtp(1.0) - peak_dbtp(QUIET_PEAK)`, so the margin has an exact expectation.
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

/// What: `under_read_db` is the level the probe misses on `gap_track`.
fn under_read_db() -> f64 {
    return peak_dbtp(1.0) - peak_dbtp(f64::from(QUIET_PEAK));
}

/// What: `policy_with_bound` is the shipped policy with a different too-loud bound.
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
    let policy = policy_with_bound(1.25);
    let split = provenance_margin(&policy, &tracks, CANDIDATE, &safe);
    let expected = under_read_db() - 1.25;
    assert!((split.margin_safe_db - expected).abs() < 1.0e-9, "safe margin {}", split.margin_safe_db);
    assert!((split.margin_unsafe_db - expected).abs() < 1.0e-9, "unsafe margin {}", split.margin_unsafe_db);
}

#[test]
fn the_shipped_policy_gives_the_margin_the_half_decibel_bound_gave() {
    let tracks = [gap_track("a.flac")];
    let feasible = evaluate_density(&default_policy(), &tracks, CANDIDATE, 1.0e9, -100.0);
    assert!((feasible.margin_db - (under_read_db() - 0.5)).abs() < 1.0e-9);
}
