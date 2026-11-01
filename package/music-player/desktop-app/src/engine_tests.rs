// What:     Integration test for `engine.rs`, pulled in by
//           `#[cfg(test)] #[path = "engine_tests.rs"] mod tests;` at the bottom of
//           `engine.rs`. Compiles only under `cargo nextest run` / `cargo test`.
// Why:      Cross the WHOLE live-update seam in one test: the file watcher's change
//           callback -> `CommandSender::send(Command::Rescan)` -> channel -> `unpark`
//           wakes the parked worker -> drain loop -> `Controller`'s `Rescan` handler ->
//           `Update::Queue`. Every other test exercises only one half of this (watch_tests
//           proves notify->callback; controller_tests calls `handle_command` directly), so
//           nothing else proves the parked worker actually wakes and re-derives the queue.

// What:     `use super::*;` bring the parent `engine` module's items (Engine) and the names
//           it imports (Command, Update) into scope.
// Why:      The test spawns an `Engine`, sends a `Command`, and matches an `Update`.
use super::*;

// What:     `use std::sync::mpsc;`. Channel module.
// Why:      The update callback forwards queue lengths to the test thread over a channel.
use std::sync::mpsc;

// What:     `use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};`. A span, a
//           monotonic clock reading, a wall clock, and the epoch reference.
// Why:      `Duration` bounds the wait, `Instant` builds the deadline, `SystemTime`/
//           `UNIX_EPOCH` build a unique throwaway directory name.
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// What:     `fn wait_for_len(rx: &mpsc::Receiver<usize>, want: usize, secs: u64) -> bool`.
//           Read queue-length updates until one equals `want`, or `secs` elapse. Returns
//           whether `want` was seen in time.
// Why:      The engine emits several updates (now-playing, position, queue); the test cares
//           only about a `Queue` update of a specific length, and an OS filesystem event is
//           inherently async, so a bounded wait that skips other lengths is the correct way
//           to await it.
fn wait_for_len(rx: &mpsc::Receiver<usize>, want: usize, secs: u64) -> bool {
    // What:     `let deadline = Instant::now() + Duration::from_secs(secs);`. The instant the
    //           wait gives up.
    // Why:      Bound total waiting across however many non-matching updates arrive first.
    let deadline = Instant::now() + Duration::from_secs(secs);
    // What:     `loop { ... }`. Keep receiving until a match, the deadline, or a closed
    //           channel.
    // Why:      A `while`-style cursor over an unbounded event stream (not a structural
    //           walk), so a loop, not recursion.
    loop {
        // What:     `let now = Instant::now(); if now >= deadline { return false; }`. Stop if
        //           time is up.
        // Why:      Never wait past the budget.
        let now = Instant::now();
        if now >= deadline {
            return false;
        }
        // What:     `match rx.recv_timeout(deadline - now) { ... }`. Wait for the next update
        //           for the remaining budget.
        // Why:      Shrink the per-wait timeout as the deadline nears so total wait stays
        //           bounded.
        match rx.recv_timeout(deadline - now) {
            // What:     `Ok(len) if len == want => return true`. The wanted queue length
            //           arrived.
            // Why:      Success: the seam delivered the expected `Update::Queue`.
            Ok(len) if len == want => return true,
            // What:     `Ok(_) => continue`. A different update (a different length): ignore
            //           and keep waiting.
            // Why:      Only the target length proves the assertion; others are noise.
            Ok(_) => continue,
            // What:     `Err(_) => return false`. Timed out or the engine dropped its sender.
            // Why:      Either way the wanted update will not come.
            Err(_) => return false,
        }
    }
}

