//! Unit tests for the shared Catmull-Rom meter.

// `use super::*` pulls in the private `catmull_rom`, the `TruePeakMeter`, the
// constants, and `true_peak_interleaved` from the parent `meter` module.
use super::*;

// Float comparison tolerance for interpolation results.
const EPS: f32 = 1e-5;

// Catmull-Rom must reproduce the second control point at t = 0 and the third at t = 1.
#[test]
fn catmull_rom_reproduces_control_points() {
    let p0 = 0.2_f32;
    let p1 = 0.5_f32;
    let p2 = 0.8_f32;
    let p3 = 0.1_f32;
    assert!((catmull_rom(p0, p1, p2, p3, 0.0) - p1).abs() < EPS);
    assert!((catmull_rom(p0, p1, p2, p3, 1.0) - p2).abs() < EPS);
}

// A curve that rises between two equal samples overshoots them: the meter must report
// the inter-sample peak, not the largest stored sample.
#[test]
fn detects_inter_sample_peak_above_samples() {
    // Mono. Window reaches [0.0, 0.9, 0.9, 0.0]; the 0.9..0.9 segment peaks at ~1.0125.
    let samples = [0.0_f32, 0.9, 0.9, 0.0];
    let peak = true_peak_interleaved(&samples, 1);
    assert!(
        peak > 0.9,
        "inter-sample peak should exceed the 0.9 samples, got {peak}"
    );
    assert!((peak - 1.0125).abs() < 1e-3, "expected ~1.0125, got {peak}");
}

// Silence is a zero peak (the gain math maps that to unity gain), never a NaN.
#[test]
fn silence_is_zero_peak() {
    let samples = [0.0_f32; 128];
    assert_eq!(true_peak_interleaved(&samples, 2), 0.0);
}

// A zero-channel request is handled as silence, not a divide-by-zero.
#[test]
fn zero_channels_is_zero_peak() {
    let samples = [0.5_f32; 8];
    assert_eq!(true_peak_interleaved(&samples, 0), 0.0);
}

// The raw sample magnitude is always a lower bound on the reported peak.
#[test]
fn peak_is_at_least_max_sample() {
    let samples = [0.1_f32, -0.7, 0.3, 0.6, -0.2, 0.4];
    let peak = true_peak_interleaved(&samples, 2);
    assert!(
        peak >= 0.7 - EPS,
        "peak {peak} should be at least the 0.7 sample magnitude"
    );
}

// The channel cursor must persist across feed calls, so splitting a stereo stream at a
// mid-frame (odd) boundary yields the same peak as feeding it whole.
#[test]
fn channel_routing_survives_mid_frame_chunk_split() {
    // Distinct L/R envelopes so mis-routing would change the interpolated peak.
    let stereo: Vec<f32> = (0..64)
        .flat_map(|frame| {
            let left = if frame % 2 == 0 { 0.9 } else { 0.1 };
            let right = if frame % 2 == 0 { -0.2 } else { 0.5 };
            return [left, right];
        })
        .collect();

    let whole = true_peak_interleaved(&stereo, 2);

    // Feed the same data in chunks that deliberately break mid-frame (length 3, odd).
    let mut meter = TruePeakMeter::new(2);
    for chunk in stereo.chunks(3) {
        meter.feed(chunk);
    }
    let split = meter.peak();

    assert!(
        (whole - split).abs() < EPS,
        "whole {whole} vs mid-frame split {split}"
    );
}

// take_peak reads and resets the running max while keeping the window continuous, so the
// max of the per-segment peaks equals the continuous whole-buffer peak exactly.
#[test]
fn take_peak_segments_match_continuous_peak() {
    let samples = [0.0_f32, 0.9, 0.9, 0.0];
    let whole = true_peak_interleaved(&samples, 1);

    let mut meter = TruePeakMeter::new(1);
    meter.feed(&samples[0..2]);
    let segment_one = meter.take_peak();
    meter.feed(&samples[2..4]);
    let segment_two = meter.take_peak();

    // The inter-sample peak straddles the segment boundary; continuity must preserve it.
    assert!((segment_one.max(segment_two) - whole).abs() < 1e-6);
    // After a take, the running peak is cleared.
    assert_eq!(meter.peak(), 0.0);
}

// Feeding sample-by-sample matches feeding the whole buffer (window state persists).
#[test]
fn per_sample_feeding_matches_whole_buffer() {
    let samples = [0.0_f32, 0.8, -0.6, 0.95, 0.3, -0.9, 0.1, 0.7];
    let whole = true_peak_interleaved(&samples, 1);

    let mut meter = TruePeakMeter::new(1);
    for &sample in &samples {
        meter.feed(&[sample]);
    }
    assert!((whole - meter.peak()).abs() < EPS);
}
