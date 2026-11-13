//! What:
//!  The wrapper makes socket activation for a worktree's commit service exist the first time a commit needs it:
//! it points a
//!  per-user link at the release it is running,
//! writes the two template unit files into the user's runtime unit directory,
//! asks the
//!  user manager (through the session bus) to start the worktree's socket instance,
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
//! What:
//!  The service template names a link (`service_instance.rs`,
//! `program_link`) instead of the absolute path of the
//!  release that wrote it,
//! so its text is the same in every release,
//! in every build of a release and in a build made in a
//!  worktree:
//! an upgrade,
//! a downgrade and two releases used in turn write nothing the manager has to read again.
//! Every commit that reaches the service compares the two installed templates with what this wrapper would
//!  write (two small reads) and points the link before it touches the bus,
//! so a release change costs one read and,
//!  when the program moved,
//! one rename.
//! Why:
//!  A manager told about a new text re-reads every unit it holds,
//! which stalls every other client of it for a
//!  time that grows with everything else it runs;
//! on 2026-11-12 one of those stalls ended a desktop session
//!  (`doc/troubleshooting/plasma-session-ended-after-service-manager-timeouts.md`).
//! Reading every unit again took 1 to 13 seconds on this host,
//! and a manager that answered past the
//!  exchange's bound failed the first commit of a release.
//! The one thing left to do when the manager
//!  may hold a unit at an older text is to let that unit go and start it again,
//! which touches this worktree's units
//!  alone (`service_provision_instance_unloaded.rs`).
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
//! await provision({ runtimeDir, configHome, busPath, serviceBinary, git, instance }) // points the link, writes templates, starts the socket
//! ```

