// What:     Unit tests for `peak_swap.rs`, pulled in by
//           `#[cfg(test)] #[path = "peak_swap_tests.rs"] mod tests;` at the
//           bottom of `peak_swap.rs`.
// Why:      Keep current-track peak swap tests beside the code they exercise.

// What:     `use super::*;`. Import the parent module's items into the test scope.
// Why:      Tests exercise crate-visible and private helpers.
use super::*;

// What:     `use truepeak_core::{Decision, DecisionKind};`. The cached value and its tag.
// Why:      The cache-hit test seeds a `Decision` and reads its gain back.
use truepeak_core::{Decision, DecisionKind};

// What:     `use std::path::{Path, PathBuf};`. Borrowed and owned path types.
// Why:      Tests borrow fixture paths and create owned temp cache paths.
use std::path::{Path, PathBuf};

// What:     `use std::thread;`. Rust's standard OS-thread API.
// Why:      Tests pass the current test thread handle into the peak worker.
use std::thread;

// What:     `use std::time::{Duration, SystemTime, UNIX_EPOCH};`. `Duration` is a
//           timeout span; `SystemTime` and `UNIX_EPOCH` build unique temp names.
// Why:      Async tests wait for worker results and isolate cache files.
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// What:     `fn temp_cache(tag: &str) -> PathBuf`. Build a fresh disposable cache path.
// Why:      Tests must never touch the user's real peak cache.
fn temp_cache(tag: &str) -> PathBuf {
    // What:     `let nanos = ...as_nanos();`. Current time since 1970 in nanoseconds.
    // Why:      Make a collision-resistant filename for parallel test runs.
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after Unix epoch")
        .as_nanos();
    // What:     `crate::test_scratch::scratch_root().join(format!(...))`. Join a formatted filename
    //           under the system temp directory. Tail expression returns it.
    // Why:      Keep disposable cache state out of the repo and real config dir.
    return crate::test_scratch::scratch_root().join(format!(
        "mp-peak-swap-{}-{}-{}.db",
        std::process::id(),
        nanos,
        tag
    ))
}

// What:     `fn fixture() -> &'static Path`. Return the committed FLAC fixture path.
// Why:      Multiple tests need a real decodable file.
fn fixture() -> &'static Path {
    // What:     `Path::new("fixture/tone.flac")`. Borrow a static string as a path.
    //           Tail expression returns it.
    // Why:      Existing true-peak tests already use this quiet fixture.
    return Path::new("fixture/tone.flac")
}

// What:     `fn test_cache(path: &Path) -> CacheHandle`. Build a cache handle backed by a
//           disposable database path.
// Why:      Match production helper signatures while isolating persistence.
fn test_cache(path: &Path) -> CacheHandle {
    // What:     `CacheHandle::open_at(path.to_path_buf())`. Start the cache actor on the
    //           given temp database file.
    // Why:      The measurement worker and test thread share the same cache handle.
    return CacheHandle::open_at(path.to_path_buf())
}

// What:     `fn wait_cached(cache: &CacheHandle, key: u64)`. Poll until the key is cached,
//           up to ~2s, then return whether it landed.
// Why:      Writes are fire-and-forget, so a test must wait before reading the value back.
fn wait_cached(cache: &CacheHandle, key: u64) -> bool {
    // What:     up to 100 polls, 20ms apart.
    // Why:      Bounded wait for the async write to land.
    for _ in 0..100 {
        if cache.get(key).is_some() {
            return true;
        }
        thread::sleep(Duration::from_millis(20));
    }
    return false
}

// What:     `fn approx_eq(a: f32, b: f32) -> bool`. Float comparison helper.
// Why:      Gains are f32 values and should be compared with tolerance.
fn approx_eq(a: f32, b: f32) -> bool {
    // What:     `const TOLERANCE: f32 = 1e-4;`. Allowed float difference.
    // Why:      Fixture decode and f32 math can vary in tiny low-order bits.
    const TOLERANCE: f32 = 1e-4;
    // What:     `(a - b).abs() < TOLERANCE`. Difference-based comparison. Tail
    //           expression returns the boolean.
    // Why:      Avoid brittle exact float equality.
    return (a - b).abs() < TOLERANCE
}

// What:     `#[test]` marks the next function as a unit test.
// Why:      The test runner discovers and runs it.
#[test]
fn cached_track_gain_returns_measured_gain() {
    // What:     `let cache_path = temp_cache("hit");`. Disposable cache file path.
    // Why:      Keep test state isolated.
    let cache_path = temp_cache("hit");
    // What:     `let cache = test_cache(&cache_path);`. Shared cache at the temp path.
    // Why:      Match production cache ownership.
    let cache = test_cache(&cache_path);
    // What:     `let key = peakcache::fingerprint(fixture()).unwrap();`. Compute the
    //           fixture's opaque cache key and unwrap it.
    // Why:      Seed a known cache hit without decoding the fixture.
    let key = peakcache::fingerprint(fixture()).unwrap();
    // What:     `let seeded = Decision { gain: fallback_track_gain() / 2.0, ... };`. A stored
    //           decision whose gain is half the ceiling fallback (the gain a peak of 2.0
    //           would produce).
    // Why:      The shared cache stores decisions with a precomputed gain, so seed a decision,
    //           not a raw peak.
    let seeded = Decision {
        gain: fallback_track_gain() / 2.0,
        kind: DecisionKind::FullScanExact,
        measured_peak: 2.0,
        duration_secs: 200.0,
    };
    // What:     `cache.upsert(key, seeded);` then wait for it to land.
    // Why:      Seed a known cache hit; the write is async, so wait before reading it back.
    cache.upsert(key, seeded);
    assert!(wait_cached(&cache, key), "seeded decision did not land");

    // What:     `let gain = cached_track_gain(fixture(), &cache).unwrap();`. Read the cached
    //           decision's gain through the cache-hit fast path.
    // Why:      Exercise the cache-hit fast path.
    let gain = cached_track_gain(fixture(), &cache).unwrap();
    // What:     `assert!(approx_eq(gain, fallback_track_gain() / 2.0));`. The returned gain is
    //           the seeded decision's gain.
    // Why:      Confirms the cache-hit path returns the stored decision's gain unchanged.
    assert!(approx_eq(gain, fallback_track_gain() / 2.0));

    // What:     `let _ = std::fs::remove_file(&cache_path);`. Best-effort cleanup.
    // Why:      Leave no temp cache file behind.
    let _ = std::fs::remove_file(&cache_path);
}

