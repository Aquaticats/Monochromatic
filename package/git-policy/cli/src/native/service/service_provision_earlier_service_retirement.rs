//! What:
//!  The retirement of the single service of the earlier releases:
//! the user's own unit directory cleared of the wrapper's earlier units,
//! the earlier service asked to retire and its units stopped,
//! and the wait until the manager reports them stopped.
//! Why:
//!  `service_provision.rs` runs the retirement before it writes the current units;
//! the retirement uses the host and the manager (`service_provision_host_and_manager.rs`) and nothing of the provisioning,
//! so it sits in a file of its own and `service_provision.rs` calls it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reload = await retireEarlierService(host, manager); // true: the manager must reload
//! ```

use crate::service_client_listen_and_ask_to_retire::ask_to_retire;
use crate::service_instance::LEGACY_SOCKET_RELATIVE;
use crate::service_messages::ClientFailure;
use crate::service_provision_host_and_manager::{LEGACY_SERVICE_UNIT_NAME, LEGACY_SOCKET_UNIT_NAME, Manager, ProvisionHost, refusal, runtime_unit_dir};
use crate::service_provision_unit_file_ownership_and_writes::{any_named_anywhere, carries_own_marker, is_own_unit_file, remove_settled};
use crate::systemd_dbus::{CallError, EXCHANGE_WAIT, NO_SUCH_UNIT};
use std::path::PathBuf;

/// What the user's own unit directory holds under the names the wrapper uses.
pub(crate) fn clear_earlier_units(host: &ProvisionHost) -> Result<bool, ClientFailure> {
    let Some(directory) = host.user_unit_dir.as_deref() else { return Ok(false) };
    let mut removed: bool = false;
    for name in [LEGACY_SOCKET_UNIT_NAME, LEGACY_SERVICE_UNIT_NAME] {
        let path: PathBuf = directory.join(name);
        let Ok(text) = std::fs::read_to_string(&path) else {
            if path.exists() {
                return Err(ClientFailure::UnitInTheWay(path.display().to_string()));
            }
            continue;
        };
        if !carries_own_marker(text.as_str()) {
            return Err(ClientFailure::UnitInTheWay(path.display().to_string()));
        }
        remove_settled(path.as_path())?;
        removed = true;
    }
    // The enabling link an earlier release made points at the file just removed.
    let link: PathBuf = directory.join("sockets.target.wants").join(LEGACY_SOCKET_UNIT_NAME);
    if matches!(std::fs::symlink_metadata(&link), Ok(meta) if meta.file_type().is_symlink()) {
        let _removed = std::fs::remove_file(&link);
        removed = true;
    }
    return Ok(removed);
}

/// What:
///  Retires the single service of the earlier releases:
/// asks it to finish its batch and exit,
/// stops its units,
/// and removes
///  its unit files and socket file.
/// Whether the manager must reload.
/// Why:
///   One service for every worktree is what the owner replaced with one per worktree.
/// Its socket and units belong to the
///  runtime directory,
/// which the wrapper owns;
/// a unit file that does not carry the wrapper's marker is someone else's and is
///  left alone.
/// Work it had claimed stays in the work directories and is claimed again by the instance of its worktree;
/// the
///  batches it left open are in its journal,
/// where the instance of their worktree finishes them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await retireEarlierService(host, manager) // true: reload
/// ```
pub(crate) fn retire_earlier_service(host: &ProvisionHost, manager: &mut dyn Manager) -> Result<bool, ClientFailure> {
    let directory: PathBuf = runtime_unit_dir(host.runtime_dir.as_path());
    // The releases that wrote into the user's own directory left their service running just the same.
    let directories: Vec<PathBuf> = std::iter::once(directory.clone()).chain(host.user_unit_dir.clone()).collect();
    let socket_file: PathBuf = host.runtime_dir.join(LEGACY_SOCKET_RELATIVE);
    let both: [&str; 2] = [LEGACY_SOCKET_UNIT_NAME, LEGACY_SERVICE_UNIT_NAME];
    // Only a unit whose definition the manager reads is ours to stop: a file of the same name in the runtime directory
    //  does not make a unit that the user's own directory defines ours.
    let mut units: Vec<&str> = Vec::new();
    for name in both {
        let ours: bool = match effective_unit_file(host, name) {
            Some(file) => is_own_unit_file(file.as_path()),
            None => false,
        };
        if ours {
            units.push(name);
        }
    }
    // Only a file that carries the marker is ours to remove, whichever file defines the unit.
    let mut owned_files: Vec<PathBuf> = Vec::new();
    for name in both {
        let file: PathBuf = directory.join(name);
        if is_own_unit_file(file.as_path()) {
            owned_files.push(file);
        }
    }
    // A socket file with no unit file of that name anywhere is what a release that removed the files but not the service left
    //  behind: the service may still run, so both names are stopped. A file of that name that is someone else's blocks this.
    if units.is_empty() && owned_files.is_empty() && socket_file.exists() && !any_named_anywhere(directories.as_slice(), &both) {
        units = both.to_vec();
    }
    if units.is_empty() && owned_files.is_empty() {
        return Ok(false);
    }
    if !units.is_empty() {
        ask_to_retire(socket_file.as_path());
        // The time spent asking the earlier service to retire is not taken from the stop calls.
        manager.within(EXCHANGE_WAIT);
        // The socket first, so that the manager cannot start the service again for a connection that arrives meanwhile.
        let mut sockets: Vec<&str> = Vec::new();
        let mut services: Vec<&str> = Vec::new();
        for name in &units {
            if *name == LEGACY_SOCKET_UNIT_NAME {
                sockets.push(name);
            } else {
                services.push(name);
            }
        }
        stop_units(manager, sockets.as_slice())?;
        // The manager's stop ends the whole unit, the Git children of a landing in progress with it, and a Git step that cannot run
        //  twice may be one of them. The service is asked to retire, which it handles between batches, and is given the wait to
        //  leave by itself; the stop goes to a service that has left, and a service that is still there when the wait ends fails
        //  the commit with the sentence that names the stop for the person to choose (sol-1 round 2).
        wait_until_stopped(host, manager, services.as_slice())?;
        stop_units(manager, services.as_slice())?;
        wait_until_stopped(host, manager, units.as_slice())?;
    }
    for file in &owned_files {
        remove_settled(file.as_path())?;
    }
    let _socket_removed = std::fs::remove_file(&socket_file);
    return Ok(!owned_files.is_empty());
}

