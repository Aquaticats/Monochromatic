//! What:
//!  A control's seam in the lock pass: the place between reading a repository's records and walking its lock files,
//!  where another command's Git makes a lock the reading could not know about.
//! Why:
//!  That window is what makes the order of the two steps a rule, and no control can place a second command's lock inside
//!  a running pass without a place to stand.
//!  A control arms the seam and calls the pass; the pass runs the code once, where the window is.
//!  Kept in a file of its own so the shipped `abandoned_locks.rs` holds no inline test module (`rust/tests-in-sidecars`);
//!  it is declared there under `#[cfg(test)]`,
//! so the release executable never contains it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // a module that exists only in test builds
//! export const hooks = { afterTheRecordsAreRead, placeAfterTheRecordsAreRead };
//! ```

use std::cell::RefCell;

thread_local! {
    /// The code to run at the seam,
    ///  once,
    ///  on the thread that armed it.
    static AFTER_THE_RECORDS_ARE_READ: RefCell<Option<Box<dyn FnOnce()>>> = const { RefCell::new(None) };
}

/// What:
///  Place `code` at the seam:
///  the pass runs it once, after it read the repository's records and before it walks the lock files.
/// Why:
///   A control that wants a lock to appear in that window places one there and nothing waits on a clock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// placeAfterTheRecordsAreRead(() => makeTheLock())
/// ```
pub(crate) fn place_after_the_records_are_read(code: Box<dyn FnOnce()>) {
    AFTER_THE_RECORDS_ARE_READ.with(|slot| { *slot.borrow_mut() = Some(code); });
}

/// What:
///  Run whatever a control placed at the seam,
///  and forget it so the next pass runs nothing.
/// Why:
///   The pass calls this once per invocation,
///  and a hook left armed would run on a later command of the same test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// afterTheRecordsAreRead() // takes and runs the placed code, if any
/// ```
pub(crate) fn after_the_records_are_read() {
    let placed: Option<Box<dyn FnOnce()>> = AFTER_THE_RECORDS_ARE_READ.with(|slot| { return slot.borrow_mut().take(); });
    if let Some(code) = placed {
        code();
    }
}
