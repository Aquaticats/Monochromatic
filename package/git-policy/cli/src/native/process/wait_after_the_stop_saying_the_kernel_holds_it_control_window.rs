//! What:
//!  A control's hold on the reap of a step past its stop.
//! Why:
//!   The wait this package's step keeps after a ceiling lasts as long as the kernel has not released the step,
//!   and every step a control can start acts on `SIGKILL` at once,
//!  so its reap returns before a look after the pause is made.
//!  A control states that the reap has not returned,
//!  which is the one fact a real uninterruptible call would give it,
//!  and the kernel's answer about the process's state is stated beside it
//!  (`commit/scratch_step_look_below_control_window.rs`).
//!  It is one window for the process,
//!  so a control takes it before it holds anything.
//! Limits:
//!  Test-only.
//!  A release build reads no atomic and holds no lock here.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

/// Whether a control states that the step's end has not been collected.
static HELD: AtomicBool = AtomicBool::new(false);
/// Held by the control that holds a reap.
static WINDOW: Mutex<()> = Mutex::new(());
/// The longest a control's hold lasts,
/// so a control's mistake ends the run with this wait's own answer.
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
    return;
}
