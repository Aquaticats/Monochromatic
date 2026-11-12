//! What:
//!  The wait for a step whose ceiling's `SIGKILL` did not end it:
//!  the step is reaped here,
//!  and while the kernel has not released it,
//!  one line says so,
//!  naming the process the kernel is holding.
//! Why:
//!   A process inside an uninterruptible call makes no transition until that call returns,
//!  so its `SIGKILL` stays pending
//!  (`doc/troubleshooting/sigkill-does-not-end-a-process-in-uninterruptible-sleep.md`)
//!  and the commit waits for the kernel rather than for its own ceiling.
//!  The owner chose that wait (2026-11-10,
//!  "Keep waiting and say so") and the line is said once:
//!  a person who met the same sentence three times in one failed commit read nothing new the second time,
//!  and the command has already said how long the step lasted before it stopped it.
//!  The words come from the step the commit runs (`commit/scratch_step_held_by_the_kernel.rs`),
//!  and this wait says them because this wait is the one that lasts:
//!  the notice thread that also ends a step may never have been started,
//!  and a thread the system refused cannot say anything at all.
//!  The line is handed to the writer thread of the command's notice,
//!  never written here:
//!  a reader that stopped reading holds that thread,
//!  and it must never hold the wait.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const status = await untilReaped(owned, waitedAtTheStop, lines); // says the held process once
//! ```

/// The child this wait owns,
/// and the descriptor that names it for good.
use crate::child_awaited_through_a_pidfd::OwnedChild;
/// The end of the child,
/// and its status.
use std::process::ExitStatus;
/// Where the line goes:
/// the writer thread of the command's notice,
/// and what a queue that would not take it says.
use std::sync::mpsc::SyncSender;
/// Time spans.
use std::time::Duration;
/// What one look at the step says about the kernel holding it,
/// and the look itself.
use crate::scratch_step_look_below::{WhatTheLookSays, what_the_look_says};

/// What:
///  How long after the signal the first look at the step is made,
/// and how soon a look that could not be made,
/// or a line the writer did not take,
/// is tried again.
/// Why:
///   A step can be inside a write when the signal is sent and back in a processor a moment later,
///  and only a process still in uninterruptible sleep after this pause is one the kernel is holding:
///  a look made with the signal alone would name a process that acts on it in the next microsecond.
///  A look that failed decides nothing,
///  so it is made again rather than read as the kernel having let go,
///  and a line the writer could not take is said again as soon as it can be,
///  never a whole notice interval later.
///  A look that found no process in uninterruptible sleep does not end the wait either:
///  the step is still unreaped,
///  so the kernel may still be holding it,
///  and the wait ends when the step is reaped and never when a look says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const afterTheStop = 250; // ms
/// ```
pub(crate) const AFTER_THE_STOP: Duration = Duration::from_millis(250);

/// What:
///  Wait for `owned` to be reaped,
///  saying once that the kernel has not released the process it stopped.
/// Why:
///  This is the wait the ceiling leaves behind when its `SIGKILL` did not end the step,
///  and it is the only wait that can say so:
///  the stop was sent here,
///  the step is the child this value owns,
///  and this value keeps waiting whatever the notice's own thread did.
///  A line is dropped rather than waited for when the writer cannot take it,
///  because a reader that stopped reading must never hold this wait,
///  and a line it could not take is tried again at the next look.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const status = await untilReaped(owned, waited, lines);
/// ```
///
/// # Errors
///
/// The error of a wait the system failed.
pub(crate) async fn until_reaped(owned: &mut OwnedChild, waited: Duration, lines: Option<&SyncSender<String>>) -> std::io::Result<ExitStatus> {
    let Some(queue) = lines else {
        return owned.wait().await;
    };
    let Some(step) = i32::try_from(owned.id()).ok() else {
        return owned.wait().await;
    };
    // The step was found held by the look after the stop, so the line that follows is made from a look taken after the pause.
    let mut seen_held: bool = false;
    let mut said: bool = false;
    let mut pause: Duration = Duration::ZERO;
    loop {
        tokio::select! {
            biased;
            status = reaped(owned) => return status,
            () = tokio::time::sleep(pause) => {}
        }
        match the_look(step) {
            // What: The one line that says the kernel has not released the step.
            // Why:  A line the writer could not take is tried again at the next look rather than counted as said,
            //       and one it took is never said a second time: one stop is one sentence, not three.
            WhatTheLookSays::TheKernelHoldsIt(held) if seen_held && !said => {
                let line: String = crate::scratch_step_held_by_the_kernel::held_by_the_kernel_line(crate::scratch_step_held_by_the_kernel::HeldByTheKernel { step, waited, held });
                said = queue.try_send(line).is_ok();
                pause = AFTER_THE_STOP;
            }
            WhatTheLookSays::TheKernelHoldsIt(_held) => {
                seen_held = true;
                pause = AFTER_THE_STOP;
            }
            // What: Neither a look that found nothing nor a look that could not be made ends this wait.
            // Why:  The step is still unreaped, so the kernel may still be holding it, and a look that failed states
            //       nothing about it: reading one as the kernel having let go took the whole explanation away from a wait
            //       that then lasted (finding 2 of the 2026-11-10 review of the held-step notice).
            WhatTheLookSays::NotHeld | WhatTheLookSays::CouldNotLook => {
                seen_held = false;
                pause = AFTER_THE_STOP;
            }
        }
    }
}

/// The child's end,
/// and a control's hold on it while it states that the kernel has not released the step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const status = await reaped(owned);
/// ```
async fn reaped(owned: &mut OwnedChild) -> std::io::Result<ExitStatus> {
    #[cfg(all(test, target_os = "linux"))]
    the_controls_window::until_the_control_lets_the_reap_go().await;
    return owned.wait().await;
}

/// What:
///  One look at the step and the programs below it,
///  and whether the kernel is holding one of them.
/// Why:
///   The step's own state and its programs' are read together,
///  by the look the notice also makes,
///  so this wait asks the kernel the same question the notice asks
///  and names no process it did not see.
///  The three answers are the look's own (`scratch_step_look_below.rs`),
///  so a step whose state could not be read is never read here as one the kernel let go.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const look = whatTheLookSays(step); // { held } | 'not-held' | 'could-not-look'
/// ```
fn the_look(step: i32) -> WhatTheLookSays {
    return what_the_look_says(u32::try_from(step).unwrap_or(u32::MAX));
}

/// A control's hold on the reap of a step the kernel is said to hold.
#[cfg(all(test, target_os = "linux"))]
#[path = "wait_after_the_stop_saying_the_kernel_holds_it_control_window.rs"]
pub(crate) mod the_controls_window;

/// The wait and its controls stay out of the release executable.
#[cfg(test)]
#[path = "wait_after_the_stop_saying_the_kernel_holds_it_tests.rs"]
mod tests;
