//! Tests for the frontier-zoom probe:
//!  its pass-one placement,
//!  the decoded-seconds accounting,
//!  and the three measures a margin is judged by.

use super::*;
use crate::test_fixtures::{flat_bins, levels_match, track};
use truepeak_core::{default_policy, peak_dbtp};

/// The short-scan bound every fixture splits on.
const SHORT_SCAN_MAX_SECS: f64 = 2.0;

/// The pass-one coverage every fixture probes with.
const PASS1_COVERAGE: f64 = 0.1;

/// Seconds in every long fixture track,
///  so a coverage fraction of a tenth is ten seconds.
const LONG_SECS: u64 = 100;

/// The seconds of long-track audio the budget pays for,
///  above the tenth pass one spends so the climb has budget of its own.
const LONG_BUDGET_SECS: f64 = 15.0;

/// The peak every bin of a flat fixture track carries.
const FLAT_PEAK: f32 = 0.5;

/// The quiet peak of a needle fixture track.
const QUIET_PEAK: f32 = 0.02;

/// The peak of the one loud bin.
const HOT_PEAK: f32 = 1.0;

/// The peak of the loud bin a hill climbs through,
///  low enough that pass one alone misses the crest beside it.
const HILL_PEAK: f32 = 0.5;

/// The bins the zoom probe decodes for `LONG_SECS` at `LONG_BUDGET_SECS`:
///  the coverage fraction of the bins,
/// floored.
fn expected_decoded_bins() -> usize {
    let total_coverage = LONG_BUDGET_SECS / LONG_SECS as f64 - 0.0001;
    return (total_coverage * LONG_SECS as f64).floor() as usize;
}

/// A flat long track the zoom probe samples at `FLAT_PEAK`.
fn flat_track() -> Track {
    return track("flat.flac", LONG_SECS, FLAT_PEAK, flat_bins(LONG_SECS as usize, FLAT_PEAK))
}

/// What:
///  the short tracks are decoded in full,
///  the long track costs exactly the bins its budget buys,
/// and its row carries the true level and the sampled level.
#[test]
fn the_zoom_pass_charges_shorts_in_full_and_longs_by_their_bins() {
    let short_secs = SHORT_SCAN_MAX_SECS as u64;
    let short = track("short.flac", short_secs, HOT_PEAK, flat_bins(short_secs as usize, HOT_PEAK));
    let long = flat_track();
    let budget_secs = SHORT_SCAN_MAX_SECS + LONG_BUDGET_SECS;
    let (decoded, rows) = evaluate_zoom(&[short, long], SHORT_SCAN_MAX_SECS, PASS1_COVERAGE, budget_secs, &HashSet::new());
    let expected = SHORT_SCAN_MAX_SECS + expected_decoded_bins() as f64;
    assert!(levels_match(decoded, expected), "decoded {decoded}, expected {expected}");
    assert_eq!(rows.len(), 1, "only long tracks get a row");
    assert!(levels_match(rows[0].full_db, peak_dbtp(f64::from(FLAT_PEAK))), "full level {}", rows[0].full_db);
    assert!(levels_match(rows[0].probe_db, peak_dbtp(f64::from(FLAT_PEAK))), "probe level {}", rows[0].probe_db);
    assert!(!rows[0].safe, "no provenance set means unsafe");
}

/// What:
///  a crest next to the loudest pass-one bin is reached by the climb,
///  so the zoom sees more than an even pass alone would.
#[test]
fn the_climb_reaches_the_crest_beside_the_loudest_pass_one_bin() {
    let mut bins = flat_bins(LONG_SECS as usize, QUIET_PEAK);
    bins[0] = HILL_PEAK;
    bins[1] = HOT_PEAK;
    let hill = track("hill.flac", LONG_SECS, HOT_PEAK, bins);
    let (_, rows) = evaluate_zoom(&[hill], SHORT_SCAN_MAX_SECS, PASS1_COVERAGE, LONG_BUDGET_SECS, &HashSet::new());
    assert_eq!(rows.len(), 1);
    assert!(levels_match(rows[0].full_db, peak_dbtp(f64::from(HOT_PEAK))), "full {}", rows[0].full_db);
    assert!(
        levels_match(rows[0].probe_db, peak_dbtp(f64::from(HOT_PEAK))),
        "the climb must reach the crest at 1.0, got {}",
        rows[0].probe_db,
    );
    assert!(rows[0].probe_db > peak_dbtp(f64::from(HILL_PEAK)), "an even pass alone would stop at the hill");
}

/// What:
///  a corpus of short tracks decodes exactly its own length and yields no row.
#[test]
fn a_corpus_of_short_tracks_decodes_its_length_and_yields_no_row() {
    let short = track("short.flac", SHORT_SCAN_MAX_SECS as u64, HOT_PEAK, flat_bins(SHORT_SCAN_MAX_SECS as usize, HOT_PEAK));
    let (decoded, rows) = evaluate_zoom(&[short], SHORT_SCAN_MAX_SECS, PASS1_COVERAGE, LONG_BUDGET_SECS, &HashSet::new());
    assert!(levels_match(decoded, SHORT_SCAN_MAX_SECS), "decoded {decoded}");
    assert!(rows.is_empty(), "no long track means no row");
}

