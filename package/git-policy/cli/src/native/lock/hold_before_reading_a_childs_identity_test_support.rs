//! What:
//!  A hold a control places between a child's start and the reading of its identity,
//!  so the window a record written after a start must close can be shown without making a real machine slow.
//! Why:
//!  A child that dies at once is one the parent may be too late to name;
//!  a control that widens that window shows whether the record of that child is written at all.
//!  Kept in a file of its own so the shipped `lock_holder_records.rs` holds no inline test module
//!  (`rust/tests-in-sidecars`) and the runtime blocker scan reads no sleeping clock in release code;
//!  it is declared there under `#[cfg(test)]`,
//!  so the release executable never contains it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // a module that exists only in test builds
//! export const hold = { place, waitIfPlaced };
//! ```

use std::cell::Cell;
use std::time::Duration;

thread_local! {
    /// How long this thread waits before it reads the identity of a process it is naming.
    static HOLD: Cell<Option<Duration>> = const { Cell::new(None) };
}

/// What:
///  Make this thread wait `hold` before every identity it reads while naming a child,
///  until it is given nothing.
/// Why:
///   The wait is the window: the child runs freely while it lasts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// place(200) // milliseconds
/// ```
pub(crate) fn hold(hold: Option<Duration>) {
    HOLD.with(|cell| { cell.set(hold); });
}

/// What:
///  Wait the time a control placed, if it placed any.
/// Why:
///   Called where a child's identity is about to be read,
///  so a control decides how late that read is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// waitIfPlaced()
/// ```
pub(crate) fn wait_if_a_control_placed_one() {
    if let Some(hold) = HOLD.with(|cell| { return cell.get(); }) {
        std::thread::sleep(hold);
    }
}
