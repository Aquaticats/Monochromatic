//! What:
//!  Whether a stopped commit service's processes are gone,
//!  which is what lets the journal it writes be removed.
//! Why:
//!   A stop call returns when the manager queued the job (`systemd_dbus.rs`),
//! so the answer that a unit was stopped is not the answer that nothing writes any more.
//! The sweep of removed worktrees and the removal a person runs both ask this before they remove a journal
//! (`service_units_of_removed_worktrees.rs`,
//! `worktree_removal_stops_units.rs`),
//! and a journal whose service may still run keeps the note and the socket file that name its instance,
//! so a later sweep finishes the removal instead of a record being deleted under a running writer.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await serviceIsGone(manager, instance, SERVICE_GONE_AFTER_A_STOP) // true only when no process of the unit is left
//! ```

use crate::service_instance::service_unit_instance;
use crate::service_provision_host_and_manager::Manager;
use crate::wait_ceilings::SERVICE_GONE_AFTER_A_STOP;
use std::time::{Duration, Instant};

/// How often the manager is asked whether a stopped service's processes are gone.
/// Why:
///   The same interval the retirement of an earlier service polls at,
/// which is short against every stop and long against the manager's own work.
const SERVICE_GONE_POLL: Duration = Duration::from_millis(50);

/// What:
///  Whether the service unit of an instance has no process left,
///  asked for at most `SERVICE_GONE_AFTER_A_STOP`.
/// Why:
///   Only the unit's own state says that the service and its processes are gone,
/// and a manager that cannot answer proves nothing,
/// so the answer is `false` then.
///  Bounded:
/// a unit that outlasts the wait just keeps its journal for a later sweep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await serviceIsGone(manager, instance) // true only when the manager reports no process
/// ```
pub(crate) fn service_is_gone(manager: &mut dyn Manager, instance: &str) -> bool {
    let deadline: Instant = Instant::now() + SERVICE_GONE_AFTER_A_STOP;
    let unit: String = service_unit_instance(instance);
    loop {
        match manager.active_state(unit.as_str()) {
            // A unit the manager does not hold has no process of ours.
            Ok(None) => return true,
            Ok(Some(state)) if matches!(state.as_str(), "inactive" | "failed") => return true,
            Ok(Some(_running)) => {}
            Err(_unanswerable) => return false,
        }
        let now: Instant = Instant::now();
        if now >= deadline {
            return false;
        }
        std::thread::sleep(SERVICE_GONE_POLL.min(deadline.saturating_duration_since(now)));
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "service_is_gone_after_a_stop_tests.rs"]
mod tests;
