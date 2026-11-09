//! What:
//!  Controls for the guard around the linked linter:
//!  a result comes back,
//! a panic comes back as text,
//! a job that shows no sign of work for the window is abandoned,
//!  and a long wait prints notices that name what it waits for.
//! Why:
//!  The linter shares the wrapper's address space,
//! so these are the only things that keep a planted infinite loop
//!  or a panic from hanging or crashing a commit.
//!  Each control plants its fault through the job the guard runs,
//!  and each fails against a guard that runs the job on the caller's thread.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(await runGuarded(() => { throw new Error('planted'); }, ...)).rejects.toEqual({ kind: 'panicked' });
//! ```

use super::{GuardFailure, GuardLimits, NoticeSink, run_guarded};
use crate::test_support::{ActionSink, SharedSink};
use crate::wait_progress::Cadence;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::Arc;
use std::time::Duration;

/// A quiet window short enough for a test to wait out.
/// It is under the sampling interval of the meter,
/// which is fine for a
///  blocked job:
/// it shows nothing at any sample.
const QUIET: Duration = Duration::from_millis(100);

/// A window no control reaches:
/// the jobs that return,
/// panic or are released by a notice always end first.
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


/// What:
///  A notice write that never returns does not defeat the guard's ceiling.
/// Why:
///  A job that keeps working is abandoned at the ceiling (owner,
/// 2026-11-04,
/// "Silence plus a ceiling (Recommended)").
///  The notice is advisory output to a stream whose reader may have stopped draining it,
///  which is the state a full still-open stderr pipe has;
///  waiting for that write would hold the caller past the only deadline that ends such a wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // const sink = new ActionSink(shared, 1, () => { entered.send(); release.recv(); });
/// // const outcome = await onThread(() => runGuarded(blocksForever, { window: 60s, ceiling: 50ms }, cadence(0, 3600s), 'a job', sink));
/// // expect(await receiveWithin(outcome, ceiling * 10)).toEqual({ kind: 'ceiling' });
/// ```
#[test]
fn audit_a_blocked_notice_write_does_not_defeat_the_ceiling() {
    let (entered, notice_entered): (Sender<()>, Receiver<()>) = channel();
    let (release_the_write, held): (Sender<()>, Receiver<()>) = channel();
    let (finished, outcome): (Sender<Result<(), GuardFailure>>, Receiver<Result<(), GuardFailure>>) = channel();
    let ceiling: Duration = Duration::from_millis(50);
    std::thread::spawn(move || {
        let shared: SharedSink = SharedSink::default();
        // The write is entered once the first line is finished and then held, as a full stream holds it.
        let sink: ActionSink = ActionSink::new(&shared, 1, move || {
            let _ = entered.send(());
            let _ = held.recv_timeout(Duration::from_secs(10));
        });
        let mut notices: NoticeSink = Some(Box::new(sink));
        let stopped: Result<(), GuardFailure> = run_guarded(
            || {
                // Blocked, and not quiet for the window this control gives, so only the ceiling can end the wait.
                std::thread::sleep(Duration::from_secs(30));
            },
            GuardLimits { window: LONG_WINDOW, ceiling },
            NOTICE_AT_ONCE,
            "a job",
            &mut notices,
        );
        let _ = finished.send(stopped);
    });
    notice_entered.recv_timeout(Duration::from_secs(5)).expect("the notice write started");
    let answered = outcome.recv_timeout(ceiling * 10);
    // The write is released whatever the answer, so the control leaves no thread holding the sink.
    let _ = release_the_write.send(());
    assert!(
        answered.is_ok(),
        "a notice write must not defeat the guard ceiling: {answered:?}"
    );
    assert_eq!(answered.expect("answered"), Err(GuardFailure::Ceiling(ceiling)));
}

/// What:
///  A notice write that never returns does not keep a finished job's result from its caller.
/// Why:
///  The job has answered,
///  and the notice is advisory:
///  waiting for that write would hold the result past every deadline the caller was promised.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // expect(await receiveWithin(outcome, ceiling * 10)).resolves.toBe(7);
/// ```
#[test]
fn audit_a_blocked_notice_write_does_not_hold_a_finished_result() {
    let (entered, notice_entered): (Sender<()>, Receiver<()>) = channel();
    let (release_the_write, held): (Sender<()>, Receiver<()>) = channel();
    let (finished, outcome): (Sender<Result<u8, GuardFailure>>, Receiver<Result<u8, GuardFailure>>) = channel();
    let ceiling: Duration = Duration::from_millis(50);
    std::thread::spawn(move || {
        let shared: SharedSink = SharedSink::default();
        let sink: ActionSink = ActionSink::new(&shared, 1, move || {
            let _ = entered.send(());
            let _ = held.recv_timeout(Duration::from_secs(10));
        });
        let mut notices: NoticeSink = Some(Box::new(sink));
        let answered: Result<u8, GuardFailure> = run_guarded(
            || {
                return 7;
            },
            GuardLimits { window: LONG_WINDOW, ceiling },
            NOTICE_AT_ONCE,
            "a job",
            &mut notices,
        );
        let _ = finished.send(answered);
    });
    notice_entered.recv_timeout(Duration::from_secs(5)).expect("the notice write started");
    let answered = outcome.recv_timeout(ceiling * 10);
    let _ = release_the_write.send(());
    assert!(
        answered.is_ok(),
        "a notice write must not hold a finished result: {answered:?}"
    );
    assert_eq!(answered.expect("answered"), Ok(7));
}

