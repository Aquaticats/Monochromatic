//! What:
//!  Stops this service's own socket when the service was retired before it served anything.
//! Why:
//!   A client of another release asks this service to retire,
//! and the manager starts the service again for that client's next
//!  connection,
//! so a client that retries every hundred milliseconds starts and retires one service after another until the
//!  manager's start limit ends the unit as failed
//! (`doc/troubleshooting/`,
//! the install of `0.6.1-alpha.21` on
//!  2026-11-13).
//! A service that has served nothing has nothing to protect by staying,
//! and a socket that is stopped cannot start it
//!  again;
//! the client's own wait starts the unit when it finds it inactive,
//! which is its documented repair
//!  (`service_client_reach_service.rs`).
//! Only the socket is stopped,
//! and in the mode that leaves the service alone:
//! this process is already
//!  ending,
//! and a stop of its own unit from inside would signal it before it finished what it was doing.
//! A manager that cannot
//!  be asked only logs,
//! and the next command in that worktree starts the unit again.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! stopOwnSocketBeforeServingAnything(instance) // true: the socket was stopped
//! ```

use crate::service_instance::socket_unit_instance;
use crate::service_provision_host_and_manager::Manager;
use crate::service_route::non_empty_path_variable;
use crate::systemd_dbus::{Bus, CallError, EXCHANGE_WAIT, NO_SUCH_UNIT, session_bus_path};

/// What:
///  Stops the socket of `instance` when this service is of one and was retired before it served anything.
/// Why:
///   The unit's socket is what starts the service for every arriving connection,
/// so a service that keeps exiting at once keeps the socket
///  starting it,
/// and the manager's start limit turns that into a unit that stays failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stopOwnSocketBeforeServingAnything(instance)
/// ```
pub fn stop_own_socket_before_serving_anything(instance: &str) -> bool {
    let Some(runtime_dir) = non_empty_path_variable("XDG_RUNTIME_DIR") else { return false };
    let address: Option<String> = std::env::var("DBUS_SESSION_BUS_ADDRESS").ok();
    let Some(socket) = session_bus_path(address.as_deref(), Some(runtime_dir.as_path())) else {
        crate::service_warn!(
            "this commit service of {instance} was retired before it served anything, but no session bus was found to stop its socket, so a connection starts it again"
        );
        return false;
    };
    let mut bus: Bus = match Bus::connect(socket.as_path(), EXCHANGE_WAIT) {
        Ok(connected) => connected,
        Err(error) => {
            crate::service_warn!(
                "this commit service of {instance} was retired before it served anything, but the session bus could not be reached ({error:?}) to stop its socket, so a connection starts it again"
            );
            return false;
        }
    };
    bus.within(EXCHANGE_WAIT);
    return match stop_alone(&mut bus, socket_unit_instance(instance).as_str()) {
        Ok(()) => {
            crate::service_log!("this commit service of {instance} was retired before it served anything, and it stopped its own socket so no connection starts it again");
            true
        }
        Err(error) => {
            crate::service_warn!("the socket of this commit service of {instance} was not stopped ({error:?}), so a connection starts the service again");
            false
        }
    };
}

/// One stop of a unit and of nothing that requires it,
/// where a unit the manager does not hold is stopped already.
fn stop_alone(manager: &mut dyn Manager, unit: &str) -> Result<(), CallError> {
    return match manager.stop_alone(unit) {
        Err(CallError::Manager { name, .. }) if name == NO_SUCH_UNIT => Ok(()),
        other => other,
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "service_own_socket_of_a_retire_that_served_nothing_tests.rs"]
mod tests;