pub use crate::service_provision_unit_file_ownership_and_writes::OWN_MARKERS;
use crate::service_provision_unit_file_ownership_and_writes::{link_atomically, own_unit_file_in_runtime_or_user_directory, write_atomically};
use crate::service_messages::ClientFailure;
pub use crate::service_provision_host_and_manager::{LEGACY_SERVICE_UNIT_NAME, LEGACY_SOCKET_UNIT_NAME, Manager, ProvisionHost, RETIREMENT_WAIT, STOP_TIMEOUT_SECONDS, platform_without_manager, runtime_unit_dir};
use crate::service_provision_host_and_manager::refusal;
use crate::service_provision_earlier_service_retirement::{clear_earlier_units, retire_earlier_service};
use crate::service_provision_instance_unloaded::{Unloaded, describe, is_at_rest, unload_instance};
use crate::service_provision_slow_manager_notice::SlowManagerNotice;
use crate::service_provision_refusals_before_the_wait::{
    in_plain_words, refuse_a_path_no_socket_address_can_carry, refuse_a_service_that_listens_elsewhere,
};
use crate::service_provision_loaded_unit_texts::{record as record_loaded_texts, remembers as manager_was_told};
use crate::earlier_unix_sockets_that_listen::earlier_socket_of_instance_listens;
use crate::service_instance::{LEGACY_SOCKET_RELATIVE, earlier_service_unit_instance, earlier_socket_path, earlier_socket_unit_instance, program_link, service_unit_instance, socket_path, socket_unit_instance};
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
/// The service's program is the per-user link,
/// never the absolute path of the release that happens to be running,
/// so
/// the text is the same in every release (`program_link`).
pub fn unit_texts(host: &ProvisionHost) -> [(&'static str, String); 2] {
    let program: PathBuf = program_link(host.runtime_dir.as_path());
    let service: String = SERVICE_UNIT
        .replace(BINARY_PLACEHOLDER, program.to_string_lossy().as_ref())
        .replace(GIT_PLACEHOLDER, host.git.to_string_lossy().as_ref())
        .replace(STOP_TIMEOUT_PLACEHOLDER, STOP_TIMEOUT_SECONDS.to_string().as_str());
    return [(SOCKET_UNIT_NAME, SOCKET_UNIT.to_string()), (SERVICE_UNIT_NAME, service)];
}

/// What:
///  Points the per-user link the service unit runs at `host.service_binary`,
/// unless it already names it.
/// Why:
///   The manager starts whatever the link names at the moment of the start,
/// so a commit that provisions after an upgrade makes
///  the next start run its own release,
/// and it does so by writing one link,
/// not by changing the text the manager holds.
/// Every
///  commit that reaches the service points it,
/// the fast path included,
/// because a service the manager starts later
///  (after its idle exit,
/// or after a restart) must run the release of the commit that needed it.
/// A link that cannot
///  be written never fails the commit:
/// the unit files are still written,
/// and a start then runs the release the link named
///  before,
/// which the client asks to retire in the ordinary way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await pointProgramLink(host) // the link names host.serviceBinary
/// ```
fn point_program_link(host: &ProvisionHost) -> Result<(), ClientFailure> {
    let link: PathBuf = program_link(host.runtime_dir.as_path());
    if matches!(std::fs::read_link(&link), Ok(current) if current == host.service_binary) {
        return Ok(());
    }
    return match link_atomically(link.as_path(), host.service_binary.as_path()) {
        Ok(()) => Ok(()),
        Err(error) => Err(ClientFailure::CouldNotStart(format!(
            "{} cannot be pointed at {} ({}), and the commit service is started through it, so set XDG_RUNTIME_DIR to a runtime directory this account can write or make that one writable",
            link.display(),
            host.service_binary.display(),
            in_plain_words(&error)
        ))),
    };
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
/// inPlace(host) // false: write, unload, start
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
    // What: The manager last read these very texts for this instance.
    // Why:  A file that matches says what this release would write, not what the manager loaded. A process killed between
    //       writing a template and making the manager read it leaves exactly that, and every later commit would then keep
    //       reaching the service of the release that was replaced
    //       (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`, the interrupted upgrade). One
    //       small read of the record beside the notes decides it. The record is per instance, because the manager holds one
    //       instance at a time: another worktree's freshly read units say nothing about this one's.
    let manager_holds: bool = manager_was_told(host.runtime_dir.as_path(), host.instance.as_str(), texts.as_slice());
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
///  manager's copy of a unit it may hold at an older text let go,
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
    // The manager starts whatever the link names, so it names this release before any unit can be started.
    point_program_link(host)?;
    // The earlier single service and the units of the earliest releases are stopped and removed here; neither needs the
    //  manager to re-read anything, because a unit that is gone from memory is read from its file the next time it is
    //  started.
    let _retired: bool = retire_earlier_service(host, manager)?;
    let _earlier_cleared: bool = clear_earlier_units(host)?;
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
    }
    // What: The note names the worktree before the manager is asked to start the units.
    // Why:  The service reads the note to learn whose units it runs, and a service that finds none answers `Unmapped`, never asks
    //       whether its worktree is still there, and cannot be named by the sweep that stops the units of a removed worktree. A
    //       kill between the start and the note left exactly that service behind, for up to the idle exit
    //       (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`, finding 4). The note names a directory
    //       that exists and needs no manager, so writing it first costs nothing; a refused provisioning leaves a note whose units
    //       never started, which no service reads and the next provisioning writes again.
    note_worktree(host.runtime_dir.as_path(), host.instance.as_str(), host.git_dir.as_path());
    // What: The record of the texts the manager was told about, read once: it says whether the manager holds this instance at
    //       what is on disk, which is the one question both the fast path and the repairing path turn on.
    // Why:  A file that matches says what this release would write, not what the manager loaded. A process killed between
    //       writing a template and making the manager read it leaves exactly that, and every later commit would then keep
    //       reaching the service of the release that was replaced
    //       (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`, the interrupted upgrade).
    let told: bool = manager_was_told(host.runtime_dir.as_path(), host.instance.as_str(), texts.as_slice());
    let socket: PathBuf = socket_path(host.runtime_dir.as_path(), host.instance.as_str());
    // What: A provisioning that found the units and the record as it would write them, the socket file there and no earlier
    //       socket of this worktree listening asks the manager nothing at all.
    // Why:  Everything such a call would ask is either already true or is repaired by the client's own wait for the service:
    //       a unit the manager reports `inactive` is started there, and a service of another release is asked to retire there.
    //       Asking anyway cost three manager round trips on a commit whose only change was the program link, and a manager that
    //       is busy answers those round trips only after whatever it is doing (`service_client_reach_service.rs`).
    if written.is_empty() && told && socket.exists() && !earlier_socket_of_instance_listens(Path::new("/proc"), host.instance.as_str()) {
        return Ok(());
    }
    // What: A commit that is about to wait for the manager says so while it waits.
    // Why:  Provisioning can spend the retirement wait of this worktree's own service and a whole exchange with the manager,
    //       and a wait nobody can see is indistinguishable from a hang.
    let notice: SlowManagerNotice = SlowManagerNotice::arm(socket_unit_instance(host.instance.as_str()).as_str());
    // Whatever the retirement of the earlier service took, the question about the address a held unit listens at and the
    //  start have the time of a whole exchange.
    manager.within(EXCHANGE_WAIT);
    // What: A socket unit the manager already holds that listens where this command does not look ends the provision before
    //       anything is started.
    // Why:  That unit is another runtime directory's service for this worktree: starting here would create and start a unit this
    //       command can never use, and a client whose runtime directory the manager does not use would leave one more socket
    //       listening for a worktree nothing reaches (`service_provision_refusals_before_the_wait.rs`). A manager that holds no
    //       such unit yet proves nothing, so a first provision goes on and the same question is asked once more after the start.
    refuse_a_service_that_listens_elsewhere(host, manager)?;
    // What: What this call wrote, or a record that is missing, may leave the manager holding this worktree's units at an older
    //       text: those units are let go, so that the next start reads the files as they are now.
    // Why:  A unit is only re-read when the manager does not hold it, and this is the one operation that makes the manager drop
    //       this worktree's units alone, at a cost that is this worktree's own
    //       (`service_provision_instance_unloaded.rs`). Nothing on this path makes the manager read every unit it holds, which
    //       no Git command may ask for (`doc/troubleshooting/plasma-session-ended-after-service-manager-timeouts.md`). The
    //       upgrade a kill interrupted had already written the template that now matches, so the loop above finds nothing to
    //       write and the manager still holds the text it read before; the record of what it read is what tells the two apart
    //       (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`, the interrupted upgrade).
    let stale: bool = !written.is_empty() || !told;
    // What: Whether the manager may be taken to hold these very texts once this call is done with it.
    // Why:  The record is rewritten only by a call that proved it: the record said so already, or the manager was watched
    //       dropping this worktree's units, so its next start reads the files as they are now. A unit is re-read only when the
    //       manager does not hold it, so anything a manager that could not be asked leaves behind is a text the record must
    //       not claim it read.
    let mut holds_these_texts: bool = told;
    if stale {
        let unloaded: Unloaded = unload_instance(host, manager)?;
        holds_these_texts = unloaded == Unloaded::Dropped;
    }
    manager.within(EXCHANGE_WAIT);
    let asked: Result<(), CallError> = ask_manager(manager, known, host.instance.as_str());
    match asked {
        Ok(()) => {}
        // What: A manager that did not answer within the exchange's time leaves the commit to go on and the next commit to ask
        //       again, and never fails this one.
        // Why:  The manager is busy or is not there at all, and neither is about the commit's own change: the units are
        //       written, a service of this release may already be running and answer the request, and the record of what the
        //       manager was told stays unwritten, which is what makes the next commit unload and start again. A commit that
        //       ended here told the person this account has no user services, which was false, and left the change uncommitted
        //       (two commits of 2026-11-11T20:11-05:00, twenty seconds of provisioning each).
        Err(CallError::TimedOut) => {
            crate::service_warn!(
                "the service manager did not answer within {} seconds while the units of this worktree's commit service were being set up ({}); the units are written and the commit goes on, and the next commit asks the manager again",
                EXCHANGE_WAIT.as_secs(),
                socket_unit_instance(host.instance.as_str())
            );
            drop(notice);
            return Ok(());
        }
        Err(error) => {
            // What: A start the manager refused leaves the files and no record of a manager that read them.
            // Why:  Only the record says the manager holds these texts, so the next commit unloads and starts again instead of
            //       taking a file the manager never read for an installed unit. Withdrawing the file is not needed, and
            //       withdrawing it would leave a machine whose manager refuses every start with no unit file at all.
            drop(notice);
            return Err(refusal(error));
        }
    }
    // What: A service the manager made listen somewhere this command does not look ends the provision here.
    // Why:  The unit's address is built from the manager's own runtime directory, so a command given another one waits out
    //       the whole start ceiling for a service that is running, healthy and out of reach
    //       (`service_provision_refusals_before_the_wait.rs`).
    refuse_a_service_that_listens_elsewhere(host, manager)?;
    // What: The manager read these texts for this instance, whether this call unloaded it or found the record that says so.
    // Why:  A start that loaded the socket from a text the record does not hold is repaired by the next commit instead of being
    //       taken as installed. A start on its own is no proof of what the manager read: a manager that could not say what it
    //       holds may have started the unit from a text an upgrade left behind, so the record stays unwritten and the next
    //       commit asks again.
    if holds_these_texts
        && let Err(error) = record_loaded_texts(host.runtime_dir.as_path(), host.instance.as_str(), texts.as_slice())
    {
        crate::service_warn!(
            "the unit texts this program wrote cannot be recorded as the ones the service manager reads ({error}), so the next commit unloads this worktree's units again and starts them, and no commit is lost"
        );
    }
    stop_earlier_instance_socket(host, manager);
    drop(notice);
    return Ok(());
}

