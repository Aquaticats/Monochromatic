//! What:
//!  The wrapper makes socket activation for a worktree's commit service exist the first time a commit needs it: it writes
//!  the two template unit files into the user's runtime unit directory, asks the user manager (through the session bus) to
//!  reload and to start the worktree's socket instance, and returns. The manager, not the wrapper, starts the service when
//!  the first connection arrives.
//! Why:
//!  There is no install step to remember (`doc/planning/cli-git-resident-service.md`, "Provisioning"): like Bazel's
//!  client, the wrapper sets up what it needs and tells nobody. One service runs per worktree, so the units are templates
//!  (`cli-git@.socket`, `cli-git@.service`) and each worktree's instance is named by a hash of its own Git directory
//!  (`service_instance.rs`). The units live under `$XDG_RUNTIME_DIR/systemd/user`, which disappears at logout, so nothing is
//!  left behind in the home directory and the next first use provisions again. Every step is idempotent (a file is written,
//!  by temporary file and rename, only when its content differs), so several commits racing to provision agree without any
//!  lock. Every commit that goes to the service compares the two installed templates with what this wrapper would write
//!  (two small reads) before it touches the bus, so an upgrade or a downgrade, which moves the service program the template
//!  names, is repaired by the first commit that follows it: templates rewritten, manager reloaded, failed state cleared,
//!  the worktree's socket started. The single service of the earlier releases (`cli-git.socket` and `cli-git.service`, one
//!  socket for every worktree) is asked to retire and stopped, and its units and socket file are removed, by the first
//!  commit of the new release; its open batches are finished by the instance of the worktree they belong to, and the work
//!  it had claimed is claimed again by the next drain. Units this wrapper's earliest releases wrote into
//!  `~/.config/systemd/user` outrank the runtime ones in the unit search path, so they are recognized by their content and
//!  removed; a unit of the same name that something else wrote is never touched, and the commit stops with a message naming
//!  the file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await provision({ runtimeDir, configHome, busPath, serviceBinary, git, instance }) // writes templates, reloads, starts the socket
//! ```

use crate::service_messages::{ClientFailure, Platform};
use crate::service_client::ask_to_retire;
use crate::service_instance::{LEGACY_SOCKET_RELATIVE, service_unit_instance, socket_path, socket_unit_instance};
use crate::systemd_dbus::{Bus, CallError, EXCHANGE_WAIT, NO_SUCH_UNIT};
use std::path::{Path, PathBuf};

/// The socket template's text, with the marker on its first line.
pub const SOCKET_UNIT: &str = include_str!("service_units/cli-git@.socket");

/// The service template's text, with `@SERVICE_BINARY@` and `@GIT@` still to fill in.
pub const SERVICE_UNIT: &str = include_str!("service_units/cli-git@.service");

/// File name of the socket template.
pub const SOCKET_UNIT_NAME: &str = "cli-git@.socket";

/// File name of the service template.
pub const SERVICE_UNIT_NAME: &str = "cli-git@.service";

/// File name of the socket unit of the single service every worktree shared until one service per worktree.
pub const LEGACY_SOCKET_UNIT_NAME: &str = "cli-git.socket";

/// File name of the service unit of the single service every worktree shared until one service per worktree.
pub const LEGACY_SERVICE_UNIT_NAME: &str = "cli-git.service";

/// Name of the service program, next to the wrapper.
pub const SERVICE_PROGRAM: &str = "cli-git-service";

/// The words that mark a unit file as written by this wrapper: the marker its units carry now, and the descriptions the
///  units of earlier releases carried.
pub const OWN_MARKERS: [&str; 3] =
    ["(cli-git-unit)", "Description=Commit service socket for the git wrapper", "Description=Commit service for the git wrapper"];

const BINARY_PLACEHOLDER: &str = "@SERVICE_BINARY@";
const GIT_PLACEHOLDER: &str = "@GIT@";

/// What provisioning needs to know about the machine.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvisionHost {
    /// The user's runtime directory, which holds the units and the service's socket.
    pub runtime_dir: PathBuf,
    /// The directory a user's own unit files live in (`~/.config/systemd/user`), where earlier releases wrote theirs.
    pub user_unit_dir: Option<PathBuf>,
    /// The session bus socket.
    pub bus: PathBuf,
    /// The service program.
    pub service_binary: PathBuf,
    /// The real Git the service runs.
    pub git: PathBuf,
    /// The instance of the worktree the commit is made in (`service_instance::instance_of`).
    pub instance: String,
    /// How long retiring the earlier single service may take before the commit fails instead of starting an instance beside it.
    pub retirement_wait: std::time::Duration,
}

