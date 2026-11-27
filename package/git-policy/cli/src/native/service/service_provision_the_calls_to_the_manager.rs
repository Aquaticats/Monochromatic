//! What:
//!  The calls provisioning makes to the service manager:
//!  a reset of a failed unit when one may have failed under the manager,
//!  the start of the worktree's socket,
//!  the line that says an earlier release's unit was not stopped,
//!  and the stop of the socket the earlier release set up for this worktree.
//! Why:
//!   A start that failed under an old unit leaves the service failed and its start counter full,
//!  and the new unit would be refused as "repeated too quickly" until the interval passed;
//!  while the earlier release's socket listens,
//!  a connection starts the earlier helper at any moment,
//!  which keeps the person's earlier batches from being settled on request.
//!  Nothing here fails the commit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await askManager(manager, known, instance); await stopEarlierInstanceSocket(host, manager);
//! ```

use crate::service_instance::{
    earlier_service_unit_instance,
    earlier_socket_unit_instance,
    service_unit_instance,
    socket_unit_instance,
};
use crate::service_provision_host_and_manager::{
    Manager,
    ProvisionHost,
};
use crate::service_provision_instance_unloaded::{
    describe,
    is_at_rest,
};
use crate::systemd_dbus::{
    CallError,
    EXCHANGE_WAIT,
    NO_SUCH_UNIT,
};
/// The calls provisioning makes:
/// a reset of failed state when a unit may have failed under the manager (the reset is best effort:
/// a unit the manager does not hold has nothing to reset),
/// and the start.
pub(crate) fn ask_manager(
    manager: &mut dyn Manager,
    known: bool,
    instance: &str,
) -> Result<(), CallError> {
    if known {
        // A start that failed under an old unit leaves the service failed and its start counter full, and the socket
        //  failed with it; the new unit would be refused as "repeated too quickly" until the interval passed.
        for unit in [
            service_unit_instance(instance),
            socket_unit_instance(instance),
        ] {
            let _reset = manager.reset_failed(unit.as_str());
        }
    }
    return manager.start(socket_unit_instance(instance).as_str());
}

/// Say in the service's log that the earlier release's unit for this worktree was not stopped,
/// and why.
fn warn_not_stopped(
    unit: &str,
    error: &CallError,
) {
    crate::service_warn!(
        "the commit service of the earlier release for this worktree ({unit}) was not stopped: {}; the batches it left open stay open, and the next commit tries again",
        describe(error)
    );
}

/// What:
///  Stops the socket unit `cli-git@<instance>.socket` that the earlier release set up for this very worktree,
/// when no helper
///  of that unit runs.
/// Nothing here fails the commit.
/// Why:
///   The owner's pick of 2026-11-07 ("Stop the old socket"):
/// while that socket listens,
/// a connection starts the earlier helper
///  at any moment,
/// which keeps the person's earlier batches from being settled on request
///  (`earlier_service_journal_adopted.rs`).
/// The instance name is the same text in both releases (the same hash of the same Git
///  directory),
/// so the unit is this worktree's and no other's.
/// The earlier release wrote only two shared templates,
/// from which
///  every worktree's instance is made,
/// so there is no file of this instance to remove and the templates stay;
/// a stopped instance
///  is unloaded by the manager.
/// A helper that runs is left as it is:
/// the earlier service `Requires=` its socket,
/// so a plain stop
///  of the socket stops it mid-batch,
/// and the stop that is sent names the one unit.
/// The socket file is never unlinked,
/// because
///  an earlier-release wrapper can make its unit listen again at that path between any answer of the manager and an unlink.
///  Whether the stop is still to be done is read from the table of listening sockets (`in_place_with`).
/// Every failure is logged
///  and ignored,
/// each call is bounded by one exchange's time,
/// and an earlier-release wrapper that sets the unit up again
///  undoes this (the owner accepted that).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await stopEarlierInstanceSocket(host, manager) // logs and returns on any failure
/// ```
pub(crate) fn stop_earlier_instance_socket(
    host: &ProvisionHost,
    manager: &mut dyn Manager,
) {
    let socket: String = earlier_socket_unit_instance(host.instance.as_str());
    let service: String = earlier_service_unit_instance(host.instance.as_str());
    // The start that came before may have used the time of its exchange; these calls have a time of their own.
    manager.within(EXCHANGE_WAIT);
    // A helper that runs finishes its own batches; it is not touched.
    match manager.active_state(service.as_str()) {
        Ok(Some(state)) if !is_at_rest(state.as_str()) => return,
        Ok(_) => {}
        Err(error) => return warn_not_stopped(service.as_str(), &error),
    }
    let held: Option<String> = match manager.active_state(socket.as_str()) {
        Ok(state) => state,
        Err(error) => return warn_not_stopped(socket.as_str(), &error),
    };
    if !matches!(&held, Some(state) if !is_at_rest(state.as_str())) {
        return;
    }
    match manager.stop_alone(socket.as_str()) {
        Ok(()) => {}
        Err(CallError::Manager {
            name,
            ..
        }) if name == NO_SUCH_UNIT => {}
        Err(error) => warn_not_stopped(socket.as_str(), &error),
    }
}
