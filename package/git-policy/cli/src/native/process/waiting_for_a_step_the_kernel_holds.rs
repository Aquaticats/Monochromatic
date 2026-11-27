//! What:
//!  The wait for an owned step the kernel holds in a call a signal cannot end:
//!  the bound it is given,
//!  counted from the step's own start,
//!  the one line that says why the wait lasts,
//!  and the three ways it ends.
//! Why:
//!  A step inside an uninterruptible call is slow and not stuck,
//!  so ending it at the ceiling it reached fails a commit that would have landed.
//!  The wait stays and says so once,
//!  because a person who read the same sentence three times in one failed commit learned nothing the first two times;
//!  and a step that ends by itself is noticed on the way to the next look,
//!  so a commit whose step landed is never waited for to that bound.
//!
//! Files beside this one,
//!  and why the wait is not one file:
//!  `child_awaited_until_it_exits_the_stop_or_the_ceiling.rs` owns the child,
//!  ends it at its ceiling and reaps it there,
//!  and hands this file the bound the step is waited for and the words it says meanwhile.
//!  `rust/max-lines` splits the wait by role,
//!  and what a step the kernel holds is told is the role a reader can hold alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ended = await untilItEndsOrTheHoldBound(owned, stop, stalled);
//! // { itEnded: status } | { theBound: true } | { theStopOfTheService: true }
//! ```

/// The child this wait keeps holding until one of its three ends comes.
use crate::child_awaited_through_a_pidfd::OwnedChild;
/// The one decision this wait shares with the notice's thread,
///  whose stand-aside is what gives the step this bound.
use crate::scratch_step_stall_decided_once::WhyTheStepWasWaitedFor;
/// The arm a stop of the service is told through.
use crate::stop_of_the_service_noticed_by_its_children::StopArm;
/// The queue the command's lines go through,
///  which the one line this wait says is handed to.
use std::sync::mpsc::SyncSender;
/// Time spans and the monotonic clock.
use std::time::{
    Duration,
    Instant,
};
/// Waiting on the runtime's timer.
use tokio::time::sleep;

/// What:
///  One wait for a step the kernel holds in a disk wait is given.
/// Why:
///   The bound is counted from the step's own start,
///  the line names the process the kernel is holding,
///  and the wait that lasts is the one that says it,
///  because a notice thread the system refused says nothing at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AStalledStep = { began: number; waited: number; until: number; why: WhyTheStepWasWaitedFor; lines?: Queue };
/// ```
pub(crate) struct AStalledStep<'a> {
    /// When the step started.
    pub(crate) began: Instant,
    /// How long it had lasted when its ceiling came.
    pub(crate) waited: Duration,
    /// The longest it may last while the ceiling stands aside.
    pub(crate) until: Duration,
    /// Why the ceiling stood aside:
    /// the process the kernel is holding,
    /// or nothing when nothing could be learned about the step.
    pub(crate) why: WhyTheStepWasWaitedFor,
    /// Where the command's lines go,
    /// and nothing when it has no writer to hand one to.
    pub(crate) lines: Option<&'a SyncSender<String>>,
}

/// What:
///  How a wait for a step the kernel holds ended.
/// Why:
///   The three answers end the command differently:
///  a step that ended by itself lands,
///  a bound that came stops the step,
///  and a stop of the service is reported as itself and never as a ceiling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HowTheStallEnded = { itEnded: number } | { theBound: true } | { theStopOfTheService: true };
/// ```
pub(crate) enum HowTheStallEnded {
    /// The step ended by itself while the kernel held it.
    ItEnded(std::process::ExitStatus),
    /// The bound came and the kernel still held it.
    TheBound,
    /// The service's stop was told while the step waited.
    TheStopOfTheService,
}

/// What:
///  Wait for a step the kernel holds until it ends,
///  the bound comes,
///  or the service's stop ends it,
///  saying once why the wait lasts.
/// Why:
///  A step inside an uninterruptible call is slow and not stuck,
///  so ending it at the ceiling it reached fails a commit that would have landed.
///  The wait stays and says so once:
///  a person who read the same sentence three times in one failed commit learned nothing the first two times.
///  A line is dropped rather than waited for when the writer cannot take it,
///  because a reader that stopped reading must never hold this wait,
///  and the line is handed over again at the next look while nothing was taken.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ended = await untilItEndsOrTheHoldBound(owned, stop, stalled); if (ended.itEnded) return [ended.itEnded, undefined];
/// ```
pub(crate) async fn until_it_ends_or_the_hold_bound(
    owned: &mut OwnedChild,
    stop: &mut Option<StopArm>,
    stalled: AStalledStep<'_>,
) -> std::io::Result<HowTheStallEnded> {
    let AStalledStep {
        began,
        waited,
        until,
        why,
        lines,
    } = stalled;
    let mut said: bool = false;
    let mut pause: Duration = Duration::ZERO;
    loop {
        if began.elapsed() >= until {
            return Ok(HowTheStallEnded::TheBound);
        }
        // What: The service's stop is waited for beside the step.
        // Why:  A shutdown must not wait the bound this wait keeps;
        //       a step it stops is reported as the stop and never as a ceiling.
        let told = async {
            return match stop.as_mut() {
                Some(arm) => arm.stop_told().await,
                None => std::future::pending().await,
            };
        };
        tokio::select! {
            biased;
            () = told => return Ok(HowTheStallEnded::TheStopOfTheService),
            status = owned.wait() => return Ok(HowTheStallEnded::ItEnded(status?)),
            () = sleep(pause) => {}
        }
        if !said {
            // What: The line that says why the ceiling did not come, and the process the kernel is holding.
            // Why:  A step nothing could be learned about has no such process to name,
            //       and a line that named one would state a fact no look found (`scratch_step_held_by_the_kernel.rs`).
            let line: String = crate::scratch_step_held_by_the_kernel::a_stall_is_waited_for(
                crate::scratch_step_held_by_the_kernel::AStall {
                    step: i32::try_from(owned.id()).unwrap_or(0),
                    waited,
                    until,
                    why: why.clone(),
                },
            );
            said = match lines {
                Some(queue) => queue.try_send(line).is_ok(),
                None => true,
            };
        }
        pause = crate::wait_after_the_stop_saying_the_kernel_holds_it::AFTER_THE_STOP;
    }
}
