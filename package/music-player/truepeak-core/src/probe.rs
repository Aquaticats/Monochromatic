//! The frontier-zoom probe:
//!  measure a sparse even pass,
//!  then climb every heard hill.
//!
//! A probe's under-read is set by how many short windows anywhere in the track come
//! near its crest;
//!  most misses land on the shoulder of a loud passage.
//!  The climb turns
//! every measured loud window into a path:
//!  repeatedly measure the windows on either
//! side of the loudest window measured so far,
//!  until the track's bin budget is spent.
//! Optional bones seeds (byte-rate hot slots from a lossless file's framing) start the
//! climb on the right hills at a fraction of the coverage.

/// Imports the fallible-source error.
use crate::error::TruePeakError;
/// Imports the shared streaming meter.
use crate::meter::TruePeakMeter;
/// Imports the decoded-audio contract the probe drives.
use crate::source::TruePeakSource;
/// Imports the max-heap driving the frontier expansion.
use std::collections::BinaryHeap;

/// One track's zoom plan:
///  the bin grid and the coverage the probe may spend.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ZoomPlan<'hot> {
    /// Frames per probe bin (the policy's probe window).
    pub bin_frames: u64,
    /// The track's total frames;
    ///  bins tile it,
    ///  the last bin may be short.
    pub total_frames: u64,
    /// Fraction of bins the probe may measure in total.
    pub coverage_fraction: f64,
    /// Fraction of bins the even pass measures before the climb.
    pub even_coverage_fraction: f64,
    /// Byte-rate hot slots to seed (lossless bones),
    ///  each measured with its neighbors.
    pub bones_hot_bins: Option<&'hot [usize]>,
}

/// Measure one window from the source's current position with a window-local meter.
///
/// What:
///  reads up to `window_frames` frames and returns their true peak.
///  Why:
///  each
/// window gets its own meter so a seek seam cannot fabricate an inter-sample spike.
pub(crate) fn measure_window(
    source: &mut dyn TruePeakSource,
    channels: usize,
    window_frames: u64,
) -> Result<f32, TruePeakError> {
    // A fresh meter per window; feed chunks until the window is full or EOF.
    let mut meter = TruePeakMeter::new(channels);
    let mut fed: u64 = 0;
    while fed < window_frames {
        let chunk = source.next_chunk()?;
        if chunk.is_empty() {
            break;
        }
        fed += chunk.len() as u64 / channels as u64;
        meter.feed(&chunk);
    }
    return Ok(meter.peak())
}

/// The frontier state one zoom run threads through its measurements.
struct Frontier {
    /// Whether each bin was measured already.
    measured: Vec<bool>,
    /// Measured bins by loudness;
    ///  non-negative f32 bits order exactly like the values.
    heap: BinaryHeap<(u32, usize)>,
    /// Bins measured so far.
    used: usize,
    /// Loudest window peak seen.
    peak: f32,
}

/// Measure bin `index` and fold it into the frontier.
///
/// What:
///  seeks to the bin's start frame,
///  measures one window,
///  records it.
///  Why:
///  the one
/// fallible step every phase (bones,
///  even,
///  climb) shares.
fn measure_bin(
    source: &mut dyn TruePeakSource,
    channels: usize,
    plan: &ZoomPlan,
    frontier: &mut Frontier,
    index: usize,
) -> Result<(), TruePeakError> {
    // Seek to the bin start and measure one window; a failure names the bin at debug before
    // it propagates (the service fold warns on the typed error).
    source
        .seek_to_frame(index as u64 * plan.bin_frames)
        .inspect_err(|error| tracing::debug!(bin = index, cause = %error, "probe seek failed"))?;
    let window_peak = measure_window(source, channels, plan.bin_frames)
        .inspect_err(|error| tracing::debug!(bin = index, cause = %error, "probe measure failed"))?;
    frontier.measured[index] = true;
    frontier.used += 1;
    if window_peak > frontier.peak {
        frontier.peak = window_peak;
    }
    frontier.heap.push((window_peak.to_bits(), index));
    return Ok(())
}

/// Run the zoom probe and return the loudest measured window peak.
///
/// What:
///  bones seeds (each with neighbors),
///  then the even pass,
///  then the climb,
///  all
/// capped by the bin budget.
///  Why:
///  this is the probe the resolver uses for long tracks.
pub(crate) fn zoom_probe(
    source: &mut dyn TruePeakSource,
    channels: usize,
    plan: &ZoomPlan,
) -> Result<f32, TruePeakError> {
    // A function-scoped span tags every event below with this function name; the guard
    // drops on every return path. Mirrors the TS per-function tagged logger.
    let span = tracing::debug_span!("zoom_probe");
    let _guard = span.enter();
    // The bin grid: the last bin may be short; measure_window stops at EOF anyway.
    let bin_count = (plan.total_frames.div_ceil(plan.bin_frames.max(1)).max(1)) as usize;
    let budget = (((plan.coverage_fraction * bin_count as f64).floor() as usize).max(1)).min(bin_count);
    // The plan the three phases spend: total bins and the measurement budget.
    tracing::debug!(bin_count, budget, bones = plan.bones_hot_bins.is_some(), "zoom plan");
    let mut frontier = Frontier {
        measured: vec![false; bin_count],
        heap: BinaryHeap::new(),
        used: 0,
        peak: 0.0,
    };
    // Phase one: bones seeds, each measured with its immediate neighbors.
    if let Some(hot_bins) = plan.bones_hot_bins {
        for &slot in hot_bins {
            let center = slot.min(bin_count - 1);
            let lo = center.saturating_sub(1);
            let hi = (center + 1).min(bin_count - 1);
            for index in lo..=hi {
                if !frontier.measured[index] && frontier.used < budget {
                    measure_bin(source, channels, plan, &mut frontier, index)?;
                }
            }
        }
    }
    // Phase two: the even pass, endpoints included, over whatever budget remains.
    let even_count = ((plan.even_coverage_fraction * bin_count as f64).round() as usize).max(1);
    // The index range the even pass spreads over; named apart from the tracing
    // `span` above so neither binding shadows.
    let even_span = bin_count - 1;
    for step in 0..even_count {
        if frontier.used >= budget {
            break;
        }
        let index = if even_count <= 1 {
            even_span / 2
        } else {
            ((step as f64 / (even_count - 1) as f64) * even_span as f64).round() as usize
        };
        if !frontier.measured[index] {
            measure_bin(source, channels, plan, &mut frontier, index)?;
        }
    }
    // Phase three: the climb; measure the unmeasured neighbors of the loudest bin.
    while frontier.used < budget {
        let Some((_, index)) = frontier.heap.pop() else {
            break;
        };
        if index > 0 && !frontier.measured[index - 1] && frontier.used < budget {
            measure_bin(source, channels, plan, &mut frontier, index - 1)?;
        }
        if index + 1 < bin_count && !frontier.measured[index + 1] && frontier.used < budget {
            measure_bin(source, channels, plan, &mut frontier, index + 1)?;
        }
    }
    // The zoom is done: how many bins were measured and the loudest window found.
    tracing::debug!(used = frontier.used, peak = frontier.peak, "zoom done");
    return Ok(frontier.peak)
}

/// What:
///      `#[cfg(test)] #[path = "probe_tests.rs"] mod tests;`.
///  Test-only submodule in
///           the sibling file,
///  gated to test builds.
/// Why:
///       Keep this file to production code;
///  sibling `*_tests.rs` is max-lines exempt.
#[cfg(test)]
#[path = "probe_tests.rs"]
mod tests;
