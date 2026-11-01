//! Unit tests for the frontier-zoom probe,
//!  driven by a fake seekable source.

use super::*;
use crate::source::AudioSpec;

// A fake decoded mono source over an in-memory buffer, recording its seeks.
struct FakeSource {
    samples: Vec<f32>,
    rate: u32,
    cursor: usize,
    seeks: Vec<u64>,
}

impl FakeSource {
    fn new(samples: Vec<f32>, rate: u32) -> FakeSource {
        return FakeSource { samples, rate, cursor: 0, seeks: Vec::new() }
    }
}

impl TruePeakSource for FakeSource {
    fn spec(&self) -> AudioSpec {
        return AudioSpec {
            rate: self.rate,
            channels: 1,
            duration_secs: self.samples.len() as f64 / f64::from(self.rate),
        }
    }

    fn next_chunk(&mut self) -> Result<Vec<f32>, TruePeakError> {
        if self.cursor >= self.samples.len() {
            return Ok(Vec::new());
        }
        let end = (self.cursor + 4).min(self.samples.len());
        let block = self.samples[self.cursor..end].to_vec();
        self.cursor = end;
        return Ok(block)
    }

    fn seek_to_frame(&mut self, frame: u64) -> Result<(), TruePeakError> {
        self.seeks.push(frame);
        self.cursor = (frame as usize).min(self.samples.len());
        return Ok(())
    }
}

// 100 bins of 10 frames at rate 100 (10 s track), base level `base`, with per-bin
// overrides laid on top.
fn track(base: f32, overrides: &[(usize, f32)]) -> Vec<f32> {
    let mut samples = vec![base; 1000];
    for &(bin, level) in overrides {
        samples[bin * 10..(bin + 1) * 10].fill(level);
    }
    return samples
}

// The climb walks up a shoulder the even pass only grazes and reads the hilltop.
#[test]
fn climb_reads_the_hilltop() {
    // Hill: bins 42..=46 rise to 0.8 at bin 44; the even pass lands on bin 44's edge
    // neighborhood via bin 44 itself only if placement hits it, so put the top off-grid.
    let samples = track(0.1, &[(41, 0.2), (42, 0.4), (43, 0.8), (44, 0.4), (45, 0.2)]);
    let mut source = FakeSource::new(samples, 100);
    let plan = ZoomPlan {
        bin_frames: 10,
        total_frames: 1000,
        coverage_fraction: 0.2,
        even_coverage_fraction: 0.1,
        bones_hot_bins: None,
    };
    let peak = zoom_probe(&mut source, 1, &plan).expect("probe runs");
    // The hilltop bin holds 0.8; Catmull-Rom overshoot at value steps stays small.
    assert!(peak >= 0.8, "climb must reach the hilltop, got {peak}");
    // The budget is a fifth of 100 bins; every seek is one measured bin.
    assert!(source.seeks.len() <= 20, "budget exceeded: {} bins", source.seeks.len());
}

// Bones seeds reach an isolated needle no even pass or climb would find.
#[test]
fn bones_seeds_reach_the_needle() {
    let samples = track(0.05, &[(70, 0.9)]);
    let plan_without = ZoomPlan {
        bin_frames: 10,
        total_frames: 1000,
        coverage_fraction: 0.1,
        even_coverage_fraction: 0.1,
        bones_hot_bins: None,
    };
    let mut blind = FakeSource::new(samples.clone(), 100);
    let blind_peak = zoom_probe(&mut blind, 1, &plan_without).expect("probe runs");
    assert!(blind_peak < 0.5, "the even pass must miss the off-grid needle, got {blind_peak}");
    let hot = [70usize];
    let plan_with = ZoomPlan { bones_hot_bins: Some(&hot), ..plan_without };
    let mut guided = FakeSource::new(samples, 100);
    let guided_peak = zoom_probe(&mut guided, 1, &plan_with).expect("probe runs");
    assert!(guided_peak >= 0.9, "bones must reach the needle, got {guided_peak}");
}

// A silent track probes to zero and stays within budget.
#[test]
fn silence_probes_to_zero() {
    let mut source = FakeSource::new(vec![0.0; 1000], 100);
    let plan = ZoomPlan {
        bin_frames: 10,
        total_frames: 1000,
        coverage_fraction: 0.2,
        even_coverage_fraction: 0.1,
        bones_hot_bins: None,
    };
    let peak = zoom_probe(&mut source, 1, &plan).expect("probe runs");
    assert_eq!(peak, 0.0);
    assert!(source.seeks.len() <= 20);
}

// measure_window reads exactly one window from the current position.
#[test]
fn measure_window_reads_one_window() {
    let mut source = FakeSource::new(track(0.1, &[(1, 0.7)]), 100);
    source.seek_to_frame(10).expect("seek");
    let peak = measure_window(&mut source, 1, 10).expect("window measures");
    assert!(peak >= 0.7);
}
