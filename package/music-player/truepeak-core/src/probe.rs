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
//!
//! Which bins to visit is [the selection's](crate::probe_selection) decision;
//! this module supplies the measurement:
//!  a seek to the bin's start frame and one window's true peak.

/// Imports the fallible-source error.
use crate::error::TruePeakError;
/// Imports the shared streaming meter.
use crate::meter::TruePeakMeter;
/// Imports the one owner of the probe's bin selection,
///  and the measurement contract it asks.
use crate::probe_selection::{ProbeBins, ProbeGrid, select_probe_bins};
/// Imports the decoded-audio contract the probe drives.
use crate::source::TruePeakSource;

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

/// Measure bin `index` from the source.
///
/// What:
///  seeks to the bin's start frame and measures one window there.
///  Why:
///  the one fallible step the selection asks for,
///  once per bin it decides to measure;
///  the failures name the bin at debug before they propagate (the service fold warns on
///  the typed error).
fn measure_bin(
    source: &mut dyn TruePeakSource,
    channels: usize,
    plan: &ZoomPlan,
    index: usize,
) -> Result<f32, TruePeakError> {
    // Seek to the bin start and measure one window.
    let seek_result = source.seek_to_frame(index as u64 * plan.bin_frames);
    if let Err(error) = &seek_result {
        tracing::debug!(bin = index, cause = %error, "probe seek failed");
    }
    seek_result?;
    let measure_result = measure_window(source, channels, plan.bin_frames);
    if let Err(error) = &measure_result {
        tracing::debug!(bin = index, cause = %error, "probe measure failed");
    }
    return measure_result
}

/// The production probe's answer to the selection's question:
///  one decoded window at one bin.
///
/// What:
///  carries the decoder,
///  the channel count,
///  and the plan that says how wide one bin is.
///  Why:
///  the selection asks for bins one at a time,
///  so the measurement state travels as the one value it is handed.
struct SourceBins<'source, 'plan> {
    /// The decoder to seek and read.
    source: &'source mut dyn TruePeakSource,
    /// Channels in the decoded stream.
    channels: usize,
    /// The production plan:
    ///  bin width and the frames it spans.
    plan: &'plan ZoomPlan<'plan>,
}

/// What:
///  measuring a decoded bin is a seek and one window.
/// Why:
///  this is the only measurement the selection asks the production probe for.
impl ProbeBins for SourceBins<'_, '_> {
    /// A seek or a decode that failed.
    type Failure = TruePeakError;

    /// Seeks to the bin's start frame and measures one window there.
    fn bin_peak(&mut self, index: usize) -> Result<f32, TruePeakError> {
        return measure_bin(self.source, self.channels, self.plan, index)
    }
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
    // The selection decides which bins to measure; this source-backed measurement answers
    // the one question it asks.
    let mut bins = SourceBins { source, channels, plan };
    let selection = select_probe_bins(
        ProbeGrid {
            bin_count,
            coverage_fraction: plan.coverage_fraction,
            even_coverage_fraction: plan.even_coverage_fraction,
            bones_hot_bins: plan.bones_hot_bins,
        },
        &mut bins,
    )?;
    return Ok(selection.peak)
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
