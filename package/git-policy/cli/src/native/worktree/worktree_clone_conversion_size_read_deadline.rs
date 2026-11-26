//! What:
//!  The join's shipped deadline,
//! and the control that shortens it for one thread.
//! Why:
//!   The size-read module keeps under the line limit,
//! and a deadline and its control are one subject:
//! the module's join reads the shipped value through this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const deadline = aControlShortenedIt() ?? THE_JOIN_DEADLINE;
//! ```

use std::time::Duration;

/// The deadline in force for this join:
/// the shipped one,
/// or one a control asked for on this thread.
/// Why:
///   The shipped deadline is two minutes long,
/// so a control that shows the deadline's own sentence cannot spend it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const deadline = THE_JOIN_DEADLINE_FOR_THIS_THREAD.get() ?? THE_JOIN_DEADLINE;
/// ```
pub(crate) fn the_deadline(shipped: Duration) -> Duration {
    #[cfg(test)]
    if let Some(asked) = THE_JOIN_DEADLINE_FOR_THIS_THREAD.with(std::cell::Cell::get) {
        return asked;
    }
    return shipped;
}

#[cfg(test)]
thread_local! {
    /// A control's own deadline for the join on this thread;
    /// nothing outside a control.
    static THE_JOIN_DEADLINE_FOR_THIS_THREAD: std::cell::Cell<Option<Duration>> = const { std::cell::Cell::new(None) };
}

/// What:
///  A shortened join deadline for this thread,
/// cleared when the value is dropped.
/// Why:
///   The shipped deadline is two minutes;
/// a control that shows its sentence needs a moment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const deadline = aControlShortensTheDeadlineTo(50); // deadline === 50 ms until dropped
/// ```
#[cfg(test)]
pub(crate) fn a_control_shortens_the_deadline_to(within: Duration) -> TheDeadlineForTheControl {
    THE_JOIN_DEADLINE_FOR_THIS_THREAD.with(|deadline| {
        deadline.set(Some(within));
    });
    return TheDeadlineForTheControl;
}

/// The stand-in of `a_control_shortens_the_deadline_to`,
/// cleared when it is dropped.
#[cfg(test)]
pub(crate) struct TheDeadlineForTheControl;

/// Clears the shortened deadline.
#[cfg(test)]
impl Drop for TheDeadlineForTheControl {
    fn drop(&mut self) {
        THE_JOIN_DEADLINE_FOR_THIS_THREAD.with(|deadline| {
            deadline.set(None);
        });
    }
}
