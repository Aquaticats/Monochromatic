// What:     Unit tests for `peakcache.rs`, pulled in by
//           `#[cfg(test)] #[path = "peakcache_tests.rs"] mod tests;` at
//           the bottom of `peakcache.rs`. Compiles only under
//           `cargo nextest run` / `cargo test`; reaches the module items
//           (including private ones) via `use super::*` because this file is
//           the `tests` CHILD of peakcache.
// Why:      Keep the tests beside the code without inflating
//           `peakcache.rs` or its max-lines budget (sibling
//           `*_tests.rs` files are exempt from the linter).

// What:     `use super::*;`. Bring the module's items into the test scope.
// Why:      Tests use `fingerprint`, `CacheHandle`, `PathBuf`.
use super::*;
// What:     `use truepeak_core::{Decision, DecisionKind};`. The cached value and its tag.
// Why:      Round-trip tests build and compare `Decision`s.
use truepeak_core::{Decision, DecisionKind};
// What:     `use std::fs;`. Filesystem helpers for fixtures.
// Why:      Create real temp files to fingerprint.
use std::fs;
// What:     `use std::thread;` and `use std::time::{Duration, SystemTime, UNIX_EPOCH};`.
//           Sleeping while polling the async cache, plus clock + epoch for unique names.
// Why:      Upserts are fire-and-forget, so a test waits for the value to land; the clock
//           builds collision-free temp paths.
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// What:     `fn exact(gain: f32) -> Decision`. A full-scan-exact decision for the tests.
// Why:      An exact decision appears in the known-fingerprint (exact) snapshot.
fn exact(gain: f32) -> Decision {
    return Decision { gain, kind: DecisionKind::FullScanExact, measured_peak: 1.2, duration_secs: 200.0 }
}

// What:     `fn wait_for_decision(cache: &CacheHandle, key: u64) -> Option<Decision>`. Poll the
//           cache until the key appears, up to ~2s.
// Why:      `upsert` is fire-and-forget, so the row may not be committed on the first read.
fn wait_for_decision(cache: &CacheHandle, key: u64) -> Option<Decision> {
    // What:     up to 100 polls, 20ms apart.
    // Why:      Bounded wait for the async write to land without hanging forever.
    for _ in 0..100 {
        if let Some(decision) = cache.get(key) {
            return Some(decision);
        }
        thread::sleep(Duration::from_millis(20));
    }
    return None
}

// What:     `fn unique_path(suffix: &str) -> PathBuf`. A fresh throwaway path under
//           the system temp dir, tagged with pid + nanoseconds + `suffix`.
// Why:      Tests must not collide or touch real state.
fn unique_path(suffix: &str) -> PathBuf {
    // What:     Read nanoseconds elapsed since the Unix epoch.
    // Why:      High-resolution uniqueness prevents fixture collisions.
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after Unix epoch")
        .as_nanos();
    // What:     `crate::test_scratch::scratch_root().join(format!("music-player-peak-{}-{}-{}", std::process::id(), nanos, suffix))`.
    //           Build the path. Tail -> return.
    // Why:      Unique per process, call, and purpose.
    return crate::test_scratch::scratch_root().join(format!(
        "music-player-peak-{}-{}-{}",
        std::process::id(),
        nanos,
        suffix
    ))
}

// What:     `#[test]` fingerprint behaviour.
// Why:      Determinism, change-on-edit, and that a missing file has no key.
#[test]
fn fingerprint_is_stable_and_change_sensitive() {
    // What:     create a real temp file to stat.
    // Why:      `fingerprint` needs metadata.
    let file = unique_path("a.flac");
    fs::write(&file, b"hello").expect("first fingerprint fixture should be written");

    // What:     `let first = fingerprint(&file).unwrap();`. Compute it once (a `u64`).
    // Why:      Baseline.
    let first = fingerprint(&file).unwrap();
    // What:     same file fingerprints identically.
    // Why:      Determinism for cache hits.
    assert_eq!(fingerprint(&file).unwrap(), first);

    // What:     a file with different bytes (so a different size) fingerprints differently.
    // Why:      Change-sensitivity: size is part of the key, so a re-encode invalidates it.
    let other = unique_path("b.flac");
    fs::write(&other, b"hello world")
        .expect("changed fingerprint fixture should be written");
    assert_ne!(fingerprint(&other).unwrap(), first);

    // What:     a missing path yields no fingerprint.
    // Why:      Stat failure -> `None`.
    assert!(fingerprint(&unique_path("nope.flac")).is_none());

    // What:     clean up.
    // Why:      No droppings.
    let _ = fs::remove_file(&file);
    let _ = fs::remove_file(&other);
}

