// What:     Unit tests for `measure.rs`, pulled in by
//           `#[cfg(test)] #[path = "measure_tests.rs"] mod tests;` at
//           the bottom of `measure.rs`. Compiles only under
//           `cargo nextest run` / `cargo test`; reaches the module items
//           (including private ones) via `use super::*` because this file is
//           the `tests` CHILD of measure.
// Why:      Keep the tests beside the code without inflating
//           `measure.rs` or its max-lines budget (sibling
//           `*_tests.rs` files are exempt from the linter).

// What:     `use super::*;`. Bring the module's items into the test scope.
// Why:      Tests use `spawn_queue_measurement`, `CacheHandle`, `thread`, and helpers.
use super::*;
// What:     `use truepeak_core::Decision;`. The cached value the sweep stores.
// Why:      The poll collects a `Decision` and asserts on its measured peak.
use truepeak_core::Decision;
// What:     `use std::time::{Duration, SystemTime, UNIX_EPOCH};`. `Duration` for the poll
//           sleep (measure.rs no longer imports it), clock + epoch for unique names.
// Why:      Wait for the async sweep, and build a collision-free throwaway cache path.
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::collections::HashSet;
use std::sync::{Mutex, PoisonError};

// What:     `fn temp_cache(tag: &str) -> PathBuf`. A fresh throwaway cache-file path.
// Why:      Point the cache at disposable state, never the real config dir.
/// What: `fn sweep_job(tracks, cache) -> SweepJob` builds a sweep with no current track and no sink.
/// Why: the sweep tests check the cache, not the reports, so the reports go nowhere.
fn sweep_job(tracks: Vec<PathBuf>, cache: CacheHandle) -> SweepJob {
    return SweepJob {
        tracks,
        current: None,
        cache,
        sink: crate::sweep_report::noop_sink(),
        generation: 1,
        gate: Arc::new(SweepGate::new()),
        claims: Arc::new(Mutex::new(HashSet::new())),
        quiet: false,
        listed: false,
        announce: Vec::new(),
        refresh_id: None,
    };
}

fn temp_cache(tag: &str) -> PathBuf {
    // What:     `let nanos = ...as_nanos();`. Nanoseconds since 1970 for uniqueness.
    // Why:      Avoid collisions across tests/runs.
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after Unix epoch")
        .as_nanos();
    // What:     build the path under the system temp dir. Tail -> return.
    // Why:      Disposable location.
    return crate::test_scratch::scratch_root().join(format!(
        "mp-measure-{}-{}-{}.db",
        std::process::id(),
        nanos,
        tag
    ))
}

// What:     `#[test]` for the background sweep.
// Why:      Spawning over a track must populate the shared cache.
#[test]
fn spawn_queue_measurement_populates_cache() {
    // What:     `let path = temp_cache("sweep");`. Throwaway cache file.
    // Why:      Disposable state.
    let path = temp_cache("sweep");
    // What:     cache handle backed by the temp database.
    // Why:      The sweep writes here.
    let cache = CacheHandle::open_at(path.clone());
    // What:     `let fixture = PathBuf::from("fixture/tone.flac");`. The track to sweep.
    // Why:      A real file the sweep can measure.
    let fixture = PathBuf::from("fixture/tone.flac");

    // What:     `spawn_queue_measurement(vec![fixture.clone()], Arc::clone(&cache));`.
    //           Start the detached sweep over a one-track queue.
    // Why:      The behaviour under test.
    spawn_queue_measurement(sweep_job(vec![fixture.clone()], cache.clone()));

    // What:     `let key = peakcache::fingerprint(&fixture).unwrap();`. The cache key.
    // Why:      Poll for it.
    let key = peakcache::fingerprint(&fixture).unwrap();
    // What:     `let mut found: Option<Decision> = None;`. The decision once the sweep stores
    //           it.
    // Why:      Collected by polling.
    let mut found: Option<Decision> = None;
    // What:     `for _ in 0..100 { ... }`. Poll up to 100 times (~5s) for the entry.
    //           `_` ignores the loop counter.
    // Why:      The sweep runs on another thread; wait for it without hanging forever.
    for _ in 0..100 {
        // What:     `if let Some(decision) = cache.get(key) { found = Some(decision); break; }`.
        //           Check the shared cache; stop once present.
        // Why:      Detect completion.
        if let Some(decision) = cache.get(key) {
            found = Some(decision);
            break;
        }
        // What:     `thread::sleep(Duration::from_millis(50));`. Wait before re-checking.
        // Why:      Give the sweep time without busy-spinning.
        thread::sleep(Duration::from_millis(50));
    }

    // What:     `let decision = found.expect("background sweep did not populate the cache");`.
    //           Unwrap the polled value or fail with a message.
    // Why:      The sweep must have measured the fixture.
    let decision = found.expect("background sweep did not populate the cache");
    // What:     `assert!(decision.measured_peak > 0.05 && decision.measured_peak < 0.2, ...)`.
    //           The fixture's real level; warming full-scans, so it is the exact true peak.
    // Why:      Confirm a sane measured value, not garbage.
    assert!(
        decision.measured_peak > 0.05 && decision.measured_peak < 0.2,
        "peak was {}",
        decision.measured_peak
    );
    // What:     `assert_eq!(decision.kind, DecisionKind::ShortFullScan);`. The fixture is
    //           short, so warming's exact scan tags it short.
    // Why:      Confirm the warming path resolves an exact short decision.
    assert_eq!(decision.kind, truepeak_core::DecisionKind::ShortFullScan);

    // What:     clean up the temp cache file.
    // Why:      No droppings.
    let _ = std::fs::remove_file(&path);
}

