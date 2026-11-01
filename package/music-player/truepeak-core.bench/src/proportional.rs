//! The decided policy:
//!  a proportional-coverage probe with a fixed margin,
//!  plus the
//! margin-versus-clamp tradeoff.
//!
//! A fixed-length probe under-covers long tracks (a 45s probe is ~1% of an hour-long mix),
//! so the worst under-read is a long track.
//!  Proportional coverage instead probes a fixed
//! fraction of every long track,
//!  giving uniform coverage.
//!  The under-read distribution is
//! extremely skewed (median near 0.14 dB),
//!  so guaranteeing the last percent costs about a
//! decibel of margin on every track.
//!  The margin is therefore a decision:
//!  a smaller margin
//! keeps most tracks louder and lets the realtime clamp catch the rare too-loud transient,
//! which background warming later corrects to an exact cached gain.

/// Imports the corpus track record.
use crate::corpus::Track;
/// Imports the safe-provenance path set.
use std::collections::HashSet;
/// Imports the shared dB and policy math.
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

/// One long track's under-read and whether its provenance is reliably not hot.
#[derive(Clone, Copy, Debug)]
pub struct UnderRead {
    /// True peak above the loudest sampled window in dB.
    pub under_read_db: f64,
    /// Whether the track is lossless or yt-dlp (a smaller margin can serve it).
    pub safe: bool,
}

/// The loudest sampled window (linear) when a track is probed at `coverage` of its length
/// with windows of `window_secs`,
///  placed evenly across its bins.
fn sampled_max(track: &Track, coverage: f64, window_secs: f64) -> f64 {
    let bins = &track.bin_peaks;
    let n = bins.len();
    let window_bins = (window_secs / track.bin_seconds).round().max(1.0) as usize;
    let count = ((coverage * track.duration_secs) / window_secs).round().max(1.0) as usize;
    let span = n.saturating_sub(window_bins);
    // Evenly spaced window starts, each covering `window_bins` bins; take the loudest bin.
    return (0..count)
        .map(|index| {
            let start = if count <= 1 {
                span / 2
            } else {
                ((index as f64 / (count - 1) as f64) * span as f64).round() as usize
            };
            let hi = (start + window_bins).min(n);
            return bins.get(start..hi)
                .unwrap_or(&[])
                .iter()
                .fold(0.0_f32, |peak, &bin| return peak.max(bin))
        })
        .fold(0.0_f64, |peak, window| return peak.max(f64::from(window)))
}

/// Evaluate the proportional probe over the corpus.
///
/// What:
///  full-scans short tracks and probes a fraction of long tracks,
///  returning the total
/// decoded seconds and the under-read of every loud long track.
///  Why:
///  decoded seconds fix
/// the budget and the under-reads drive the margin/clamp decision.
pub fn evaluate_proportional(
    tracks: &[Track],
    short_scan_max_secs: f64,
    coverage: f64,
    window_secs: f64,
    ceiling_dbtp: f64,
    safe_paths: &HashSet<String>,
) -> (f64, Vec<UnderRead>) {
    // Fold each track into the running decoded total and, for loud long tracks, its under-read.
    let mut decoded = 0.0f64;
    let mut under_reads = Vec::new();
    for track in tracks {
        if track.duration_secs <= short_scan_max_secs {
            decoded += track.duration_secs;
            continue;
        }
        decoded += coverage * track.duration_secs;
        let full_db = db(f64::from(track.full_peak));
        if full_db <= ceiling_dbtp {
            continue;
        }
        let sampled = sampled_max(track, coverage, window_secs);
        under_reads.push(UnderRead {
            under_read_db: full_db - db(sampled),
            safe: safe_paths.contains(&track.path),
        });
    }
    return (decoded, under_reads)
}

/// The quantile value of a set of under-reads,
///  sorted ascending.
pub fn under_read_quantile(under_reads: &[UnderRead], fraction: f64) -> f64 {
    let mut values: Vec<f64> = under_reads.iter().map(|u| return u.under_read_db).collect();
    values.sort_by(f64::total_cmp);
    let index = ((values.len() as f64 - 1.0) * fraction).round() as usize;
    return values.get(index).copied().unwrap_or(0.0)
}

/// One row of the margin/clamp tradeoff table.
#[derive(Clone, Copy, Debug)]
pub struct MarginRow {
    /// The fixed margin in dB.
    pub margin_db: f64,
    /// Worst-case too-quiet error this margin causes (its negation).
    pub worst_quiet_db: f64,
    /// Loud long tracks whose under-read exceeds the margin (a clamped cold-start transient).
    pub clamped: usize,
    /// Of the clamped tracks,
    ///  how many are safe-provenance.
    pub clamped_safe: usize,
    /// Total loud long tracks.
    pub total: usize,
}

/// Build the margin/clamp tradeoff:
///  for each margin,
///  how many tracks the clamp must catch.
///
/// What:
///  a track is clamped when its under-read exceeds `margin + too_loud`.
///  Why:
///  this is
/// the decision surface,
///  worst-case too-quiet against the count of cold-start clamps.
pub fn margin_clamp_table(under_reads: &[UnderRead], margins: &[f64], too_loud_db: f64) -> Vec<MarginRow> {
    // For each margin, count the tracks whose under-read the fixed margin cannot cover.
    return margins
        .iter()
        .map(|&margin_db| {
            let clamped: Vec<&UnderRead> = under_reads
                .iter()
                .filter(|u| return u.under_read_db - margin_db > too_loud_db)
                .collect();
            return MarginRow {
                margin_db,
                worst_quiet_db: -margin_db,
                clamped: clamped.len(),
                clamped_safe: clamped.iter().filter(|u| return u.safe).count(),
                total: under_reads.len(),
            }
        })
        .collect()
}
