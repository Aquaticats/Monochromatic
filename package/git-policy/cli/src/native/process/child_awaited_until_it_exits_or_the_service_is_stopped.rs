//! What:
//!  Awaits an owned child's exit,
//! or ends the child and says so when the service's stop is told first.
//! Why:
//!  The wait of a Git step that the service runs through a signal guard is the one wait that is not a watched step and not a push:
//! it owns its child,
//! so it is the waiter that ends it when the shutdown tells the stop (`stop_of_the_service_noticed_by_its_children.rs`).
//! A file of its own keeps `signal_relay.rs` under its line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const status = await Promise.race([child.exited, arm.stopTold().then(() => endChild(child))]);
//! ```

use crate::child_awaited_through_a_pidfd::OwnedChild;
use crate::stop_of_the_service_noticed_by_its_children::{StopArm, Unfinished, begin_unfinished_work};
use std::process::ExitStatus;

/// What:
///  Waits for `owned` to exit and returns its status;
/// when `stop` is told first,
/// ends the child with the grace every stop of a child has,
/// and returns the error that says the stop did it.
/// Why:
///  A child that outlived the service's shutdown would be left running by the process that ends right after it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return await Promise.race([child.exited, stopTold(arm).then(() => { await stopChild(child, STOP_GRACE); throw endedByTheStop(what); })]);
/// ```
pub async fn wait_until_exit_or_stop(mut owned: OwnedChild, mut stop: Option<StopArm>, what: &str) -> std::io::Result<ExitStatus> {
    let Some(arm) = stop.as_mut() else {
        return owned.wait().await;
    };
    return tokio::select! {
        biased;
        () = arm.stop_told() => {
            crate::bounded_push::stop_push(owned, crate::bounded_push::STOP_GRACE, crate::stopped_git_locks::LockLedger::none()).await;
            Err(crate::child_watch::ended_by_the_stop(what))
        }
        finished = owned.wait() => finished,
    };
}

/// What:
///  Counts the child about to start as work the shutdown waits for,
/// and gives the arm that ends it,
/// or refuses it when the service is already stopping.
/// Why:
///  Called before the child starts,
/// so a stop between the count and the start is seen by one of the two;
/// the guard is kept until the child is awaited.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [work, arm] = beginAnAwaitedChild('a Git step of a commit'); // throws EndedByStop when stopping
/// ```
pub fn begin_an_awaited_child(what: &str) -> std::io::Result<(Unfinished, Option<StopArm>)> {
    return match begin_unfinished_work(false, what) {
        Ok(mut counted) => {
            let arm: Option<StopArm> = counted.take_arm();
            Ok((counted, arm))
        }
        Err(refusal) => Err(crate::child_watch::refused_while_stopping(&refusal)),
    };
}