// What:     `#[test]` parallel sweep over many tracks across codecs.
// Why:      Exercise the N-worker fan-out and the concurrent upsert path: every fingerprint
//           must land, with no worker dropping or duplicating a track.
#[test]
fn parallel_sweep_measures_every_track() {
    // What:     a throwaway cache database.
    // Why:      Disposable state.
    let path = temp_cache("parallel");
    // What:     cache handle backed by the temp database.
    // Why:      The sweep writes here.
    let cache = CacheHandle::open_at(path.clone());
    // What:     several real, decodable fixtures across codecs.
    // Why:      Give the workers enough distinct tracks to run concurrently.
    let tracks = vec![
        PathBuf::from("fixture/tone.flac"),
        PathBuf::from("fixture/tone.mp3"),
        PathBuf::from("fixture/tone.ogg"),
        PathBuf::from("fixture/tone.wav"),
        PathBuf::from("fixture/tone.opus"),
        PathBuf::from("fixture/tone.aac.m4a"),
        PathBuf::from("fixture/tone.alac.m4a"),
    ];

    // What:     run the parallel sweep over all tracks.
    // Why:      The behaviour under test.
    spawn_queue_measurement(sweep_job(tracks.clone(), cache.clone()));

    // What:     every track's fingerprint must become cached within the poll window.
    // Why:      Confirm no worker dropped a track and every upsert landed.
    for track in &tracks {
        let key = peakcache::fingerprint(track).unwrap();
        let mut found = false;
        for _ in 0..200 {
            if cache.get(key).is_some() {
                found = true;
                break;
            }
            thread::sleep(Duration::from_millis(25));
        }
        assert!(found, "track not measured: {}", track.display());
    }

    // What:     clean up the temp database + Turso sidecars.
    // Why:      No droppings.
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(path.with_extension("db-wal"));
    let _ = std::fs::remove_file(path.with_extension("db-shm"));
}

/// What: `collecting_sink` builds a sink that keeps every report, and the list it fills.
/// Why: the gate tests read the progress reports to tell a finished sweep from a parked one.
fn collecting_sink() -> (SweepSink, Arc<Mutex<Vec<SweepReport>>>) {
    let reports = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::clone(&reports);
    let sink: SweepSink = Arc::new(move |report| {
        log.lock().unwrap_or_else(PoisonError::into_inner).push(report);
    });
    return (sink, reports);
}

/// What: `final_progress_seen` is true once the sweep has reported `done == total` for `total` tracks.
/// Why: the coordinator sends that report after every worker has been joined, so it marks the sweep's end.
fn final_progress_seen(reports: &Arc<Mutex<Vec<SweepReport>>>, total: usize) -> bool {
    return reports
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .iter()
        .any(|report| return matches!(report, SweepReport::Progress { done, total: all, .. } if *done == total && *all == total));
}

/// What: `three_fixture_tracks` lists three short decodable fixtures.
/// Why: the gate tests need a few distinct tracks to claim.
fn three_fixture_tracks() -> Vec<PathBuf> {
    return vec![
        PathBuf::from("fixture/tone.flac"),
        PathBuf::from("fixture/tone.mp3"),
        PathBuf::from("fixture/tone.wav"),
    ];
}

