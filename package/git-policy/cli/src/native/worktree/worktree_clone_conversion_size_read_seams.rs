//! What:
//!  The seams a control arms to stand where no real read can be made to stand:
//!  a read held before its Git run,
//!  a read held after its child ended and before the sizes are parsed,
//!  and a read whose thread dies before it answers.
//! Why:
//!   No fixture makes a real `git ls-tree` hang or die,
//!  and the sentences the join gives for those states are the facts a control has to see;
//!  each seam is a fact of the whole process,
//!  so its arm refuses outside the child process a control runs its body in.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const hold = aControlHoldsTheRead(); // every read of this process waits here until hold is dropped
//! ```

#[cfg(test)]
use std::time::Duration;

#[cfg(test)]
static THE_READ_IS_HELD: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[cfg(test)]
static THE_READ_DIES: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[cfg(test)]
static THE_HELD_READ: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
static THE_HELD_READ_LETS_GO: std::sync::Condvar = std::sync::Condvar::new();

/// What:
///  Hold the read until a control lets it go,
/// so the join meets its deadline with a read that never answered.
/// Why:
///   No fixture can make a real `git ls-tree` hang,
/// and the sentence a deadline gives is the one a failed read gives,
///  which is the fact a control has to see.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (THE_READ_IS_HELD) await new Promise(() => {}); // never answers
/// ```
#[cfg(test)]
pub(crate) fn hold_the_read_where_a_control_asked() {
    if !THE_READ_IS_HELD.load(std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    let held = THE_HELD_READ
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _lets_go = THE_HELD_READ_LETS_GO
        .wait_timeout(held, Duration::from_secs(30))
        .unwrap_or_else(std::sync::PoisonError::into_inner);
}

#[cfg(test)]
static THE_SIZES_ARE_HELD_AFTER_THE_WAIT: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

#[cfg(test)]
static THE_SIZES_WERE_HELD: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

#[cfg(test)]
static THE_HELD_SIZES: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
static THE_HELD_SIZES_LET_GO: std::sync::Condvar = std::sync::Condvar::new();

/// What:
///  Hold the read after its child ended and before the sizes are parsed,
///  which is where the old code still named the reaped child.
/// Why:
///   The integration recheck's Low 1 of 2026-11-22 is that an abandon in that window signals a process id
///  the kernel has already reaped and may have reused;
/// a control has to stand in that window to show the cell is cleared before it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (THE_SIZES_ARE_HELD_AFTER_THE_WAIT) await new Promise(() => {});
/// ```
#[cfg(test)]
pub(crate) fn hold_the_sizes_after_the_wait_where_a_control_asked() {
    if !THE_SIZES_ARE_HELD_AFTER_THE_WAIT.load(std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    THE_SIZES_WERE_HELD.store(true, std::sync::atomic::Ordering::SeqCst);
    let held = THE_HELD_SIZES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _lets_go = THE_HELD_SIZES_LET_GO
        .wait_timeout(held, Duration::from_secs(30))
        .unwrap_or_else(std::sync::PoisonError::into_inner);
}

/// What:
///  A control that holds the read after its child ended and before the parse,
///  with the hold's own start of it.
/// Why:
///   See `hold_the_sizes_after_the_wait_where_a_control_asked`;
/// the hold is a seam of the whole process,
///  so the control runs in a child process of its own
/// (`crate::process_wide_state_child_test_support`),
///  and `refuse_outside_the_child` fails a body that arms it beside other tests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hold = aControlHoldsTheSizesAfterTheWait(); // the read waits there until hold is dropped
/// ```
#[cfg(test)]
pub(crate) fn a_control_holds_the_sizes_after_the_wait() -> TheHeldSizesForTheControl {
    crate::process_wide_state_child_test_support::refuse_outside_the_child(
        "THE_SIZES_ARE_HELD_AFTER_THE_WAIT, the hold every read of this process waits at after its child ended",
    );
    THE_SIZES_WERE_HELD.store(false, std::sync::atomic::Ordering::SeqCst);
    THE_SIZES_ARE_HELD_AFTER_THE_WAIT.store(true, std::sync::atomic::Ordering::SeqCst);
    return TheHeldSizesForTheControl;
}

/// Whether the held read has reached the seam after its wait,
/// so a control can look at the cell exactly there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reached = () => THE_SIZES_WERE_HELD;
/// ```
#[cfg(test)]
pub(crate) fn the_sizes_hold_was_reached() -> bool {
    return THE_SIZES_WERE_HELD.load(std::sync::atomic::Ordering::SeqCst);
}

/// The hold of `a_control_holds_the_sizes_after_the_wait`,
/// released when it is dropped.
#[cfg(test)]
pub(crate) struct TheHeldSizesForTheControl;

/// Lets the held read go after the wait.
#[cfg(test)]
impl Drop for TheHeldSizesForTheControl {
    fn drop(&mut self) {
        THE_SIZES_ARE_HELD_AFTER_THE_WAIT.store(false, std::sync::atomic::Ordering::SeqCst);
        THE_HELD_SIZES_LET_GO.notify_all();
    }
}

/// What:
///  Make the read's thread die before it answers.
/// Why:
///   A dead thread is the other way the join answers nothing,
/// and no fixture makes a real read die.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (THE_READ_DIES) throw new Error('a control makes the read die');
/// ```
#[cfg(test)]
pub(crate) fn die_where_a_control_asked() {
    if THE_READ_DIES.load(std::sync::atomic::Ordering::SeqCst) {
        panic!("a control makes the size read die before it answers");
    }
}

/// What:
///  A control that holds the read until this value is dropped,
/// with the read's own start of it.
/// Why:
///   The seams are process-wide because the read runs on a thread of its own,
///  so the control runs in a child process of its own
/// (`crate::process_wide_state_child_test_support`),
///  and `refuse_outside_the_child` fails a body that arms it beside other tests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hold = aControlHoldsTheRead(); // the read waits at its start until hold.release()
/// ```
#[cfg(test)]
pub(crate) fn a_control_holds_the_read() -> TheHeldReadForTheControl {
    crate::process_wide_state_child_test_support::refuse_outside_the_child(
        "THE_READ_IS_HELD, the hold every read of this process waits at before its Git run",
    );
    THE_READ_IS_HELD.store(true, std::sync::atomic::Ordering::SeqCst);
    return TheHeldReadForTheControl;
}

/// The hold of `a_control_holds_the_read`,
/// released when it is dropped.
#[cfg(test)]
pub(crate) struct TheHeldReadForTheControl;

/// Lets the held read go,
/// so the process ends without it.
#[cfg(test)]
impl Drop for TheHeldReadForTheControl {
    fn drop(&mut self) {
        THE_READ_IS_HELD.store(false, std::sync::atomic::Ordering::SeqCst);
        THE_HELD_READ_LETS_GO.notify_all();
    }
}

/// What:
///  A control that makes the read's task die before it answers.
/// Why:
///   See `die_where_a_control_asked`;
/// the stand-in is a seam of the whole process,
///  so the control runs in a child process of its own
/// (`crate::process_wide_state_child_test_support`),
///  and `refuse_outside_the_child` fails a body that arms it beside other tests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const dies = aControlMakesTheReadDie(); // the next read throws until dies is dropped
/// ```
#[cfg(test)]
pub(crate) fn a_control_makes_the_read_die() -> TheDeadReadForTheControl {
    crate::process_wide_state_child_test_support::refuse_outside_the_child(
        "THE_READ_DIES, the stand-in that makes every read of this process die before it answers",
    );
    THE_READ_DIES.store(true, std::sync::atomic::Ordering::SeqCst);
    return TheDeadReadForTheControl;
}

/// The stand-in of `a_control_makes_the_read_die`,
/// cleared when it is dropped.
#[cfg(test)]
pub(crate) struct TheDeadReadForTheControl;

/// Clears the control's stand-in.
#[cfg(test)]
impl Drop for TheDeadReadForTheControl {
    fn drop(&mut self) {
        THE_READ_DIES.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}
