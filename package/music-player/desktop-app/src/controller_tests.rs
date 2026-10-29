// What:     Unit tests for controller peak-swap state, pulled in by
//           `#[cfg(test)] #[path = "controller_tests.rs"] mod tests;` at the
//           bottom of `controller.rs`.
// Why:      Keep tests beside the controller without inflating production code.

// What:     `use super::*;`. Import the parent controller module's private and
//           crate-visible items into this child test module.
// Why:      Tests directly inspect `Controller` fields and helper methods.
use super::*;

// What:     `use std::path::PathBuf;`. Owned filesystem path buffer.
// Why:      The skip-current sweep test builds a queue of owned fixture paths.
use std::path::PathBuf;

// What:     `use std::sync::mpsc;`. Import Rust's channel module.
// Why:      Tests build manual pending peak receivers without spawning decoder threads.
use std::sync::mpsc;

// What:     `use std::time::Duration;`. A monotonic span of time.
// Why:      Tests pass a tiny timeout instead of waiting the production one-second window.
use std::time::Duration;

// What:     `use std::time::{SystemTime, UNIX_EPOCH};`. Clock plus 1970 epoch reference.
// Why:      Build unique disposable cache filenames.
use std::time::{SystemTime, UNIX_EPOCH};

// What:     `use crate::peak_swap::{...};`. Import fallback gain, pending wrapper,
//           and manual result type from the peak-swap module.
// Why:      Tests need to seed pending state and assert gain changes.
use crate::peak_swap::{fallback_track_gain, PeakGainResult, PendingPeakMeasurement};

// What:     `use crate::peakcache;`. Import the peak-cache module. `CacheHandle` arrives via
//           `use super::*` (the controller re-imports it).
// Why:      The skip-current sweep test computes fingerprints and opens a temp-backed cache.
use crate::peakcache;

// What:     `fn approx_eq(a: f32, b: f32) -> bool`. Float comparison helper.
// Why:      Peak gains are f32 values, so tests compare with a small tolerance.
fn approx_eq(a: f32, b: f32) -> bool {
    // What:     `const TOLERANCE: f32 = 1e-6;`. Small allowed float difference.
    // Why:      Avoid exact equality on floating point values.
    const TOLERANCE: f32 = 1e-6;
    // What:     `(a - b).abs() < TOLERANCE`. Difference-based comparison. Tail
    //           expression returns the boolean.
    // Why:      Values that differ only by f32 rounding count as equal.
    return (a - b).abs() < TOLERANCE
}

// What:     `fn test_controller() -> Controller`. Build a controller with no audio
//           output, a no-op update callback, and a degraded (no-disk) cache.
// Why:      Peak-swap state does not need PipeWire/cpal or UI machinery, and tests must never
//           open or create the real peaks.db.
fn test_controller() -> Controller {
    // What:     `Controller::new(Box::new(|_| {}), None, CacheHandle::open_degraded())`.
    //           Construct the controller; `Box::new` heap-boxes the callback closure, `None`
    //           means silent mode, and the degraded cache touches no disk.
    // Why:      Give tests a real controller without an audio device or real cache state.
    return Controller::new(Box::new(|_| {}), None, CacheHandle::open_degraded())
}

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
        "mp-controller-{}-{}-{}.json",
        std::process::id(),
        nanos,
        tag
    ))
}

// What:     `#[test]` marks the next function as a Rust unit test.
// Why:      The test runner discovers and runs this case.
/// What: `fn test_decision(gain: f32) -> Decision` builds a placeholder decision for the pending-peak results.
/// Why: the result carries the decision, and these tests only check the gain.
fn test_decision(gain: f32) -> truepeak_core::Decision {
    return truepeak_core::Decision {
        gain,
        kind: truepeak_core::DecisionKind::ShortFullScan,
        measured_peak: 0.5,
        duration_secs: 1.0,
    };
}

