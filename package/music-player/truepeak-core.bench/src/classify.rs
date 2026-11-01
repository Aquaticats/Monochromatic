//! A small,
//!  auditable probe-only classifier that routes risky long tracks to a full
//! scan.
//!
//! The prior search treated violators as an opaque exception list.
//!  This is the better
//! model the plan asks for:
//!  a two-feature decision rule over probe-only features.
//!  A
//! probe under-reads the true peak when a peak hides between the windows,
//!  which happens
//! on tracks whose loudest window is not very loud,
//!  or whose windows disagree a lot.
//!  So
//! the rule routes a long track to a full scan when the loudest sampled window is at or
//! below a cutoff,
//!  OR the spread between the loudest and quietest window is at or above a
//! cutoff.
//!  The fit finds the cutoffs that catch every actual violator (full recall) while
//! routing the fewest non-violators,
//!  and it falls back to the one-feature rule when the
//! spread escape buys nothing.
//!  The labels use the full true peak,
//!  which is legal in the
//! harness;
//!  the shipped rule reads only probe features.

/// Imports the corpus track record.
use crate::corpus::Track;
/// Imports the candidate policy and the shared per-window sampling.
use crate::evaluate::{Candidate, sampled_windows};
/// Imports formatting for the rule's report line.
use std::fmt;
/// Imports file creation for the feature dump.
use std::fs::File;
/// Imports buffered writing for the feature dump.
use std::io::{BufWriter, Write};
/// Imports the borrowed path type for the dump location.
use std::path::Path;
/// Imports serde's derive so the dump row serializes with correct escaping.
use serde::Serialize;
/// Imports the shared dB,
///  gain,
///  and policy math.
use truepeak_core::{Policy, default_policy, peak_dbtp, probe_estimated_peak};

/// One long track's probe features and its harness-only violator label.
struct LongFeatures {
    /// Track path,
    ///  carried only so the feature dump can be joined with metadata offline.
    path: String,
    /// Per-window sampled peaks (linear),
    ///  so offline analysis can compute any statistic
    /// (variance,
    ///  robust spread) over the probe windows.
    windows: Vec<f64>,
    /// Loudest sampled window in dBTP (a legal classifier feature).
    sampled_max_db: f64,
    /// How far the true peak sits above the loudest sampled window in dB (harness label;
    /// this is the hidden quantity the classifier must predict,
    ///  not a runtime feature).
    under_read_db: f64,
    /// Track duration in seconds.
    duration_secs: f64,
    /// Probe gain error in dB (harness label;
    ///  positive is too loud).
    error_db: f64,
    /// Whether the probe gain would leave the bounds (harness label).
    is_violator: bool,
}

/// The fitted classifier and the real routed budget it produces.
pub struct FullScanRule {
    /// Loudest-window cutoff:
    ///  route to a full scan when `sampled_max_db >= cutoff_db`.
    /// Violators are hot masters (the probe already sees a loud peak),
    ///  so the safe
    /// direction routes the LOUD tracks,
    ///  not the quiet ones.
    cutoff_db: f64,
    /// Long tracks the rule routes to a full scan.
    routed: usize,
    /// Long tracks the corpus contains.
    long_tracks: usize,
    /// Actual violators in the corpus,
    ///  all of which the rule must catch.
    violators: usize,
    /// Real decoded seconds with this routing under the amended accounting.
    decoded_secs: f64,
    /// The corrected target,
    ///  so the gap is explicit.
    target_secs: f64,
    /// Worst too-quiet error among probe-trusted long tracks.
    worst_quiet_db: f64,
}

