//! Fixtures shared by the bench's unit tests:
//!  corpus tracks whose bin arithmetic is exact,
//!  and the tolerance the level assertions compare with.
//!
//! What:
//!  a fixture track runs at `RATE` frames per second,
//!  one bin per second,
//!  and one peak per bin,
//!  so a window starting at frame `s` covers exactly the bins from second `s` on.
//! Why:
//!  the scoring math converts frames to seconds and seconds to bin indices;
//!  one frame per second keeps every one of those conversions exact,
//!  so a test's expectation reads off the fixture instead of re-deriving it.

/// Imports the corpus track record every fixture builds.
use crate::corpus::Track;

/// Frames per second in every fixture track:
///  one frame is one second,
///  so a window boundary always lands on a bin boundary.
pub(crate) const RATE: u32 = 1;

/// Seconds covered by each fixture bin,
///  one second at `RATE`,
///  so a bin index is also a second.
pub(crate) const BIN_SECONDS: f64 = 1.0;

/// The tolerance a level assertion allows,
///  far below the hundredth of a decibel the reports print.
pub(crate) const LEVEL_TOLERANCE: f64 = 1.0e-9;

/// Whether two levels agree within `LEVEL_TOLERANCE`.
pub(crate) fn levels_match(left: f64, right: f64) -> bool {
    return (left - right).abs() <= LEVEL_TOLERANCE
}

/// One fixture track of `duration_secs` seconds,
///  whose `bin_peaks` carry one peak per second.
pub(crate) fn track(path: &str, duration_secs: u64, full_peak: f32, bin_peaks: Vec<f32>) -> Track {
    return Track {
        path: path.to_owned(),
        duration_secs: duration_secs as f64,
        rate: RATE,
        decoded_frames: duration_secs * u64::from(RATE),
        full_peak,
        bin_seconds: BIN_SECONDS,
        bin_peaks,
    }
}

/// `count` bins that all carry `peak`,
///  so any window over them samples `peak`.
pub(crate) fn flat_bins(count: usize, peak: f32) -> Vec<f32> {
    return vec![peak; count]
}

/// `count` quiet bins carrying `quiet`,
///  with the bin at `hot_bin` carrying `hot`,
///  so a window that misses `hot_bin` samples only `quiet`.
pub(crate) fn bins_with_one_hot(count: usize, quiet: f32, hot_bin: usize, hot: f32) -> Vec<f32> {
    let mut bins = flat_bins(count, quiet);
    bins[hot_bin] = hot;
    return bins
}