/// How long the earlier single service may take to stop; it finishes the batch it holds first. The wait and every question to
///  the manager in it share this one budget, set apart from the exchange's own, so a slow stop is reported as that and not as
///  an unreachable manager.
pub const RETIREMENT_WAIT: std::time::Duration = std::time::Duration::from_secs(10);

/// What the manager is asked to do, so a control can record it instead of reaching a manager.
pub trait Manager {
    /// Gives the calls that follow `wait` from now in all, so that each phase of provisioning has a time of its own.
    fn within(&mut self, wait: std::time::Duration);
    /// Reloads the unit files.
    fn reload(&mut self) -> Result<(), CallError>;
    /// Clears a unit's failed state and start-rate counter.
    fn reset_failed(&mut self, unit: &str) -> Result<(), CallError>;
    /// Starts a unit.
    fn start(&mut self, unit: &str) -> Result<(), CallError>;
    /// Stops a unit.
    fn stop(&mut self, unit: &str) -> Result<(), CallError>;
    /// The unit's state as the manager reports it (`active`, `deactivating`, `inactive`, `failed`), or nothing when the
    ///  manager holds no such unit.
    fn active_state(&mut self, unit: &str) -> Result<Option<String>, CallError>;
}

impl Manager for Bus {
    fn within(&mut self, wait: std::time::Duration) {
        Bus::within(self, wait);
    }

    fn reload(&mut self) -> Result<(), CallError> {
        return Bus::reload(self);
    }

    fn reset_failed(&mut self, unit: &str) -> Result<(), CallError> {
        return Bus::reset_failed(self, unit);
    }

    fn start(&mut self, unit: &str) -> Result<(), CallError> {
        return Bus::start_unit(self, unit);
    }

    fn stop(&mut self, unit: &str) -> Result<(), CallError> {
        return Bus::stop_unit(self, unit);
    }

    fn active_state(&mut self, unit: &str) -> Result<Option<String>, CallError> {
        return Bus::active_state(self, unit);
    }
}

/// The platform whose remedy applies when no user service manager answers.
pub fn platform_without_manager() -> Platform {
    if cfg!(windows) {
        return Platform::Windows;
    }
    if cfg!(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd", target_os = "dragonfly")) {
        return Platform::Bsd;
    }
    if cfg!(target_os = "linux") {
        return Platform::LinuxWithoutUserServices;
    }
    return Platform::Other;
}

