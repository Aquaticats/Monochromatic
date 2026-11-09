//! Controls for the unit-test fixtures:
//! where they live and that they go away however their test ends (issue 1809).

use super::*;

/// A test thread that panics after creating a fixture leaves nothing behind.
#[test]
fn a_fixture_is_removed_when_its_test_thread_panics() {
    let (sender, receiver) = std::sync::mpsc::channel::<PathBuf>();
    let handle = std::thread::spawn(move || {
        let root: PathBuf = fixture("guard-panic");
        std::fs::write(root.join("file"), "content").expect("file");
        sender.send(root).expect("send the root");
        panic!("the test fails before it removes its fixture");
    });
    assert!(handle.join().is_err(), "the thread panics on purpose");
    let root: PathBuf = receiver.recv().expect("the root");
    assert!(!root.exists(), "{} was left behind", root.display());
}

/// A test thread that ends normally leaves nothing behind either,
/// with or without an explicit `remove`.
#[test]
fn a_fixture_is_removed_when_its_test_thread_ends() {
    let (sender, receiver) = std::sync::mpsc::channel::<PathBuf>();
    std::thread::spawn(move || {
        let kept: PathBuf = fixture("guard-ends");
        let removed: PathBuf = fixture("guard-removed");
        remove(removed.as_path());
        sender.send(kept).expect("send the root");
        sender.send(removed).expect("send the root");
    })
    .join()
    .expect("the thread ends normally");
    for root in receiver.try_iter() {
        assert!(!root.exists(), "{} was left behind", root.display());
    }
}

/// Settling keeps a tree on request,
/// removes it otherwise,
/// and accepts a tree that is already gone.
#[test]
fn settling_keeps_removes_and_accepts_a_tree_that_is_gone() {
    let root: PathBuf = fixture("settle");
    std::fs::write(root.join("file"), "content").expect("file");
    settle_fixture(root.as_path(), true);
    assert!(root.join("file").exists(), "a kept fixture stays");
    settle_fixture(root.as_path(), false);
    assert!(!root.exists(), "a settled fixture goes");
    settle_fixture(root.as_path(), false);
}

/// Fixtures are created beside the build output,
/// never in the system temporary directory,
/// unless the person moved the
/// root on purpose.
#[test]
fn the_fixture_root_is_not_the_system_temporary_directory() {
    if std::env::var_os(FIXTURE_ROOT_VARIABLE).is_some() {
        return;
    }
    let root: PathBuf = fixture_root();
    // The last component is this process's own directory, so the names under it that are not fixtures' own are the
    //  process's: the build output above `test-fixtures` is shared by every checkout that builds here.
    assert!(root.parent().is_some_and(|above| { return above.ends_with("test-fixtures"); }), "{}", root.display());
    assert!(root.ends_with(format!("process-{}", std::process::id()).as_str()), "{}", root.display());
    // The system temporary directory is `TMPDIR` when set and `/tmp` otherwise; reading the variable compares against it
    // without calling the temporary-directory function that the lint configuration bans.
    let mut system: Vec<PathBuf> = vec![PathBuf::from("/tmp"), PathBuf::from("/var/tmp")];
    system.extend(std::env::var_os("TMPDIR").map(PathBuf::from));
    for directory in system {
        assert!(!root.starts_with(&directory), "{} is under the system temporary directory {}", root.display(), directory.display());
    }
}

/// A process a test starts cannot write a core dump:
/// tests make services abort or kill them on purpose,
/// and every dump costs
///  the system's core-dump handler processor time and disk writes on a disk other sessions share (several hundred dumps of
///  `cli-git-service` from test runs on 2026-11-03).
/// The limit is the harness's,
/// set where fixtures are made;
/// the product
///  sets none.
#[test]
fn a_process_a_test_starts_may_not_write_a_core_dump() {
    let _root: PathBuf = fixture("core-limit");
    let output = std::process::Command::new("sh").args(["-c", "ulimit -c; ulimit -H -c"]).output().expect("sh runs");
    assert_eq!(String::from_utf8_lossy(&output.stdout).split_whitespace().collect::<Vec<&str>>(), vec!["0", "0"]);
}

/// A wait whose event has already happened returns at once,
/// without looking at the run.
/// How long the call took cannot say that:
/// a thread the scheduler holds just before it makes a wait
///  that looked slow, and one it holds just after a sleep makes one that slept look quick,
/// so the control reads the
///  looks this thread took.
#[test]
fn a_wait_whose_event_has_happened_returns_at_once() {
    let before: u64 = crate::run_progress::looks_taken_by_this_thread();
    assert!(wait_until(&|| { return true; }));
    assert_eq!(
        crate::run_progress::looks_taken_by_this_thread(),
        before,
        "the wait looked at the run although its event had already happened"
    );
}

