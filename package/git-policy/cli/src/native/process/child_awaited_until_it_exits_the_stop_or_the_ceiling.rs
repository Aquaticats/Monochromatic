//! What:
//!  Waits for a child this path owns until that child ends,
//! a stop of the service ends it,
//! or the ceiling of the step comes;
//! when the ceiling comes,
//! the child and everything it started are ended,
//! the child is reaped here,
//! and the stop is answered as a record beside the child's end.
//! Why:
//!  The wait that owns a child is the only place where its number cannot be reused:
//! the kernel keeps an unreaped child's number,
//! and this path is the only one that reaps it,
//! so the signal and the reap are one path in sequence and no reuse falls between them.
//!  A notice thread ends the step through a descriptor taken at its start when it can,
//! but that thread may never have started (the system refused it),
//! and a start the kernel refused a descriptor for leaves it naming nothing for good.
//! In both cases the command waited with no bound over the step at all,
//! which is finding 1 of the fifth review of this step and the rule the owner set on 2026-11-03,
//! "nothing should ever wait forever or indefinitely".
//!  The ceiling here is counted from the step's own start,
//! which this path is the moment of,
//! so it cannot be lost to a look that ran late.
//!  A step in an uninterruptible call is not collected here past the grace:
//! a task of the runtime collects it,
//! as the stop of a push does with the child it leaves.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [outcome, ceilingStop] = await waitUntilItEndsTheStopOrTheCeiling(owned, stop, ceiling, 'a Git step of a commit');
//! ```

/// The child,
/// held until it is reaped,
/// and the token that takes it back for a task to collect.
use crate::child_awaited_through_a_pidfd::{OwnedChild, adopt};
/// The end of a child as the wrapper's exit reads it.
use crate::forwarding::{ChildOutcome, outcome_of};
/// The arm a stop of the service is told through,
/// and the error that says the stop ended the step.
use crate::stop_of_the_service_noticed_by_its_children::StopArm;
/// The processes the step started,
/// read while its number still names it.
use crate::process_tree::{TreeMember, descendants_of_read};
/// Ending what the step started,
/// each through a descriptor opened while its number still named it,
/// and what that came to.
use crate::member_signalled_through_its_pidfd::{Delivered, kill_members, unended_in};
/// The grace a stopped child gets before a task of the runtime collects it.
use crate::bounded_push::STOP_GRACE;
/// The stop of a push,
/// which a stop of the service asks for.
use crate::bounded_push::stop_push;
/// The locks a stopped push leaves,
/// none of which this wait claims:
/// the step runs on a private copy of the index.
use crate::stopped_git_locks::LockLedger;
/// Waiting on the runtime's timer.
use tokio::time::sleep;
/// Time spans and the monotonic clock.
use std::time::{Duration, Instant};

/// What:
///  A ceiling that came,
///  as the record states it.
/// Why:
///   The step's stop is recorded once,
///  whoever ended it,
///  so the wait answers the same shape the notice's thread does:
///  how long the step lasted,
///  and whether the programs it started could be listed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CeilingStop = { waited: number; belowListed: boolean; belowUnended: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CeilingStop {
    /// How long the step lasted before it was ended.
    pub waited: Duration,
    /// Whether the programs below the step were listed,
    /// so one that was not is not reported as gone.
    pub below_listed: bool,
    /// How many programs below the step were not ended:
    /// one the kernel refused a descriptor for,
    /// or one it refused the signal to.
    /// Such a program may still be running,
    /// which the record and the stop's words say.
    pub below_unended: u32,
}