/// The two unit files for a host, as written.
pub fn unit_texts(host: &ProvisionHost) -> [(&'static str, String); 2] {
    let service: String = SERVICE_UNIT
        .replace(BINARY_PLACEHOLDER, host.service_binary.to_string_lossy().as_ref())
        .replace(GIT_PLACEHOLDER, host.git.to_string_lossy().as_ref());
    return [(SOCKET_UNIT_NAME, SOCKET_UNIT.to_string()), (SERVICE_UNIT_NAME, service)];
}

/// The directory the runtime units are written to.
pub fn runtime_unit_dir(runtime_dir: &Path) -> PathBuf {
    return runtime_dir.join("systemd").join("user");
}

/// What:
///  Whether the installed templates are the ones this wrapper would write, the worktree's socket is there, and nothing of
///  the single service of the earlier releases is left.
/// Why:
///   The socket belongs to the manager and outlives every upgrade, so its presence says nothing about which program the
///  service template names; the text does. A few small reads and two `stat` calls decide, and the bus is not touched when
///  they agree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// inPlace(host) // false: write, reload, start
/// ```
pub fn in_place(host: &ProvisionHost) -> bool {
    let directory: PathBuf = runtime_unit_dir(host.runtime_dir.as_path());
    let current = |(name, text): (&str, String)| -> bool {
        return std::fs::read_to_string(directory.join(name)).is_ok_and(|installed| { return installed == text; });
    };
    // A file of an earlier name that something else wrote is not the wrapper's to retire, and does not keep it from being in place.
    let earlier_left: bool = [LEGACY_SOCKET_UNIT_NAME, LEGACY_SERVICE_UNIT_NAME].iter().any(|name| {
        return std::iter::once(&directory).chain(host.user_unit_dir.iter()).any(|found| { return is_own_unit_file(found.join(name).as_path()); });
    }) || host.runtime_dir.join(LEGACY_SOCKET_RELATIVE).exists();
    return unit_texts(host).into_iter().all(current)
        && socket_path(host.runtime_dir.as_path(), host.instance.as_str()).exists()
        && !earlier_left;
}

static TEMPORARY_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Writes `text` to `path` through a temporary file and a rename, so a reader sees the old file or the new one whole.
fn write_atomically(path: &Path, text: &str) -> std::io::Result<()> {
    let number: u64 = TEMPORARY_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let temporary: PathBuf = path.with_extension(format!("tmp-{}-{number}", std::process::id()));
    std::fs::write(&temporary, text.as_bytes())?;
    let renamed: std::io::Result<()> = std::fs::rename(&temporary, path);
    if renamed.is_err() {
        let _removed = std::fs::remove_file(&temporary);
    }
    return renamed;
}

/// What the user's own unit directory holds under the names the wrapper uses.
fn clear_earlier_units(host: &ProvisionHost) -> Result<bool, ClientFailure> {
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
        if !OWN_MARKERS.iter().any(|marker| { return text.contains(marker); }) {
            return Err(ClientFailure::UnitInTheWay(path.display().to_string()));
        }
        remove_settled(path.as_path())?;
        removed = true;
    }
    // The enabling link an earlier release made points at the file just removed.
    let link: PathBuf = directory.join("sockets.target.wants").join(LEGACY_SOCKET_UNIT_NAME);
    if std::fs::symlink_metadata(&link).is_ok_and(|meta| { return meta.file_type().is_symlink(); }) {
        let _removed = std::fs::remove_file(&link);
        removed = true;
    }
    return Ok(removed);
}

/// What:
///  Retires the single service of the earlier releases: asks it to finish its batch and exit, stops its units, and removes
///  its unit files and socket file. Whether the manager must reload.
/// Why:
///   One service for every worktree is what the owner replaced with one per worktree. Its socket and units belong to the
///  runtime directory, which the wrapper owns; a unit file that does not carry the wrapper's marker is someone else's and is
///  left alone. Work it had claimed stays in the work directories and is claimed again by the instance of its worktree; the
///  batches it left open are in its journal, where the instance of their worktree finishes them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await retireEarlierService(host, manager) // true: reload
/// ```
fn retire_earlier_service(host: &ProvisionHost, manager: &mut dyn Manager) -> Result<bool, ClientFailure> {
    let directory: PathBuf = runtime_unit_dir(host.runtime_dir.as_path());
    // The releases that wrote into the user's own directory left their service running just the same.
    let directories: Vec<PathBuf> = std::iter::once(directory.clone()).chain(host.user_unit_dir.clone()).collect();
    let socket_file: PathBuf = host.runtime_dir.join(LEGACY_SOCKET_RELATIVE);
    let named_anywhere = |name: &str| -> bool { return directories.iter().any(|found| { return found.join(name).exists(); }); };
    let both: [&str; 2] = [LEGACY_SOCKET_UNIT_NAME, LEGACY_SERVICE_UNIT_NAME];
    // Only a unit whose definition the manager reads is ours to stop: a file of the same name in the runtime directory
    //  does not make a unit that the user's own directory defines ours.
    let mut units: Vec<&str> = both.into_iter().filter(|name| { return effective_unit_file(host, name).is_some_and(|file| { return is_own_unit_file(file.as_path()); }); }).collect();
    // Only a file that carries the marker is ours to remove, whichever file defines the unit.
    let owned_files: Vec<PathBuf> = both.into_iter().map(|name| { return directory.join(name); }).filter(|file| { return is_own_unit_file(file.as_path()); }).collect();
    // A socket file with no unit file of that name anywhere is what a release that removed the files but not the service left
    //  behind: the service may still run, so both names are stopped. A file of that name that is someone else's blocks this.
    if units.is_empty() && owned_files.is_empty() && socket_file.exists() && !both.iter().any(|name| { return named_anywhere(name); }) {
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
        for name in &units {
            match manager.stop(name) {
                Ok(()) => {}
                // A unit the manager never loaded has nothing to stop.
                Err(CallError::Manager { name: error, .. }) if error == NO_SUCH_UNIT => {}
                Err(error) => return Err(refusal(error)),
            }
        }
        wait_until_stopped(host, manager, units.as_slice())?;
    }
    for file in &owned_files {
        remove_settled(file.as_path())?;
    }
    let _socket_removed = std::fs::remove_file(&socket_file);
    return Ok(!owned_files.is_empty());
}

