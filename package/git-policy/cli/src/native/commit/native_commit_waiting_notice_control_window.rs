//! What:
//!  A control's window on the notice:
//  the ceiling it plants,
//  the looks that found the step,
//  and the looks that failed once the ceiling had come,
//  and the step a control states the stop did not end.
// Why:
//   A control that sleeps a fixed window races a notice thread on a loaded host:
//   when its own setup outruns the ceiling,
//   the ceiling comes while the window it means to test does not exist yet,
//   and the control measures an ordinary stop.
//  A ceiling planted here comes when the control says so,
//  and the counts are states it waits for,
//  so a loaded host makes the control slower and never wrong.
//  It is one window for the process, so a control takes it before it plants anything.
// Limits:
//  Test-only. A release build reads no atomic and holds no lock here.

use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU32, Ordering};
use std::time::{Duration, Instant};

/// The ceiling a control planted,
/// in milliseconds,
/// or -1 for none.
static CEILING: AtomicI64 = AtomicI64::new(-1);
/// How many looks found the step,
/// which arms the ceiling from the first of them.
static LOOKS_THAT_FOUND_THE_STEP: AtomicU32 = AtomicU32::new(0);
/// Whether a control stated that the look after the stop still finds the step.
static STOP_DID_NOT_END_THE_STEP: AtomicBool = AtomicBool::new(false);
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
    LOOKS_THAT_FOUND_THE_STEP.store(0, Ordering::SeqCst);
    STOP_DID_NOT_END_THE_STEP.store(false, Ordering::SeqCst);
    return taken;
}

/// What:
///  State that the look after the ceiling's stop still finds the step,
///  held in uninterruptible sleep.
/// Why:
///   No control on this host can put a process into that state:
///  it takes a stalled disk,
///  a dead network file system or a stopped file-system server,
///  and the gate's runs mount nothing (`BOX`).
///  The stop itself is real in every control that plants this;
///  only the kernel's answer about the step's state is stated.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holdTheStepPastTheStop(); // a stand-in for a process in state `D`
/// ```
pub(crate) fn plant_a_step_the_stop_did_not_end() {
    STOP_DID_NOT_END_THE_STEP.store(true, Ordering::SeqCst);
}

/// Whether a control stated that the stop did not end the step.
pub(crate) fn a_control_says_the_stop_did_not_end_the_step() -> bool {
    return STOP_DID_NOT_END_THE_STEP.load(Ordering::SeqCst);
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
