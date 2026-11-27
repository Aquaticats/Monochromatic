//! What:
//!  A stand-in for the user manager a control's fixture has none of:
//!  a thread that starts again any service of the fixture that ends,
//!  as production does at the next connection to the manager's socket,
//!  and a flag that ends that thread however the control ends.
//! Why:
//!  A fault control kills or stops a service on purpose,
//!  and a wrapper command that then asks for one must find a manager that starts it again;
//!  the stand-in is its own job,
//!  so it lives in a file of its own and keeps the run file inside the per-file line budget.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! withManager(fixture, () => { /* a control whose services may end */ });
//! ```

use super::support::{
    Fixture,
    restart_ended_services,
};
use std::sync::atomic::{
    AtomicBool,
    Ordering,
};
use std::time::Duration;

/// Sets a flag when dropped,
/// so a thread that polls it ends however its owner ends.
struct EndsOnDrop<'flag>(&'flag AtomicBool);

impl Drop for EndsOnDrop<'_> {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

/// How often the stand-in for the user manager looks for an ended service.
const MANAGER_POLL: Duration = Duration::from_millis(5);

/// Runs `body` while a stand-in for the user manager starts again any service of the fixture that ends,
/// as production does at the
///  next connection to the manager's socket.
pub(super) fn with_manager<T>(
    fixture: &Fixture,
    body: impl FnOnce() -> T,
) -> T {
    let finished: AtomicBool = AtomicBool::new(false);
    return std::thread::scope(|scope| {
        scope.spawn(|| {
            while !finished.load(Ordering::SeqCst) {
                restart_ended_services(fixture);
                std::thread::sleep(MANAGER_POLL);
            }
        });
        let _ends = EndsOnDrop(&finished);
        return body();
    });
}