// What:     `#[test] fn idle_root_emits_no_extra_queue_updates()`. Open a root holding real
//           audio fixtures, then assert that with NO filesystem change the engine emits the
//           one open-scan `Queue` and nothing more, across several debounce windows.
// Why:      Regression guard for the rescan feedback loop: the background peak measurement (and
//           the decoder, during playback) reads the fixtures UNDER the watched root, and on
//           Linux inotify reports those reads as access events. Before the empty-diff guard in
//           the `Rescan` handler, each such event drove a `Rescan` that re-emitted `Queue`
//           (snapping the UI back to the first page) and re-measured (re-reading the files),
//           spinning a ~500ms loop that made every page-tab selection bounce to the first tab.
#[test]
fn idle_root_emits_no_extra_queue_updates() {
    // What:     Build a unique suffix for the throwaway directory.
    // Why:      Isolate this run's scratch root (THR: verify on a throwaway).
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after Unix epoch")
        .as_nanos();
    // What:     Build the source root path under the system temp directory.
    // Why:      A real directory the OS watcher can observe.
    let dir = crate::test_scratch::scratch_root().join(format!("mp_idle_{nanos}"));
    // What:     Create the source root before opening it.
    // Why:      The Source Root must exist to be scanned and watched.
    std::fs::create_dir_all(&dir).expect("source fixture directory should be created");
    // What:     `for name in [...] { std::fs::copy(manifest/fixture/name, dir/name) }`. Copy
    //           two REAL fixtures (not empty files) into the root. `CARGO_MANIFEST_DIR` is the
    //           package dir, so `fixture/` resolves regardless of the test's working directory.
    // Why:      The peak measurement must actually decode (read) the files to reproduce the
    //           read-driven watcher events that the loop fed on; empty files decode to nothing.
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for name in ["tone.flac", "tone.opus"] {
        std::fs::copy(manifest.join("fixture").join(name), dir.join(name))
            .expect("audio fixture should be copied");
    }

    // What:     `let (tx, rx) = mpsc::channel::<usize>();`. A channel of list-update lengths.
    // Why:      The callback forwards each `Queue`/`Reconciled` length; the test counts them.
    let (tx, rx) = mpsc::channel::<usize>();
    // What:     `let engine = Engine::spawn_with_cache(move |update| { ... }, CacheHandle::open_degraded());`.
    //           Start the real worker (wiring the watcher in `run`) with a degraded, no-disk
    //           cache so the test never opens or creates the real peaks.db.
    // Why:      Exercise the whole live-update seam, not a stub, without touching real state.
    let engine = Engine::spawn_with_cache(
        move |update| {
            // What:     `match &update { Queue(p) | Reconciled { names: p, .. } => send(p.len()), _ => {} }`.
            //           Forward the length of BOTH queue-list updates: a fresh `Queue` and a
            //           live `Reconciled`.
            // Why:      A resurfaced rescan loop would now show up as spontaneous `Reconciled`
            //           updates, not `Queue`, so the guard must watch both.
            let len = match &update {
                Update::Queue(paths) => Some(paths.len()),
                Update::Reconciled { names, .. } => Some(names.len()),
                _ => None,
            };
            if let Some(queue_len) = len {
                let _ = tx.send(queue_len);
            }
        },
        CacheHandle::open_degraded(),
    );

    // What:     `engine.send(Command::OpenRoot { ... select: None, play: false });`. Open the
    //           root with nothing cued and paused, arming the watcher.
    // Why:      The default UI state (no auto-selected track), under which the loop's `Queue`
    //           updates reset the page with nothing to follow back to it.
    engine.send(Command::OpenRoot {
        root: dir.clone(),
        select: None,
        play: false,
    });

    // What:     `assert_eq!(rx.recv_timeout(Duration::from_secs(10)).ok(), Some(2), ...)`. The
    //           open scan must report exactly one `Queue(2)`.
    // Why:      Establish the baseline: one legitimate queue update, and the watch is now armed.
    assert_eq!(
        rx.recv_timeout(Duration::from_secs(10)).ok(),
        Some(2),
        "expected one Queue(2) from the open scan"
    );

    // What:     `let extra = rx.recv_timeout(Duration::from_secs(3));`. Wait several debounce
    //           windows (the debounce is 500ms) for ANY further queue update, changing nothing
    //           on disk.
    // Why:      The peak measurement's reads fire the watcher; the empty-diff guard must turn
    //           the resulting `Rescan` into a no-op, so this wait must TIME OUT.
    let extra = rx.recv_timeout(Duration::from_secs(3));
    // What:     `assert!(extra.is_err(), ...)`. A received value means a spontaneous `Queue`
    //           update arrived (the loop resurfaced); only a timeout passes.
    // Why:      Encode the regression: no rescan storm from the app's own reads.
    assert!(
        extra.is_err(),
        "a Queue update arrived with no filesystem change ({extra:?}); the rescan feedback loop resurfaced"
    );

    // What:     `drop(engine);` then `std::fs::remove_dir_all(&dir).ok();`. Stop the worker and
    //           remove the scratch root.
    // Why:      Tear down cleanly and leave no test droppings.
    drop(engine);
    std::fs::remove_dir_all(&dir).ok();
}

