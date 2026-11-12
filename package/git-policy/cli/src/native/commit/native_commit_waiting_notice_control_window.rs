//! What:
//!  A control's window on the notice:
//  the ceiling it plants,
//  the bound a step the kernel holds is waited for,
//  the looks that found the step,
//  and the looks that failed once the ceiling had come.
// Why:
//   A control that sleeps a fixed window races a notice thread on a loaded host:
//   when its own setup outruns the ceiling,
//   the ceiling comes while the window it means to test does not exist yet,
//   and the control measures an ordinary stop.
//  A ceiling planted here comes when the control says so,
//  and the counts are states it waits for,
//  so a loaded host makes the control slower and never wrong.
//  The bound a step the kernel holds is waited for is planted here for the same reason:
//  the shipped one is five minutes,
//  and the control that let it stand would wait them.
//  It is one window for the process, so a control takes it before it plants anything.
// Limits:
//  Test-only. A release build reads no atomic and holds no lock here.

use std::sync::atomic::{AtomicI64, AtomicU32, Ordering};
use std::time::{Duration, Instant};

/// The ceiling a control planted,
/// in milliseconds,
/// or -1 for none.
static CEILING: AtomicI64 = AtomicI64::new(-1);
/// The bound a control planted for a step the kernel holds,
/// in milliseconds,
/// or -1 for none.
static STALL_CEILING: AtomicI64 = AtomicI64::new(-1);
/// How many looks found the step,
/// which arms the ceiling from the first of them.
static LOOKS_THAT_FOUND_THE_STEP: AtomicU32 = AtomicU32::new(0);
/// Held by the control that plants a ceiling.
static WINDOW: std::sync::Mutex<()> = std::sync::Mutex::new(());
/// How often a wait looks at a count.
const LOOK_PAUSE: Duration = Duration::from_millis(5);

/// What:
///  Take the window for the control that is about to use it,
///  with no ceiling planted and every count at zero.
/// Why:
///   One notice process runs one control,
///  and a control that found another's counts would wait for the wrong look.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using window = takeTheWindow();
/// ```
pub(crate) fn take_the_window() -> std::sync::MutexGuard<'static, ()> {
    // A poisoned lock means another control panicked while holding it; the counts are still counts.
    let taken = WINDOW.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    CEILING.store(-1, Ordering::SeqCst);
    STALL_CEILING.store(-1, Ordering::SeqCst);
    LOOKS_THAT_FOUND_THE_STEP.store(0, Ordering::SeqCst);
    return taken;
}

/// What:
///  Plant the bound a step the kernel holds is waited for,
///  so a control waits a second of it rather than the five minutes a real hold lasts.
/// Why:
///   The shipped bound is far from an ordinary slow disk,
///  which is more than a control of the notice can pay,
///  and a step the kernel holds is exactly what a control states
///  (`commit/scratch_step_look_below_control_window.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plantTheStallCeiling(2_000);
/// ```
pub(crate) fn plant_the_stall_ceiling(bound: Duration) {
    STALL_CEILING.store(i64::try_from(bound.as_millis()).unwrap_or(i64::MAX), Ordering::SeqCst);
}

/// The bound a control planted for a step the kernel holds,
/// or nothing when it planted none.
pub(crate) fn the_stall_ceiling_the_control_planted() -> Option<Duration> {
    return match STALL_CEILING.load(Ordering::SeqCst) {
        -1 => None,
        planted => Some(Duration::from_millis(u64::try_from(planted).unwrap_or(u64::MAX))),
    };
}

/// Make the ceiling come at the next look.
pub(crate) fn plant_the_ceiling(ceiling: Duration) {
    CEILING.store(i64::try_from(ceiling.as_millis()).unwrap_or(i64::MAX), Ordering::SeqCst);
}

/// The ceiling a control planted,
/// or nothing when it planted none.
pub(crate) fn the_ceiling_the_control_planted() -> Option<Duration> {
    return match CEILING.load(Ordering::SeqCst) {
        -1 => None,
        planted => Some(Duration::from_millis(u64::try_from(planted).unwrap_or(u64::MAX))),
    };
}

/// Note that a look found the step.
pub(crate) fn noted_a_look_that_found_the_step() {
    LOOKS_THAT_FOUND_THE_STEP.fetch_add(1, Ordering::SeqCst);
}

/// Wait,
/// no longer than `bound`,
/// for a look to have found the step.
pub(crate) fn until_a_look_finds_the_step(bound: Duration) -> bool {
    return until(bound, || { return LOOKS_THAT_FOUND_THE_STEP.load(Ordering::SeqCst) > 0; });
}

/// Wait for `reached`,
/// no longer than `bound`.
fn until(bound: Duration, reached: impl Fn() -> bool) -> bool {
    let deadline: Instant = Instant::now() + bound;
    loop {
        if reached() {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(LOOK_PAUSE);
    }
}
