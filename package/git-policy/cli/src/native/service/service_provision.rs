//! What:
//!  The wrapper makes socket activation for a worktree's commit service exist the first time a commit needs it:
//! it writes
//!  the two template unit files into the user's runtime unit directory,
//! asks the user manager (through the session bus) to
//!  reload and to start the worktree's socket instance,
//! and returns.
//! The manager,
//! not the wrapper,
//! starts the service when
//!  the first connection arrives.
//! Why:
//!  There is no install step to remember (`doc/planning/cli-git-resident-service.md`,
//! "Provisioning"):
//! like Bazel's
//!  client,
//! the wrapper sets up what it needs and tells nobody.
//! One service runs per worktree,
//! so the units are templates
//!  (`cli-git@.socket`,
//! `cli-git@.service`) and each worktree's instance is named by a hash of its own Git directory
//!  (`service_instance.rs`).
//! The units live under `$XDG_RUNTIME_DIR/systemd/user`,
//! which disappears at logout,
//! so nothing is
//!  left behind in the home directory and the next first use provisions again.
//! Every step is idempotent (a file is written,
//!  by temporary file and rename,
//! only when its content differs),
//! so several commits racing to provision agree without any
//!  lock.
//! Every commit that goes to the service compares the two installed templates with what this wrapper would write
//!  (two small reads) before it touches the bus,
//! so an upgrade or a downgrade,
//! which moves the executable the template
//!  names,
//! is repaired by the first commit that follows it:
//! templates rewritten,
//! manager reloaded,
//! failed state cleared,
//!  the worktree's socket started.
//! The single service of the earlier releases (`cli-git.socket` and `cli-git.service`,
//! one
//!  socket for every worktree) is asked to retire and stopped,
//! and its units and socket file are removed,
//! by the first
//!  commit of the new release;
//! its open batches are finished by the instance of the worktree they belong to,
//! and the work
//!  it had claimed is claimed again by the next drain.
//! Units this wrapper's earliest releases wrote into
//!  `~/.config/systemd/user` outrank the runtime ones in the unit search path,
//! so they are recognized by their content and
//!  removed;
//! a unit of the same name that something else wrote is never touched,
//! and the commit stops with a message naming
//!  the file.
//!  The socket that the earlier release set up for the same worktree (`cli-git@<instance>.socket`,
//! the same instance text) is
//!  stopped after this release's socket has started,
//! so that the earlier helper cannot be started by a connection and the
//!  batches it left open can be finished (`earlier_service_journal_adopted.rs`);
//! see `stop_earlier_instance_socket`.
//! The stop
//!  never fails a commit.
//! A helper of the earlier release that is running is not stopped and its socket stays;
//! the next commit
//!  looks again.
//! The templates the earlier release wrote are shared by every worktree's instance and are left alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await provision({ runtimeDir, configHome, busPath, serviceBinary, git, instance }) // writes templates, reloads, starts the socket
//! ```

pub use crate::service_provision_unit_file_ownership_and_writes::OWN_MARKERS;
use crate::service_provision_unit_file_ownership_and_writes::{own_unit_file_in_runtime_or_user_directory, write_atomically};
use crate::service_messages::ClientFailure;
pub use crate::service_provision_host_and_manager::{LEGACY_SERVICE_UNIT_NAME, LEGACY_SOCKET_UNIT_NAME, Manager, ProvisionHost, RETIREMENT_WAIT, STOP_TIMEOUT_SECONDS, platform_without_manager, runtime_unit_dir};
use crate::service_provision_host_and_manager::refusal;
use crate::service_provision_earlier_service_retirement::{clear_earlier_units, retire_earlier_service};
use crate::service_provision_refusals_before_the_wait::{
    in_plain_words, refuse_a_path_no_socket_address_can_carry, refuse_a_service_that_listens_elsewhere,
};
use crate::service_provision_loaded_unit_texts::{record as record_loaded_texts, remembers as manager_was_told};
use crate::earlier_unix_sockets_that_listen::earlier_socket_of_instance_listens;
use crate::service_instance::{LEGACY_SOCKET_RELATIVE, earlier_service_unit_instance, earlier_socket_path, earlier_socket_unit_instance, service_unit_instance, socket_path, socket_unit_instance};
use crate::service_units_of_removed_worktrees::note_worktree;
use crate::systemd_dbus::{Bus, CallError, EXCHANGE_WAIT, NO_SUCH_UNIT};
use std::path::{Path, PathBuf};

