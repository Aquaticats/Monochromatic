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
///  A start now would read the text the manager loaded earlier instead of the files this release wrote,
/// and that is the
///  one thing this repair is for;
/// the person is told which unit and what to try.
/// What that text runs is this release's own
///  program while it names the program link (every release since `0.6.1-alpha.14`),
/// and the release that wrote it when the
///  text is older than the link,
/// which the client's own repair for a service of another release answers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stillLoaded(unit) // could-not-start with the unit's name
/// ```
fn still_loaded(unit: &str) -> ClientFailure {
    return ClientFailure::CouldNotStart(format!(
        "the service manager still holds this worktree's unit ({unit}) {:.0} seconds after it was stopped, so a start now would read the text it loaded earlier rather than the files; run the commit again in a moment",
        UNLOAD_WAIT.as_secs_f64()
    ));
}

/// What:
///  What a call to `unload_instance` came to.
/// Why:
///   The caller may write the record of what the manager read only when it was watched dropping this worktree's units;
/// a question the manager could not answer leaves them as they were,
/// and a record written then would tell every later
///  commit the manager read texts it never read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Unloaded = { dropped: true } | { dropped: false };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Unloaded {
    /// The manager was asked and holds neither of this worktree's units,
    /// so its next start reads the files as they are now.
    Dropped,
    /// The manager could not be asked,
    /// so nothing was dropped and nothing about what it holds is known.
    NotAsked,
}

/// What:
///  What a wait for this worktree's own service to leave came to.
/// Why:
///   A service that is still running at the retirement wait is landing a batch,
/// and a commit that would land through it must not
///  fail because the text it reads cannot be dropped yet;
/// the three outcomes are as different as a service this repair can
///  finish,
/// one it may not,
/// and a manager that could not be asked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ServiceLeft = { gone: true } | { stillRunning: true } | { notAsked: CallError };
/// ```
enum ServiceLeft {
    /// The manager says the service runs nothing,
    /// so the unit may be stopped and dropped.
    Gone,
    /// It was still running when the retirement wait ran out,
    /// so it is landing a batch or will not leave.
    StillRunning,
    /// The manager could not be asked what the service does.
    NotAsked(CallError),
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
pub(crate) fn unload_instance(host: &ProvisionHost, manager: &mut dyn Manager) -> Result<Unloaded, ClientFailure> {
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
            return Ok(Unloaded::NotAsked);
        }
    };
    let socket_state: Option<String> = match manager.active_state(socket.as_str()) {
        Ok(found) => found,
        Err(error) => {
            crate::service_warn!(
                "the service manager could not say whether it holds this worktree's socket unit ({socket}): {}; the commit goes on without unloading it, and the next commit asks again",
                describe(&error)
            );
            return Ok(Unloaded::NotAsked);
        }
    };
    if service_state.is_none() && socket_state.is_none() {
        return Ok(Unloaded::Dropped);
    }
    // A service that runs is asked to leave by itself before anything is stopped, so the batch it holds is finished.
    if !matches!(&service_state, Some(state) if is_at_rest(state.as_str())) {
        ask_to_retire(socket_path(host.runtime_dir.as_path(), instance).as_path());
        match wait_until_at_rest(host, manager, service.as_str()) {
            ServiceLeft::Gone => {}
            // What: A service still there at the wait is landing a batch, so nothing of it is stopped and the commit goes on
            //       through it.
            // Why:  A landing may hold the index for longer than this wait (`wait_ceilings.rs`, `LANDING_LOCK_CEILING`), and the
            //       commit does not need the text to be fresh to land: the service is alive and answers it, the unit is left as
            //       the manager holds it, and the record stays unwritten so the next commit drops the unit once it is at rest.
            //       Stopping it here would cut the landing, which is the one thing this repair must not do
            //       (`stop_of_the_service_noticed_by_its_children.rs`).
            ServiceLeft::StillRunning => {
                crate::service_warn!(
                    "the commit service of this worktree ({service}) is still running {:.0} seconds after it was asked to retire, which is a batch landing, so this commit goes on through it and the next commit drops the unit once it is at rest",
                    host.retirement_wait.as_secs_f64()
                );
                return Ok(Unloaded::NotAsked);
            }
            // What: A manager that cannot say what the service does leaves the units as they are and the commit going.
            // Why:  The answer only decides whether they have to be dropped; nothing can be dropped on a guess, the commit does
            //       not need the repair to land, and the record is not written, so the next commit asks again.
            ServiceLeft::NotAsked(error) => {
                crate::service_warn!(
                    "the service manager could not say whether the commit service of this worktree ({service}) is still running: {}; the units stay as they are and the commit goes on, and the next commit asks again",
                    describe(&error)
                );
                return Ok(Unloaded::NotAsked);
            }
        }
    }
    manager.within(crate::systemd_dbus::EXCHANGE_WAIT);
    let stopped: Result<(), CallError> = if socket_state.is_some() { manager.stop(socket.as_str()) } else { manager.stop(service.as_str()) };
    match stopped {
        Ok(()) => {}
        Err(CallError::Manager { name, .. }) if name == crate::systemd_dbus::NO_SUCH_UNIT => {}
        // What: A stop the manager did not take leaves the units as they are and the commit going.
        // Why:  The stop is part of a repair the next commit repeats; failing here would end a commit over a call that changed
        //       nothing, and a line that names a missing user service manager would be false of a manager that just refused one
        //       request.
        Err(error) => {
            crate::service_warn!(
                "the service manager did not take the stop of this worktree's own unit ({socket}): {}; the units stay as they are and the commit goes on, and the next commit asks again",
                describe(&error)
            );
            return Ok(Unloaded::NotAsked);
        }
    }
    return wait_until_dropped(manager, [service.as_str(), socket.as_str()].as_slice());
}