#[test]
fn unfinished_measurement_keeps_fallback_then_later_result_swaps_gain() {
    // What:     `let mut controller = test_controller();`. Build mutable test state.
    // Why:      The wait and poll helpers mutate peak fields.
    let mut controller = test_controller();
    // What:     `controller.peak_generation = 1;`. Seed the current track generation.
    // Why:      The manual result below must match this generation to be accepted.
    controller.peak_generation = 1;
    // What:     `let fallback = fallback_track_gain();`. Read the temporary ceiling gain.
    // Why:      Assert the timeout path leaves it unchanged.
    let fallback = fallback_track_gain();
    // What:     `controller.track_gain = fallback;`. Seed current gain to fallback.
    // Why:      This is the state after loading an uncached track.
    controller.track_gain = fallback;
    // What:     `let (sender, receiver) = mpsc::channel();`. Create a manual one-shot
    //           channel for a future peak result.
    // Why:      Keep measurement pending through the short timeout, then send a result.
    let (sender, receiver) = mpsc::channel();
    // What:     `controller.pending_peak = Some(PendingPeakMeasurement::from_receiver(receiver));`.
    //           Wrap the receiver in `Some` and store it on the controller.
    // Why:      Simulate an in-flight current-track measurement.
    controller.pending_peak = Some(PendingPeakMeasurement::from_receiver(receiver));

    // What:     `assert!(!controller.poll_pending_peak());`. Poll with no value sent yet.
    // Why:      A measurement that has not landed must leave the fallback gain and the pending handle alone.
    assert!(!controller.poll_pending_peak());
    assert!(controller.pending_peak.is_some());

    // What:     `assert!(approx_eq(controller.track_gain, fallback));`. Gain stayed fallback.
    // Why:      Timeout must not invent or clear a measured gain.
    assert!(approx_eq(controller.track_gain, fallback));
    // What:     `assert!(controller.pending_peak.is_some());`. Pending receiver remains.
    // Why:      A later measurement result should still be able to swap gain.
    assert!(controller.pending_peak.is_some());

    // What:     Send a matching-generation measured gain through the channel.
    // Why:      The next poll should accept it and replace fallback.
    sender
        .send(PeakGainResult {
            generation: 1,
            gain: 0.5,
            path: PathBuf::from("/music/current.flac"),
            decision: test_decision(0.5),
        })
        .expect("pending peak receiver should remain connected");
    // What:     `assert!(controller.poll_pending_peak());`. Poll and expect a live
    //           current-track result to apply.
    // Why:      Confirms late results still swap future decoded samples.
    assert!(controller.poll_pending_peak());
    // What:     `assert!(approx_eq(controller.track_gain, 0.5));`. The measured gain landed.
    // Why:      Future decoded samples now use the measured value.
    assert!(approx_eq(controller.track_gain, 0.5));
    // What:     `assert!(controller.pending_peak.is_none());`. Receiver was consumed.
    // Why:      The current measurement is complete.
    assert!(controller.pending_peak.is_none());
}

// What:     `#[test]` marks the next function as a Rust unit test.
// Why:      The test runner discovers and runs this case.
#[test]
fn stale_generation_result_is_ignored() {
    // What:     `let mut controller = test_controller();`. Build mutable test state.
    // Why:      The poll helper mutates pending state.
    let mut controller = test_controller();
    // What:     `controller.peak_generation = 2;`. Current track has generation 2.
    // Why:      The result below pretends to belong to an older track.
    controller.peak_generation = 2;
    // What:     `let fallback = fallback_track_gain();`. Read fallback gain.
    // Why:      Assert stale result does not change it.
    let fallback = fallback_track_gain();
    // What:     `controller.track_gain = fallback;`. Seed fallback state.
    // Why:      Stale result should leave this value untouched.
    controller.track_gain = fallback;
    // What:     `let (sender, receiver) = mpsc::channel();`. Create a manual channel.
    // Why:      Feed one stale result to the controller.
    let (sender, receiver) = mpsc::channel();
    // What:     `controller.pending_peak = Some(PendingPeakMeasurement::from_receiver(receiver));`.
    //           Store the pending receiver.
    // Why:      Make `poll_pending_peak` read the stale result.
    controller.pending_peak = Some(PendingPeakMeasurement::from_receiver(receiver));
    // What:     Send an older generation.
    // Why:      This simulates a previous track finishing after the current one loaded.
    sender
        .send(PeakGainResult {
            generation: 1,
            gain: 0.5,
            path: PathBuf::from("/music/current.flac"),
            decision: test_decision(0.5),
        })
        .expect("pending peak receiver should remain connected");

    // What:     `assert!(!controller.poll_pending_peak());`. Poll returns false because
    //           the result was not applied.
    // Why:      Stale generation must be ignored.
    assert!(!controller.poll_pending_peak());
    // What:     `assert!(approx_eq(controller.track_gain, fallback));`. Gain stayed fallback.
    // Why:      Old track results must not affect the current track.
    assert!(approx_eq(controller.track_gain, fallback));
    // What:     `assert!(controller.pending_peak.is_none());`. The stale pending result
    //           was consumed and cleared.
    // Why:      There is no useful result left in that receiver.
    assert!(controller.pending_peak.is_none());
}