/// A wait whose event never happens and whose run shows nothing ends with the window,
/// and says it gave up because the run stopped moving.
#[test]
fn a_wait_gives_up_when_the_run_shows_nothing() {
    let window: std::time::Duration = std::time::Duration::from_millis(300);
    let began: std::time::Instant = std::time::Instant::now();
    let held: bool = wait_until_for(WaitingFor { window, done: &|| { return false; } });
    assert!(!held, "an event that never happened was reported as held");
    assert!(began.elapsed() >= window, "the wait ended before its window passed: {:?}", began.elapsed());
}

/// How much of its window a wait is held for here:
/// two windows' worth of looks,
/// worked out from the window and the interval between looks.
/// Why:
///   The event is delivered by the wait's own predicate after this many looks,
/// so the control holds the worker working for
///  longer than the window without depending on the clock: a thread the scheduler holds only delays the looks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const looksPastTheWindow = 2 * (windowMs / lookEveryMs);
/// ```
fn looks_past_the_window(window: std::time::Duration) -> usize {
    let per_window: u128 = window.as_millis() / WAIT_LOOK_EVERY.as_millis();
    return usize::try_from(per_window * 2).expect("a window that fits a count of looks");
}

/// A wait outlives many windows while the run keeps working,
/// so a slow event is waited for however long it takes.
/// The worker stops only once the wait has looked at the run for two of its windows,
/// which is what the
///  predicate counts: the control cannot pass with a worker that finished before the wait began.
#[test]
fn a_wait_outlives_its_window_while_the_run_keeps_working() {
    let window: std::time::Duration = std::time::Duration::from_millis(200);
    let working: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let still_working: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::clone(&working);
    let worker = std::thread::spawn(move || {
        use std::io::Write as _;
        let mut sink: std::fs::File = std::fs::OpenOptions::new().write(true).open("/dev/null").expect("a sink");
        while still_working.load(std::sync::atomic::Ordering::SeqCst) {
            sink.write_all(&[b'x'; 4096]).expect("a write");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    });
    let looks: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let held_for: usize = looks_past_the_window(window);
    let stop_after = std::sync::atomic::AtomicBool::new(false);
    let began: std::time::Instant = std::time::Instant::now();
    let held: bool = wait_until_for(WaitingFor {
        window,
        done: &|| {
            if looks.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1 >= held_for {
                stop_after.store(true, std::sync::atomic::Ordering::SeqCst);
                working.store(false, std::sync::atomic::Ordering::SeqCst);
            }
            return stop_after.load(std::sync::atomic::Ordering::SeqCst);
        },
    });
    working.store(false, std::sync::atomic::Ordering::SeqCst);
    worker.join().expect("the worker ends");
    assert!(held, "an event the working run delivered was given up on");
    assert!(began.elapsed() >= window, "the wait ended inside its window: {:?}", began.elapsed());
}

/// How long a control waits for two of its waits to give up before it fails the control.
/// Why:
///   A wait the run keeps alive never ends on its own,
/// so the control needs a bound of its own to report the
///  finding instead of leaving its threads behind.
const GIVE_UP_PATIENCE: std::time::Duration = std::time::Duration::from_secs(30);

/// How many waits the control below starts at once.
const WAITS: usize = 2;

/// Two waits for events that never happen end with their own windows:
/// neither takes the other's readings of the run for the run's work.
#[test]
fn two_waits_do_not_keep_each_other_alive() {
    let window: std::time::Duration = std::time::Duration::from_millis(300);
    let stop: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let (told, heard) = std::sync::mpsc::channel::<bool>();
    let mut waits: Vec<std::thread::JoinHandle<()>> = Vec::new();
    for _ in 0..WAITS {
        let given_up: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::clone(&stop);
        let tell = told.clone();
        waits.push(std::thread::spawn(move || {
            let held: bool = wait_until_for(WaitingFor { window, done: &|| { return given_up.load(std::sync::atomic::Ordering::SeqCst); } });
            let _sent = tell.send(held);
        }));
    }
    drop(told);
    let began: std::time::Instant = std::time::Instant::now();
    let mut answers: Vec<bool> = Vec::new();
    while answers.len() < WAITS && began.elapsed() < GIVE_UP_PATIENCE {
        if let Ok(held) = heard.recv_timeout(WAIT_LOOK_EVERY) {
            answers.push(held);
        }
    }
    // Ends a wait that the run kept alive, so the control reports its finding rather than leaving its threads behind.
    stop.store(true, std::sync::atomic::Ordering::SeqCst);
    for wait in waits {
        wait.join().expect("the waiting thread ends");
    }
    assert_eq!(answers, vec![false; WAITS], "{WAITS} waits for events that never happen kept each other alive for {:?} by counting each other's readings as the run's work", began.elapsed());
}

/// The window a control's wait uses is shorter than the watchdog's,
/// so a control reports why it gave up before the run is ended.
#[test]
fn the_controls_window_is_below_the_watchdogs() {
    assert!(STILL_WORKING_WINDOW < QUIET_WINDOW, "{STILL_WORKING_WINDOW:?} is not below {QUIET_WINDOW:?}");
}
