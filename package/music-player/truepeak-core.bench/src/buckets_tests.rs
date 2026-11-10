//! Tests for the bucket-first composite:
//!  the per-provenance coverage and margin it charges,
//!  the extension fallback for an untagged track,
//!  and the bones-seeded probe.

use super::*;
use crate::test_fixtures::{BIN_SECONDS, bins_with_one_hot, flat_bins, levels_match, track};
use truepeak_core::{default_policy, peak_dbtp};

/// Seconds in every long fixture track,
///  past the shipped short-scan bound.
const LONG_SECS: u64 = 200;

/// The bare peak a flat fixture track carries.
const FLAT_PEAK: f32 = 0.5;

/// The quiet peak of the bones fixture track.
const QUIET_PEAK: f32 = 0.02;

/// The peak of the one hot slot.
const HOT_PEAK: f32 = 1.0;

/// The slot the bones profile points at.
const HOT_SLOT: usize = 10;

/// A flat long track of `FLAT_PEAK` under `path`.
fn flat_track(path: &str) -> Track {
    return track(path, LONG_SECS, FLAT_PEAK, flat_bins(LONG_SECS as usize, FLAT_PEAK))
}

/// The seconds one long track's probe decodes under `bucket`,
///  the coverage the shipped table assigns it times the bins it has.
fn probe_secs(policy: &Policy, bucket: Bucket, bones_present: bool) -> f64 {
    let coverage = bucket_probe(policy, bucket, bones_present).coverage_fraction;
    let decoded_bins = (coverage * LONG_SECS as f64).floor().max(1.0);
    return decoded_bins * BIN_SECONDS;
}

/// What:
///  every long track is probed at its own bucket's coverage,
///  and its row carries that bucket's margin.
#[test]
fn each_long_track_pays_its_bucket_coverage_and_carries_its_margin() {
    let policy = default_policy();
    let tagged = [
        ("bones.flac", Bucket::Flac),
        ("plain.flac", Bucket::Flac),
        ("store.m4a", Bucket::Store),
        ("purl.opus", Bucket::Purl),
        ("bare.mp3", Bucket::Bare),
    ];
    let buckets: HashMap<String, Bucket> = tagged
        .iter()
        .map(|&(path, bucket)| return (path.to_owned(), bucket))
        .collect();
    let bones: HashMap<String, Vec<usize>> = HashMap::from([("bones.flac".to_owned(), vec![HOT_SLOT])]);
    let short_secs = policy.short_scan_max_secs as u64;
    let mut tracks = vec![track("short.flac", short_secs, FLAT_PEAK, flat_bins(short_secs as usize, FLAT_PEAK))];
    for (path, _bucket) in tagged {
        tracks.push(flat_track(path));
    }

    let (decoded, rows) = evaluate_buckets(&tracks, &policy, &buckets, &bones);
    assert_eq!(rows.len(), tagged.len(), "one row per long track");
    let mut expected = policy.short_scan_max_secs;
    for (index, &(path, bucket)) in tagged.iter().enumerate() {
        let bones_present = bones.contains_key(path);
        expected += probe_secs(&policy, bucket, bones_present);
        let row = &rows[index];
        assert_eq!(row.bucket, bucket, "{path} bucket");
        assert_eq!(
            row.margin_db,
            bucket_probe(&policy, bucket, bones_present).probe_margin_db,
            "{path} margin",
        );
        assert!(levels_match(row.full_db, peak_dbtp(f64::from(FLAT_PEAK))), "{path} full level");
        assert!(levels_match(row.probe_db, peak_dbtp(f64::from(FLAT_PEAK))), "{path} probe level");
    }
    assert!(levels_match(decoded, expected), "decoded {decoded}, expected {expected}");
}

/// What:
///  a track the tag sweep missed takes its bucket from the file extension.
#[test]
fn a_track_missing_from_the_tag_sweep_falls_back_to_its_extension() {
    let policy = default_policy();
    let tracks = [flat_track("untagged.flac"), flat_track("untagged.mp3")];
    let (_, rows) = evaluate_buckets(&tracks, &policy, &HashMap::new(), &HashMap::new());
    assert_eq!(rows[0].bucket, Bucket::Flac, "a .flac path is lossless");
    assert_eq!(rows[1].bucket, Bucket::Bare, "an untagged lossy path is the risk bucket");
}

/// What:
///  a bones profile seeds the hot slot,
///  so the probe sees the crest an even pass would have to climb to.
#[test]
fn a_bones_profile_seeds_the_hot_slot() {
    let policy = default_policy();
    let path = "hot.flac";
    let bins = bins_with_one_hot(LONG_SECS as usize, QUIET_PEAK, HOT_SLOT, HOT_PEAK);
    let tracks = [track(path, LONG_SECS, HOT_PEAK, bins)];
    let buckets = HashMap::from([(path.to_owned(), Bucket::Flac)]);
    let bones = HashMap::from([(path.to_owned(), vec![HOT_SLOT])]);
    let (decoded, rows) = evaluate_buckets(&tracks, &policy, &buckets, &bones);
    assert_eq!(rows.len(), 1);
    assert!(levels_match(rows[0].probe_db, peak_dbtp(f64::from(HOT_PEAK))), "probe {}", rows[0].probe_db);
    assert!(levels_match(rows[0].full_db, peak_dbtp(f64::from(HOT_PEAK))), "full {}", rows[0].full_db);
    assert!(
        levels_match(decoded, probe_secs(&policy, Bucket::Flac, true)),
        "the bones probe decodes only its budget, got {decoded}",
    );
}

/// What:
///  an empty corpus decodes nothing and reports no row.
#[test]
fn an_empty_corpus_decodes_nothing() {
    let (decoded, rows) = evaluate_buckets(&[], &default_policy(), &HashMap::new(), &HashMap::new());
    assert_eq!(decoded, 0.0);
    assert!(rows.is_empty());
}