// What:     `#[test]` marks the next function as a Rust unit test.
// Why:      The test runner discovers and runs this case.
#[test]
fn play_start_does_not_wait_for_pending_peak() {
    // What:     `let mut controller = test_controller();`. Build mutable test state.
    // Why:      `set_playing` mutates gain and playing fields.
    let mut controller = test_controller();
    // What:     `controller.peak_generation = 3;`. Seed the current track generation.
    // Why:      The later manual result must match this generation to be applied.
    controller.peak_generation = 3;
    // What:     `controller.track_gain = fallback_track_gain();`. Seed the fallback gain.
    // Why:      This is the state of a track whose measurement is still in flight.
    controller.track_gain = fallback_track_gain();
    // What:     `let (sender, receiver) = mpsc::channel();`. Create a manual channel and keep the sender open.
    // Why:      The measurement stays pending when playback starts. The old start path blocked on it for one second.
    let (sender, receiver) = mpsc::channel();
    // What:     `controller.pending_peak = Some(PendingPeakMeasurement::from_receiver(receiver));`.
    //           Store the pending receiver on the controller.
    // Why:      Simulate a cache miss at the moment of a Play command.
    controller.pending_peak = Some(PendingPeakMeasurement::from_receiver(receiver));

    // What:     `let started = std::time::Instant::now();`. Start a stopwatch around the call.
    // Why:      The assertion below bounds how long the start path may take.
    let started = std::time::Instant::now();
    // What:     `controller.set_playing(true);`. Start playback while the measurement is pending.
    // Why:      This is the path that used to wait up to the one-second swap window.
    controller.set_playing(true);
    // What:     `let blocked_for = started.elapsed();`. Read the elapsed time once.
    // Why:      Compare it with the bound without reading the clock twice.
    let blocked_for = started.elapsed();

    // What:     `assert!(controller.playing);`. Playback is on.
    // Why:      Starting must not be refused because the gain is not known yet.
    assert!(controller.playing);
    // What:     `assert!(blocked_for < Duration::from_millis(500));`. The start returned well inside the old one-second wait.
    // Why:      A timing bound is the only way to tell a blocking start from a non-blocking one, since both leave the same state.
    assert!(
        blocked_for < Duration::from_millis(500),
        "set_playing blocked for {blocked_for:?}"
    );
    // What:     `assert!(approx_eq(controller.track_gain, fallback_track_gain()));`. Fallback gain stays in place.
    // Why:      Playback starts with the safe ceiling until the exact gain arrives.
    assert!(approx_eq(controller.track_gain, fallback_track_gain()));
    // What:     `assert!(controller.pending_peak.is_some());`. The measurement is still pending.
    // Why:      Nothing has been dropped; the engine loop will poll it.
    assert!(controller.pending_peak.is_some());

    // What:     `sender.send(PeakGainResult { ... })`. The measurement lands after playback started.
    // Why:      The exact gain must replace the fallback on the next poll.
    sender
        .send(PeakGainResult {
            generation: 3,
            gain: 0.5,
            path: PathBuf::from("/music/current.flac"),
            decision: test_decision(0.5),
        })
        .expect("pending peak receiver should remain connected");
    // What:     `assert!(controller.poll_pending_peak());`. The engine loop's poll applies it.
    // Why:      The swap happens after start, not during it.
    assert!(controller.poll_pending_peak());
    assert!(approx_eq(controller.track_gain, 0.5));
    assert!(controller.pending_peak.is_none());
}

