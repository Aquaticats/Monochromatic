//! Controls for the queue of the landing's syncs:
//! what a publishing landing queues,
//! what a landing that publishes nothing makes
//! at once,
//! and the flush of the index that follows `core.fsync`.

use super::*;
use crate::test_support::{fixture, remove};

/// The names of the syncs a closure's landing queued or waited for,
/// from the events of this thread.
fn events_of(run: impl FnOnce()) -> Vec<String> {
    crate::service_fault::hooks::take_events();
    run();
    return crate::service_fault::hooks::take_events();
}

/// A record file to flush.
fn record(name: &str) -> (std::path::PathBuf, File) {
    let root = fixture(name);
    let path = root.join("record");
    std::fs::write(&path, b"record").expect("record");
    let file = File::open(&path).expect("open");
    return (root, file);
}

/// A landing that publishes queues its flush and the background thread makes it;
/// nothing is made on the caller's thread.
#[test]
fn a_published_landing_queues_and_the_background_thread_flushes() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let (root, file) = record("durable-published");
        let before = BACKGROUND_SYNCED.load(std::sync::atomic::Ordering::SeqCst);
        let events = events_of(|| {
            let deferral = begin(false);
            flush_file(&file, "x").expect("queued");
            deferral.settle(true);
        });
        assert_eq!(events, vec![String::from("sync-queued:x")]);
        drain();
        assert!(BACKGROUND_SYNCED.load(std::sync::atomic::Ordering::SeqCst) > before, "the background thread flushed it");
        remove(root.as_path());
    });
}

/// A landing that publishes nothing flushes what it queued before `settle` returns.
#[test]
fn an_unpublished_landing_forces_what_it_queued() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let (root, file) = record("durable-unpublished");
        let events = events_of(|| {
            let deferral = begin(false);
            flush_file(&file, "x").expect("queued");
            deferral.settle(false);
        });
        assert_eq!(events, vec![String::from("sync-queued:x"), String::from("sync-waited:x")]);
        remove(root.as_path());
    });
}

/// A deferral dropped without settling (a panic in the landing) forces its queue too,
/// and ends the deferral.
#[test]
fn a_dropped_deferral_forces_and_ends() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let (root, file) = record("durable-dropped");
        let events = events_of(|| {
            let deferral = begin(false);
            flush_file(&file, "x").expect("queued");
            drop(deferral);
            flush_file(&file, "y").expect("made now");
        });
        assert_eq!(events, vec![String::from("sync-queued:x"), String::from("sync-waited:x"), String::from("sync-waited:y")]);
        remove(root.as_path());
    });
}

/// Outside a landing a flush is made where it is asked for,
/// as recovery needs it.
#[test]
fn a_flush_outside_a_landing_is_made_now() {
    let (root, file) = record("durable-outside");
    let events = events_of(|| { flush_file(&file, "z").expect("made now"); });
    assert_eq!(events, vec![String::from("sync-waited:z")]);
    remove(root.as_path());
}

/// The index is flushed only when the landing was told it is hardened.
#[test]
fn the_index_is_flushed_only_when_it_is_hardened() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let (root, file) = record("durable-index");
        let events = events_of(|| {
            let deferral = begin(false);
            flush_index(&file, "index").expect("skipped");
            deferral.settle(true);
            let hardened = begin(true);
            flush_index(&file, "index").expect("made now");
            hardened.settle(true);
        });
        assert_eq!(events, vec![String::from("sync-waited:index")]);
        remove(root.as_path());
    });
}

/// What:
///  A control that reads the landing's answer to "is the index flushed before it is installed?"
/// after the landing began
///  without `core.fsync` naming the index.
///  `begin(false)` starts a landing and returns a `Deferral` (a handle that must be settled);
/// `index_is_hardened()` reads
///  the answer.
/// Why:
///  The answer must follow what the landing was told,
/// so a landing begun with `false` reports `false`;
/// a function that always
///  said `true` would have the index flushed where native Git does not flush it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const landing = begin(false);
/// assert(indexIsHardened() === false);
/// landing.settle(true);
/// ```
#[test]
fn a_landing_begun_without_the_index_named_reports_it_unhardened() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        // `let deferral = begin(false);` starts the landing on this thread; `false` says `core.fsync` did not name the index.
        let deferral = begin(false);
        // The answer while the landing runs is the one it was told.
        assert!(!index_is_hardened(), "core.fsync did not name the index, so it is not flushed");
        // `settle(true)` ends the landing as one that published a commit; nothing was queued, so nothing is handed over.
        deferral.settle(true);
    });
}