/// Whether the unit file at `path` carries a marker of this wrapper's units.
fn is_own_unit_file(path: &Path) -> bool {
    return std::fs::read_to_string(path).is_ok_and(|text| { return OWN_MARKERS.iter().any(|marker| { return text.contains(marker); }); });
}

/// What:
///  The file that defines the unit `name` for the user's manager: the one in the user's own unit directory when there is
///  one, else the one in the runtime unit directory.
/// Why:
///   The user's own directory outranks the runtime one in the manager's search path (`systemd.unit(5)`, "Unit File Load
///  Path"), so a runtime file of the same name is shadowed and says nothing about the unit that runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// effectiveUnitFile(host, 'cli-git.service') // the path the manager reads
/// ```
fn effective_unit_file(host: &ProvisionHost, name: &str) -> Option<PathBuf> {
    return host
        .user_unit_dir
        .iter()
        .map(|directory| { return directory.join(name); })
        .chain(std::iter::once(runtime_unit_dir(host.runtime_dir.as_path()).join(name)))
        .find(|file| { return file.exists(); });
}

/// What:
///  Removes a unit file whose ownership was just checked; a file that is gone by now is a removal someone else finished.
/// Why:
///   Several commits provision at once without any lock, so two of them can read the same owned file and both remove it:
///  the second one's removal finds nothing, and the cleanup it wanted is done.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(path, { force: true }) // other failures throw
/// ```
fn remove_settled(path: &Path) -> Result<(), ClientFailure> {
    return match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(ClientFailure::CouldNotStart(format!("{} cannot be removed: {error}", path.display()))),
    };
}

/// What:
///  Waits until the manager reports each unit inactive, failed or not held, and fails when that takes longer than the host
///  allows.
/// Why:
///   A stop call returns when the job is queued. The service finishes the batch it holds, and its Git children, before it
///  exits; an instance started meanwhile would recover and claim work the earlier service is still doing, so the commit
///  fails instead, naming what is still stopping. Its files stay, so the next commit finds the same state and tries again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await waitUntilStopped(host, manager, ['cli-git.socket', 'cli-git.service'])
/// ```
fn wait_until_stopped(host: &ProvisionHost, manager: &mut dyn Manager, units: &[&str]) -> Result<(), ClientFailure> {
    let deadline: std::time::Instant = std::time::Instant::now() + host.retirement_wait;
    let still_stopping = |unit: &str| -> ClientFailure {
        return ClientFailure::CouldNotStart(format!(
            "the earlier commit service ({unit}) is still stopping after {:.1} seconds; run the commit again in a moment, or stop it with `systemctl --user stop {LEGACY_SERVICE_UNIT_NAME} {LEGACY_SOCKET_UNIT_NAME}`",
            host.retirement_wait.as_secs_f64()
        ));
    };
    loop {
        let mut running: Option<&str> = None;
        for name in units {
            // A question to the manager may not outlast the wait: it is given what is left of it, and a question that runs out
            //  of that is a service that is still stopping, not a manager that is not there.
            manager.within(deadline.saturating_duration_since(std::time::Instant::now()));
            let state: Option<String> = match manager.active_state(name) {
                Ok(found) => found,
                Err(CallError::TimedOut) => return Err(still_stopping(name)),
                Err(error) => return Err(refusal(error)),
            };
            if state.is_some_and(|text| { return !matches!(text.as_str(), "inactive" | "failed"); }) {
                running = Some(name);
            }
        }
        let Some(unit) = running else { return Ok(()) };
        if std::time::Instant::now() >= deadline {
            return Err(still_stopping(unit));
        }
        std::thread::sleep(RETIREMENT_POLL);
    }
}

/// How often the manager is asked whether the earlier service has stopped.
const RETIREMENT_POLL: std::time::Duration = std::time::Duration::from_millis(50);

