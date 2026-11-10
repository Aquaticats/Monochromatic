//! The probe's bin selection:
//!  bones seeds,
//!  the even pass,
//!  and the neighbour climb,
//!  decided once for every caller.
//!
//! What:
//!  one function turns a bin grid and a budget into the bins a probe measures,
//!  asking for one bin's peak at a time through a one-method trait.
//! Why:
//!  the production probe measures a window from the decoder,
//!  and the bench sidecar looks a precomputed bin peak up,
//!  so the shape lives here and neither caller keeps a copy.
//!  Two copies had already drifted:
//!  the bench seeded bones slots differently and never capped the budget at the bin
//!  count,
//!  which let corpus evidence diverge from what playback runs.
//!
//! The selection is a hill climb over measurement:
//!  an even pass slights no region,
//!  and then every measured loud bin becomes a path,
//!  repeatedly measuring the unmeasured bins on either side of the loudest bin
//!  measured so far until the budget is spent.
//!  A miss most often lands on the shoulder of a loud passage,
//!  so climbing collapses the under-read that a purely even pass leaves behind.

/// Imports the max-heap driving the frontier expansion.
use std::collections::BinaryHeap;
/// Imports the impossible error of a measurement that cannot fail.
use std::convert::Infallible;

/// The bin grid one probe selection runs over.
///
/// What:
///  the bin count and the two coverage shares the selection spends,
///  plus the optional lossless bones seeds.
/// Why:
///  the budget and the even pass are fractions of the bin count,
///  so the grid is everything the selection needs besides a way to measure a bin.
#[derive(Clone, Copy, Debug)]
pub struct ProbeGrid<'hot> {
    /// How many bins the track has.
    pub bin_count: usize,
    /// Fraction of bins the probe may measure in total.
    pub coverage_fraction: f64,
    /// Fraction of bins the even pass measures before the climb.
    pub even_coverage_fraction: f64,
    /// Byte-rate hot slots to seed (lossless bones),
    ///  each measured with its neighbours.
    pub bones_hot_bins: Option<&'hot [usize]>,
}

/// One measurement a probe selection asks for:
///  the peak of one bin.
///
/// What:
///  an implementation answers "what is bin `index`'s peak" and names its own failure.
/// Why:
///  the production probe measures a decoded window,
///  and the bench sidecar looks a precomputed bin peak up;
///  the selection decides which bins,
///  never how they are measured.
pub trait ProbeBins {
    /// What a measurement failure is;
    ///  a caller that cannot fail names its own impossible type.
    type Failure;

    /// Measure the peak of bin `index`.
    ///
    /// What:
    ///  answers one bin's peak,
    ///  or the implementation's failure,
    ///  which the selection hands back untouched.
    ///  Why:
    ///  the selection asks one bin at a time,
    ///  in the order its budget and the loudest measured bin decide.
    fn bin_peak(&mut self, index: usize) -> Result<f32, Self::Failure>;
}

/// What:
///  a precomputed bin-peak slice answers its own measurements.
/// Why:
///  replaying a corpus needs no decoder,
///  so the bin peaks a collector measured are the measurement.
impl ProbeBins for &[f32] {
    /// A copy of a precomputed peak cannot fail.
    type Failure = Infallible;

    /// Reads the bin's precomputed peak out of the slice.
    fn bin_peak(&mut self, index: usize) -> Result<f32, Infallible> {
        return Ok(self[index])
    }
}

/// What one probe selection measured.
///
/// What:
///  the loudest bin peak the selection read,
///  and how many bins it measured.
/// Why:
///  the production probe takes the peak for its gain,
///  and the bench charges the measured bins as its decoded-seconds cost.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProbeSelection {
    /// The loudest bin peak the selection measured.
    pub peak: f32,
    /// How many bins the selection measured.
    pub used_bins: usize,
}

/// The measured set one selection threads through its three phases.
struct Frontier {
    /// Whether each bin was measured already.
    measured: Vec<bool>,
    /// Measured bins by loudness;
    ///  non-negative f32 bits order exactly like the values.
    loudest_first: BinaryHeap<(u32, usize)>,
    /// Bins measured so far.
    used_bins: usize,
    /// Bins the selection may measure in total.
    budget_bins: usize,
    /// Loudest bin peak read so far.
    peak: f32,
}

/// What:
///  the measured set's own steps:
///  fold a measurement in,
///  and report whether the budget has room.
/// Why:
///  every phase records through this one step,
///  so the three phases cannot disagree about what
/// "measured" means.
impl Frontier {
    /// Fold one measured bin into the frontier.
    ///
    /// What:
    ///  marks the bin measured,
    ///  counts it,
    ///  raises the peak if it is the loudest so far,
    ///  and queues it for the climb.
    ///  Why:
    ///  every phase records through this one step,
    ///  so the three phases cannot disagree about what "measured" means.
    fn record(&mut self, index: usize, bin_peak: f32) {
        self.measured[index] = true;
        self.used_bins += 1;
        if bin_peak > self.peak {
            self.peak = bin_peak;
        }
        self.loudest_first.push((bin_peak.to_bits(), index));
    }

