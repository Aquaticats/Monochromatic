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

/// What:
///  A flush no background task takes is left to the operating system's write-back and never made on the caller's thread.
/// Why:
///  The marker names the process that holds the lock and nothing else,
/// so a power cut that loses it loses a name no one needs,
/// and the
///  step before a landing removes exactly that file at once (`service_index_unheld_removal.rs`);
/// a flush made where it was asked for
///  puts a device's whole wait on the commit's path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const events = listen(() => deferFileTo((batch) => batch)); assert(events === ['sync-queued:lock-marker']);
/// ```
#[test]
fn a_flush_no_background_task_takes_is_never_made_on_the_callers_thread() {
    let (root, file) = record("durable-lock-marker-refused");
    let events = events_of(|| {
        defer_file_with(&|batch: Vec<Pending>| { return Some(batch); }, &file, "lock-marker").expect("queued, or left to the write-back");
    });
    assert_eq!(events, vec![String::from("sync-queued:lock-marker")], "no flush is waited for when no task takes it");
    remove(root.as_path());
}

/// What:
///  A landing that published a commit and whose queue no background task takes waits for no flush either.
/// Why:
///  The owner's decision for the landing's records is "Force at leaving,
/// else background":
/// the person waits for no forced write,
///  and a machine crash loses that attempt as it does with native Git.
/// A queue made on the landing's thread instead would put the whole
///  device's wait back on the commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// inLanding(() => { const d = begin(false); flushFile(f, 'x'); d.settleWith(true, (batch) => batch); }); assert(events === ['sync-queued:x']);
/// ```
#[test]
fn a_published_landing_whose_queue_no_task_takes_waits_for_nothing() {
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let (root, file) = record("durable-published-refused");
        let events = events_of(|| {
            let deferral = begin(false);
            flush_file(&file, "x").expect("queued");
            deferral.settle_with(true, &|batch: Vec<Pending>| { return Some(batch); });
        });
        assert_eq!(events, vec![String::from("sync-queued:x")], "the commit's thread waits for no flush");
        remove(root.as_path());
    });
}

/// The most a control of the exit's wait lets it take, when the ceiling it states is not what the control asserts:
/// far beyond the ceiling, so only a wait without an end reaches it.
const THE_CONTROLS_PATIENCE: Duration = Duration::from_secs(30);

/// The ceiling a control of the exit's wait states when it expects the queue to empty:
/// five seconds, which is far more than the sync of one record takes on this host and less than the patience above.
const A_CEILING_THAT_IS_NOT_REACHED: Duration = Duration::from_secs(5);

/// What:
///  A queue that empties before the ceiling is answered `Made`,
///  and the sync of the record is made.
/// Why:
///   This is the end every ordinary exit reaches:
/// the batch of a landing's records is synced, and the wait answers at once rather than at its ceiling,
/// so the ceiling costs an exit nothing when nothing is slow.
#[test]
fn the_exit_answers_made_for_a_queue_that_emptied() {
    serve_on_again_for_a_control();
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let (root, file) = record("durable-exit-made");
        let before: u64 = BACKGROUND_SYNCED.load(std::sync::atomic::Ordering::SeqCst);
        let deferral = begin(false);
        flush_file(&file, "x").expect("queued");
        deferral.settle(true);
        let began: Instant = Instant::now();
        let came_to: TheSyncsOfTheLastLandings = drain_at_the_exit_within(A_CEILING_THAT_IS_NOT_REACHED);
        let took: Duration = began.elapsed();
        assert_eq!(came_to, TheSyncsOfTheLastLandings::Made, "the batch of a landing's records was synced, so the exit waited for nothing it could not have");
        assert!(BACKGROUND_SYNCED.load(std::sync::atomic::Ordering::SeqCst) > before, "the queued sync was made");
        assert!(took < A_CEILING_THAT_IS_NOT_REACHED, "an exit whose records are synced does not spend its ceiling: {took:?}");
        remove(root.as_path());
    });
}

