//! The bench's one bridge to the production probe selection.
//!
//! What:
//!  runs `truepeak_core::select_probe_bins` over a track's precomputed bin peaks.
//! Why:
//!  the bench scores a corpus offline,
//!  so its measurement is a lookup rather than a decode;
//!  which bins that lookup visits is the shipped selection,
//!  and a bench-only copy of it would let corpus evidence drift from playback.

/// Imports the production selection,
///  the grid it reads,
///  and what it reports.
use truepeak_core::{ProbeGrid, ProbeSelection, select_probe_bins};

/// Select the bins a probe measures over a track's precomputed bin peaks.
///
/// What:
///  hands the production selection a bin-peak slice,
///  which measures itself,
///  and answers the bins it spent.
///  Why:
///  a lookup cannot fail,
///  so the selection's impossible error is matched away rather than unwrapped.
pub fn select_over_bin_peaks(grid: ProbeGrid<'_>, bin_peaks: &[f32]) -> ProbeSelection {
    let mut lookup = bin_peaks;
    let selection = match select_probe_bins(grid, &mut lookup) {
        Ok(selection) => selection,
        Err(never) => match never {},
    };
    return selection
}
