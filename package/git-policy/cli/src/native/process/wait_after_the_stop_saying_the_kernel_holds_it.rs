//! What:
//!  The wait for a step whose ceiling's `SIGKILL` did not end it:
//!  the step is reaped here,
//!  and while the kernel has not released it,
//!  a line says so,
//!  naming the process the kernel is holding.
//! Why:
//!   A process inside an uninterruptible call makes no transition until that call returns,
//!  so its `SIGKILL` stays pending
//!  (`doc/troubleshooting/sigkill-does-not-end-a-process-in-uninterruptible-sleep.md`)
//!  and the commit waits for the kernel rather than for its own ceiling.
//!  The owner chose that wait and asked for it to be said
//!  (2026-11-10,
//!  "Keep waiting and say so"),
//!  so the command keeps saying it for as long as the wait lasts:
//!  a person who sees the line once and then a silent terminal reads the silence as a hang.
//!  The words come from the step the commit runs (`commit/scratch_step_held_by_the_kernel.rs`),
//!  and this wait says them because this wait is the one that lasts:
//!  the notice thread that also ends a step may never have been started,
//!  and a thread the system refused cannot say anything at all.
//!  The line is handed to the writer thread of the command's notice
//!  (`HeldStepWords::lines`),
//!  never written here:
//!  a reader that stopped reading holds that thread,
//!  and it must never hold the wait.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const status = await untilReaped(owned, waitedAtTheStop, words); // says the held process on each look
//! ```

/// The child this wait owns,
/// and the descriptor that names it for good.
use crate::child_awaited_through_a_pidfd::OwnedChild;
/// The end of the child,
/// and its status.
use std::process::ExitStatus;
/// Where the line goes:
/// the writer thread of the command's notice.
use std::sync::mpsc::SyncSender;
/// Time spans.
use std::time::Duration;

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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const afterTheStop = 250; // ms
/// ```
pub(crate) const AFTER_THE_STOP: Duration = Duration::from_millis(250);

/// What:
///  Where a wait that must say why it lasts hands its line,
///  and how often it says the line again while the wait lasts.
/// Why:
///   The words and the sink belong to the command that knows what its step is,
///  and the timing belongs to the wait that lasts,
///  so the wait is given them and asks nothing else.
///  The interval is the notice's own,
///  so the person reads the same rhythm they read before the ceiling came.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HeldStepWords = { lines: SyncSender<string>; againEvery: number };
/// ```
pub struct HeldStepWords<'a> {
    /// The queue the command's writer thread takes its lines from.
    pub lines: &'a SyncSender<String>,
    /// How long the wait goes between two lines while the kernel holds the step.
    pub again_every: Duration,
}