// What:
//  A control that ends the step in the interval between the ceiling and the signal.
// Why:
//   That interval is a few instructions wide on a real host,
//  so a control that races a real exit proves nothing;
//  here the wait itself holds the step until the kernel says it ended,
//  and the status waits for the reap below.
#[cfg(all(test, target_os = "linux"))]
thread_local! {
    /// Whether the next ceiling holds its step until that step has ended by itself.
    static A_STEP_ENDS_BEFORE_THE_SIGNAL: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// The longest a control's step is waited for once the ceiling has come.
///  Longer than any step a control starts takes to end,
/// so a starved runtime delays the control rather than failing it,
/// and a control whose step never ended fails rather than passing on a window it never made.
#[cfg(all(test, target_os = "linux"))]
const A_CONTROLS_BOUND: Duration = Duration::from_secs(30);

/// Arm the next ceiling to hold its step until that step has ended by itself.
#[cfg(all(test, target_os = "linux"))]
pub(crate) fn a_control_ends_the_step_here() {
    A_STEP_ENDS_BEFORE_THE_SIGNAL.with(|armed| { armed.set(true); });
}

/// What:
///  Hold the step at the moment between the ceiling and the signal,
///  when a control asked for it,
///  until the kernel says the step ended.
/// Why:
///   A step that finishes there was ended by its own exit,
///  and the reap below still collects that status,
///  which is what the caller must answer with.
///  The wait is bounded so a control's mistake ends the run with the wait's own answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await aControlHoldsTheStepUntilItEnded(owned);
/// ```
#[cfg(all(test, target_os = "linux"))]
async fn a_control_holds_the_step_until_it_ended(owned: &OwnedChild) {
    if !A_STEP_ENDS_BEFORE_THE_SIGNAL.with(|armed| { return armed.replace(false); }) {
        return;
    }
    let deadline: Instant = Instant::now() + A_CONTROLS_BOUND;
    while !owned.has_ended_without_reaping() {
        if Instant::now() >= deadline {
            return;
        }
        sleep(Duration::from_millis(1)).await;
    }
    return;
}

/// What:
///  Wait for `owned` until it ends,
///  a stop of the service ends it,
///  or `ceiling` comes,
///  and answer how it ended,
///  with the stop when the ceiling came first.
/// Why:
///  This is the one wait that can name the step it owns and send its signal without a number a reuse can reach,
///  so a ceiling that no notice thread enforces is still a ceiling.
///  The step is signalled and reaped here,
///  never by a number read at the ceiling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const began = now(); const ceilingCame = await race([owned.exited, stop.arm, sleep(ceiling)]); if (!ceilingCame) return [outcome, undefined]; killOwned(owned); return [{ signal: 9 }, { waited: now() - began, belowListed }];
/// ```
///
/// # Errors
///
/// The error of a wait the system failed,
/// or `EndedByStop` when the service's stop ended the child.
pub async fn wait_until_it_exits_the_stop_or_the_ceiling(
    mut owned: OwnedChild,
    mut stop: Option<StopArm>,
    ceiling: Duration,
    what: &str,
) -> std::io::Result<(ChildOutcome, Option<CeilingStop>)> {
    let began: Instant = Instant::now();
    // The child's own end wins over the ceiling when both are ready:
    // a step that finished by itself landed, and stopped after its ceiling is a different record.
    let ended: Option<std::process::ExitStatus> = match stop.as_mut() {
        Some(arm) => tokio::select! {
            biased;
            () = arm.stop_told() => {
                stop_push(owned, STOP_GRACE, LockLedger::none()).await;
                return Err(crate::child_watch::ended_by_the_stop(what));
            }
            finished = owned.wait() => Some(finished?),
            () = sleep(ceiling) => None,
        },
        None => tokio::select! {
            biased;
            finished = owned.wait() => Some(finished?),
            () = sleep(ceiling) => None,
        },
    };
    if let Some(status) = ended {
        return Ok((outcome_of(status), None));
    }
    let waited: Duration = began.elapsed();
    // What the step started is read now, while the step is alive:
    // a number cannot be reused while its process holds it, and everything below the step is found by that number only
    // then.
    let walked: Option<Vec<TreeMember>> = i32::try_from(owned.id()).ok().and_then(descendants_of_read);
    // What: The interval between the ceiling and the signal.
    // Why:  A step that finishes there was ended by its own exit, and the status the reap collects says so;
    //       the control holds the step here so that order is its own and not a clock's.
    #[cfg(all(test, target_os = "linux"))]
    a_control_holds_the_step_until_it_ended(&owned).await;
    // The step itself is killed through the child this path still owns and has not reaped,
    // so no process that took its number can be reached.
    let _killed: std::io::Result<()> = owned.kill();
    let ended_below: Vec<Delivered> = kill_members(walked.as_deref().unwrap_or_default());
    // What: The programs below the step that this wait could not end.
    // Why:  A signal the kernel refused leaves the program running, and the stop must say so rather than report a walk
    //       that ended everything it listed.
    let below_unended: u32 = unended_in(ended_below.as_slice());
    // A killed step is reaped here at once unless the kernel holds it in an uninterruptible call;
    // past the grace a task of the runtime collects it, so the kernel's release of it never holds the command.
    let reaped: Option<std::process::ExitStatus> = match tokio::time::timeout(STOP_GRACE, owned.wait()).await {
        Ok(Ok(collected)) => Some(collected),
        // What: A status the wait could not collect.
        // Why:  The step was signalled here and the wait knows nothing of what ended it, which is what the answer says;
        //       a commit is not failed over the reading of a status it already ended the step for
        //       (owner: "Commits should never fail.").
        Ok(Err(_not_collected)) => None,
        Err(_still_held) => {
            hand_to_a_task_that_collects_it(owned);
            None
        }
    };
    // A status the reap collected decides what ended the step: one that is not the signal this wait sent proves the
    //  step finished by itself in the interval before the signal, and its exit is what the caller must answer.
    if let Some(status) = reaped
        && it_ended_by_itself(status)
    {
        return Ok((outcome_of(status), None));
    }
    return Ok((ChildOutcome::Signaled(libc::SIGKILL), Some(CeilingStop { waited, below_listed: walked.is_some(), below_unended })));
}

/// What:
///  Whether a reaped status proves the step ended by itself,
///  rather than by the signal this wait sent.
/// Why:
///   `SIGKILL` is the only signal sent here,
///  so a status that is not it is the step's own end,
///  and a status with a code is an exit that the kill could not change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const itEndedByItself = (status) => status.code !== null || status.signal !== 'SIGKILL';
/// ```
fn it_ended_by_itself(status: std::process::ExitStatus) -> bool {
    // `if let Some(code) = ...` runs only when the child exited normally, and a kill cannot turn an exit into a code.
    if status.code().is_some() {
        return true;
    }
    #[cfg(unix)]
    {
        // The trait adds `.signal()` to exit statuses on Unix.
        use std::os::unix::process::ExitStatusExt;
        return status.signal() != Some(libc::SIGKILL);
    }
    #[cfg(not(unix))]
    return false;
}

/// What:
///  Give an unreaped child to a task of the runtime that waits for it.
/// Why:
///   A killed process in an uninterruptible call is released by the kernel when it is,
///  and a wait for that is the one wait nothing can bound.
///  The child is handed over as the standard library's value and adopted inside the task,
///  so a runtime with no worker to run it leaves the child as it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// void runtime.spawn(async () => { await child.exited; });
/// ```
fn hand_to_a_task_that_collects_it(owned: OwnedChild) {
    let Some(child) = owned.into_std_when_unreaped() else {
        return;
    };
    // Nothing reads the status: the step was stopped, and the words the person reads say so.
    let _task = crate::lazy_runtime::spawn_on_runtime(async move {
        let mut still_a_child: OwnedChild = adopt(child);
        let _status = still_a_child.wait().await;
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "child_awaited_until_it_exits_the_stop_or_the_ceiling_tests.rs"]
mod tests;
