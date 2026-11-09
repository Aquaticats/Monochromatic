//! What:
//!  Controls for the wait that keeps a step's ceiling when nothing the notice holds names the step:
//!  a child that ends by itself,
//!  one the ceiling ends here,
//!  and what that child had started.
//! Why:
//!   The point of this wait is that its signal cannot reach a process that took the step's number,
//!  so the controls must show the step itself ended and reaped,
//!  and the programs below it ended with it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const [outcome, stop] = await waitUntilItExitsTheStopOrTheCeiling(adopt(child), undefined, 200, 'a step');
//! ```

use super::*;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// How long a control waits before it calls a process it did not end alive:
///  longer than a `SIGKILL` on its way takes.
const STILL_RUNNING_BOUND: Duration = Duration::from_millis(500);

/// A step that would outlive every ceiling a control gives it.
fn a_step_that_never_answers() -> Child {
    return crate::test_support::spawn_owned(Command::new("sleep").arg("600").stdin(Stdio::null())).expect("a long-running child");
}

/// Whether a process number names no process any more,
/// which is also how a child that was reaped and one that never existed look.
fn is_gone(number: i32) -> bool {
    return crate::process_tree::member_read(number) == crate::process_tree::MemberRead::Gone;
}

/// The number of a process waits until it is gone,
/// answering whether it went within the bound.
fn went_within(number: i32, bound: Duration) -> bool {
    let deadline: Instant = Instant::now() + bound;
    while Instant::now() < deadline {
        if is_gone(number) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    return is_gone(number);
}

/// What:
///  A step that ends by itself is reported as it ended,
///  with no ceiling stop beside it.
/// Why:
///   A ceiling that records a stop for a step that landed would turn a commit that succeeded into a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // expect(await wait(adopt(sh('true')), undefined, 30_000)).toEqual([{ exited: 0 }, undefined])
/// ```
#[cfg(unix)]
#[test]
fn a_step_that_ends_by_itself_is_reported_with_no_ceiling_stop() {
    let mut command: Command = Command::new("true");
    let step: Child = crate::test_support::spawn_owned(&mut command).expect("a child that ends");
    let (outcome, a_stop): (ChildOutcome, Option<CeilingStop>) = crate::lazy_runtime::run_on_runtime(wait_until_it_exits_the_stop_or_the_ceiling(
        adopt(step),
        None,
        Duration::from_secs(30),
        "a step",
    ))
    .expect("the runtime")
    .expect("the child ends");
    assert_eq!(outcome, ChildOutcome::Exited(0));
    assert_eq!(a_stop, None, "a step that ended by itself is no ceiling stop");
}

/// What:
///  A step that outlasts its ceiling is ended and reaped by this wait,
///  and the stop says how long it lasted.
/// Why:
///   The reap is what makes the signal safe:
///   the step's number cannot pass to another process while this path owns it and has not reaped it,
///  so the control also shows the number names nothing afterwards,
///  which a killed but unreaped child would not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // const pid = sh('sleep 600'); const [outcome, stop] = await wait(adopt(pid), undefined, 200); expect(outcome).toEqual({ signal: 9 }); expect(exists(`/proc/${pid}`)).toBe(false)
/// ```
#[cfg(unix)]
#[test]
fn a_step_that_outlasts_its_ceiling_is_ended_and_reaped_here() {
    let step: Child = a_step_that_never_answers();
    let number: i32 = i32::try_from(step.id()).expect("a process number");
    let ceiling: Duration = Duration::from_millis(200);
    let began: Instant = Instant::now();
    let (outcome, a_stop): (ChildOutcome, Option<CeilingStop>) = crate::lazy_runtime::run_on_runtime(wait_until_it_exits_the_stop_or_the_ceiling(adopt(step), None, ceiling, "a step"))
        .expect("the runtime")
        .expect("the wait answers");
    let lasted: Duration = began.elapsed();
    let stop: CeilingStop = a_stop.expect("the ceiling came");
    assert_eq!(outcome, ChildOutcome::Signaled(libc::SIGKILL), "the step was killed at its ceiling");
    assert!(stop.waited >= ceiling, "the stop is counted from the step's start: {:?}", stop.waited);
    assert!(lasted < Duration::from_secs(20), "the ceiling did not end the step: {lasted:?}");
    assert!(went_within(number, STILL_RUNNING_BOUND), "the step was killed and not reaped");
}

/// What:
///  A program the step started is ended with it,
///  and a walk that could not list the programs below the step says so.
/// Why:
///   A ceiling that ended only the `git commit` leaves a signing program holding the agent's socket and the terminal,
/// and the record must then say that something below may still run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // const step = sh('sleep 600 & echo $!; wait'); const [outcome, stop] = await wait(adopt(step), undefined, 200); expect(stop.belowListed).toBe(true); expect(exists(helper)).toBe(false)
/// ```
#[cfg(target_os = "linux")]
#[test]
fn a_program_the_step_started_is_ended_with_it() {
    let mut command: Command = Command::new("sh");
    command.arg("-c").arg("sleep 600 & echo $!; wait").stdout(Stdio::piped()).stdin(Stdio::null());
    let mut step: Child = crate::test_support::spawn_owned(&mut command).expect("a step with a helper");
    let ceiling: Duration = Duration::from_millis(400);
    let helper: i32 = {
        use std::io::{BufRead, BufReader};
        let output = step.stdout.take().expect("the helper's number");
        let mut line: String = String::new();
        BufReader::new(output).read_line(&mut line).expect("a number");
        line.trim().parse::<i32>().expect("a process number")
    };
    let (outcome, a_stop): (ChildOutcome, Option<CeilingStop>) = crate::lazy_runtime::run_on_runtime(wait_until_it_exits_the_stop_or_the_ceiling(adopt(step), None, ceiling, "a step"))
        .expect("the runtime")
        .expect("the wait answers");
    assert_eq!(outcome, ChildOutcome::Signaled(libc::SIGKILL));
    let stop: CeilingStop = a_stop.expect("the ceiling came");
    assert!(stop.below_listed, "the programs below the step were listed");
    assert!(went_within(helper, STILL_RUNNING_BOUND), "the program the step started was left running");
}

/// What:
///  A stop of the service ends the step first,
///  and the caller is told the stop did it.
/// Why:
///   The ceiling must not take over the answer when the service is shutting down:
///   that stop is reported as itself,
///   so the command says the service stopped it rather than that it ran too long.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // prepareToNotice(); arm; const [result] = await join([wait(adopt(sh('sleep 60')), arm, 60_000), stopAfter(300)]); expect(result).toBeEndedByStop()
/// ```
#[cfg(unix)]
#[test]
fn a_stop_of_the_service_ends_the_step_before_its_ceiling() {
    use crate::stop_of_the_service_noticed_by_its_children::{
        Settled, prepare_to_notice, record_from_a_signal_handler, tell_the_stop_and_wait_for_the_work, wake_the_shutdown_from_a_signal_handler,
    };
    prepare_to_notice().expect("the wake pipe");
    let step: Child = a_step_that_never_answers();
    let result: (std::io::Result<(ChildOutcome, Option<CeilingStop>)>, Settled) = crate::lazy_runtime::run_on_runtime(async {
        // The count is held until the wait answers, as the relay holds it, so the shutdown sees this step as work in
        //  flight and nothing else.
        let (unfinished, stop): (crate::stop_of_the_service_noticed_by_its_children::Unfinished, Option<crate::stop_of_the_service_noticed_by_its_children::StopArm>) =
            crate::child_awaited_until_it_exits_or_the_service_is_stopped::begin_an_awaited_child("a step").expect("counted work");
        assert!(stop.is_some(), "the service hands each waiter an arm");
        return tokio::join!(
            async move {
                let answered = wait_until_it_exits_the_stop_or_the_ceiling(adopt(step), stop, Duration::from_secs(60), "a step").await;
                drop(unfinished);
                return answered;
            },
            async {
                tokio::time::sleep(Duration::from_millis(300)).await;
                assert!(record_from_a_signal_handler(libc::SIGTERM));
                assert!(wake_the_shutdown_from_a_signal_handler());
                return tell_the_stop_and_wait_for_the_work(Duration::from_secs(60)).await;
            }
        );
    })
    .expect("the runtime");
    let error: std::io::Error = result.0.expect_err("a stopped step has no outcome");
    assert!(crate::child_watch::is_ended_by_stop(&error), "{error}");
    assert_eq!(result.1, Settled::Completely);
}
