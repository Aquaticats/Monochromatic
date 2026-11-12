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
use crate::child_awaited_through_a_pidfd::adopt;
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
        Ceilings::of(Duration::from_secs(30)),
        "a step",
        None,
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
    let (outcome, a_stop): (ChildOutcome, Option<CeilingStop>) = crate::lazy_runtime::run_on_runtime(wait_until_it_exits_the_stop_or_the_ceiling(adopt(step), None, Ceilings::of(ceiling), "a step", None))
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
///  A step that ended by itself between the ceiling and the signal is reported by its own status,
///  with no ceiling stop beside it.
/// Why:
///   That interval is a few instructions wide on a real host,
///  and a step that finishes there landed:
///  reading its reaped status as a ceiling kill turns a preparation that succeeded into a failure with nothing committed.
///  A control holds the step at that instruction,
///  so the order is the control's and not a clock's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // const pid = sh('sleep 0.2; exit 0'); endTheStepBeforeTheSignal(); const [outcome, stop] = await wait(adopt(pid), undefined, 20); expect(outcome).toEqual({ exited: 0 }); expect(stop).toBeUndefined()
/// ```
#[cfg(target_os = "linux")]
#[test]
fn a_step_that_ended_before_the_signal_is_reported_by_its_own_status() {
    let (outcome, a_stop): (ChildOutcome, Option<CeilingStop>) = crate::lazy_runtime::run_on_runtime(async {
        // The step ends by itself well after the ceiling, and was adopted inside the runtime so the wait holds its
        //  descriptor, which is what the control looks at.
        let mut command: Command = Command::new("sh");
        command.arg("-c").arg("sleep 0.2; exit 0").stdin(Stdio::null());
        let step: Child = crate::test_support::spawn_owned(&mut command).expect("a step that ends by itself");
        a_control_ends_the_step_here();
        return wait_until_it_exits_the_stop_or_the_ceiling(adopt(step), None, Ceilings::of(Duration::from_millis(20)), "a step", None).await.expect("the wait answers");
    })
    .expect("the runtime");
    assert_eq!(outcome, ChildOutcome::Exited(0), "a step that finished between the ceiling and the signal landed, and its own status decides");
    assert_eq!(a_stop, None, "a step that ended by itself is no ceiling stop");
}

/// What:
///  A program below the step whose descriptor the kernel refused is recorded in the stop,
///  so the person is told it may still be running.
/// Why:
///   Round six of the review:
///  a ceiling that ends the `git commit` and silently fails to end the signing program
///  leaves that program holding the person's terminal and the agent's socket,
///  and a stop that says every program was ended leaves them to find it with `ps`.
///  The refusal stands where a descriptor limit makes it real,
///  and the helper really is left running,
///  which is what the record now says.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // const pid = sh('sleep 600 & echo $!; wait'); refuseTheNextDescriptor(); const [outcome, stop] = await wait(adopt(pid), undefined, 400); expect(stop.belowUnended).toBe(1)
/// ```
#[cfg(target_os = "linux")]
#[test]
fn a_program_the_kernel_refused_to_signal_is_recorded_in_the_stop() {
    let mut command: Command = Command::new("sh");
    command.arg("-c").arg("sleep 600 & echo $!; wait").stdout(Stdio::piped()).stdin(Stdio::null());
    let mut step: Child = crate::test_support::spawn_owned(&mut command).expect("a step with a helper");
    let helper: i32 = number_below(&mut step);
    // The step's own descriptor is opened first, so the refusal falls on the helper's.
    let owned: OwnedChild = adopt(step);
    crate::member_signalled_through_its_pidfd::refuse_the_next_descriptors(1);
    let (outcome, a_stop): (ChildOutcome, Option<CeilingStop>) = crate::lazy_runtime::run_on_runtime(wait_until_it_exits_the_stop_or_the_ceiling(owned, None, Ceilings::of(Duration::from_millis(400)), "a step", None))
        .expect("the runtime")
        .expect("the wait answers");
    assert_eq!(outcome, ChildOutcome::Signaled(libc::SIGKILL), "the step itself is ended through the child this wait owns");
    let stop: CeilingStop = a_stop.expect("the ceiling came");
    assert!(stop.below_listed, "the programs below the step were listed");
    assert_eq!(stop.below_unended, 1, "the program whose signal was refused is recorded, so the person is told it may still run");
    assert!(!went_within(helper, STILL_RUNNING_BOUND), "the refusal this control injects is real: the helper was left running");
}

