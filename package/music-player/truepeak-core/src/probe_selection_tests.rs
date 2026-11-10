//! Unit tests for the probe's bin selection:
//!  the even pass placement,
//!  the budget,
//!  the bones seeds,
//!  the climb,
//!  and how a measurement failure comes back.

use super::*;
use std::convert::Infallible;

/// The peak a quiet bin carries.
const QUIET: f32 = 0.02;

/// The peak of the loudest bin.
const LOUD: f32 = 1.0;

/// The peak of a shoulder bin the climb walks through.
const SHOULDER: f32 = 0.5;

/// The bin count most tests grid over.
const BINS: usize = 100;

/// Quiet bins,
///  with `overrides` laid on top.
fn bins_with(overrides: &[(usize, f32)]) -> Vec<f32> {
    let mut bins = vec![QUIET; BINS];
    for &(index, peak) in overrides {
        bins[index] = peak;
    }
    return bins
}

/// A bin-peak lookup that records every bin the selection asks for,
///  the bench's own measurement shape.
struct Lookup<'peaks> {
    /// The precomputed peak of each bin.
    bin_peaks: &'peaks [f32],
    /// The bins the selection asked for,
    ///  in the order it asked.
    visited: Vec<usize>,
}

impl ProbeBins for Lookup<'_> {
    type Failure = Infallible;

    /// Records the asked-for bin and answers its precomputed peak.
    fn bin_peak(&mut self, index: usize) -> Result<f32, Infallible> {
        self.visited.push(index);
        return Ok(self.bin_peaks[index])
    }
}

impl Lookup<'_> {
    /// Run one selection over the precomputed peaks.
    ///
    /// What:
    ///  returns the selection's result,
    ///  which cannot fail here,
    ///  and leaves the visited bins in `self.visited`.
    ///  Why:
    ///  a lookup cannot fail,
    ///  so the impossible error is matched away rather than unwrapped.
    fn select(&mut self, grid: ProbeGrid<'_>) -> ProbeSelection {
        let selection = match select_probe_bins(grid, self) {
            Ok(selection) => selection,
            Err(never) => match never {},
        };
        return selection
    }
}

/// What:
///  the even pass spreads its bins evenly with both endpoints,
///  and its single sample sits mid-track.
/// Why:
///  the pass must slight no region,
///  and one sample has no spread to keep.
#[test]
fn the_even_pass_samples_evenly_with_both_endpoints() {
    let share = 0.1;
    let count = (share * BINS as f64).round() as usize;
    let span = BINS - 1;
    let mut indices: Vec<usize> = Vec::with_capacity(count);
    for step in 0..count {
        indices.push(even_bin_at(step, count, span));
    }
    assert_eq!(indices.len(), count, "the pass count is its share");
    assert_eq!(indices[0], 0, "the pass starts at the first bin");
    assert_eq!(indices[count - 1], span, "the pass ends at the last bin");
    assert!(indices.windows(2).all(|pair| return pair[0] < pair[1]), "indices ascend: {indices:?}");

    assert_eq!(even_bin_at(0, 1, span), span / 2, "a single sample sits mid-track");
    assert_eq!(even_bin_at(0, 1, 0), 0, "one bin has one place to be");
}

/// What:
///  a coverage above one cannot promise more bins than the grid holds,
///  so every bin is measured once and the climb finds nothing left.
/// Why:
///  the budget is a share of the bins,
///  and a share above one is still bounded by the bins there are.
#[test]
fn a_coverage_above_one_measures_each_bin_once() {
    let bin_count = 8;
    let bin_peaks = vec![QUIET; bin_count];
    let mut lookup = Lookup { bin_peaks: &bin_peaks, visited: Vec::new() };
    let selection = lookup.select(ProbeGrid {
        bin_count,
        coverage_fraction: 5.0,
        even_coverage_fraction: 1.0,
        bones_hot_bins: None,
    });
    assert_eq!(selection.used_bins, bin_count, "the budget stops at the bins there are");
    assert_eq!(lookup.visited, (0..bin_count).collect::<Vec<usize>>(), "every bin once, in order");
    assert_eq!(selection.peak, QUIET);
}

/// What:
///  the climb walks from a shoulder the even pass measured to the hilltop beside it.
/// Why:
///  a shoulder the even pass grazes must lead to the crest it belongs to.
#[test]
fn the_climb_reads_the_hilltop_beside_the_even_pass() {
    // Bin 44 is one of the even pass's own places, so its shoulder is measured first and
    // the climb starts there; the crest sits beside it at bin 43.
    let shoulder = 44usize;
    let hilltop = shoulder - 1;
    let bin_peaks = bins_with(&[(shoulder, SHOULDER), (hilltop, LOUD)]);
    let mut lookup = Lookup { bin_peaks: &bin_peaks, visited: Vec::new() };
    let selection = lookup.select(ProbeGrid {
        bin_count: BINS,
        coverage_fraction: 0.2,
        even_coverage_fraction: 0.1,
        bones_hot_bins: None,
    });
    assert!(lookup.visited.contains(&hilltop), "the climb reaches bin {hilltop}, visited {:?}", lookup.visited);
    assert_eq!(selection.peak, LOUD, "the loudest measured bin is the hilltop");
    assert_eq!(selection.used_bins, 20, "a fifth of the bins");
}

