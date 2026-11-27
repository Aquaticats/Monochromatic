//! What:
//!  A control's window on the notice:
//  the ceiling it plants,
//  the bound a step the kernel holds is waited for,
//  the stretch it states between two looks of one step,
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
//  The stretch between two looks is planted here for the third reason:
//  a host under load woke the notice's thread late and turned its 250 ms look into a 319.7 ms one,
//  so a control's fixed window held one look fewer than its figures assumed (issue 6592),
//  and no control makes that stretch by waiting on an unloaded run.
//  It is one window for the process, so a control takes it before it plants anything.
// Limits:
//  Test-only. A release build reads no atomic and holds no lock here.

use std::sync::atomic::{
    AtomicI64,
    AtomicU32,
    Ordering,
};
use std::time::{
    Duration,
    Instant,
};

/// The ceiling a control planted,
/// in milliseconds,
/// or -1 for none.
static CEILING: AtomicI64 = AtomicI64::new(-1);
/// The bound a control planted for a step the kernel holds,
/// in milliseconds,
/// or -1 for none.
static STALL_CEILING: AtomicI64 = AtomicI64::new(-1);
/// What:
///  The stretch a control planted between two looks of one step's notice,
///  with the number of the step it applies to,
///  and nothing when no control planted one.
/// Why:
///  The step is named in the plant so a notice of another control in the same process keeps its own pace,
///  as the process the kernel holds is named in the look's window
///  (`scratch_step_look_below_control_window.rs`),
///  and a stretch stated for every notice would slow the lines another control times.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let aDelayBetweenLooks: { step: number; delay: Duration } | undefined = undefined;
/// ```
static A_DELAY_BETWEEN_LOOKS: std::sync::Mutex<Option<(i32, Duration)>> =
    std::sync::Mutex::new(None);
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
/// A control that needs this window and `scratch_step_look_below`'s takes the look window first,
/// as every control does,
///  because two orders in one process hold each other for ever in a threaded run
/// (`doc/troubleshooting/git-policy-cli-control-windows-taken-in-two-orders.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using window = takeTheWindow();
/// ```
pub(crate) fn take_the_window() -> std::sync::MutexGuard<'static, ()> {
    // A poisoned lock means another control panicked while holding it; the counts are still counts.
    let taken = WINDOW
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    CEILING.store(-1, Ordering::SeqCst);
    STALL_CEILING.store(-1, Ordering::SeqCst);
    LOOKS_THAT_FOUND_THE_STEP.store(0, Ordering::SeqCst);
    *A_DELAY_BETWEEN_LOOKS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
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
    STALL_CEILING.store(
        i64::try_from(bound.as_millis()).unwrap_or(i64::MAX),
        Ordering::SeqCst,
    );
}

/// The bound a control planted for a step the kernel holds,
/// or nothing when it planted none.
pub(crate) fn the_stall_ceiling_the_control_planted() -> Option<Duration> {
    return match STALL_CEILING.load(Ordering::SeqCst) {
        -1 => None,
        planted => Some(Duration::from_millis(
            u64::try_from(planted).unwrap_or(u64::MAX),
        )),
    };
}

/// Make the ceiling come at the next look.
pub(crate) fn plant_the_ceiling(ceiling: Duration) {
    CEILING.store(
        i64::try_from(ceiling.as_millis()).unwrap_or(i64::MAX),
        Ordering::SeqCst,
    );
}

/// The ceiling a control planted,
/// or nothing when it planted none.
pub(crate) fn the_ceiling_the_control_planted() -> Option<Duration> {
    return match CEILING.load(Ordering::SeqCst) {
        -1 => None,
        planted => Some(Duration::from_millis(
            u64::try_from(planted).unwrap_or(u64::MAX),
        )),
    };
}

/// What:
///  Note that a look found the step.
/// Why:
///  A control waits for the looks its assertion rests on rather than for a clock,
///  so every look that found the step is counted here,
///  and the figure a control asserts on rests on how many of them its own notice made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// looksThatFoundTheStep++;
/// ```
pub(crate) fn noted_a_look_that_found_the_step() {
    LOOKS_THAT_FOUND_THE_STEP.fetch_add(1, Ordering::SeqCst);
}

/// What:
///  Plant a stretch between two looks of `number`'s notice,
///  as a host under load put between two of them.
/// Why:
///   A loaded host woke the notice's thread late and stretched each look interval,
///   so looks that should have proved three intervals of a control's window proved two,
///   and the gate's record of `looks: 4, looks_found: 3` credited 639.46 ms where the control
///   asserted 750 ms (issue 6592).
///   A control asserts under that stretch instead of hoping for a punctual host.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// aLookOfThisStepWaits(stepNumber, 450);
/// ```
pub(crate) fn a_look_of_this_step_waits(
    number: i32,
    delay: Duration,
) {
    *A_DELAY_BETWEEN_LOOKS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some((number, delay));
}

/// What:
///  The stretch a control planted for this step's looks,
///  or nothing when it planted none for this step.
/// Why:
///   The notice thread reads this at every pause,
///   and a plant that named another step's number is not this step's stretch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const delay = aDelayBetweenLooks?.step === stepNumber ? aDelayBetweenLooks.delay : undefined;
/// ```
pub(crate) fn the_delay_between_looks_of(step: Option<i32>) -> Option<Duration> {
    let planted: Option<(i32, Duration)> = *A_DELAY_BETWEEN_LOOKS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    return match (planted, step) {
        (Some((number, delay)), Some(watched)) if number == watched => Some(delay),
        _ => None,
    };
}

/// Wait,
/// no longer than `bound`,
/// for a look to have found the step.
pub(crate) fn until_a_look_finds_the_step(bound: Duration) -> bool {
    return until_looks_find_the_step(1, bound);
}

/// What:
///  Wait,
///  no longer than `bound`,
///  for `count` looks to have found the step.
/// Why:
///   The record credits a program below the step with the time between the looks that found it,
///   one interval for each look after the first,
///   so a control that waits for a count of them holds the figure its assertion rests on
///   instead of racing a clock with it (issue 6592).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await until(() => looksThatFoundTheStep >= count, bound);
/// ```
pub(crate) fn until_looks_find_the_step(
    count: u32,
    bound: Duration,
) -> bool {
    return until(bound, || {
        return LOOKS_THAT_FOUND_THE_STEP.load(Ordering::SeqCst) >= count;
    });
}

/// Wait for `reached`,
/// no longer than `bound`.
fn until(
    bound: Duration,
    reached: impl Fn() -> bool,
) -> bool {
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
