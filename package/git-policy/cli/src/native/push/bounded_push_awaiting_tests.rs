//! Controls that the waits of a push and of a query are awaited on the runtime and hold no thread:
//!  a task beside the wait keeps running while the child runs and while a stop ends it.

use super::*;
use crate::test_support::scratch_environment;
use crate::ticker_beside_a_wait::with_a_ticker_beside;

/// A shell command running `script`.
#[cfg(unix)]
fn shell(script: &str) -> Command {
    let mut command: Command = Command::new("sh");
    command.arg("-c").arg(script);
    return command;
}

/// The wait for a push that runs for a while is the runtime's:
/// a task beside it runs while the push does.
#[cfg(unix)]
#[test]
fn the_wait_for_a_running_push_lets_other_tasks_run() {
    let (attempt, ticks): (Result<PushAttempt, SingleFlightError>, u32) = with_a_ticker_beside(run_bounded_push_awaiting(
        shell("sleep 0.3; echo done"),
        Patience::Fixed(Duration::from_secs(60)),
        Duration::from_secs(1),
        scratch_environment().as_slice(),
    ));
    let finished: PushAttempt = attempt.expect("runs");
    assert_eq!(finished.exit_code, 0);
    assert_eq!(finished.output, "done");
    assert!(ticks > 0, "the wait for the push held the thread: the task beside it never ran");
}

/// The stop of a push that ignores the request is the runtime's too:
/// a task beside it runs through the grace,
/// the kill and the
///  collection.
/// The stop is run alone,
/// so that ticks from the wait before it cannot stand for ticks of the stop.
#[cfg(unix)]
#[test]
fn the_stop_of_a_push_that_ignores_the_request_lets_other_tasks_run() {
    use std::io::BufRead;
    let mut command: Command = shell("trap '' TERM; echo ready; exec sleep 30");
    command.stdout(Stdio::piped());
    let locks: LockLedger = LockLedger::before_start(&command);
    let mut started: Child = command.spawn().expect("start the push");
    // The request is sent only once the shell ignores it, so the stop has the whole grace to wait through.
    let mut ready: String = String::new();
    std::io::BufReader::new(started.stdout.take().expect("the piped output")).read_line(&mut ready).expect("the ready line");
    assert_eq!(ready, "ready\n");
    let ((), ticks): ((), u32) = with_a_ticker_beside(async move {
        stop_push(adopt(started), Duration::from_millis(300), locks).await;
    });
    assert!(ticks > 0, "the stop of the push held the thread: the task beside it never ran");
}

/// The wait for a query is the runtime's:
/// a task beside it runs while the query does.
#[cfg(unix)]
#[test]
fn the_wait_for_a_running_query_lets_other_tasks_run() {
    let (output, ticks): (Result<crate::bounded_query::QueryOutput, SingleFlightError>, u32) = with_a_ticker_beside(
        crate::bounded_query::run_bounded_query_awaiting(
            shell("sleep 0.3; echo out"),
            "fixture",
            Duration::from_secs(60),
            Duration::from_secs(1),
            scratch_environment().as_slice(),
        ),
    );
    assert_eq!(output.expect("runs").stdout, b"out\n".to_vec());
    assert!(ticks > 0, "the wait for the query held the thread: the task beside it never ran");
}

/// The service's stop, told through the shutdown,
/// after `delay`:
/// the handler's record,
/// its wake,
/// and the shutdown task that tells the waiters,
/// run to the end.
#[cfg(unix)]
async fn the_stop_told_after(delay: Duration) -> crate::stop_of_the_service_noticed_by_its_children::Settled {
    use crate::stop_of_the_service_noticed_by_its_children::{record_from_a_signal_handler, tell_the_stop_and_wait_for_the_work, wake_the_shutdown_from_a_signal_handler};
    tokio::time::sleep(delay).await;
    assert!(record_from_a_signal_handler(libc::SIGTERM));
    assert!(wake_the_shutdown_from_a_signal_handler());
    return tell_the_stop_and_wait_for_the_work(Duration::from_secs(60)).await;
}