#[test]
fn background_sweep_skips_current_track() {
    // What:     `let cache_path = temp_cache("skip-current");`. Disposable cache file path.
    // Why:      Keep the test away from real config state.
    let cache_path = temp_cache("skip-current");
    // What:     `let mut controller = test_controller();`. Build mutable controller.
    // Why:      The test replaces its cache and queue.
    let mut controller = test_controller();
    // What:     `controller.peaks = CacheHandle::open_at(cache_path.clone());`. Replace the
    //           controller's peak cache with a temp-backed handle.
    // Why:      Background sweep writes to disposable state.
    controller.peaks = CacheHandle::open_at(cache_path.clone());
    // What:     `let current = PathBuf::from("fixture/tone.flac");`. First queue track.
    // Why:      Queue starts at index 0, so this is the current track to skip.
    let current = PathBuf::from("fixture/tone.flac");
    // What:     `let other = PathBuf::from("fixture/tone.mp3");`. Second queue track.
    // Why:      Background sweep should measure this non-current track.
    let other = PathBuf::from("fixture/tone.mp3");
    // What:     `controller.queue.set_tracks(vec![current.clone(), other.clone()]);`.
    //           Replace queue with two owned fixture paths.
    // Why:      Establish current plus non-current tracks.
    controller.queue.set_tracks(vec![current.clone(), other.clone()]);

    // What:     `controller.start_queue_measurement();`. Launch background sweep.
    // Why:      Exercise its skip-current filtering.
    controller.start_queue_measurement();

    // What:     `let current_key = peakcache::fingerprint(&current).unwrap();`. Key for
    //           the current track.
    // Why:      Assert it stays uncached by this sweep.
    let current_key = peakcache::fingerprint(&current).unwrap();
    // What:     `let other_key = peakcache::fingerprint(&other).unwrap();`. Key for
    //           the non-current track.
    // Why:      Poll for this one to confirm the sweep ran.
    let other_key = peakcache::fingerprint(&other).unwrap();
    // What:     `let mut other_found = false;`. Whether non-current measurement landed.
    // Why:      Poll loop records success.
    let mut other_found = false;
    // What:     `for _ in 0..100 { ... }`. Poll briefly for the background worker.
    // Why:      The worker runs on another thread and should finish the tiny fixture.
    for _ in 0..100 {
        // What:     `if controller.peaks.get(&other_key).is_some() { ... }`. Check whether the
        //           non-current track was cached.
        // Why:      Detect sweep completion without sleeping a fixed long time.
        if controller.peaks.get(other_key).is_some() {
            other_found = true;
            break;
        }
        // What:     `std::thread::sleep(Duration::from_millis(50));`. Sleep before polling again.
        // Why:      Avoid busy-spinning while the background worker decodes.
        std::thread::sleep(Duration::from_millis(50));
    }
    // What:     `assert!(other_found);`. Non-current track was measured.
    // Why:      Prove the background sweep actually did work.
    assert!(other_found);
    // What:     `assert!(controller.peaks.get(&current_key).is_none());`. Check current track
    //           remains absent from the cache.
    // Why:      Dedicated current-track measurement, not background sweep, owns that path.
    assert!(controller.peaks.get(current_key).is_none());

    // What:     `let _ = std::fs::remove_file(&cache_path);`. Best-effort cleanup.
    // Why:      Leave no temp cache file behind.
    let _ = std::fs::remove_file(&cache_path);
}

