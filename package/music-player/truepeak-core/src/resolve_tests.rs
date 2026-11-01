//! Integration tests for the policy resolver,
//!  driven by a fake decoded source.

use super::*;
use crate::bucketpolicy::{BucketProbe, BucketTable};
use crate::source::AudioSpec;
use crate::{normalization_gain, probe_estimated_peak, true_peak_interleaved, window_frame_starts};

// A fake decoded source over an in-memory buffer, recording the frames it was seeked to.
struct FakeSource {
    samples: Vec<f32>,
    channels: u16,
    rate: u32,
    duration_secs: f64,
    cursor: usize,
    chunk: usize,
    seeks: Vec<u64>,
}

impl FakeSource {
    fn new(samples: Vec<f32>, channels: u16, rate: u32, duration_secs: f64) -> FakeSource {
        return FakeSource { samples, channels, rate, duration_secs, cursor: 0, chunk: 4, seeks: Vec::new() }
    }
}

impl TruePeakSource for FakeSource {
    fn spec(&self) -> AudioSpec {
        return AudioSpec { rate: self.rate, channels: self.channels, duration_secs: self.duration_secs }
    }

    fn next_chunk(&mut self) -> Result<Vec<f32>, TruePeakError> {
        if self.cursor >= self.samples.len() {
            return Ok(Vec::new());
        }
        let end = (self.cursor + self.chunk).min(self.samples.len());
        let block = self.samples[self.cursor..end].to_vec();
        self.cursor = end;
        return Ok(block)
    }

    fn seek_to_frame(&mut self, frame: u64) -> Result<(), TruePeakError> {
        self.seeks.push(frame);
        self.cursor = frame as usize * self.channels as usize;
        return Ok(())
    }
}

// A policy with small parameters so a tiny buffer exercises each branch: every bucket
// shares the same coverage and margin, and the even pass spends the whole budget so the
// probe visits the same evenly-placed bins the old tests reasoned about.
fn test_policy(short_scan_max_secs: f64, coverage_fraction: f64, probe_window_secs: f64, probe_margin_db: f64) -> Policy {
    let uniform = BucketProbe { coverage_fraction, probe_margin_db };
    return Policy {
        short_scan_max_secs,
        probe_window_secs,
        pass1_coverage_fraction: coverage_fraction,
        bones_even_coverage_fraction: coverage_fraction,
        bones_top_slots: 4,
        buckets: BucketTable {
            lossless: uniform,
            lossless_bones: uniform,
            store: uniform,
            youtube: uniform,
            bare: uniform,
        },
        ceiling_dbtp: -1.0,
        max_too_loud_db: 0.5,
        max_too_quiet_db: -2.0,
    }
}

// A short track is scanned in full and its gain is exact.
#[test]
fn short_track_full_scans_exactly() {
    let samples = vec![0.0_f32, 0.9, 0.9, 0.0];
    let mut source = FakeSource::new(samples.clone(), 1, 4, 1.0);
    let decision = resolve_decision(&test_policy(100.0, 0.2, 0.3, 0.8), &mut source)
        .expect("short-track resolution should succeed");

    let expected_peak = true_peak_interleaved(&samples, 1);
    assert_eq!(decision.kind, DecisionKind::ShortFullScan);
    assert!((decision.measured_peak - expected_peak).abs() < 1e-6);
    assert_eq!(decision.gain, normalization_gain(expected_peak));
    assert!(source.seeks.is_empty()); // a full scan never seeks
}

// A long track is probed: the resolver seeks to the placed windows and applies the margin.
#[test]
fn long_track_probes_with_margin() {
    let samples = vec![0.9_f32; 20]; // constant, so every window peak is 0.9
    let mut source = FakeSource::new(samples, 1, 10, 2.0);
    let policy = test_policy(0.5, 1.0, 0.5, 1.0);
    let decision = resolve_decision(&policy, &mut source)
        .expect("long-track resolution should succeed");

    assert_eq!(decision.kind, DecisionKind::ProbeEstimate);
    // total_frames=20, count=round(2.0/0.5)=4, window_frames=round(0.5*10)=5.
    assert_eq!(source.seeks, window_frame_starts(20, 4, 5));
    assert!((decision.measured_peak - 0.9).abs() < 1e-6);
    // The gain is the margin-inflated sampled peak, computed exactly as the resolver does.
    let expected = normalization_gain(probe_estimated_peak(f64::from(decision.measured_peak), 1.0) as f32);
    assert_eq!(decision.gain, expected);
    // The margin makes the probe gain quieter than the exact gain for the same sampled peak.
    assert!(decision.gain < normalization_gain(0.9));
}

// Seeking actually finds a loud region that sits between the quiet parts.
#[test]
fn probe_seek_finds_a_mid_track_loud_window() {
    let mut samples = vec![0.5_f32; 20];
    samples[10..15].fill(1.5); // a loud window at frames 10..15
    let mut source = FakeSource::new(samples, 1, 10, 2.0);
    let decision = resolve_decision(&test_policy(0.5, 1.0, 0.5, 0.8), &mut source)
        .expect("probe resolution should succeed");

    // A window starts at frame 10 and covers the loud region, so the sampled peak sees it.
    assert!(decision.measured_peak >= 1.5, "sampled peak {} should catch the loud window", decision.measured_peak);
}