/// What:
///  A control that reads the same answer after the landing began with `core.fsync` naming the index,
/// again after it
///  settled,
/// and again after a landing that was dropped.
/// Why:
///  A landing told `true` reports `true` (a function that always said `false` would leave the hardened index unflushed),
/// and
///  the thread's answer returns to its default of `true` when a landing ends,
/// however it ends,
/// so the next landing starts
///  from the default and not from the last landing's answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const landing = begin(true);
/// assert(indexIsHardened() === true);
/// landing.settle(true);
/// assert(indexIsHardened() === true);
/// const unhardened = begin(false);
/// unhardened.settle(false);
/// assert(indexIsHardened() === true); // the default is back
/// ```
#[test]
fn a_landing_begun_with_the_index_named_reports_it_hardened_and_the_default_returns_when_it_ends() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        // `begin(true)` says `core.fsync` named the index.
        let hardened = begin(true);
        assert!(index_is_hardened(), "core.fsync named the index, so it is flushed");
        hardened.settle(true);
        assert!(index_is_hardened(), "no landing runs, so the default (flushed) holds");
        // A landing begun with `false` changes the answer; ending it, here by dropping it unsettled, restores the default.
        let unhardened = begin(false);
        assert!(!index_is_hardened(), "the landing was told the index is not named");
        // `drop(x)` ends `x` early, as a panic in the landing would; the deferral's `Drop` forces its queue and ends it.
        drop(unhardened);
        assert!(index_is_hardened(), "a dropped landing leaves the default behind");
    });
}

/// The variable that names the directory the helper below works in;
/// a test run without it does nothing.
const HELPER_DIRECTORY: &str = "CLI_GIT_DURABLE_HELPER_DIRECTORY";

/// Says whether one thread can start in this process.
/// A test run without the variable does nothing.
#[test]
fn thread_probe() {
    if std::env::var_os(HELPER_DIRECTORY).is_none() {
        return;
    }
    let started: bool = std::thread::Builder::new().spawn(|| {}).is_ok();
    println!("{}", if started { "thread-started" } else { "thread-refused" });
}

/// Makes the syncs of a landing,
/// both ways (`force` as a landing that publishes nothing does,
/// `sync_batch` as the background thread does),
/// in a process of its own that the control below gives no thread to start.
/// A test run without the variable does nothing.
#[test]
fn durable_helper() {
    let Some(named) = std::env::var_os(HELPER_DIRECTORY) else { return };
    let directory: std::path::PathBuf = std::path::PathBuf::from(named);
    let pending = |what: &'static str| -> Vec<Pending> {
        return (0..3).map(|_| { return Pending { handle: File::open(directory.join("record")).expect("open"), what }; }).collect();
    };
    let before: u64 = BACKGROUND_SYNCED.load(std::sync::atomic::Ordering::SeqCst);
    force(pending("forced"));
    sync_batch(pending("background"));
    let made: u64 = BACKGROUND_SYNCED.load(std::sync::atomic::Ordering::SeqCst) - before;
    std::fs::write(directory.join("done"), made.to_string()).expect("the helper says it finished");
}

