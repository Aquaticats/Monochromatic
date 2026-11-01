//! The quarter-measure answer's probe:
//!  an even pass plus frontier zoom.
//!
//! The under-read of an evenly-placed probe is governed by how many bins sit near the
//! track's crest;
//!  the clamp tail is tracks where that count is one or two (needles).
//! Zooming cannot find a needle,
//!  but it climbs every heard hill:
//!  after a sparse even
//! pass,
//!  repeatedly decode the undecoded neighbors of the loudest decoded bin until the
//! track's bin budget is spent.
//!  That collapses the mid-tail (p90 under-read 0.63 -> 0.36
//! at the full quarter budget),
//!  so a much smaller margin buys the same clamp count and
//! every track keeps more loudness.
//!  Byte-rate profiles from container framing were also
//! measured as a needle locator and refuted (crest slots rank at the 60th byte-rank
//! percentile,
//!  worse than chance),
//!  so the probe stays purely decode-driven.

/// Imports the corpus track record.
use crate::corpus::Track;
/// Imports the max-heap driving the frontier expansion.
use std::collections::BinaryHeap;
/// Imports the safe-provenance path set for split-margin reporting.
use std::collections::HashSet;
/// Imports the shared dB conversion.
use truepeak_core::peak_dbtp;

/// Convert a linear peak to dBTP,
///  treating silence as a very negative level.
fn db(peak: f64) -> f64 {
    if peak <= 0.0 {
        return f64::NEG_INFINITY
    } else {
        return peak_dbtp(peak)
    }
}

/// One probed long track:
///  its true level,
///  its probe's level,
///  and its provenance.
#[derive(Clone, Copy, Debug)]
pub struct ZoomRow {
    /// Full-track true peak in dBTP.
    pub full_db: f64,
    /// Loudest decoded bin in dBTP under the zoom probe.
    pub probe_db: f64,
    /// Whether provenance is reliably not hot (lossless or yt-dlp).
    pub safe: bool,
}

/// Evenly spaced single-bin sample indices at `coverage` over `n` bins.
///
/// What:
///  the pass-one placement,
///  one bin every `1 / coverage` bins,
///  endpoints included.
/// Why:
///  mirrors the shipped even placement shape so pass one slights no region.
fn even_indices(n: usize, coverage: f64) -> Vec<usize> {
    // The count is proportional to coverage; a single sample sits mid-track.
    let count = ((coverage * n as f64).round() as usize).max(1);
    let span = n - 1;
    return (0..count)
        .map(|index| {
            if count <= 1 {
                return span / 2
            } else {
                return ((index as f64 / (count - 1) as f64) * span as f64).round() as usize
            }
        })
        .collect()
}

/// The zoom probe over one track:
///  even pass one,
///  then loudest-neighbor expansion.
///
/// What:
///  decodes `pass1_coverage` of the bins evenly,
///  then pops the loudest decoded bin
/// and decodes its undecoded neighbors,
///  repeating until `total_coverage` of the bins is
/// spent;
///  returns the loudest decoded bin (linear) and the decoded seconds.
/// Why:
///  local hills lead to crests;
///  spending the budget climbing them shrinks the
/// under-read far more than spreading it evenly.
fn zoom_probe(track: &Track, pass1_coverage: f64, total_coverage: f64) -> (f64, f64) {
    let bins = &track.bin_peaks;
    let n = bins.len();
    let budget_bins = ((total_coverage * n as f64).floor() as usize).max(1);
    let mut decoded = vec![false; n];
    // Peaks are non-negative, so the f32 bit pattern orders exactly like the value;
    // the heap holds (bits, index) and pops the loudest decoded bin first.
    let mut heap: BinaryHeap<(u32, usize)> = BinaryHeap::new();
    let mut used = 0usize;
    let mut peak = 0.0f32;
    // Decode a bin: mark, count, track the max, and enqueue it for expansion.
    // The closure parameters keep names of their own so none shadows the outer
    // frontier state it is handed by mutable reference.
    let decode = |index: usize,
                      decoded_flags: &mut Vec<bool>,
                      loud_heap: &mut BinaryHeap<(u32, usize)>,
                      used_count: &mut usize,
                      peak_value: &mut f32| {
        decoded_flags[index] = true;
        *used_count += 1;
        if bins[index] > *peak_value {
            *peak_value = bins[index];
        }
        loud_heap.push((bins[index].to_bits(), index));
    };
    for index in even_indices(n, pass1_coverage) {
        if used >= budget_bins {
            break;
        }
        if !decoded[index] {
            decode(index, &mut decoded, &mut heap, &mut used, &mut peak);
        }
    }
    while used < budget_bins {
        let Some((_, index)) = heap.pop() else {
            break;
        };
        if index > 0 && !decoded[index - 1] && used < budget_bins {
            decode(index - 1, &mut decoded, &mut heap, &mut used, &mut peak);
        }
        if index + 1 < n && !decoded[index + 1] && used < budget_bins {
            decode(index + 1, &mut decoded, &mut heap, &mut used, &mut peak);
        }
    }
    return (f64::from(peak), used as f64 * track.bin_seconds)
}