    /// Whether the selection may still measure a bin.
    fn has_budget(&self) -> bool {
        return self.used_bins < self.budget_bins
    }
}

/// The bin the even pass measures at `step` of `count`,
///  over a span of `span` bins.
///
/// What:
///  a fraction of the span,
///  rounded to the nearest bin;
///  a single sample sits mid-track.
///  Why:
///  the first pass spreads over the whole track so it slights no region,
///  and one sample has no spread to keep,
///  so the middle is the only defensible place for it.
fn even_bin_at(step: usize, count: usize, span: usize) -> usize {
    if count <= 1 {
        return span / 2
    }
    return ((step as f64 / (count - 1) as f64) * span as f64).round() as usize
}

/// Measure the bins a probe visits and report what it read.
///
/// What:
///  seeds the bones hot slots with their neighbours,
///  measures an even pass over the remaining budget,
///  then climbs from the loudest measured bin until the budget is spent,
///  asking `bins` for one bin's peak each time it decides to measure one.
///  Why:
///  this is the one owner of the probe's selection:
///  the production probe hands it a decoder,
///  and the bench sidecar hands it a precomputed bin peak,
///  so corpus evidence and playback measure the same bins.
///
/// The budget is `coverage_fraction` of the bins,
///  at least one bin and never more than the grid has.
/// A bones slot at or past the last bin seeds that bin and its neighbour,
///  because a profile's slot resolution need not match the bin grid.
///
/// The error type is the caller's:
///  a decode or seek failure propagates untouched,
///  and a caller that cannot fail names its own impossible error.
pub fn select_probe_bins<Bins: ProbeBins>(
    grid: ProbeGrid<'_>,
    bins: &mut Bins,
) -> Result<ProbeSelection, Bins::Failure> {
    // The bin grid, floored at one bin so an empty grid still has a place to measure; the
    // budget is the coverage share of it, at least one bin and never more than the grid
    // holds, so a coverage above one cannot promise more bins than exist.
    let bin_count = grid.bin_count.max(1);
    let budget_bins = (((grid.coverage_fraction * bin_count as f64).floor() as usize).max(1)).min(bin_count);
    tracing::debug!(
        bin_count,
        budget_bins,
        bones = grid.bones_hot_bins.is_some(),
        "probe selection plan"
    );
    let mut frontier = Frontier {
        measured: vec![false; bin_count],
        loudest_first: BinaryHeap::new(),
        used_bins: 0,
        budget_bins,
        peak: 0.0,
    };
    // Phase one: bones seeds, each measured with its immediate neighbours.
    if let Some(hot_bins) = grid.bones_hot_bins {
        for &slot in hot_bins {
            // A slot past the last bin names the last bin rather than nothing.
            let center = slot.min(bin_count - 1);
            let first = center.saturating_sub(1);
            let last = (center + 1).min(bin_count - 1);
            for index in first..=last {
                if !frontier.measured[index] && frontier.has_budget() {
                    frontier.record(index, bins.bin_peak(index)?);
                }
            }
        }
    }
    // Phase two: the even pass, endpoints included, over whatever budget remains. The
    // pass count is a fraction of the bins, at least one, so a zero share still samples;
    // the last bin it can land on is `bin_count - 1`, the span its placement spreads over.
    let even_count = ((grid.even_coverage_fraction * bin_count as f64).round() as usize).max(1);
    for step in 0..even_count {
        if !frontier.has_budget() {
            break;
        }
        let index = even_bin_at(step, even_count, bin_count - 1);
        if !frontier.measured[index] {
            frontier.record(index, bins.bin_peak(index)?);
        }
    }
    // Phase three: the climb; measure the unmeasured neighbours of the loudest bin.
    while frontier.has_budget() {
        let Some((_, index)) = frontier.loudest_first.pop() else {
            break;
        };
        if index > 0 && !frontier.measured[index - 1] && frontier.has_budget() {
            frontier.record(index - 1, bins.bin_peak(index - 1)?);
        }
        if index + 1 < bin_count && !frontier.measured[index + 1] && frontier.has_budget() {
            frontier.record(index + 1, bins.bin_peak(index + 1)?);
        }
    }
    // The selection is done: how many bins were measured and the loudest one read.
    tracing::debug!(used_bins = frontier.used_bins, peak = frontier.peak, "probe selection done");
    return Ok(ProbeSelection { peak: frontier.peak, used_bins: frontier.used_bins })
}

/// What:
///  `#[cfg(test)] #[path = "probe_selection_tests.rs"] mod tests;`.
///  Test-only submodule in the sibling file,
///  gated to test builds.
/// Why:
///  keep this file to production code;
///  sibling `*_tests.rs` is max-lines exempt.
#[cfg(test)]
#[path = "probe_selection_tests.rs"]
mod tests;