/// A landing whose syncs cannot start a thread makes them on its own thread and does not panic (shipped since 0.5.28:
/// `Scope::spawn`
/// panics when the system refuses the thread,
/// and the panic on the service's thread took the service down,
/// so the commit could not
/// land:
/// `a_memory_limit_close_to_exhaustion_on_the_service_leaves_a_recoverable_repository`,
/// gate of 2026-11-04).
/// The helper runs with
/// a process limit of one,
/// under which every start of a thread is refused.
/// The control skips where the user may exceed the limit.
#[test]
fn a_landing_whose_syncs_cannot_start_a_thread_makes_them_where_it_is_and_does_not_panic() {
    use std::os::unix::process::CommandExt;
    let apply_limit = || -> std::io::Result<()> {
        let mut found = libc::rlimit { rlim_cur: 0, rlim_max: 0 };
        // SAFETY: `getrlimit` and `setrlimit` are async-signal-safe and read and write only this local value.
        unsafe {
            if libc::getrlimit(libc::RLIMIT_NPROC, &mut found) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            let lowered = libc::rlimit { rlim_cur: 1, rlim_max: found.rlim_max };
            if libc::setrlimit(libc::RLIMIT_NPROC, &lowered) != 0 {
                return Err(std::io::Error::last_os_error());
            }
        }
        return Ok(());
    };
    // The probe is a process of the same program that tries to start one thread under the limit and says what happened.
    let test_binary: std::path::PathBuf = std::path::PathBuf::from(std::env::args_os().next().expect("the program this test runs as"));
    let mut probe = std::process::Command::new(test_binary.as_path());
    probe.args(["--exact", "landing::service_durable::tests::thread_probe", "--nocapture", "--test-threads=1"]).env(HELPER_DIRECTORY, "probe").stdin(std::process::Stdio::null()).stderr(std::process::Stdio::null());
    // SAFETY: between fork and exec only `getrlimit` and `setrlimit` run.
    unsafe { probe.pre_exec(apply_limit); }
    if String::from_utf8_lossy(probe.output().expect("the probe runs").stdout.as_slice()).contains("thread-started") {
        eprintln!("skipped: this user may exceed RLIMIT_NPROC, so no start was refused");
        return;
    }
    let (root, _file) = record("durable-no-thread");
    let mut helper = std::process::Command::new(test_binary);
    helper
        .args(["--exact", "landing::service_durable::tests::durable_helper", "--nocapture", "--test-threads=1"])
        .env(HELPER_DIRECTORY, root.as_os_str())
        .stdin(std::process::Stdio::null());
    // SAFETY: between fork and exec only `getrlimit` and `setrlimit` run.
    unsafe { helper.pre_exec(apply_limit); }
    let output = helper.output().expect("the helper runs");
    let said: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
    assert!(output.status.success(), "the helper panicked or failed: {said}");
    assert!(!said.contains("failed to spawn thread"), "{said}");
    assert_eq!(std::fs::read_to_string(root.join("done")).expect("the helper finished").trim(), "3", "the background batch was made here, handle by handle: {said}");
    remove(root.as_path());
}

/// The names of the threads this process has,
/// read from `/proc`.
fn thread_names_of_this_process() -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for task in std::fs::read_dir("/proc/self/task").expect("the tasks of this process").flatten() {
        if let Ok(name) = std::fs::read_to_string(task.path().join("comm")) {
            names.push(name.trim_end().to_string());
        }
    }
    return names;
}

/// The syncs a publishing landing queued are made by a job of the runtime's blocking pool,
/// not by a thread the module started for them.
#[test]
fn the_background_syncs_are_made_without_a_thread_of_their_own() {
    let (root, file) = record("durable-no-own-thread");
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let before = BACKGROUND_SYNCED.load(std::sync::atomic::Ordering::SeqCst);
        let deferral = begin(false);
        flush_file(&file, "x").expect("queued");
        deferral.settle(true);
        drain();
        assert!(BACKGROUND_SYNCED.load(std::sync::atomic::Ordering::SeqCst) > before, "the queued sync was made");
    });
    let own: Vec<String> = thread_names_of_this_process().into_iter().filter(|name| return name == "cli-git-durable").collect();
    assert!(own.is_empty(), "the background syncs started a thread of their own: {own:?}");
    remove(root.as_path());
}

/// What:
///  The flush the index lock marker asks for is queued in the background task and never made on the caller's thread,
///  even outside a landing,
///  and the background task makes it.
/// Why:
///  The marker names the process that holds the lock and nothing else,
///  and its flush sat on every commit's waiting path
///  (`doc/planning/git-policy-cli-btrfs-transaction-waits.md`).
/// A marker a power cut leaves empty or torn names no holder,
/// and the step that runs before a landing removes exactly that file at once
///  (`service_index_unheld_removal.rs`),
///  so the command after such a cut proceeds without waiting out any age.
#[test]
fn the_lock_markers_flush_is_queued_and_the_background_task_makes_it() {
    let (root, file) = record("durable-lock-marker");
    let before = BACKGROUND_SYNCED.load(std::sync::atomic::Ordering::SeqCst);
    let events = events_of(|| {
        crate::service_index_placement::sync_file(&file).expect("queued");
    });
    assert_eq!(events, vec![String::from("sync-queued:lock-marker")], "the flush is queued and none is waited for");
    drain();
    assert!(BACKGROUND_SYNCED.load(std::sync::atomic::Ordering::SeqCst) > before, "the background thread flushed the marker");
    remove(root.as_path());
}