/// What:
///  a bones slot at or past the last bin seeds the last bin and its neighbour,
///  and nothing else.
/// Why:
///  a profile's slot resolution need not match the bin grid,
///  so a slot past the end names the last bin rather than nothing.
#[test]
fn a_bones_slot_past_the_last_bin_seeds_the_last_two_bins() {
    let hot_slot = BINS;
    let bin_peaks = bins_with(&[(BINS - 2, LOUD)]);
    let hot_bins = [hot_slot];
    let mut lookup = Lookup { bin_peaks: &bin_peaks, visited: Vec::new() };
    let selection = lookup.select(ProbeGrid {
        bin_count: BINS,
        coverage_fraction: 0.02,
        even_coverage_fraction: 0.0,
        bones_hot_bins: Some(&hot_bins),
    });
    assert_eq!(lookup.visited, vec![BINS - 2, BINS - 1], "the seed measures the last two bins");
    assert_eq!(selection.peak, LOUD, "the seeded neighbour holds the crest");
    assert_eq!(selection.used_bins, 2, "the seed spends the whole budget");
}

/// What:
///  a grid with no bins still has one place to measure.
/// Why:
///  every later step indexes the grid,
///  so a degenerate track must not make the bin count zero.
#[test]
fn a_grid_with_no_bins_still_measures_the_first_bin() {
    let bin_peaks = [QUIET];
    let mut lookup = Lookup { bin_peaks: &bin_peaks, visited: Vec::new() };
    let selection = lookup.select(ProbeGrid {
        bin_count: 0,
        coverage_fraction: 0.1,
        even_coverage_fraction: 0.1,
        bones_hot_bins: None,
    });
    assert_eq!(lookup.visited, vec![0]);
    assert_eq!(selection.used_bins, 1);
    assert_eq!(selection.peak, QUIET);
}

/// A bin-peak lookup that fails on one chosen bin.
struct FailingLookup<'peaks, 'visited> {
    /// The precomputed peak of each bin.
    bin_peaks: &'peaks [f32],
    /// The bins the selection asked for,
    ///  in the order it asked.
    visited: &'visited mut Vec<usize>,
    /// The one bin whose measurement fails.
    failing_bin: usize,
}

impl ProbeBins for FailingLookup<'_, '_> {
    type Failure = &'static str;

    /// Records the asked-for bin and fails on the one chosen bin.
    fn bin_peak(&mut self, index: usize) -> Result<f32, &'static str> {
        self.visited.push(index);
        if index == self.failing_bin {
            return Err("decode failed")
        }
        return Ok(self.bin_peaks[index])
    }
}

/// What:
///  a measurement failure comes back to the caller untouched,
///  and the selection measures no bin after it.
/// Why:
///  the caller owns the error type,
///  so the selection must not swallow or wrap it.
#[test]
fn a_measurement_failure_comes_back_untouched() {
    let bin_peaks = bins_with(&[]);
    let failing_bin = 11usize;
    let mut visited: Vec<usize> = Vec::new();
    let result = select_probe_bins(
        ProbeGrid {
            bin_count: BINS,
            coverage_fraction: 0.1,
            even_coverage_fraction: 0.1,
            bones_hot_bins: None,
        },
        &mut FailingLookup { bin_peaks: &bin_peaks, visited: &mut visited, failing_bin },
    );
    assert_eq!(result.err(), Some("decode failed"));
    assert_eq!(visited.last(), Some(&failing_bin), "nothing is measured after the failure");
}

/// What:
///  a zero coverage share still measures one bin and reports its peak.
/// Why:
///  a probe that measured nothing could not report a level,
///  so the budget floors at one bin.
#[test]
fn a_zero_coverage_still_measures_one_bin() {
    let bin_peaks = bins_with(&[]);
    let mut lookup = Lookup { bin_peaks: &bin_peaks, visited: Vec::new() };
    let selection = lookup.select(ProbeGrid {
        bin_count: BINS,
        coverage_fraction: 0.0,
        even_coverage_fraction: 0.0,
        bones_hot_bins: None,
    });
    assert_eq!(selection.used_bins, 1, "a zero share still measures one bin");
    assert_eq!(selection.peak, QUIET);
}