/// The socket template's text,
/// with the marker on its first line.
pub const SOCKET_UNIT: &str = include_str!("service_units/monochromatic-git-policy-cli@.socket");

/// The service template's text,
/// with `@SERVICE_BINARY@` and `@GIT@` still to fill in.
pub const SERVICE_UNIT: &str = include_str!("service_units/monochromatic-git-policy-cli@.service");

/// File name of the socket template.
pub const SOCKET_UNIT_NAME: &str = "monochromatic-git-policy-cli@.socket";

/// File name of the service template.
pub const SERVICE_UNIT_NAME: &str = "monochromatic-git-policy-cli@.service";

const BINARY_PLACEHOLDER: &str = "@SERVICE_BINARY@";
const GIT_PLACEHOLDER: &str = "@GIT@";
const STOP_TIMEOUT_PLACEHOLDER: &str = "@STOP_TIMEOUT_SECONDS@";

/// The two unit files for a host,
/// as written.
pub fn unit_texts(host: &ProvisionHost) -> [(&'static str, String); 2] {
    let service: String = SERVICE_UNIT
        .replace(BINARY_PLACEHOLDER, host.service_binary.to_string_lossy().as_ref())
        .replace(GIT_PLACEHOLDER, host.git.to_string_lossy().as_ref())
        .replace(STOP_TIMEOUT_PLACEHOLDER, STOP_TIMEOUT_SECONDS.to_string().as_str());
    return [(SOCKET_UNIT_NAME, SOCKET_UNIT.to_string()), (SERVICE_UNIT_NAME, service)];
}

/// What:
///  Whether the installed templates are the ones this wrapper would write,
/// the worktree's socket is there,
/// and nothing of
///  the single service of the earlier releases is left,
/// and no socket of the earlier release for this worktree listens.
/// Why:
///   The socket belongs to the manager and outlives every upgrade,
/// so its presence says nothing about which program the
///  service template names;
/// the text does.
/// A few small reads and two `stat` calls decide,
/// and the bus is not touched when
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
    let texts: [(&'static str, String); 2] = unit_texts(host);
    // Every unit file holds exactly the text the wrapper would write.
    let mut all_current: bool = true;
    for (name, text) in &texts {
        if !matches!(std::fs::read_to_string(directory.join(name)), Ok(installed) if installed == *text) {
            all_current = false;
            break;
        }
    }
    // What: The manager was told about these very texts.
    // Why:  A file that matches says what this release would write, not what the manager loaded. A process killed between
    //       writing a template and asking for the reload leaves exactly that, and every later commit would then keep reaching
    //       the service of the release that was replaced (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
    //       the interrupted upgrade). One small read of the record beside the notes decides it.
    let manager_holds: bool = manager_was_told(host.runtime_dir.as_path(), texts.as_slice());
    // A file of an earlier name that something else wrote is not the wrapper's to retire, and does not keep it from being in place.
    let mut earlier_left: bool = false;
    for name in [LEGACY_SOCKET_UNIT_NAME, LEGACY_SERVICE_UNIT_NAME] {
        if own_unit_file_in_runtime_or_user_directory(directory.as_path(), host.user_unit_dir.as_deref(), name) {
            earlier_left = true;
            break;
        }
    }
    earlier_left = earlier_left || host.runtime_dir.join(LEGACY_SOCKET_RELATIVE).exists();
    if !all_current || !manager_holds || earlier_left || !socket_path(host.runtime_dir.as_path(), host.instance.as_str()).exists() {
        return false;
    }
    // The earlier release's socket of this worktree: a listener there starts the earlier helper on a connection, so the stop
    //  has to be tried (again). The socket's file stays after the unit stops and says nothing; the table of sockets is read
    //  only when that file is there, which a worktree the earlier release never served never has.
    let earlier_file: PathBuf = earlier_socket_path(host.runtime_dir.as_path(), host.instance.as_str());
    return !(earlier_file.exists() && earlier_socket_of_instance_listens(proc_root, host.instance.as_str()));
}

/// What:
///  Makes socket activation exist:
/// the earlier single service retired,
/// current templates written where they differ,
/// the
///  manager reloaded when a unit it may hold changed or went away,
/// and the worktree's socket instance started.
/// Why:
///   Idempotent by construction:
/// a second call with nothing to change only starts the socket,
/// which is a no-op when it is
///  already listening;
/// two calls at once write identical content through renames and both succeed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await provisionWith(host, manager)
/// ```
pub fn provision_with(host: &ProvisionHost, manager: &mut dyn Manager) -> Result<(), ClientFailure> {
    // What: An address no socket can carry ends the provision before anything is written or asked for.
    // Why:  The manager would bind the unit's address and fail, and the client would then wait out the start ceiling for a
    //       service that can never answer (`service_provision_refusals_before_the_wait.rs`).
    refuse_a_path_no_socket_address_can_carry(host)?;
    // A reload makes the manager re-read every unit and interrupts its other clients for a while, so it is asked for only
    //  when a unit it may already hold changed or went away. A file that is new is found by the manager when the unit is
    //  first started.
    let mut reload: bool = retire_earlier_service(host, manager)?;
    reload |= clear_earlier_units(host)?;
    let directory: PathBuf = runtime_unit_dir(host.runtime_dir.as_path());
    if let Err(error) = std::fs::create_dir_all(&directory) {
        return Err(ClientFailure::CouldNotStart(format!(
            "{} cannot be created ({}), and the commit service keeps its settings there, so set XDG_RUNTIME_DIR to a runtime \
             directory this account can write or make that one writable",
            directory.display(),
            in_plain_words(&error)
        )));
    }
    let texts: [(&'static str, String); 2] = unit_texts(host);
    // Whether the manager may hold state for these units: a unit file was already there, so a start may have failed under it.
    let mut known: bool = false;
    let mut written: Vec<PathBuf> = Vec::new();
    for (name, text) in &texts {
        let path: PathBuf = directory.join(name);
        let existed: bool = path.exists();
        known |= existed;
        if matches!(std::fs::read_to_string(&path), Ok(current) if current == *text) {
            continue;
        }
        if let Err(error) = write_atomically(&path, text.as_str()) {
            return Err(ClientFailure::CouldNotStart(format!(
                "{} cannot be written ({}), and the commit service keeps its settings there, so set XDG_RUNTIME_DIR to a runtime \
                 directory this account can write or make that one writable",
                path.display(),
                in_plain_words(&error)
            )));
        }
        written.push(path);
        reload |= existed;
    }
    // What: A unit file the manager may hold and was never told about is reloaded for, even though this call wrote nothing.
    // Why:  The upgrade a kill interrupted had already written the template that now matches, so the loop above finds nothing
    //       to write and would leave the manager holding the text it loaded before; the record of what it was told is what
    //       tells the two apart (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`, the interrupted
    //       upgrade). The next commit asks for the reload the killed one never made, and the commits after it find the record
    //       and take the fast path again.
    reload |= known && !manager_was_told(host.runtime_dir.as_path(), texts.as_slice());
    // What: The note names the worktree before the manager is asked to start the units.
    // Why:  The service reads the note to learn whose units it runs, and a service that finds none answers `Unmapped`, never asks
    //       whether its worktree is still there, and cannot be named by the sweep that stops the units of a removed worktree. A
    //       kill between the start and the note left exactly that service behind, for up to the idle exit
    //       (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`, finding 4). The note names a directory
    //       that exists and needs no manager, so writing it first costs nothing; a refused provisioning leaves a note whose units
    //       never started, which no service reads and the next provisioning writes again.
    note_worktree(host.runtime_dir.as_path(), host.instance.as_str(), host.git_dir.as_path());
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
    // What: A service the manager made listen somewhere this command does not look ends the provision here.
    // Why:  The unit's address is built from the manager's own runtime directory, so a command given another one waits out
    //       the whole start ceiling for a service that is running, healthy and out of reach
    //       (`service_provision_refusals_before_the_wait.rs`).
    refuse_a_service_that_listens_elsewhere(host, manager)?;
    // What: The manager now holds exactly these texts.
    // Why:  The record is written after the manager accepted the reload and never before, so it is the manager's own answer and
    //       the next commit can take the fast path. A record that cannot be written costs one reload per commit and never a
    //       commit.
    if let Err(error) = record_loaded_texts(host.runtime_dir.as_path(), texts.as_slice()) {
        crate::service_warn!("the units this program wrote cannot be recorded as the ones the service manager was told about ({error}), so the next commit provisions again and asks for a reload it does not need");
    }
    stop_earlier_instance_socket(host, manager);
    return Ok(());
}

/// The calls provisioning makes:
/// a reload when a held unit changed,
/// a reset of failed state when a unit may have failed
///  under the manager (the reset is best effort:
/// a unit the manager does not hold has nothing to reset),
/// and the start.
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

/// Whether a unit's state says it runs no process and listens on nothing:
/// `inactive` or `failed`.
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

/// Say in the service's log that the earlier release's unit for this worktree was not stopped,
/// and why.
fn warn_not_stopped(unit: &str, error: &CallError) {
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
fn stop_earlier_instance_socket(host: &ProvisionHost, manager: &mut dyn Manager) {
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
        Err(CallError::Manager { name, .. }) if name == NO_SUCH_UNIT => {}
        Err(error) => warn_not_stopped(socket.as_str(), &error),
    }
}

/// Provisions through the session bus,
/// unless the units and the socket are already in place.
pub fn provision(host: &ProvisionHost) -> Result<(), ClientFailure> {
    if in_place(host) {
        // Units set up before this release have no note of their worktree yet.
        note_worktree(host.runtime_dir.as_path(), host.instance.as_str(), host.git_dir.as_path());
        return Ok(());
    }
    // What: An address no socket can carry is refused before the bus is even reached for.
    // Why:  The manager is the one that would fail to bind it, and a bus that cannot be reached would otherwise answer
    //       first with a remedy about user services, which says nothing about the runtime directory that is too long.
    refuse_a_path_no_socket_address_can_carry(host)?;
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

/// The owner's rule that nothing of the wrapper starts at system startup or at login,
/// held by guards that fail when a unit,
/// a link,
/// a
///  file outside the runtime directory or a call to the manager would make it so.
#[cfg(test)]
#[path = "service_provision_never_starts_at_login_tests.rs"]
mod never_starts_at_login_tests;

/// What:
///  Provision the service the first time it is needed,
/// and say whether it is now provisioned.
/// Why:
///   A commit the service declined,
/// blocked by a policy,
/// or abandoned in the editor never needs one,
/// and the manager starts the service;
/// so this runs once,
/// at the first submission,
/// and the time it takes is the timing record's `provisionMs`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// provisioned = provisionWhenNeeded(host, provisioned, timing);
/// ```
pub(crate) fn provision_when_needed(named: Option<&ProvisionHost>, provisioned: bool, timing: &mut std::time::Duration) -> Result<bool, crate::commit_transaction::TransactionStop> {
    let Some(host) = named else {
        return Ok(provisioned);
    };
    if provisioned {
        return Ok(true);
    }
    let began: std::time::Instant = std::time::Instant::now();
    let provided = provision(host);
    *timing += began.elapsed();
    if let Err(failure) = provided {
        return Err(crate::commit_transaction::TransactionStop::Message(crate::service_messages::render(&failure)));
    }
    return Ok(true);
}
