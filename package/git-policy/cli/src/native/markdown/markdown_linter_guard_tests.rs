//! What:
//!  Controls for the guard around the linked linter:
//!  a result comes back, a panic comes back as text, a job that shows no sign of work for the window is abandoned,
//!  and a long wait prints notices that name what it waits for.
//! Why:
//!  The linter shares the wrapper's address space, so these are the only things that keep a planted infinite loop
//!  or a panic from hanging or crashing a commit.
//!  Each control plants its fault through the job the guard runs,
//!  and each fails against a guard that runs the job on the caller's thread.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(await runGuarded(() => { throw new Error('planted'); }, ...)).rejects.toEqual({ kind: 'panicked' });
//! ```

use super::{GuardFailure, GuardLimits, run_guarded};
use crate::test_support::{ActionSink, SharedSink};
use crate::wait_progress::Cadence;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::time::Duration;

/// A quiet window short enough for a test to wait out. It is under the sampling interval of the meter, which is fine for a
///  blocked job: it shows nothing at any sample.
const QUIET: Duration = Duration::from_millis(100);

/// A window no control reaches: the jobs that return, panic or are released by a notice always end first.
const LONG_WINDOW: Duration = Duration::from_secs(60);

/// A ceiling no control reaches unless the control is about the ceiling.
const ROOMY: Duration = Duration::from_secs(3600);

/// The limits of a control that is about the quiet window only.
fn limits(window: Duration) -> GuardLimits {
    return GuardLimits { window, ceiling: ROOMY };
}

/// A cadence whose notices come at the pace of the short window.
const FAST: Cadence = Cadence {
    grace: Duration::from_millis(5),
    interval: Duration::from_millis(5),
};

/// A job's value comes back, and the job ran on another thread.
#[test]
fn a_finished_job_returns_its_value_from_another_thread() {
    let caller = std::thread::current().id();
    let outcome: Result<bool, GuardFailure> = run_guarded(
        move || {
            return std::thread::current().id() != caller;
        },
        limits(LONG_WINDOW),
        FAST,
        "a job",
        &mut std::io::sink(),
    );
    assert_eq!(outcome, Ok(true));
}

/// A panic in the job comes back as its message, and the caller is still running.
#[test]
fn a_panic_comes_back_as_text() {
    const MESSAGE: &str = "planted panic";
    let outcome: Result<(), GuardFailure> = run_guarded(
        || {
            panic!("{MESSAGE}");
        },
        limits(LONG_WINDOW),
        FAST,
        "a job",
        &mut std::io::sink(),
    );
    assert_eq!(outcome, Err(GuardFailure::Panicked(String::from(MESSAGE))));
}

/// A job that shows no sign of work for the window is abandoned:
///  the guard answers while the job is still blocked,
///  and the abandoned thread ends once it is released.
#[test]
fn a_job_that_shows_no_sign_of_work_is_abandoned() {
    let (release, blocked) = channel::<()>();
    let finished: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let marker: Arc<AtomicBool> = Arc::clone(&finished);
    let outcome: Result<(), GuardFailure> = run_guarded(
        move || {
            // The planted stall: blocked until the test lets go, as an infinite loop would be.
            let _ = blocked.recv();
            marker.store(true, Ordering::SeqCst);
        },
        limits(QUIET),
        FAST,
        "a job",
        &mut std::io::sink(),
    );
    assert_eq!(outcome, Err(GuardFailure::Quiet(QUIET)));
    assert!(!finished.load(Ordering::SeqCst), "the guard answered while the job was blocked");
    drop(release);
}

/// A long wait prints notices that name what it waits for and where it gives up;
///  the third notice releases the job, so the wait ends on that event, whatever the host's speed.
#[test]
fn a_long_wait_prints_notices_naming_the_file() {
    let (release, blocked) = channel::<()>();
    let sink: SharedSink = SharedSink::default();
    let mut counted: ActionSink = ActionSink::new(&sink, 3, move || {
        let _ = release.send(());
    });
    let waiting_for: &str = "monochromatic-lint to check docs/a.md";
    let outcome: Result<(), GuardFailure> = run_guarded(
        move || {
            let _ = blocked.recv();
        },
        limits(LONG_WINDOW),
        FAST,
        waiting_for,
        &mut counted,
    );
    assert_eq!(outcome, Ok(()));
    let text: String = sink.text();
    assert!(text.lines().count() >= 3, "{text}");
    for line in text.lines() {
        assert!(line.starts_with(format!("monochromatic-git-policy-cli: still waiting for {waiting_for} after ").as_str()), "{text}");
        assert!(
            line.contains(format!("giving up on it only after {} seconds with no sign of work", LONG_WINDOW.as_secs()).as_str()),
            "{text}"
        );
    }
}

/// A job that keeps showing work is abandoned at the ceiling all the same, and the guard answers while it still runs: the
///  window judges quiet time, the ceiling judges length (owner, 2026-11-04, "Silence plus a ceiling (Recommended)").
#[test]
fn a_job_that_keeps_working_is_abandoned_at_its_ceiling() {
    const CEILING: Duration = Duration::from_secs(2);
    const WORKING_WINDOW: Duration = Duration::from_millis(1200);
    let release: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let marker: Arc<AtomicBool> = Arc::clone(&release);
    let scratch: std::path::PathBuf = crate::test_support::fixture("guard-ceiling");
    let progress: std::path::PathBuf = scratch.join("progress");
    let began: std::time::Instant = std::time::Instant::now();
    let outcome: Result<(), GuardFailure> = run_guarded(
        move || {
            // Writes a file a round, with a pause between, until the control lets go: never quiet, no busy loop.
            let mut round: u32 = 0;
            while !marker.load(Ordering::SeqCst) {
                std::fs::write(&progress, round.to_string()).expect("the fixture writes its progress");
                round += 1;
                std::thread::sleep(Duration::from_millis(100));
            }
        },
        GuardLimits { window: WORKING_WINDOW, ceiling: CEILING },
        FAST,
        "a job",
        &mut std::io::sink(),
    );
    let waited: Duration = began.elapsed();
    release.store(true, Ordering::SeqCst);
    assert_eq!(outcome, Err(GuardFailure::Ceiling(CEILING)));
    assert!(waited >= CEILING && waited < CEILING * 5, "the guard answered after {waited:?}");
    crate::test_support::remove(scratch.as_path());
}