/// A push that is running when the service is stopped is ended and queued for the next service,
/// as a push stopped at its deadline is:
/// the answer carries the stopped exit code and the queued line,
/// and the shutdown finds nothing left unfinished.
/// (Each control runs in a process of its own, because the stop is process-wide state.)
#[cfg(unix)]
#[test]
fn a_push_running_when_the_service_is_stopped_is_ended_and_queued() {
    use crate::stop_of_the_service_noticed_by_its_children::{Settled, prepare_to_notice};
    prepare_to_notice().expect("the wake pipe");
    let began: std::time::Instant = std::time::Instant::now();
    let environment = scratch_environment();
    let (attempt, settled): (Result<PushAttempt, SingleFlightError>, Settled) = crate::lazy_runtime::run_on_runtime(async {
        return tokio::join!(
            run_bounded_push_awaiting(shell("exec sleep 60"), Patience::Fixed(Duration::from_secs(120)), Duration::from_secs(2), environment.as_slice()),
            the_stop_told_after(Duration::from_millis(300))
        );
    })
    .expect("the runtime");
    let finished: PushAttempt = attempt.expect("a stopped push is an answer, not a failure to start");
    assert_eq!(finished.exit_code, STOPPED_EXIT_CODE, "{}", finished.output);
    assert!(finished.output.contains("service was stopped") && finished.output.contains(PUSH_QUEUED_LINE), "{}", finished.output);
    assert_eq!(settled, Settled::Completely);
    assert!(began.elapsed() < Duration::from_secs(30), "the push was ended, not waited for: {:?}", began.elapsed());
}

/// A push that would begin after the stop is not started,
/// and is queued the same way.
#[cfg(unix)]
#[test]
fn a_push_that_would_begin_after_the_stop_is_not_started_and_is_queued() {
    use crate::stop_of_the_service_noticed_by_its_children::{prepare_to_notice, record_from_a_signal_handler};
    prepare_to_notice().expect("the wake pipe");
    assert!(record_from_a_signal_handler(libc::SIGTERM));
    let marker: std::path::PathBuf = crate::test_support::fixture_root().join("refused-push-ran");
    let attempt: Result<PushAttempt, SingleFlightError> = crate::lazy_runtime::run_on_runtime(run_bounded_push_awaiting(
        shell(format!("touch '{}'", marker.display()).as_str()),
        Patience::Fixed(Duration::from_secs(60)),
        Duration::from_secs(1),
        scratch_environment().as_slice(),
    ))
    .expect("the runtime");
    let refused: PushAttempt = attempt.expect("a refused push is an answer");
    assert_eq!(refused.exit_code, STOPPED_EXIT_CODE, "{}", refused.output);
    assert!(refused.output.contains("not started") && refused.output.contains(PUSH_QUEUED_LINE), "{}", refused.output);
    assert!(!marker.exists(), "the push never ran");
}

/// A query that is running when the service is stopped is ended,
/// and the caller is told the stop did it.
#[cfg(unix)]
#[test]
fn a_query_running_when_the_service_is_stopped_is_ended_and_says_the_stop_did_it() {
    use crate::stop_of_the_service_noticed_by_its_children::prepare_to_notice;
    prepare_to_notice().expect("the wake pipe");
    let environment = scratch_environment();
    let (output, _settled) = crate::lazy_runtime::run_on_runtime(async {
        return tokio::join!(
            crate::bounded_query::run_bounded_query_awaiting(shell("exec sleep 60"), "fixture", Duration::from_secs(120), Duration::from_secs(2), environment.as_slice()),
            the_stop_told_after(Duration::from_millis(300))
        );
    })
    .expect("the runtime");
    let error: SingleFlightError = output.expect_err("a stopped query has no output");
    match error {
        SingleFlightError::Start(cause) => assert!(crate::child_watch::is_ended_by_stop(&cause), "{cause}"),
        other => panic!("the stop is told apart from a failure of the query: {other:?}"),
    }
}