/// What:
///  A sync held inside the background task is given up at the ceiling, and the exit says so.
/// Why:
///   This is the end the bound exists for:
/// a sync that does not end cannot hold the exit past the ceiling,
/// and the records it leaves are what a kill at that instruction leaves (the module doc says what that is).
/// The hold is the service's own named step, so the state is a state and not a slow disk:
/// `hold_blocking` sits inside the batch, before its syncs, and the barrier behind it is answered only after it.
#[test]
fn the_exit_gives_up_at_its_ceiling_when_a_sync_does_not_end() {
    let held: std::path::PathBuf = crate::test_support::fixture("durable-exit-held");
    std::fs::create_dir_all(held.as_path()).expect("the directory the hold goes in");
    crate::service_hold_a_step_for_a_control::hold_these_steps_for_a_control(&[crate::service_hold_a_step_for_a_control::INSIDE_A_SYNC], held.as_path());
    serve_on_again_for_a_control();
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let (root, file) = record("durable-exit-gave-up");
        let deferral = begin(false);
        flush_file(&file, "x").expect("queued");
        deferral.settle(true);
        let reached: std::path::PathBuf = crate::service_hold_a_step_for_a_control::reached_file(held.as_path(), crate::service_hold_a_step_for_a_control::INSIDE_A_SYNC);
        let waited: Instant = Instant::now();
        while !reached.exists() {
            assert!(waited.elapsed() < THE_CONTROLS_PATIENCE, "the batch never reached the step the control holds");
            std::thread::sleep(Duration::from_millis(5));
        }
        let ceiling: Duration = Duration::from_millis(300);
        let began: Instant = Instant::now();
        let came_to: TheSyncsOfTheLastLandings = drain_at_the_exit_within(ceiling);
        let took: Duration = began.elapsed();
        // The hold is let go before the assertions, so a failing control leaves no thread of this process sleeping.
        std::fs::write(crate::service_hold_a_step_for_a_control::let_go_file(held.as_path(), crate::service_hold_a_step_for_a_control::INSIDE_A_SYNC), b"").expect("the file that lets the held sync go");
        assert_eq!(came_to, TheSyncsOfTheLastLandings::LeftToTheWriteBack, "a sync that did not end within the ceiling is left to the write-back, and the exit says so");
        assert!(took >= ceiling, "the exit waited at least its ceiling for a sync that had not ended: {took:?}");
        assert!(took < THE_CONTROLS_PATIENCE, "the exit gave up at its ceiling: {took:?}");
        remove(root.as_path());
    });
    remove(held.as_path());
}

/// What:
///  A batch that waits for a place of the lane of slow work is synced at once once the service says it is leaving,
///  and the exit is answered `Made` rather than at its ceiling.
/// Why:
///   This is the half of the fix that the bound alone does not give:
/// the places of the lane are kept free for the work a person waits for,
/// and at an exit there is none left to keep them for,
/// so a batch that was waiting for one is let past it instead of being spent as a write-back.
/// Every place is held the way a trash's entries hold them (`run_a_slow_job_within_the_budget`), and the batch is queued
///  after them, so a batch that ran would have had to wait.
#[test]
fn the_exit_lets_a_batch_waiting_for_a_place_of_the_lane_run_without_one() {
    serve_on_again_for_a_control();
    crate::blocking_jobs_within_the_budget::hold_blocking_jobs_to(4);
    let places: usize = crate::blocking_jobs_within_the_budget::places_a_round_may_take();
    let (took_a_place, has_taken) = std::sync::mpsc::channel::<()>();
    let (let_go, held) = std::sync::mpsc::channel::<()>();
    let waiting: std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Receiver<()>>> = std::sync::Arc::new(std::sync::Mutex::new(held));
    let mut holders: Vec<tokio::task::JoinHandle<()>> = Vec::new();
    for _ in 0..places {
        let told = took_a_place.clone();
        let release = std::sync::Arc::clone(&waiting);
        let holder = crate::lazy_runtime::spawn_on_runtime(async move {
            let _held: Result<(), crate::blocking_job::BlockingJobError> = crate::blocking_jobs_within_the_budget::run_a_slow_job_within_the_budget(move || {
                let _said = told.send(());
                let _ = release.lock().expect("the control's release").recv();
            })
            .await;
        });
        holders.push(holder.expect("the runtime has a worker, so slow work can take its place"));
    }
    for _ in 0..places {
        assert!(has_taken.recv_timeout(THE_CONTROLS_PATIENCE).is_ok(), "a slow job never took its place, so the lane was not held");
    }
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let (root, file) = record("durable-exit-past-the-lane");
        let before: u64 = BACKGROUND_SYNCED.load(std::sync::atomic::Ordering::SeqCst);
        let deferral = begin(false);
        flush_file(&file, "x").expect("queued");
        deferral.settle(true);
        // Every place of the lane is held, so the batch asks for one and waits for it.
        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(BACKGROUND_SYNCED.load(std::sync::atomic::Ordering::SeqCst), before, "the lane was held, so the batch of records was still waiting for a place of it");
        let came_to: TheSyncsOfTheLastLandings = drain_at_the_exit_within(A_CEILING_THAT_IS_NOT_REACHED);
        assert!(BACKGROUND_SYNCED.load(std::sync::atomic::Ordering::SeqCst) > before, "the batch waiting for a place of the lane was synced once the service said it was leaving");
        assert_eq!(came_to, TheSyncsOfTheLastLandings::Made, "the exit waited for a batch it had let past the lane, and the batch ended before the ceiling");
        remove(root.as_path());
    });
    let _let_go = let_go.send(());
    for holder in holders {
        holder.abort();
    }
}