/// What:
///  Wait for `owned` to be reaped,
///  saying while it lasts that the kernel has not released the process it stopped.
/// Why:
///  This is the wait the ceiling leaves behind when its `SIGKILL` did not end the step,
///  and it is the only wait that can say so:
///  the stop was sent here,
///  the step is the child this value owns,
///  and this value keeps waiting whatever the notice's own thread did.
///  A line is dropped rather than waited for when the writer cannot take it,
///  because a reader that stopped reading must never hold this wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const status = await untilReaped(owned, waited, { lines, againEvery });
/// ```
///
/// # Errors
///
/// The error of a wait the system failed.
pub(crate) async fn until_reaped(owned: &mut OwnedChild, waited: Duration, words: Option<HeldStepWords<'_>>) -> std::io::Result<ExitStatus> {
    let Some(words) = words else {
        return owned.wait().await;
    };
    let Some(step) = i32::try_from(owned.id()).ok() else {
        return owned.wait().await;
    };
    // The step was found held by the look after the stop, so the line that follows is made from a look taken after the pause.
    let mut seen_held: bool = false;
    let mut pause: Duration = Duration::ZERO;
    loop {
        tokio::select! {
            biased;
            status = reaped(owned) => return status,
            () = tokio::time::sleep(pause) => {}
        }
        match the_look(step) {
            TheLook::Held if seen_held => {
                let line: String = crate::scratch_step_held_by_the_kernel::held_by_the_kernel_line(crate::scratch_step_held_by_the_kernel::HeldByTheKernel { step, waited });
                let _taken = words.lines.try_send(line);
                pause = words.again_every;
            }
            TheLook::Held => {
                seen_held = true;
                pause = AFTER_THE_STOP;
            }
            // The step acted on the signal, or the look could not be made: nothing is known, and the step is awaited.
            TheLook::NotHeld | TheLook::CouldNotLook => return owned.wait().await,
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
    a_control_says_the_reap_has_not_returned::until_the_control_lets_the_reap_go().await;
    return owned.wait().await;
}

// What:
//  A control that holds the reap of the step past the stop.
// Why:
//   The wait this module exists for lasts as long as the kernel has not released the step,
//   and every step a control can start here acts on `SIGKILL` at once,
//  so its reap returns before a look after the pause is made.
//  A control states that the reap has not returned,
//  which is the one fact a real uninterruptible call would give it.
// Limits:
//  Test-only, and one window for the process, so a control takes it before it holds anything.
#[cfg(all(test, target_os = "linux"))]
mod a_control_says_the_reap_has_not_returned {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Mutex, MutexGuard};
    use std::time::{Duration, Instant};

    /// Whether a control states that the step's end has not been collected.
    static HELD: AtomicBool = AtomicBool::new(false);
    /// Held by the control that holds a reap.
    static WINDOW: Mutex<()> = Mutex::new(());
    /// The longest a control's hold lasts, so a control's mistake ends the run with this wait's own answer.
    const A_CONTROLS_BOUND: Duration = Duration::from_secs(30);
    /// How often a held reap looks at the control's word.
    const LOOK_PAUSE: Duration = Duration::from_millis(5);

    /// Take the window for the control that is about to use it,
    /// with no hold planted.
    pub(crate) fn take_the_window() -> MutexGuard<'static, ()> {
        let taken: MutexGuard<'static, ()> = WINDOW.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        HELD.store(false, Ordering::SeqCst);
        return taken;
    }

    /// State that the step's end has not been collected.
    pub(crate) fn hold_the_reap() {
        HELD.store(true, Ordering::SeqCst);
    }

    /// Let the reap return.
    pub(crate) fn let_the_reap_go() {
        HELD.store(false, Ordering::SeqCst);
    }

    /// Wait,
    /// no longer than `A_CONTROLS_BOUND`,
    /// for the control to let the reap return.
    pub(crate) async fn until_the_control_lets_the_reap_go() {
        let deadline: Instant = Instant::now() + A_CONTROLS_BOUND;
        while HELD.load(Ordering::SeqCst) && Instant::now() < deadline {
            tokio::time::sleep(LOOK_PAUSE).await;
        }
    }
}

/// What:
///  What one look at the step and the programs below it answered about the kernel holding them.
/// Why:
///   A look that found no process in uninterruptible sleep,
///  and a look that could not be made at all,
///  are two different facts:
///  the first is the kernel about to release the step,
///  and the second is nothing known about it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheLook = 'held' | 'not-held' | 'could-not-look';
/// ```
enum TheLook {
    /// A process below the step is inside a call a signal cannot end.
    Held,
    /// No process below the step is in that state,
    /// so the step is about to act on its signal.
    NotHeld,
    /// `/proc` could not be read,
    /// so nothing about the step is known.
    CouldNotLook,
}

/// What:
///  One look at the step and the programs below it,
///  and whether the kernel is holding one of them.
/// Why:
///   The step's own state and its programs' are read together,
///  by the look the notice also makes,
///  so this wait asks the kernel the same question the notice asks
///  and names no process it did not see.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const look = lookBelow(step); if (!look) return 'could-not-look'; return look.some(blocked) ? 'held' : 'not-held';
/// ```
fn the_look(step: i32) -> TheLook {
    return match crate::scratch_step_look_below::look_below(step) {
        Some(below) if below.iter().any(crate::scratch_step_look_below::process_is_blocked) => TheLook::Held,
        Some(_no_process_is_held) => TheLook::NotHeld,
        None => TheLook::CouldNotLook,
    };
}

/// The wait and its controls stay out of the release executable.
#[cfg(test)]
#[path = "wait_after_the_stop_saying_the_kernel_holds_it_tests.rs"]
mod tests;
