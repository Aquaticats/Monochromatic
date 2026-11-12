//! What:
//!  A control's statements about one look at the step and the programs below it:
//!  which process the kernel holds in uninterruptible sleep,
//!  and how many of the next looks cannot be made at all.
//! Why:
//!  No run this package makes can put a process into uninterruptible sleep:
//!  it takes a stalled disk,
//!  a dead network file system or a stopped file-system server,
//!  and the gate's runs mount nothing (`BOX`).
//!  The stop a control sends is real and the step it ends is real;
//!  what a control states here is the kernel's answer about a process's state,
//!  which is the one place that state is read.
//!  A look that cannot be made is stated here for the same reason:
//!  a descriptor limit that makes `/proc` unreadable is this test process's own,
//!  and a control of the retry must choose which look fails and not race a resource.
//!  Both are one window for the process,
//!  so a control takes it before it plants anything.
//! Limits:
//!  Test-only.
//!  A release build reads no atomic and holds no lock here.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard};

/// The process a control states the kernel holds,
/// and the kernel's name of it,
/// or nothing when no control states one.
static HELD: Mutex<Option<(i32, String)>> = Mutex::new(None);
/// The name a control gives whichever process a look finds,
/// or nothing when no control states that one is held at all.
static HOLDS_WHATEVER_IT_FINDS: Mutex<Option<String>> = Mutex::new(None);
/// How many of the next looks a control states cannot be made.
static LOOKS_THAT_CANNOT_BE_MADE: AtomicUsize = AtomicUsize::new(0);
/// Held by the control that plants anything here.
static WINDOW: Mutex<()> = Mutex::new(());

/// What:
///  Take the window for the control that is about to use it,
///  with nothing planted.
/// Why:
///  A control that read another's plant would test the wrong look.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using window = takeTheWindow();
/// ```
pub(crate) fn take_the_window() -> MutexGuard<'static, ()> {
    // A poisoned lock means another control panicked while holding it; nothing planted is still nothing planted.
    let taken: MutexGuard<'static, ()> = WINDOW.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    *HELD.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    *HOLDS_WHATEVER_IT_FINDS.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    LOOKS_THAT_CANNOT_BE_MADE.store(0, Ordering::SeqCst);
    return taken;
}

/// What:
///  State that the kernel holds `number` in uninterruptible sleep,
///  as no control on this host can make it do.
/// Why:
///  The line a wait says about a process the kernel has not released is the whole of what a person learns,
///  so the process it names is the one fact a control must be able to choose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theKernelHolds(4242, 'ssh-keygen'); // a stand-in for a process in state `D`
/// ```
pub(crate) fn the_kernel_holds(number: i32, name: &str) {
    *HELD.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some((number, name.to_string()));
}

/// The process a control states the kernel holds,
/// or nothing when it states none.
pub(crate) fn a_control_says_the_kernel_holds() -> Option<(i32, String)> {
    return HELD.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
}

/// What:
///  State that the kernel holds whichever process a look finds,
///  under `name`,
///  as no control on this host can make it do.
/// Why:
///  A control that runs the real commit cannot know the number of the `git commit` its scratch repository runs
///  before that step starts,
///  so it cannot name that process.
///  What it can state is the kernel's answer about the process the step is:
///  a step waiting for a disk.
///  The step comes first in a look,
///  so the process this states is the step itself and never a program below it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theKernelHoldsWhateverTheLookFinds('git'); // a stand-in for a step in state `D`
/// ```
pub(crate) fn the_kernel_holds_whatever_the_look_finds(name: &str) {
    *HOLDS_WHATEVER_IT_FINDS.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(name.to_string());
}

/// The name a control gives whichever process a look finds,
/// or nothing when no control states one is held.
pub(crate) fn the_name_a_control_gives_the_process_it_holds() -> Option<String> {
    return HOLDS_WHATEVER_IT_FINDS.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
}

/// State that the next `count` looks cannot be made.
pub(crate) fn plant_looks_that_cannot_be_made(count: usize) {
    LOOKS_THAT_CANNOT_BE_MADE.store(count, Ordering::SeqCst);
}

/// Whether this look cannot be made,
/// which uses up one planted look.
pub(crate) fn a_control_says_this_look_cannot_be_made() -> bool {
    return LOOKS_THAT_CANNOT_BE_MADE
        .try_update(Ordering::SeqCst, Ordering::SeqCst, |left| { return left.checked_sub(1); })
        .is_ok();
}