/// Waits until the manager reports the unit `unit` at rest or not held,
/// and answers what it found when the retirement wait runs out.
/// Why:
///   A stop call returns when the job is queued,
/// so only the state says the service and its children are gone;
/// a service
///  that is still there at the retirement wait may be in the middle of a landing,
/// and stopping its unit would cut that landing,
/// so
///  the caller is told that rather than a failure that ends the commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await waitUntilAtRest(host, manager, unit)
/// ```
fn wait_until_at_rest(host: &ProvisionHost, manager: &mut dyn Manager, unit: &str) -> ServiceLeft {
    let deadline: std::time::Instant = std::time::Instant::now() + host.retirement_wait;
    loop {
        manager.within(deadline.saturating_duration_since(std::time::Instant::now()));
        let state: Option<String> = match manager.active_state(unit) {
            Ok(found) => found,
            Err(error) => return ServiceLeft::NotAsked(error),
        };
        if !matches!(&state, Some(text) if !is_at_rest(text.as_str())) {
            return ServiceLeft::Gone;
        }
        if std::time::Instant::now() >= deadline {
            return ServiceLeft::StillRunning;
        }
        std::thread::sleep(UNLOAD_POLL);
    }
}

/// What:
///  Waits until the manager reports every one of `units` not held at all,
/// and answers whether it dropped them.
/// Why:
///   A unit the manager unloaded is read from its file the next time it is started;
/// one it still holds is started from
///  the text it read earlier.
/// The one failure that ends the commit is a manager that answered every question and still holds the
///  unit after `UNLOAD_WAIT`,
/// because a start then reads the text it loaded rather than the files;
/// a question it cannot answer
///  leaves the units as they are and the commit going.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await waitUntilDropped(manager, ['<service>', '<socket>'])
/// ```
fn wait_until_dropped(manager: &mut dyn Manager, units: &[&str]) -> Result<Unloaded, ClientFailure> {
    let deadline: std::time::Instant = std::time::Instant::now() + UNLOAD_WAIT;
    loop {
        let mut held: Option<&str> = None;
        for unit in units {
            manager.within(deadline.saturating_duration_since(std::time::Instant::now()).max(UNLOAD_POLL));
            match manager.active_state(unit) {
                Ok(None) => {}
                Ok(Some(_state)) => held = Some(unit),
                // What: A question the manager cannot answer leaves the units as they are and the commit going.
                // Why:  It answered the stop and may still be dropping them; the record is not written, so the next commit asks
                //       again, and a line that says this account has no user service manager would be false of a manager that
                //       answered a moment before.
                Err(error) => {
                    crate::service_warn!(
                        "the service manager could not say whether it still holds this worktree's unit ({unit}): {}; the units stay as they are and the commit goes on, and the next commit asks again",
                        describe(&error)
                    );
                    return Ok(Unloaded::NotAsked);
                }
            }
        }
        let Some(unit) = held else { return Ok(Unloaded::Dropped) };
        if std::time::Instant::now() >= deadline {
            return Err(still_loaded(unit));
        }
        std::thread::sleep(UNLOAD_POLL);
    }
}

/// Controls of what the unload does when the manager,
/// or the service,
/// cannot give it what it asks for stay out of the
/// release executable.
#[cfg(test)]
#[path = "service_provision_instance_unloaded_tests.rs"]
mod tests;
