//! What:
//!  Drops the service manager's copy of one worktree's units,
//! so that the next start reads the unit files as they are now.
//! Why:
//!  The manager keeps a unit it once loaded until the unit is inactive and nothing references it;
//! a start of a unit it
//!  still holds runs the text it read the first time,
//! whatever the files say by then.
//! The way to make it read the files again is to let the
//!  unit go,
//! never to make the manager re-read every unit it holds:
//! a reload costs the manager its event loop for a
//!  time that grows with everything else it runs,
//! and on 2026-11-12 one of those ended a desktop session
//!  (`doc/troubleshooting/plasma-session-ended-after-service-manager-timeouts.md`).
//! A stopped unit that nothing references
//!  is collected on the manager's own event loop,
//! which was 11 ms away in a disposable manager holding thousands of units
//!  (`doc/planning/git-wrapper-reload-remedy.md`,
//! the unload measurement).
//! The service is asked to retire before it is stopped,
//! so a batch it
//!  is landing is finished rather than cut,
//! and every step here names one unit of this worktree:
//! the work stays inside the
//!  wrapper's own service's life.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await unloadInstance(host, manager) // stop this worktree's units, wait for the manager to drop them
//! ```

use crate::service_client_listen_and_ask_to_retire::ask_to_retire;
use crate::service_instance::{service_unit_instance, socket_path, socket_unit_instance};
use crate::service_messages::ClientFailure;
use crate::service_provision_host_and_manager::refusal;
use crate::service_provision_host_and_manager::{Manager, ProvisionHost};
use crate::systemd_dbus::CallError;

/// The longest the manager is given to drop a unit after it was stopped.
///  Measured:
/// a disposable user manager dropped a stopped socket and service instance on the first 50 ms poll,
/// 11 ms after the
///  stop was answered,
/// with 4,959 units loaded (`doc/planning/git-wrapper-reload-remedy.md`).
/// Five seconds is 450 times that,
/// and it is only reached by a manager that cannot make progress at all.
pub const UNLOAD_WAIT: std::time::Duration = std::time::Duration::from_secs(5);

/// How often the manager is asked whether it still holds the units.
const UNLOAD_POLL: std::time::Duration = std::time::Duration::from_millis(50);

/// The manager's failure in words for a log line.
pub(crate) fn describe(error: &CallError) -> String {
    return match error {
        CallError::Connection(reason) => format!("the session bus could not be reached ({reason})"),
        CallError::Authentication(reason) => format!("the session bus refused the connection ({reason})"),
        CallError::TimedOut => String::from("the session bus did not answer in time"),
        CallError::Manager { text, name } if text.is_empty() => format!("the service manager refused the request ({name})"),
        CallError::Manager { text, .. } => format!("the service manager refused the request ({text})"),
        CallError::Malformed(reason) => format!("the service manager's answer could not be read ({reason})"),
    };
}

/// Whether a unit's state says it runs no process and listens on nothing:
/// `inactive` or `failed`.
pub(crate) fn is_at_rest(state: &str) -> bool {
    return matches!(state, "inactive" | "failed");
}

/// What:
///  The failure for a unit the manager still holds after the unload wait.
/// Why:
///  A start now would run the text the manager read earlier,
/// which is the release that was replaced,
/// so the
///  commit must not go on as if the unit were current;
/// the person is told which unit and what to try.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stillLoaded(unit) // could-not-start with the unit's name
/// ```
fn still_loaded(unit: &str) -> ClientFailure {
    return ClientFailure::CouldNotStart(format!(
        "the service manager still holds this worktree's unit ({unit}) {:.0} seconds after it was stopped, and a start now would run the release that was replaced; run the commit again in a moment",
        UNLOAD_WAIT.as_secs_f64()
    ));
}

