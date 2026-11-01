// What:     Unit tests for `measure_peek.rs`, pulled in by `#[cfg(test)] #[path = "measure_peek_tests.rs"] mod tests;`.
// Why:      The peek must give the header length for an unmeasured track (positive control), must keep a
//           sampled cache hit out of the rows, and must return one measure per track in queue order.

// What:     `use super::*;` brings the peek functions and their imports into the test scope.
// Why:      The tests call `peek_measure` and `peek_measures` directly.
use super::*;

use std::time::Duration;

use truepeak_core::{Decision, DecisionKind};

use crate::peakcache::{self, CacheHandle};

// What:     `fn fixture(name: &str) -> PathBuf` names a committed fixture file under the package.
// Why:      The tones have a known header length, so the peek can be checked against a real file.
fn fixture(name: &str) -> PathBuf {
    return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixture").join(name);
}

#[test]
fn unmeasured_track_gets_its_header_length_and_no_peak() {
    let cache = CacheHandle::open_degraded();
    let measure = peek_measure(&fixture("tone.wav"), &cache);
    assert!(measure.duration_secs.is_some_and(|secs| return secs > 0.0));
    assert_eq!(measure.peak_dbtp, None);
    assert!(!measure.peak_exact);
}

#[test]
fn missing_track_gives_an_empty_measure() {
    let cache = CacheHandle::open_degraded();
    let measure = peek_measure(&fixture("missing.flac"), &cache);
    assert_eq!(measure, TrackMeasure::default());
}

#[test]
fn peek_measures_keeps_queue_order_and_pairs_each_path() {
    let cache = CacheHandle::open_degraded();
    let tracks = vec![fixture("tone.wav"), fixture("missing.flac")];
    let peeked = peek_measures(&tracks, &cache);
    assert_eq!(peeked.len(), 2);
    assert_eq!(peeked[0].0, tracks[0]);
    assert_eq!(peeked[1].0, tracks[1]);
    assert_eq!(peeked[1].1, TrackMeasure::default());
}

#[test]
fn cached_exact_decision_supplies_the_peak_and_its_length() {
    let db = crate::test_scratch::scratch_root().join(format!("mp-peek-{}.db", std::process::id()));
    let cache = CacheHandle::open_at(db.clone());
    let track = fixture("tone.wav");
    let key = peakcache::fingerprint(&track).expect("the fixture has a fingerprint");
    let decision = Decision {
        gain: 1.0,
        kind: DecisionKind::ShortFullScan,
        measured_peak: 0.0891,
        duration_secs: 3.5,
    };
    cache.upsert(key, decision);
    let mut stored = false;
    for _ in 0..100 {
        if cache.get(key).is_some() {
            stored = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(stored, "the upsert never reached the cache");
    let measure = peek_measure(&track, &cache);
    assert!(measure.peak_exact);
    assert_eq!(measure.duration_secs, Some(3.5));
    let peak = measure.peak_dbtp.expect("a cached decision carries a peak");
    assert!((peak + 21.0).abs() < 0.05, "peak was {peak}");
    let _ = std::fs::remove_file(&db);
    let _ = std::fs::remove_file(db.with_extension("db-wal"));
    let _ = std::fs::remove_file(db.with_extension("db-shm"));
}