/// The calls provisioning makes:
/// a reset of failed state when a unit may have failed under the manager (the reset is best effort:
/// a unit the manager does not hold has nothing to reset),
/// and the start.
fn ask_manager(manager: &mut dyn Manager, known: bool, instance: &str) -> Result<(), CallError> {
    if known {
        // A start that failed under an old unit leaves the service failed and its start counter full, and the socket
        //  failed with it; the new unit would be refused as "repeated too quickly" until the interval passed.
        for unit in [service_unit_instance(instance), socket_unit_instance(instance)] {
            let _reset = manager.reset_failed(unit.as_str());
        }
    }
    return manager.start(socket_unit_instance(instance).as_str());
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
    // What: An address no socket can carry ends the provision before the fast path reads anything.
    // Why:  An installation that looks current can stand behind a spelling the file system still walks and no connect can
    //       carry, because a socket name is copied into the address as written; the fast path would then return as
    //       provisioned and the commit would spend the start ceiling connecting to an address the kernel refuses, and end
    //       with "The commit service did not start" and nothing about the setting that made it
    //       (`service_provision_refusals_before_the_wait.rs`).
    refuse_a_path_no_socket_address_can_carry(host)?;
    // What: The link the service unit runs names this release before the fast path returns.
    // Why:  The manager starts whatever the link names at the moment a start is needed, which after this commit may be hours
    //       away (the service exits after three idle hours). A commit whose fast path skipped this would let the next start
    //       run another release, and the whole reason the unit text no longer names a release is that this link does.
    point_program_link(host)?;
    if in_place(host) {
        // Units set up before this release have no note of their worktree yet.
        note_worktree(host.runtime_dir.as_path(), host.instance.as_str(), host.git_dir.as_path());
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