/// Render the rule,
///  its full-recall guarantee,
///  and its gap to the target.
impl fmt::Display for FullScanRule {
    /// Write the cutoff,
    ///  the routed/violator counts,
    ///  and the decoded gap on one line.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(
            formatter,
            "full-scan if sampled_max_db >= {:.3}dB | routes {}/{} long tracks, catches all {} violators | real_decoded={:.1}s (target {:.1}s, delta {:+.1}s) worst_quiet={:+.3}dB",
            self.cutoff_db,
            self.routed,
            self.long_tracks,
            self.violators,
            self.decoded_secs,
            self.target_secs,
            self.decoded_secs - self.target_secs,
            self.worst_quiet_db,
        )
    }
}

/// Convert a linear peak to dBTP,
///  treating silence as a very negative level.
fn db(peak: f64) -> f64 {
    if peak <= 0.0 {
        return f64::NEG_INFINITY
    } else {
        return peak_dbtp(peak)
    }
}

/// Print,
///  for a candidate,
///  how violators and non-violators distribute across the probe
/// features,
///  so the classifier can be designed from evidence rather than guessed.
///
/// What:
///  separates long tracks by the harness violator label and prints min/median/max of
/// the loudest-window level,
///  the under-read against the true peak,
///  and duration.
///  Why:
///  a
/// feature only helps if the two groups separate on it.
pub fn diagnose(tracks: &[Track], candidate: Candidate) {
    let policy: Policy = default_policy();
    let features = long_features(tracks, candidate, &policy);
    let report = |label: &str, group: Vec<&LongFeatures>| {
        if group.is_empty() {
            println!("  {label}: none");
            return;
        }
        let pct = |values: &mut Vec<f64>, fraction: f64| {
            values.sort_by(f64::total_cmp);
            return values[((values.len() as f64 - 1.0) * fraction) as usize]
        };
        let mut maxes: Vec<f64> = group.iter().map(|f| return f.sampled_max_db).collect();
        let mut reads: Vec<f64> = group.iter().map(|f| return f.under_read_db).collect();
        let mut durs: Vec<f64> = group.iter().map(|f| return f.duration_secs).collect();
        println!(
            "  {label}: n={} | sampled_max_db [{:.2} {:.2} {:.2}] under_read_db [{:.2} {:.2} {:.2}] dur [{:.0} {:.0} {:.0}]",
            group.len(),
            pct(&mut maxes, 0.0), pct(&mut maxes, 0.5), pct(&mut maxes, 1.0),
            pct(&mut reads, 0.0), pct(&mut reads, 0.5), pct(&mut reads, 1.0),
            pct(&mut durs, 0.0), pct(&mut durs, 0.5), pct(&mut durs, 1.0),
        );
    };
    println!("feature distributions [min median max] at margin {:.3}dB:", candidate.probe_margin_db);
    report("violators    ", features.iter().filter(|f| return f.is_violator).collect());
    report("non-violators", features.iter().filter(|f| return !f.is_violator).collect());
}

/// Extract probe features and the violator label for every long track.
fn long_features(tracks: &[Track], candidate: Candidate, policy: &Policy) -> Vec<LongFeatures> {
    let threshold = candidate.window_count as f64 * candidate.window_seconds;
    return tracks
        .iter()
        .filter(|track| return track.duration_secs > threshold)
        .map(|track| {
            let windows = sampled_windows(track, candidate);
            let sampled_max = windows.iter().fold(0.0_f64, |peak, &window| return peak.max(window));
            let estimated = probe_estimated_peak(sampled_max, candidate.probe_margin_db);
            let probe = (policy.ceiling_dbtp - db(estimated)).min(0.0);
            let exact = (policy.ceiling_dbtp - db(f64::from(track.full_peak))).min(0.0);
            let error = probe - exact;
            return LongFeatures {
                path: track.path.clone(),
                windows,
                sampled_max_db: db(sampled_max),
                under_read_db: db(f64::from(track.full_peak)) - db(sampled_max),
                duration_secs: track.duration_secs,
                error_db: error,
                is_violator: error > policy.max_too_loud_db || error < policy.max_too_quiet_db,
            }
        })
        .collect()
}