// Silence yields unity gain, never a NaN.
#[test]
fn silence_yields_unity_gain() {
    let mut source = FakeSource::new(vec![0.0_f32; 16], 2, 8, 1.0);
    let decision = resolve_decision(&test_policy(100.0, 0.2, 0.3, 0.8), &mut source)
        .expect("silent-track resolution should succeed");
    assert_eq!(decision.gain, 1.0);
    assert_eq!(decision.kind, DecisionKind::ShortFullScan);
}

// An unknown-duration track is full-scanned exactly, not probed.
#[test]
fn unknown_duration_full_scans() {
    let samples = vec![0.0_f32, 0.9, 0.9, 0.0, 0.5, 0.5];
    let mut source = FakeSource::new(samples, 1, 10, 0.0); // duration 0 = unknown
    let decision = resolve_decision(&test_policy(0.1, 1.0, 0.5, 0.8), &mut source)
        .expect("unknown-duration resolution should succeed");
    assert_eq!(decision.kind, DecisionKind::FullScanExact);
    assert!(source.seeks.is_empty());
}

// A zero-channel stream is treated as silence.
#[test]
fn zero_channels_is_unity() {
    let mut source = FakeSource::new(vec![0.5_f32; 8], 0, 48000, 5.0);
    let decision = resolve_decision(&test_policy(100.0, 0.2, 0.3, 0.8), &mut source)
        .expect("zero-channel resolution should succeed");
    assert_eq!(decision.gain, 1.0);
    assert_eq!(decision.kind, DecisionKind::FullScanExact);
}

// The warming upgrade: a long track that `resolve_decision` would PROBE is instead scanned
// in full, exactly, with no seeks, and tagged `FullScanExact`.
#[test]
fn full_scan_upgrades_a_long_track_exactly() {
    let mut samples = vec![0.5_f32; 20];
    samples[10..15].fill(1.5); // a loud region a sparse probe could miss
    let source_samples = samples.clone();
    let mut source = FakeSource::new(samples, 1, 10, 2.0); // long under short_scan_max = 0.5
    let policy = test_policy(0.5, 1.0, 0.5, 0.8);

    // The probe path would tag this a probe estimate; the full-scan path never does.
    let probe = resolve_decision(
        &policy,
        &mut FakeSource::new(source_samples.clone(), 1, 10, 2.0),
    )
    .expect("probe resolution should succeed");
    assert_eq!(probe.kind, DecisionKind::ProbeEstimate);

    let decision = resolve_full_scan(&policy, &mut source)
        .expect("full-scan resolution should succeed");
    assert_eq!(decision.kind, DecisionKind::FullScanExact);
    assert!(source.seeks.is_empty()); // a full scan never seeks
    let expected_peak = true_peak_interleaved(&source_samples, 1);
    assert!((decision.measured_peak - expected_peak).abs() < 1e-6);
    assert_eq!(decision.gain, normalization_gain(expected_peak));
}

// A short track full-scanned still reports `ShortFullScan`, matching `resolve_decision`.
#[test]
fn full_scan_of_short_track_tags_short() {
    let samples = vec![0.0_f32, 0.9, 0.9, 0.0];
    let mut source = FakeSource::new(samples, 1, 4, 1.0);
    let decision = resolve_full_scan(&test_policy(100.0, 0.2, 0.3, 0.8), &mut source)
        .expect("short full-scan resolution should succeed");
    assert_eq!(decision.kind, DecisionKind::ShortFullScan);
    assert!(source.seeks.is_empty());
}

// Provenance picks the bucket: a lossless track probes at its bucket's smaller coverage,
// so it measures fewer bins than the bare default for the same audio.
#[test]
fn provenance_picks_bucket_coverage() {
    let mut policy = test_policy(0.5, 1.0, 0.5, 0.8);
    policy.buckets.lossless = BucketProbe { coverage_fraction: 0.5, probe_margin_db: 0.8 };
    let samples = vec![0.9_f32; 20]; // 4 bins of 5 frames at rate 10

    let mut bare = FakeSource::new(samples.clone(), 1, 10, 2.0);
    resolve_decision(&policy, &mut bare).expect("bare-track resolution should succeed");
    assert_eq!(bare.seeks.len(), 4); // bare coverage 1.0 measures every bin

    let mut lossless = FakeSource::new(samples, 1, 10, 2.0);
    let provenance = TrackProvenance { lossless: true, ..TrackProvenance::unknown() };
    resolve_decision_for(&policy, &mut lossless, provenance, None)
        .expect("lossless-track resolution should succeed");
    assert_eq!(lossless.seeks.len(), 2); // lossless coverage 0.5 measures half
}

// Bones seeds send the lossless probe straight to the flagged bin.
#[test]
fn bones_seeds_guide_the_lossless_probe() {
    let mut policy = test_policy(0.5, 1.0, 0.5, 0.8);
    policy.buckets.lossless_bones = BucketProbe { coverage_fraction: 0.5, probe_margin_db: 0.8 };
    policy.bones_even_coverage_fraction = 0.25;
    let mut samples = vec![0.1_f32; 20];
    samples[15..20].fill(1.4); // the loud bin is the last of the four

    let mut source = FakeSource::new(samples, 1, 10, 2.0);
    let provenance = TrackProvenance { lossless: true, ..TrackProvenance::unknown() };
    let hot = [3usize];
    let decision = resolve_decision_for(&policy, &mut source, provenance, Some(&hot))
        .expect("seeded lossless-track resolution should succeed");
    // The seed measures the flagged bin (frame 15) within the two-bin budget.
    assert!(source.seeks.contains(&15), "seeds must visit the flagged bin: {:?}", source.seeks);
    assert!(source.seeks.len() <= 2);
    assert!(decision.measured_peak >= 1.4);
}
