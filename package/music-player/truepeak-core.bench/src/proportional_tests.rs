//! Tests for the decided proportional-coverage policy:
//!  the coverage cost,
//!  the under-read it records,
//!  and the margin-versus-clamp tradeoff.

use super::*;
use crate::test_fixtures::{bins_with_one_hot, flat_bins, levels_match, track};
use truepeak_core::{default_policy, peak_dbtp};

/// The short-scan bound the policy uses,
///  so a track at or under it is decoded in full.
const SHORT_SCAN_MAX_SECS: f64 = 2.0;

/// The coverage fraction every probe in these tests takes.
const COVERAGE: f64 = 0.2;

/// Seconds per probe window,
///  one bin at the fixture's rate.
const WINDOW_SECS: f64 = 1.0;

/// Seconds in every long fixture track,
///  so its coverage cost is exactly `COVERAGE * LONG_SECS`.
const LONG_SECS: u64 = 100;

/// The peak every quiet second carries.
const QUIET_PEAK: f32 = 0.02;

/// The peak of the one loud second.
const HOT_PEAK: f32 = 1.0;

/// A second at or above the ceiling,
///  so the track has an under-read to record.
const LOUD_SEC: usize = 50;

/// The first second,
///  which the first probe window always covers.
const COVERED_SEC: usize = 0;

/// The policy ceiling,
///  which decides whether a track is loud enough to record an under-read.
fn ceiling_dbtp() -> f64 {
    return default_policy().ceiling_dbtp
}

/// Probe the fixture corpus at `COVERAGE` and `WINDOW_SECS`.
fn probe(tracks: &[Track], safe_paths: &HashSet<String>) -> (f64, Vec<UnderRead>) {
    return evaluate_proportional(tracks, SHORT_SCAN_MAX_SECS, COVERAGE, WINDOW_SECS, ceiling_dbtp(), safe_paths)
}

/// What:
///  a short track costs its whole length,
///  and a long track costs only its coverage.
#[test]
fn the_coverage_cost_is_a_full_scan_for_shorts_and_a_fraction_for_longs() {
    let short = track("short.flac", SHORT_SCAN_MAX_SECS as u64, HOT_PEAK, flat_bins(SHORT_SCAN_MAX_SECS as usize, HOT_PEAK));
    let long = track("long.flac", LONG_SECS, QUIET_PEAK, flat_bins(LONG_SECS as usize, QUIET_PEAK));
    let (decoded, under_reads) = probe(&[short, long], &HashSet::new());
    let expected = SHORT_SCAN_MAX_SECS + COVERAGE * LONG_SECS as f64;
    assert!(levels_match(decoded, expected), "decoded {decoded} is not {expected}");
    assert!(under_reads.is_empty(), "a track under the ceiling has no under-read");
}

/// What:
///  a loud long track records the level its probe missed,
///  and the provenance set marks it safe.
#[test]
fn a_loud_long_track_records_the_level_its_probe_missed() {
    let uncovered = track(
        "uncovered.flac",
        LONG_SECS,
        HOT_PEAK,
        bins_with_one_hot(LONG_SECS as usize, QUIET_PEAK, LOUD_SEC, HOT_PEAK),
    );
    let safe: HashSet<String> = HashSet::from([uncovered.path.clone()]);
    let (_, under_reads) = probe(&[uncovered], &safe);
    assert_eq!(under_reads.len(), 1);
    let under_read = under_reads[0];
    let expected = peak_dbtp(f64::from(HOT_PEAK)) - peak_dbtp(f64::from(QUIET_PEAK));
    assert!(levels_match(under_read.under_read_db, expected), "under-read {}", under_read.under_read_db);
    assert!(under_read.safe, "the provenance set marks the track safe");
}

/// What:
///  a loud second under a probe window costs nothing,
///  which is the whole point of placing windows evenly.
#[test]
fn a_loud_second_under_a_probe_window_has_no_under_read() {
    let covered = track(
        "covered.flac",
        LONG_SECS,
        HOT_PEAK,
        bins_with_one_hot(LONG_SECS as usize, QUIET_PEAK, COVERED_SEC, HOT_PEAK),
    );
    let (_, under_reads) = probe(&[covered], &HashSet::new());
    assert_eq!(under_reads.len(), 1);
    assert!(levels_match(under_reads[0].under_read_db, 0.0), "under-read {}", under_reads[0].under_read_db);
    assert!(!under_reads[0].safe, "no provenance set means unsafe");
}

/// What:
///  an empty corpus decodes nothing and records nothing.
#[test]
fn an_empty_corpus_decodes_and_records_nothing() {
    let (decoded, under_reads) = probe(&[], &HashSet::new());
    assert_eq!(decoded, 0.0);
    assert!(under_reads.is_empty());
}

/// What:
///  the quantile reads the sorted under-reads at the rounded index,
///  and answers zero for an empty set.
#[test]
fn the_quantile_reads_the_sorted_under_reads() {
    let under_reads: Vec<UnderRead> = [1.0, 2.0, 3.0, 4.0]
        .iter()
        .map(|&under_read_db| return UnderRead {
            under_read_db,
            safe: false,
        })
        .collect();
    assert_eq!(under_read_quantile(&under_reads, 0.0), 1.0);
    // Three gaps halved is 1.5, which rounds up to the third value rather than down.
    assert_eq!(under_read_quantile(&under_reads, 0.5), 3.0);
    // A fifth of three gaps is 0.6, which also rounds up to the second value.
    assert_eq!(under_read_quantile(&under_reads, 0.2), 2.0);
    assert_eq!(under_read_quantile(&under_reads, 1.0), 4.0);
    assert_eq!(under_read_quantile(&[], 0.5), 0.0);
}

/// What:
///  the tradeoff counts a track as clamped only when its under-read beats the
///  margin plus the too-loud bound,
/// and keeps the safe-provenance count beside it.
#[test]
fn the_margin_tradeoff_counts_clamped_tracks_and_their_provenance() {
    let under_reads = vec![
        UnderRead { under_read_db: 0.2, safe: true },
        UnderRead { under_read_db: 1.0, safe: false },
        UnderRead { under_read_db: 3.0, safe: true },
    ];
    let policy = default_policy();
    let margin_db = 0.5;
    let rows = margin_clamp_table(&under_reads, &[margin_db], policy.max_too_loud_db);
    assert_eq!(rows.len(), 1);
    let row = rows[0];
    assert_eq!(row.margin_db, margin_db);
    assert_eq!(row.total, under_reads.len());
    // Only the 3.0 dB under-read beats 0.5 dB of margin plus the policy's too-loud bound.
    assert_eq!(row.clamped, 1, "clamped count");
    assert_eq!(row.clamped_safe, 1, "the clamped track is safe-provenance");
    assert!(levels_match(row.worst_quiet_db, -margin_db), "worst quiet {}", row.worst_quiet_db);
}

/// What:
///  an empty under-read set still yields one row per margin,
///  with nothing clamped.
#[test]
fn the_margin_tradeoff_of_an_empty_set_clamps_nothing() {
    let policy = default_policy();
    let rows = margin_clamp_table(&[], &[0.5, 1.0], policy.max_too_loud_db);
    assert_eq!(rows.len(), 2);
    for row in rows {
        assert_eq!(row.clamped, 0);
        assert_eq!(row.clamped_safe, 0);
        assert_eq!(row.total, 0);
    }
}