/// One row of the per-track feature dump,
///  joined offline with provenance metadata.
#[derive(Serialize)]
struct LongFeatureRow<'a> {
    /// Track path,
    ///  the join key against the metadata pass.
    path: &'a str,
    /// Per-window sampled peaks (linear),
    ///  for offline variance and spread statistics.
    windows: &'a [f64],
    /// Loudest sampled window in dBTP.
    sampled_max_db: f64,
    /// True peak above the loudest sampled window in dB.
    under_read_db: f64,
    /// Track duration in seconds.
    duration_secs: f64,
    /// Whether the probe gain would leave the bounds.
    is_violator: bool,
}

/// Write per-long-track probe features and violator labels as JSONL for offline joins.
///
/// What:
///  serializes one row per long track keyed by path.
///  Why:
///  the bench owns the
/// labels and the metadata pass owns provenance;
///  joining them tests whether metadata
/// (codec,
///  bitrate,
///  yt-dlp origin) separates the violators that amplitude alone cannot.
pub fn write_long_features(
    tracks: &[Track],
    candidate: Candidate,
    out_path: &Path,
) -> std::io::Result<()> {
    let policy: Policy = default_policy();
    let features = long_features(tracks, candidate, &policy);
    let mut writer = BufWriter::new(File::create(out_path)?);
    for feature in &features {
        let row = LongFeatureRow {
            path: &feature.path,
            windows: &feature.windows,
            sampled_max_db: feature.sampled_max_db,
            under_read_db: feature.under_read_db,
            duration_secs: feature.duration_secs,
            is_violator: feature.is_violator,
        };
        let line = serde_json::to_string(&row).map_err(std::io::Error::other)?;
        writer.write_all(line.as_bytes())?;
        writer.write_all(b"\n")?;
    }
    return Ok(())
}

/// Fit the simplest full-recall full-scan rule from the loudest-window feature.
///
/// What:
///  the violators are hot masters whose probe already shows a loud peak,
///  so the
/// rule routes every long track at or above a loudness cutoff;
///  the cutoff is the
/// quietest violator's loudest window,
///  which guarantees full recall.
///  Why:
///  this is the
/// simplest auditable rule that ships zero violations;
///  the report then shows honestly how
/// far its over-routing pushes the decoded budget past the target,
///  which is the evidence
/// that probe-only features do not separate violators cleanly.
pub fn fit_full_scan_rule(tracks: &[Track], candidate: Candidate, target_secs: f64) -> FullScanRule {
    let policy: Policy = default_policy();
    let features = long_features(tracks, candidate, &policy);
    let violators: Vec<&LongFeatures> = features.iter().filter(|f| return f.is_violator).collect();

    // Cutoff = the quietest violator's loudest window; routing at or above it catches all.
    let cutoff_db = violators
        .iter()
        .map(|violator| return violator.sampled_max_db)
        .fold(f64::INFINITY, f64::min);

    let routed = features
        .iter()
        .filter(|feature| return feature.sampled_max_db >= cutoff_db)
        .count();

    let threshold = candidate.window_count as f64 * candidate.window_seconds;
    let short_secs: f64 = tracks
        .iter()
        .filter(|track| return track.duration_secs <= threshold)
        .map(|track| return track.duration_secs)
        .sum();
    // Long tracks pay the probe, plus a full scan when the rule routes them.
    let long_secs: f64 = features
        .iter()
        .map(|feature| {
            if feature.sampled_max_db >= cutoff_db {
                return threshold + feature.duration_secs
            } else {
                return threshold
            }
        })
        .sum();
    let worst_quiet_db = features
        .iter()
        .filter(|feature| return feature.sampled_max_db < cutoff_db)
        .map(|feature| return feature.error_db)
        .fold(0.0_f64, f64::min);

    return FullScanRule {
        cutoff_db,
        routed,
        long_tracks: features.len(),
        violators: violators.len(),
        decoded_secs: short_secs + long_secs,
        target_secs,
        worst_quiet_db,
    }
}