// What:     `#[test]` the wrapper hands the right inputs to the shared fingerprint function.
// Why:      The digest itself is pinned to the xxHash reference by the known-answer tests in
//           `truepeak-core`; this test pins the desktop plumbing: path text, file size, and
//           mtime nanoseconds reach `track_fingerprint` in the right slots. The mtime is set
//           explicitly so the expected value is computed from known inputs.
#[test]
fn fingerprint_feeds_path_size_and_mtime_to_the_shared_function() {
    // What:     a real temp file of exactly 5 bytes with a chosen modified-time.
    // Why:      Known size and mtime make the expected key computable.
    let file = unique_path("plumbing.flac");
    fs::write(&file, b"hello").expect("plumbing fixture should be written");
    // The nanoseconds are a multiple of 100 because Windows stores file times in 100 ns ticks; a finer value would be
    // rounded by the file system and the expected key below would not match what the wrapper reads back.
    let mtime = UNIX_EPOCH + Duration::new(1_700_000_000, 123_456_700);
    fs::File::options()
        .write(true)
        .open(&file)
        .expect("plumbing fixture should reopen")
        .set_modified(mtime)
        .expect("plumbing fixture mtime should be set");

    // What:     compute the expected key from the same inputs through the shared function.
    // Why:      A swapped size/mtime, a missing path, or a wrong unit in the wrapper changes it.
    let expected = track_fingerprint(&file.to_string_lossy(), 5, 1_700_000_000_123_456_700);
    assert_eq!(fingerprint(&file), Some(expected));

    // What:     a different mtime with the same size and path changes the key.
    // Why:      Positive control that the mtime is part of the key.
    fs::File::options()
        .write(true)
        .open(&file)
        .expect("plumbing fixture should reopen")
        .set_modified(UNIX_EPOCH + Duration::new(1_700_000_001, 123_456_700))
        .expect("plumbing fixture mtime should be set again");
    assert_ne!(fingerprint(&file), Some(expected));

    // What:     clean up.
    // Why:      No droppings.
    let _ = fs::remove_file(&file);
}

// What:     `#[test]` Turso round-trip across handles.
// Why:      An upsert must persist to disk and be readable after the handle is dropped
//           and the database reopened.
#[test]
fn upsert_persists_across_handles() {
    // What:     a throwaway database file path.
    // Why:      Never touch the real config dir.
    let db_file = unique_path("cache.db");
    // What:     `let key = 0xdead_beef_0000_0000_u64;`. A fixed fingerprint for the test.
    // Why:      A stable key to write, read, and reopen.
    let key = 0xdead_beef_0000_0000_u64;

    // What:     `let cache = CacheHandle::open_at(db_file.clone());`. Start the cache actor
    //           on the temp database.
    // Why:      Start fresh.
    let cache = CacheHandle::open_at(db_file.clone());
    // What:     upsert one exact decision, then wait for it to land.
    // Why:      Exercise the fire-and-forget write committing through the shared cache.
    cache.upsert(key, exact(0.75));
    assert_eq!(
        wait_for_decision(&cache, key),
        Some(exact(0.75)),
        "upsert did not become readable"
    );
    // What:     an unknown key misses.
    // Why:      `get` returns `None` for fingerprints never written.
    assert_eq!(cache.get(0x0000_0000_dead_beef), None);
    // What:     the exact key shows up in the known-fingerprint snapshot.
    // Why:      The sweep's skip-check reads this set (exact decisions only).
    assert!(cache.known_fingerprints().contains(&key));

    // What:     drop the handle (closing the actor), let it release the file, then reopen.
    // Why:      Prove the write reached disk, not just memory; the short settle avoids racing
    //           the first actor's connection close.
    drop(cache);
    thread::sleep(Duration::from_millis(50));
    let reopened = CacheHandle::open_at(db_file.clone());
    // What:     the entry survived the reopen.
    // Why:      Memoization works across runs.
    assert_eq!(reopened.get(key), Some(exact(0.75)));

    // What:     clean up the temp database (and any Turso sidecar files).
    // Why:      No droppings.
    let _ = fs::remove_file(&db_file);
    let _ = fs::remove_file(db_file.with_extension("db-wal"));
    let _ = fs::remove_file(db_file.with_extension("db-shm"));
}
