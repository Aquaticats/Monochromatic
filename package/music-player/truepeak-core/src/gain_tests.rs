//! Unit tests for the attenuate-only gain and the dB helpers.

use super::*;

// A non-positive (silent or invalid) peak leaves the signal unchanged at unity gain.
#[test]
fn silence_gives_unity_gain() {
    assert_eq!(normalization_gain(0.0), 1.0);
    assert_eq!(normalization_gain(-0.3), 1.0);
}

// A peak below the ceiling is left alone: the policy never amplifies.
#[test]
fn quiet_track_is_not_amplified() {
    assert_eq!(normalization_gain(0.5), 1.0);
    assert_eq!(normalization_gain(CEILING - 0.01), 1.0);
}

// A peak above the ceiling is attenuated to exactly the ceiling.
#[test]
fn loud_track_is_attenuated_to_ceiling() {
    let peak = 1.0_f32;
    let gain = normalization_gain(peak);
    assert!(
        (gain - CEILING).abs() < 1e-6,
        "gain {gain} should equal the ceiling"
    );
    // Applying the gain brings the peak down to the ceiling.
    assert!((peak * gain - CEILING).abs() < 1e-6);
}

// The gain is never above unity for any peak, including very loud ones.
#[test]
fn gain_never_exceeds_unity() {
    for step in 0..200 {
        let peak = step as f32 / 100.0; // 0.0 .. 2.0
        assert!(
            normalization_gain(peak) <= 1.0,
            "peak {peak} produced a boost"
        );
    }
}

// Full scale is 0 dBTP; half amplitude is about -6.02 dBTP.
#[test]
fn peak_dbtp_reference_points() {
    assert!((peak_dbtp(1.0) - 0.0).abs() < 1e-9);
    assert!((peak_dbtp(0.5) - (-6.020599913)).abs() < 1e-6);
}

// A zero-dB margin leaves the peak unchanged; +6.0206 dB doubles it.
#[test]
fn probe_margin_inflates_in_db() {
    assert!((probe_estimated_peak(0.7, 0.0) - 0.7).abs() < 1e-12);
    assert!((probe_estimated_peak(1.0, 6.020599913) - 2.0).abs() < 1e-6);
}

// The linear gain matches the dB-domain gain `min(0, -1 - 20*log10(peak))` for a loud
// peak, proving the two formulations agree (the equivalence the plan relies on).
#[test]
fn linear_gain_matches_db_form() {
    let peak = 1.3_f64;
    let exact_gain_db = (-1.0 - peak_dbtp(peak)).min(0.0);
    let db_form_linear = 10.0_f64.powf(exact_gain_db / 20.0);
    let linear = normalization_gain(peak as f32) as f64;
    assert!(
        (linear - db_form_linear).abs() < 1e-6,
        "linear {linear} vs dB-form {db_form_linear}"
    );
}
