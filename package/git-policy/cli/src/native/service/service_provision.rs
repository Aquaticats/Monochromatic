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
//!  (two small reads) before it touches the bus, so an upgrade or a downgrade, which moves the executable the template
//!  names, is repaired by the first commit that follows it: templates rewritten, manager reloaded, failed state cleared,
//!  the worktree's socket started. The single service of the earlier releases (`cli-git.socket` and `cli-git.service`, one
//!  socket for every worktree) is asked to retire and stopped, and its units and socket file are removed, by the first
//!  commit of the new release; its open batches are finished by the instance of the worktree they belong to, and the work
//!  it had claimed is claimed again by the next drain. Units this wrapper's earliest releases wrote into
//!  `~/.config/systemd/user` outrank the runtime ones in the unit search path, so they are recognized by their content and
//!  removed; a unit of the same name that something else wrote is never touched, and the commit stops with a message naming
//!  the file.
//!  The socket that the earlier release set up for the same worktree (`cli-git@<instance>.socket`, the same instance text) is
//!  stopped after this release's socket has started, so that the earlier helper cannot be started by a connection and the
//!  batches it left open can be finished (`earlier_service_journal_adopted.rs`); see `stop_earlier_instance_socket`. The stop
//!  never fails a commit. A helper of the earlier release that is running is not stopped and its socket stays; the next commit
//!  looks again. The templates the earlier release wrote are shared by every worktree's instance and are left alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await provision({ runtimeDir, configHome, busPath, serviceBinary, git, instance }) // writes templates, reloads, starts the socket
//! ```

pub use crate::service_provision_unit_file_ownership_and_writes::OWN_MARKERS;
use crate::service_provision_unit_file_ownership_and_writes::{any_named_anywhere, carries_own_marker, is_own_unit_file, own_unit_file_in_runtime_or_user_directory, remove_settled, write_atomically};
use crate::service_messages::{ClientFailure, Platform};
use crate::service_client_listen_and_ask_to_retire::ask_to_retire;
use crate::earlier_unix_sockets_that_listen::earlier_socket_of_instance_listens;
use crate::service_instance::{LEGACY_SOCKET_RELATIVE, earlier_service_unit_instance, earlier_socket_path, earlier_socket_unit_instance, service_unit_instance, socket_path, socket_unit_instance};
use crate::systemd_dbus::{Bus, CallError, EXCHANGE_WAIT, NO_SUCH_UNIT};
use std::path::{Path, PathBuf};

/// The socket template's text, with the marker on its first line.
pub const SOCKET_UNIT: &str = include_str!("service_units/monochromatic-git-policy-cli@.socket");

/// The service template's text, with `@SERVICE_BINARY@` and `@GIT@` still to fill in.
pub const SERVICE_UNIT: &str = include_str!("service_units/monochromatic-git-policy-cli@.service");

/// File name of the socket template.
pub const SOCKET_UNIT_NAME: &str = "monochromatic-git-policy-cli@.socket";

/// File name of the service template.
pub const SERVICE_UNIT_NAME: &str = "monochromatic-git-policy-cli@.service";

/// File name of the socket unit of the single service every worktree shared until one service per worktree.
pub const LEGACY_SOCKET_UNIT_NAME: &str = "cli-git.socket";

/// File name of the service unit of the single service every worktree shared until one service per worktree.
pub const LEGACY_SERVICE_UNIT_NAME: &str = "cli-git.service";

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
    /// The program the service unit starts: the wrapper's own executable, which runs the service when the words after it ask
    ///  for that (`service_main.rs`).
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
    /// Stops a unit and none of the units that require it.
    fn stop_alone(&mut self, unit: &str) -> Result<(), CallError>;
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

    fn stop_alone(&mut self, unit: &str) -> Result<(), CallError> {
        return Bus::stop_unit_alone(self, unit);
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
///  the single service of the earlier releases is left, and no socket of the earlier release for this worktree listens.
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
    return in_place_with(host, Path::new("/proc"));
}