/// Asks the manager to stop each unit;
/// a unit it never loaded has nothing to stop.
fn stop_units(manager: &mut dyn Manager, units: &[&str]) -> Result<(), ClientFailure> {
    for name in units {
        match manager.stop(name) {
            Ok(()) => {}
            Err(CallError::Manager { name: error, .. }) if error == NO_SUCH_UNIT => {}
            Err(error) => return Err(refusal(error)),
        }
    }
    return Ok(());
}

/// What:
///  The file that defines the unit `name` for the user's manager:
/// the one in the user's own unit directory when there is
///  one,
/// else the one in the runtime unit directory.
/// Why:
///   The user's own directory outranks the runtime one in the manager's search path (`systemd.unit(5)`,
/// "Unit File Load
///  Path"),
/// so a runtime file of the same name is shadowed and says nothing about the unit that runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// effectiveUnitFile(host, 'cli-git.service') // the path the manager reads
/// ```
fn effective_unit_file(host: &ProvisionHost, name: &str) -> Option<PathBuf> {
    // The user's own directory first, then the runtime directory.
    if let Some(directory) = host.user_unit_dir.as_ref() {
        let file: PathBuf = directory.join(name);
        if file.exists() {
            return Some(file);
        }
    }
    let runtime_file: PathBuf = runtime_unit_dir(host.runtime_dir.as_path()).join(name);
    if runtime_file.exists() {
        return Some(runtime_file);
    }
    return None;
}

/// What:
///  The failure for an earlier commit service that is still running after the retirement wait.
/// Why:
///  The service may be in the middle of a landing that stopping it would cut,
/// so the person is told to run the commit again or
///  to stop it by hand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stillStopping = (host, unit) => new ClientFailure('could-not-start', `the earlier commit service (${unit}) is still running ...`);
/// ```
fn still_stopping(host: &ProvisionHost, unit: &str) -> ClientFailure {
    return ClientFailure::CouldNotStart(format!(
        "the earlier commit service ({unit}) is still running after {:.1} seconds, and it may be in the middle of a landing that stopping it would cut; run the commit again in a moment, or, when no commit is running, stop it with `systemctl --user stop {LEGACY_SERVICE_UNIT_NAME} {LEGACY_SOCKET_UNIT_NAME}`",
        host.retirement_wait.as_secs_f64()
    ));
}

/// What:
///  Waits until the manager reports each unit inactive,
/// failed or not held,
/// and fails when that takes longer than the host
///  allows.
/// Why:
///   A stop call returns when the job is queued,
/// and a service that was asked to retire finishes the batch it holds,
/// and its Git
///  children,
/// before it exits (the wait comes before the stop of the service,
/// which would cut that batch);
/// an instance started meanwhile would recover and claim work the earlier service is still doing,
/// so the commit
///  fails instead,
/// naming what is still stopping.
/// Its files stay,
/// so the next commit finds the same state and tries again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await waitUntilStopped(host, manager, ['cli-git.socket', 'cli-git.service'])
/// ```
fn wait_until_stopped(host: &ProvisionHost, manager: &mut dyn Manager, units: &[&str]) -> Result<(), ClientFailure> {
    let deadline: std::time::Instant = std::time::Instant::now() + host.retirement_wait;
    loop {
        let mut running: Option<&str> = None;
        for name in units {
            // A question to the manager may not outlast the wait: it is given what is left of it, and a question that runs out
            //  of that is a service that is still stopping, not a manager that is not there.
            manager.within(deadline.saturating_duration_since(std::time::Instant::now()));
            let state: Option<String> = match manager.active_state(name) {
                Ok(found) => found,
                Err(CallError::TimedOut) => return Err(still_stopping(host, name)),
                Err(error) => return Err(refusal(error)),
            };
            if matches!(&state, Some(text) if !matches!(text.as_str(), "inactive" | "failed")) {
                running = Some(name);
            }
        }
        let Some(unit) = running else { return Ok(()) };
        if std::time::Instant::now() >= deadline {
            return Err(still_stopping(host, unit));
        }
        std::thread::sleep(RETIREMENT_POLL);
    }
}

/// How often the manager is asked whether the earlier service has stopped.
const RETIREMENT_POLL: std::time::Duration = std::time::Duration::from_millis(50);