/// What:
///  A control's wait for the queue goes on past the bound an exit is given, and ends when the sync does.
/// Why:
///   `drain` is what eight controls call to see the queue empty,
/// and a bound on it would make a control fail for the bound rather than for the queue:
/// a control that met a sync slower than five seconds would count fewer syncs than it made
/// (the review of `1bb658349d` names the five call sites that assert a count right after it).
/// So the wait a control gets is the one `main` had, and this test is what says so:
/// a sync held past the exit's own bound is still waited for.
#[test]
fn a_controls_drain_waits_past_the_bound_an_exit_is_given() {
    let held: std::path::PathBuf = crate::test_support::fixture("durable-drain-held");
    std::fs::create_dir_all(held.as_path()).expect("the directory the hold goes in");
    crate::service_hold_a_step_for_a_control::hold_these_steps_for_a_control(&[crate::service_hold_a_step_for_a_control::INSIDE_A_SYNC], held.as_path());
    serve_on_again_for_a_control();
    let (root, file) = record("durable-drain-waits");
    let before: u64 = BACKGROUND_SYNCED.load(std::sync::atomic::Ordering::SeqCst);
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        let deferral = begin(false);
        flush_file(&file, "x").expect("queued");
        deferral.settle(true);
    });
    let reached: std::path::PathBuf = crate::service_hold_a_step_for_a_control::reached_file(held.as_path(), crate::service_hold_a_step_for_a_control::INSIDE_A_SYNC);
    let waited: Instant = Instant::now();
    while !reached.exists() {
        assert!(waited.elapsed() < THE_CONTROLS_PATIENCE, "the batch never reached the step the control holds");
        std::thread::sleep(Duration::from_millis(5));
    }
    let (answered, told): (std::sync::mpsc::Sender<()>, std::sync::mpsc::Receiver<()>) = std::sync::mpsc::channel::<()>();
    let _draining = std::thread::spawn(move || {
        drain();
        let _said = answered.send(());
    });
    // Longer than the bound an exit gets, so a drain that had been given that bound would have answered by now.
    let past_the_exits_bound: Duration = crate::wait_ceilings::THE_EXITS_SYNCS_ARE_MADE_WITHIN.saturating_add(Duration::from_secs(1));
    let answered_early: bool = told.recv_timeout(past_the_exits_bound).is_ok();
    std::fs::write(crate::service_hold_a_step_for_a_control::let_go_file(held.as_path(), crate::service_hold_a_step_for_a_control::INSIDE_A_SYNC), b"").expect("the file that lets the held sync go");
    let answered_at_all: bool = answered_early || told.recv_timeout(THE_CONTROLS_PATIENCE).is_ok();
    assert!(!answered_early, "drain answered before a held sync was let go, so a control could see fewer syncs than it made");
    assert!(answered_at_all, "drain never answered after the sync was let go");
    assert!(BACKGROUND_SYNCED.load(std::sync::atomic::Ordering::SeqCst) > before, "the sync the control waited for was made");
    remove(root.as_path());
    remove(held.as_path());
}