/// What: `wait_for` polls `condition` every 25 ms up to `seconds` seconds.
/// Why: the sweep runs on detached threads, so tests wait for its effects.
fn wait_for(seconds: u64, mut condition: impl FnMut() -> bool) -> bool {
    for _ in 0..(seconds * 40) {
        if condition() {
            return true;
        }
        thread::sleep(Duration::from_millis(25));
    }
    return condition();
}

// What: a sweep started while the gate is paused claims no track, and resuming lets it measure every track.
// Why: D107: pausing stops new claims and resume continues them. This is the guard test for the gate: without the
// admission check in `run_worker` the paused sweep measures at once and the first assertion fails.
#[test]
fn a_paused_sweep_claims_no_track_until_it_is_resumed() {
    let path = temp_cache("gate-pause");
    let cache = CacheHandle::open_at(path.clone());
    let tracks = three_fixture_tracks();
    let (sink, reports) = collecting_sink();
    let gate = Arc::new(SweepGate::new());
    gate.set_paused(true);
    spawn_queue_measurement(SweepJob {
        tracks: tracks.clone(),
        current: None,
        cache: cache.clone(),
        sink,
        generation: 1,
        gate: Arc::clone(&gate),
        claims: Arc::new(Mutex::new(HashSet::new())),
        quiet: false,
        listed: false,
        announce: Vec::new(),
        refresh_id: None,
    });

    thread::sleep(Duration::from_millis(600));
    for track in &tracks {
        let key = peakcache::fingerprint(track).expect("fixture has a fingerprint");
        assert!(cache.get(key).is_none(), "a paused sweep measured {}", track.display());
    }
    assert!(!final_progress_seen(&reports, tracks.len()), "a paused sweep finished");

    gate.set_paused(false);
    assert!(wait_for(20, || return final_progress_seen(&reports, tracks.len())), "the resumed sweep never finished");
    for track in &tracks {
        let key = peakcache::fingerprint(track).expect("fixture has a fingerprint");
        assert!(wait_for(10, || return cache.get(key).is_some()), "the resumed sweep skipped {}", track.display());
    }

    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(path.with_extension("db-wal"));
    let _ = std::fs::remove_file(path.with_extension("db-shm"));
}

// What: closing the gate while the sweep is parked ends the sweep without measuring anything.
// Why: shutdown while paused must not hang the coordinator that joins the workers.
#[test]
fn closing_the_gate_while_paused_ends_the_sweep_without_hanging() {
    let path = temp_cache("gate-close");
    let cache = CacheHandle::open_at(path.clone());
    let tracks = three_fixture_tracks();
    let (sink, reports) = collecting_sink();
    let gate = Arc::new(SweepGate::new());
    gate.set_paused(true);
    spawn_queue_measurement(SweepJob {
        tracks: tracks.clone(),
        current: None,
        cache: cache.clone(),
        sink,
        generation: 1,
        gate: Arc::clone(&gate),
        claims: Arc::new(Mutex::new(HashSet::new())),
        quiet: false,
        listed: false,
        announce: Vec::new(),
        refresh_id: None,
    });

    thread::sleep(Duration::from_millis(300));
    assert!(!final_progress_seen(&reports, tracks.len()));
    gate.close();
    assert!(wait_for(20, || return final_progress_seen(&reports, tracks.len())), "the sweep hung after the gate closed");
    for track in &tracks {
        let key = peakcache::fingerprint(track).expect("fixture has a fingerprint");
        assert!(cache.get(key).is_none(), "a closed gate still let {} be measured", track.display());
    }

    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(path.with_extension("db-wal"));
    let _ = std::fs::remove_file(path.with_extension("db-shm"));
}

/// What: `peak_reports_for` counts the measures with a true peak that the reports carry for tracks whose path ends with `name`.
/// Why: one such measure is one decode of that file.
fn peak_reports_for(reports: &Arc<Mutex<Vec<SweepReport>>>, name: &str) -> usize {
    return reports
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .iter()
        .map(|report| {
            return match report {
                SweepReport::Measures { measures, .. } => measures
                    .iter()
                    .filter(|(path, measure)| return path.ends_with(name) && measure.peak_dbtp.is_some())
                    .count(),
                _ => 0,
            };
        })
        .sum();
}

