//! What:
//!  Controls for the guard around the linked linter:
//!  a result comes back, a panic comes back as text, a job that never ends is abandoned at its deadline,
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

use super::{GuardFailure, run_guarded};
use crate::test_support::{ActionSink, SharedSink};
use crate::wait_progress::Cadence;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::time::Duration;

/// A deadline short enough for a test and long enough for a thread to start and answer.
const DEADLINE: Duration = Duration::from_millis(100);

/// A deadline the notice test never reaches.
const LONG_DEADLINE: Duration = Duration::from_secs(60);

/// A cadence whose notices come at the pace of the deadline.
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
        DEADLINE,
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
        DEADLINE,
        FAST,
        "a job",
        &mut std::io::sink(),
    );
    assert_eq!(outcome, Err(GuardFailure::Panicked(String::from(MESSAGE))));
}

/// A job that never returns is abandoned at its deadline:
///  the guard answers while the job is still blocked,
///  and the abandoned thread ends once it is released.
#[test]
fn a_job_that_outlives_its_deadline_is_abandoned() {
    let (release, blocked) = channel::<()>();
    let finished: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let marker: Arc<AtomicBool> = Arc::clone(&finished);
    let outcome: Result<(), GuardFailure> = run_guarded(
        move || {
            // The planted stall: blocked until the test lets go, as an infinite loop would be.
            let _ = blocked.recv();
            marker.store(true, Ordering::SeqCst);
        },
        DEADLINE,
        FAST,
        "a job",
        &mut std::io::sink(),
    );
    assert_eq!(outcome, Err(GuardFailure::TimedOut(DEADLINE)));
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
        LONG_DEADLINE,
        FAST,
        waiting_for,
        &mut counted,
    );
    assert_eq!(outcome, Ok(()));
    let text: String = sink.text();
    assert!(text.lines().count() >= 3, "{text}");
    for line in text.lines() {
        assert!(line.starts_with(format!("cli-git: still waiting for {waiting_for} after ").as_str()), "{text}");
        assert!(
            line.contains(format!("giving up on it after {} seconds", LONG_DEADLINE.as_secs()).as_str()),
            "{text}"
        );
    }
}
