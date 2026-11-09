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
/// each through a descriptor opened while its number still named it.
use crate::member_signalled_through_its_pidfd::kill_members;
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
/// type CeilingStop = { waited: number; belowListed: boolean };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CeilingStop {
    /// How long the step lasted before it was ended.
    pub waited: Duration,
    /// Whether the programs below the step were listed,
    /// so one that was not is not reported as gone.
    pub below_listed: bool,
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
    // The step itself is killed through the child this path still owns and has not reaped,
    // so no process that took its number can be reached.
    let _killed: std::io::Result<()> = owned.kill();
    let _below_ended = kill_members(walked.as_deref().unwrap_or_default());
    // A killed step is reaped here at once unless the kernel holds it in an uninterruptible call;
    // past the grace a task of the runtime collects it, so the kernel's release of it never holds the command.
    match tokio::time::timeout(STOP_GRACE, owned.wait()).await {
        Ok(_reaped) => {}
        Err(_still_held) => hand_to_a_task_that_collects_it(owned),
    }
    return Ok((ChildOutcome::Signaled(libc::SIGKILL), Some(CeilingStop { waited, below_listed: walked.is_some() })));
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