/// What a refused call is to the person: the manager's own words, or a plain statement; its error name is internal vocabulary.
fn refusal(error: CallError) -> ClientFailure {
    return match error {
        CallError::Manager { text, .. } if text.is_empty() => ClientFailure::CouldNotStart(String::from("the service manager refused the request")),
        CallError::Manager { text, .. } => ClientFailure::CouldNotStart(format!("the service manager refused the request ({text})")),
        CallError::Connection(_) | CallError::Authentication(_) | CallError::Malformed(_) | CallError::TimedOut => {
            ClientFailure::NoActivation(platform_without_manager())
        }
    };
}

/// What:
///  Makes socket activation exist: the earlier single service retired, current templates written where they differ, the
///  manager reloaded when a unit it may hold changed or went away, and the worktree's socket instance started.
/// Why:
///   Idempotent by construction: a second call with nothing to change only starts the socket, which is a no-op when it is
///  already listening; two calls at once write identical content through renames and both succeed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await provisionWith(host, manager)
/// ```
pub fn provision_with(host: &ProvisionHost, manager: &mut dyn Manager) -> Result<(), ClientFailure> {
    // A reload makes the manager re-read every unit and interrupts its other clients for a while, so it is asked for only
    //  when a unit it may already hold changed or went away. A file that is new is found by the manager when the unit is
    //  first started.
    let mut reload: bool = retire_earlier_service(host, manager)?;
    reload |= clear_earlier_units(host)?;
    let directory: PathBuf = runtime_unit_dir(host.runtime_dir.as_path());
    std::fs::create_dir_all(&directory).map_err(|error| { return ClientFailure::CouldNotStart(format!("{} cannot be created: {error}", directory.display())); })?;
    // Whether the manager may hold state for these units: a unit file was already there, so a start may have failed under it.
    let mut known: bool = false;
    let mut written: Vec<PathBuf> = Vec::new();
    for (name, text) in unit_texts(host) {
        let path: PathBuf = directory.join(name);
        let existed: bool = path.exists();
        known |= existed;
        if std::fs::read_to_string(&path).is_ok_and(|current| { return current == text; }) {
            continue;
        }
        write_atomically(&path, text.as_str())
            .map_err(|error| { return ClientFailure::CouldNotStart(format!("{} cannot be written: {error}", path.display())); })?;
        written.push(path);
        reload |= existed;
    }
    // Whatever the retirement of the earlier service took, the reload and the start have the time of a whole exchange.
    manager.within(EXCHANGE_WAIT);
    let asked: Result<(), CallError> = ask_manager(manager, reload, known, host.instance.as_str());
    if let Err(error) = asked {
        // A file the manager never heard about must not look installed to the next commit, which would then skip the
        //  reload that this one did not complete.
        for path in &written {
            let _removed = std::fs::remove_file(path);
        }
        return Err(refusal(error));
    }
    return Ok(());
}

/// The calls provisioning makes: a reload when a held unit changed, a reset of failed state when a unit may have failed
///  under the manager (the reset is best effort: a unit the manager does not hold has nothing to reset), and the start.
fn ask_manager(manager: &mut dyn Manager, reload: bool, known: bool, instance: &str) -> Result<(), CallError> {
    if reload {
        manager.reload()?;
    }
    if known {
        // A start that failed under an old unit leaves the service failed and its start counter full, and the socket
        //  failed with it; the new unit would be refused as "repeated too quickly" until the interval passed.
        for unit in [service_unit_instance(instance), socket_unit_instance(instance)] {
            let _reset = manager.reset_failed(unit.as_str());
        }
    }
    return manager.start(socket_unit_instance(instance).as_str());
}

/// Provisions through the session bus, unless the units and the socket are already in place.
pub fn provision(host: &ProvisionHost) -> Result<(), ClientFailure> {
    if in_place(host) {
        return Ok(());
    }
    let mut bus: Bus = Bus::connect(host.bus.as_path(), EXCHANGE_WAIT).map_err(|_| { return ClientFailure::NoActivation(platform_without_manager()); })?;
    return provision_with(host, &mut bus);
}

/// Provisioning controls stay out of the release executable.
#[cfg(test)]
#[path = "service_provision_tests.rs"]
mod tests;