/// A cadence whose first notice is due before the first look at the job,
///  and whose second is due long after any control ends:
///  so one notice is written and no control depends on a second.
const NOTICE_AT_ONCE: Cadence = Cadence {
    grace: Duration::ZERO,
    interval: Duration::from_secs(3600),
};

/// What:
///  A notice the linter wrote while its job finished is kept for the next wait.
/// Why:
///  The notice is written on a thread of its own,
///  and the job can answer while that write is still in the air;
///  a
///       guard that returned there would drop the sink the notice thread hands back,
///  and every later
///       candidate of the same linter would print no notice at all,
///  which is silence on a slow run rather
///       than a stream nobody reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // const notices = [sink]; await runGuarded(jobThatAnswersAfterItsNotice, ...); expect(notices[0]).toBeDefined()
/// ```
#[test]
fn a_notice_that_finishes_with_the_job_keeps_its_sink() {
    let (written, notice_written): (Sender<()>, Receiver<()>) = channel();
    let shared: SharedSink = SharedSink::default();
    // The first line is finished inside the notice thread, and the job answers at once: the same iteration.
    let signalling: ActionSink = ActionSink::new(&shared, 1, move || {
        let _ = written.send(());
    });
    let mut notices: NoticeSink = Some(Box::new(signalling));
    let outcome: Result<u8, GuardFailure> = run_guarded(
        move || {
            let _ = notice_written.recv_timeout(Duration::from_secs(5));
            return 7;
        },
        limits(LONG_WINDOW),
        NOTICE_AT_ONCE,
        "a job",
        &mut notices,
    );
    assert_eq!(outcome, Ok(7));
    assert!(
        notices.is_some(),
        "the sink a finished notice returned is kept for the next wait"
    );
    // Positive control: the kept sink writes the next wait's own notice.
    let second: Result<(), GuardFailure> = run_guarded(
        || {},
        limits(LONG_WINDOW),
        NOTICE_AT_ONCE,
        "a job",
        &mut notices,
    );
    assert_eq!(second, Ok(()));
    assert_eq!(
        shared.text().lines().count(),
        2,
        "the kept sink wrote the later notice: {}",
        shared.text()
    );
}

/// A job's value comes back,
/// and the job ran on another thread.
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
        &mut Some(Box::new(std::io::sink())),
    );
    assert_eq!(outcome, Ok(true));
}

/// A panic in the job comes back as its message,
/// and the caller is still running.
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
        &mut Some(Box::new(std::io::sink())),
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
        &mut Some(Box::new(std::io::sink())),
    );
    assert_eq!(outcome, Err(GuardFailure::Quiet(QUIET)));
    assert!(!finished.load(Ordering::SeqCst), "the guard answered while the job was blocked");
    drop(release);
}

/// A long wait prints notices that name what it waits for and where it gives up;
///  the third notice releases the job,
/// so the wait ends on that event,
/// whatever the host's speed.
#[test]
fn a_long_wait_prints_notices_naming_the_file() {
    let (release, blocked) = channel::<()>();
    let sink: SharedSink = SharedSink::default();
    let counted: ActionSink = ActionSink::new(&sink, 3, move || {
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
        &mut Some(Box::new(counted)),
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

/// A job that keeps showing work is abandoned at the ceiling all the same,
/// and the guard answers while it still runs:
/// the
///  window judges quiet time,
/// the ceiling judges length (owner,
/// 2026-11-04,
/// "Silence plus a ceiling (Recommended)").
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
        &mut Some(Box::new(std::io::sink())),
    );
    let waited: Duration = began.elapsed();
    release.store(true, Ordering::SeqCst);
    assert_eq!(outcome, Err(GuardFailure::Ceiling(CEILING)));
    assert!(waited >= CEILING && waited < CEILING * 5, "the guard answered after {waited:?}");
    crate::test_support::remove(scratch.as_path());
}

/// What:
///  A system that refuses every thread refuses the guard's thread and says which limit did it.
/// Why:
///  The markdown check of a commit runs the linked linter on a thread of its own, so a refused thread must end the
///  check with the limit that refused it: a person cannot change anything in the command to make a thread appear.
///  The child is refused every thread for real (`RLIMIT_NPROC`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // under prlimit --nproc=1: expect(runGuarded(job, limits, cadence, 'the markdown check')).rejects.toMatchObject({ start: /ulimit -u/ });
/// ```
#[test]
fn where_no_thread_can_start_the_guard_names_the_limit() {
    if crate::no_threads_child_test_support::ran_in_child(module_path!(), "where_no_thread_can_start_the_guard_names_the_limit") {
        return;
    }
    let outcome: Result<u8, GuardFailure> = run_guarded(|| return 7_u8, limits(LONG_WINDOW), FAST, "the markdown check", &mut Some(Box::new(std::io::sink())));
    let failure: GuardFailure = outcome.expect_err("no thread could be started to run the check");
    let GuardFailure::Start(said) = failure else {
        panic!("a refused start is what this control made: {failure:?}");
    };
    assert!(said.contains("ulimit -u") && said.contains("pids.max"), "the refusal names the limits that refused the thread: {said}");
}
