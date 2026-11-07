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

/// The wait for a push that runs for a while is the runtime's: a task beside it runs while the push does.
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

/// The stop of a push that ignores the request is the runtime's too: a task beside it runs through the grace, the kill and the
///  collection. The stop is run alone, so that ticks from the wait before it cannot stand for ticks of the stop.
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

/// The wait for a query is the runtime's: a task beside it runs while the query does.
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