// What:     Two overlapping sweeps of one load decode each track once: the shared claims leave a track to whichever sweep took it.
// Why:      A report with a peak is either a decode or a peek at the cache. The sweep that is not quiet peeks the cache once,
//           before its workers start, and reports every hit. If the quiet sweep had already stored a track by then (a thread
//           that starts late, as on a loaded Windows runner, does that), the track was reported twice for one decode and the
//           count below was 2. The quiet sweep is therefore started only after the first sweep has finished its peek (its
//           first progress report comes right after it), so the cache is empty at the peek and every report is a decode.
#[test]
fn two_sweeps_of_one_load_decode_a_track_only_once() {
    let path = temp_cache("claims");
    let cache = CacheHandle::open_at(path.clone());
    let tracks = three_fixture_tracks();
    let (sink, reports) = collecting_sink();
    let claims: Arc<Mutex<HashSet<u64>>> = Arc::new(Mutex::new(HashSet::new()));
    for quiet in [false, true] {
        spawn_queue_measurement(SweepJob {
            tracks: tracks.clone(),
            current: None,
            cache: cache.clone(),
            sink: Arc::clone(&sink),
            generation: 1,
            gate: Arc::new(SweepGate::new()),
            claims: Arc::clone(&claims),
            quiet,
            listed: false,
            announce: Vec::new(),
            refresh_id: None,
        });
        if !quiet {
            assert!(
                wait_for(20, || return reports.lock().unwrap_or_else(PoisonError::into_inner).iter().any(|report| return matches!(report, SweepReport::Progress { .. }))),
                "the first sweep finished its peek and began"
            );
        }
    }
    assert!(wait_for(20, || return final_progress_seen(&reports, tracks.len())), "the sweep of the load finished");
    thread::sleep(Duration::from_millis(1_000));
    for name in ["tone.flac", "tone.mp3", "tone.wav"] {
        assert_eq!(peak_reports_for(&reports, name), 1, "{name} was decoded once, not once per sweep");
    }
    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_file_that_cannot_be_decoded_is_tried_again_by_the_next_sweep_of_the_same_load() {
    let path = temp_cache("retry");
    let cache = CacheHandle::open_at(path.clone());
    let broken = crate::test_scratch::scratch_root().join(format!("music-player-broken-{}.flac", std::process::id()));
    std::fs::write(&broken, b"this is not audio").expect("the broken file is written");
    let (sink, reports) = collecting_sink();
    let claims: Arc<Mutex<HashSet<u64>>> = Arc::new(Mutex::new(HashSet::new()));
    let unreadable = |log: &Arc<Mutex<Vec<SweepReport>>>| {
        return log
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .filter(|report| return matches!(report, SweepReport::Unreadable { .. }))
            .count();
    };
    for (round, quiet) in [(1, false), (2, true)] {
        spawn_queue_measurement(SweepJob {
            tracks: vec![broken.clone()],
            current: None,
            cache: cache.clone(),
            sink: Arc::clone(&sink),
            generation: 1,
            gate: Arc::new(SweepGate::new()),
            claims: Arc::clone(&claims),
            quiet,
            listed: false,
            announce: Vec::new(),
            refresh_id: None,
        });
        assert!(wait_for(10, || return unreadable(&reports) >= round), "sweep {round} reported the broken file");
    }
    let _ = std::fs::remove_file(&broken);
    let _ = std::fs::remove_file(&path);
}

/// What: `lower_current_thread_to_idle` leaves the calling thread, and only that thread, in `SCHED_IDLE`.
/// Why: musl's `sched_setscheduler` wrapper always fails with ENOSYS, so a call through the C library would leave
/// the sweep at normal priority on Alpine while still passing on glibc; reading the policy back from the kernel
/// proves the direct system call took effect on whichever libc the test runs on.
#[cfg(target_os = "linux")]
#[test]
fn lower_current_thread_to_idle_puts_only_the_calling_thread_in_the_idle_class() {
    let policy_of_calling_thread = || {
        return unsafe { libc::syscall(libc::SYS_sched_getscheduler, 0) };
    };
    let before = policy_of_calling_thread();
    let lowered = std::thread::spawn(|| {
        lower_current_thread_to_idle();
        return unsafe { libc::syscall(libc::SYS_sched_getscheduler, 0) };
    })
    .join()
    .expect("the lowering thread finished");
    assert_eq!(lowered, libc::c_long::from(libc::SCHED_IDLE), "the lowered thread is in SCHED_IDLE");
    assert_eq!(policy_of_calling_thread(), before, "the thread that spawned it keeps its own policy");
}