/// `in_place` with the process table to read the listening sockets from.
pub fn in_place_with(host: &ProvisionHost, proc_root: &Path) -> bool {
    let directory: PathBuf = runtime_unit_dir(host.runtime_dir.as_path());
    // Every unit file holds exactly the text the wrapper would write.
    let mut all_current: bool = true;
    for (name, text) in unit_texts(host) {
        if !matches!(std::fs::read_to_string(directory.join(name)), Ok(installed) if installed == text) {
            all_current = false;
            break;
        }
    }
    // A file of an earlier name that something else wrote is not the wrapper's to retire, and does not keep it from being in place.
    let mut earlier_left: bool = false;
    for name in [LEGACY_SOCKET_UNIT_NAME, LEGACY_SERVICE_UNIT_NAME] {
        if own_unit_file_in_runtime_or_user_directory(directory.as_path(), host.user_unit_dir.as_deref(), name) {
            earlier_left = true;
            break;
        }
    }
    earlier_left = earlier_left || host.runtime_dir.join(LEGACY_SOCKET_RELATIVE).exists();
    if !all_current || earlier_left || !socket_path(host.runtime_dir.as_path(), host.instance.as_str()).exists() {
        return false;
    }
    // The earlier release's socket of this worktree: a listener there starts the earlier helper on a connection, so the stop
    //  has to be tried (again). The socket's file stays after the unit stops and says nothing; the table of sockets is read
    //  only when that file is there, which a worktree the earlier release never served never has.
    let earlier_file: PathBuf = earlier_socket_path(host.runtime_dir.as_path(), host.instance.as_str());
    return !(earlier_file.exists() && earlier_socket_of_instance_listens(proc_root, host.instance.as_str()));
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

/// Asks the manager to stop each unit; a unit it never loaded has nothing to stop.
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
///  The service may be in the middle of a landing that stopping it would cut, so the person is told to run the commit again or
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
///  Waits until the manager reports each unit inactive, failed or not held, and fails when that takes longer than the host
///  allows.
/// Why:
///   A stop call returns when the job is queued, and a service that was asked to retire finishes the batch it holds, and its Git
///  children, before it exits (the wait comes before the stop of the service, which would cut that batch); an instance started meanwhile would recover and claim work the earlier service is still doing, so the commit
///  fails instead, naming what is still stopping. Its files stay, so the next commit finds the same state and tries again.
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
    if let Err(error) = std::fs::create_dir_all(&directory) {
        return Err(ClientFailure::CouldNotStart(format!("{} cannot be created: {error}", directory.display())));
    }
    // Whether the manager may hold state for these units: a unit file was already there, so a start may have failed under it.
    let mut known: bool = false;
    let mut written: Vec<PathBuf> = Vec::new();
    for (name, text) in unit_texts(host) {
        let path: PathBuf = directory.join(name);
        let existed: bool = path.exists();
        known |= existed;
        if matches!(std::fs::read_to_string(&path), Ok(current) if current == text) {
            continue;
        }
        if let Err(error) = write_atomically(&path, text.as_str()) {
            return Err(ClientFailure::CouldNotStart(format!("{} cannot be written: {error}", path.display())));
        }
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
    stop_earlier_instance_socket(host, manager);
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

/// Whether a unit's state says it runs no process and listens on nothing: `inactive` or `failed`.
fn is_at_rest(state: &str) -> bool {
    return matches!(state, "inactive" | "failed");
}

/// The manager's failure in words for a log line.
fn describe(error: &CallError) -> String {
    return match error {
        CallError::Connection(reason) => format!("the session bus could not be reached ({reason})"),
        CallError::Authentication(reason) => format!("the session bus refused the connection ({reason})"),
        CallError::TimedOut => String::from("the session bus did not answer in time"),
        CallError::Manager { text, name } if text.is_empty() => format!("the service manager refused the request ({name})"),
        CallError::Manager { text, .. } => format!("the service manager refused the request ({text})"),
        CallError::Malformed(reason) => format!("the service manager's answer could not be read ({reason})"),
    };
}

/// What:
///  Stops the socket unit `cli-git@<instance>.socket` that the earlier release set up for this very worktree, when no helper
///  of that unit runs. Nothing here fails the commit.
/// Why:
///   The owner's pick of 2026-11-07 ("Stop the old socket"): while that socket listens, a connection starts the earlier helper
///  at any moment, which keeps the person's earlier batches from being settled on request
///  (`earlier_service_journal_adopted.rs`). The instance name is the same text in both releases (the same hash of the same Git
///  directory), so the unit is this worktree's and no other's. The earlier release wrote only two shared templates, from which
///  every worktree's instance is made, so there is no file of this instance to remove and the templates stay; a stopped instance
///  is unloaded by the manager. A helper that runs is left as it is: the earlier service `Requires=` its socket, so a plain stop
///  of the socket stops it mid-batch, and the stop that is sent names the one unit. The socket file is never unlinked, because
///  an earlier-release wrapper can make its unit listen again at that path between any answer of the manager and an unlink.
///  Whether the stop is still to be done is read from the table of listening sockets (`in_place_with`). Every failure is logged
///  and ignored, each call is bounded by one exchange's time, and an earlier-release wrapper that sets the unit up again
///  undoes this (the owner accepted that).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await stopEarlierInstanceSocket(host, manager) // logs and returns on any failure
/// ```
fn stop_earlier_instance_socket(host: &ProvisionHost, manager: &mut dyn Manager) {
    let socket: String = earlier_socket_unit_instance(host.instance.as_str());
    let service: String = earlier_service_unit_instance(host.instance.as_str());
    // The start that came before may have used the time of its exchange; these calls have a time of their own.
    manager.within(EXCHANGE_WAIT);
    let unreadable = |unit: &str, error: &CallError| {
        crate::service_warn!(
            "the commit service of the earlier release for this worktree ({unit}) was not stopped: {}; the batches it left open stay open, and the next commit tries again",
            describe(error)
        );
    };
    // A helper that runs finishes its own batches; it is not touched.
    match manager.active_state(service.as_str()) {
        Ok(Some(state)) if !is_at_rest(state.as_str()) => return,
        Ok(_) => {}
        Err(error) => return unreadable(service.as_str(), &error),
    }
    let held: Option<String> = match manager.active_state(socket.as_str()) {
        Ok(state) => state,
        Err(error) => return unreadable(socket.as_str(), &error),
    };
    if !matches!(&held, Some(state) if !is_at_rest(state.as_str())) {
        return;
    }
    match manager.stop_alone(socket.as_str()) {
        Ok(()) => {}
        Err(CallError::Manager { name, .. }) if name == NO_SUCH_UNIT => {}
        Err(error) => unreadable(socket.as_str(), &error),
    }
}

/// Provisions through the session bus, unless the units and the socket are already in place.
pub fn provision(host: &ProvisionHost) -> Result<(), ClientFailure> {
    if in_place(host) {
        return Ok(());
    }
    let mut bus: Bus = match Bus::connect(host.bus.as_path(), EXCHANGE_WAIT) {
        Ok(connected) => connected,
        Err(_refused) => return Err(ClientFailure::NoActivation(platform_without_manager())),
    };
    return provision_with(host, &mut bus);
}

/// Provisioning controls stay out of the release executable.
#[cfg(test)]
#[path = "service_provision_tests.rs"]
mod tests;