/// The number of the program a step printed below itself.
#[cfg(target_os = "linux")]
fn number_below(step: &mut Child) -> i32 {
    use std::io::{BufRead, BufReader};
    let output = step.stdout.take().expect("the helper's number");
    let mut line: String = String::new();
    BufReader::new(output).read_line(&mut line).expect("a number");
    return line.trim().parse::<i32>().expect("a process number");
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
    let (outcome, a_stop): (ChildOutcome, Option<CeilingStop>) = crate::lazy_runtime::run_on_runtime(wait_until_it_exits_the_stop_or_the_ceiling(adopt(step), None, Ceilings::of(ceiling), "a step", None))
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
                let answered = wait_until_it_exits_the_stop_or_the_ceiling(adopt(step), stop, Ceilings::of(Duration::from_secs(60)), "a step", None).await;
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

/// What:
///  A step the kernel holds in a disk wait past its ceiling is waited for, and lands when it ends.
/// Why:
///   Ending a step that waits for the disk fails a commit that would have landed:
///  on 2026-11-11 two commits failed that way,
///  their steps held for 37,153 and 47,978 ms under a 16,332 ms ceiling.
///  The step here is a real child that ends by itself;
///  what a control states is the kernel's answer about that child's state,
///  which no run on this host can make real (`commit/scratch_step_look_below_control_window.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // const pid = sh('sleep 0.6'); theKernelHolds(pid); const [outcome, stop] = await wait(adopt(pid), undefined, { usually: 100, whileTheKernelHoldsIt: 30_000 }); expect(outcome).toEqual({ exited: 0 }); expect(stop).toBeUndefined()
/// ```
#[cfg(target_os = "linux")]
#[test]
fn a_step_the_kernel_holds_past_its_ceiling_lands_when_it_ends() {
    let _look_window = crate::scratch_step_look_below::the_controls_window::take_the_window();
    let mut command: Command = Command::new("sh");
    command.arg("-c").arg("sleep 0.6; exit 0").stdin(Stdio::null());
    let step: Child = crate::test_support::spawn_owned(&mut command).expect("a step that ends by itself");
    let number: i32 = i32::try_from(step.id()).expect("a process number");
    crate::scratch_step_look_below::the_controls_window::the_kernel_holds(number, "git");
    let ceilings: Ceilings = Ceilings { usually: Duration::from_millis(100), while_the_kernel_holds_it: Duration::from_secs(30) };
    let began: Instant = Instant::now();
    let (outcome, a_stop): (ChildOutcome, Option<CeilingStop>) =
        crate::lazy_runtime::run_on_runtime(wait_until_it_exits_the_stop_or_the_ceiling(adopt(step), None, ceilings, "a step", None))
            .expect("the runtime")
            .expect("the wait answers");
    let lasted: Duration = began.elapsed();
    assert_eq!(outcome, ChildOutcome::Exited(0), "a step the kernel held past its ceiling landed, and its own status decides");
    assert_eq!(a_stop, None, "the ceiling stood aside, so no stop is recorded");
    assert!(lasted >= Duration::from_millis(600), "the step was ended at its ceiling and never waited for: {lasted:?}");
    assert!(went_within(number, STILL_RUNNING_BOUND), "the step was killed instead of waited for: {lasted:?}");
}

/// What:
///  A step the kernel never releases is ended at the bound it is waited for,
///  and the stop says how long it lasted.
/// Why:
///   A step that is really stuck is still ended:
///  the bound is six times the longest step the kernel ever held here,
///  so an ordinary slow disk never reaches it,
///  and a step it does reach is one nothing will release.
///  The step here is a real child that does not end by itself,
///  and the bound a control gives is a second.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // const pid = sh('sleep 600'); theKernelHolds(pid); const [outcome, stop] = await wait(adopt(pid), undefined, { usually: 100, whileTheKernelHoldsIt: 1_000 }); expect(outcome).toEqual({ signal: 9 }); expect(stop.waited >= 1_000).toBe(true)
/// ```
#[cfg(target_os = "linux")]
#[test]
fn a_step_the_kernel_never_releases_is_ended_at_the_bound_it_is_waited_for() {
    let _look_window = crate::scratch_step_look_below::the_controls_window::take_the_window();
    let step: Child = a_step_that_never_answers();
    let number: i32 = i32::try_from(step.id()).expect("a process number");
    crate::scratch_step_look_below::the_controls_window::the_kernel_holds(number, "git");
    let bound: Duration = Duration::from_secs(1);
    let ceilings: Ceilings = Ceilings { usually: Duration::from_millis(100), while_the_kernel_holds_it: bound };
    let began: Instant = Instant::now();
    let (outcome, a_stop): (ChildOutcome, Option<CeilingStop>) =
        crate::lazy_runtime::run_on_runtime(wait_until_it_exits_the_stop_or_the_ceiling(adopt(step), None, ceilings, "a step", None))
            .expect("the runtime")
            .expect("the wait answers");
    let lasted: Duration = began.elapsed();
    let stop: CeilingStop = a_stop.expect("the bound came, so the step was ended");
    assert_eq!(outcome, ChildOutcome::Signaled(libc::SIGKILL), "a step the kernel never released was ended here");
    assert!(stop.waited >= bound, "the step was ended at its ceiling instead of the bound it is waited for: {:?}", stop.waited);
    assert!(lasted >= bound && lasted < Duration::from_secs(20), "the bound did not end the step: {lasted:?}");
    assert!(went_within(number, STILL_RUNNING_BOUND), "the step was killed and not reaped");
}

/// What:
///  A step the kernel holds that ends between the bound and the signal is reported by its own status,
///  with no ceiling stop beside it.
/// Why:
///   A step that finishes in that interval landed,
///  and reading its reaped status as a stop turns a preparation that succeeded into a failure with nothing committed.
///  A control holds the step at that instruction,
///  so the order is the control's and not a clock's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // const pid = sh('sleep 0.6'); theKernelHolds(pid); endTheStepBeforeTheSignal(); const [outcome, stop] = await wait(adopt(pid), undefined, { usually: 50, whileTheKernelHoldsIt: 300 }); expect(outcome).toEqual({ exited: 0 }); expect(stop).toBeUndefined()
/// ```
#[cfg(target_os = "linux")]
#[test]
fn a_step_the_kernel_holds_that_ends_after_the_bound_is_reported_by_its_own_status() {
    let _look_window = crate::scratch_step_look_below::the_controls_window::take_the_window();
    let (outcome, a_stop): (ChildOutcome, Option<CeilingStop>) = crate::lazy_runtime::run_on_runtime(async {
        let mut command: Command = Command::new("sh");
        command.arg("-c").arg("sleep 0.6; exit 0").stdin(Stdio::null());
        let step: Child = crate::test_support::spawn_owned(&mut command).expect("a step that ends by itself");
        let number: i32 = i32::try_from(step.id()).expect("a process number");
        crate::scratch_step_look_below::the_controls_window::the_kernel_holds(number, "git");
        a_control_ends_the_step_here();
        let ceilings: Ceilings = Ceilings { usually: Duration::from_millis(50), while_the_kernel_holds_it: Duration::from_millis(300) };
        return wait_until_it_exits_the_stop_or_the_ceiling(adopt(step), None, ceilings, "a step", None).await.expect("the wait answers");
    })
    .expect("the runtime");
    assert_eq!(outcome, ChildOutcome::Exited(0), "a step that finished between the bound and the signal landed, and its own status decides");
    assert_eq!(a_stop, None, "a step that ended by itself is no ceiling stop");
}
