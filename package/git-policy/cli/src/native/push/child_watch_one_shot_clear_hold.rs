//! What:
//!  The hold a control takes between a watched step's end
//!  and the clearing of the cell that published the child's number:
//!  the four facts of the whole process that make the window stand still,
//!  the seam the runner waits at,
//!  and the two calls a control makes on them.
//! Why:
//!   A control has to stand in that window to show that an abandon which comes there
//!  is not erased by the clear,
//!  and no fixture can make a real `git ls-tree` end at a chosen moment inside the runner;
//!  the hold and its release sit in a file of their own,
//!  because the run they hold is more than the package's length limit allows one file
//!  (`rust/max-lines`),
//!  and `child_watch_one_shot_run.rs` re-exports the two calls a control makes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const hold = aControlHoldsTheClear(); // the runner waits there until hold is dropped
//! theClearHoldWasReached(); // false until the runner stands in the window
//! ```

/// What:
///  Whether a control asked the runner to hold before its clear.
/// Why:
///   The runner reads it at the seam,
///  and a control arms it only inside a child process of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let THE_CLEAR_IS_HELD = false;
/// ```
#[cfg(test)]
static THE_CLEAR_IS_HELD: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// What:
///  Whether the held runner has reached the seam yet.
/// Why:
///   A control waits on it before it abandons the read,
///  so the abandon lands inside the window rather than anywhere near it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let THE_CLEAR_WAS_REACHED = false;
/// ```
#[cfg(test)]
static THE_CLEAR_WAS_REACHED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// What:
///  The lock the held runner waits on.
/// Why:
///   A wait needs one,
///  and no work is done under it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_HELD_CLEAR = new Mutex();
/// ```
#[cfg(test)]
static THE_HELD_CLEAR: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// What:
///  The condition a control signals to let the held runner go.
/// Why:
///   The runner must not be woken by anything but the end of the hold,
///  or the window it stands in would close on its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_HELD_CLEAR_LETS_GO = new Condvar();
/// ```
#[cfg(test)]
static THE_HELD_CLEAR_LETS_GO: std::sync::Condvar = std::sync::Condvar::new();

/// What:
///  Hold the runner between the child's end and the clear of the cell that published it.
/// Why:
///   A control has to stand in that window to show that an abandon which comes there
///  is not erased by the clear;
/// no fixture can make a real `git ls-tree` end at a chosen moment inside the runner.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (THE_CLEAR_IS_HELD) await new Promise(() => {}); // the runner waits here until a control lets go
/// ```
#[cfg(test)]
pub(crate) fn hold_the_clear_where_a_control_asked() {
    if !THE_CLEAR_IS_HELD.load(std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    THE_CLEAR_WAS_REACHED.store(true, std::sync::atomic::Ordering::SeqCst);
    let held = THE_HELD_CLEAR
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _lets_go = THE_HELD_CLEAR_LETS_GO
        .wait_timeout(held, std::time::Duration::from_secs(30))
        .unwrap_or_else(std::sync::PoisonError::into_inner);
}

/// What:
///  A control that holds the runner before its clear,
///  with the hold's own start of it.
/// Why:
///   See `hold_the_clear_where_a_control_asked`;
/// the hold is a seam of the whole process,
///  so the control runs in a child process of its own
/// (`crate::process_wide_state_child_test_support`),
///  and `refuse_outside_the_child` fails a body that arms it beside other tests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hold = aControlHoldsTheClear(); // the runner waits there until hold is dropped
/// ```
#[cfg(test)]
pub(crate) fn a_control_holds_the_clear() -> TheHeldClearForTheControl {
    crate::process_wide_state_child_test_support::refuse_outside_the_child(
        "THE_CLEAR_IS_HELD, the hold every watched Git run of this process waits at before its clear",
    );
    THE_CLEAR_WAS_REACHED.store(false, std::sync::atomic::Ordering::SeqCst);
    THE_CLEAR_IS_HELD.store(true, std::sync::atomic::Ordering::SeqCst);
    return TheHeldClearForTheControl;
}

/// The hold of `a_control_holds_the_clear`,
/// released when it is dropped.
#[cfg(test)]
pub(crate) struct TheHeldClearForTheControl;

/// Lets the held runner go.
#[cfg(test)]
impl Drop for TheHeldClearForTheControl {
    fn drop(&mut self) {
        THE_CLEAR_IS_HELD.store(false, std::sync::atomic::Ordering::SeqCst);
        THE_HELD_CLEAR_LETS_GO.notify_all();
    }
}

/// What:
///  Whether the held runner has reached the seam before its clear,
/// so a control can abandon the read exactly there.
/// Why:
///   The window is between the child's end and the clear;
/// a control that looked at any other moment would see the cell already cleared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reached = () => THE_CLEAR_WAS_REACHED;
/// ```
#[cfg(test)]
pub(crate) fn the_clear_hold_was_reached() -> bool {
    return THE_CLEAR_WAS_REACHED.load(std::sync::atomic::Ordering::SeqCst);
}