// What:     `#[test]` marks the next function as a unit test.
// Why:      The test runner discovers and runs it.
// What:     `fn wait_until_ready(pending: &PendingPeakMeasurement, timeout: Duration) -> PendingPeakStatus`.
//           Polls the handle until a result, a disconnect, or the deadline.
// Why:      The engine polls the same handle without blocking, so the test does the same with a deadline.
fn wait_until_ready(pending: &PendingPeakMeasurement, timeout: Duration) -> PendingPeakStatus {
    // What:     `let deadline = std::time::Instant::now() + timeout;`. The last moment to keep polling.
    // Why:      Bounds the test so a broken worker fails instead of hanging.
    let deadline = std::time::Instant::now() + timeout;
    loop {
        // What:     `let status = pending.try_result();`. Check the handle once.
        // Why:      Same non-blocking read the engine loop performs.
        let status = pending.try_result();
        // What:     `if !matches!(status, PendingPeakStatus::Pending) || ... { return status; }`. Stop on a final status or at the deadline.
        // Why:      A still-pending status is the only one that means "poll again".
        if !matches!(status, PendingPeakStatus::Pending) || std::time::Instant::now() >= deadline {
            return status;
        }
        // What:     `std::thread::sleep(Duration::from_millis(10));`. Short pause between polls.
        // Why:      Avoids spinning a core while the fixture decodes.
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn async_current_track_measurement_populates_cache_and_returns_gain() {
    // What:     `let cache_path = temp_cache("async");`. Disposable cache file path.
    // Why:      Keep test state isolated.
    let cache_path = temp_cache("async");
    // What:     `let cache = test_cache(&cache_path);`. Shared empty cache.
    // Why:      The worker will populate this cache.
    let cache = test_cache(&cache_path);
    // What:     `let generation = 7;`. Arbitrary generation id for this test.
    // Why:      The worker should echo it in the result.
    let generation = 7;
    // What:     `let pending = match prepare_track_gain(...) { ... }`. Prepare gain for
    //           an uncached fixture and expect the pending branch.
    // Why:      Empty cache should spawn current-track measurement.
    let pending = match prepare_track_gain(fixture(), &cache, generation, thread::current()) {
        // What:     `TrackGainResolution::Pending(pending) => pending`. Extract the
        //           pending measurement handle.
        // Why:      The test waits for its result.
        TrackGainResolution::Pending(pending) => pending,
        // What:     `TrackGainResolution::Ready(_) => panic!(...)`. Unexpected cache hit.
        // Why:      A fresh temp cache should not have an entry.
        TrackGainResolution::Ready(_) => panic!("expected pending measurement"),
    };

    // What:     `let status = wait_until_ready(&pending, Duration::from_secs(5));`. Poll for up to
    //           five seconds for the real fixture decode.
    // Why:      The worker runs on another thread and should finish quickly.
    let status = wait_until_ready(&pending, Duration::from_secs(5));
    // What:     `let result = match status { ... }`. Extract the ready result or fail.
    // Why:      The async measurement must complete and report a gain.
    let result = match status {
        // What:     `PendingPeakStatus::Ready(result) => result`. Use the measured result.
        // Why:      Continue assertions.
        PendingPeakStatus::Ready(result) => result,
        // What:     `_ => panic!(...)`. Any non-ready status is a failure.
        // Why:      Worker should not time out or close on the committed fixture.
        _ => panic!("measurement did not finish"),
    };
    // What:     `assert_eq!(result.generation, generation);`. Result generation matches.
    // Why:      Controller stale checks depend on this id round trip.
    assert_eq!(result.generation, generation);
    // What:     `assert!(approx_eq(result.gain, 1.0));`. Quiet fixture stays unattenuated.
    // Why:      Attenuate-only normalization should not boost or cut this track.
    assert!(approx_eq(result.gain, 1.0));
    // What:     `let key = peakcache::fingerprint(fixture()).unwrap();`. Recompute the
    //           fixture cache key.
    // Why:      Confirm the worker warmed the cache.
    let key = peakcache::fingerprint(fixture()).unwrap();
    // What:     `assert!(wait_cached(&cache, key));`. Wait for and check the cache entry.
    // Why:      Current-track measurement must warm the shared cache (write is async).
    assert!(wait_cached(&cache, key), "worker did not warm the cache");

    // What:     `let _ = std::fs::remove_file(&cache_path);`. Best-effort cleanup.
    // Why:      Leave no temp cache file behind.
    let _ = std::fs::remove_file(&cache_path);
}