/// What:
///  pass one samples the bins evenly with both endpoints,
///  and a single sample sits mid-track.
#[test]
fn pass_one_samples_evenly_with_both_endpoints() {
    let count = (PASS1_COVERAGE * LONG_SECS as f64).round() as usize;
    let indices = even_indices(LONG_SECS as usize, PASS1_COVERAGE);
    assert_eq!(indices.len(), count);
    assert_eq!(indices[0], 0, "the pass starts at the first bin");
    assert_eq!(indices[count - 1], LONG_SECS as usize - 1, "the pass ends at the last bin");
    assert!(indices.windows(2).all(|pair| return pair[0] < pair[1]), "indices ascend: {indices:?}");

    let single = even_indices(LONG_SECS as usize, 0.0);
    assert_eq!(single.len(), 1, "coverage zero still samples once");
    assert_eq!(single[0], (LONG_SECS as usize - 1) / 2, "a single sample sits mid-track");
    assert_eq!(even_indices(1, PASS1_COVERAGE), vec![0], "one bin has one place to be");
}

/// What:
///  the measures count a clamp only when the margin cannot cover the under-read,
///  and divide the quiet error by every track in the corpus.
#[test]
fn the_measures_count_clamps_and_average_over_every_track() {
    let policy = default_policy();
    let rows = vec![
        ZoomRow { full_db: 0.0, probe_db: -3.0, safe: true },
        ZoomRow { full_db: -0.5, probe_db: -2.0, safe: false },
        ZoomRow { full_db: -2.0, probe_db: -3.0, safe: false },
        ZoomRow { full_db: policy.ceiling_dbtp, probe_db: policy.ceiling_dbtp, safe: false },
    ];
    let margin_db = 0.5;
    let all_tracks = 4;
    let measures = measure_zoom(&rows, &|_| return margin_db, policy.max_too_loud_db, policy.ceiling_dbtp, all_tracks);
    assert_eq!(measures.clamped, 2, "the two rows whose under-read beats the margin");
    assert_eq!(measures.clamped_safe, 1, "one clamped row is safe-provenance");
    // The fourth row is needlessly attenuated by the margin alone, over four corpus tracks.
    assert!(levels_match(measures.avg_quiet_db, margin_db / all_tracks as f64), "average {}", measures.avg_quiet_db);
    assert!(levels_match(measures.worst_quiet_db, margin_db), "worst quiet {}", measures.worst_quiet_db);
    assert!(levels_match(measures.worst_over_db, -policy.ceiling_dbtp), "worst over {}", measures.worst_over_db);
}

/// What:
///  the margin applies per row,
///  so a provenance split can quiet one group without moving the other.
#[test]
fn a_per_row_margin_covers_only_the_group_it_names() {
    let policy = default_policy();
    let rows = vec![
        ZoomRow { full_db: 0.0, probe_db: -3.0, safe: true },
        ZoomRow { full_db: -0.5, probe_db: -2.0, safe: false },
    ];
    let safe_margin_db = 3.0;
    let unsafe_margin_db = 0.5;
    let measures = measure_zoom(
        &rows,
        &|row| {
            if row.safe {
                return safe_margin_db
            } else {
                return unsafe_margin_db
            }
        },
        policy.max_too_loud_db,
        policy.ceiling_dbtp,
        rows.len(),
    );
    assert_eq!(measures.clamped, 1, "the wide safe margin clears the safe row only");
    assert_eq!(measures.clamped_safe, 0);
}

/// What:
///  an empty row set measures nothing.
#[test]
fn an_empty_row_set_measures_nothing() {
    let policy = default_policy();
    let measures = measure_zoom(&[], &|_| return 0.5, policy.max_too_loud_db, policy.ceiling_dbtp, 0);
    assert_eq!(measures.clamped, 0);
    assert_eq!(measures.clamped_safe, 0);
    assert_eq!(measures.avg_quiet_db, 0.0);
    assert_eq!(measures.worst_quiet_db, 0.0);
    assert_eq!(measures.worst_over_db, 0.0);
}

/// What:
///  the under-read quantile reads the loud long rows only,
///  sorted ascending,
/// and answers zero for an empty set.
#[test]
fn the_under_read_quantile_reads_the_loud_rows_only() {
    let policy = default_policy();
    let rows = vec![
        ZoomRow { full_db: 0.0, probe_db: -2.0, safe: false },
        ZoomRow { full_db: -0.5, probe_db: -1.0, safe: false },
        ZoomRow { full_db: policy.ceiling_dbtp - 4.0, probe_db: policy.ceiling_dbtp - 9.0, safe: false },
    ];
    assert!(levels_match(zoom_under_read_quantile(&rows, policy.ceiling_dbtp, 0.0), 0.5));
    // The single gap halved rounds up to the second value rather than down.
    assert!(levels_match(zoom_under_read_quantile(&rows, policy.ceiling_dbtp, 0.5), 2.0));
    assert!(levels_match(zoom_under_read_quantile(&rows, policy.ceiling_dbtp, 1.0), 2.0));
    assert!(levels_match(zoom_under_read_quantile(&[], policy.ceiling_dbtp, 0.5), 0.0));
}