// What:     `#[test] fn rescan_reflects_disk_and_preserves_selection_by_path()`. Drive the
//           `Command::Rescan` reconcile over a real disposable directory.
// Why:      The live-update core must reflect added/removed files and keep the Selected
//           Track selected by path (clearing it only when its file leaves the root).
#[test]
fn rescan_reflects_disk_and_preserves_selection_by_path() {
    // What:     `let nanos = ...as_nanos();`. A unique suffix for a throwaway directory.
    // Why:      Each run gets its own scratch root (THR: verify on a throwaway).
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after Unix epoch")
        .as_nanos();
    // What:     `let dir = crate::test_scratch::scratch_root().join(format!("mp_rescan_{nanos}"));`. The
    //           disposable Source Root.
    // Why:      A real directory so `expand_paths` actually scans it.
    let dir = crate::test_scratch::scratch_root().join(format!("mp_rescan_{nanos}"));
    // What:     Create the disposable source root.
    // Why:      The root must exist before scanning.
    std::fs::create_dir_all(&dir).expect("source fixture directory should be created");
    // What:     two audio-extension files written with one byte each.
    // Why:      `expand_paths` keeps only audio extensions; content is irrelevant here.
    let a = dir.join("a.flac");
    let b = dir.join("b.flac");
    std::fs::write(&a, b"x").expect("first source fixture should be written");
    std::fs::write(&b, b"x").expect("second source fixture should be written");

    // What:     build a silent controller and point its Source Root at the scratch dir.
    // Why:      Drive the real `Rescan` handler without audio or UI.
    let mut controller = test_controller();
    controller.source_root = Some(dir.clone());
    // What:     `controller.handle_command(Command::Rescan);`. Run the reconcile.
    // Why:      Build the queue from the directory.
    controller.handle_command(Command::Rescan);
    // What:     both files present after the scan.
    // Why:      The queue mirrors the directory.
    assert_eq!(controller.queue.tracks().len(), 2);
    assert!(controller.queue.tracks().contains(&a));
    assert!(controller.queue.tracks().contains(&b));

    // What:     select `b` by its scanned position.
    // Why:      Establish a Selected Track to preserve across the next rescan.
    let bi = controller.queue.tracks().iter().position(|p| return *p == b).unwrap();
    controller.queue.play_index(bi);
    assert_eq!(controller.queue.current_path(), Some(&b));

    // What:     add a file that sorts before the others, then rescan.
    // Why:      Adding a file shifts `b`'s index; the selection must follow by path.
    let zero = dir.join("0.flac");
    std::fs::write(&zero, b"x").expect("new source fixture should be written");
    controller.handle_command(Command::Rescan);
    // What:     the new file is in the queue and `b` is still the selection.
    // Why:      Live add is reflected; selection preserved by path despite the index shift.
    assert!(controller.queue.tracks().contains(&zero));
    assert_eq!(controller.queue.current_path(), Some(&b));

    // What:     remove the selected file `b`, then rescan.
    // Why:      The Selected Track's file left the root, so the selection must clear.
    std::fs::remove_file(&b).expect("selected fixture should be removed");
    controller.handle_command(Command::Rescan);
    // What:     `b` is gone from the queue and nothing is selected.
    // Why:      Live remove is reflected; the missing selection is cleared.
    assert!(!controller.queue.tracks().contains(&b));
    assert_eq!(controller.queue.current_path(), None);

    // What:     `std::fs::remove_dir_all(&dir).ok();`. Best-effort cleanup of the scratch dir.
    // Why:      Leave no test droppings.
    std::fs::remove_dir_all(&dir).ok();
}

// What:     `fn a_plain_open_measures_the_first_track_too()`. Open a folder with no preselected file and wait for the sweep.
// Why:      LS9: `set_tracks` leaves the cursor on the first track until the open clears it, and a sweep started before
//           the clear skipped that track as "current". A plain open selects nothing, so every track must be measured.
#[test]
fn a_plain_open_measures_the_first_track_too() {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).expect("system clock should be after Unix epoch").as_nanos();
    let root = crate::test_scratch::scratch_root().join(format!("mp-open-sweep-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&root).expect("the scratch library is created");
    let first = root.join("a.flac");
    let second = root.join("b.mp3");
    std::fs::copy("fixture/tone.flac", &first).expect("the first fixture is copied");
    std::fs::copy("fixture/tone.mp3", &second).expect("the second fixture is copied");
    let cache_path = temp_cache("open-sweep");
    let mut controller = test_controller();
    controller.peaks = CacheHandle::open_at(cache_path.clone());

    controller.handle_command(Command::OpenRoot { root: root.clone(), select: None, play: false });
    assert_eq!(controller.queue.current_path(), None, "a plain open selects nothing");

    let first_key = peakcache::fingerprint(&first).expect("the first fixture has a fingerprint");
    let second_key = peakcache::fingerprint(&second).expect("the second fixture has a fingerprint");
    let mut both_found = false;
    for _ in 0..200 {
        if controller.peaks.get(first_key).is_some() && controller.peaks.get(second_key).is_some() {
            both_found = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(both_found, "the sweep measured the first track as well as the second");

    let _ = std::fs::remove_file(&cache_path);
    std::fs::remove_dir_all(&root).ok();
}