/// Evaluate the zoom policy over the corpus at the full decoded-seconds budget.
///
/// What:
///  full-scans short tracks,
///  spreads the remaining budget as one proportional
/// coverage over long tracks,
///  probes each with the zoom,
///  and returns the decoded total
/// plus one row per long track.
///  Why:
///  rows carry everything the margin decision needs.
pub fn evaluate_zoom(
    tracks: &[Track],
    short_scan_max_secs: f64,
    pass1_coverage: f64,
    budget_secs: f64,
    safe_paths: &HashSet<String>,
) -> (f64, Vec<ZoomRow>) {
    // Split the corpus; shorts are exact and free of error, longs share the budget.
    let short_secs: f64 = tracks
        .iter()
        .filter(|track| return track.duration_secs <= short_scan_max_secs)
        .map(|track| return track.duration_secs)
        .sum();
    let long_secs: f64 = tracks
        .iter()
        .filter(|track| return track.duration_secs > short_scan_max_secs)
        .map(|track| return track.duration_secs)
        .sum();
    // A hair under the exact fraction so per-track floor rounding stays inside budget.
    let coverage_epsilon = 0.0001;
    let total_coverage = (budget_secs - short_secs) / long_secs - coverage_epsilon;
    let mut decoded = short_secs;
    let mut rows = Vec::new();
    for track in tracks {
        if track.duration_secs <= short_scan_max_secs {
            continue;
        }
        let (peak, used_secs) = zoom_probe(track, pass1_coverage, total_coverage);
        decoded += used_secs;
        rows.push(ZoomRow {
            full_db: db(f64::from(track.full_peak)),
            probe_db: db(peak),
            safe: safe_paths.contains(&track.path),
        });
    }
    return (decoded, rows)
}

/// One margin's outcome over the zoom rows:
///  the three measures of the letter.
#[derive(Clone, Copy, Debug)]
pub struct ZoomMeasures {
    /// Loud long tracks whose under-read exceeds `margin + too_loud` (cold-start clamps).
    pub clamped: usize,
    /// Of the clamped tracks,
    ///  how many are safe-provenance.
    pub clamped_safe: usize,
    /// Mean needless attenuation across `all_tracks` tracks in dB.
    pub avg_quiet_db: f64,
    /// Worst needless attenuation in dB.
    pub worst_quiet_db: f64,
    /// Worst true overshoot above the ceiling after gain,
    ///  in dB (the clamp catches it).
    pub worst_over_db: f64,
}

/// Measure one margin assignment over the rows (margin may depend on provenance).
///
/// What:
///  applies attenuate-only gain from `probe + margin` toward the ceiling and
/// accumulates the letter's three measures;
///  `all_tracks` divides the quiet average so
/// exact short tracks count as zero error.
///  Why:
///  this is the decision surface the
/// answer reports.
pub fn measure_zoom(
    rows: &[ZoomRow],
    margin_for: &dyn Fn(&ZoomRow) -> f64,
    too_loud_db: f64,
    ceiling_dbtp: f64,
    all_tracks: usize,
) -> ZoomMeasures {
    // Fold every row into the three running measures.
    let mut clamped = 0usize;
    let mut clamped_safe = 0usize;
    let mut quiet_sum = 0.0f64;
    let mut worst_quiet = 0.0f64;
    let mut worst_over = 0.0f64;
    for row in rows {
        let margin = margin_for(row);
        if row.full_db > ceiling_dbtp && row.full_db - row.probe_db - margin > too_loud_db {
            clamped += 1;
            if row.safe {
                clamped_safe += 1;
            }
        }
        let necessary = (row.full_db - ceiling_dbtp).max(0.0);
        let applied = (row.probe_db + margin - ceiling_dbtp).max(0.0);
        quiet_sum += (applied - necessary).max(0.0);
        worst_quiet = worst_quiet.max((applied - necessary).max(0.0));
        worst_over = worst_over.max((necessary - applied).max(0.0));
    }
    return ZoomMeasures {
        clamped,
        clamped_safe,
        avg_quiet_db: quiet_sum / all_tracks.max(1) as f64,
        worst_quiet_db: worst_quiet,
        worst_over_db: worst_over,
    }
}

/// The under-read quantile over the loud long rows,
///  sorted ascending.
pub fn zoom_under_read_quantile(rows: &[ZoomRow], ceiling_dbtp: f64, fraction: f64) -> f64 {
    let mut values: Vec<f64> = rows
        .iter()
        .filter(|row| return row.full_db > ceiling_dbtp)
        .map(|row| return row.full_db - row.probe_db)
        .collect();
    values.sort_by(f64::total_cmp);
    let index = ((values.len() as f64 - 1.0) * fraction).round() as usize;
    return values.get(index).copied().unwrap_or(0.0)
}
