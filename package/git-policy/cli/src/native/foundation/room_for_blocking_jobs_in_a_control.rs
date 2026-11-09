//! What:
//!  How much room a control gives this process for blocking jobs at once,
//!  overriding what the CPUs it may use would allow.
//! Why:
//!  A control that proves its work overlaps puts every item in flight at once,
//! and the process's room comes from the CPUs the process may use
//! (`budgeted_workers`),
//!  so on a machine with few CPUs such a control could only fail or pass without running,
//! and a control that does not run proves nothing there
//! (`doc/planning/git-policy-cli-full-tokio.md`,
//! "the process's one bound on blocking jobs").
//! The room is set before the runtime is built and before the bound is made,
//!  so both the runtime's blocking pool and the bound on blocking jobs take it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! forAControl(4); // this process may run four blocking jobs at once, whatever the host says
//! ```

use std::sync::atomic::{AtomicUsize, Ordering};

/// The room a control asked for,
/// or zero when none did.
static ROOM: AtomicUsize = AtomicUsize::new(0);

/// What:
///  Gives this process room for `jobs` blocking jobs at once,
///  whatever the CPUs it may use would allow,
///  and answers with the room that is now in force.
/// Why:
///  Call it first in a control:
/// a runtime and a bound that are already made keep their sizes,
///  so a later call changes nothing.
pub(crate) fn for_a_control(jobs: usize) -> usize {
    ROOM.store(jobs.max(1), Ordering::SeqCst);
    return jobs.max(1);
}

/// The room a control asked for,
/// or nothing when none did.
pub(crate) fn the_room() -> Option<usize> {
    return match ROOM.load(Ordering::SeqCst) {
        0 => None,
        asked => Some(asked),
    };
}