/// What:
///  Stops this worktree's service and socket units and waits until the manager no longer holds them.
/// Why:
///   Until it holds neither,
/// a start may run the text it read before the files changed.
/// The socket is stopped in the mode
///  that applies the job to it and to the units that require it,
/// so the service goes with it and the socket cannot start
///  it again meanwhile;
/// a manager that holds no socket has the service stopped alone,
/// which is what a manager that
///  unloaded the socket on its own leaves.
/// A service that runs is asked to retire first,
/// and the wait for it to leave
///  is the retirement wait of the host,
/// because a batch it is landing is finished rather than cut.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await unloadInstance(host, manager) // throws when the manager keeps the unit
/// ```
pub(crate) fn unload_instance(host: &ProvisionHost, manager: &mut dyn Manager) -> Result<(), ClientFailure> {
    let instance: &str = host.instance.as_str();
    let service: String = service_unit_instance(instance);
    let socket: String = socket_unit_instance(instance);
    // What the manager holds of the two, read once: a manager that holds neither has nothing to drop, and the two questions
    //  are about this worktree's units alone.
    manager.within(host.retirement_wait);
    // What: A question the manager cannot answer leaves this worktree's units as they are and the commit going.
    // Why:  The answer only decides whether they have to be dropped; nothing can be dropped on a guess, the commit does not
    //       need the repair to land, and the record is not written, so the next commit asks again.
    let service_state: Option<String> = match manager.active_state(service.as_str()) {
        Ok(found) => found,
        Err(error) => {
            crate::service_warn!(
                "the service manager could not say whether it holds this worktree's service unit ({service}): {}; the commit goes on without unloading it, and the next commit asks again",
                describe(&error)
            );
            return Ok(());
        }
    };
    let socket_state: Option<String> = match manager.active_state(socket.as_str()) {
        Ok(found) => found,
        Err(error) => {
            crate::service_warn!(
                "the service manager could not say whether it holds this worktree's socket unit ({socket}): {}; the commit goes on without unloading it, and the next commit asks again",
                describe(&error)
            );
            return Ok(());
        }
    };
    if service_state.is_none() && socket_state.is_none() {
        return Ok(());
    }
    // A service that runs is asked to leave by itself before anything is stopped, so the batch it holds is finished.
    if !matches!(&service_state, Some(state) if is_at_rest(state.as_str())) {
        ask_to_retire(socket_path(host.runtime_dir.as_path(), instance).as_path());
        wait_until_at_rest(host, manager, service.as_str())?;
    }
    manager.within(crate::systemd_dbus::EXCHANGE_WAIT);
    let stopped: Result<(), CallError> = if socket_state.is_some() { manager.stop(socket.as_str()) } else { manager.stop(service.as_str()) };
    match stopped {
        Ok(()) => {}
        Err(CallError::Manager { name, .. }) if name == crate::systemd_dbus::NO_SUCH_UNIT => {}
        Err(error) => return Err(refusal(error)),
    }
    return wait_until_dropped(manager, [service.as_str(), socket.as_str()].as_slice());
}

/// Waits until the manager reports the unit `unit` at rest or not held,
/// and fails when that takes longer than the retirement wait.
/// Why:
///   A stop call returns when the job is queued,
/// so only the state says the service and its children are gone;
/// a service
///  that is still stopping after the retirement wait may be in the middle of a landing,
/// and stopping its unit again would
///  cut that landing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await waitUntilAtRest(host, manager, unit)
/// ```
fn wait_until_at_rest(host: &ProvisionHost, manager: &mut dyn Manager, unit: &str) -> Result<(), ClientFailure> {
    let deadline: std::time::Instant = std::time::Instant::now() + host.retirement_wait;
    loop {
        manager.within(deadline.saturating_duration_since(std::time::Instant::now()));
        let state: Option<String> = match manager.active_state(unit) {
            Ok(found) => found,
            Err(CallError::TimedOut) => return Err(still_loaded(unit)),
            Err(error) => return Err(refusal(error)),
        };
        if !matches!(&state, Some(text) if !is_at_rest(text.as_str())) {
            return Ok(());
        }
        if std::time::Instant::now() >= deadline {
            return Err(ClientFailure::CouldNotStart(format!(
                "the commit service of this worktree ({unit}) is still running after {:.0} seconds, and it may be in the middle of a landing that stopping it would cut; run the commit again in a moment",
                host.retirement_wait.as_secs_f64()
            )));
        }
        std::thread::sleep(UNLOAD_POLL);
    }
}

/// What:
///  Waits until the manager reports every one of `units` not held at all,
/// and fails when that takes longer than `UNLOAD_WAIT`.
/// Why:
///   A unit the manager unloaded is read from its file the next time it is started;
/// one it still holds is started from
///  the text it read earlier.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await waitUntilDropped(manager, ['<service>', '<socket>'])
/// ```
fn wait_until_dropped(manager: &mut dyn Manager, units: &[&str]) -> Result<(), ClientFailure> {
    let deadline: std::time::Instant = std::time::Instant::now() + UNLOAD_WAIT;
    loop {
        let mut held: Option<&str> = None;
        for unit in units {
            manager.within(deadline.saturating_duration_since(std::time::Instant::now()).max(UNLOAD_POLL));
            match manager.active_state(unit) {
                Ok(None) => {}
                Ok(Some(_state)) => held = Some(unit),
                Err(error) => return Err(refusal(error)),
            }
        }
        let Some(unit) = held else { return Ok(()) };
        if std::time::Instant::now() >= deadline {
            return Err(still_loaded(unit));
        }
        std::thread::sleep(UNLOAD_POLL);
    }
}