// What:     `#[test] fn watcher_drives_rescan_update_through_engine()`. Open a throwaway
//           directory as the Source Root through a real `Engine`, then create a file in it
//           and assert a `Queue` update reflecting the new file arrives.
// Why:      Prove the live-update path end to end: the watcher actually wakes the parked
//           worker and the worker actually re-derives and re-emits the queue.
#[test]
fn watcher_drives_rescan_update_through_engine() {
    // What:     Build a unique suffix for the throwaway directory.
    // Why:      Isolate this run's scratch root (THR: verify on a throwaway).
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after Unix epoch")
        .as_nanos();
    // What:     Build the source root path under the system temp directory.
    // Why:      A real directory the OS watcher can observe.
    let dir = crate::test_scratch::scratch_root().join(format!("mp_engine_{nanos}"));
    // What:     Create the source root before opening it.
    // Why:      The Source Root must exist to be scanned and watched.
    std::fs::create_dir_all(&dir).expect("source fixture directory should be created");
    // What:     Write one zero-byte audio file before opening the root.
    // Why:      Give the initial scan exactly one track to report.
    std::fs::write(dir.join("a.flac"), b"").expect("first audio fixture should be written");

    // What:     `let (tx, rx) = mpsc::channel::<usize>();`. A channel of list-update lengths.
    // Why:      The engine's update callback runs on the worker thread; it forwards each
    //           `Queue`/`Reconciled` length so the test thread can await specific lengths.
    let (tx, rx) = mpsc::channel::<usize>();
    // What:     `let engine = Engine::spawn_with_cache(move |update| { ... }, CacheHandle::open_degraded());`.
    //           Start the worker with a degraded, no-disk cache and a callback that forwards
    //           `Queue` AND `Reconciled` lengths and drops other updates.
    // Why:      Exercise the real `Engine` (which wires the watcher in `run`), not a stub,
    //           without touching real state. The OPEN reports a `Queue`; the live RESCAN now
    //           reports a `Reconciled`.
    let engine = Engine::spawn_with_cache(
        move |update| {
            // What:     `match &update { Queue(p) | Reconciled { names: p, .. } => send(p.len()), _ => {} }`.
            //           Forward the count for both list updates; ignore now-playing/position.
            //           `let _` drops the send error that occurs only after the test thread is gone.
            // Why:      The test asserts on queue size across the open (`Queue`) and the live
            //           rescan (`Reconciled`).
            let len = match &update {
                Update::Queue(paths) => Some(paths.len()),
                Update::Reconciled { names, .. } => Some(names.len()),
                _ => None,
            };
            if let Some(queue_len) = len {
                let _ = tx.send(queue_len);
            }
        },
        CacheHandle::open_degraded(),
    );

    // What:     `engine.send(Command::OpenRoot { root: dir.clone(), select: None, play: false });`.
    //           Open the directory as the Source Root. The handler scans it (emitting
    //           `Update::Queue`) AND arms the watcher on it.
    // Why:      Establish the watched root; the resulting `Queue(1)` also confirms the watch
    //           is armed (the handler watches before it emits).
    engine.send(Command::OpenRoot {
        root: dir.clone(),
        select: None,
        play: false,
    });

    // What:     `const WAIT_SECS: u64 = 10;`. The bounded wait for each expected update. The
    //           debounce window is 500ms; ten seconds is generous slack for OS event
    //           delivery under load.
    // Why:      A fixed, comfortable upper bound keeps the test reliable without hanging.
    const WAIT_SECS: u64 = 10;
    // What:     `assert!(wait_for_len(&rx, 1, WAIT_SECS), ...)`. The open scan must report one
    //           track.
    // Why:      Gate the second phase on the open having finished. The watch registers on its own thread
    //           and asks for one rescan when it is armed, so a file created before that is still reconciled.
    assert!(
        wait_for_len(&rx, 1, WAIT_SECS),
        "expected Queue(1) after opening a root with one audio file"
    );

    // What:     Add a second audio file to the watched root.
    // Why:      This on-disk change is what the watcher must turn into a live `Rescan`.
    std::fs::write(dir.join("b.flac"), b"").expect("second audio fixture should be written");
    // What:     `assert!(wait_for_len(&rx, 2, WAIT_SECS), ...)`. A list update of length 2 must
    //           arrive without any further command from the test: watcher -> Rescan ->
    //           re-derived queue -> `Reconciled` emit (the live rescan path no longer emits
    //           `Queue`, which is reserved for a fresh open/restore).
    // Why:      This is the seam under test; only an end-to-end pass proves the parked worker
    //           woke and re-scanned. The non-empty diff (1 -> 2 files) is what makes the rescan
    //           emit instead of no-op'ing.
    assert!(
        wait_for_len(&rx, 2, WAIT_SECS),
        "expected a Reconciled(2) after a file appeared in the watched root (live rescan seam)"
    );

    // What:     `drop(engine);` then `std::fs::remove_dir_all(&dir).ok();`. Stop the worker
    //           (its `Drop` sends `Quit`, unparks, and joins) and remove the scratch root.
    // Why:      Tear down the watcher and audio output and leave no test droppings.
    drop(engine);
    std::fs::remove_dir_all(&dir).ok();
}
